//! Kiro account groups page (`/admin/kiro-gateway/groups`).
//!
//! Server-paginated group cards with a collapsed create form; the member
//! picker (shared by create and edit) needs the full account inventory, which
//! is why this page is the only one that loads it.

use std::collections::HashSet;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    api::{
        create_admin_kiro_account_group, delete_admin_kiro_account_group,
        fetch_admin_kiro_account_groups_page, fetch_admin_kiro_accounts,
        patch_admin_kiro_account_group, AdminAccountGroupView, CreateAdminAccountGroupInput,
        KiroAccountView, PatchAdminAccountGroupInput,
    },
    components::{pagination::Pagination, search_box::SearchBox},
    pages::llm_access_shared::{confirm_destructive, format_float2},
    router::Route,
};

const GROUP_PAGE_SIZE: usize = 24;

fn group_total_pages(total: usize, page_size: usize) -> usize {
    total.max(1).div_ceil(page_size.max(1))
}

/// Case-insensitive match over group name / id / provider / member accounts.
fn group_matches_query(group: &AdminAccountGroupView, query_lower: &str) -> bool {
    group.name.to_lowercase().contains(query_lower)
        || group.id.to_lowercase().contains(query_lower)
        || group.provider_type.to_lowercase().contains(query_lower)
        || group
            .account_names
            .iter()
            .any(|name| name.to_lowercase().contains(query_lower))
}

/// Checkbox grid used by both the create form and the group editor.
fn member_picker(
    accounts: &[KiroAccountView],
    selected: &[String],
    on_toggle: &Callback<String>,
) -> Html {
    html! {
        <div class={classes!("grid", "gap-2", "xl:grid-cols-2")}>
            { for accounts.iter().map(|account| {
                let checked = selected.iter().any(|name| name == &account.name);
                let account_name = account.name.clone();
                let on_toggle = on_toggle.clone();
                let balance_hint = account
                    .balance
                    .as_ref()
                    .map(|balance| format!(
                        "remaining {} / {}",
                        format_float2(balance.remaining),
                        format_float2(balance.usage_limit)
                    ))
                    .unwrap_or_else(|| "balance loading".to_string());
                html! {
                    <label class={classes!(
                        "flex", "cursor-pointer", "items-center", "gap-3", "rounded-[var(--r-field)]", "border", "px-3", "py-2.5",
                        if checked {
                            "border-[var(--info)] bg-[var(--info-soft)]"
                        } else {
                            "border-[var(--border)] bg-[var(--card-2)]"
                        }
                    )}>
                        <input
                            type="checkbox"
                            class={classes!("min-h-0", "w-auto")}
                            checked={checked}
                            onchange={Callback::from(move |_| on_toggle.emit(account_name.clone()))}
                        />
                        <div class={classes!("min-w-0", "flex-1")}>
                            <div class={classes!("font-semibold")}>{ account.name.clone() }</div>
                            <div class={classes!("mono", "mt-1", "text-[11px]", "text-[var(--muted-foreground)]")}>
                                { balance_hint }
                            </div>
                        </div>
                    </label>
                }
            }) }
        </div>
    }
}

/// Sorted, deduplicated toggle of `account_name` within `names`.
fn toggled_member_names(names: &[String], account_name: String) -> Vec<String> {
    let mut names = names.to_vec();
    if let Some(index) = names.iter().position(|name| name == &account_name) {
        names.remove(index);
    } else {
        names.push(account_name);
        names.sort();
        names.dedup();
    }
    names
}

#[derive(Properties, PartialEq)]
struct GroupEditorCardProps {
    group_item: AdminAccountGroupView,
    accounts: Vec<KiroAccountView>,
    on_reload: Callback<()>,
    on_flash: Callback<(String, bool)>,
}

