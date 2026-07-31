use std::collections::HashSet;

use js_sys::Date;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::{use_navigator, Link};

use crate::{
    api::{
        check_admin_llm_gateway_proxy_config, check_admin_llm_gateway_proxy_config_full_chain,
        delete_admin_llm_gateway_account_group, delete_admin_llm_gateway_key,
        delete_admin_llm_gateway_proxy_config,
        fetch_admin_llm_gateway_account_contribution_requests, fetch_admin_llm_gateway_keys_page,
        fetch_admin_llm_gateway_sponsor_requests, fetch_admin_llm_gateway_token_requests,
        patch_admin_llm_gateway_account_group, patch_admin_llm_gateway_key,
        patch_admin_llm_gateway_proxy_config, refresh_admin_llm_gateway_proxy_traffic,
        reset_admin_llm_gateway_proxy_config_override, AccountSummaryView,
        AdminAccountGroupOptionView, AdminAccountGroupView,
        AdminLlmGatewayAccountContributionRequestsQuery, AdminLlmGatewayKeyView,
        AdminLlmGatewayKeysSummaryView, AdminLlmGatewaySponsorRequestsQuery,
        AdminLlmGatewayTokenRequestsQuery, AdminProxyTrafficSnapshotView,
        AdminUpstreamProxyCheckResponse, AdminUpstreamProxyCheckTargetView,
        AdminUpstreamProxyConfigView, AdminUpstreamProxyEndpointCheckView,
        PatchAdminAccountGroupInput, PatchAdminLlmGatewayKeyRequest,
        PatchAdminUpstreamProxyConfigInput,
    },
    components::status_badge::StatusBadge,
    pages::llm_access_shared::{
        confirm_destructive, format_latency_ms, format_ms, format_number_i64, format_number_u64,
        format_optional_bytes_human, MaskedSecretCode,
    },
    router::Route,
};

pub(crate) const USAGE_PAGE_SIZE: usize = 20;
pub(crate) const USAGE_SOURCE_HOT: &str = "hot";
pub(crate) const USAGE_SOURCE_ARCHIVE: &str = "archive";
pub(crate) const USAGE_SOURCE_ALL: &str = "all";
pub(crate) const USAGE_STATUS_KIND_ALL: &str = "all";
pub(crate) const USAGE_STATUS_KIND_OK: &str = "ok";
pub(crate) const USAGE_STATUS_KIND_NON_OK: &str = "non_ok";
/// First-page scan size for the pending-request badge.
const PENDING_SCAN_PAGE_SIZE: usize = 20;
const PROXY_TRAFFIC_QUERY_WINDOW_DAYS: u64 = 30;
/// Page size for the Usage tab's server-side key filter search.
pub(crate) const USAGE_KEY_OPTION_LIMIT: usize = 20;
const TAB_KEYS: &str = "keys";
const TAB_GROUPS: &str = "groups";
const TAB_ACCOUNTS: &str = "accounts";
const TAB_USAGE: &str = "usage";
const TAB_JOURNAL: &str = "journal";
const TAB_REQUESTS: &str = "requests";
const TAB_SETTINGS: &str = "settings";

pub(crate) fn admin_group_total_pages(total: usize, page_size: usize) -> usize {
    total.max(1).div_ceil(page_size.max(1))
}

/// Render a horizontal tab bar with an optional numeric badge on one tab.
/// `badge_tab` is `Some((tab_id, count))` to show a pending-count pill.
// NOTE: the implementation moved to `crate::components::tab_bar::render_tab_bar`.
// Keep this comment block to preserve git blame context for reviewers.

#[wasm_bindgen(inline_js = r#"
export function copy_text(text) {
    if (navigator.clipboard) {
        navigator.clipboard.writeText(text).catch(function(){});
    }
}
"#)]
extern "C" {
    fn copy_text(text: &str);
}

pub(crate) fn format_optional_latency_ms(latency_ms: Option<i32>) -> String {
    latency_ms
        .map(format_latency_ms)
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn format_optional_latency_ms_or_na(
    latency_ms: Option<i32>,
    applicable: bool,
) -> String {
    if applicable {
        format_optional_latency_ms(latency_ms)
    } else {
        "n/a".to_string()
    }
}

pub(crate) fn usage_account_label(
    account_name: &Option<String>,
    request_url: &str,
    endpoint: &str,
) -> String {
    if let Some(account_name) = account_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return account_name.to_string();
    }
    if request_url.contains("/kiro-gateway") || endpoint.contains("generateAssistantResponse") {
        "not captured".to_string()
    } else {
        "legacy auth".to_string()
    }
}

fn routing_total_ms_from_diagnostics(raw: Option<&str>) -> Option<i32> {
    let route_total_ms = serde_json::from_str::<serde_json::Value>(raw?).ok()?;
    let route_total_ms = route_total_ms.get("route_total_ms")?.as_u64()?;
    Some(route_total_ms.min(i32::MAX as u64) as i32)
}

pub(crate) fn effective_routing_wait_ms(
    routing_wait_ms: Option<i32>,
    routing_diagnostics_json: Option<&str>,
) -> Option<i32> {
    routing_wait_ms.or_else(|| routing_total_ms_from_diagnostics(routing_diagnostics_json))
}

pub(crate) fn format_optional_bytes(bytes: Option<u64>) -> String {
    format_optional_bytes_human(bytes)
}

fn proxy_traffic_window_label(retention_days: u64) -> String {
    let retained_days = retention_days.clamp(1, PROXY_TRAFFIC_QUERY_WINDOW_DAYS);
    if retained_days >= PROXY_TRAFFIC_QUERY_WINDOW_DAYS {
        format!("{PROXY_TRAFFIC_QUERY_WINDOW_DAYS}d traffic")
    } else {
        format!("retained {retained_days}d traffic")
    }
}

fn proxy_traffic_snapshot_badge(snapshot: Option<&AdminProxyTrafficSnapshotView>) -> String {
    match snapshot {
        Some(snapshot) => format!(
            "{} {}",
            proxy_traffic_window_label(snapshot.retention_days),
            format_optional_bytes(Some(snapshot.totals.total_bytes))
        ),
        None => "traffic not calculated".to_string(),
    }
}

fn proxy_traffic_snapshot_meta(snapshot: Option<&AdminProxyTrafficSnapshotView>) -> String {
    match snapshot {
        Some(snapshot) => format!(
            "traffic refreshed {} · traffic events {}",
            format_ms(snapshot.refreshed_at_ms),
            snapshot.totals.event_count
        ),
        None => "traffic not calculated".to_string(),
    }
}

pub(crate) fn format_optional_duration_ms(age_ms: Option<i64>) -> String {
    let Some(age_ms) = age_ms.filter(|value| *value >= 0) else {
        return "-".to_string();
    };
    if age_ms >= 3_600_000 {
        format!("{:.1} h", age_ms as f64 / 3_600_000.0)
    } else if age_ms >= 60_000 {
        format!("{:.1} min", age_ms as f64 / 60_000.0)
    } else if age_ms >= 1_000 {
        format!("{:.1} s", age_ms as f64 / 1_000.0)
    } else {
        format!("{age_ms} ms")
    }
}

pub(crate) fn usage_retry_title(count: u64, delay_ms: i64, reasons: &[String]) -> String {
    let mut title = format!(
        "same-account retry {count} · total sleep {}",
        format_latency_ms(delay_ms.clamp(0, i64::from(i32::MAX)) as i32)
    );
    if !reasons.is_empty() {
        title.push_str(" · ");
        title.push_str(&reasons.join(", "));
    }
    title
}

pub(crate) fn usage_stream_state_label(
    stream_completed_cleanly: Option<bool>,
    downstream_disconnect: Option<bool>,
) -> &'static str {
    if downstream_disconnect == Some(true) {
        "disconnect"
    } else if stream_completed_cleanly == Some(true) {
        "clean"
    } else if stream_completed_cleanly == Some(false) {
        "incomplete"
    } else {
        "n/a"
    }
}

pub(crate) fn usage_stream_state_badge_classes(
    stream_completed_cleanly: Option<bool>,
    downstream_disconnect: Option<bool>,
) -> Classes {
    let mut classes = classes!(
        "inline-flex",
        "rounded-full",
        "border",
        "px-2.5",
        "py-1",
        "text-[11px]",
        "font-semibold",
        "uppercase",
        "tracking-[0.12em]"
    );
    match usage_stream_state_label(stream_completed_cleanly, downstream_disconnect) {
        "clean" => {
            classes.push("border-emerald-500/20");
            classes.push("bg-emerald-500/10");
            classes.push("text-emerald-700");
            classes.push("dark:text-emerald-200");
        },
        "disconnect" => {
            classes.push("border-red-500/20");
            classes.push("bg-red-500/10");
            classes.push("text-red-700");
            classes.push("dark:text-red-200");
        },
        "incomplete" => {
            classes.push("border-amber-500/20");
            classes.push("bg-amber-500/10");
            classes.push("text-amber-700");
            classes.push("dark:text-amber-200");
        },
        _ => {
            classes.push("border-slate-500/20");
            classes.push("bg-slate-500/10");
            classes.push("text-slate-700");
            classes.push("dark:text-slate-200");
        },
    }
    classes
}

pub(crate) fn format_stream_summary(
    stream_completed_cleanly: Option<bool>,
    downstream_disconnect: Option<bool>,
    final_event_type: Option<&str>,
    bytes_streamed: Option<u64>,
) -> String {
    let final_event_type = final_event_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-");
    format!(
        "state {} · final {} · bytes {}",
        usage_stream_state_label(stream_completed_cleanly, downstream_disconnect),
        final_event_type,
        format_optional_bytes(bytes_streamed),
    )
}

pub(crate) fn compute_other_latency_ms(
    latency_ms: i32,
    routing_wait_ms: Option<i32>,
    upstream_headers_ms: Option<i32>,
    post_headers_body_ms: Option<i32>,
) -> Option<i32> {
    if routing_wait_ms.is_none() && upstream_headers_ms.is_none() && post_headers_body_ms.is_none()
    {
        return None;
    }
    let measured_ms: i64 = [routing_wait_ms, upstream_headers_ms, post_headers_body_ms]
        .into_iter()
        .flatten()
        .map(|value| i64::from(value.max(0)))
        .sum();
    Some((i64::from(latency_ms.max(0)) - measured_ms).clamp(0, i64::from(i32::MAX)) as i32)
}

#[derive(Clone, Copy)]
pub(crate) struct LatencyBreakdown {
    pub(crate) latency_ms: i32,
    pub(crate) routing_wait_ms: Option<i32>,
    pub(crate) upstream_headers_ms: Option<i32>,
    pub(crate) post_headers_body_ms: Option<i32>,
    pub(crate) request_body_bytes: Option<u64>,
    pub(crate) request_body_read_ms: Option<i32>,
    pub(crate) request_json_parse_ms: Option<i32>,
    pub(crate) pre_handler_ms: Option<i32>,
    pub(crate) first_sse_write_ms: Option<i32>,
    pub(crate) stream_finish_ms: Option<i32>,
    pub(crate) other_latency_ms: Option<i32>,
    pub(crate) quota_failover_count: u64,
}

