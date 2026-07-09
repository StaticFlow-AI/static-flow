//! Kiro private keys page (`/admin/kiro-gateway/keys`).
//!
//! Owns the create-key form, the server-paginated key inventory and the
//! per-key editor. The heavyweight `KiroKeyEditorCard` and its cache-policy
//! editor stay defined in `admin_kiro_gateway` and are reused here so the
//! editor logic is not duplicated.

use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_kiro_gateway::{
    admin_kiro_key_total_pages, format_json_for_textarea, parse_kiro_cache_policy_form_json,
    KiroCachePolicyForm, KiroKeyEditorCard, DEFAULT_KIRO_KEY_PAGE_SIZE,
};
use crate::{
    api::{
        create_admin_kiro_key, fetch_admin_anthropic_upstream_channels,
        fetch_admin_kiro_account_group_options, fetch_admin_kiro_keys_page,
        fetch_admin_llm_gateway_config, fetch_kiro_models, AdminAccountGroupOptionView,
        AdminAnthropicUpstreamChannelView, AdminLlmGatewayKeyView, AdminLlmGatewayKeysSummaryView,
        KiroModelView,
    },
    components::{copy_button::copy_to_clipboard, pagination::Pagination, search_box::SearchBox},
    router::Route,
};

/// Global cache-policy + billable-multiplier defaults the editor compares each
/// key against. Fetched once so the per-key "inherit vs override" display is
/// accurate.
#[derive(Clone, PartialEq, Default)]
struct GlobalPolicyDefaults {
    policy_form: KiroCachePolicyForm,
    billable_multiplier_json: String,
}

fn key_matches_query(key_item: &AdminLlmGatewayKeyView, query_lower: &str) -> bool {
    [
        key_item.name.to_lowercase(),
        key_item.id.to_lowercase(),
        key_item.provider_type.to_lowercase(),
        key_item.status.to_lowercase(),
    ]
    .iter()
    .any(|value| value.contains(query_lower))
}