#[function_component(GroupEditorCard)]
fn group_editor_card(props: &GroupEditorCardProps) -> Html {
    let name = use_state(|| props.group_item.name.clone());
    let expanded = use_state(|| false);
    let account_names = use_state(Vec::<String>::new);
    let saving = use_state(|| false);
    let feedback = use_state(|| None::<String>);

    {
        let group_item = props.group_item.clone();
        let accounts = props.accounts.clone();
        let name = name.clone();
        let account_names = account_names.clone();
        use_effect_with((props.group_item.clone(), props.accounts.clone()), move |_| {
            let valid_names = accounts
                .iter()
                .map(|account| account.name.as_str())
                .collect::<HashSet<_>>();
            let mut names = group_item
                .account_names
                .iter()
                .filter(|member| valid_names.contains(member.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            names.sort();
            names.dedup();
            name.set(group_item.name.clone());
            account_names.set(names);
            || ()
        });
    }

    let on_toggle_account = {
        let account_names = account_names.clone();
        Callback::from(move |account_name: String| {
            account_names.set(toggled_member_names(&account_names, account_name));
        })
    };

    let on_save = {
        let group_id = props.group_item.id.clone();
        let name = name.clone();
        let account_names = account_names.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            if *saving {
                return;
            }
            let group_id = group_id.clone();
            let name_value = (*name).trim().to_string();
            let account_names_value = (*account_names).clone();
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_flash = on_flash.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if name_value.is_empty() {
                    let message = "组名不能为空".to_string();
                    feedback.set(Some(message.clone()));
                    on_flash.emit((message, true));
                    return;
                }
                if account_names_value.is_empty() {
                    let message = "账号组至少需要选择一个账号".to_string();
                    feedback.set(Some(message.clone()));
                    on_flash.emit((message, true));
                    return;
                }
                saving.set(true);
                match patch_admin_kiro_account_group(&group_id, PatchAdminAccountGroupInput {
                    name: Some(&name_value),
                    account_names: Some(account_names_value.as_slice()),
                })
                .await
                {
                    Ok(_) => {
                        feedback.set(Some("Saved.".to_string()));
                        on_flash.emit((format!("已保存 Kiro 账号组 `{name_value}`"), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("保存 Kiro 账号组失败\n{err}"), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_delete = {
        let group_id = props.group_item.id.clone();
        let group_name = props.group_item.name.clone();
        let saving = saving.clone();
        let on_flash = props.on_flash.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            if !confirm_destructive("确认删除这个 Kiro 账号组？") {
                return;
            }
            let group_id = group_id.clone();
            let group_name = group_name.clone();
            let saving = saving.clone();
            let on_flash = on_flash.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                match delete_admin_kiro_account_group(&group_id).await {
                    Ok(_) => {
                        on_flash.emit((format!("已删除 Kiro 账号组 `{group_name}`"), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        on_flash.emit((format!("删除 Kiro 账号组失败\n{err}"), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    html! {
        <article class={classes!("panel")}>
            <div class={classes!("panel-head")}>
                <div class={classes!("min-w-0")}>
                    <h3>{ props.group_item.name.clone() }</h3>
                    <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted-foreground)]", "break-words")}>
                        {
                            if props.group_item.account_names.is_empty() {
                                "没有成员账号".to_string()
                            } else {
                                format!("成员: {}", props.group_item.account_names.join(", "))
                            }
                        }
                    </p>
                </div>
                <div class={classes!("flex", "items-center", "gap-2", "shrink-0")}>
                    <span class={classes!("badge")}>{ format!("{} 个账号", props.group_item.account_names.len()) }</span>
                    <button
                        type="button"
                        class={classes!("ghost")}
                        onclick={{
                            let expanded = expanded.clone();
                            Callback::from(move |_| expanded.set(!*expanded))
                        }}
                    >
                        { if *expanded { "收起" } else { "编辑" } }
                    </button>
                    <button class={classes!("danger")} onclick={on_delete} disabled={*saving}>
                        { "删除" }
                    </button>
                </div>
            </div>

            if *expanded {
                <div class={classes!("panel-body", "space-y-3")}>
                    <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                        { "组名" }
                        <input
                            type="text"
                            value={(*name).clone()}
                            oninput={{
                                let name = name.clone();
                                Callback::from(move |event: InputEvent| {
                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                        name.set(target.value());
                                    }
                                })
                            }}
                        />
                    </label>

                    <div class={classes!("space-y-2")}>
                        <div class={classes!("text-xs", "text-[var(--muted-foreground)]")}>{ "成员账号" }</div>
                        { member_picker(&props.accounts, &account_names, &on_toggle_account) }
                    </div>

                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <span class={classes!("text-xs", "text-[var(--faint)]")}>
                            { format!("当前成员: {}", if account_names.is_empty() { "无".to_string() } else { account_names.join(", ") }) }
                        </span>
                        <button class={classes!("primary")} onclick={on_save} disabled={*saving}>
                            { if *saving { "保存中..." } else { "保存账号组" } }
                        </button>
                    </div>

                    if let Some(feedback) = (*feedback).clone() {
                        <div class={classes!("text-xs", "text-[var(--muted-foreground)]")}>{ feedback }</div>
                    }
                </div>
            }
        </article>
    }
}

#[function_component(AdminKiroGatewayGroupsPage)]
pub fn admin_kiro_gateway_groups_page() -> Html {
    let accounts = use_state(Vec::<KiroAccountView>::new);
    let groups = use_state(Vec::<AdminAccountGroupView>::new);
    let groups_total = use_state(|| 0usize);
    let page = use_state(|| 1usize);
    let page_limit = use_state(|| GROUP_PAGE_SIZE);
    let search = use_state(String::new);
    let loading = use_state(|| true);
    let flash = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0u32);

    let create_form_expanded = use_state(|| false);
    let create_name = use_state(String::new);
    let create_account_names = use_state(Vec::<String>::new);
    let creating = use_state(|| false);

    let notify = {
        let flash = flash.clone();
        let error = error.clone();
        Callback::from(move |(message, is_error): (String, bool)| {
            if is_error {
                error.set(Some(message));
                flash.set(None);
            } else {
                flash.set(Some(message));
                error.set(None);
            }
        })
    };

    let on_reload = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_| refresh_tick.set(refresh_tick.wrapping_add(1)))
    };

    // Member candidates: the full account inventory (with balances) loads once
    // per refresh; group paging does not re-fetch it.
    {
        let accounts = accounts.clone();
        let error = error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let accounts = accounts.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_kiro_accounts().await {
                    Ok(response) => accounts.set(response.accounts),
                    Err(err) => error.set(Some(err)),
                }
            });
            || ()
        });
    }

    {
        let groups = groups.clone();
        let groups_total = groups_total.clone();
        let page = page.clone();
        let page_limit = page_limit.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with((*page, *refresh_tick), move |(requested_page, _)| {
            let groups = groups.clone();
            let groups_total = groups_total.clone();
            let page = page.clone();
            let page_limit = page_limit.clone();
            let loading = loading.clone();
            let error = error.clone();
            let requested_page = (*requested_page).max(1);
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let limit = (*page_limit).max(1);
                let offset = requested_page.saturating_sub(1).saturating_mul(limit);
                match fetch_admin_kiro_account_groups_page(limit, offset).await {
                    Ok(response) => {
                        let effective_limit = response.limit.max(1);
                        let total_pages = group_total_pages(response.total, effective_limit);
                        groups_total.set(response.total);
                        page_limit.set(effective_limit);
                        if requested_page > total_pages {
                            page.set(total_pages);
                        } else {
                            groups.set(response.groups);
                        }
                        error.set(None);
                    },
                    Err(err) => error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_toggle_create_member = {
        let create_account_names = create_account_names.clone();
        Callback::from(move |account_name: String| {
            create_account_names.set(toggled_member_names(&create_account_names, account_name));
        })
    };

    let on_create_group = {
        let create_name = create_name.clone();
        let create_account_names = create_account_names.clone();
        let creating = creating.clone();
        let notify = notify.clone();
        let on_reload = on_reload.clone();
        Callback::from(move |_| {
            if *creating {
                return;
            }
            let group_name = (*create_name).trim().to_string();
            let account_names = (*create_account_names).clone();
            let create_name = create_name.clone();
            let create_account_names = create_account_names.clone();
            let creating = creating.clone();
            let notify = notify.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if group_name.is_empty() {
                    notify.emit(("账号组名称不能为空".to_string(), true));
                    return;
                }
                if account_names.is_empty() {
                    notify.emit(("账号组至少需要选择一个账号".to_string(), true));
                    return;
                }
                creating.set(true);
                match create_admin_kiro_account_group(CreateAdminAccountGroupInput {
                    name: &group_name,
                    account_names: account_names.as_slice(),
                })
                .await
                {
                    Ok(_) => {
                        create_name.set(String::new());
                        create_account_names.set(Vec::new());
                        notify.emit((format!("已创建 Kiro 账号组 `{group_name}`"), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        notify.emit((format!("创建 Kiro 账号组失败\n{err}"), true));
                    },
                }
                creating.set(false);
            });
        })
    };

    let on_search_change = {
        let search = search.clone();
        Callback::from(move |value: String| search.set(value))
    };

    let query_lower = (*search).trim().to_lowercase();
    let filtered_groups: Vec<AdminAccountGroupView> = if query_lower.is_empty() {
        (*groups).clone()
    } else {
        (*groups)
            .iter()
            .filter(|group| group_matches_query(group, &query_lower))
            .cloned()
            .collect()
    };
    let total_pages = group_total_pages(*groups_total, *page_limit);
    let current_page = (*page).clamp(1, total_pages);
    let on_page_change = {
        let page = page.clone();
        Callback::from(move |next: usize| page.set(next))
    };

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "Kiro Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Account Groups" }</h1>
                        <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted-foreground)]")}>
                            { "先维护账号组，再让 key 选择组。固定路由请选择单账号组；自动路由可以选任意组，留空则继续使用全账号池。" }
                        </p>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminKiroGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                        <Link<Route> to={Route::AdminKiroGatewayKeys} classes={classes!("linkbtn")}>{ "Keys" }</Link<Route>>
                        <button type="button" class={classes!("primary")} disabled={*loading} onclick={{
                            let on_reload = on_reload.clone();
                            Callback::from(move |_| on_reload.emit(()))
                        }}>
                            { if *loading { "Loading..." } else { "Refresh" } }
                        </button>
                    </div>
                </header>

                if let Some(message) = (*flash).clone() {
                    <div class={classes!("okline", "text-sm")}>{ message }</div>
                }
                if let Some(err) = (*error).clone() {
                    <div class={classes!("errorline", "text-sm")}>{ err }</div>
                }

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <div>
                            <h2>{ "Create Kiro Account Group" }</h2>
                            <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "默认收起，需要时再展开。" }
                            </p>
                        </div>
                        <button
                            type="button"
                            class={classes!("ghost")}
                            onclick={{
                                let create_form_expanded = create_form_expanded.clone();
                                Callback::from(move |_| create_form_expanded.set(!*create_form_expanded))
                            }}
                        >
                            { if *create_form_expanded { "收起" } else { "展开" } }
                        </button>
                    </div>
                    if *create_form_expanded {
                        <div class={classes!("panel-body", "space-y-3")}>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]", "max-w-md")}>
                                { "Group Name" }
                                <input
                                    value={(*create_name).clone()}
                                    oninput={{
                                        let create_name = create_name.clone();
                                        Callback::from(move |event: InputEvent| {
                                            let input: HtmlInputElement = event.target_unchecked_into();
                                            create_name.set(input.value());
                                        })
                                    }}
                                />
                            </label>
                            <div class={classes!("space-y-2")}>
                                <div class={classes!("text-xs", "text-[var(--muted-foreground)]")}>{ "成员账号" }</div>
                                if (*accounts).is_empty() {
                                    <div class={classes!("empty")}>
                                        <span>{ "当前没有可加入账号组的 Kiro 账号" }</span>
                                    </div>
                                } else {
                                    { member_picker(&accounts, &create_account_names, &on_toggle_create_member) }
                                }
                            </div>
                            <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                <span class={classes!("text-xs", "text-[var(--faint)]")}>
                                    { format!(
                                        "当前成员: {}",
                                        if create_account_names.is_empty() {
                                            "无".to_string()
                                        } else {
                                            create_account_names.join(", ")
                                        }
                                    ) }
                                </span>
                                <button type="button" class={classes!("primary")} onclick={on_create_group} disabled={*creating}>
                                    { if *creating { "Creating..." } else { "Create Group" } }
                                </button>
                            </div>
                        </div>
                    }
                </section>

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <h2>{ "Groups" }</h2>
                        <div class={classes!("flex", "items-center", "gap-3", "min-w-0", "flex-1", "justify-end")}>
                            if !query_lower.is_empty() {
                                <span class={classes!("badge")}>
                                    { format!("匹配 {}/{}", filtered_groups.len(), groups.len()) }
                                </span>
                            }
                            <div class={classes!("w-full", "max-w-md")}>
                                <SearchBox
                                    value={(*search).clone()}
                                    on_change={on_search_change}
                                    placeholder={AttrValue::Static("搜索账号组名 / id / 成员账号")}
                                />
                            </div>
                        </div>
                    </div>
                    if *loading && (*groups).is_empty() {
                        <div class={classes!("skeleton", "px-4", "py-4")}>
                            <i></i><i></i><i></i><i></i>
                        </div>
                    } else if (*groups).is_empty() {
                        <div class={classes!("empty")}>
                            <span>{ "当前还没有 Kiro 账号组" }</span>
                        </div>
                    } else if filtered_groups.is_empty() {
                        <div class={classes!("empty")}>
                            <span>{ "当前过滤条件下没有匹配的账号组" }</span>
                        </div>
                    } else {
                        <div class={classes!("grid", "gap-4", "p-4", "xl:grid-cols-2", "items-start")}>
                            { for filtered_groups.iter().map(|group_item| html! {
                                <GroupEditorCard
                                    key={group_item.id.clone()}
                                    group_item={group_item.clone()}
                                    accounts={(*accounts).clone()}
                                    on_reload={on_reload.clone()}
                                    on_flash={notify.clone()}
                                />
                            }) }
                        </div>
                        <div class={classes!("pager", "px-4", "pb-3", "flex-wrap")}>
                            <span>
                                { format!("总数 {} · 第 {}/{} 页 · 每页 {}", *groups_total, current_page, total_pages, *page_limit) }
                            </span>
                            <Pagination
                                current_page={current_page}
                                total_pages={total_pages}
                                on_page_change={on_page_change}
                            />
                        </div>
                    }
                </section>
            </div>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::{group_matches_query, group_total_pages, toggled_member_names};
    use crate::api::AdminAccountGroupView;

    fn group(name: &str, id: &str, members: &[&str]) -> AdminAccountGroupView {
        AdminAccountGroupView {
            name: name.to_string(),
            id: id.to_string(),
            account_names: members.iter().map(|member| member.to_string()).collect(),
            ..AdminAccountGroupView::default()
        }
    }

    #[test]
    fn group_query_matches_name_id_and_members() {
        let sample = group("Primary Pool", "grp-1", &["kiro-a", "kiro-b"]);

        assert!(group_matches_query(&sample, "primary"));
        assert!(group_matches_query(&sample, "grp-1"));
        assert!(group_matches_query(&sample, "kiro-b"));
        assert!(!group_matches_query(&sample, "missing"));
    }

    #[test]
    fn toggled_member_names_adds_sorted_and_removes() {
        let names = vec!["b".to_string()];

        let added = toggled_member_names(&names, "a".to_string());
        assert_eq!(added, vec!["a".to_string(), "b".to_string()]);

        let removed = toggled_member_names(&added, "b".to_string());
        assert_eq!(removed, vec!["a".to_string()]);
    }

    #[test]
    fn group_total_pages_never_drops_below_one() {
        assert_eq!(group_total_pages(0, 24), 1);
        assert_eq!(group_total_pages(25, 24), 2);
    }
}
