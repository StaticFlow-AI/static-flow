//! Legacy Yew Codex keys page (`/admin/llm-gateway/keys`).
//!
//! Default production builds render a handoff to
//! `deps/llm-access/apps/llm-access-frontend` at `/console/codex/keys`.
//! This page is mounted only when `STATICFLOW_ENABLE_LEGACY_LLM_ADMIN=1`.
//!
//! The heavyweight `KeyEditorCard` stays defined in `admin_llm_gateway` and
//! is reused here so the editor logic is not duplicated.

use gloo_timers::callback::Timeout;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_llm_gateway::{admin_group_total_pages, KeyEditorCard};
use crate::{
    api::{
        create_admin_llm_gateway_key, fetch_admin_llm_gateway_account_group_options,
        fetch_admin_llm_gateway_keys_page_with_query, AdminAccountGroupOptionView,
        AdminLlmGatewayKeyPageQuery, AdminLlmGatewayKeyView, AdminLlmGatewayKeysSummaryView,
    },
    components::{copy_button::copy_to_clipboard, pagination::Pagination, search_box::SearchBox},
    router::Route,
};

const KEY_PAGE_SIZE: usize = 8;

#[derive(Clone, Copy, PartialEq)]
enum KeySortMode {
    None,
    QuotaAsc,
    QuotaDesc,
    UsageAsc,
    UsageDesc,
}

/// Inputs for the "create new API key" panel at the top of the Keys page.
/// Bundled so the submit callback and `.set(next)` paths read a single clone
/// of the struct.
#[derive(Clone, PartialEq)]
struct CreateKeyForm {
    name: String,
    quota: String,
    public: bool,
    request_max_concurrency: String,
    request_min_start_interval_ms: String,
}

impl Default for CreateKeyForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            quota: "100000".to_string(),
            public: true,
            request_max_concurrency: String::new(),
            request_min_start_interval_ms: String::new(),
        }
    }
}

