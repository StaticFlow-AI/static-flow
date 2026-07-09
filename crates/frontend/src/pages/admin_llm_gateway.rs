use std::collections::HashSet;

use gloo_timers::callback::Timeout;
use js_sys::Date;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::{use_navigator, Link};

use crate::{
    api::{
        admin_approve_and_issue_llm_gateway_account_contribution_request,
        admin_approve_and_issue_llm_gateway_token_request,
        admin_approve_llm_gateway_sponsor_request,
        admin_reject_llm_gateway_account_contribution_request,
        admin_reject_llm_gateway_token_request,
        admin_validate_llm_gateway_account_contribution_request,
        check_admin_llm_gateway_proxy_config, check_admin_llm_gateway_proxy_config_full_chain,
        delete_admin_llm_gateway_account_group, delete_admin_llm_gateway_key,
        delete_admin_llm_gateway_proxy_config, delete_admin_llm_gateway_sponsor_request,
        fetch_admin_llm_gateway_account_contribution_requests, fetch_admin_llm_gateway_keys_page,
        fetch_admin_llm_gateway_sponsor_requests, fetch_admin_llm_gateway_token_requests,
        patch_admin_llm_gateway_account_group, patch_admin_llm_gateway_key,
        patch_admin_llm_gateway_proxy_config, refresh_admin_llm_gateway_proxy_traffic,
        reset_admin_llm_gateway_proxy_config_override, AccountSummaryView,
        AdminAccountGroupOptionView, AdminAccountGroupView,
        AdminLlmGatewayAccountContributionRequestView,
        AdminLlmGatewayAccountContributionRequestsQuery, AdminLlmGatewayKeyView,
        AdminLlmGatewayKeysSummaryView, AdminLlmGatewaySponsorRequestView,
        AdminLlmGatewaySponsorRequestsQuery, AdminLlmGatewayTokenRequestView,
        AdminLlmGatewayTokenRequestsQuery, AdminProxyTrafficSnapshotView,
        AdminUpstreamProxyCheckResponse, AdminUpstreamProxyCheckTargetView,
        AdminUpstreamProxyConfigView, AdminUpstreamProxyEndpointCheckView,
        PatchAdminAccountGroupInput, PatchAdminLlmGatewayKeyRequest,
        PatchAdminUpstreamProxyConfigInput,
    },
    components::{
        empty_state::EmptyState, pagination::Pagination, status_badge::StatusBadge,
        tab_bar::render_tab_bar,
    },
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
const TOKEN_REQUEST_PAGE_SIZE: usize = 20;
const ACCOUNT_CONTRIBUTION_REQUEST_PAGE_SIZE: usize = 20;
const SPONSOR_REQUEST_PAGE_SIZE: usize = 20;
const PROXY_TRAFFIC_QUERY_WINDOW_DAYS: u64 = 30;
/// Page size for the Usage tab's server-side key filter search.
pub(crate) const USAGE_KEY_OPTION_LIMIT: usize = 20;
const TAB_OVERVIEW: &str = "overview";
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

fn copyable_token_preview(label: &str, value: &str, on_copy: &Callback<(String, String)>) -> Html {
    html! {
        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2")}>
            <div class={classes!("flex", "items-center", "justify-between", "gap-3")}>
                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>
                    { label }
                </div>
                { copy_icon_button(value, on_copy) }
            </div>
            <code class={classes!("mt-2", "block", "break-all", "text-xs", "text-[var(--text)]")}>
                { preview_text(value, 96) }
            </code>
        </div>
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
                        type="checkbox"
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
                            type="checkbox"
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
                        type="checkbox"
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
                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                    <input
                        type="checkbox"
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
                        type="checkbox"
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
                        type="checkbox"
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
                                        type="checkbox"
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

/// Props for [`AdminLlmGatewayPage`]. The active section is route-driven:
/// `/admin/llm-gateway` renders the overview and
/// `/admin/llm-gateway/{keys,groups,accounts,usage,journal,requests,settings}`
/// select the matching section.
#[derive(Properties, PartialEq, Default)]
pub struct AdminLlmGatewayPageProps {
    #[prop_or_default]
    pub tab: Option<AttrValue>,
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
pub fn admin_llm_gateway_page(props: &AdminLlmGatewayPageProps) -> Html {
    let keys_summary = use_state(AdminLlmGatewayKeysSummaryView::default);
    let token_requests = use_state(Vec::<AdminLlmGatewayTokenRequestView>::new);
    let token_request_total = use_state(|| 0_usize);
    let token_request_page = use_state(|| 1_usize);
    let token_request_loading = use_state(|| false);
    let token_request_status_filter = use_state(String::new);
    let token_request_action_inflight = use_state(HashSet::<String>::new);
    let account_contribution_requests =
        use_state(Vec::<AdminLlmGatewayAccountContributionRequestView>::new);
    let account_contribution_request_total = use_state(|| 0_usize);
    let account_contribution_request_page = use_state(|| 1_usize);
    let account_contribution_request_loading = use_state(|| false);
    let account_contribution_request_status_filter = use_state(String::new);
    let account_contribution_request_action_inflight = use_state(HashSet::<String>::new);
    let sponsor_requests = use_state(Vec::<AdminLlmGatewaySponsorRequestView>::new);
    let sponsor_request_total = use_state(|| 0_usize);
    let sponsor_request_page = use_state(|| 1_usize);
    let sponsor_request_loading = use_state(|| false);
    let sponsor_request_status_filter = use_state(String::new);
    let sponsor_request_action_inflight = use_state(HashSet::<String>::new);
    let loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);
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
    let active_tab = props
        .tab
        .as_ref()
        .map(|tab| tab.to_string())
        .unwrap_or_else(|| TAB_OVERVIEW.to_string());
    let navigator = use_navigator();
    // Legacy deep links used `?tab=`; forward them once onto the dedicated
    // per-section routes so old bookmarks keep working.
    {
        let navigator = navigator.clone();
        use_effect_with(props.tab.clone(), move |tab| {
            if tab.is_none() {
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
            }
            || ()
        });
    }
    let on_tab_click = {
        let navigator = navigator.clone();
        Callback::from(move |tab: String| {
            if let Some(navigator) = navigator.clone() {
                navigator.push(&llm_tab_route(&tab));
            }
        })
    };

    // Usage events are fetched independently so paging and key filters do not
    // need to re-fetch the rest of the admin page chrome.
    let reload_token_requests = {
        let token_requests = token_requests.clone();
        let token_request_total = token_request_total.clone();
        let token_request_page = token_request_page.clone();
        let token_request_loading = token_request_loading.clone();
        let token_request_status_filter = token_request_status_filter.clone();
        let load_error = load_error.clone();
        Callback::from(move |(requested_page, override_status): (Option<usize>, Option<String>)| {
            let token_requests = token_requests.clone();
            let token_request_total = token_request_total.clone();
            let token_request_page = token_request_page.clone();
            let token_request_loading = token_request_loading.clone();
            let token_request_status_filter = token_request_status_filter.clone();
            let load_error = load_error.clone();
            let page = requested_page.unwrap_or(*token_request_page).max(1);
            let selected_status =
                override_status.unwrap_or_else(|| (*token_request_status_filter).clone());
            token_request_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let query = AdminLlmGatewayTokenRequestsQuery {
                    status: (!selected_status.is_empty()).then_some(selected_status),
                    limit: Some(TOKEN_REQUEST_PAGE_SIZE),
                    offset: Some((page - 1) * TOKEN_REQUEST_PAGE_SIZE),
                };
                match fetch_admin_llm_gateway_token_requests(&query).await {
                    Ok(resp) => {
                        token_request_total.set(resp.total);
                        token_requests.set(resp.requests);
                        token_request_page.set(page);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                token_request_loading.set(false);
            });
        })
    };

    let reload_account_contribution_requests = {
        let account_contribution_requests = account_contribution_requests.clone();
        let account_contribution_request_total = account_contribution_request_total.clone();
        let account_contribution_request_page = account_contribution_request_page.clone();
        let account_contribution_request_loading = account_contribution_request_loading.clone();
        let account_contribution_request_status_filter =
            account_contribution_request_status_filter.clone();
        let load_error = load_error.clone();
        Callback::from(move |(requested_page, override_status): (Option<usize>, Option<String>)| {
            let account_contribution_requests = account_contribution_requests.clone();
            let account_contribution_request_total = account_contribution_request_total.clone();
            let account_contribution_request_page = account_contribution_request_page.clone();
            let account_contribution_request_loading = account_contribution_request_loading.clone();
            let account_contribution_request_status_filter =
                account_contribution_request_status_filter.clone();
            let load_error = load_error.clone();
            let page = requested_page
                .unwrap_or(*account_contribution_request_page)
                .max(1);
            let selected_status = override_status
                .unwrap_or_else(|| (*account_contribution_request_status_filter).clone());
            account_contribution_request_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let query = AdminLlmGatewayAccountContributionRequestsQuery {
                    status: (!selected_status.is_empty()).then_some(selected_status),
                    limit: Some(ACCOUNT_CONTRIBUTION_REQUEST_PAGE_SIZE),
                    offset: Some((page - 1) * ACCOUNT_CONTRIBUTION_REQUEST_PAGE_SIZE),
                };
                match fetch_admin_llm_gateway_account_contribution_requests(&query).await {
                    Ok(resp) => {
                        account_contribution_request_total.set(resp.total);
                        account_contribution_requests.set(resp.requests);
                        account_contribution_request_page.set(page);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                account_contribution_request_loading.set(false);
            });
        })
    };

    let reload_sponsor_requests = {
        let sponsor_requests = sponsor_requests.clone();
        let sponsor_request_total = sponsor_request_total.clone();
        let sponsor_request_page = sponsor_request_page.clone();
        let sponsor_request_loading = sponsor_request_loading.clone();
        let sponsor_request_status_filter = sponsor_request_status_filter.clone();
        let load_error = load_error.clone();
        Callback::from(move |(requested_page, override_status): (Option<usize>, Option<String>)| {
            let sponsor_requests = sponsor_requests.clone();
            let sponsor_request_total = sponsor_request_total.clone();
            let sponsor_request_page = sponsor_request_page.clone();
            let sponsor_request_loading = sponsor_request_loading.clone();
            let sponsor_request_status_filter = sponsor_request_status_filter.clone();
            let load_error = load_error.clone();
            let page = requested_page.unwrap_or(*sponsor_request_page).max(1);
            let selected_status =
                override_status.unwrap_or_else(|| (*sponsor_request_status_filter).clone());
            sponsor_request_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let query = AdminLlmGatewaySponsorRequestsQuery {
                    status: (!selected_status.is_empty()).then_some(selected_status),
                    limit: Some(SPONSOR_REQUEST_PAGE_SIZE),
                    offset: Some((page - 1) * SPONSOR_REQUEST_PAGE_SIZE),
                };
                match fetch_admin_llm_gateway_sponsor_requests(&query).await {
                    Ok(resp) => {
                        sponsor_request_total.set(resp.total);
                        sponsor_requests.set(resp.requests);
                        sponsor_request_page.set(page);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                sponsor_request_loading.set(false);
            });
        })
    };

    // This reload keeps the inventory, runtime config, and the current usage
    // page in sync after any admin write operation.
    // Tracks whether the tab-independent base data (config, summaries, proxy
    // configs/bindings) has been loaded once; plain tab switches and paging
    // reuse it instead of re-fetching, while mutations force a refresh.
    let reload_base_loaded = use_state(|| false);
    let reload = {
        let keys_summary = keys_summary.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        let reload_base_loaded = reload_base_loaded.clone();
        Callback::from(move |force_base: bool| {
            let keys_summary = keys_summary.clone();
            let loading = loading.clone();
            let load_error = load_error.clone();
            let reload_base_loaded = reload_base_loaded.clone();
            let refresh_base = force_base || !*reload_base_loaded;
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if refresh_base {
                    match fetch_admin_llm_gateway_keys_page(1, 0).await {
                        Ok(resp) => {
                            keys_summary.set(resp.summary);
                            reload_base_loaded.set(true);
                            load_error.set(None);
                        },
                        Err(err) => load_error.set(Some(err)),
                    }
                }
                loading.set(false);
            });
        })
    };

    {
        let reload = reload.clone();
        let active_tab = active_tab.clone();
        use_effect_with((active_tab.clone(),), move |_| {
            // Tab switches reuse the already-loaded base data; the first run
            // (mount) still fetches it because nothing is loaded yet.
            reload.emit(false);
            || ()
        });
    }

    {
        let reload_token_requests = reload_token_requests.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        use_effect_with((), move |_| {
            reload_token_requests.emit((Some(1), Some(String::new())));
            reload_account_contribution_requests.emit((Some(1), Some(String::new())));
            reload_sponsor_requests.emit((Some(1), Some(String::new())));
            || ()
        });
    }


    let token_request_total_pages = (*token_request_total)
        .max(1)
        .div_ceil(TOKEN_REQUEST_PAGE_SIZE);
    let account_contribution_request_total_pages = (*account_contribution_request_total)
        .max(1)
        .div_ceil(ACCOUNT_CONTRIBUTION_REQUEST_PAGE_SIZE);
    let sponsor_request_total_pages = (*sponsor_request_total)
        .max(1)
        .div_ceil(SPONSOR_REQUEST_PAGE_SIZE);

    let on_token_request_status_filter_change = {
        let token_request_status_filter = token_request_status_filter.clone();
        let token_request_page = token_request_page.clone();
        let reload_token_requests = reload_token_requests.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                let status = target.value();
                token_request_status_filter.set(status.clone());
                token_request_page.set(1);
                reload_token_requests.emit((Some(1), Some(status)));
            }
        })
    };

    let on_token_request_page_change = {
        let token_request_page = token_request_page.clone();
        let reload_token_requests = reload_token_requests.clone();
        Callback::from(move |page: usize| {
            token_request_page.set(page);
            reload_token_requests.emit((Some(page), None));
        })
    };

    let on_approve_token_request = {
        let token_request_action_inflight = token_request_action_inflight.clone();
        let token_requests = token_requests.clone();
        let reload = reload.clone();
        let reload_token_requests = reload_token_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let token_request_action_inflight = token_request_action_inflight.clone();
            let token_requests = token_requests.clone();
            let reload = reload.clone();
            let reload_token_requests = reload_token_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*token_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                token_request_action_inflight.set(inflight);

                match admin_approve_and_issue_llm_gateway_token_request(&request_id, None).await {
                    Ok(updated) => {
                        let mut list = (*token_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        token_requests.set(list);
                        load_error.set(None);
                        reload.emit(true);
                        reload_token_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*token_request_action_inflight).clone();
                inflight.remove(&request_id);
                token_request_action_inflight.set(inflight);
            });
        })
    };

    let on_reject_token_request = {
        let token_request_action_inflight = token_request_action_inflight.clone();
        let token_requests = token_requests.clone();
        let reload_token_requests = reload_token_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let token_request_action_inflight = token_request_action_inflight.clone();
            let token_requests = token_requests.clone();
            let reload_token_requests = reload_token_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*token_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                token_request_action_inflight.set(inflight);

                match admin_reject_llm_gateway_token_request(&request_id, None).await {
                    Ok(updated) => {
                        let mut list = (*token_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        token_requests.set(list);
                        load_error.set(None);
                        reload_token_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*token_request_action_inflight).clone();
                inflight.remove(&request_id);
                token_request_action_inflight.set(inflight);
            });
        })
    };

    let on_account_contribution_status_filter_change = {
        let account_contribution_request_status_filter =
            account_contribution_request_status_filter.clone();
        let account_contribution_request_page = account_contribution_request_page.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                let status = target.value();
                account_contribution_request_status_filter.set(status.clone());
                account_contribution_request_page.set(1);
                reload_account_contribution_requests.emit((Some(1), Some(status)));
            }
        })
    };

    let on_account_contribution_page_change = {
        let account_contribution_request_page = account_contribution_request_page.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        Callback::from(move |page: usize| {
            account_contribution_request_page.set(page);
            reload_account_contribution_requests.emit((Some(page), None));
        })
    };

    let on_validate_account_contribution_request = {
        let account_contribution_request_action_inflight =
            account_contribution_request_action_inflight.clone();
        let account_contribution_requests = account_contribution_requests.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let account_contribution_request_action_inflight =
                account_contribution_request_action_inflight.clone();
            let account_contribution_requests = account_contribution_requests.clone();
            let reload_account_contribution_requests = reload_account_contribution_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                account_contribution_request_action_inflight.set(inflight);

                match admin_validate_llm_gateway_account_contribution_request(&request_id, None)
                    .await
                {
                    Ok(updated) => {
                        let mut list = (*account_contribution_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        account_contribution_requests.set(list);
                        load_error.set(None);
                        reload_account_contribution_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.remove(&request_id);
                account_contribution_request_action_inflight.set(inflight);
            });
        })
    };

    let on_approve_account_contribution_request = {
        let account_contribution_request_action_inflight =
            account_contribution_request_action_inflight.clone();
        let account_contribution_requests = account_contribution_requests.clone();
        let reload = reload.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let account_contribution_request_action_inflight =
                account_contribution_request_action_inflight.clone();
            let account_contribution_requests = account_contribution_requests.clone();
            let reload = reload.clone();
            let reload_account_contribution_requests = reload_account_contribution_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                account_contribution_request_action_inflight.set(inflight);

                match admin_approve_and_issue_llm_gateway_account_contribution_request(
                    &request_id,
                    None,
                )
                .await
                {
                    Ok(updated) => {
                        let mut list = (*account_contribution_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        account_contribution_requests.set(list);
                        load_error.set(None);
                        reload.emit(true);
                        reload_account_contribution_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.remove(&request_id);
                account_contribution_request_action_inflight.set(inflight);
            });
        })
    };

    let on_reject_account_contribution_request = {
        let account_contribution_request_action_inflight =
            account_contribution_request_action_inflight.clone();
        let account_contribution_requests = account_contribution_requests.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let account_contribution_request_action_inflight =
                account_contribution_request_action_inflight.clone();
            let account_contribution_requests = account_contribution_requests.clone();
            let reload_account_contribution_requests = reload_account_contribution_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                account_contribution_request_action_inflight.set(inflight);

                match admin_reject_llm_gateway_account_contribution_request(&request_id, None).await
                {
                    Ok(updated) => {
                        let mut list = (*account_contribution_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        account_contribution_requests.set(list);
                        load_error.set(None);
                        reload_account_contribution_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.remove(&request_id);
                account_contribution_request_action_inflight.set(inflight);
            });
        })
    };

    let on_sponsor_request_status_filter_change = {
        let sponsor_request_status_filter = sponsor_request_status_filter.clone();
        let sponsor_request_page = sponsor_request_page.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                let status = target.value();
                sponsor_request_status_filter.set(status.clone());
                sponsor_request_page.set(1);
                reload_sponsor_requests.emit((Some(1), Some(status)));
            }
        })
    };

    let on_sponsor_request_page_change = {
        let sponsor_request_page = sponsor_request_page.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        Callback::from(move |page: usize| {
            sponsor_request_page.set(page);
            reload_sponsor_requests.emit((Some(page), None));
        })
    };

    let on_approve_sponsor_request = {
        let sponsor_request_action_inflight = sponsor_request_action_inflight.clone();
        let sponsor_requests = sponsor_requests.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let sponsor_request_action_inflight = sponsor_request_action_inflight.clone();
            let sponsor_requests = sponsor_requests.clone();
            let reload_sponsor_requests = reload_sponsor_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*sponsor_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                sponsor_request_action_inflight.set(inflight);

                match admin_approve_llm_gateway_sponsor_request(&request_id, None).await {
                    Ok(updated) => {
                        let mut list = (*sponsor_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        sponsor_requests.set(list);
                        load_error.set(None);
                        reload_sponsor_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*sponsor_request_action_inflight).clone();
                inflight.remove(&request_id);
                sponsor_request_action_inflight.set(inflight);
            });
        })
    };

    let on_delete_sponsor_request = {
        let sponsor_request_action_inflight = sponsor_request_action_inflight.clone();
        let sponsor_requests = sponsor_requests.clone();
        let sponsor_request_total = sponsor_request_total.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            if !confirm_destructive("确认删除这条 Sponsor 请求？") {
                return;
            }

            let sponsor_request_action_inflight = sponsor_request_action_inflight.clone();
            let sponsor_requests = sponsor_requests.clone();
            let sponsor_request_total = sponsor_request_total.clone();
            let reload_sponsor_requests = reload_sponsor_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*sponsor_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                sponsor_request_action_inflight.set(inflight);

                match delete_admin_llm_gateway_sponsor_request(&request_id).await {
                    Ok(_) => {
                        let filtered = (*sponsor_requests)
                            .iter()
                            .filter(|item| item.request_id != request_id)
                            .cloned()
                            .collect::<Vec<_>>();
                        sponsor_requests.set(filtered);
                        sponsor_request_total.set((*sponsor_request_total).saturating_sub(1));
                        load_error.set(None);
                        reload_sponsor_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*sponsor_request_action_inflight).clone();
                inflight.remove(&request_id);
                sponsor_request_action_inflight.set(inflight);
            });
        })
    };

    let on_copy = {
        let flash = flash.clone();
        Callback::from(move |(label, value): (String, String)| {
            copy_text(&value);
            flash.emit((format!("已复制{}", label), false));
        })
    };

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
    let pending_token_requests = token_requests
        .iter()
        .filter(|r| r.status == "pending")
        .count();
    let pending_contribution_requests = account_contribution_requests
        .iter()
        .filter(|r| r.status == "pending" || r.status == "failed" || r.status == "validated")
        .count();
    let pending_sponsor_requests = sponsor_requests
        .iter()
        .filter(|r| r.status == "submitted" || r.status == "payment_email_sent")
        .count();
    let total_pending =
        pending_token_requests + pending_contribution_requests + pending_sponsor_requests;

    // Build the full-screen modal for a selected usage event (request detail,
    // headers, last message, copy buttons). Rendered outside the tab flow so
    // it overlays the entire viewport.
    // Client-side filters for Keys, Account Groups, and the Usage key picker.
    // Matches are case-insensitive. `use_memo` avoids re-filtering on unrelated
    // parent re-renders. These are pre-computed at component top-level because
    // the html! macro does not permit `let` bindings inside conditional branches.
    html! {
        <main class={classes!(
            "min-h-screen",
            "bg-[var(--bg)]",
            "px-4",
            "py-8",
            "lg:px-6",
            "lg:py-10"
        )}>
            <div class={classes!("mx-auto", "max-w-6xl", "space-y-4")}>
                <section class={classes!(
                    "rounded-xl",
                    "border",
                    "border-[var(--border)]",
                    "bg-[var(--surface)]",
                    "p-5"
                )}>
                    <div class={classes!("flex", "items-start", "justify-between", "gap-4", "flex-wrap")}>
                        <h1 class={classes!("m-0", "font-mono", "text-xl", "font-bold")}>
                            { "LLM Gateway Admin" }
                        </h1>
                        <div class={classes!("flex", "gap-2", "flex-wrap")}>
                            <Link<Route> to={Route::Admin} classes={classes!("btn-terminal")}>{ "Admin 首页" }</Link<Route>>
                            <Link<Route> to={Route::AdminLlmGatewayMonitor} classes={classes!("btn-terminal")}>{ "监控页" }</Link<Route>>
                            <Link<Route> to={Route::AdminLlmGatewayModeration} classes={classes!("btn-terminal")}>{ "关键词审核" }</Link<Route>>
                            <Link<Route> to={Route::LlmAccess} classes={classes!("btn-terminal", "btn-terminal-primary")}>{ "公共页" }</Link<Route>>
                        </div>
                    </div>

                    if let Some(err) = (*load_error).clone() {
                        <div class={classes!("mt-4", "rounded-lg", "border", "border-red-400/35", "bg-red-500/8", "px-4", "py-3", "text-sm", "text-red-700", "dark:text-red-200")}>
                            { err }
                        </div>
                    }
                </section>

                // ── Tab Bar (always visible) ──
                { render_tab_bar(&active_tab, &[
                    (TAB_OVERVIEW, "Overview"),
                    (TAB_KEYS, "Keys"),
                    (TAB_GROUPS, "Groups"),
                    (TAB_ACCOUNTS, "Accounts"),
                    (TAB_USAGE, "Usage"),
                    (TAB_JOURNAL, "Journal"),
                    (TAB_REQUESTS, "Requests"),
                    (TAB_SETTINGS, "Settings"),
                ], &on_tab_click, Some((TAB_REQUESTS, total_pending))) }

                // ── Overview Tab ──
                if active_tab == TAB_OVERVIEW {
                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Dashboard" }</h2>
                        <button
                            class={classes!("btn-terminal")}
                            title="刷新 Dashboard"
                            aria-label="刷新 Dashboard"
                            onclick={{
                                let reload = reload.clone();
                                Callback::from(move |_| reload.emit(true))
                            }}
                            disabled={*loading}
                        >
                            <i class={classes!("fas", if *loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                        </button>
                    </div>
                    <div class={classes!("mt-4", "grid", "gap-3", "grid-cols-2", "xl:grid-cols-4")}>
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
                } // end TAB_OVERVIEW

                // ── Journal Tab ──





                // ── Requests Tab ──
                if active_tab == TAB_REQUESTS {
                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Token Wishes" }</h2>
                            <p class={classes!("mt-1", "m-0", "text-xs", "text-[var(--muted)]")}>
                                { "只有在这里审核通过后，系统才会真正创建 key 并通过邮件发给申请人。" }
                            </p>
                        </div>
                        <button
                            class={classes!("btn-terminal")}
                            onclick={{
                                let reload_token_requests = reload_token_requests.clone();
                                Callback::from(move |_| reload_token_requests.emit((None, None)))
                            }}
                            disabled={*token_request_loading}
                        >
                            <i class={classes!("fas", if *token_request_loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                        </button>
                    </div>

                    <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-[minmax(0,16rem)_auto]")}>
                        <label class={classes!("text-sm")}>
                            <span class={classes!("text-[var(--muted)]")}>{ "状态" }</span>
                            <select
                                key={format!("token-request-filter-{}", (*token_request_status_filter).clone())}
                                class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                onchange={on_token_request_status_filter_change}
                            >
                                <option value="" selected={(*token_request_status_filter).is_empty()}>{ "全部" }</option>
                                <option value="pending" selected={*token_request_status_filter == "pending"}>{ "pending" }</option>
                                <option value="failed" selected={*token_request_status_filter == "failed"}>{ "failed" }</option>
                                <option value="issued" selected={*token_request_status_filter == "issued"}>{ "issued" }</option>
                                <option value="rejected" selected={*token_request_status_filter == "rejected"}>{ "rejected" }</option>
                            </select>
                        </label>
                    </div>

                    if token_requests.is_empty() && !*token_request_loading {
                        <div class={classes!("mt-4")}>
                            <EmptyState icon="fa-inbox" title="当前筛选下还没有 token 许愿。" />
                        </div>
                    } else {
                        <div class={classes!("mt-4", "space-y-3")}>
                            { for token_requests.iter().map(|item| {
                                let request_id = item.request_id.clone();
                                let approve_request_id = item.request_id.clone();
                                let reject_request_id = item.request_id.clone();
                                let approve_cb = on_approve_token_request.clone();
                                let reject_cb = on_reject_token_request.clone();
                                let action_busy = token_request_action_inflight.contains(&request_id);
                                html! {
                                    <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
                                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("min-w-0", "space-y-1")}>
                                                <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                                    <StatusBadge status={item.status.clone()} />
                                                    <span class={classes!("font-semibold")}>{ item.requester_email.clone() }</span>
                                                    <span class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>{ item.request_id.clone() }</span>
                                                </div>
                                                <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                    { format!("{} / {} · created {}", item.client_ip, item.ip_region, format_ms(item.created_at)) }
                                                </div>
                                            </div>
                                            <div class={classes!("text-right")}>
                                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "申请 token" }</div>
                                                <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format_number_u64(item.requested_quota_billable_limit) }</div>
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]")}>
                                            <div>
                                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "缘由" }</div>
                                                <div class={classes!("mt-2", "whitespace-pre-wrap", "break-words", "text-sm", "leading-6", "text-[var(--text)]")}>
                                                    { item.request_reason.clone() }
                                                </div>
                                            </div>
                                            <div class={classes!("space-y-2", "text-sm")}>
                                                if let Some(frontend_page_url) = item.frontend_page_url.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "页面" }</div>
                                                        <div class={classes!("mt-1", "break-all", "text-[var(--text)]")}>{ frontend_page_url }</div>
                                                    </div>
                                                }
                                                if let Some(issued_key_name) = item.issued_key_name.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "已发放 Key" }</div>
                                                        <div class={classes!("mt-1", "text-[var(--text)]")}>
                                                            { format!("{} ({})", issued_key_name, item.issued_key_id.clone().unwrap_or_default()) }
                                                        </div>
                                                    </div>
                                                }
                                                if let Some(admin_note) = item.admin_note.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Admin Note" }</div>
                                                        <div class={classes!("mt-1", "whitespace-pre-wrap", "break-words", "text-[var(--text)]")}>{ admin_note }</div>
                                                    </div>
                                                }
                                                if let Some(failure_reason) = item.failure_reason.clone() {
                                                    <div class={classes!("rounded-lg", "border", "border-red-400/25", "bg-red-500/8", "px-3", "py-2", "text-red-700", "dark:text-red-200")}>
                                                        { failure_reason }
                                                    </div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                { item.processed_at.map(format_ms).map(|value| format!("processed {}", value)).unwrap_or_else(|| "尚未处理".to_string()) }
                                            </div>
                                            <div class={classes!("flex", "items-center", "gap-2")}>
                                                if item.status == "pending" || item.status == "failed" {
                                                    <button
                                                        class={classes!("btn-terminal", "btn-terminal-primary")}
                                                        onclick={Callback::from(move |_| approve_cb.emit(approve_request_id.clone()))}
                                                        disabled={action_busy}
                                                    >
                                                        { if action_busy { "处理中..." } else { "批准并发放" } }
                                                    </button>
                                                }
                                                if item.status == "pending" || item.status == "failed" {
                                                    <button
                                                        class={classes!("btn-terminal", "btn-terminal-danger")}
                                                        onclick={Callback::from(move |_| reject_cb.emit(reject_request_id.clone()))}
                                                        disabled={action_busy}
                                                    >
                                                        { "拒绝" }
                                                    </button>
                                                }
                                            </div>
                                        </div>
                                    </article>
                                }
                            }) }
                        </div>
                    }

                    <div class={classes!("mt-5")}>
                        <Pagination current_page={*token_request_page} total_pages={token_request_total_pages} on_page_change={on_token_request_page_change} />
                    </div>
                </section>

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Account Contributions" }</h2>
                                <p class={classes!("mt-1", "m-0", "text-xs", "text-[var(--muted)]")}>
                                    { "公开页提交的 Codex 账号贡献申请会先进入这里；先验证 auth refresh，validated 后才能入库并发放绑定该账号路由的 token。" }
                                </p>
                        </div>
                        <button
                            class={classes!("btn-terminal")}
                            onclick={{
                                let reload_account_contribution_requests = reload_account_contribution_requests.clone();
                                Callback::from(move |_| reload_account_contribution_requests.emit((None, None)))
                            }}
                            disabled={*account_contribution_request_loading}
                        >
                            <i class={classes!("fas", if *account_contribution_request_loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                        </button>
                    </div>

                    <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-[minmax(0,16rem)_auto]")}>
                        <label class={classes!("text-sm")}>
                            <span class={classes!("text-[var(--muted)]")}>{ "状态" }</span>
                            <select
                                key={format!("account-contribution-filter-{}", (*account_contribution_request_status_filter).clone())}
                                class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                onchange={on_account_contribution_status_filter_change}
                            >
                                <option value="" selected={(*account_contribution_request_status_filter).is_empty()}>{ "全部" }</option>
                                    <option value="pending" selected={*account_contribution_request_status_filter == "pending"}>{ "pending" }</option>
                                    <option value="validated" selected={*account_contribution_request_status_filter == "validated"}>{ "validated" }</option>
                                    <option value="failed" selected={*account_contribution_request_status_filter == "failed"}>{ "failed" }</option>
                                <option value="issued" selected={*account_contribution_request_status_filter == "issued"}>{ "issued" }</option>
                                <option value="rejected" selected={*account_contribution_request_status_filter == "rejected"}>{ "rejected" }</option>
                            </select>
                        </label>
                    </div>

                    if account_contribution_requests.is_empty() && !*account_contribution_request_loading {
                        <div class={classes!("mt-4")}>
                            <EmptyState icon="fa-inbox" title="当前筛选下还没有账号贡献申请。" />
                        </div>
                    } else {
                        <div class={classes!("mt-4", "space-y-3")}>
                            { for account_contribution_requests.iter().map(|item| {
                                    let request_id = item.request_id.clone();
                                    let validate_request_id = item.request_id.clone();
                                    let approve_request_id = item.request_id.clone();
                                    let reject_request_id = item.request_id.clone();
                                    let validate_cb = on_validate_account_contribution_request.clone();
                                    let approve_cb = on_approve_account_contribution_request.clone();
                                let reject_cb = on_reject_account_contribution_request.clone();
                                let on_copy = on_copy.clone();
                                let action_busy =
                                    account_contribution_request_action_inflight.contains(&request_id);
                                html! {
                                    <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
                                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("min-w-0", "space-y-1")}>
                                                <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                                    <StatusBadge status={item.status.clone()} />
                                                    <span class={classes!("font-semibold")}>{ item.account_name.clone() }</span>
                                                        if !item.requester_email.trim().is_empty() {
                                                            <span class={classes!("text-xs", "text-[var(--muted)]")}>{ item.requester_email.clone() }</span>
                                                        }
                                                    <span class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>{ item.request_id.clone() }</span>
                                                </div>
                                                <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                    { format!("{} / {} · created {}", item.client_ip, item.ip_region, format_ms(item.created_at)) }
                                                </div>
                                            </div>
                                            <div class={classes!("text-right", "space-y-1")}>
                                                if let Some(github_id) = item.github_id.clone() {
                                                    <div class={classes!("text-sm", "font-semibold")}>{ format!("@{}", github_id) }</div>
                                                }
                                                if let Some(account_id) = item.account_id.clone() {
                                                    <div class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>{ account_id }</div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]")}>
                                            <div>
                                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "留言" }</div>
                                                <div class={classes!("mt-2", "whitespace-pre-wrap", "break-words", "text-sm", "leading-6", "text-[var(--text)]")}>
                                                    { item.contributor_message.clone() }
                                                </div>
                                            </div>
                                            <div class={classes!("space-y-2", "text-sm")}>
                                                if let Some(frontend_page_url) = item.frontend_page_url.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "页面" }</div>
                                                        <div class={classes!("mt-1", "break-all", "text-[var(--text)]")}>{ frontend_page_url }</div>
                                                    </div>
                                                }
                                                if let Some(imported_account_name) = item.imported_account_name.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "已导入账号" }</div>
                                                        <div class={classes!("mt-1", "text-[var(--text)]")}>{ imported_account_name }</div>
                                                    </div>
                                                }
                                                if let Some(issued_key_name) = item.issued_key_name.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "已发放 Key" }</div>
                                                        <div class={classes!("mt-1", "text-[var(--text)]")}>
                                                            { format!("{} ({})", issued_key_name, item.issued_key_id.clone().unwrap_or_default()) }
                                                        </div>
                                                    </div>
                                                }
                                                if let Some(admin_note) = item.admin_note.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Admin Note" }</div>
                                                        <div class={classes!("mt-1", "whitespace-pre-wrap", "break-words", "text-[var(--text)]")}>{ admin_note }</div>
                                                    </div>
                                                }
                                                if let Some(failure_reason) = item.failure_reason.clone() {
                                                    <div class={classes!("rounded-lg", "border", "border-red-400/25", "bg-red-500/8", "px-3", "py-2", "text-red-700", "dark:text-red-200")}>
                                                        { failure_reason }
                                                    </div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-3")}>
                                            { copyable_token_preview("access_token", &item.access_token, &on_copy) }
                                            { copyable_token_preview("id_token", &item.id_token, &on_copy) }
                                            { copyable_token_preview("refresh_token", &item.refresh_token, &on_copy) }
                                        </div>

                                        <div class={classes!("mt-4", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                { item.processed_at.map(format_ms).map(|value| format!("processed {}", value)).unwrap_or_else(|| "尚未处理".to_string()) }
                                            </div>
                                            <div class={classes!("flex", "items-center", "gap-2")}>
                                                    if item.status == "pending" || item.status == "failed" {
                                                        <button
                                                            class={classes!("btn-terminal", "btn-terminal-primary")}
                                                            onclick={Callback::from(move |_| validate_cb.emit(validate_request_id.clone()))}
                                                            disabled={action_busy}
                                                        >
                                                            { if action_busy { "验证中..." } else { "验证" } }
                                                        </button>
                                                    }
                                                    if item.status == "validated" {
                                                        <button
                                                            class={classes!("btn-terminal", "btn-terminal-primary")}
                                                            onclick={Callback::from(move |_| approve_cb.emit(approve_request_id.clone()))}
                                                            disabled={action_busy}
                                                        >
                                                            { if action_busy { "入库中..." } else { "入库并发放" } }
                                                        </button>
                                                    }
                                                if item.status == "pending" || item.status == "failed" {
                                                    <button
                                                        class={classes!("btn-terminal", "btn-terminal-danger")}
                                                        onclick={Callback::from(move |_| reject_cb.emit(reject_request_id.clone()))}
                                                        disabled={action_busy}
                                                    >
                                                        { "拒绝" }
                                                    </button>
                                                }
                                            </div>
                                        </div>
                                    </article>
                                }
                            }) }
                        </div>
                    }

                    <div class={classes!("mt-5")}>
                        <Pagination
                            current_page={*account_contribution_request_page}
                            total_pages={account_contribution_request_total_pages}
                            on_page_change={on_account_contribution_page_change}
                        />
                    </div>
                </section>

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Sponsors" }</h2>
                            <p class={classes!("mt-1", "m-0", "text-xs", "text-[var(--muted)]")}>
                                { "这批请求是「先填邮箱，再发付款说明邮件」的人工确认流。你确认对方已经按邮件说明完成赞助后，再在这里标记通过。" }
                            </p>
                        </div>
                        <button
                            class={classes!("btn-terminal")}
                            onclick={{
                                let reload_sponsor_requests = reload_sponsor_requests.clone();
                                Callback::from(move |_| reload_sponsor_requests.emit((None, None)))
                            }}
                            disabled={*sponsor_request_loading}
                        >
                            <i class={classes!("fas", if *sponsor_request_loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                        </button>
                    </div>

                    <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-[minmax(0,16rem)_auto]")}>
                        <label class={classes!("text-sm")}>
                            <span class={classes!("text-[var(--muted)]")}>{ "状态" }</span>
                            <select
                                key={format!("sponsor-filter-{}", (*sponsor_request_status_filter).clone())}
                                class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                onchange={on_sponsor_request_status_filter_change}
                            >
                                <option value="" selected={(*sponsor_request_status_filter).is_empty()}>{ "全部" }</option>
                                <option value="submitted" selected={*sponsor_request_status_filter == "submitted"}>{ "submitted" }</option>
                                <option value="payment_email_sent" selected={*sponsor_request_status_filter == "payment_email_sent"}>{ "payment_email_sent" }</option>
                                <option value="approved" selected={*sponsor_request_status_filter == "approved"}>{ "approved" }</option>
                            </select>
                        </label>
                    </div>

                    if sponsor_requests.is_empty() && !*sponsor_request_loading {
                        <div class={classes!("mt-4")}>
                            <EmptyState icon="fa-inbox" title="当前筛选下还没有 Sponsor 请求。" />
                        </div>
                    } else {
                        <div class={classes!("mt-4", "space-y-3")}>
                            { for sponsor_requests.iter().map(|item| {
                                let request_id = item.request_id.clone();
                                let approve_request_id = item.request_id.clone();
                                let delete_request_id = item.request_id.clone();
                                let approve_cb = on_approve_sponsor_request.clone();
                                let delete_cb = on_delete_sponsor_request.clone();
                                let action_busy = sponsor_request_action_inflight.contains(&request_id);
                                html! {
                                    <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
                                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("min-w-0", "space-y-1")}>
                                                <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                                    <StatusBadge status={item.status.clone()} />
                                                    <span class={classes!("font-semibold")}>{ item.requester_email.clone() }</span>
                                                    <span class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>{ item.request_id.clone() }</span>
                                                </div>
                                                <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                    { format!("{} / {} · created {}", item.client_ip, item.ip_region, format_ms(item.created_at)) }
                                                </div>
                                            </div>
                                            <div class={classes!("text-right", "space-y-1")}>
                                                if let Some(display_name) = item.display_name.clone() {
                                                    <div class={classes!("text-sm", "font-semibold")}>{ display_name }</div>
                                                }
                                                if let Some(github_id) = item.github_id.clone() {
                                                    <div class={classes!("text-xs", "font-semibold", "text-[var(--muted)]")}>{ format!("@{}", github_id) }</div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]")}>
                                            <div>
                                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "留言" }</div>
                                                <div class={classes!("mt-2", "whitespace-pre-wrap", "break-words", "text-sm", "leading-6", "text-[var(--text)]")}>
                                                    { item.sponsor_message.clone() }
                                                </div>
                                            </div>
                                            <div class={classes!("space-y-2", "text-sm")}>
                                                if let Some(frontend_page_url) = item.frontend_page_url.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "页面" }</div>
                                                        <div class={classes!("mt-1", "break-all", "text-[var(--text)]")}>{ frontend_page_url }</div>
                                                    </div>
                                                }
                                                if let Some(payment_email_sent_at) = item.payment_email_sent_at {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "付款说明邮件" }</div>
                                                        <div class={classes!("mt-1", "text-[var(--text)]")}>{ format_ms(payment_email_sent_at) }</div>
                                                    </div>
                                                }
                                                if let Some(admin_note) = item.admin_note.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Admin Note" }</div>
                                                        <div class={classes!("mt-1", "whitespace-pre-wrap", "break-words", "text-[var(--text)]")}>{ admin_note }</div>
                                                    </div>
                                                }
                                                if let Some(failure_reason) = item.failure_reason.clone() {
                                                    <div class={classes!("rounded-lg", "border", "border-red-400/25", "bg-red-500/8", "px-3", "py-2", "text-red-700", "dark:text-red-200")}>
                                                        { failure_reason }
                                                    </div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                { item.processed_at.map(format_ms).map(|value| format!("processed {}", value)).unwrap_or_else(|| "尚未确认".to_string()) }
                                            </div>
                                            <div class={classes!("flex", "items-center", "gap-2")}>
                                                if item.status != "approved" {
                                                    <button
                                                        class={classes!("btn-terminal", "btn-terminal-primary")}
                                                        onclick={Callback::from(move |_| approve_cb.emit(approve_request_id.clone()))}
                                                        disabled={action_busy}
                                                    >
                                                        { if action_busy { "处理中..." } else { "标记已确认" } }
                                                    </button>
                                                }
                                                <button
                                                    class={classes!("btn-terminal", "btn-terminal-danger")}
                                                    onclick={Callback::from(move |_| delete_cb.emit(delete_request_id.clone()))}
                                                    disabled={action_busy}
                                                >
                                                    { "删除" }
                                                </button>
                                            </div>
                                        </div>
                                    </article>
                                }
                            }) }
                        </div>
                    }

                    <div class={classes!("mt-5")}>
                        <Pagination
                            current_page={*sponsor_request_page}
                            total_pages={sponsor_request_total_pages}
                            on_page_change={on_sponsor_request_page_change}
                        />
                    </div>
                </section>
                } // end TAB_REQUESTS

            </div>


            if let Some((message, is_error)) = (*toast).clone() {
                <div class={classes!(
                    "fixed", "bottom-5", "right-5", "z-[90]",
                    "max-w-[min(34rem,calc(100vw-2.5rem))]",
                    "rounded-xl", "border", "px-4", "py-3",
                    "text-sm", "font-semibold", "leading-5", "whitespace-pre-wrap",
                    "shadow-[0_8px_24px_rgba(0,0,0,0.15)]",
                    if is_error {
                        classes!("border-red-400/35", "bg-red-500/92", "text-white")
                    } else {
                        classes!("border-emerald-400/35", "bg-emerald-500/92", "text-white")
                    }
                )}>
                    { message }
                </div>
            }
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