pub(crate) fn format_latency_breakdown(parts: LatencyBreakdown) -> String {
    let other_latency_ms = parts.other_latency_ms.or_else(|| {
        compute_other_latency_ms(
            parts.latency_ms,
            parts.routing_wait_ms,
            parts.upstream_headers_ms,
            parts.post_headers_body_ms,
        )
    });
    let sse_applicable = parts.first_sse_write_ms.is_some();
    format!(
        "total {} · ingress {} body {} parse {} pre-handler {} · route {} · upstream headers {} · \
         post-headers body {} · first SSE {} · stream finish {} · other {} · quota failover {}",
        format_latency_ms(parts.latency_ms),
        format_optional_bytes(parts.request_body_bytes),
        format_optional_latency_ms(parts.request_body_read_ms),
        format_optional_latency_ms(parts.request_json_parse_ms),
        format_optional_latency_ms(parts.pre_handler_ms),
        format_optional_latency_ms(parts.routing_wait_ms),
        format_optional_latency_ms(parts.upstream_headers_ms),
        format_optional_latency_ms(parts.post_headers_body_ms),
        format_optional_latency_ms_or_na(parts.first_sse_write_ms, sse_applicable),
        format_optional_latency_ms(parts.stream_finish_ms),
        format_optional_latency_ms(other_latency_ms),
        parts.quota_failover_count
    )
}

pub(crate) fn routing_diagnostics_summary(raw: &str) -> Vec<(String, String)> {
    let Some(value) = serde_json::from_str::<serde_json::Value>(raw).ok() else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    let mut push_ms = |label: &str, key: &str| {
        if let Some(value) = value.get(key).and_then(|value| value.as_u64()) {
            rows.push((label.to_string(), format!("{value} ms")));
        }
    };
    push_ms("Route total", "route_total_ms");
    push_ms("Status load", "status_load_ms");
    push_ms("Selection", "selection_ms");
    push_ms("Local queue", "local_queue_wait_ms");
    push_ms("Cooldown wait", "upstream_cooldown_wait_ms");
    for (label, key) in [
        ("Attempts", "account_attempt_count"),
        ("Skipped", "skipped_account_count"),
        ("Codex failover", "failover_count"),
        ("Quota failover", "quota_failover_count"),
        ("Rate-limit failover", "rate_limit_failover_count"),
        ("Retry next", "retry_next_count"),
    ] {
        if let Some(count) = value.get(key).and_then(|value| value.as_u64()) {
            rows.push((label.to_string(), count.to_string()));
        }
    }
    if let Some(account) = value
        .get("selected_account")
        .and_then(|value| value.as_str())
    {
        rows.push(("Selected".to_string(), account.to_string()));
    }
    rows
}

pub(crate) fn format_credit4(value: f64) -> String {
    format!("{value:.4}")
}

fn key_credit_display(key_item: &AdminLlmGatewayKeyView) -> String {
    if key_item.usage_credit_total > 0.0 || key_item.usage_credit_missing_events > 0 {
        format_credit4(key_item.usage_credit_total)
    } else {
        "-".to_string()
    }
}

pub(crate) fn usage_source_label(value: &str) -> &'static str {
    match value {
        USAGE_SOURCE_ARCHIVE => "历史归档",
        USAGE_SOURCE_ALL => "全部",
        _ => "在线",
    }
}

pub(crate) fn usage_status_kind_label(value: &str) -> &'static str {
    match value {
        USAGE_STATUS_KIND_OK => "正常",
        USAGE_STATUS_KIND_NON_OK => "异常",
        _ => "全部状态",
    }
}

pub(crate) fn parse_datetime_local_input_to_ms(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = Date::new(&JsValue::from_str(trimmed)).get_time();
    (!parsed.is_nan()).then_some(parsed as i64)
}

pub(crate) fn format_datetime_local_input(ms: i64) -> String {
    let date = Date::new(&JsValue::from_f64(ms as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}")
}

pub(crate) fn usage_time_description(start_input: &str, end_input: &str) -> String {
    match (start_input.trim(), end_input.trim()) {
        ("", "") => "全部时间".to_string(),
        (start, "") => format!("{start} -> now"),
        ("", end) => format!("start -> {end}"),
        (start, end) => format!("{start} -> {end}"),
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct UsageReloadArgs {
    pub(crate) page: Option<usize>,
    pub(crate) key_id: Option<String>,
    pub(crate) start_input: Option<String>,
    pub(crate) end_input: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) account_name: Option<String>,
    pub(crate) endpoint: Option<String>,
    pub(crate) status_kind: Option<String>,
    pub(crate) refresh_filter_options: bool,
}

impl Default for UsageReloadArgs {
    fn default() -> Self {
        Self {
            page: None,
            key_id: None,
            start_input: None,
            end_input: None,
            source: None,
            model: None,
            account_name: None,
            endpoint: None,
            status_kind: None,
            refresh_filter_options: true,
        }
    }
}

pub(crate) fn normalized_usage_filter_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(crate) fn normalize_optional_form_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn recommended_socks5h_proxy_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let prefix_len = "socks5://".len();
    if trimmed
        .get(..prefix_len)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("socks5://"))
    {
        Some(format!("socks5h://{}", &trimmed[prefix_len..]))
    } else {
        None
    }
}

