use std::collections::BTreeMap;

use llm_access_core::store as llm_store;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    api::{
        create_admin_anthropic_upstream_channel, delete_admin_anthropic_upstream_channel,
        fetch_admin_anthropic_upstream_channels, fetch_admin_llm_gateway_proxy_configs,
        patch_admin_anthropic_upstream_channel, refresh_admin_anthropic_upstream_models,
        test_admin_anthropic_upstream_model, AdminAnthropicUpstreamChannelView,
        AdminUpstreamProxyConfigView, CreateAdminAnthropicUpstreamChannelInput,
        PatchAdminAnthropicUpstreamChannelInput, TestAdminAnthropicUpstreamModelInput,
    },
    pages::llm_access_shared::{confirm_destructive, format_number_u64, format_timestamp_opt},
    router::Route,
};

fn status_classes(status: &str) -> Classes {
    if status == "ok" || status == "active" {
        classes!(
            "rounded-full",
            "bg-emerald-500/10",
            "px-2",
            "py-1",
            "font-mono",
            "text-xs",
            "text-emerald-700",
            "dark:text-emerald-200"
        )
    } else if status == "unchecked" || status.is_empty() {
        classes!(
            "rounded-full",
            "border",
            "border-[var(--border)]",
            "px-2",
            "py-1",
            "font-mono",
            "text-xs",
            "text-[var(--muted)]"
        )
    } else {
        classes!(
            "rounded-full",
            "bg-amber-500/10",
            "px-2",
            "py-1",
            "font-mono",
            "text-xs",
            "text-amber-700",
            "dark:text-amber-200"
        )
    }
}

fn parse_proxy_choice(raw: &str) -> (String, Option<String>) {
    let trimmed = raw.trim();
    if trimmed == "direct" {
        ("direct".to_string(), None)
    } else if let Some(proxy_config_id) = trimmed.strip_prefix("fixed:") {
        ("fixed".to_string(), Some(proxy_config_id.to_string()))
    } else {
        ("inherit".to_string(), None)
    }
}

/// Build a full-field channel patch from the edit form's raw strings.
fn build_channel_patch(
    base_url: &str,
    weight: &str,
    max_concurrency: &str,
    min_start_interval_ms: &str,
    proxy_choice: &str,
) -> Result<PatchAdminAnthropicUpstreamChannelInput, String> {
    let base_url = base_url.trim();
    if base_url.is_empty() {
        return Err("Base URL must not be empty.".to_string());
    }
    let weight = weight
        .trim()
        .parse::<u64>()
        .map_err(|_| "Weight must be an integer.".to_string())?;
    let max_concurrency = max_concurrency
        .trim()
        .parse::<u64>()
        .map_err(|_| "Concurrency must be an integer.".to_string())?;
    let min_start_interval_ms = min_start_interval_ms
        .trim()
        .parse::<u64>()
        .map_err(|_| "Min interval must be an integer.".to_string())?;
    let (proxy_mode, proxy_config_id) = parse_proxy_choice(proxy_choice);
    Ok(PatchAdminAnthropicUpstreamChannelInput {
        base_url: Some(base_url.to_string()),
        weight: Some(weight),
        max_concurrency: Some(max_concurrency),
        min_start_interval_ms: Some(min_start_interval_ms),
        proxy_mode: Some(proxy_mode),
        proxy_config_id: Some(proxy_config_id),
        ..PatchAdminAnthropicUpstreamChannelInput::default()
    })
}