#[function_component(AdminLlmGatewayKeysPage)]
pub fn admin_llm_gateway_keys_page() -> Html {
    let keys = use_state(Vec::<AdminLlmGatewayKeyView>::new);
    let keys_summary = use_state(AdminLlmGatewayKeysSummaryView::default);
    let keys_search = use_state(String::new);
    let keys_sort_mode = use_state(|| KeySortMode::None);
    let keys_show_active_only = use_state(|| false);
    let keys_page = use_state(|| 1_usize);
    let keys_total = use_state(|| 0_usize);
    let keys_page_limit = use_state(|| KEY_PAGE_SIZE);
    let account_group_options = use_state(Vec::<AdminAccountGroupOptionView>::new);
    let create_key = use_state(CreateKeyForm::default);
    let creating = use_state(|| false);
    let refreshing_key_id = use_state(|| None::<String>);
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

    let on_copy = {
        let flash = flash.clone();
        Callback::from(move |(label, value): (String, String)| {
            copy_to_clipboard(&value);
            flash.emit((format!("已复制{}", label), false));
        })
    };

    // Editor dropdown data (account group options) loads once per refresh so
    // key paging does not re-fetch it.
    {
        let account_group_options = account_group_options.clone();
        let load_error = load_error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let account_group_options = account_group_options.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_llm_gateway_account_group_options().await {
                    Ok(options) => account_group_options.set(options),
                    Err(err) => load_error.set(Some(err)),
                }
            });
            || ()
        });
    }

    // Server-paginated key inventory. Search / sort / active-only all run
    // server-side, so every filter change re-fetches the page.
    {
        let keys = keys.clone();
        let keys_summary = keys_summary.clone();
        let keys_total = keys_total.clone();
        let keys_page = keys_page.clone();
        let keys_page_limit = keys_page_limit.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        use_effect_with(
            (
                *keys_page,
                (*keys_search).clone(),
                *keys_sort_mode,
                *keys_show_active_only,
                *refresh_tick,
            ),
            move |(requested_page, search, sort_mode, active_only, _)| {
                let requested_page = (*requested_page).max(1);
                let key_query = AdminLlmGatewayKeyPageQuery {
                    q: Some(search.clone()),
                    active_only: *active_only,
                    sort: Some(
                        match sort_mode {
                            KeySortMode::QuotaAsc => "quota_asc",
                            KeySortMode::QuotaDesc => "quota_desc",
                            KeySortMode::UsageAsc => "usage_asc",
                            KeySortMode::UsageDesc => "usage_desc",
                            KeySortMode::None => "",
                        }
                        .to_string(),
                    ),
                };
                let keys = keys.clone();
                let keys_summary = keys_summary.clone();
                let keys_total = keys_total.clone();
                let keys_page = keys_page.clone();
                let keys_page_limit = keys_page_limit.clone();
                let loading = loading.clone();
                let load_error = load_error.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    loading.set(true);
                    let limit = (*keys_page_limit).max(1);
                    let offset = requested_page.saturating_sub(1).saturating_mul(limit);
                    match fetch_admin_llm_gateway_keys_page_with_query(limit, offset, &key_query)
                        .await
                    {
                        Ok(response) => {
                            let effective_limit = response.limit.max(1);
                            let total_pages =
                                admin_group_total_pages(response.total, effective_limit);
                            keys_summary.set(response.summary);
                            keys_total.set(response.total);
                            keys_page_limit.set(effective_limit);
                            if requested_page > total_pages {
                                keys_page.set(total_pages);
                            } else {
                                keys.set(response.keys);
                            }
                            load_error.set(None);
                        },
                        Err(err) => load_error.set(Some(err)),
                    }
                    loading.set(false);
                });
                || ()
            },
        );
    }

    let on_create = {
        let create_key = create_key.clone();
        let creating = creating.clone();
        let load_error = load_error.clone();
        let flash = flash.clone();
        let on_reload = on_reload.clone();
        Callback::from(move |_| {
            let current = (*create_key).clone();
            let name = current.name.trim().to_string();
            let quota = current.quota.trim().parse::<u64>();
            let public_visible = current.public;
            let request_max_concurrency = current.request_max_concurrency.trim().to_string();
            let request_min_start_interval_ms =
                current.request_min_start_interval_ms.trim().to_string();
            let creating = creating.clone();
            let load_error = load_error.clone();
            let flash = flash.clone();
            let on_reload = on_reload.clone();
            let create_key = create_key.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(quota) = quota else {
                    let message = "主额度必须是正整数".to_string();
                    load_error.set(Some(message.clone()));
                    flash.emit((message, true));
                    return;
                };
                let request_max_concurrency = if request_max_concurrency.is_empty() {
                    None
                } else {
                    match request_max_concurrency.parse::<u64>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            let message = "并发上限必须是整数，留空表示不限制".to_string();
                            load_error.set(Some(message.clone()));
                            flash.emit((message, true));
                            return;
                        },
                    }
                };
                let request_min_start_interval_ms = if request_min_start_interval_ms.is_empty() {
                    None
                } else {
                    match request_min_start_interval_ms.parse::<u64>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            let message = "请求间隔必须是整数毫秒，留空表示不限制".to_string();
                            load_error.set(Some(message.clone()));
                            flash.emit((message, true));
                            return;
                        },
                    }
                };
                creating.set(true);
                match create_admin_llm_gateway_key(
                    &name,
                    quota,
                    public_visible,
                    request_max_concurrency,
                    request_min_start_interval_ms,
                )
                .await
                {
                    Ok(_) => {
                        // Reset the form inputs after a successful create;
                        // leave `public` / `quota` defaults as-is so the next
                        // create has the same baseline.
                        let mut next = (*create_key).clone();
                        next.name = String::new();
                        next.request_max_concurrency = String::new();
                        next.request_min_start_interval_ms = String::new();
                        create_key.set(next);
                        load_error.set(None);
                        flash.emit((format!("已创建 key `{}`", name), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((format!("创建 key `{}` 失败\n{err}", name), true));
                    },
                }
                creating.set(false);
            });
        })
    };

    // A per-card refresh re-reads the latest counters for a single key by
    // re-fetching the current page.
    let on_refresh_key = {
        let on_reload = on_reload.clone();
        let flash = flash.clone();
        let refreshing_key_id = refreshing_key_id.clone();
        Callback::from(move |(key_id, key_name): (String, String)| {
            refreshing_key_id.set(Some(key_id));
            on_reload.emit(());
            flash.emit((format!("已触发 key `{}` 刷新", key_name), false));
            refreshing_key_id.set(None);
        })
    };

    let key_summary = *keys_summary;
    let keys_total_pages = admin_group_total_pages(*keys_total, *keys_page_limit);
    let keys_current_page = (*keys_page).clamp(1, keys_total_pages);
    let keys_page_entries: Vec<&AdminLlmGatewayKeyView> = keys.iter().collect();
    let on_keys_page_change = {
        let keys_page = keys_page.clone();
        Callback::from(move |p: usize| keys_page.set(p))
    };
    let on_keys_search_change = {
        let keys_search = keys_search.clone();
        Callback::from(move |v: String| keys_search.set(v))
    };

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "LLM Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Keys" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminLlmGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                        <Link<Route> to={Route::AdminLlmGatewayGroups} classes={classes!("linkbtn")}>{ "Groups" }</Link<Route>>
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
                    <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Create Key" }</h2>
                    <div class={classes!("mt-3", "grid", "gap-3")}>
                        <div class={classes!("grid", "gap-3", "md:grid-cols-2")}>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "名称" }</span>
                                <input
                                    type="text"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={create_key.name.clone()}
                                    oninput={{
                                        let create_key = create_key.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                let mut next = (*create_key).clone();
                                                next.name = target.value();
                                                create_key.set(next);
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "主额度上限" }</span>
                                <input
                                    type="number"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={create_key.quota.clone()}
                                    oninput={{
                                        let create_key = create_key.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                let mut next = (*create_key).clone();
                                                next.quota = target.value();
                                                create_key.set(next);
                                            }
                                        })
                                    }}
                                />
                            </label>
                        </div>
                        <div class={classes!("grid", "gap-3", "md:grid-cols-2")}>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "并发上限" }</span>
                                <input
                                    type="number"
                                    placeholder="留空表示不限制"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={create_key.request_max_concurrency.clone()}
                                    oninput={{
                                        let create_key = create_key.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                let mut next = (*create_key).clone();
                                                next.request_max_concurrency = target.value();
                                                create_key.set(next);
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "请求起始间隔 ms" }</span>
                                <input
                                    type="number"
                                    placeholder="留空表示不限制"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={create_key.request_min_start_interval_ms.clone()}
                                    oninput={{
                                        let create_key = create_key.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                let mut next = (*create_key).clone();
                                                next.request_min_start_interval_ms = target.value();
                                                create_key.set(next);
                                            }
                                        })
                                    }}
                                />
                            </label>
                        </div>
                        <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                            <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                                <input
                                    type="checkbox" class={classes!("min-h-0", "w-auto")}
                                    checked={create_key.public}
                                    onchange={{
                                        let create_key = create_key.clone();
                                        Callback::from(move |event: Event| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                let mut next = (*create_key).clone();
                                                next.public = target.checked();
                                                create_key.set(next);
                                            }
                                        })
                                    }}
                                />
                                <span>{ "公开" }</span>
                            </label>
                            <button class={classes!("primary")} onclick={on_create} disabled={*creating}>
                                { if *creating { "创建中..." } else { "创建" } }
                            </button>
                        </div>
                    </div>
                </section>

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Key Inventory" }</h2>
                    </div>
                    <div class={classes!("mt-4", "max-w-md")}>
                        <SearchBox
                            value={(*keys_search).clone()}
                            on_change={on_keys_search_change.clone()}
                            placeholder={AttrValue::Static("搜索 key 名称 / id / provider / 状态")}
                        />
                    </div>
                    // Sort & filter toolbar
                    <div class={classes!("mt-3", "flex", "items-center", "gap-2", "flex-wrap")}>
                        <button
                            type="button"
                            class={classes!(
                                "rounded-full", "px-3", "py-1.5", "text-xs", "font-semibold", "border", "transition-colors",
                                if *keys_show_active_only {
                                    "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-400/50"
                                } else {
                                    "bg-[var(--surface)] text-[var(--muted)] border-[var(--border)] hover:text-[var(--text)]"
                                }
                            )}
                            onclick={{
                                let keys_show_active_only = keys_show_active_only.clone();
                                let keys_page = keys_page.clone();
                                Callback::from(move |_| {
                                    keys_show_active_only.set(!*keys_show_active_only);
                                    keys_page.set(1);
                                })
                            }}
                        >
                            { "Active" }
                        </button>
                        <span class={classes!("w-px", "h-5", "bg-[var(--border)]")} />
                        <button
                            type="button"
                            class={classes!(
                                "rounded-full", "px-3", "py-1.5", "text-xs", "font-semibold", "border", "transition-colors",
                                if matches!(*keys_sort_mode, KeySortMode::QuotaAsc | KeySortMode::QuotaDesc) {
                                    "bg-teal-500/15 text-teal-700 dark:text-teal-300 border-teal-400/50"
                                } else {
                                    "bg-[var(--surface)] text-[var(--muted)] border-[var(--border)] hover:text-[var(--text)]"
                                }
                            )}
                            onclick={{
                                let keys_sort_mode = keys_sort_mode.clone();
                                let keys_page = keys_page.clone();
                                Callback::from(move |_| {
                                    let next = match *keys_sort_mode {
                                        KeySortMode::QuotaAsc => KeySortMode::QuotaDesc,
                                        KeySortMode::QuotaDesc => KeySortMode::None,
                                        _ => KeySortMode::QuotaAsc,
                                    };
                                    keys_sort_mode.set(next);
                                    keys_page.set(1);
                                })
                            }}
                        >
                            { match *keys_sort_mode {
                                KeySortMode::QuotaAsc => "Quota \u{2191}",
                                KeySortMode::QuotaDesc => "Quota \u{2193}",
                                _ => "Quota",
                            }}
                        </button>
                        <button
                            type="button"
                            class={classes!(
                                "rounded-full", "px-3", "py-1.5", "text-xs", "font-semibold", "border", "transition-colors",
                                if matches!(*keys_sort_mode, KeySortMode::UsageAsc | KeySortMode::UsageDesc) {
                                    "bg-violet-500/15 text-violet-700 dark:text-violet-300 border-violet-400/50"
                                } else {
                                    "bg-[var(--surface)] text-[var(--muted)] border-[var(--border)] hover:text-[var(--text)]"
                                }
                            )}
                            onclick={{
                                let keys_sort_mode = keys_sort_mode.clone();
                                let keys_page = keys_page.clone();
                                Callback::from(move |_| {
                                    let next = match *keys_sort_mode {
                                        KeySortMode::UsageAsc => KeySortMode::UsageDesc,
                                        KeySortMode::UsageDesc => KeySortMode::None,
                                        _ => KeySortMode::UsageAsc,
                                    };
                                    keys_sort_mode.set(next);
                                    keys_page.set(1);
                                })
                            }}
                        >
                            { match *keys_sort_mode {
                                KeySortMode::UsageAsc => "Usage \u{2191}",
                                KeySortMode::UsageDesc => "Usage \u{2193}",
                                _ => "Usage",
                            }}
                        </button>
                    </div>
                    <div class={classes!("mt-2", "flex", "items-center", "justify-between", "text-xs", "text-[var(--muted)]")}>
                        <span>{ format!("总数 {} · 当前筛选 {} · 本页 {}", key_summary.total, *keys_total, keys.len()) }</span>
                        if keys_total_pages > 1 {
                            <span class={classes!("font-mono")}>{ format!("{}/{}", keys_current_page, keys_total_pages) }</span>
                        }
                    </div>
                    <div class={classes!("mt-3", "grid", "gap-4", "2xl:grid-cols-2")}>
                        if keys_page_entries.is_empty() {
                            <div class={classes!("rounded-xl", "border", "border-dashed", "border-[var(--border)]", "px-4", "py-10", "text-center", "text-[var(--muted)]")}>
                                { if keys.is_empty() {
                                    "当前还没有可管理的 key。"
                                } else {
                                    "当前过滤条件下没有匹配的 key。"
                                }}
                            </div>
                        } else {
                            { for keys_page_entries.iter().map(|key_item| html! {
                                <KeyEditorCard
                                    key={key_item.id.clone()}
                                    key_item={(*key_item).clone()}
                                    on_changed={on_reload.clone()}
                                    on_refresh={on_refresh_key.clone()}
                                    on_copy={on_copy.clone()}
                                    on_flash={flash.clone()}
                                    refreshing={(*refreshing_key_id).as_deref() == Some(key_item.id.as_str())}
                                    account_groups={(*account_group_options).clone()}
                                />
                            }) }
                        }
                    </div>
                    <div class={classes!("mt-4")}>
                        <Pagination
                            current_page={keys_current_page}
                            total_pages={keys_total_pages}
                            on_page_change={on_keys_page_change.clone()}
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