pub(crate) fn proxy_url_after_socks5h_confirmation(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(recommended) = recommended_socks5h_proxy_url(trimmed) else {
        return trimmed.to_string();
    };
    if confirm_destructive(
        "检测到当前代理 URL 使用 socks5://。\n\n对 ChatGPT/Codex 这类依赖 CDN/DNS \
         路由的上游，推荐使用 socks5h://，让代理服务器解析域名，避免本机 DNS 解析出的 IP \
         在代理出口不可用。\n\n点击“确定”自动转换为 socks5h://；点击“取消”继续保留 socks5://。",
    ) {
        recommended
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn normalized_usage_status_kind(value: &str) -> Option<String> {
    match value.trim() {
        USAGE_STATUS_KIND_OK => Some(USAGE_STATUS_KIND_OK.to_string()),
        USAGE_STATUS_KIND_NON_OK => Some(USAGE_STATUS_KIND_NON_OK.to_string()),
        _ => None,
    }
}

fn sanitize_auto_account_names(names: &[String], accounts: &[AccountSummaryView]) -> Vec<String> {
    let valid_names = accounts
        .iter()
        .map(|account| account.name.as_str())
        .collect::<HashSet<_>>();
    let mut sanitized = names
        .iter()
        .filter(|name| valid_names.contains(name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    sanitized.sort();
    sanitized.dedup();
    sanitized
}

fn sanitize_account_group_id(
    value: Option<&str>,
    groups: &[AdminAccountGroupOptionView],
    _allow_empty: bool,
) -> String {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return String::new();
    };
    if groups.iter().any(|group| group.id == value) {
        value.to_string()
    } else {
        String::new()
    }
}

fn group_name_for_id(groups: &[AdminAccountGroupOptionView], group_id: &str) -> String {
    groups
        .iter()
        .find(|group| group.id == group_id)
        .map(|group| group.name.clone())
        .unwrap_or_else(|| group_id.to_string())
}

fn format_proxy_check_target_line(target: &AdminUpstreamProxyCheckTargetView) -> String {
    if target.reachable {
        format!(
            "{}: {} in {} ms",
            target.target,
            target
                .status_code
                .map(|status| status.to_string())
                .unwrap_or_else(|| "ok".to_string()),
            target.latency_ms.max(0)
        )
    } else {
        format!(
            "{}: {}",
            target.target,
            target
                .error_message
                .clone()
                .unwrap_or_else(|| "request failed".to_string())
        )
    }
}

fn format_proxy_check_message(result: &AdminUpstreamProxyCheckResponse) -> String {
    let mut lines = vec![if result.ok {
        format!(
            "{} 代理检查成功：{}",
            result.provider_type.to_uppercase(),
            result.proxy_config_name
        )
    } else {
        format!(
            "{} 代理检查失败：{}",
            result.provider_type.to_uppercase(),
            result.proxy_config_name
        )
    }];
    lines.push(format!("使用认证：{}", result.auth_label));
    lines.extend(result.targets.iter().map(format_proxy_check_target_line));
    lines.join("\n")
}

fn format_proxy_endpoint_check_summary(
    provider_label: &str,
    check: Option<&AdminUpstreamProxyEndpointCheckView>,
) -> String {
    let Some(check) = check else {
        return format!("{provider_label}: 未检测");
    };
    let status = check
        .status_code
        .map(|status| format!("HTTP {status}"))
        .unwrap_or_else(|| {
            if check.reachable {
                "reachable".to_string()
            } else {
                "failed".to_string()
            }
        });
    format!(
        "{provider_label}: {} ms · {} · {}",
        check.latency_ms.max(0),
        status,
        format_ms(check.checked_at)
    )
}

fn proxy_endpoint_check_tone(check: Option<&AdminUpstreamProxyEndpointCheckView>) -> &'static str {
    match check {
        Some(check) if !check.reachable => {
            "border-red-500/30 bg-red-500/8 text-red-700 dark:text-red-200"
        },
        Some(_) => "border-emerald-500/30 bg-emerald-500/8 text-emerald-700 dark:text-emerald-200",
        None => "border-[var(--border)] bg-[var(--surface-alt)] text-[var(--muted)]",
    }
}

pub(crate) fn preview_text(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }
    let total_chars = trimmed.chars().count();
    if total_chars <= max_chars {
        trimmed.to_string()
    } else {
        let prefix = trimmed.chars().take(max_chars).collect::<String>();
        format!("{prefix}...")
    }
}

// Render a compact status pill that matches the current key state.
// Keep copy affordances visually small so dense diagnostics tables stay
// readable.
pub(crate) fn copy_icon_button(text: &str, on_copy: &Callback<(String, String)>) -> Html {
    let value = text.to_string();
    let on_copy = on_copy.clone();
    html! {
        <button
            type="button"
            class={classes!(
                "inline-flex",
                "h-8",
                "w-8",
                "items-center",
                "justify-center",
                "rounded-full",
                "border",
                "border-[var(--border)]",
                "bg-[var(--surface)]",
                "text-[var(--muted)]",
                "transition-colors",
                "hover:text-[var(--primary)]",
                "hover:bg-[var(--surface-alt)]"
            )}
            title="复制"
            aria-label="复制"
            onclick={Callback::from(move |_| on_copy.emit(("".to_string(), value.clone())))}
        >
            <i class={classes!("fas", "fa-copy", "text-xs")} />
        </button>
    }
}

// Reformat stored header JSON before showing it in the modal dialog.
pub(crate) fn pretty_headers_json(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .unwrap_or_else(|| raw.to_string())
}

pub(crate) fn usage_journal_preview_message(
    preview: &crate::api::AdminUsageJournalPreviewEventView,
) -> String {
    preview
        .last_message_content
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn usage_journal_preview_has_full_message(
    preview: &crate::api::AdminUsageJournalPreviewEventView,
) -> bool {
    let message = usage_journal_preview_message(preview);
    message != "-"
}

pub(crate) fn pretty_json_text(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .unwrap_or_else(|| raw.to_string())
}

#[derive(Properties, PartialEq)]
pub(crate) struct KeyEditorCardProps {
    pub(crate) key_item: AdminLlmGatewayKeyView,
    pub(crate) on_changed: Callback<()>,
    pub(crate) on_refresh: Callback<(String, String)>,
    pub(crate) on_copy: Callback<(String, String)>,
    pub(crate) on_flash: Callback<(String, bool)>,
    pub(crate) refreshing: bool,
    pub(crate) account_groups: Vec<AdminAccountGroupOptionView>,
}

#[function_component(KeyEditorCard)]
pub(crate) fn key_editor_card(props: &KeyEditorCardProps) -> Html {
    let key_item = props.key_item.clone();
    let key_name_for_actions = key_item.name.clone();
    let name = use_state(|| key_item.name.clone());
    let quota = use_state(|| key_item.quota_billable_limit.to_string());
    let public_visible = use_state(|| key_item.public_visible);
    let status = use_state(|| key_item.status.clone());
    let route_strategy = use_state(|| {
        key_item
            .route_strategy
            .clone()
            .unwrap_or_else(|| "auto".to_string())
    });
    let account_group_id = use_state(|| {
        sanitize_account_group_id(key_item.account_group_id.as_deref(), &props.account_groups, true)
    });
    let request_max_concurrency = use_state(|| {
        key_item
            .request_max_concurrency
            .map(|value| value.to_string())
            .unwrap_or_default()
    });
    let request_min_start_interval_ms = use_state(|| {
        key_item
            .request_min_start_interval_ms
            .map(|value| value.to_string())
            .unwrap_or_default()
    });
    let moderation_enabled = use_state(|| key_item.moderation_enabled);
    let codex_fast_enabled = use_state(|| key_item.codex_fast_enabled);
    let codex_responses_lite_enabled = use_state(|| key_item.codex_responses_lite_enabled);
    let codex_full_request_logging_enabled =
        use_state(|| key_item.codex_full_request_logging_enabled);
    let codex_strict_session_rejection_enabled =
        use_state(|| key_item.codex_strict_session_rejection_enabled);
    let codex_image_standalone_generation_enabled =
        use_state(|| key_item.codex_image_standalone_generation_enabled);
    let codex_image_direct_generation_enabled =
        use_state(|| key_item.codex_image_direct_generation_enabled);
    let saving = use_state(|| false);
    let feedback = use_state(|| None::<String>);

    {
        // Reset editor controls whenever the parent list refreshes this card.
        let key_item = props.key_item.clone();
        let account_groups = props.account_groups.clone();
        let name = name.clone();
        let quota = quota.clone();
        let public_visible = public_visible.clone();
        let status = status.clone();
        let route_strategy = route_strategy.clone();
        let account_group_id = account_group_id.clone();
        let request_max_concurrency = request_max_concurrency.clone();
        let request_min_start_interval_ms = request_min_start_interval_ms.clone();
        let moderation_enabled = moderation_enabled.clone();
        let codex_fast_enabled = codex_fast_enabled.clone();
        let codex_responses_lite_enabled = codex_responses_lite_enabled.clone();
        let codex_full_request_logging_enabled = codex_full_request_logging_enabled.clone();
        let codex_strict_session_rejection_enabled = codex_strict_session_rejection_enabled.clone();
        let codex_image_standalone_generation_enabled =
            codex_image_standalone_generation_enabled.clone();
        let codex_image_direct_generation_enabled = codex_image_direct_generation_enabled.clone();
        use_effect_with((props.key_item.clone(), props.account_groups.clone()), move |_| {
            name.set(key_item.name.clone());
            quota.set(key_item.quota_billable_limit.to_string());
            public_visible.set(key_item.public_visible);
            status.set(key_item.status.clone());
            route_strategy.set(
                key_item
                    .route_strategy
                    .clone()
                    .unwrap_or_else(|| "auto".to_string()),
            );
            account_group_id.set(sanitize_account_group_id(
                key_item.account_group_id.as_deref(),
                &account_groups,
                true,
            ));
            request_max_concurrency.set(
                key_item
                    .request_max_concurrency
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            );
            request_min_start_interval_ms.set(
                key_item
                    .request_min_start_interval_ms
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            );
            moderation_enabled.set(key_item.moderation_enabled);
            codex_fast_enabled.set(key_item.codex_fast_enabled);
            codex_responses_lite_enabled.set(key_item.codex_responses_lite_enabled);
            codex_full_request_logging_enabled.set(key_item.codex_full_request_logging_enabled);
            codex_strict_session_rejection_enabled
                .set(key_item.codex_strict_session_rejection_enabled);
            codex_image_standalone_generation_enabled
                .set(key_item.codex_image_standalone_generation_enabled);
            codex_image_direct_generation_enabled
                .set(key_item.codex_image_direct_generation_enabled);
            || ()
        });
    }

    if key_item.provider_type == "kiro" {
        return html! {
            <article class={classes!(
                "rounded-xl",
                "border",
                "border-[var(--border)]",
                "bg-[var(--surface)]",
                "p-5",
                "transition-all",
                "duration-200",
                "hover:shadow-lg",
                "hover:shadow-black/5"
            )}>
                <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                        <span class={classes!("inline-flex", "items-center", "rounded-full", "bg-slate-900", "px-2.5", "py-1", "font-mono", "text-[11px]", "font-semibold", "uppercase", "tracking-[0.16em]", "text-emerald-300")}>
                            { "Kiro Key" }
                        </span>
                        <h3 class={classes!("m-0", "text-base", "font-bold")}>{ key_item.name.clone() }</h3>
                    </div>
                    <Link<Route> to={Route::AdminKiroGateway} classes={classes!("btn-terminal")}>
                        { "前往 /admin/kiro-gateway" }
                    </Link<Route>>
                </div>

                <div class={classes!("mt-3", "rounded-lg", "bg-slate-950", "px-3", "py-2", "text-xs", "text-emerald-200")}>
                    <MaskedSecretCode
                        value={key_item.secret.clone()}
                        copy_label={"Kiro Key"}
                        on_copy={props.on_copy.clone()}
                        code_class={classes!("text-emerald-200")}
                    />
                </div>

                <div class={classes!("mt-3", "flex", "items-center", "gap-3", "flex-wrap", "text-xs", "text-[var(--muted)]")}>
                    <span>{ format!("status {}", key_item.status) }</span>
                    <span>{ format!("created {}", format_ms(key_item.created_at)) }</span>
                    <button
                        class={classes!("btn-terminal", "ml-auto")}
                        onclick={{
                            let on_copy = props.on_copy.clone();
                            let secret = key_item.secret.clone();
                            Callback::from(move |_| on_copy.emit(("Kiro Key".to_string(), secret.clone())))
                        }}
                    >
                        { "复制" }
                    </button>
                </div>
            </article>
        };
    }

    let on_save = {
        let key_id = key_item.id.clone();
        let name = name.clone();
        let quota = quota.clone();
        let public_visible = public_visible.clone();
        let status = status.clone();
        let route_strategy = route_strategy.clone();
        let account_group_id = account_group_id.clone();
        let request_max_concurrency = request_max_concurrency.clone();
        let request_min_start_interval_ms = request_min_start_interval_ms.clone();
        let moderation_enabled = moderation_enabled.clone();
        let codex_fast_enabled = codex_fast_enabled.clone();
        let codex_responses_lite_enabled = codex_responses_lite_enabled.clone();
        let codex_full_request_logging_enabled = codex_full_request_logging_enabled.clone();
        let codex_strict_session_rejection_enabled = codex_strict_session_rejection_enabled.clone();
        let codex_image_standalone_generation_enabled =
            codex_image_standalone_generation_enabled.clone();
        let codex_image_direct_generation_enabled = codex_image_direct_generation_enabled.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        let on_changed = props.on_changed.clone();
        let key_name_for_actions = key_name_for_actions.clone();
        Callback::from(move |_| {
            let key_id = key_id.clone();
            let key_name = key_name_for_actions.clone();
            let name_value = (*name).trim().to_string();
            let quota_value = (*quota).trim().parse::<u64>();
            let public_visible_value = *public_visible;
            let status_value = (*status).clone();
            let route_strategy_value = (*route_strategy).clone();
            let account_group_id_value = (*account_group_id).clone();
            let request_max_concurrency_value = (*request_max_concurrency).trim().to_string();
            let request_min_start_interval_ms_value =
                (*request_min_start_interval_ms).trim().to_string();
            let moderation_enabled_value = *moderation_enabled;
            let codex_fast_enabled_value = *codex_fast_enabled;
            let codex_responses_lite_enabled_value = *codex_responses_lite_enabled;
            let codex_full_request_logging_enabled_value = *codex_full_request_logging_enabled;
            let codex_strict_session_rejection_enabled_value =
                *codex_strict_session_rejection_enabled;
            let codex_image_standalone_generation_enabled_value =
                *codex_image_standalone_generation_enabled;
            let codex_image_direct_generation_enabled_value =
                *codex_image_direct_generation_enabled;
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_flash = on_flash.clone();
            let on_changed = on_changed.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if *saving {
                    return;
                }
                let Ok(quota_value) = quota_value else {
                    let message = "额度必须是正整数".to_string();
                    feedback.set(Some(message.clone()));
                    on_flash.emit((message, true));
                    return;
                };
                let request_max_concurrency_value = if request_max_concurrency_value.is_empty() {
                    None
                } else {
                    match request_max_concurrency_value.parse::<u64>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            let message = "并发上限必须是整数，留空表示不限制".to_string();
                            feedback.set(Some(message.clone()));
                            on_flash.emit((message, true));
                            return;
                        },
                    }
                };
                let request_min_start_interval_ms_value =
                    if request_min_start_interval_ms_value.is_empty() {
                        None
                    } else {
                        match request_min_start_interval_ms_value.parse::<u64>() {
                            Ok(value) => Some(value),
                            Err(_) => {
                                let message = "请求间隔必须是整数毫秒，留空表示不限制".to_string();
                                feedback.set(Some(message.clone()));
                                on_flash.emit((message, true));
                                return;
                            },
                        }
                    };
                saving.set(true);
                match patch_admin_llm_gateway_key(&key_id, PatchAdminLlmGatewayKeyRequest {
                    name: Some(&name_value),
                    status: Some(&status_value),
                    public_visible: Some(public_visible_value),
                    quota_billable_limit: Some(quota_value),
                    route_strategy: Some(&route_strategy_value),
                    account_group_id: Some(&account_group_id_value),
                    fixed_account_name: None,
                    auto_account_names: None,
                    preferred_pool_strategy: None,
                    kiro_anthropic_upstream_pool_mode: None,
                    model_name_map: None,
                    kiro_model_group_preferences: None,
                    kiro_model_channel_preferences: None,
                    request_max_concurrency: request_max_concurrency_value,
                    request_min_start_interval_ms: request_min_start_interval_ms_value,
                    moderation_enabled: Some(moderation_enabled_value),
                    codex_fast_enabled: Some(codex_fast_enabled_value),
                    codex_responses_lite_enabled: Some(codex_responses_lite_enabled_value),
                    codex_full_request_logging_enabled: Some(
                        codex_full_request_logging_enabled_value,
                    ),
                    codex_strict_session_rejection_enabled: Some(
                        codex_strict_session_rejection_enabled_value,
                    ),
                    codex_image_generation_enabled: None,
                    codex_image_standalone_generation_enabled: Some(
                        codex_image_standalone_generation_enabled_value,
                    ),
                    codex_image_direct_generation_enabled: Some(
                        codex_image_direct_generation_enabled_value,
                    ),
                    kiro_request_validation_enabled: None,
                    kiro_cache_estimation_enabled: None,
                    kiro_zero_cache_debug_enabled: None,
                    kiro_full_request_logging_enabled: None,
                    kiro_remote_media_resolution_enabled: None,
                    kiro_latency_routing_enabled: None,
                    kiro_protected_content_validation_enabled: None,
                    kiro_cctest_text_handling_enabled: None,
                    kiro_cache_policy_override_json: None,
                    kiro_billable_model_multipliers_override_json: None,
                    request_max_concurrency_unlimited: request_max_concurrency_value.is_none(),
                    request_min_start_interval_ms_unlimited: request_min_start_interval_ms_value
                        .is_none(),
                })
                .await
                {
                    Ok(_) => {
                        feedback.set(Some("已保存".to_string()));
                        on_flash.emit((format!("已保存 key `{}`", key_name), false));
                        on_changed.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("保存 key `{}` 失败\n{err}", key_name), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_delete = {
        let key_id = key_item.id.clone();
        let on_changed = props.on_changed.clone();
        let feedback = feedback.clone();
        let saving = saving.clone();
        let on_flash = props.on_flash.clone();
        let key_name_for_actions = key_name_for_actions.clone();
        Callback::from(move |_| {
            if !confirm_destructive("确认删除这个 API key？") {
                return;
            }
            let key_id = key_id.clone();
            let key_name = key_name_for_actions.clone();
            let feedback = feedback.clone();
            let saving = saving.clone();
            let on_flash = on_flash.clone();
            let on_changed = on_changed.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                match delete_admin_llm_gateway_key(&key_id).await {
                    Ok(_) => {
                        feedback.set(Some("已删除".to_string()));
                        on_flash.emit((format!("已删除 key `{}`", key_name), false));
                        on_changed.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("删除 key `{}` 失败\n{err}", key_name), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    let fixed_route_groups = props
        .account_groups
        .iter()
        .filter(|group| group.account_count == 1)
        .cloned()
        .collect::<Vec<_>>();
    let current_route_summary = if *route_strategy == "fixed" {
        if (*account_group_id).is_empty() {
            "固定组：未选择".to_string()
        } else {
            format!(
                "固定组：{}",
                group_name_for_id(&props.account_groups, (*account_group_id).as_str())
            )
        }
    } else if (*account_group_id).is_empty() {
        "自动：全账号池".to_string()
    } else {
        format!("自动：{}", group_name_for_id(&props.account_groups, (*account_group_id).as_str()))
    };

    html! {
        <article class={classes!(
            "rounded-xl",
            "border",
            "border-[var(--border)]",
            "bg-[var(--surface)]",
            "p-5",
            "transition-all",
            "duration-200",
            "hover:shadow-lg",
            "hover:shadow-black/5"
        )}>
            <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                <div class={classes!("flex", "items-center", "gap-2")}>
                    <StatusBadge status={key_item.status.clone()} />
                    <h3 class={classes!("m-0", "text-base", "font-bold")}>{ key_item.name.clone() }</h3>
                    <span class={classes!("text-xs", "text-[var(--muted)]")}>{ format_ms(key_item.created_at) }</span>
                </div>
                <div class={classes!("flex", "gap-2")}>
                    <button
                        class={classes!("btn-terminal")}
                        title="刷新额度"
                        aria-label="刷新额度"
                        onclick={{
                            let on_refresh = props.on_refresh.clone();
                            let key_id = key_item.id.clone();
                            let key_name = key_item.name.clone();
                            Callback::from(move |_| on_refresh.emit((key_id.clone(), key_name.clone())))
                        }}
                        disabled={props.refreshing}
                    >
                        <i class={classes!("fas", if props.refreshing { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                    </button>
                    <button
                        class={classes!("btn-terminal")}
                        onclick={{
                            let on_copy = props.on_copy.clone();
                            let secret = key_item.secret.clone();
                            Callback::from(move |_| on_copy.emit(("Key".to_string(), secret.clone())))
                        }}
                    >
                        { "复制" }
                    </button>
                    <button class={classes!("btn-terminal", "btn-terminal-danger")} onclick={on_delete} disabled={*saving}>
                        { "删除" }
                    </button>
                </div>
            </div>

            <div class={classes!("mt-3", "rounded-lg", "bg-slate-950", "px-3", "py-2", "text-xs", "text-emerald-200")}>
                <MaskedSecretCode
                    value={key_item.secret.clone()}
                    copy_label={"Key"}
                    on_copy={props.on_copy.clone()}
                    code_class={classes!("text-emerald-200")}
                />
            </div>

            <div class={classes!("mt-3", "grid", "gap-3", "xl:grid-cols-2")}>
                <label class={classes!("text-sm")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "名称" }</span>
                    <input
                        type="text"
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
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
                <label class={classes!("text-sm")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "额度上限" }</span>
                    <input
                        type="number"
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                        value={(*quota).clone()}
                        oninput={{
                            let quota = quota.clone();
                            Callback::from(move |event: InputEvent| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    quota.set(target.value());
                                }
                            })
                        }}
                    />
                </label>
            </div>

            <div class={classes!("mt-3", "grid", "gap-3", "xl:grid-cols-2")}>
                <label class={classes!("text-sm")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "并发上限" }</span>
                    <input
                        type="number"
                        placeholder="留空表示不限制"
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                        value={(*request_max_concurrency).clone()}
                        oninput={{
                            let request_max_concurrency = request_max_concurrency.clone();
                            Callback::from(move |event: InputEvent| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    request_max_concurrency.set(target.value());
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
                        value={(*request_min_start_interval_ms).clone()}
                        oninput={{
                            let request_min_start_interval_ms = request_min_start_interval_ms.clone();
                            Callback::from(move |event: InputEvent| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    request_min_start_interval_ms.set(target.value());
                                }
                            })
                        }}
                    />
                </label>
            </div>

            <div class={classes!("mt-3", "flex", "items-center", "gap-3", "flex-wrap")}>
                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                    <input
                        type="checkbox" class={classes!("min-h-0", "w-auto")}
                        checked={*codex_full_request_logging_enabled}
                        onchange={{
                            let codex_full_request_logging_enabled =
                                codex_full_request_logging_enabled.clone();
                            Callback::from(move |event: Event| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    codex_full_request_logging_enabled.set(target.checked());
                                }
                            })
                        }}
                    />
                    <span>{ "记录完整请求报文" }</span>
                </label>
                <span class={classes!("text-xs", "leading-5", "text-[var(--muted)]")}>
                    {
                        if *codex_full_request_logging_enabled {
                            "ON · 此 key 的成功和失败请求都会保留 client、upstream 与完整 body"
                        } else {
                            "OFF · 成功请求不保留完整 body；失败请求仍保留诊断报文"
                        }
                    }
                </span>
                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                    <input
                        type="checkbox" class={classes!("min-h-0", "w-auto")}
                        checked={*public_visible}
                        onchange={{
                            let public_visible = public_visible.clone();
                            Callback::from(move |event: Event| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    public_visible.set(target.checked());
                                }
                            })
                        }}
                    />
                    <span>{ "公开" }</span>
                </label>
                <div class={classes!("flex", "min-w-[260px]", "flex-col", "gap-1", "text-sm")}>
                    <label class={classes!("flex", "items-center", "gap-2")}>
                        <input
                            type="checkbox" class={classes!("min-h-0", "w-auto")}
                            checked={*moderation_enabled}
                            onchange={{
                                let moderation_enabled = moderation_enabled.clone();
                                Callback::from(move |event: Event| {
                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                        moderation_enabled.set(target.checked());
                                    }
                                })
                            }}
                        />
                        <span>{ "审核拦截" }</span>
                    </label>
                    <span class={classes!("text-xs", "leading-5", "text-[var(--muted)]")}>
                        {
                            if *moderation_enabled {
                                "ON · 命中关键词会封禁该 key 的当前 session，并返回可定位的 review id"
                            } else {
                                "OFF · 仅此 key 跳过审核拦截；历史记录保留，重新开启后继续生效"
                            }
                        }
                    </span>
                </div>
                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                    <input
                        type="checkbox" class={classes!("min-h-0", "w-auto")}
                        checked={*codex_fast_enabled}
                        onchange={{
                            let codex_fast_enabled = codex_fast_enabled.clone();
                            Callback::from(move |event: Event| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    codex_fast_enabled.set(target.checked());
                                }
                            })
                        }}
                    />
                    <span>{ "允许 Fast（service_tier，计费 x2）" }</span>
                </label>
                <div class={classes!("flex", "min-w-[260px]", "flex-col", "gap-1", "text-sm")}>
                    <label class={classes!("flex", "items-center", "gap-2")}>
                        <input
                            type="checkbox" class={classes!("min-h-0", "w-auto")}
                            checked={*codex_responses_lite_enabled}
                            onchange={{
                                let codex_responses_lite_enabled =
                                    codex_responses_lite_enabled.clone();
                                Callback::from(move |event: Event| {
                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                        codex_responses_lite_enabled.set(target.checked());
                                    }
                                })
                            }}
                        />
                        <span>{ "Lite Responses" }</span>
                    </label>
                    <span class={classes!("text-xs", "leading-5", "text-[var(--muted)]")}>
                        {
                            if *codex_responses_lite_enabled {
                                "ON · Luna / Sol / Terra 使用 Lite wire contract"
                            } else {
                                "OFF · 所有模型按普通 Responses 转发，仅保留模型名差异"
                            }
                        }
                    </span>
                </div>
                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                    <input
                        type="checkbox" class={classes!("min-h-0", "w-auto")}
                        checked={*codex_strict_session_rejection_enabled}
                        onchange={{
                            let codex_strict_session_rejection_enabled =
                                codex_strict_session_rejection_enabled.clone();
                            Callback::from(move |event: Event| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    codex_strict_session_rejection_enabled.set(target.checked());
                                }
                            })
                        }}
                    />
                    <span>{ "严格拒绝 fatal session" }</span>
                </label>
                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                    <input
                        type="checkbox" class={classes!("min-h-0", "w-auto")}
                        checked={*codex_image_standalone_generation_enabled}
                        onchange={{
                            let codex_image_standalone_generation_enabled =
                                codex_image_standalone_generation_enabled.clone();
                            Callback::from(move |event: Event| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    codex_image_standalone_generation_enabled.set(target.checked());
                                }
                            })
                        }}
                    />
                    <span>{ "独立 Image2 入口" }</span>
                </label>
                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                    <input
                        type="checkbox" class={classes!("min-h-0", "w-auto")}
                        checked={*codex_image_direct_generation_enabled}
                        onchange={{
                            let codex_image_direct_generation_enabled =
                                codex_image_direct_generation_enabled.clone();
                            Callback::from(move |event: Event| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    codex_image_direct_generation_enabled.set(target.checked());
                                }
                            })
                        }}
                    />
                    <span>{ "Codex API 直连 Image2" }</span>
                </label>
                <select
                    key={format!("{}-status-{}", key_item.id, (*status).clone())}
                    class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-1.5", "text-sm")}
                    onchange={{
                        let status = status.clone();
                        Callback::from(move |event: Event| {
                            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                status.set(target.value());
                            }
                        })
                    }}
                >
                    <option value="active" selected={*status == "active"}>{ "active" }</option>
                    <option value="disabled" selected={*status == "disabled"}>{ "disabled" }</option>
                </select>
                <button class={classes!("btn-terminal", "btn-terminal-primary", "ml-auto")} onclick={on_save} disabled={*saving}>
                    { if *saving { "保存中..." } else { "保存" } }
                </button>
            </div>

            <div class={classes!("mt-3", "flex", "items-center", "gap-3", "flex-wrap", "overflow-hidden")}>
                <label class={classes!("flex", "items-center", "gap-2", "text-sm", "min-w-0")}>
                    <span class={classes!("text-[var(--muted)]", "shrink-0")}>{ "路由" }</span>
                    <select
                        key={format!("{}-route-{}", key_item.id, (*route_strategy).clone())}
                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-1.5", "text-sm")}
                        onchange={{
                            let route_strategy = route_strategy.clone();
                            Callback::from(move |event: Event| {
                                if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                    route_strategy.set(target.value());
                                }
                            })
                        }}
                    >
                        <option value="auto" selected={*route_strategy == "auto"}>{ "自动 (按额度)" }</option>
                        <option value="fixed" selected={*route_strategy == "fixed"}>{ "绑定账号" }</option>
                    </select>
                </label>
                if *route_strategy == "fixed" {
                    <label class={classes!("flex", "items-center", "gap-2", "text-sm", "min-w-0")}>
                        <span class={classes!("text-[var(--muted)]", "shrink-0")}>{ "单账号组" }</span>
                        <select
                            key={format!("{}-group-fixed-{}", key_item.id, (*account_group_id).clone())}
                            class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-1.5", "text-sm", "max-w-[220px]", "truncate")}
                            onchange={{
                                let account_group_id = account_group_id.clone();
                                Callback::from(move |event: Event| {
                                    if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                        account_group_id.set(target.value());
                                    }
                                })
                            }}
                        >
                            <option value="" selected={(*account_group_id).is_empty()}>{ "-- 选择组 --" }</option>
                            { for fixed_route_groups.iter().map(|group| html! {
                                <option value={group.id.clone()} selected={*account_group_id == group.id}>
                                    { format!(
                                        "{} ({})",
                                        group.name,
                                        group
                                            .single_account_name
                                            .clone()
                                            .unwrap_or_else(|| format!("{} 个账号", group.account_count))
                                    ) }
                                </option>
                            }) }
                        </select>
                    </label>
                } else {
                    <label class={classes!("flex", "items-center", "gap-2", "text-sm", "min-w-0")}>
                        <span class={classes!("text-[var(--muted)]", "shrink-0")}>{ "账号组" }</span>
                        <select
                            key={format!("{}-group-auto-{}", key_item.id, (*account_group_id).clone())}
                            class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-1.5", "text-sm", "max-w-[220px]", "truncate")}
                            onchange={{
                                let account_group_id = account_group_id.clone();
                                Callback::from(move |event: Event| {
                                    if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                        account_group_id.set(target.value());
                                    }
                                })
                            }}
                        >
                            <option value="" selected={(*account_group_id).is_empty()}>{ "全账号池" }</option>
                            { for props.account_groups.iter().map(|group| html! {
                                <option value={group.id.clone()} selected={*account_group_id == group.id}>{ format!("{} ({} 个账号)", group.name, group.account_count) }</option>
                            }) }
                        </select>
                    </label>
                }
                <span class={classes!("text-xs", "text-[var(--muted)]", "min-w-0", "break-all")}>
                    { current_route_summary }
                </span>
            </div>

            <div class={classes!("mt-3", "flex", "flex-wrap", "items-center", "gap-4", "text-xs", "text-[var(--muted)]")}>
                <span>{ format!("剩余 {}", format_number_i64(key_item.remaining_billable)) }</span>
                <span>{ format!("输入 {}", format_number_u64(key_item.usage_input_uncached_tokens)) }</span>
                <span>{ format!("缓存 {}", format_number_u64(key_item.usage_input_cached_tokens)) }</span>
                <span>{ format!("输出 {}", format_number_u64(key_item.usage_output_tokens)) }</span>
                <span>{ format!(
                    "并发 {}",
                    key_item.request_max_concurrency.map(|value| value.to_string()).unwrap_or_else(|| "∞".to_string())
                ) }</span>
                <span>{ format!(
                    "间隔 {}ms",
                    key_item.request_min_start_interval_ms.map(|value| value.to_string()).unwrap_or_else(|| "∞".to_string())
                ) }</span>
                <span>{ if key_item.codex_image_standalone_generation_enabled { "独立生图 on" } else { "独立生图 off" } }</span>
                <span>{ if key_item.codex_image_direct_generation_enabled { "直连生图 on" } else { "直连生图 off" } }</span>
                if key_item.provider_type == "codex" {
                    <span>{ format!("Image2 {}", format_number_u64(key_item.codex_image_usage_tokens)) }</span>
                    if key_item.codex_image_usage_missing_events > 0 {
                        <span>{ format!("image partial {}", key_item.codex_image_usage_missing_events) }</span>
                    }
                }
                <span>{ format!("Credit {}", key_credit_display(&key_item)) }</span>
                if key_item.usage_credit_missing_events > 0 {
                    <span>{ format!("partial {}", key_item.usage_credit_missing_events) }</span>
                }
            </div>

            if let Some(feedback) = (*feedback).clone() {
                <p class={classes!("mt-2", "m-0", "text-xs", "text-[var(--muted)]")}>{ feedback }</p>
            }
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct AccountGroupEditorCardProps {
    pub(crate) group_item: AdminAccountGroupView,
    pub(crate) accounts: Vec<AccountSummaryView>,
    pub(crate) on_changed: Callback<()>,
    pub(crate) on_flash: Callback<(String, bool)>,
}

