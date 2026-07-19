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
        classes!("badge", "ok")
    } else if status == "unchecked" || status.is_empty() {
        classes!("badge")
    } else {
        classes!("badge", "warn")
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

fn parse_cache_hit_rate_limits(
    raw: &str,
) -> Result<Vec<llm_store::AnthropicCacheHitRateLimit>, String> {
    let mut limits = Vec::new();
    for raw_rule in raw.split([',', '\n']) {
        let rule = raw_rule.trim();
        if rule.is_empty() {
            continue;
        }
        let Some((threshold, rate)) = rule.split_once(':') else {
            return Err(format!("Cache cap rule `{rule}` must use context:percent."));
        };
        let min_context_tokens = threshold
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("Cache cap context `{threshold}` must be an integer."))?;
        let rate = rate.trim().trim_end_matches('%').trim();
        let percent = rate
            .parse::<f64>()
            .map_err(|_| format!("Cache cap rate `{rate}` must be a percentage."))?;
        if !percent.is_finite() || !(0.0..=100.0).contains(&percent) {
            return Err(format!("Cache cap rate `{rate}` must be between 0 and 100."));
        }
        let basis_points = percent * 100.0;
        if (basis_points - basis_points.round()).abs() > 1e-9 {
            return Err(format!("Cache cap rate `{rate}` supports at most two decimals."));
        }
        limits.push(llm_store::AnthropicCacheHitRateLimit {
            min_context_tokens,
            max_cache_hit_rate_basis_points: basis_points.round() as u32,
        });
    }
    llm_store::validate_anthropic_cache_hit_rate_limits(&limits).map_err(|err| err.to_string())?;
    Ok(limits)
}

