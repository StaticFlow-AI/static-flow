//! LLM gateway account groups page (`/admin/llm-gateway/groups`).
//!
//! Server-paginated group cards with a collapsed create form; the member
//! picker (shared by the create form and the group editors) needs the full
//! account inventory, which is why this page is the only LLM section that
//! loads it. The heavyweight `AccountGroupEditorCard` stays defined in
//! `admin_llm_gateway` and is reused here.

use gloo_timers::callback::Timeout;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_llm_gateway::{admin_group_total_pages, AccountGroupEditorCard};
use crate::{
    api::{
        create_admin_llm_gateway_account_group, fetch_admin_llm_gateway_account_groups_page,
        fetch_admin_llm_gateway_accounts, AccountSummaryView, AdminAccountGroupView,
        CreateAdminAccountGroupInput,
    },
    components::{pagination::Pagination, search_box::SearchBox},
    router::Route,
};

const GROUP_PAGE_SIZE: usize = 24;

#[function_component(AdminLlmGatewayGroupsPage)]
pub fn admin_llm_gateway_groups_page() -> Html {
    let account_groups_page_items = use_state(Vec::<AdminAccountGroupView>::new);
    let account_groups_total = use_state(|| 0_usize);
    let account_groups_page = use_state(|| 1_usize);
    let account_groups_page_limit = use_state(|| GROUP_PAGE_SIZE);
    let account_groups_search = use_state(String::new);
    let accounts = use_state(Vec::<AccountSummaryView>::new);
    let accounts_loading = use_state(|| true);
    let create_account_group_name = use_state(String::new);
    let create_account_group_account_names = use_state(Vec::<String>::new);
    let creating_account_group = use_state(|| false);
    let account_group_form_expanded = use_state(|| false);
    let loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0_u32);
    let toast = use_state(|| None::<(String, bool)>);
    let toast_timeout = use_mut_ref(|| None::<Timeout>);
    let flash = {
        let toast = toast.clone();
        let toast_timeout = toast_timeout.clone();
        Callback::from(move |(message, is_error): (String, bool)| {
            toast.set(Some((message, is_error)));
            toast_timeout.borrow_mut().take();
            let toast = toast.clone();
            let clear_handle = toast_timeout.clone();
            let timeout = Timeout::new(2600, move || {
                toast.set(None);
                clear_handle.borrow_mut().take();
            });
            *toast_timeout.borrow_mut() = Some(timeout);
        })
    };

    let on_reload = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_: ()| refresh_tick.set(refresh_tick.wrapping_add(1)))
    };

    // Server-paginated group inventory; the search box filters the current
    // page client-side.
    {
        let account_groups_page_items = account_groups_page_items.clone();
        let account_groups_total = account_groups_total.clone();
        let account_groups_page = account_groups_page.clone();
        let account_groups_page_limit = account_groups_page_limit.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        use_effect_with((*account_groups_page, *refresh_tick), move |(requested_page, _)| {
            let account_groups_page_items = account_groups_page_items.clone();
            let account_groups_total = account_groups_total.clone();
            let account_groups_page = account_groups_page.clone();
            let account_groups_page_limit = account_groups_page_limit.clone();
            let loading = loading.clone();
            let load_error = load_error.clone();
            let requested_page = (*requested_page).max(1);
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let limit = (*account_groups_page_limit).max(1);
                let offset = requested_page.saturating_sub(1).saturating_mul(limit);
                match fetch_admin_llm_gateway_account_groups_page(limit, offset).await {
                    Ok(resp) => {
                        let effective_limit = resp.limit.max(1);
                        let total_pages = admin_group_total_pages(resp.total, effective_limit);
                        account_groups_total.set(resp.total);
                        account_groups_page_limit.set(effective_limit);
                        if requested_page > total_pages {
                            account_groups_page.set(total_pages);
                        } else {
                            account_groups_page_items.set(resp.groups);
                        }
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    // The member picker needs the full account inventory; groups paging never
    // re-fetches it.
    {
        let accounts = accounts.clone();
        let accounts_loading = accounts_loading.clone();
        let load_error = load_error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let accounts = accounts.clone();
            let accounts_loading = accounts_loading.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                accounts_loading.set(true);
                match fetch_admin_llm_gateway_accounts().await {
                    Ok(resp) => accounts.set(resp.accounts),
                    Err(err) => load_error.set(Some(err)),
                }
                accounts_loading.set(false);
            });
            || ()
        });
    }

    let on_toggle_create_account_group_member = {
        let create_account_group_account_names = create_account_group_account_names.clone();
        Callback::from(move |account_name: String| {
            let mut names = (*create_account_group_account_names).clone();
            if let Some(index) = names.iter().position(|name| name == &account_name) {
                names.remove(index);
            } else {
                names.push(account_name);
                names.sort();
                names.dedup();
            }
            create_account_group_account_names.set(names);
        })
    };

    let on_toggle_account_group_form = {
        let account_group_form_expanded = account_group_form_expanded.clone();
        Callback::from(move |_| {
            account_group_form_expanded.set(!*account_group_form_expanded);
        })
    };

    let on_create_account_group = {
        let create_account_group_name = create_account_group_name.clone();
        let create_account_group_account_names = create_account_group_account_names.clone();
        let creating_account_group = creating_account_group.clone();
        let flash = flash.clone();
        let load_error = load_error.clone();
        let on_reload = on_reload.clone();
        Callback::from(move |_| {
            if *creating_account_group {
                return;
            }
            let group_name = (*create_account_group_name).trim().to_string();
            let account_names = (*create_account_group_account_names).clone();
            let create_account_group_name = create_account_group_name.clone();
            let create_account_group_account_names = create_account_group_account_names.clone();
            let creating_account_group = creating_account_group.clone();
            let flash = flash.clone();
            let load_error = load_error.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if group_name.is_empty() {
                    let message = "账号组名称不能为空".to_string();
                    load_error.set(Some(message.clone()));
                    flash.emit((message, true));
                    return;
                }
                if account_names.is_empty() {
                    let message = "账号组至少需要选择一个账号".to_string();
                    load_error.set(Some(message.clone()));
                    flash.emit((message, true));
                    return;
                }
                creating_account_group.set(true);
                match create_admin_llm_gateway_account_group(CreateAdminAccountGroupInput {
                    name: &group_name,
                    account_names: account_names.as_slice(),
                })
                .await
                {
                    Ok(_) => {
                        create_account_group_name.set(String::new());
                        create_account_group_account_names.set(Vec::new());
                        load_error.set(None);
                        flash.emit((format!("已创建账号组 `{group_name}`"), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((format!("创建账号组失败\n{err}"), true));
                    },
                }
                creating_account_group.set(false);
            });
        })
    };

    let account_groups_query_lower = (*account_groups_search).trim().to_lowercase();
    let filtered_account_groups: Vec<AdminAccountGroupView> = {
        let q = account_groups_query_lower.clone();
        use_memo(((*account_groups_page_items).clone(), q.clone()), move |(items, q)| {
            if q.is_empty() {
                items.clone()
            } else {
                items
                    .iter()
                    .filter(|g| {
                        if g.name.to_lowercase().contains(q)
                            || g.id.to_lowercase().contains(q)
                            || g.provider_type.to_lowercase().contains(q)
                        {
                            return true;
                        }
                        g.account_names.iter().any(|n| n.to_lowercase().contains(q))
                    })
                    .cloned()
                    .collect()
            }
        })
        .as_ref()
        .clone()
    };
    let account_groups_total_pages =
        admin_group_total_pages(*account_groups_total, *account_groups_page_limit);
    let account_groups_current_page = (*account_groups_page).clamp(1, account_groups_total_pages);
    let on_account_groups_page_change = {
        let account_groups_page = account_groups_page.clone();
        let account_groups_page_items = account_groups_page_items.clone();
        let account_groups_total = account_groups_total.clone();
        let account_groups_page_limit = account_groups_page_limit.clone();
        let load_error = load_error.clone();
        Callback::from(move |page: usize| {
            let page = page.max(1);
            account_groups_page.set(page);
            let account_groups_page_items = account_groups_page_items.clone();
            let account_groups_total = account_groups_total.clone();
            let account_groups_page_limit = account_groups_page_limit.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let limit = (*account_groups_page_limit).max(1);
                let offset = page.saturating_sub(1) * limit;
                match fetch_admin_llm_gateway_account_groups_page(limit, offset).await {
                    Ok(resp) => {
                        account_groups_total.set(resp.total);
                        account_groups_page_limit.set(resp.limit.max(1));
                        account_groups_page_items.set(resp.groups);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
            });
        })
    };
    let on_account_groups_search_change = {
        let account_groups_search = account_groups_search.clone();
        Callback::from(move |v: String| account_groups_search.set(v))
    };

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "LLM Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Groups" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminLlmGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                        <Link<Route> to={Route::AdminLlmGatewayKeys} classes={classes!("linkbtn")}>{ "Keys" }</Link<Route>>
                        <button type="button" class={classes!("primary")} disabled={*loading} onclick={{
                            let on_reload = on_reload.clone();
                            Callback::from(move |_| on_reload.emit(()))
                        }}>
                            { if *loading { "刷新中..." } else { "刷新" } }
                        </button>
                    </div>
                </header>

                if let Some(err) = (*load_error).clone() {
                    <div class={classes!("errorline", "text-sm")}>{ err }</div>
                }

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Account Groups" }</h2>
                            <p class={classes!("mt-2", "mb-0", "text-sm", "text-[var(--muted)]")}>
                                { "先为账号分组，再让 key 选择组而不是直接勾账号。固定路由请选择单账号组；自动路由可以选任意组，留空则继续使用全账号池。" }
                            </p>
                        </div>
                        <button
                            class={classes!("ghost")}
                            onclick={{
                                let on_reload = on_reload.clone();
                                Callback::from(move |_| on_reload.emit(()))
                            }}
                            disabled={*loading}
                        >
                            { if *loading { "刷新中..." } else { "刷新账号组" } }
                        </button>
                    </div>

                    <div class={classes!("mt-4", "max-w-md")}>
                        <SearchBox
                            value={(*account_groups_search).clone()}
                            on_change={on_account_groups_search_change.clone()}
                            placeholder={AttrValue::Static("搜索账号组名 / id / 成员账号")}
                        />
                    </div>
                    if !account_groups_query_lower.is_empty() {
                        <p class={classes!("mt-2", "text-xs", "text-[var(--muted)]", "font-mono")}>
                            { format!("当前页匹配 {}/{} · 总数 {}", filtered_account_groups.len(), account_groups_page_items.len(), *account_groups_total) }
                        </p>
                    }

                    <div class={classes!("mt-4", "rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                        <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                            <div>
                                <h3 class={classes!("m-0", "text-sm", "font-semibold")}>{ "创建账号组" }</h3>
                                <p class={classes!("mt-1", "mb-0", "text-xs", "text-[var(--muted)]")}>
                                    { "默认收起，只在需要新增轮询号池时展开。" }
                                </p>
                            </div>
                            <button
                                type="button"
                                class={classes!("ghost")}
                                onclick={on_toggle_account_group_form.clone()}
                            >
                                { if *account_group_form_expanded { "收起 ▲" } else { "展开 ▼" } }
                            </button>
                        </div>
                        if *account_group_form_expanded {
                            <div class={classes!("mt-4", "grid", "gap-3")}>
                                <label class={classes!("text-sm")}>
                                    <span class={classes!("text-[var(--muted)]")}>{ "组名" }</span>
                                    <input
                                        type="text"
                                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                        value={(*create_account_group_name).clone()}
                                        oninput={{
                                            let create_account_group_name = create_account_group_name.clone();
                                            Callback::from(move |event: InputEvent| {
                                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                    create_account_group_name.set(target.value());
                                                }
                                            })
                                        }}
                                    />
                                </label>
                                <div class={classes!("space-y-2")}>
                                    <div class={classes!("text-sm", "text-[var(--muted)]")}>{ "成员账号" }</div>
                                    if *accounts_loading {
                                        <div class={classes!("rounded-lg", "border", "border-dashed", "border-[var(--border)]", "px-3", "py-3", "text-xs", "text-[var(--muted)]")}>
                                            { "正在加载账号候选..." }
                                        </div>
                                    } else if accounts.is_empty() {
                                        <div class={classes!("rounded-lg", "border", "border-dashed", "border-[var(--border)]", "px-3", "py-3", "text-xs", "text-[var(--muted)]")}>
                                            { "当前没有可加入账号组的账号。" }
                                        </div>
                                    } else {
                                        <div class={classes!("grid", "gap-2", "xl:grid-cols-2")}>
                                            { for accounts.iter().map(|account| {
                                                let checked = create_account_group_account_names.iter().any(|name| name == &account.name);
                                                let account_name = account.name.clone();
                                                let on_toggle_create_account_group_member =
                                                    on_toggle_create_account_group_member.clone();
                                                html! {
                                                    <label class={classes!(
                                                        "flex", "cursor-pointer", "items-center", "gap-3", "rounded-lg", "border", "px-3", "py-2.5",
                                                        if checked {
                                                            "border-sky-500/30 bg-sky-500/8"
                                                        } else {
                                                            "border-[var(--border)] bg-[var(--surface)]"
                                                        }
                                                    )}>
                                                        <input
                                                            type="checkbox" class={classes!("min-h-0", "w-auto")}
                                                            checked={checked}
                                                            onchange={Callback::from(move |_| {
                                                                on_toggle_create_account_group_member.emit(account_name.clone())
                                                            })}
                                                        />
                                                        <div class={classes!("min-w-0", "flex-1")}>
                                                            <div class={classes!("font-semibold", "text-[var(--text)]")}>{ account.name.clone() }</div>
                                                            if account.status != "disabled" {
                                                                <div class={classes!("mt-1", "font-mono", "text-[11px]", "text-[var(--muted)]")}>
                                                                    { format!(
                                                                        "5h {} / wk {}",
                                                                        account.primary_remaining_percent.map(|value| format!("{value:.0}%")).unwrap_or_else(|| "-".to_string()),
                                                                        account.secondary_remaining_percent.map(|value| format!("{value:.0}%")).unwrap_or_else(|| "-".to_string())
                                                                    ) }
                                                                </div>
                                                            }
                                                        </div>
                                                    </label>
                                                }
                                            }) }
                                        </div>
                                    }
                                </div>
                                <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                    <span class={classes!("text-xs", "text-[var(--muted)]")}>
                                        { format!(
                                            "当前成员: {}",
                                            if create_account_group_account_names.is_empty() {
                                                "无".to_string()
                                            } else {
                                                create_account_group_account_names.join(", ")
                                            }
                                        ) }
                                    </span>
                                    <button
                                        class={classes!("primary")}
                                        onclick={on_create_account_group}
                                        disabled={*creating_account_group}
                                    >
                                        { if *creating_account_group { "创建中..." } else { "创建账号组" } }
                                    </button>
                                </div>
                            </div>
                        }
                    </div>

                    <div class={classes!("mt-5", "grid", "gap-4", "2xl:grid-cols-2")}>
                        if account_groups_page_items.is_empty() && !*loading {
                            <div class={classes!("rounded-xl", "border", "border-dashed", "border-[var(--border)]", "px-4", "py-10", "text-center", "text-[var(--muted)]")}>
                                { "当前还没有账号组。" }
                            </div>
                        } else if filtered_account_groups.is_empty() {
                            <div class={classes!("rounded-xl", "border", "border-dashed", "border-[var(--border)]", "px-4", "py-6", "text-center", "text-[var(--muted)]")}>
                                { "当前过滤条件下没有匹配的账号组。" }
                            </div>
                        } else {
                            { for filtered_account_groups.iter().map(|group_item| html! {
                                <AccountGroupEditorCard
                                    key={group_item.id.clone()}
                                    group_item={group_item.clone()}
                                    accounts={(*accounts).clone()}
                                    on_changed={on_reload.clone()}
                                    on_flash={flash.clone()}
                                />
                            }) }
                        }
                    </div>
                    <div class={classes!("mt-4")}>
                        <div class={classes!("mb-2", "text-xs", "text-[var(--muted)]", "font-mono")}>
                            { format!("总数 {} · 第 {}/{} 页 · 每页 {}", *account_groups_total, account_groups_current_page, account_groups_total_pages, *account_groups_page_limit) }
                        </div>
                        <Pagination
                            current_page={account_groups_current_page}
                            total_pages={account_groups_total_pages}
                            on_page_change={on_account_groups_page_change.clone()}
                        />
                    </div>
                </section>
            </div>
            if let Some((message, is_error)) = (*toast).clone() {
                <div class={classes!("toasts")}>
                    <div class={classes!("toast", if is_error { "error" } else { "ok" })}>
                        { message }
                    </div>
                </div>
            }
        </main>
    }
}