#[function_component(AccountGroupEditorCard)]
pub(crate) fn account_group_editor_card(props: &AccountGroupEditorCardProps) -> Html {
    let name = use_state(|| props.group_item.name.clone());
    let account_names =
        use_state(|| sanitize_auto_account_names(&props.group_item.account_names, &props.accounts));
    let expanded = use_state(|| false);
    let saving = use_state(|| false);
    let feedback = use_state(|| None::<String>);

    {
        let group_item = props.group_item.clone();
        let accounts = props.accounts.clone();
        let name = name.clone();
        let account_names = account_names.clone();
        use_effect_with((props.group_item.clone(), props.accounts.clone()), move |_| {
            name.set(group_item.name.clone());
            account_names.set(sanitize_auto_account_names(&group_item.account_names, &accounts));
            || ()
        });
    }

    let on_toggle_account = {
        let account_names = account_names.clone();
        Callback::from(move |account_name: String| {
            let mut names = (*account_names).clone();
            if let Some(index) = names.iter().position(|name| name == &account_name) {
                names.remove(index);
            } else {
                names.push(account_name);
                names.sort();
            }
            account_names.set(names);
        })
    };

    let on_save = {
        let group_id = props.group_item.id.clone();
        let name = name.clone();
        let account_names = account_names.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        let on_changed = props.on_changed.clone();
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
            let on_changed = on_changed.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                match patch_admin_llm_gateway_account_group(
                    &group_id,
                    PatchAdminAccountGroupInput {
                        name: Some(&name_value),
                        account_names: Some(account_names_value.as_slice()),
                    },
                )
                .await
                {
                    Ok(_) => {
                        feedback.set(Some("已保存".to_string()));
                        on_flash.emit((format!("已保存账号组 `{}`", name_value), false));
                        on_changed.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("保存账号组失败\n{err}"), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_delete = {
        let group_id = props.group_item.id.clone();
        let group_name = props.group_item.name.clone();
        let on_changed = props.on_changed.clone();
        let on_flash = props.on_flash.clone();
        let saving = saving.clone();
        Callback::from(move |_| {
            if !confirm_destructive("确认删除这个账号组？") {
                return;
            }
            let group_id = group_id.clone();
            let group_name = group_name.clone();
            let on_changed = on_changed.clone();
            let on_flash = on_flash.clone();
            let saving = saving.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                match delete_admin_llm_gateway_account_group(&group_id).await {
                    Ok(_) => {
                        on_flash.emit((format!("已删除账号组 `{}`", group_name), false));
                        on_changed.emit(());
                    },
                    Err(err) => {
                        on_flash.emit((format!("删除账号组失败\n{err}"), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    html! {
        <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
            <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                <div>
                    <h3 class={classes!("m-0", "text-base", "font-bold")}>{ props.group_item.name.clone() }</h3>
                    <p class={classes!("mt-1", "mb-0", "text-xs", "text-[var(--muted)]")}>
                        {
                            if props.group_item.account_names.is_empty() {
                                "没有成员账号".to_string()
                            } else {
                                format!("成员: {}", props.group_item.account_names.join(", "))
                            }
                        }
                    </p>
                </div>
                <div class={classes!("flex", "items-center", "gap-2")}>
                    <span class={classes!("text-xs", "text-[var(--muted)]")}>{ format!("{} 个账号", props.group_item.account_names.len()) }</span>
                    <button
                        type="button"
                        class={classes!("btn-terminal")}
                        onclick={{
                            let expanded = expanded.clone();
                            Callback::from(move |_| expanded.set(!*expanded))
                        }}
                    >
                        { if *expanded { "收起 ▲" } else { "展开 ▼" } }
                    </button>
                    <button class={classes!("btn-terminal", "text-red-600", "dark:text-red-300")} onclick={on_delete} disabled={*saving}>
                        { "删除" }
                    </button>
                </div>
            </div>

            if *expanded {
                <label class={classes!("mt-3", "block", "text-sm")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "组名" }</span>
                    <input
                        type="text"
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
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

                <div class={classes!("mt-3", "space-y-2")}>
                    <div class={classes!("text-sm", "text-[var(--muted)]")}>{ "成员账号" }</div>
                    <div class={classes!("grid", "gap-2", "xl:grid-cols-2")}>
                        { for props.accounts.iter().map(|account| {
                            let checked = account_names.iter().any(|name| name == &account.name);
                            let account_name = account.name.clone();
                            let on_toggle_account = on_toggle_account.clone();
                            html! {
                                <label class={classes!(
                                    "flex", "cursor-pointer", "items-center", "gap-3", "rounded-lg", "border", "px-3", "py-2.5",
                                    if checked {
                                        "border-sky-500/30 bg-sky-500/8"
                                    } else {
                                        "border-[var(--border)] bg-[var(--surface-alt)]"
                                    }
                                )}>
                                    <input
                                        type="checkbox" class={classes!("min-h-0", "w-auto")}
                                        checked={checked}
                                        onchange={Callback::from(move |_| on_toggle_account.emit(account_name.clone()))}
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
                </div>

                <div class={classes!("mt-4", "flex", "items-center", "justify-between", "gap-3")}>
                    <span class={classes!("text-xs", "text-[var(--muted)]")}>
                        { format!("当前成员: {}", if account_names.is_empty() { "无".to_string() } else { account_names.join(", ") }) }
                    </span>
                    <button class={classes!("btn-terminal", "btn-terminal-primary")} onclick={on_save} disabled={*saving}>
                        { if *saving { "保存中..." } else { "保存账号组" } }
                    </button>
                </div>

                if let Some(feedback) = (*feedback).clone() {
                    <p class={classes!("mt-2", "m-0", "text-xs", "text-[var(--muted)]")}>{ feedback }</p>
                }
            }
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct ProxyConfigEditorCardProps {
    pub(crate) proxy_config: AdminUpstreamProxyConfigView,
    pub(crate) on_changed: Callback<()>,
    pub(crate) on_copy: Callback<(String, String)>,
    pub(crate) on_flash: Callback<(String, bool)>,
}

/// Editable fields for the proxy-config form. Grouped into a single struct so
/// every field update is one `form.set(next)` instead of juggling five
/// independent `UseStateHandle`s across effect + save callback.
#[derive(Clone, PartialEq)]
struct ProxyForm {
    name: String,
    proxy_url: String,
    proxy_username: String,
    proxy_password: String,
    status: String,
}

impl ProxyForm {
    fn from_config(cfg: &AdminUpstreamProxyConfigView) -> Self {
        Self {
            name: cfg.name.clone(),
            proxy_url: cfg.proxy_url.clone(),
            proxy_username: cfg.proxy_username.clone().unwrap_or_default(),
            proxy_password: cfg.proxy_password.clone().unwrap_or_default(),
            status: cfg.status.clone(),
        }
    }
}

#[function_component(ProxyConfigEditorCard)]
pub(crate) fn proxy_config_editor_card(props: &ProxyConfigEditorCardProps) -> Html {
    let proxy_config = props.proxy_config.clone();
    let can_edit_slot_metadata = proxy_config.can_edit_slot_metadata;
    let scope_node_label = proxy_config
        .scope_node_id
        .clone()
        .unwrap_or_else(|| "core".to_string());
    let effective_source_label = match proxy_config.effective_source.as_str() {
        "node_override" => "本机覆盖",
        "core" => "继承 core",
        other => other,
    };
    let form = use_state(|| ProxyForm::from_config(&proxy_config));
    let saving = use_state(|| false);
    let checking = use_state(|| None::<String>);
    let feedback = use_state(|| None::<String>);
    let traffic_snapshot = use_state(|| proxy_config.traffic_snapshot.clone());
    let refreshing_traffic = use_state(|| false);
    let traffic_badge = proxy_traffic_snapshot_badge((*traffic_snapshot).as_ref());
    let traffic_meta = proxy_traffic_snapshot_meta((*traffic_snapshot).as_ref());

    {
        let form = form.clone();
        use_effect_with(props.proxy_config.clone(), move |cfg| {
            form.set(ProxyForm::from_config(cfg));
            || ()
        });
    }

    {
        let traffic_snapshot = traffic_snapshot.clone();
        use_effect_with(props.proxy_config.traffic_snapshot.clone(), move |snapshot| {
            traffic_snapshot.set(snapshot.clone());
            || ()
        });
    }

    let on_save = {
        let proxy_id = proxy_config.id.clone();
        let form = form.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_changed = props.on_changed.clone();
        let on_flash = props.on_flash.clone();
        Callback::from(move |_| {
            let proxy_id = proxy_id.clone();
            let current = (*form).clone();
            let proxy_url = proxy_url_after_socks5h_confirmation(&current.proxy_url);
            if proxy_url != current.proxy_url.trim() {
                let mut next = current.clone();
                next.proxy_url = proxy_url.clone();
                form.set(next);
            }
            let input = PatchAdminUpstreamProxyConfigInput {
                name: if can_edit_slot_metadata {
                    Some(current.name.trim().to_string())
                } else {
                    None
                },
                proxy_url: Some(proxy_url),
                proxy_username: {
                    let value = current.proxy_username.trim().to_string();
                    if value.is_empty() {
                        None
                    } else {
                        Some(value)
                    }
                },
                proxy_password: {
                    let value = current.proxy_password.trim().to_string();
                    if value.is_empty() {
                        None
                    } else {
                        Some(value)
                    }
                },
                status: Some(current.status.trim().to_string()),
            };
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_changed = on_changed.clone();
            let on_flash = on_flash.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                match patch_admin_llm_gateway_proxy_config(&proxy_id, &input).await {
                    Ok(_) => {
                        feedback.set(Some("Saved.".to_string()));
                        on_flash.emit(("已保存代理配置".to_string(), false));
                        on_changed.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("保存代理配置失败\n{err}"), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_delete = {
        let proxy_id = proxy_config.id.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_changed = props.on_changed.clone();
        let on_flash = props.on_flash.clone();
        Callback::from(move |_| {
            if !confirm_destructive("确认删除这个代理配置？绑定该配置的账号会回退到默认行为。")
            {
                return;
            }
            let proxy_id = proxy_id.clone();
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_changed = on_changed.clone();
            let on_flash = on_flash.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                match delete_admin_llm_gateway_proxy_config(&proxy_id).await {
                    Ok(_) => {
                        on_flash.emit(("已删除代理配置".to_string(), false));
                        on_changed.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("删除代理配置失败\n{err}"), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_reset_override = {
        let proxy_id = proxy_config.id.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_changed = props.on_changed.clone();
        let on_flash = props.on_flash.clone();
        Callback::from(move |_| {
            if !confirm_destructive("确认移除这个节点上的代理覆盖？移除后会继承 core 配置。")
            {
                return;
            }
            let proxy_id = proxy_id.clone();
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_changed = on_changed.clone();
            let on_flash = on_flash.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                match reset_admin_llm_gateway_proxy_config_override(&proxy_id).await {
                    Ok(_) => {
                        feedback.set(Some("Override reset.".to_string()));
                        on_flash.emit(("已移除本机代理覆盖".to_string(), false));
                        on_changed.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("移除本机代理覆盖失败\n{err}"), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_check_provider = {
        let proxy_id = proxy_config.id.clone();
        let checking = checking.clone();
        let feedback = feedback.clone();
        let on_changed = props.on_changed.clone();
        let on_flash = props.on_flash.clone();
        Callback::from(move |(provider_type, full_chain): (String, bool)| {
            let proxy_id = proxy_id.clone();
            let checking = checking.clone();
            let feedback = feedback.clone();
            let on_changed = on_changed.clone();
            let on_flash = on_flash.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if (*checking).is_some() {
                    return;
                }
                let action_key = format!(
                    "{}-{}",
                    provider_type,
                    if full_chain { "full-chain" } else { "connectivity" }
                );
                let action_label = if full_chain {
                    format!("{} 全链路", provider_type.to_uppercase())
                } else {
                    provider_type.to_uppercase()
                };
                checking.set(Some(action_key));
                let result = if full_chain {
                    check_admin_llm_gateway_proxy_config_full_chain(&proxy_id, &provider_type).await
                } else {
                    check_admin_llm_gateway_proxy_config(&proxy_id, &provider_type).await
                };
                match result {
                    Ok(result) => {
                        let message = format_proxy_check_message(&result);
                        feedback.set(Some(if result.ok {
                            format!("{action_label} 检查完成")
                        } else {
                            format!("{action_label} 检查失败")
                        }));
                        on_flash.emit((message, !result.ok));
                        on_changed.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("{action_label} 代理检查失败\n{err}"), true));
                    },
                }
                checking.set(None);
            });
        })
    };

    let on_refresh_traffic = {
        let proxy_id = proxy_config.id.clone();
        let traffic_snapshot = traffic_snapshot.clone();
        let refreshing_traffic = refreshing_traffic.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        Callback::from(move |_| {
            if *refreshing_traffic {
                return;
            }
            let proxy_id = proxy_id.clone();
            let traffic_snapshot = traffic_snapshot.clone();
            let refreshing_traffic = refreshing_traffic.clone();
            let feedback = feedback.clone();
            let on_flash = on_flash.clone();
            wasm_bindgen_futures::spawn_local(async move {
                refreshing_traffic.set(true);
                match refresh_admin_llm_gateway_proxy_traffic(&proxy_id).await {
                    Ok(response) => {
                        traffic_snapshot.set(Some(response.traffic_snapshot));
                        feedback.set(Some("Traffic refreshed".to_string()));
                        on_flash.emit(("Refreshed proxy traffic".to_string(), false));
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((format!("Failed to refresh proxy traffic\n{err}"), true));
                    },
                }
                refreshing_traffic.set(false);
            });
        })
    };

    html! {
        <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
            <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                <div>
                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                        <h3 class={classes!("m-0", "text-base", "font-semibold")}>{ props.proxy_config.name.clone() }</h3>
                        <span class={classes!("inline-flex", "items-center", "rounded-full", "px-2.5", "py-1", "text-[11px]", "font-semibold", "uppercase", "tracking-[0.16em]",
                            if props.proxy_config.status == "active" { "bg-emerald-500/12 text-emerald-700 dark:text-emerald-200" } else { "bg-slate-500/12 text-slate-700 dark:text-slate-200" })}>
                            { props.proxy_config.status.clone() }
                        </span>
                        <span class={classes!("inline-flex", "items-center", "rounded-full", "bg-cyan-500/12", "px-2.5", "py-1", "text-[11px]", "font-semibold", "text-cyan-700", "dark:text-cyan-200")}>
                            { effective_source_label }
                        </span>
                        <span class={classes!("inline-flex", "items-center", "rounded-full", "bg-[var(--surface-alt)]", "px-2.5", "py-1", "text-[11px]", "font-semibold", "text-[var(--muted)]")}>
                            { format!("scope: {}", scope_node_label) }
                        </span>
                        <span class={classes!("inline-flex", "items-center", "rounded-full", "bg-teal-500/10", "px-2.5", "py-1", "text-[11px]", "font-semibold", "text-teal-700", "dark:text-teal-200")}>
                            { traffic_badge }
                        </span>
                    </div>
                    <p class={classes!("mt-2", "mb-0", "text-xs", "font-mono", "text-[var(--muted)]")}>
                        { format!("created {} · updated {} · {}", format_ms(props.proxy_config.created_at), format_ms(props.proxy_config.updated_at), traffic_meta) }
                    </p>
                    <div class={classes!("mt-3", "grid", "gap-2", "sm:grid-cols-2")}>
                        <div class={classes!("rounded-lg", "border", "px-3", "py-2", "text-xs", proxy_endpoint_check_tone(props.proxy_config.latest_codex_check.as_ref()))}>
                            { format_proxy_endpoint_check_summary("Codex", props.proxy_config.latest_codex_check.as_ref()) }
                        </div>
                        <div class={classes!("rounded-lg", "border", "px-3", "py-2", "text-xs", proxy_endpoint_check_tone(props.proxy_config.latest_kiro_check.as_ref()))}>
                            { format_proxy_endpoint_check_summary("Kiro", props.proxy_config.latest_kiro_check.as_ref()) }
                        </div>
                    </div>
                </div>
                <div class={classes!("flex", "items-center", "gap-2")}>
                    { copy_icon_button(&props.proxy_config.proxy_url, &props.on_copy) }
                </div>
            </div>

            <div class={classes!("mt-4", "grid", "gap-3", "md:grid-cols-2")}>
                <label class={classes!("text-sm")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "Name" }</span>
                    <input
                        type="text"
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2")}
                        value={form.name.clone()}
                        disabled={!can_edit_slot_metadata}
                        oninput={{
                            let form = form.clone();
                            Callback::from(move |event: InputEvent| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    let mut next = (*form).clone();
                                    next.name = target.value();
                                    form.set(next);
                                }
                            })
                        }}
                    />
                </label>
                <label class={classes!("text-sm")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "Status" }</span>
                    <select
                        key={format!("proxy-config-status-{}-{}", proxy_config.id, form.status)}
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2")}
                        value={form.status.clone()}
                        onchange={{
                            let form = form.clone();
                            Callback::from(move |event: Event| {
                                if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                    let mut next = (*form).clone();
                                    next.status = target.value();
                                    form.set(next);
                                }
                            })
                        }}
                    >
                        <option value="active" selected={form.status == "active"}>{ "active" }</option>
                        <option value="disabled" selected={form.status == "disabled"}>{ "disabled" }</option>
                    </select>
                </label>
                <label class={classes!("text-sm", "md:col-span-2")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "Proxy URL" }</span>
                    <input
                        type="text"
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "font-mono")}
                        value={form.proxy_url.clone()}
                        oninput={{
                            let form = form.clone();
                            Callback::from(move |event: InputEvent| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    let mut next = (*form).clone();
                                    next.proxy_url = target.value();
                                    form.set(next);
                                }
                            })
                        }}
                    />
                </label>
                <label class={classes!("text-sm")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "Proxy Username" }</span>
                    <input
                        type="text"
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2")}
                        value={form.proxy_username.clone()}
                        oninput={{
                            let form = form.clone();
                            Callback::from(move |event: InputEvent| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    let mut next = (*form).clone();
                                    next.proxy_username = target.value();
                                    form.set(next);
                                }
                            })
                        }}
                    />
                </label>
                <label class={classes!("text-sm")}>
                    <span class={classes!("text-[var(--muted)]")}>{ "Proxy Password" }</span>
                    <input
                        type="text"
                        class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2")}
                        value={form.proxy_password.clone()}
                        oninput={{
                            let form = form.clone();
                            Callback::from(move |event: InputEvent| {
                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                    let mut next = (*form).clone();
                                    next.proxy_password = target.value();
                                    form.set(next);
                                }
                            })
                        }}
                    />
                </label>
            </div>

            <div class={classes!("mt-4", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3")}>
                <div class={classes!("flex", "items-center", "justify-between", "gap-3")}>
                    <div class={classes!("min-w-0")}>
                        <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Visible Credentials" }</div>
                        <code class={classes!("mt-2", "block", "break-all", "font-mono", "text-xs")}>
                            { format!("{} @ {}", props.proxy_config.proxy_username.clone().unwrap_or_else(|| "-".to_string()), props.proxy_config.proxy_url.clone()) }
                        </code>
                        if let Some(password) = props.proxy_config.proxy_password.as_deref() {
                            <code class={classes!("mt-1", "block", "break-all", "font-mono", "text-xs")}>
                                { password }
                            </code>
                        }
                    </div>
                    <div class={classes!("flex", "items-center", "gap-2")}>
                        { copy_icon_button(&props.proxy_config.proxy_url, &props.on_copy) }
                        if let Some(username) = props.proxy_config.proxy_username.as_deref() {
                            { copy_icon_button(username, &props.on_copy) }
                        }
                        if let Some(password) = props.proxy_config.proxy_password.as_deref() {
                            { copy_icon_button(password, &props.on_copy) }
                        }
                    </div>
                </div>
            </div>

            <div class={classes!("mt-4", "flex", "items-center", "gap-2", "flex-wrap")}>
                <button
                    class={classes!("btn-terminal")}
                    onclick={on_refresh_traffic}
                    disabled={*refreshing_traffic}
                >
                    { if *refreshing_traffic { "Calculating..." } else { "Refresh Traffic" } }
                </button>
                <button
                    class={classes!("btn-terminal")}
                    onclick={{
                        let on_check_provider = on_check_provider.clone();
                        Callback::from(move |_| on_check_provider.emit(("codex".to_string(), false)))
                    }}
                    disabled={*saving || (*checking).is_some()}
                >
                    { if (*checking).as_deref() == Some("codex-connectivity") { "检查中..." } else { "检查 Codex" } }
                </button>
                <button
                    class={classes!("btn-terminal")}
                    onclick={{
                        let on_check_provider = on_check_provider.clone();
                        Callback::from(move |_| on_check_provider.emit(("kiro".to_string(), false)))
                    }}
                    disabled={*saving || (*checking).is_some()}
                >
                    { if (*checking).as_deref() == Some("kiro-connectivity") { "检查中..." } else { "检查 Kiro" } }
                </button>
                <button
                    class={classes!("btn-terminal")}
                    onclick={{
                        let on_check_provider = on_check_provider.clone();
                        Callback::from(move |_| on_check_provider.emit(("codex".to_string(), true)))
                    }}
                    disabled={*saving || (*checking).is_some()}
                >
                    { if (*checking).as_deref() == Some("codex-full-chain") { "请求中..." } else { "全链路 Codex" } }
                </button>
                <button
                    class={classes!("btn-terminal")}
                    onclick={{
                        let on_check_provider = on_check_provider.clone();
                        Callback::from(move |_| on_check_provider.emit(("kiro".to_string(), true)))
                    }}
                    disabled={*saving || (*checking).is_some()}
                >
                    { if (*checking).as_deref() == Some("kiro-full-chain") { "请求中..." } else { "全链路 Kiro" } }
                </button>
                <button class={classes!("btn-terminal", "btn-terminal-primary")} onclick={on_save.clone()} disabled={*saving}>
                    { if *saving { "保存中..." } else { "保存" } }
                </button>
                if props.proxy_config.has_node_override {
                    <button class={classes!("btn-terminal")} onclick={on_reset_override} disabled={*saving}>
                        { "移除本机覆盖" }
                    </button>
                }
                if can_edit_slot_metadata {
                    <button class={classes!("btn-terminal", "text-red-600", "dark:text-red-400")} onclick={on_delete} disabled={*saving}>
                        { "删除" }
                    </button>
                }
            </div>

            if let Some(feedback) = (*feedback).clone() {
                <p class={classes!("mt-2", "m-0", "text-xs", "text-[var(--muted)]")}>{ feedback }</p>
            }
        </article>
    }
}

/// One navigation card on the LLM overview linking to a dedicated section.
fn overview_nav_card(title: &'static str, desc: &'static str, to: Route) -> Html {
    html! {
        <Link<Route>
            to={to}
            classes={classes!(
                "panel", "p-4", "no-underline", "text-[var(--foreground)]",
                "transition-colors", "hover:border-[var(--ring)]"
            )}
        >
            <div class={classes!("flex", "items-center", "justify-between", "gap-2")}>
                <span class={classes!("font-semibold")}>{ title }</span>
                <span class={classes!("text-[var(--muted-foreground)]")}>{ "\u{2192}" }</span>
            </div>
            <div class={classes!("mt-1", "text-xs", "text-[var(--muted-foreground)]")}>{ desc }</div>
        </Link<Route>>
    }
}

/// The route for one LLM admin section id (inverse of the router mapping).
fn llm_tab_route(tab: &str) -> Route {
    match tab {
        TAB_KEYS => Route::AdminLlmGatewayKeys,
        TAB_GROUPS => Route::AdminLlmGatewayGroups,
        TAB_ACCOUNTS => Route::AdminLlmGatewayAccounts,
        TAB_USAGE => Route::AdminLlmGatewayUsage,
        TAB_JOURNAL => Route::AdminLlmGatewayJournal,
        TAB_REQUESTS => Route::AdminLlmGatewayRequests,
        TAB_SETTINGS => Route::AdminLlmGatewaySettings,
        _ => Route::AdminLlmGateway,
    }
}

#[function_component(AdminLlmGatewayPage)]
/// LLM gateway overview (`/admin/llm-gateway`): key/quota stat tiles, the
/// pending-request counter, and navigation cards into the dedicated section
/// pages (keys / groups / accounts / usage / journal / requests / settings).
pub fn admin_llm_gateway_page() -> Html {
    let keys_summary = use_state(AdminLlmGatewayKeysSummaryView::default);
    let total_pending = use_state(|| 0_usize);
    let loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0_u32);

    // Legacy deep links used `?tab=`; forward them once onto the dedicated
    // per-section routes so old bookmarks keep working.
    {
        let navigator = use_navigator();
        use_effect_with((), move |_| {
            let legacy = crate::pages::llm_access_shared::initial_tab_from_url(
                &[
                    TAB_KEYS,
                    TAB_GROUPS,
                    TAB_ACCOUNTS,
                    TAB_USAGE,
                    TAB_JOURNAL,
                    TAB_REQUESTS,
                    TAB_SETTINGS,
                ],
                "",
            );
            if !legacy.is_empty() {
                if let Some(navigator) = navigator {
                    navigator.replace(&llm_tab_route(&legacy));
                }
            }
            || ()
        });
    }

    let on_reload = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_: ()| refresh_tick.set(refresh_tick.wrapping_add(1)))
    };

    // Key summary for the stat tiles plus the pending-request scan; the scan
    // counts the same "needs action" statuses the requests page surfaces,
    // over the first page of each queue.
    {
        let keys_summary = keys_summary.clone();
        let total_pending = total_pending.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let keys_summary = keys_summary.clone();
            let total_pending = total_pending.clone();
            let loading = loading.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let result = async {
                    let (keys_resp, token_resp, contribution_resp, sponsor_resp) = futures::join!(
                        fetch_admin_llm_gateway_keys_page(1, 0),
                        fetch_admin_llm_gateway_token_requests(
                            &AdminLlmGatewayTokenRequestsQuery {
                                status: None,
                                limit: Some(PENDING_SCAN_PAGE_SIZE),
                                offset: Some(0),
                            }
                        ),
                        fetch_admin_llm_gateway_account_contribution_requests(
                            &AdminLlmGatewayAccountContributionRequestsQuery {
                                status: None,
                                limit: Some(PENDING_SCAN_PAGE_SIZE),
                                offset: Some(0),
                            }
                        ),
                        fetch_admin_llm_gateway_sponsor_requests(
                            &AdminLlmGatewaySponsorRequestsQuery {
                                status: None,
                                limit: Some(PENDING_SCAN_PAGE_SIZE),
                                offset: Some(0),
                            }
                        ),
                    );
                    Ok::<_, String>((keys_resp?, token_resp?, contribution_resp?, sponsor_resp?))
                }
                .await;
                match result {
                    Ok((keys_resp, token_resp, contribution_resp, sponsor_resp)) => {
                        keys_summary.set(keys_resp.summary);
                        let pending = token_resp
                            .requests
                            .iter()
                            .filter(|r| r.status == "pending")
                            .count()
                            + contribution_resp
                                .requests
                                .iter()
                                .filter(|r| {
                                    r.status == "pending"
                                        || r.status == "failed"
                                        || r.status == "validated"
                                })
                                .count()
                            + sponsor_resp
                                .requests
                                .iter()
                                .filter(|r| {
                                    r.status == "submitted" || r.status == "payment_email_sent"
                                })
                                .count();
                        total_pending.set(pending);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let key_summary = *keys_summary;
    let total_remaining = key_summary.remaining_billable_sum;
    let public_visible_count = key_summary.public_visible_count;
    let active_key_count = key_summary.active_count;
    let total_quota = key_summary.quota_billable_limit_sum;
    let total_used = key_summary.usage_billable_tokens_sum;
    let credit_keys_present =
        key_summary.usage_credit_total > 0.0 || key_summary.usage_credit_missing_events > 0;
    let total_credit_used = key_summary.usage_credit_total;
    let total_credit_missing_events = key_summary.usage_credit_missing_events;
    // Derive usage percentage from quota and remaining (billable-token basis).
    let usage_percent = if total_quota > 0 {
        let used = total_quota as f64 - (total_remaining.max(0) as f64);
        (used / total_quota as f64 * 100.0)
            .clamp(0.0, 100.0)
            .round() as u64
    } else {
        0
    };
    let total_pending = *total_pending;

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "LLM Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Overview" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::Admin} classes={classes!("linkbtn")}>{ "Admin 首页" }</Link<Route>>
                        <Link<Route> to={Route::LlmAccess} classes={classes!("linkbtn")}>{ "公共页" }</Link<Route>>
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

                <section class={classes!("panel", "p-5")}>
                    <div class={classes!("grid", "gap-3", "grid-cols-2", "xl:grid-cols-4")}>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Key 总数" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ key_summary.total }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "公开 / Active" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format!("{} / {}", public_visible_count, active_key_count) }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "剩余额度" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format_number_i64(total_remaining) }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "总额度" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format_number_u64(total_quota) }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "已用量" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format_number_u64(total_used) }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("flex", "items-center", "justify-between")}>
                                <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "使用率" }</div>
                                <div class={classes!("font-mono", "text-sm", "font-bold", "text-[var(--text)]")}>{ format!("{}%", usage_percent) }</div>
                            </div>
                            <div class={classes!("mt-2", "h-2", "w-full", "overflow-hidden", "rounded-full", "bg-[var(--surface-alt)]")}>
                                <div
                                    class={classes!(
                                        "h-full", "rounded-full",
                                        "transition-all", "duration-700", "ease-out",
                                        if usage_percent >= 90 { "bg-red-500" }
                                        else if usage_percent >= 70 { "bg-amber-500" }
                                        else { "bg-emerald-500" }
                                    )}
                                    style={format!("width: {}%", usage_percent)}
                                />
                            </div>
                            <div class={classes!("mt-1.5", "flex", "justify-between", "font-mono", "text-[10px]", "text-[var(--muted)]")}>
                                <span>{ format!("剩余 {}", format_number_i64(total_remaining)) }</span>
                                <span>{ format!("总计 {}", format_number_u64(total_quota)) }</span>
                            </div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Credit 已记录" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>
                                { if credit_keys_present { format_credit4(total_credit_used) } else { "-".to_string() } }
                            </div>
                            if total_credit_missing_events > 0 {
                                <div class={classes!("mt-1", "text-xs", "text-amber-700", "dark:text-amber-200")}>
                                    { format!("partial · {} events missing", total_credit_missing_events) }
                                </div>
                            }
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "待审核" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black", if total_pending > 0 { "text-amber-600" } else { "" })}>{ total_pending }</div>
                        </div>
                    </div>
                </section>

                <section class={classes!("grid", "gap-3", "sm:grid-cols-2", "xl:grid-cols-3")}>
                    { overview_nav_card("Keys", "创建与管理 API key、配额与路由策略", Route::AdminLlmGatewayKeys) }
                    { overview_nav_card("Groups", "维护账号组，供 key 固定/自动路由选择", Route::AdminLlmGatewayGroups) }
                    { overview_nav_card("Accounts", "Codex 账号导入、状态与调度设置", Route::AdminLlmGatewayAccounts) }
                    { overview_nav_card("Usage", "分页查看用量事件与详情", Route::AdminLlmGatewayUsage) }
                    { overview_nav_card("Journal", "浏览 usage journal 热数据预览", Route::AdminLlmGatewayJournal) }
                    { overview_nav_card("Requests", "审核 token 许愿、账号贡献与赞助请求", Route::AdminLlmGatewayRequests) }
                    { overview_nav_card("Settings", "运行时配置、代理绑定与共享代理槽位", Route::AdminLlmGatewaySettings) }
                    { overview_nav_card("Monitor", "实时请求与账号健康监控", Route::AdminLlmGatewayMonitor) }
                    { overview_nav_card("Moderation", "关键词审核与拦截配置", Route::AdminLlmGatewayModeration) }
                </section>
            </div>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_traffic_window_label_matches_retained_analytics_window() {
        assert_eq!(proxy_traffic_window_label(7), "retained 7d traffic");
        assert_eq!(proxy_traffic_window_label(14), "retained 14d traffic");
        assert_eq!(proxy_traffic_window_label(30), "30d traffic");
        assert_eq!(proxy_traffic_window_label(45), "30d traffic");
    }

    #[test]
    fn socks5h_recommendation_converts_plain_socks5_scheme() {
        assert_eq!(
            recommended_socks5h_proxy_url("socks5://user:pass@proxy.example:443").as_deref(),
            Some("socks5h://user:pass@proxy.example:443")
        );
        assert_eq!(
            recommended_socks5h_proxy_url("  socks5://proxy.example:1080  ").as_deref(),
            Some("socks5h://proxy.example:1080")
        );
    }

    #[test]
    fn socks5h_recommendation_ignores_other_schemes() {
        assert_eq!(recommended_socks5h_proxy_url("socks5h://user:pass@proxy.example:443"), None);
        assert_eq!(recommended_socks5h_proxy_url("http://user:pass@proxy.example:443"), None);
        assert_eq!(recommended_socks5h_proxy_url("ééééé"), None);
        assert_eq!(recommended_socks5h_proxy_url(""), None);
    }

    #[test]
    fn proxy_traffic_snapshot_helpers_show_uncalculated_and_persisted_badge_state() {
        assert_eq!(proxy_traffic_snapshot_badge(None), "traffic not calculated");
        assert_eq!(proxy_traffic_snapshot_meta(None), "traffic not calculated");

        let snapshot = AdminProxyTrafficSnapshotView {
            refreshed_at_ms: 1_700_000_000_000,
            retention_days: 7,
            totals: crate::api::AdminLlmGatewayProxyTrafficTotalsView {
                event_count: 42,
                request_bytes: 512,
                response_bytes: 1_024,
                total_bytes: 1_536,
            },
            ..AdminProxyTrafficSnapshotView::default()
        };

        assert!(proxy_traffic_snapshot_badge(Some(&snapshot)).starts_with("retained 7d traffic "));
    }

    #[test]
    fn llm_tab_route_round_trips_section_ids() {
        assert_eq!(llm_tab_route("overview"), Route::AdminLlmGateway);
        assert_eq!(llm_tab_route(TAB_KEYS), Route::AdminLlmGatewayKeys);
        assert_eq!(llm_tab_route(TAB_GROUPS), Route::AdminLlmGatewayGroups);
        assert_eq!(llm_tab_route(TAB_ACCOUNTS), Route::AdminLlmGatewayAccounts);
        assert_eq!(llm_tab_route(TAB_USAGE), Route::AdminLlmGatewayUsage);
        assert_eq!(llm_tab_route(TAB_JOURNAL), Route::AdminLlmGatewayJournal);
        assert_eq!(llm_tab_route(TAB_REQUESTS), Route::AdminLlmGatewayRequests);
        assert_eq!(llm_tab_route(TAB_SETTINGS), Route::AdminLlmGatewaySettings);
        assert_eq!(llm_tab_route("unknown"), Route::AdminLlmGateway);
    }

    #[test]
    fn usage_journal_preview_message_prefers_summary_content() {
        let event = crate::api::AdminUsageJournalPreviewEventView {
            last_message_content: Some("hello".to_string()),
            ..crate::api::AdminUsageJournalPreviewEventView::default()
        };

        assert_eq!(usage_journal_preview_message(&event), "hello");
    }

    #[test]
    fn usage_journal_preview_message_presence_detects_real_content() {
        let with_message = crate::api::AdminUsageJournalPreviewEventView {
            last_message_content: Some("hello".to_string()),
            ..crate::api::AdminUsageJournalPreviewEventView::default()
        };
        let without_message = crate::api::AdminUsageJournalPreviewEventView {
            last_message_content: Some("   ".to_string()),
            ..crate::api::AdminUsageJournalPreviewEventView::default()
        };

        assert!(usage_journal_preview_has_full_message(&with_message));
        assert!(!usage_journal_preview_has_full_message(&without_message));
    }

    #[test]
    fn kiro_usage_account_label_distinguishes_uncaptured_account_from_legacy_auth() {
        assert_eq!(
            usage_account_label(
                &None,
                "https://ackingliu.top/api/kiro-gateway/v1/messages",
                "/generateAssistantResponse",
            ),
            "not captured"
        );
        assert_eq!(
            usage_account_label(
                &None,
                "https://ackingliu.top/api/llm-gateway/v1/responses",
                "/v1/responses"
            ),
            "legacy auth"
        );
    }

    #[test]
    fn latency_breakdown_marks_first_sse_not_applicable_when_stream_never_started() {
        let summary = format_latency_breakdown(LatencyBreakdown {
            latency_ms: 502,
            routing_wait_ms: Some(12),
            upstream_headers_ms: Some(34),
            post_headers_body_ms: None,
            request_body_bytes: Some(512),
            request_body_read_ms: Some(1),
            request_json_parse_ms: Some(0),
            pre_handler_ms: Some(2),
            first_sse_write_ms: None,
            stream_finish_ms: Some(502),
            other_latency_ms: None,
            quota_failover_count: 0,
        });

        assert!(summary.contains("route 12 ms"));
        assert!(summary.contains("first SSE n/a"));
    }

    #[test]
    fn stream_summary_marks_disconnect_and_formats_bytes() {
        assert_eq!(
            format_stream_summary(Some(false), Some(true), Some("message_stop"), Some(2048)),
            "state disconnect · final message_stop · bytes 2.0 KiB"
        );
        assert_eq!(usage_stream_state_label(Some(true), Some(false)), "clean");
        assert_eq!(usage_stream_state_label(None, None), "n/a");
    }

    #[test]
    fn effective_route_latency_uses_routing_diagnostics_when_column_is_missing() {
        assert_eq!(effective_routing_wait_ms(None, Some(r#"{"route_total_ms":321}"#)), Some(321));
        assert_eq!(
            effective_routing_wait_ms(Some(12), Some(r#"{"route_total_ms":321}"#)),
            Some(12)
        );
        assert_eq!(effective_routing_wait_ms(None, Some("not-json")), None);
    }

    #[test]
    fn routing_diagnostics_summary_includes_codex_failover_count() {
        let rows = routing_diagnostics_summary(
            r#"{"route_total_ms":12,"account_attempt_count":2,"failover_count":1}"#,
        );

        assert!(rows
            .iter()
            .any(|(label, value)| label == "Route total" && value == "12 ms"));
        assert!(rows
            .iter()
            .any(|(label, value)| label == "Codex failover" && value == "1"));
    }
}