#[function_component(AdminKiroGatewayKeysPage)]
pub fn admin_kiro_gateway_keys_page() -> Html {
    let keys = use_state(Vec::<AdminLlmGatewayKeyView>::new);
    let keys_summary = use_state(AdminLlmGatewayKeysSummaryView::default);
    let keys_total = use_state(|| 0usize);
    let keys_page = use_state(|| 1usize);
    let keys_page_limit = use_state(|| DEFAULT_KIRO_KEY_PAGE_SIZE);
    let keys_search = use_state(String::new);

    let account_group_options = use_state(Vec::<AdminAccountGroupOptionView>::new);
    let kiro_models = use_state(Vec::<KiroModelView>::new);
    let anthropic_channels = use_state(Vec::<AdminAnthropicUpstreamChannelView>::new);
    let global_defaults = use_state(GlobalPolicyDefaults::default);

    let loading = use_state(|| true);
    let flash = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0u32);

    let new_key_name = use_state(|| "kiro-private".to_string());
    let new_key_quota = use_state(|| "1000000".to_string());
    let creating_key = use_state(|| false);

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

    let on_copy = {
        let notify = notify.clone();
        Callback::from(move |(label, value): (String, String)| {
            copy_to_clipboard(&value);
            notify.emit((format!("Copied {label} to clipboard."), false));
        })
    };

    // Global cache/billable defaults load once per refresh (independent of key
    // paging) so key cards can render their inherit/override comparison.
    {
        let global_defaults = global_defaults.clone();
        let error = error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let global_defaults = global_defaults.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_llm_gateway_config().await {
                    Ok(config) => {
                        match parse_kiro_cache_policy_form_json(&config.kiro_cache_policy_json) {
                            Ok(policy_form) => global_defaults.set(GlobalPolicyDefaults {
                                policy_form,
                                billable_multiplier_json: format_json_for_textarea(
                                    &config.kiro_billable_model_multipliers_json,
                                ),
                            }),
                            Err(err) => error.set(Some(err)),
                        }
                    },
                    Err(err) => error.set(Some(err)),
                }
            });
            || ()
        });
    }

    // Editor dropdown data (group options, models, upstream channels) loads
    // once per refresh; key paging does not re-fetch it.
    {
        let account_group_options = account_group_options.clone();
        let kiro_models = kiro_models.clone();
        let anthropic_channels = anthropic_channels.clone();
        let error = error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let account_group_options = account_group_options.clone();
            let kiro_models = kiro_models.clone();
            let anthropic_channels = anthropic_channels.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = async {
                    let options = fetch_admin_kiro_account_group_options().await?;
                    let models = fetch_kiro_models().await?;
                    let channels = fetch_admin_anthropic_upstream_channels().await?;
                    Ok::<_, String>((options, models, channels))
                }
                .await;
                match result {
                    Ok((options, models, channels)) => {
                        account_group_options.set(options);
                        kiro_models.set(models.data);
                        anthropic_channels.set(channels.channels);
                    },
                    Err(err) => error.set(Some(err)),
                }
            });
            || ()
        });
    }

    {
        let keys = keys.clone();
        let keys_summary = keys_summary.clone();
        let keys_total = keys_total.clone();
        let keys_page = keys_page.clone();
        let keys_page_limit = keys_page_limit.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with((*keys_page, *refresh_tick), move |(requested_page, _)| {
            let keys = keys.clone();
            let keys_summary = keys_summary.clone();
            let keys_total = keys_total.clone();
            let keys_page = keys_page.clone();
            let keys_page_limit = keys_page_limit.clone();
            let loading = loading.clone();
            let error = error.clone();
            let requested_page = (*requested_page).max(1);
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let limit = (*keys_page_limit).max(1);
                let offset = requested_page.saturating_sub(1).saturating_mul(limit);
                match fetch_admin_kiro_keys_page(limit, offset).await {
                    Ok(response) => {
                        let effective_limit = response.limit.max(1);
                        let total_pages =
                            admin_kiro_key_total_pages(response.total, effective_limit);
                        keys_summary.set(response.summary);
                        keys_total.set(response.total);
                        keys_page_limit.set(effective_limit);
                        if requested_page > total_pages {
                            keys_page.set(total_pages);
                        } else {
                            keys.set(response.keys);
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

    let on_create_key = {
        let new_key_name = new_key_name.clone();
        let new_key_quota = new_key_quota.clone();
        let notify = notify.clone();
        let on_reload = on_reload.clone();
        let creating_key = creating_key.clone();
        Callback::from(move |_| {
            if *creating_key {
                return;
            }
            let name = (*new_key_name).clone();
            let quota = (*new_key_quota).clone();
            let notify = notify.clone();
            let on_reload = on_reload.clone();
            let creating_key = creating_key.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(parsed_quota) = quota.trim().parse::<u64>() else {
                    notify.emit(("Quota must be a valid integer.".to_string(), true));
                    return;
                };
                creating_key.set(true);
                match create_admin_kiro_key(name.trim(), parsed_quota).await {
                    Ok(key) => {
                        notify.emit((format!("Created Kiro key `{}`.", key.name), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        notify.emit((format!("Failed to create Kiro key.\n{err}"), true));
                    },
                }
                creating_key.set(false);
            });
        })
    };

    let query_lower = (*keys_search).trim().to_lowercase();
    let filtered_keys: Vec<AdminLlmGatewayKeyView> = if query_lower.is_empty() {
        (*keys).clone()
    } else {
        (*keys)
            .iter()
            .filter(|key_item| key_matches_query(key_item, &query_lower))
            .cloned()
            .collect()
    };
    let total_pages = admin_kiro_key_total_pages(*keys_total, *keys_page_limit);
    let current_page = (*keys_page).clamp(1, total_pages);
    let summary = *keys_summary;

    let on_search_change = {
        let keys_search = keys_search.clone();
        Callback::from(move |value: String| keys_search.set(value))
    };
    let on_page_change = {
        let keys_page = keys_page.clone();
        Callback::from(move |page: usize| keys_page.set(page))
    };

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "Kiro Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Keys" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminKiroGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                        <Link<Route> to={Route::AdminKiroGatewayGroups} classes={classes!("linkbtn")}>{ "Groups" }</Link<Route>>
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
                    <div class={classes!("stat-strip")}>
                        <div class={classes!("stat")}>
                            <span>{ "Total Keys" }</span>
                            <b>{ summary.total }</b>
                        </div>
                        <div class={classes!("stat", (summary.total.saturating_sub(summary.active_count) > 0).then_some("warn"))}>
                            <span>{ "Active" }</span>
                            <b>{ summary.active_count }</b>
                        </div>
                    </div>
                </section>

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <h2>{ "Create Kiro Key" }</h2>
                    </div>
                    <div class={classes!("panel-body")}>
                        <div class={classes!("grid", "gap-3", "items-end", "sm:grid-cols-3")}>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Key Name" }
                                <input value={(*new_key_name).clone()} oninput={{
                                    let new_key_name = new_key_name.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: web_sys::HtmlInputElement = event.target_unchecked_into();
                                        new_key_name.set(input.value());
                                    })
                                }} />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Quota" }
                                <input class={classes!("mono")} value={(*new_key_quota).clone()} oninput={{
                                    let new_key_quota = new_key_quota.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: web_sys::HtmlInputElement = event.target_unchecked_into();
                                        new_key_quota.set(input.value());
                                    })
                                }} />
                            </label>
                            <button type="button" class={classes!("primary")} onclick={on_create_key} disabled={*creating_key}>
                                { if *creating_key { "Creating..." } else { "Create Kiro Key" } }
                            </button>
                        </div>
                    </div>
                </section>

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <div>
                            <h2>{ "Kiro Key Inventory" }</h2>
                            <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { format!("总数 {} · 第 {}/{} 页 · 每页 {}", *keys_total, current_page, total_pages, *keys_page_limit) }
                            </p>
                        </div>
                        <div class={classes!("flex", "items-center", "gap-3", "min-w-0", "flex-1", "justify-end")}>
                            if !query_lower.is_empty() {
                                <span class={classes!("badge")}>{ format!("匹配 {}/{}", filtered_keys.len(), keys.len()) }</span>
                            }
                            <div class={classes!("w-full", "max-w-md")}>
                                <SearchBox
                                    value={(*keys_search).clone()}
                                    on_change={on_search_change}
                                    placeholder={AttrValue::Static("搜索 key 名称 / id / provider / 状态")}
                                />
                            </div>
                        </div>
                    </div>
                    if *loading && (*keys).is_empty() {
                        <div class={classes!("skeleton", "px-4", "py-4")}>
                            <i></i><i></i><i></i><i></i>
                        </div>
                    } else if (*keys).is_empty() {
                        <div class={classes!("empty")}>
                            <span>{ "还没有 Kiro key" }</span>
                            <span class={classes!("text-xs")}>{ "先创建一个，然后把 base URL 和 key 发给 Claude Code 或 Anthropic SDK 使用。" }</span>
                        </div>
                    } else if filtered_keys.is_empty() {
                        <div class={classes!("empty")}>
                            <span>{ "当前过滤条件下没有匹配的 Kiro key" }</span>
                        </div>
                    } else {
                        <div class={classes!("grid", "gap-4", "p-4", "xl:grid-cols-2", "items-start")}>
                            { for filtered_keys.iter().map(|key_item| html! {
                                <KiroKeyEditorCard
                                    key={key_item.id.clone()}
                                    key_item={key_item.clone()}
                                    persisted_global_policy_form={global_defaults.policy_form.clone()}
                                    persisted_global_billable_multiplier_json={global_defaults.billable_multiplier_json.clone()}
                                    available_models={(*kiro_models).clone()}
                                    account_groups={(*account_group_options).clone()}
                                    anthropic_channels={(*anthropic_channels).clone()}
                                    on_reload={on_reload.clone()}
                                    on_copy={on_copy.clone()}
                                    on_flash={notify.clone()}
                                />
                            }) }
                        </div>
                        <div class={classes!("pager", "px-4", "pb-3", "flex-wrap")}>
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
    use super::key_matches_query;
    use crate::api::AdminLlmGatewayKeyView;

    fn key(name: &str, id: &str, provider: &str, status: &str) -> AdminLlmGatewayKeyView {
        AdminLlmGatewayKeyView {
            name: name.to_string(),
            id: id.to_string(),
            provider_type: provider.to_string(),
            status: status.to_string(),
            ..AdminLlmGatewayKeyView::default()
        }
    }

    #[test]
    fn key_query_matches_name_id_provider_and_status_case_insensitively() {
        let sample = key("Primary", "sfk-abc", "kiro", "active");

        assert!(key_matches_query(&sample, "primary"));
        assert!(key_matches_query(&sample, "sfk-abc"));
        assert!(key_matches_query(&sample, "kiro"));
        assert!(key_matches_query(&sample, "active"));
        assert!(!key_matches_query(&sample, "missing"));
    }
}