fn format_cache_hit_rate_limits(limits: &[llm_store::AnthropicCacheHitRateLimit]) -> String {
    limits
        .iter()
        .map(|limit| {
            let basis_points = limit.max_cache_hit_rate_basis_points;
            let rate = if basis_points % 100 == 0 {
                (basis_points / 100).to_string()
            } else if basis_points % 10 == 0 {
                format!("{}.{:01}", basis_points / 100, (basis_points % 100) / 10)
            } else {
                format!("{}.{:02}", basis_points / 100, basis_points % 100)
            };
            format!("{}:{rate}", limit.min_context_tokens)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Build a full-field channel patch from the edit form's raw strings.
fn build_channel_patch(
    base_url: &str,
    weight: &str,
    max_concurrency: &str,
    rpm_limit: &str,
    min_start_interval_ms: &str,
    cache_hit_rate_limits: &str,
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
    let rpm_limit = rpm_limit
        .trim()
        .parse::<u64>()
        .map_err(|_| "RPM must be an integer.".to_string())?;
    if rpm_limit == 0 {
        return Err("RPM must be greater than zero.".to_string());
    }
    let min_start_interval_ms = min_start_interval_ms
        .trim()
        .parse::<u64>()
        .map_err(|_| "Min interval must be an integer.".to_string())?;
    let cache_hit_rate_limits = parse_cache_hit_rate_limits(cache_hit_rate_limits)?;
    let (proxy_mode, proxy_config_id) = parse_proxy_choice(proxy_choice);
    Ok(PatchAdminAnthropicUpstreamChannelInput {
        base_url: Some(base_url.to_string()),
        weight: Some(weight),
        max_concurrency: Some(max_concurrency),
        rpm_limit: Some(rpm_limit),
        min_start_interval_ms: Some(min_start_interval_ms),
        cache_hit_rate_limits: Some(cache_hit_rate_limits),
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
    use super::{
        build_channel_patch, format_cache_hit_rate_limits, parse_cache_hit_rate_limits,
        proxy_choice_for_channel,
    };
    use crate::api::AdminAnthropicUpstreamChannelView;

    #[test]
    fn build_channel_patch_parses_fields_and_fixed_proxy() {
        let patch = build_channel_patch(
            " https://api.anthropic.com ",
            "50",
            "2",
            "5",
            "250",
            "0:90, 32000:70, 128000:40",
            "fixed:proxy-1",
        )
        .expect("patch should build");

        assert_eq!(patch.base_url.as_deref(), Some("https://api.anthropic.com"));
        assert_eq!(patch.weight, Some(50));
        assert_eq!(patch.max_concurrency, Some(2));
        assert_eq!(patch.rpm_limit, Some(5));
        assert_eq!(patch.min_start_interval_ms, Some(250));
        assert_eq!(patch.cache_hit_rate_limits.as_ref().map(Vec::len), Some(3));
        assert_eq!(patch.proxy_mode.as_deref(), Some("fixed"));
        assert_eq!(patch.proxy_config_id, Some(Some("proxy-1".to_string())));
        assert_eq!(patch.status, None);
        assert_eq!(patch.api_key, None);
        assert!(!patch.clear_last_error);
    }

    #[test]
    fn build_channel_patch_clears_proxy_binding_for_inherit() {
        let patch =
            build_channel_patch("https://api.anthropic.com", "100", "3", "5", "0", "", "inherit")
                .expect("patch should build");

        assert_eq!(patch.proxy_mode.as_deref(), Some("inherit"));
        assert_eq!(patch.proxy_config_id, Some(None));
    }

    #[test]
    fn build_channel_patch_rejects_invalid_numbers_and_empty_base_url() {
        assert!(build_channel_patch("https://x.dev", "abc", "3", "5", "0", "", "direct").is_err());
        assert!(build_channel_patch("https://x.dev", "1", "-2", "5", "0", "", "direct").is_err());
        assert!(build_channel_patch("https://x.dev", "1", "2", "0", "0", "", "direct").is_err());
        assert!(build_channel_patch("  ", "1", "3", "5", "0", "", "direct").is_err());
    }

    #[test]
    fn cache_hit_rate_limit_text_round_trips_and_rejects_increasing_rates() {
        let limits = parse_cache_hit_rate_limits("0:90, 32000:70.5, 128000:40.25%")
            .expect("cache cap rules should parse");

        assert_eq!(format_cache_hit_rate_limits(&limits), "0:90, 32000:70.5, 128000:40.25");
        assert!(parse_cache_hit_rate_limits("0:50, 32000:60").is_err());
        assert!(parse_cache_hit_rate_limits("0:80.123").is_err());
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
    let rpm_limit = use_state(|| llm_store::DEFAULT_ANTHROPIC_UPSTREAM_RPM_LIMIT.to_string());
    let min_start_interval_ms =
        use_state(|| llm_store::DEFAULT_ANTHROPIC_UPSTREAM_MIN_START_INTERVAL_MS.to_string());
    let cache_hit_rate_limits = use_state(String::new);
    let proxy_mode = use_state(|| "inherit".to_string());
    let saving = use_state(|| false);
    let refreshing_channel = use_state(|| None::<String>);
    let testing_channel = use_state(|| None::<String>);
    let selected_models = use_state(BTreeMap::<String, String>::new);
    let editing_channel = use_state(|| None::<String>);
    let edit_base_url = use_state(String::new);
    let edit_weight = use_state(String::new);
    let edit_max_concurrency = use_state(String::new);
    let edit_rpm_limit = use_state(String::new);
    let edit_min_start_interval_ms = use_state(String::new);
    let edit_cache_hit_rate_limits = use_state(String::new);
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
        let rpm_limit = rpm_limit.clone();
        let min_start_interval_ms = min_start_interval_ms.clone();
        let cache_hit_rate_limits = cache_hit_rate_limits.clone();
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
            let rpm_value = (*rpm_limit).trim().parse::<u64>();
            let min_value = (*min_start_interval_ms).trim().parse::<u64>();
            let cache_limits_value = parse_cache_hit_rate_limits(&cache_hit_rate_limits);
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
                let Ok(rpm_value) = rpm_value else {
                    notify.emit(("RPM must be an integer.".to_string(), true));
                    return;
                };
                if rpm_value == 0 {
                    notify.emit(("RPM must be greater than zero.".to_string(), true));
                    return;
                }
                let Ok(min_value) = min_value else {
                    notify.emit(("Min interval must be an integer.".to_string(), true));
                    return;
                };
                let cache_hit_rate_limits = match cache_limits_value {
                    Ok(value) => value,
                    Err(message) => {
                        notify.emit((message, true));
                        return;
                    },
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
                    rpm_limit: Some(rpm_value),
                    min_start_interval_ms: Some(min_value),
                    cache_hit_rate_limits,
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
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "Kiro / Anthropic" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Upstream Channels" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminKiroGateway} classes={classes!("linkbtn")}>{ "Kiro Overview" }</Link<Route>>
                        <button
                            type="button"
                            class={classes!("primary")}
                            disabled={*loading}
                            onclick={{
                                let reload = reload.clone();
                                Callback::from(move |_| reload.emit(()))
                            }}
                        >
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
                        <div class={classes!("stat", (active_channels < channels.len()).then_some("warn"))}>
                            <span>{ "Active / Total" }</span>
                            <b>{ format!("{active_channels} / {}", channels.len()) }</b>
                        </div>
                        <div class={classes!("stat")}>
                            <span>{ "Tokens" }</span>
                            <b>{ format_number_u64(total_tokens) }</b>
                        </div>
                        <div class={classes!("stat")}>
                            <span>{ "Billable" }</span>
                            <b>{ format_number_u64(total_billable) }</b>
                        </div>
                    </div>
                </section>

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <h2>{ "New Channel" }</h2>
                        <span class={classes!("text-xs", "text-[var(--muted-foreground)]")}>{ "直连 Anthropic 渠道；创建后可编辑全部路由参数或单独轮换密钥。Cache hit caps 只限制额度/计费中的 cache-read 命中率，不改写 Anthropic 响应；空值表示不限制。" }</span>
                    </div>
                    <div class={classes!("panel-body")}>
                        <div class={classes!("grid", "gap-3", "items-end", "sm:grid-cols-2", "lg:grid-cols-10")}>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Name" }
                                <input value={(*name).clone()} oninput={{
                                    let name = name.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        name.set(input.value());
                                    })
                                }} />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]", "lg:col-span-2")}>
                                { "Base URL" }
                                <input class={classes!("mono")} value={(*base_url).clone()} oninput={{
                                    let base_url = base_url.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        base_url.set(input.value());
                                    })
                                }} />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]", "lg:col-span-2")}>
                                { "API Key" }
                                <input type="password" class={classes!("mono")} value={(*api_key).clone()} oninput={{
                                    let api_key = api_key.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        api_key.set(input.value());
                                    })
                                }} />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Proxy" }
                                <select value={(*proxy_mode).clone()} onchange={{
                                    let proxy_mode = proxy_mode.clone();
                                    Callback::from(move |event: Event| {
                                        let input: HtmlSelectElement = event.target_unchecked_into();
                                        proxy_mode.set(input.value());
                                    })
                                }}>
                                    <option value="inherit" selected={*proxy_mode == "inherit"}>{ "Inherit" }</option>
                                    <option value="direct" selected={*proxy_mode == "direct"}>{ "Direct" }</option>
                                    { for proxy_configs.iter().map(|proxy_config| {
                                        let value = format!("fixed:{}", proxy_config.id);
                                        let selected = *proxy_mode == value;
                                        html! { <option value={value} selected={selected}>{ format!("Fixed · {}", proxy_config.name) }</option> }
                                    }) }
                                </select>
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Weight" }
                                <input class={classes!("mono")} value={(*weight).clone()} oninput={{
                                    let weight = weight.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        weight.set(input.value());
                                    })
                                }} />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Concurrency" }
                                <input class={classes!("mono")} value={(*max_concurrency).clone()} oninput={{
                                    let max_concurrency = max_concurrency.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        max_concurrency.set(input.value());
                                    })
                                }} />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "RPM" }
                                <input type="number" min="1" class={classes!("mono")} value={(*rpm_limit).clone()} oninput={{
                                    let rpm_limit = rpm_limit.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        rpm_limit.set(input.value());
                                    })
                                }} />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Min ms" }
                                <input class={classes!("mono")} value={(*min_start_interval_ms).clone()} oninput={{
                                    let min_start_interval_ms = min_start_interval_ms.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        min_start_interval_ms.set(input.value());
                                    })
                                }} />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]", "lg:col-span-3")}>
                                { "Cache hit caps · context:percent" }
                                <input
                                    class={classes!("mono")}
                                    placeholder="0:90, 32000:70, 128000:40"
                                    value={(*cache_hit_rate_limits).clone()}
                                    oninput={{
                                        let cache_hit_rate_limits = cache_hit_rate_limits.clone();
                                        Callback::from(move |event: InputEvent| {
                                            let input: HtmlInputElement = event.target_unchecked_into();
                                            cache_hit_rate_limits.set(input.value());
                                        })
                                    }}
                                />
                            </label>
                            <button type="button" class={classes!("primary")} disabled={*saving} onclick={on_create}>
                                { if *saving { "Creating..." } else { "Create" } }
                            </button>
                        </div>
                    </div>
                </section>

                <section class={classes!("panel", "overflow-x-auto")}>
                    <div class={classes!("grid", "min-w-[74rem]", "grid-cols-[1.2fr_1fr_1.1fr_1.1fr_1.3fr]", "gap-0", "border-b", "border-[var(--border)]", "bg-[var(--card-2)]", "px-4", "py-2", "text-[11px]", "font-semibold", "uppercase", "tracking-[0.08em]", "text-[var(--muted-foreground)]")}>
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
                            let edit_rpm_limit = edit_rpm_limit.clone();
                            let edit_min_start_interval_ms = edit_min_start_interval_ms.clone();
                            let edit_cache_hit_rate_limits = edit_cache_hit_rate_limits.clone();
                            let edit_proxy_choice = edit_proxy_choice.clone();
                            let channel_name = channel_name.clone();
                            let channel_base_url = channel.base_url.clone();
                            let channel_weight = channel.weight.to_string();
                            let channel_max_concurrency = channel.max_concurrency.to_string();
                            let channel_rpm_limit = channel.rpm_limit.to_string();
                            let channel_min_start_interval_ms = channel.min_start_interval_ms.to_string();
                            let channel_cache_hit_rate_limits =
                                format_cache_hit_rate_limits(&channel.cache_hit_rate_limits);
                            let channel_proxy_choice = proxy_choice_for_channel(channel);
                            Callback::from(move |_| {
                                if (*editing_channel).as_ref().is_some_and(|name| name == &channel_name) {
                                    editing_channel.set(None);
                                    return;
                                }
                                edit_base_url.set(channel_base_url.clone());
                                edit_weight.set(channel_weight.clone());
                                edit_max_concurrency.set(channel_max_concurrency.clone());
                                edit_rpm_limit.set(channel_rpm_limit.clone());
                                edit_min_start_interval_ms.set(channel_min_start_interval_ms.clone());
                                edit_cache_hit_rate_limits.set(channel_cache_hit_rate_limits.clone());
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
                            let edit_rpm_limit = edit_rpm_limit.clone();
                            let edit_min_start_interval_ms = edit_min_start_interval_ms.clone();
                            let edit_cache_hit_rate_limits = edit_cache_hit_rate_limits.clone();
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
                                    &edit_rpm_limit,
                                    &edit_min_start_interval_ms,
                                    &edit_cache_hit_rate_limits,
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
                                        <span class={classes!("font-semibold")}>{ channel.name.clone() }</span>
                                        <span class={status_classes(&channel.status)}>{ channel.status.clone() }</span>
                                    </div>
                                    <div class={classes!("mono", "mt-1", "break-all", "text-[var(--muted-foreground)]")}>{ channel.base_url.clone() }</div>
                                    <div class={classes!("mono", "mt-1", "text-[var(--muted-foreground)]")}>
                                        { format!("w={} · c={} · rpm={} · min={}ms · proxy={}", channel.weight, channel.max_concurrency, channel.rpm_limit, channel.min_start_interval_ms, channel.proxy_mode) }
                                    </div>
                                    <div class={classes!("mono", "mt-1", "text-[var(--muted-foreground)]")}>
                                        {
                                            if channel.cache_hit_rate_limits.is_empty() {
                                                "cache cap=unlimited".to_string()
                                            } else {
                                                format!("cache cap={}", format_cache_hit_rate_limits(&channel.cache_hit_rate_limits))
                                            }
                                        }
                                    </div>
                                </div>
                                <div class={classes!("mono", "space-y-1")}>
                                    <div class={classes!("text-[var(--muted-foreground)]")}>{ format!("input {}", format_number_u64(total_input(channel))) }</div>
                                    <div class={classes!("text-[var(--muted-foreground)]")}>{ format!("cached {}", format_number_u64(channel.usage.input_cached_tokens)) }</div>
                                    <div class={classes!("text-[var(--muted-foreground)]")}>{ format!("output {}", format_number_u64(channel.usage.output_tokens)) }</div>
                                    <div class={classes!("font-semibold")}>{ format!("billable {}", format_number_u64(channel.usage.billable_tokens)) }</div>
                                    <div class={classes!("text-[var(--faint)]")}>{ format!("missing {} · {}", channel.usage.usage_missing_events, format_timestamp_opt(channel.usage.last_used_at)) }</div>
                                </div>
                                <div class={classes!("mono", "min-w-0", "pr-3", "space-y-2")}>
                                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                        <span class={status_classes(&models_status)}>{ models_status }</span>
                                        <span class={classes!("text-[var(--muted-foreground)]")}>{ format!("{} models", channel.models.len()) }</span>
                                    </div>
                                    <div class={classes!("text-[var(--faint)]")}>
                                        { format!("{} · {}", channel.last_models_latency_ms.map(|value| format!("{value}ms")).unwrap_or_else(|| "-".to_string()), format_timestamp_opt(channel.last_models_checked_at)) }
                                    </div>
                                    if let Some(error) = channel.last_models_error.as_deref() {
                                        <div class={classes!("break-words", "text-[var(--warning)]")}>{ error }</div>
                                    }
                                </div>
                                <div class={classes!("mono", "min-w-0", "pr-3", "space-y-2")}>
                                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                        <span class={status_classes(&test_status)}>{ test_status }</span>
                                        <span class={classes!("text-[var(--muted-foreground)]")}>{ channel.last_test_model.clone().unwrap_or_else(|| "-".to_string()) }</span>
                                    </div>
                                    <div class={classes!("text-[var(--faint)]")}>
                                        { format!("{} · {}", channel.last_test_latency_ms.map(|value| format!("{value}ms")).unwrap_or_else(|| "-".to_string()), format_timestamp_opt(channel.last_test_at)) }
                                    </div>
                                    if let Some(error) = channel.last_test_error.as_deref() {
                                        <div class={classes!("break-words", "text-[var(--warning)]")}>{ error }</div>
                                    }
                                </div>
                                <div class={classes!("space-y-2")}>
                                    <div class={classes!("flex", "gap-2", "flex-wrap")}>
                                        <button type="button" class={classes!("text-xs")} disabled={is_refreshing} onclick={on_refresh_models}>{ if is_refreshing { "Refreshing..." } else { "Refresh Status" } }</button>
                                        <button type="button" class={classes!("text-xs")} onclick={on_toggle_edit.clone()}>{ if is_editing { "Close" } else { "Edit" } }</button>
                                        <button type="button" class={classes!("text-xs")} onclick={on_toggle}>{ if channel.status == "active" { "Disable" } else { "Enable" } }</button>
                                        <button type="button" class={classes!("text-xs", "ghost")} onclick={on_rotate_key}>{ "Rotate" }</button>
                                        <button type="button" class={classes!("text-xs", "danger")} onclick={on_delete}>{ "Delete" }</button>
                                    </div>
                                    <div class={classes!("flex", "items-center", "gap-2")}>
                                        <select class={classes!("mono", "min-w-0", "flex-1")} value={selected_model.clone()} disabled={channel.models.is_empty() || is_testing} onchange={on_select_model}>
                                            {
                                                if channel.models.is_empty() {
                                                    html! { <option value="">{ "Refresh to select model" }</option> }
                                                } else {
                                                    html! {
                                                        for channel.models.iter().map(|model| html! {
                                                            <option value={model.clone()} selected={*model == selected_model}>{ model.clone() }</option>
                                                        })
                                                    }
                                                }
                                            }
                                        </select>
                                        <button type="button" class={classes!("text-xs", "primary")} disabled={channel.models.is_empty() || is_testing} onclick={on_test_model}>
                                            { if is_testing { "Testing..." } else { "Test Model" } }
                                        </button>
                                    </div>
                                    if let Some(error) = channel.last_error.as_deref() {
                                        <div class={classes!("errorline", "text-xs")}>
                                            <span class={classes!("min-w-0", "break-words", "mono")}>{ error }</span>
                                            <button type="button" class={classes!("text-xs", "ghost", "shrink-0")} onclick={on_clear_error}>{ "Clear" }</button>
                                        </div>
                                    }
                                </div>
                            </div>
                            if is_editing {
                                <div class={classes!("border-b", "border-[var(--border)]", "bg-[var(--card-2)]", "px-4", "py-3")}>
                                    <div class={classes!("grid", "min-w-[74rem]", "gap-3", "items-end", "lg:grid-cols-10")}>
                                        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]", "lg:col-span-2")}>
                                            { "Base URL" }
                                            <input class={classes!("mono")} value={(*edit_base_url).clone()} oninput={{
                                                let edit_base_url = edit_base_url.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_base_url.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                            { "Weight" }
                                            <input class={classes!("mono")} value={(*edit_weight).clone()} oninput={{
                                                let edit_weight = edit_weight.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_weight.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                            { "Concurrency" }
                                            <input class={classes!("mono")} value={(*edit_max_concurrency).clone()} oninput={{
                                                let edit_max_concurrency = edit_max_concurrency.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_max_concurrency.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                            { "RPM" }
                                            <input type="number" min="1" class={classes!("mono")} value={(*edit_rpm_limit).clone()} oninput={{
                                                let edit_rpm_limit = edit_rpm_limit.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_rpm_limit.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                            { "Min ms" }
                                            <input class={classes!("mono")} value={(*edit_min_start_interval_ms).clone()} oninput={{
                                                let edit_min_start_interval_ms = edit_min_start_interval_ms.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    edit_min_start_interval_ms.set(input.value());
                                                })
                                            }} />
                                        </label>
                                        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                            { "Proxy" }
                                            <select value={(*edit_proxy_choice).clone()} onchange={{
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
                                        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]", "lg:col-span-3")}>
                                            { "Cache hit caps · context:percent" }
                                            <input
                                                class={classes!("mono")}
                                                placeholder="empty = unlimited"
                                                value={(*edit_cache_hit_rate_limits).clone()}
                                                oninput={{
                                                    let edit_cache_hit_rate_limits = edit_cache_hit_rate_limits.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        let input: HtmlInputElement = event.target_unchecked_into();
                                                        edit_cache_hit_rate_limits.set(input.value());
                                                    })
                                                }}
                                            />
                                        </label>
                                        <div class={classes!("flex", "items-end", "gap-2")}>
                                            <button type="button" class={classes!("primary", "w-full")} disabled={*edit_saving} onclick={on_save_edit}>
                                                { if *edit_saving { "Saving..." } else { "Save" } }
                                            </button>
                                            <button type="button" class={classes!("w-full")} onclick={on_toggle_edit}>{ "Cancel" }</button>
                                        </div>
                                    </div>
                                </div>
                            }
                            </>
                        }
                    }) }
                    if *loading && channels.is_empty() {
                        <div class={classes!("skeleton", "px-4", "py-4")}>
                            <i></i><i></i><i></i><i></i><i></i>
                        </div>
                    } else if channels.is_empty() {
                        <div class={classes!("empty")}>
                            <span>{ "还没有配置任何 Anthropic 上游渠道" }</span>
                            <span class={classes!("text-xs")}>{ "用上面的 New Channel 表单创建第一个。" }</span>
                        </div>
                    }
                </section>
            </div>
        </main>
    }
}