/// The `<select>` value that represents a channel's current proxy setting.
fn proxy_choice_for_channel(channel: &AdminAnthropicUpstreamChannelView) -> String {
    match (channel.proxy_mode.as_str(), channel.proxy_config_id.as_deref()) {
        ("fixed", Some(proxy_config_id)) => format!("fixed:{proxy_config_id}"),
        ("direct", _) => "direct".to_string(),
        _ => "inherit".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_channel_patch, proxy_choice_for_channel};
    use crate::api::AdminAnthropicUpstreamChannelView;

    #[test]
    fn build_channel_patch_parses_fields_and_fixed_proxy() {
        let patch =
            build_channel_patch(" https://api.anthropic.com ", "50", "2", "250", "fixed:proxy-1")
                .expect("patch should build");

        assert_eq!(patch.base_url.as_deref(), Some("https://api.anthropic.com"));
        assert_eq!(patch.weight, Some(50));
        assert_eq!(patch.max_concurrency, Some(2));
        assert_eq!(patch.min_start_interval_ms, Some(250));
        assert_eq!(patch.proxy_mode.as_deref(), Some("fixed"));
        assert_eq!(patch.proxy_config_id, Some(Some("proxy-1".to_string())));
        assert_eq!(patch.status, None);
        assert_eq!(patch.api_key, None);
        assert!(!patch.clear_last_error);
    }

    #[test]
    fn build_channel_patch_clears_proxy_binding_for_inherit() {
        let patch = build_channel_patch("https://api.anthropic.com", "100", "3", "0", "inherit")
            .expect("patch should build");

        assert_eq!(patch.proxy_mode.as_deref(), Some("inherit"));
        assert_eq!(patch.proxy_config_id, Some(None));
    }

    #[test]
    fn build_channel_patch_rejects_invalid_numbers_and_empty_base_url() {
        assert!(build_channel_patch("https://x.dev", "abc", "3", "0", "direct").is_err());
        assert!(build_channel_patch("https://x.dev", "1", "-2", "0", "direct").is_err());
        assert!(build_channel_patch("  ", "1", "3", "0", "direct").is_err());
    }

    #[test]
    fn proxy_choice_round_trips_channel_proxy_setting() {
        let mut channel = AdminAnthropicUpstreamChannelView::default();
        assert_eq!(proxy_choice_for_channel(&channel), "inherit");

        channel.proxy_mode = "direct".to_string();
        assert_eq!(proxy_choice_for_channel(&channel), "direct");

        channel.proxy_mode = "fixed".to_string();
        channel.proxy_config_id = Some("proxy-9".to_string());
        assert_eq!(proxy_choice_for_channel(&channel), "fixed:proxy-9");
    }
}

fn total_input(channel: &AdminAnthropicUpstreamChannelView) -> u64 {
    channel
        .usage
        .input_uncached_tokens
        .saturating_add(channel.usage.input_cached_tokens)
}

#[function_component(AdminKiroAnthropicUpstreamsPage)]
pub fn admin_kiro_anthropic_upstreams_page() -> Html {
    let channels = use_state(Vec::<AdminAnthropicUpstreamChannelView>::new);
    let proxy_configs = use_state(Vec::<AdminUpstreamProxyConfigView>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let flash = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0u64);

    let name = use_state(String::new);
    let base_url = use_state(|| llm_store::DEFAULT_ANTHROPIC_UPSTREAM_BASE_URL.to_string());
    let api_key = use_state(String::new);
    let weight = use_state(|| llm_store::DEFAULT_ANTHROPIC_UPSTREAM_WEIGHT.to_string());
    let max_concurrency =
        use_state(|| llm_store::DEFAULT_ANTHROPIC_UPSTREAM_MAX_CONCURRENCY.to_string());
    let min_start_interval_ms =
        use_state(|| llm_store::DEFAULT_ANTHROPIC_UPSTREAM_MIN_START_INTERVAL_MS.to_string());
    let proxy_mode = use_state(|| "inherit".to_string());
    let saving = use_state(|| false);
    let refreshing_channel = use_state(|| None::<String>);
    let testing_channel = use_state(|| None::<String>);
    let selected_models = use_state(BTreeMap::<String, String>::new);
    let editing_channel = use_state(|| None::<String>);
    let edit_base_url = use_state(String::new);
    let edit_weight = use_state(String::new);
    let edit_max_concurrency = use_state(String::new);
    let edit_min_start_interval_ms = use_state(String::new);
    let edit_proxy_choice = use_state(|| "inherit".to_string());
    let edit_saving = use_state(|| false);

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

    let reload = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_| refresh_tick.set((*refresh_tick).saturating_add(1)))
    };

    {
        let channels = channels.clone();
        let proxy_configs = proxy_configs.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let channels = channels.clone();
            let proxy_configs = proxy_configs.clone();
            let loading = loading.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let (channels_result, proxy_configs_result) = futures::join!(
                    fetch_admin_anthropic_upstream_channels(),
                    fetch_admin_llm_gateway_proxy_configs()
                );
                match (channels_result, proxy_configs_result) {
                    (Ok(channel_resp), Ok(proxy_resp)) => {
                        channels.set(channel_resp.channels);
                        proxy_configs.set(proxy_resp.proxy_configs);
                        error.set(None);
                    },
                    (Err(err), _) | (_, Err(err)) => error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_create = {
        let name = name.clone();
        let base_url = base_url.clone();
        let api_key = api_key.clone();
        let weight = weight.clone();
        let max_concurrency = max_concurrency.clone();
        let min_start_interval_ms = min_start_interval_ms.clone();
        let proxy_mode = proxy_mode.clone();
        let saving = saving.clone();
        let notify = notify.clone();
        let reload = reload.clone();
        Callback::from(move |_| {
            if *saving {
                return;
            }
            let name_value = (*name).trim().to_string();
            let base_url_value = (*base_url).trim().to_string();
            let api_key_value = (*api_key).trim().to_string();
            let weight_value = (*weight).trim().parse::<u64>();
            let max_value = (*max_concurrency).trim().parse::<u64>();
            let min_value = (*min_start_interval_ms).trim().parse::<u64>();
            let proxy_choice = (*proxy_mode).clone();
            let name = name.clone();
            let api_key = api_key.clone();
            let saving = saving.clone();
            let notify = notify.clone();
            let reload = reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(weight_value) = weight_value else {
                    notify.emit(("Weight must be an integer.".to_string(), true));
                    return;
                };
                let Ok(max_value) = max_value else {
                    notify.emit(("Concurrency must be an integer.".to_string(), true));
                    return;
                };
                let Ok(min_value) = min_value else {
                    notify.emit(("Min interval must be an integer.".to_string(), true));
                    return;
                };
                let (proxy_mode, proxy_config_id) = parse_proxy_choice(&proxy_choice);
                saving.set(true);
                let input = CreateAdminAnthropicUpstreamChannelInput {
                    name: name_value,
                    base_url: base_url_value,
                    api_key: api_key_value,
                    status: Some("active".to_string()),
                    weight: Some(weight_value),
                    max_concurrency: Some(max_value),
                    min_start_interval_ms: Some(min_value),
                    proxy_mode: Some(proxy_mode),
                    proxy_config_id,
                };
                match create_admin_anthropic_upstream_channel(&input).await {
                    Ok(channel) => {
                        name.set(String::new());
                        api_key.set(String::new());
                        notify.emit((format!("Created `{}`.", channel.name), false));
                        reload.emit(());
                    },
                    Err(err) => notify.emit((format!("Create failed.\n{err}"), true)),
                }
                saving.set(false);
            });
        })
    };

    let total_billable = channels
        .iter()
        .fold(0u64, |sum, channel| sum.saturating_add(channel.usage.billable_tokens));
    let total_tokens = channels.iter().fold(0u64, |sum, channel| {
        sum.saturating_add(total_input(channel))
            .saturating_add(channel.usage.output_tokens)
    });
    let active_channels = channels
        .iter()
        .filter(|channel| channel.status == "active")
        .count();

    html! {
        <main class={classes!("min-h-screen", "bg-[var(--bg)]", "px-4", "py-8", "lg:px-6", "lg:py-10")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-start", "justify-between", "gap-4", "flex-wrap")}>
                        <div>
                            <div class={classes!("font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Kiro / Anthropic" }</div>
                            <h1 class={classes!("mt-1", "mb-0", "font-mono", "text-xl", "font-bold", "text-[var(--text)]")}>{ "Upstream Channels" }</h1>
                        </div>
                        <div class={classes!("flex", "gap-2", "flex-wrap")}>
                            <Link<Route> to={Route::AdminKiroGateway} classes={classes!("btn-terminal")}>{ "Kiro Overview" }</Link<Route>>
                            <button
                                type="button"
                                class={classes!("btn-terminal", "btn-terminal-primary")}
                                disabled={*loading}
                                onclick={{
                                    let reload = reload.clone();
                                    Callback::from(move |_| reload.emit(()))
                                }}
                            >
                                { if *loading { "Loading..." } else { "Refresh" } }
                            </button>
                        </div>
                    </div>
                    <div class={classes!("mt-4", "grid", "gap-3", "sm:grid-cols-3")}>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Active / Total" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format!("{active_channels} / {}", channels.len()) }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Tokens" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format_number_u64(total_tokens) }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Billable" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format_number_u64(total_billable) }</div>
                        </div>
                    </div>
                    if let Some(message) = (*flash).clone() {
                        <div class={classes!("mt-4", "rounded-lg", "bg-emerald-500/10", "px-3", "py-2", "text-sm", "text-emerald-700", "dark:text-emerald-200")}>{ message }</div>
                    }
                    if let Some(err) = (*error).clone() {
                        <div class={classes!("mt-4", "rounded-lg", "bg-red-500/10", "px-3", "py-2", "text-sm", "text-red-700", "dark:text-red-200")}>{ err }</div>
                    }
                </section>

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("grid", "gap-3", "lg:grid-cols-8")}>
                        <label class={classes!("block", "text-sm")}>
                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Name" }</div>
                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*name).clone()} oninput={{
                                let name = name.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    name.set(input.value());
                                })
                            }} />
                        </label>
                        <label class={classes!("block", "text-sm", "lg:col-span-2")}>
                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Base URL" }</div>
                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*base_url).clone()} oninput={{
                                let base_url = base_url.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    base_url.set(input.value());
                                })
                            }} />
                        </label>
                        <label class={classes!("block", "text-sm", "lg:col-span-2")}>
                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "API Key" }</div>
                            <input type="password" class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*api_key).clone()} oninput={{
                                let api_key = api_key.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    api_key.set(input.value());
                                })
                            }} />
                        </label>
                        <label class={classes!("block", "text-sm")}>
                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Proxy" }</div>
                            <select class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "text-sm")} value={(*proxy_mode).clone()} onchange={{
                                let proxy_mode = proxy_mode.clone();
                                Callback::from(move |event: Event| {
                                    let input: HtmlSelectElement = event.target_unchecked_into();
                                    proxy_mode.set(input.value());
                                })
                            }}>
                                <option value="inherit">{ "Inherit" }</option>
                                <option value="direct">{ "Direct" }</option>
                                { for proxy_configs.iter().map(|proxy_config| {
                                    let value = format!("fixed:{}", proxy_config.id);
                                    html! { <option value={value}>{ format!("Fixed · {}", proxy_config.name) }</option> }
                                }) }
                            </select>
                        </label>
                        <label class={classes!("block", "text-sm")}>
                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Weight" }</div>
                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*weight).clone()} oninput={{
                                let weight = weight.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    weight.set(input.value());
                                })
                            }} />
                        </label>
                        <label class={classes!("block", "text-sm")}>
                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Concurrency" }</div>
                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*max_concurrency).clone()} oninput={{
                                let max_concurrency = max_concurrency.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    max_concurrency.set(input.value());
                                })
                            }} />
                        </label>
                        <label class={classes!("block", "text-sm")}>
                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Min ms" }</div>
                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*min_start_interval_ms).clone()} oninput={{
                                let min_start_interval_ms = min_start_interval_ms.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    min_start_interval_ms.set(input.value());
                                })
                            }} />
                        </label>
                        <div class={classes!("flex", "items-end")}>
                            <button type="button" class={classes!("btn-terminal", "btn-terminal-primary", "w-full")} disabled={*saving} onclick={on_create}>
                                { if *saving { "Creating..." } else { "Create" } }
                            </button>
                        </div>
                    </div>
                </section>

                <section class={classes!("overflow-x-auto", "rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]")}>
                    <div class={classes!("grid", "min-w-[74rem]", "grid-cols-[1.2fr_1fr_1.1fr_1.1fr_1.3fr]", "gap-0", "border-b", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-4", "py-2", "font-mono", "text-[11px]", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>
                        <div>{ "Channel" }</div>
                        <div>{ "Usage" }</div>
                        <div>{ "Models" }</div>
                        <div>{ "Last Test" }</div>
                        <div>{ "Actions" }</div>
                    </div>
                    { for channels.iter().map(|channel| {
                        let channel_name = channel.name.clone();
                        let selected_model = selected_models
                            .get(&channel_name)
                            .cloned()
                            .filter(|value| channel.models.iter().any(|model| model == value))
                            .or_else(|| channel.models.first().cloned())
                            .unwrap_or_default();
                        let models_status = channel.last_models_status.clone().unwrap_or_else(|| "unchecked".to_string());
                        let test_status = channel.last_test_status.clone().unwrap_or_else(|| "unchecked".to_string());
                        let is_refreshing = (*refreshing_channel).as_ref().is_some_and(|name| name == &channel_name);
                        let is_testing = (*testing_channel).as_ref().is_some_and(|name| name == &channel_name);
                        let on_select_model = {
                            let selected_models = selected_models.clone();
                            let channel_name = channel_name.clone();
                            Callback::from(move |event: Event| {
                                let select: HtmlSelectElement = event.target_unchecked_into();
                                let mut next = (*selected_models).clone();
                                next.insert(channel_name.clone(), select.value());
                                selected_models.set(next);
                            })
                        };
                        let on_refresh_models = {
                            let notify = notify.clone();
                            let reload = reload.clone();
                            let refreshing_channel = refreshing_channel.clone();
                            let channel_name = channel_name.clone();
                            Callback::from(move |_| {
                                if (*refreshing_channel).is_some() {
                                    return;
                                }
                                refreshing_channel.set(Some(channel_name.clone()));
                                let notify = notify.clone();
                                let reload = reload.clone();
                                let refreshing_channel = refreshing_channel.clone();
                                let channel_name = channel_name.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    match refresh_admin_anthropic_upstream_models(&channel_name).await {
                                        Ok(response) => {
                                            notify.emit((format!("Refreshed `{channel_name}`: {}.", response.status), !response.ok));
                                            reload.emit(());
                                        },
                                        Err(err) => notify.emit((format!("Refresh `{channel_name}` failed.\n{err}"), true)),
                                    }
                                    refreshing_channel.set(None);
                                });
                            })
                        };
                        let on_test_model = {
                            let notify = notify.clone();
                            let reload = reload.clone();
                            let testing_channel = testing_channel.clone();
                            let channel_name = channel_name.clone();
                            let model = selected_model.clone();
                            Callback::from(move |_| {
                                if (*testing_channel).is_some() {
                                    return;
                                }
                                let model = model.trim().to_string();
                                if model.is_empty() {
                                    notify.emit(("Select a model before testing.".to_string(), true));
                                    return;
                                }
                                testing_channel.set(Some(channel_name.clone()));
                                let notify = notify.clone();
                                let reload = reload.clone();
                                let testing_channel = testing_channel.clone();
                                let channel_name = channel_name.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let input = TestAdminAnthropicUpstreamModelInput { model: model.clone() };
                                    match test_admin_anthropic_upstream_model(&channel_name, &input).await {
                                        Ok(response) => {
                                            notify.emit((format!("Tested `{channel_name}` / `{model}`: {} ms.", response.latency_ms), !response.ok));
                                            reload.emit(());
                                        },
                                        Err(err) => notify.emit((format!("Test `{channel_name}` / `{model}` failed.\n{err}"), true)),
                                    }
                                    testing_channel.set(None);
                                });
                            })
                        };
                        let on_toggle = {
                            let notify = notify.clone();
                            let reload = reload.clone();
                            let channel_name = channel_name.clone();
                            let next_status = if channel.status == "active" { "disabled" } else { "active" }.to_string();
                            Callback::from(move |_| {
                                let notify = notify.clone();
                                let reload = reload.clone();
                                let channel_name = channel_name.clone();
                                let next_status = next_status.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let input = PatchAdminAnthropicUpstreamChannelInput {
                                        status: Some(next_status),
                                        ..PatchAdminAnthropicUpstreamChannelInput::default()
                                    };
                                    match patch_admin_anthropic_upstream_channel(&channel_name, &input).await {
                                        Ok(_) => {
                                            notify.emit((format!("Updated `{channel_name}`."), false));
                                            reload.emit(());
                                        },
                                        Err(err) => notify.emit((format!("Update `{channel_name}` failed.\n{err}"), true)),
                                    }
                                });
                            })
                        };
                        let on_rotate_key = {
                            let notify = notify.clone();
                            let reload = reload.clone();
                            let channel_name = channel_name.clone();
                            Callback::from(move |_| {
                                let Some(window) = web_sys::window() else {
                                    notify.emit(("Browser window is unavailable.".to_string(), true));
                                    return;
                                };
                                let prompt = format!("New API key for `{channel_name}`");
                                let Ok(Some(api_key)) = window.prompt_with_message(&prompt) else {
                                    return;
                                };
                                let api_key = api_key.trim().to_string();
                                if api_key.is_empty() {
                                    notify.emit(("API key must not be empty.".to_string(), true));
                                    return;
                                }
                                let notify = notify.clone();
                                let reload = reload.clone();
                                let channel_name = channel_name.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let input = PatchAdminAnthropicUpstreamChannelInput {
                                        api_key: Some(api_key),
                                        ..PatchAdminAnthropicUpstreamChannelInput::default()
                                    };
                                    match patch_admin_anthropic_upstream_channel(&channel_name, &input).await {
                                        Ok(_) => {
                                            notify.emit((format!("Rotated key for `{channel_name}`."), false));
                                            reload.emit(());
                                        },
                                        Err(err) => notify.emit((format!("Rotate `{channel_name}` failed.\n{err}"), true)),
                                    }
                                });
                            })
                        };
                        let on_delete = {
                            let notify = notify.clone();
                            let reload = reload.clone();
                            let channel_name = channel_name.clone();
                            Callback::from(move |_| {
                                if !confirm_destructive(&format!("Delete `{channel_name}`?")) {
                                    return;
                                }
                                let notify = notify.clone();
                                let reload = reload.clone();
                                let channel_name = channel_name.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    match delete_admin_anthropic_upstream_channel(&channel_name).await {
                                        Ok(_) => {
                                            notify.emit((format!("Deleted `{channel_name}`."), false));
                                            reload.emit(());
                                        },
                                        Err(err) => notify.emit((format!("Delete `{channel_name}` failed.\n{err}"), true)),
                                    }
                                });
                            })
                        };
                        let is_editing = (*editing_channel).as_ref().is_some_and(|name| name == &channel_name);
                        let on_toggle_edit = {
                            let editing_channel = editing_channel.clone();
                            let edit_base_url = edit_base_url.clone();
                            let edit_weight = edit_weight.clone();
                            let edit_max_concurrency = edit_max_concurrency.clone();
                            let edit_min_start_interval_ms = edit_min_start_interval_ms.clone();
                            let edit_proxy_choice = edit_proxy_choice.clone();
                            let channel_name = channel_name.clone();
                            let channel_base_url = channel.base_url.clone();
                            let channel_weight = channel.weight.to_string();
                            let channel_max_concurrency = channel.max_concurrency.to_string();
                            let channel_min_start_interval_ms = channel.min_start_interval_ms.to_string();
                            let channel_proxy_choice = proxy_choice_for_channel(channel);
                            Callback::from(move |_| {
                                if (*editing_channel).as_ref().is_some_and(|name| name == &channel_name) {
                                    editing_channel.set(None);
                                    return;
                                }
                                edit_base_url.set(channel_base_url.clone());
                                edit_weight.set(channel_weight.clone());
                                edit_max_concurrency.set(channel_max_concurrency.clone());
                                edit_min_start_interval_ms.set(channel_min_start_interval_ms.clone());
                                edit_proxy_choice.set(channel_proxy_choice.clone());
                                editing_channel.set(Some(channel_name.clone()));
                            })
                        };
                        let on_save_edit = {
                            let notify = notify.clone();
                            let reload = reload.clone();
                            let editing_channel = editing_channel.clone();
                            let edit_base_url = edit_base_url.clone();
                            let edit_weight = edit_weight.clone();
                            let edit_max_concurrency = edit_max_concurrency.clone();
                            let edit_min_start_interval_ms = edit_min_start_interval_ms.clone();
                            let edit_proxy_choice = edit_proxy_choice.clone();
                            let edit_saving = edit_saving.clone();
                            let channel_name = channel_name.clone();
                            Callback::from(move |_| {
                                if *edit_saving {
                                    return;
                                }
                                let patch = match build_channel_patch(
                                    &edit_base_url,
                                    &edit_weight,
                                    &edit_max_concurrency,
                                    &edit_min_start_interval_ms,
                                    &edit_proxy_choice,
                                ) {
                                    Ok(patch) => patch,
                                    Err(message) => {
                                        notify.emit((message, true));
                                        return;
                                    },
                                };
                                edit_saving.set(true);
                                let notify = notify.clone();
                                let reload = reload.clone();
                                let editing_channel = editing_channel.clone();
                                let edit_saving = edit_saving.clone();
                                let channel_name = channel_name.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    match patch_admin_anthropic_upstream_channel(&channel_name, &patch).await {
                                        Ok(_) => {
                                            notify.emit((format!("Updated `{channel_name}`."), false));
                                            editing_channel.set(None);
                                            reload.emit(());
                                        },
                                        Err(err) => notify.emit((format!("Update `{channel_name}` failed.\n{err}"), true)),
                                    }
                                    edit_saving.set(false);
                                });
                            })
                        };
                        let on_clear_error = {
                            let notify = notify.clone();
                            let reload = reload.clone();
                            let channel_name = channel_name.clone();
                            Callback::from(move |_| {
                                let notify = notify.clone();
                                let reload = reload.clone();
                                let channel_name = channel_name.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    let input = PatchAdminAnthropicUpstreamChannelInput {
                                        clear_last_error: true,
                                        ..PatchAdminAnthropicUpstreamChannelInput::default()
                                    };
                                    match patch_admin_anthropic_upstream_channel(&channel_name, &input).await {
                                        Ok(_) => {
                                            notify.emit((format!("Cleared error for `{channel_name}`."), false));
                                            reload.emit(());
                                        },
                                        Err(err) => notify.emit((format!("Clear `{channel_name}` failed.\n{err}"), true)),
                                    }
                                });
                            })
                        };
                        html! {
                            <>
                            <div class={classes!("grid", "min-w-[74rem]", "grid-cols-[1.2fr_1fr_1.1fr_1.1fr_1.3fr]", "gap-0", "border-b", "border-[var(--border)]", "px-4", "py-3", "text-sm", "last:border-b-0")}>
                                <div class={classes!("min-w-0", "pr-4")}>
                                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                        <span class={classes!("font-mono", "font-semibold", "text-[var(--text)]")}>{ channel.name.clone() }</span>
                                        <span class={status_classes(&channel.status)}>{ channel.status.clone() }</span>
                                    </div>
                                    <div class={classes!("mt-1", "break-all", "font-mono", "text-xs", "text-[var(--muted)]")}>{ channel.base_url.clone() }</div>
                                    <div class={classes!("mt-1", "font-mono", "text-xs", "text-[var(--muted)]")}>
                                        { format!("w={} · c={} · min={}ms · proxy={}", channel.weight, channel.max_concurrency, channel.min_start_interval_ms, channel.proxy_mode) }
                                    </div>
                                </div>
                                <div class={classes!("font-mono", "text-xs", "space-y-1")}>
                                    <div>{ format!("input {}", format_number_u64(total_input(channel))) }</div>
                                    <div>{ format!("cached {}", format_number_u64(channel.usage.input_cached_tokens)) }</div>
                                    <div>{ format!("output {}", format_number_u64(channel.usage.output_tokens)) }</div>
                                    <div class={classes!("font-semibold", "text-[var(--text)]")}>{ format!("billable {}", format_number_u64(channel.usage.billable_tokens)) }</div>
                                    <div class={classes!("text-[var(--muted)]")}>{ format!("missing {} · {}", channel.usage.usage_missing_events, format_timestamp_opt(channel.usage.last_used_at)) }</div>
                                </div>
                                <div class={classes!("min-w-0", "pr-3", "font-mono", "text-xs", "space-y-2")}>
                                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                        <span class={status_classes(&models_status)}>{ models_status }</span>
                                        <span class={classes!("text-[var(--muted)]")}>{ format!("{} models", channel.models.len()) }</span>
                                    </div>
                                    <div class={classes!("text-[var(--muted)]")}>
                                        { format!("{} · {}", channel.last_models_latency_ms.map(|value| format!("{value}ms")).unwrap_or_else(|| "-".to_string()), format_timestamp_opt(channel.last_models_checked_at)) }
                                    </div>
                                    if let Some(error) = channel.last_models_error.as_deref() {
                                        <div class={classes!("break-words", "text-amber-700", "dark:text-amber-200")}>{ error }</div>
                                    }
                                </div>
                                <div class={classes!("min-w-0", "pr-3", "font-mono", "text-xs", "space-y-2")}>
                                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                        <span class={status_classes(&test_status)}>{ test_status }</span>
                                        <span class={classes!("text-[var(--muted)]")}>{ channel.last_test_model.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class={classes!("text-[var(--muted)]")}>
                                        { format!("{} · {}", channel.last_test_latency_ms.map(|value| format!("{value}ms")).unwrap_or_else(|| "-".to_string()), format_timestamp_opt(channel.last_test_at)) }
                                    </div>
                                    if let Some(error) = channel.last_test_error.as_deref() {
                                        <div class={classes!("break-words", "text-amber-700", "dark:text-amber-200")}>{ error }</div>
                                    }
                                </div>
                                <div class={classes!("space-y-2")}>
                                    <div class={classes!("flex", "gap-2", "flex-wrap")}>
                                        <button type="button" class={classes!("btn-terminal", "text-xs")} disabled={is_refreshing} onclick={on_refresh_models}>{ if is_refreshing { "Refreshing..." } else { "Refresh Status" } }</button>
                                        <button type="button" class={classes!("btn-terminal", "text-xs")} onclick={on_toggle_edit.clone()}>{ if is_editing { "Close" } else { "Edit" } }</button>
                                        <button type="button" class={classes!("btn-terminal", "text-xs")} onclick={on_toggle}>{ if channel.status == "active" { "Disable" } else { "Enable" } }</button>
                                        <button type="button" class={classes!("btn-terminal", "text-xs")} onclick={on_rotate_key}>{ "Rotate" }</button>
                                        <button type="button" class={classes!("btn-terminal", "text-xs")} onclick={on_delete}>{ "Delete" }</button>
                                    </div>
                                    <div class={classes!("flex", "items-center", "gap-2")}>
                                        <select class={classes!("min-w-0", "flex-1", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-2", "py-2", "font-mono", "text-xs")} value={selected_model.clone()} disabled={channel.models.is_empty() || is_testing} onchange={on_select_model}>
                                            {
                                                if channel.models.is_empty() {
                                                    html! { <option value="">{ "Refresh to select model" }</option> }
                                                } else {
                                                    html! {
                                                        for channel.models.iter().map(|model| html! {
                                                            <option value={model.clone()}>{ model.clone() }</option>
                                                        })
                                                    }
                                                }
                                            }
                                        </select>
                                        <button type="button" class={classes!("btn-terminal", "btn-terminal-primary", "text-xs")} disabled={channel.models.is_empty() || is_testing} onclick={on_test_model}>
                                            { if is_testing { "Testing..." } else { "Test Model" } }
                                        </button>
                                    </div>
                                    if let Some(error) = channel.last_error.as_deref() {
                                        <div class={classes!("flex", "items-start", "gap-2")}>
                                            <div class={classes!("min-w-0", "break-words", "font-mono", "text-xs", "text-red-700", "dark:text-red-200")}>{ error }</div>
                                            <button type="button" class={classes!("btn-terminal", "text-xs", "shrink-0")} onclick={on_clear_error}>{ "Clear" }</button>
                                        </div>
                                    }
                                </div>
                            </div>
                            if is_editing {
                                <div class={classes!("border-b", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-4", "py-3")}>
                                    <div class={classes!("grid", "min-w-[74rem]", "gap-3", "lg:grid-cols-7")}>
                                        <label class={classes!("block", "text-sm", "lg:col-span-2")}>
                                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Base URL" }</div>
                                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*edit_base_url).clone()} oninput={{
                                                let edit_base_url = edit_base_url.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_base_url.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("block", "text-sm")}>
                                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Weight" }</div>
                                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*edit_weight).clone()} oninput={{
                                                let edit_weight = edit_weight.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_weight.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("block", "text-sm")}>
                                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Concurrency" }</div>
                                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*edit_max_concurrency).clone()} oninput={{
                                                let edit_max_concurrency = edit_max_concurrency.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_max_concurrency.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("block", "text-sm")}>
                                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Min ms" }</div>
                                            <input class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-sm")} value={(*edit_min_start_interval_ms).clone()} oninput={{
                                                let edit_min_start_interval_ms = edit_min_start_interval_ms.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_min_start_interval_ms.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("block", "text-sm")}>
                                            <div class={classes!("mb-1", "font-mono", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Proxy" }</div>
                                            <select class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm")} value={(*edit_proxy_choice).clone()} onchange={{
                                                let edit_proxy_choice = edit_proxy_choice.clone();
                                                Callback::from(move |event: Event| {
                                                    let input: HtmlSelectElement = event.target_unchecked_into();
                                                    edit_proxy_choice.set(input.value());
                                                })
                                            }}>
                                                <option value="inherit" selected={*edit_proxy_choice == "inherit"}>{ "Inherit" }</option>
                                                <option value="direct" selected={*edit_proxy_choice == "direct"}>{ "Direct" }</option>
                                                { for proxy_configs.iter().map(|proxy_config| {
                                                    let value = format!("fixed:{}", proxy_config.id);
                                                    let selected = *edit_proxy_choice == value;
                                                    html! { <option value={value} selected={selected}>{ format!("Fixed · {}", proxy_config.name) }</option> }
                                                }) }
                                            </select>
                                        </label>
                                        <div class={classes!("flex", "items-end", "gap-2")}>
                                            <button type="button" class={classes!("btn-terminal", "btn-terminal-primary", "w-full")} disabled={*edit_saving} onclick={on_save_edit}>
                                                { if *edit_saving { "Saving..." } else { "Save" } }
                                            </button>
                                            <button type="button" class={classes!("btn-terminal", "w-full")} onclick={on_toggle_edit}>{ "Cancel" }</button>
                                        </div>
                                    </div>
                                </div>
                            }
                            </>
                        }
                    }) }
                    if channels.is_empty() && !*loading {
                        <div class={classes!("px-4", "py-8", "text-sm", "text-[var(--muted)]")}>{ "No Anthropic upstream channels configured." }</div>
                    }
                </section>
            </div>
        </main>
    }
}
