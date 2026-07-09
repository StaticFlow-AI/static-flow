use std::collections::{BTreeMap, HashSet};

use gloo_timers::callback::{Interval, Timeout};
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
        consume_admin_llm_gateway_account_rate_limit_reset_credit,
        create_admin_llm_gateway_account_group, create_admin_llm_gateway_account_import_job,
        delete_admin_llm_gateway_account, delete_admin_llm_gateway_account_group,
        delete_admin_llm_gateway_key, delete_admin_llm_gateway_proxy_config,
        delete_admin_llm_gateway_sponsor_request,
        fetch_admin_llm_gateway_account_contribution_requests,
        fetch_admin_llm_gateway_account_groups_page, fetch_admin_llm_gateway_account_import_job,
        fetch_admin_llm_gateway_account_import_jobs, fetch_admin_llm_gateway_accounts,
        fetch_admin_llm_gateway_accounts_page, fetch_admin_llm_gateway_accounts_page_with_query,
        fetch_admin_llm_gateway_keys_page, fetch_admin_llm_gateway_proxy_configs,
        fetch_admin_llm_gateway_sponsor_requests, fetch_admin_llm_gateway_token_requests,
        fetch_llm_gateway_status, import_admin_llm_gateway_account,
        patch_admin_llm_gateway_account, patch_admin_llm_gateway_account_group,
        patch_admin_llm_gateway_key, patch_admin_llm_gateway_proxy_config,
        probe_admin_llm_gateway_account_models, refresh_admin_llm_gateway_account_auth,
        refresh_admin_llm_gateway_account_usage, refresh_admin_llm_gateway_proxy_traffic,
        reset_admin_llm_gateway_proxy_config_override, AccountSummaryView,
        AdminAccountGroupOptionView, AdminAccountGroupView, AdminAccountsSummaryView,
        AdminLlmGatewayAccountContributionRequestView,
        AdminLlmGatewayAccountContributionRequestsQuery, AdminLlmGatewayAccountPageQuery,
        AdminLlmGatewayKeyView, AdminLlmGatewayKeysSummaryView, AdminLlmGatewaySponsorRequestView,
        AdminLlmGatewaySponsorRequestsQuery, AdminLlmGatewayTokenRequestView,
        AdminLlmGatewayTokenRequestsQuery, AdminProxyTrafficSnapshotView,
        AdminUpstreamProxyCheckResponse, AdminUpstreamProxyCheckTargetView,
        AdminUpstreamProxyConfigView, AdminUpstreamProxyEndpointCheckView,
        CodexAccountImportJobDetailView, CodexAccountImportJobSummaryView,
        CreateAdminAccountGroupInput, LlmGatewayRateLimitBucketView,
        LlmGatewayRateLimitStatusResponse, LlmGatewayRateLimitWindowView,
        PatchAdminAccountGroupInput, PatchAdminLlmGatewayAccountInput,
        PatchAdminLlmGatewayKeyRequest, PatchAdminUpstreamProxyConfigInput,
    },
    components::{
        empty_state::EmptyState, pagination::Pagination, search_box::SearchBox,
        status_badge::StatusBadge, tab_bar::render_tab_bar,
    },
    pages::llm_access_shared::{
        confirm_destructive, format_latency_ms, format_ms, format_number_i64, format_number_u64,
        format_optional_bytes_human, format_percent, format_reset_hint, MaskedSecretCode,
    },
    router::Route,
};

pub(crate) const USAGE_PAGE_SIZE: usize = 20;
const DEFAULT_ADMIN_GROUP_PAGE_SIZE: usize = 24;
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
const ADMIN_CODEX_IMPORT_JOB_LIST_LIMIT: usize = 10;
const ACCOUNT_PAGE_SIZE: usize = 8;
/// Page size for the Usage tab's server-side key filter search.
pub(crate) const USAGE_KEY_OPTION_LIMIT: usize = 20;
const CODEX_IMAGE_DEFAULT_CONCURRENCY: u64 = 3;
const CODEX_IMAGE_MAX_CONCURRENCY: u64 = 1024;
const ACCOUNT_ACCENT_BORDERS: &[&str] = &[
    "border-l-4 border-l-teal-500/70",
    "border-l-4 border-l-violet-500/70",
    "border-l-4 border-l-amber-500/70",
    "border-l-4 border-l-sky-500/70",
    "border-l-4 border-l-rose-500/70",
];

const TAB_OVERVIEW: &str = "overview";
const TAB_KEYS: &str = "keys";
const TAB_GROUPS: &str = "groups";
const TAB_ACCOUNTS: &str = "accounts";
const TAB_USAGE: &str = "usage";
const TAB_JOURNAL: &str = "journal";
const TAB_REQUESTS: &str = "requests";
const TAB_SETTINGS: &str = "settings";

fn should_load_llm_gateway_import_jobs(active_tab: &str) -> bool {
    active_tab == TAB_ACCOUNTS
}

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

struct ParsedAdminCodexAuthJson {
    id_token: String,
    access_token: String,
    refresh_token: String,
    account_id: Option<String>,
}

fn parse_admin_codex_auth_json(raw: &str) -> Result<ParsedAdminCodexAuthJson, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| "auth.json 不是合法 JSON".to_string())?;
    if !value.is_object() {
        return Err("auth.json 必须是 JSON object".to_string());
    }
    let id_token = optional_auth_json_string(&value, &["id_token", "idToken"]).unwrap_or_default();
    let access_token =
        optional_auth_json_string(&value, &["access_token", "accessToken"]).unwrap_or_default();
    let refresh_token =
        optional_auth_json_string(&value, &["refresh_token", "refreshToken"]).unwrap_or_default();
    if id_token.is_empty() && access_token.is_empty() && refresh_token.is_empty() {
        return Err("auth.json 没有识别到可用 token 字段".to_string());
    }
    Ok(ParsedAdminCodexAuthJson {
        id_token,
        access_token,
        refresh_token,
        account_id: optional_auth_json_string(&value, &["account_id", "accountId"]),
    })
}

fn optional_auth_json_string(value: &serde_json::Value, fields: &[&str]) -> Option<String> {
    fields
        .iter()
        .find_map(|field| value.get(*field).and_then(serde_json::Value::as_str))
        .or_else(|| {
            value.get("tokens").and_then(|tokens| {
                fields
                    .iter()
                    .find_map(|field| tokens.get(*field).and_then(serde_json::Value::as_str))
            })
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn parse_admin_codex_batch_import_json(raw: &str) -> Result<Vec<serde_json::Value>, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| "批量导入内容不是合法 JSON".to_string())?;
    let items = value
        .as_array()
        .ok_or_else(|| "批量导入内容必须是 JSON array".to_string())?;
    if items.is_empty() {
        return Err("批量导入内容不能为空".to_string());
    }
    let mut normalized = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let Some(mut object) = item.as_object().cloned() else {
            return Err(format!("第 {} 项必须是 JSON object", index + 1));
        };
        let name = object
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("第 {} 项缺少有效的 name", index + 1))?;
        let auth_json = object.get("auth_json");
        let tokens = object.get("tokens");
        if auth_json.is_none() && tokens.is_none() {
            return Err(format!("第 {} 项缺少 auth_json 或 tokens", index + 1));
        }
        if let Some(value) = auth_json {
            if !value.is_object() {
                return Err(format!("第 {} 项的 auth_json 必须是 JSON object", index + 1));
            }
        }
        if let Some(value) = tokens {
            if !value.is_object() {
                return Err(format!("第 {} 项的 tokens 必须是 JSON object", index + 1));
            }
        }
        object.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        normalized.push(serde_json::Value::Object(object));
    }
    Ok(normalized)
}

fn codex_import_status_tone(status: &str) -> &'static str {
    match status {
        "completed" | "imported" => "text-emerald-600 dark:text-emerald-300",
        "failed" | "conflict" => "text-red-600 dark:text-red-300",
        "running" | "queued" => "text-amber-600 dark:text-amber-300",
        "skipped" => "text-[var(--muted)]",
        _ => "text-[var(--muted)]",
    }
}

fn codex_import_job_is_terminal(status: &str) -> bool {
    matches!(status, "completed" | "failed")
}

fn upsert_codex_import_job_summary(
    jobs: &[CodexAccountImportJobSummaryView],
    summary: CodexAccountImportJobSummaryView,
) -> Vec<CodexAccountImportJobSummaryView> {
    let mut next = jobs
        .iter()
        .filter(|job| job.job_id != summary.job_id)
        .cloned()
        .collect::<Vec<_>>();
    next.insert(0, summary);
    next.truncate(ADMIN_CODEX_IMPORT_JOB_LIST_LIMIT);
    next
}

fn account_proxy_select_value(account: &AccountSummaryView) -> String {
    match account.proxy_mode.as_str() {
        "direct" => "direct".to_string(),
        "fixed" => account
            .proxy_config_id
            .as_deref()
            .map(|id| format!("fixed:{id}"))
            .unwrap_or_else(|| "inherit".to_string()),
        _ => "inherit".to_string(),
    }
}

fn account_configured_proxy_label(account: &AccountSummaryView) -> String {
    match account.proxy_mode.as_str() {
        "direct" => "configured: direct".to_string(),
        "fixed" => account
            .effective_proxy_config_name
            .as_deref()
            .map(|name| format!("configured: fixed ({name})"))
            .or_else(|| {
                account
                    .proxy_config_id
                    .as_deref()
                    .map(|id| format!("configured: fixed ({id})"))
            })
            .unwrap_or_else(|| "configured: fixed".to_string()),
        _ => "configured: inherit provider".to_string(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum AccountSortMode {
    None,
    PrimaryAsc,
    PrimaryDesc,
    SecondaryAsc,
    SecondaryDesc,
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


fn format_future_duration_ms(remaining_ms: i64) -> String {
    if remaining_ms >= 24 * 3_600_000 {
        format!("{:.1} d", remaining_ms as f64 / (24.0 * 3_600_000.0))
    } else if remaining_ms >= 3_600_000 {
        format!("{:.1} h", remaining_ms as f64 / 3_600_000.0)
    } else if remaining_ms >= 60_000 {
        format!("{:.1} min", remaining_ms as f64 / 60_000.0)
    } else if remaining_ms >= 1_000 {
        format!("{:.1} s", remaining_ms as f64 / 1_000.0)
    } else {
        format!("{remaining_ms} ms")
    }
}

fn format_access_token_expiry(now_ms: i64, expires_at_ms: Option<i64>) -> String {
    let Some(expires_at_ms) = expires_at_ms else {
        return "access token expiry -".to_string();
    };
    let absolute = format_ms(expires_at_ms);
    let remaining_ms = expires_at_ms.saturating_sub(now_ms);
    if remaining_ms > 0 {
        format!(
            "access token expires {} · ~{} left",
            absolute,
            format_future_duration_ms(remaining_ms)
        )
    } else {
        format!(
            "access token expired {} ago · {}",
            format_optional_duration_ms(Some(remaining_ms.saturating_abs())),
            absolute
        )
    }
}

fn account_rate_limit_bucket<'a>(
    status: Option<&'a LlmGatewayRateLimitStatusResponse>,
    account_name: &str,
) -> Option<&'a LlmGatewayRateLimitBucketView> {
    let status = status?;
    status
        .buckets
        .iter()
        .find(|bucket| {
            bucket.account_name.as_deref() == Some(account_name)
                && bucket.is_primary
                && bucket.limit_id == "codex"
        })
        .or_else(|| {
            status.buckets.iter().find(|bucket| {
                bucket.account_name.as_deref() == Some(account_name) && bucket.is_primary
            })
        })
        .or_else(|| {
            status
                .buckets
                .iter()
                .find(|bucket| bucket.account_name.as_deref() == Some(account_name))
        })
}

fn account_image_rate_limit_bucket<'a>(
    status: Option<&'a LlmGatewayRateLimitStatusResponse>,
    account_name: &str,
) -> Option<&'a LlmGatewayRateLimitBucketView> {
    let status = status?;
    status.buckets.iter().find(|bucket| {
        if bucket.account_name.as_deref() != Some(account_name) {
            return false;
        }
        let limit_id = bucket.limit_id.to_ascii_lowercase();
        let limit_name = bucket
            .limit_name
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let display_name = bucket.display_name.to_ascii_lowercase();
        [limit_id.as_str(), limit_name.as_str(), display_name.as_str()]
            .iter()
            .any(|value| value.contains("image"))
    })
}

fn account_limit_remaining_percent(
    window: Option<&LlmGatewayRateLimitWindowView>,
    fallback: Option<f64>,
) -> Option<f64> {
    window.map(|window| window.remaining_percent).or(fallback)
}

fn account_limit_width(
    window: Option<&LlmGatewayRateLimitWindowView>,
    fallback: Option<f64>,
) -> f64 {
    account_limit_remaining_percent(window, fallback)
        .unwrap_or(100.0)
        .clamp(0.0, 100.0)
}

fn account_limit_percent_label(
    window: Option<&LlmGatewayRateLimitWindowView>,
    fallback: Option<f64>,
) -> String {
    account_limit_remaining_percent(window, fallback)
        .map(format_percent)
        .unwrap_or_else(|| "-".to_string())
}

fn account_limit_used_label(window: Option<&LlmGatewayRateLimitWindowView>) -> String {
    window
        .map(|window| format!("已用 {}", format_percent(window.used_percent)))
        .unwrap_or_else(|| "已用 -".to_string())
}

fn account_limit_reset_label(window: Option<&LlmGatewayRateLimitWindowView>) -> String {
    window
        .map(|window| format_reset_hint(window.resets_at))
        .unwrap_or_else(|| "重置时间未知".to_string())
}

fn render_account_limit_tile(
    label: &str,
    percent_label: &str,
    used_label: &str,
    reset_label: &str,
    width: f64,
    accent: &'static str,
) -> Html {
    html! {
        <div class={classes!(
            "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]",
            "px-3", "py-2.5", "min-w-0",
        )}>
            <div class={classes!("flex", "items-start", "justify-between", "gap-2")}>
                <span class={classes!("font-mono", "text-[11px]", "font-semibold", "text-[var(--muted)]", "uppercase", "tracking-wider")}>
                    { label }
                </span>
                <span class={classes!("font-mono", "text-base", "font-black", "leading-none", "text-[var(--text)]")}>
                    { percent_label }
                </span>
            </div>
            <div class={classes!("mt-2", "h-2", "overflow-hidden", "rounded-full", "bg-[var(--surface)]")}>
                <div
                    class={classes!("h-full", "rounded-full", "transition-[width]", "duration-500", accent)}
                    style={format!("width: {width:.1}%;")}
                />
            </div>
            <div class={classes!("mt-2", "grid", "gap-1", "font-mono", "text-[10px]", "leading-tight", "text-[var(--muted)]")}>
                <span>{ used_label }</span>
                <span class={classes!("text-[var(--text)]")}>{ reset_label }</span>
            </div>
        </div>
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

fn is_gpt_pro_account(plan_type: Option<&str>) -> bool {
    plan_type.map(str::trim).is_some_and(|plan| {
        let normalized = plan.to_ascii_lowercase();
        normalized == "pro" || normalized == "gpt pro"
    })
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
struct AccountGroupEditorCardProps {
    group_item: AdminAccountGroupView,
    accounts: Vec<AccountSummaryView>,
    on_changed: Callback<()>,
    on_flash: Callback<(String, bool)>,
}

#[function_component(AccountGroupEditorCard)]
fn account_group_editor_card(props: &AccountGroupEditorCardProps) -> Html {
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
    let account_groups_page_items = use_state(Vec::<AdminAccountGroupView>::new);
    let account_groups_total = use_state(|| 0_usize);
    let account_groups_page = use_state(|| 1_usize);
    let account_groups_page_limit = use_state(|| DEFAULT_ADMIN_GROUP_PAGE_SIZE);
    let account_groups_search = use_state(String::new);
    let account_group_candidate_accounts = use_state(Vec::<AccountSummaryView>::new);
    let account_group_candidate_loading = use_state(|| false);
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
    let proxy_configs = use_state(Vec::<AdminUpstreamProxyConfigView>::new);
    let create_account_group_name = use_state(String::new);
    let create_account_group_account_names = use_state(Vec::<String>::new);
    let creating_account_group = use_state(|| false);
    let account_group_form_expanded = use_state(|| false);
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
    let accounts = use_state(Vec::<AccountSummaryView>::new);
    let accounts_summary = use_state(AdminAccountsSummaryView::default);
    let codex_rate_limit_status = use_state(|| None::<LlmGatewayRateLimitStatusResponse>);
    let import_name = use_state(String::new);
    let import_id_token = use_state(String::new);
    let import_access_token = use_state(String::new);
    let import_refresh_token = use_state(String::new);
    let import_account_id = use_state(String::new);
    let import_raw_auth_json = use_state(String::new);
    let import_raw_auth_feedback = use_state(|| None::<(String, bool)>);
    let importing = use_state(|| false);
    let show_batch_import_form = use_state(|| false);
    let batch_import_raw_json = use_state(String::new);
    let batch_import_feedback = use_state(|| None::<(String, bool)>);
    let batch_import_validate_before_import = use_state(|| true);
    let batch_importing = use_state(|| false);
    let recent_import_jobs = use_state(Vec::<CodexAccountImportJobSummaryView>::new);
    let active_import_job = use_state(|| None::<CodexAccountImportJobDetailView>);
    let account_action_inflight = use_state(HashSet::<String>::new);
    let account_proxy_inputs = use_state(BTreeMap::<String, String>::new);
    let account_route_weight_tier_inputs = use_state(BTreeMap::<String, String>::new);
    let account_request_max_inputs = use_state(BTreeMap::<String, String>::new);
    let account_request_min_inputs = use_state(BTreeMap::<String, String>::new);
    let account_image_enabled_inputs = use_state(BTreeMap::<String, bool>::new);
    let account_image_concurrency_inputs = use_state(BTreeMap::<String, String>::new);
    let show_import_form = use_state(|| false);
    let account_search = use_state(String::new);
    let account_active_query = use_state(String::new);
    let account_sort_mode = use_state(|| AccountSortMode::None);
    let account_show_unhealthy = use_state(|| false);
    let account_show_active_only = use_state(|| false);
    let account_page = use_state(|| 1_usize);
    let accounts_total = use_state(|| 0_usize);
    let account_page_limit = use_state(|| ACCOUNT_PAGE_SIZE);
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
        let proxy_configs = proxy_configs.clone();
        let account_groups_page_items = account_groups_page_items.clone();
        let account_groups_total = account_groups_total.clone();
        let account_groups_page = account_groups_page.clone();
        let account_groups_page_limit = account_groups_page_limit.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        let accounts = accounts.clone();
        let accounts_summary = accounts_summary.clone();
        let codex_rate_limit_status = codex_rate_limit_status.clone();
        let accounts_total = accounts_total.clone();
        let account_page_limit = account_page_limit.clone();
        let active_tab = active_tab.clone();
        let recent_import_jobs = recent_import_jobs.clone();
        let account_proxy_inputs = account_proxy_inputs.clone();
        let account_route_weight_tier_inputs = account_route_weight_tier_inputs.clone();
        let account_request_max_inputs = account_request_max_inputs.clone();
        let account_request_min_inputs = account_request_min_inputs.clone();
        let account_image_enabled_inputs = account_image_enabled_inputs.clone();
        let account_image_concurrency_inputs = account_image_concurrency_inputs.clone();
        let account_group_candidate_accounts = account_group_candidate_accounts.clone();
        let account_group_candidate_loading = account_group_candidate_loading.clone();
        let account_active_query = account_active_query.clone();
        let account_sort_mode = account_sort_mode.clone();
        let account_show_unhealthy = account_show_unhealthy.clone();
        let account_show_active_only = account_show_active_only.clone();
        let account_page = account_page.clone();
        let reload_base_loaded = reload_base_loaded.clone();
        Callback::from(move |force_base: bool| {
            let keys_summary = keys_summary.clone();
            let proxy_configs = proxy_configs.clone();
            let account_groups_page_items = account_groups_page_items.clone();
            let account_groups_total = account_groups_total.clone();
            let account_groups_page = account_groups_page.clone();
            let account_groups_page_limit = account_groups_page_limit.clone();
            let loading = loading.clone();
            let load_error = load_error.clone();
            let accounts = accounts.clone();
            let accounts_summary = accounts_summary.clone();
            let codex_rate_limit_status = codex_rate_limit_status.clone();
            let accounts_total = accounts_total.clone();
            let account_page_limit = account_page_limit.clone();
            let active_tab = active_tab.clone();
            let recent_import_jobs = recent_import_jobs.clone();
            let account_proxy_inputs = account_proxy_inputs.clone();
            let account_route_weight_tier_inputs = account_route_weight_tier_inputs.clone();
            let account_request_max_inputs = account_request_max_inputs.clone();
            let account_request_min_inputs = account_request_min_inputs.clone();
            let account_image_enabled_inputs = account_image_enabled_inputs.clone();
            let account_image_concurrency_inputs = account_image_concurrency_inputs.clone();
            let account_group_candidate_accounts = account_group_candidate_accounts.clone();
            let account_group_candidate_loading = account_group_candidate_loading.clone();
            let account_active_query = account_active_query.clone();
            let account_sort_mode = account_sort_mode.clone();
            let account_show_unhealthy = account_show_unhealthy.clone();
            let account_show_active_only = account_show_active_only.clone();
            let account_page = account_page.clone();
            let reload_base_loaded = reload_base_loaded.clone();
            let refresh_base = force_base || !*reload_base_loaded;
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let active_tab_value = active_tab.clone();
                let current_group_page = (*account_groups_page).max(1);
                let current_account_page = (*account_page).max(1);
                let account_query = AdminLlmGatewayAccountPageQuery {
                    q: Some((*account_active_query).clone()),
                    active_only: *account_show_active_only,
                    unhealthy_only: *account_show_unhealthy,
                    sort: Some(
                        match *account_sort_mode {
                            AccountSortMode::PrimaryAsc => "primary_asc",
                            AccountSortMode::PrimaryDesc => "primary_desc",
                            AccountSortMode::SecondaryAsc => "secondary_asc",
                            AccountSortMode::SecondaryDesc => "secondary_desc",
                            AccountSortMode::None => "",
                        }
                        .to_string(),
                    ),
                };
                let result = async {
                    let base = if refresh_base {
                        let (key_summary_result, account_summary_result, proxy_configs_result) =
                            futures::join!(
                                fetch_admin_llm_gateway_keys_page(1, 0),
                                fetch_admin_llm_gateway_accounts_page(1, 0),
                                fetch_admin_llm_gateway_proxy_configs(),
                            );
                        Some((
                            key_summary_result?,
                            account_summary_result?,
                            proxy_configs_result?.proxy_configs,
                        ))
                    } else {
                        None
                    };
                    let account_groups_page_resp = if active_tab_value == TAB_GROUPS {
                        let limit = *account_groups_page_limit;
                        let offset = current_group_page.saturating_sub(1) * limit.max(1);
                        Some(fetch_admin_llm_gateway_account_groups_page(limit, offset).await?)
                    } else {
                        None
                    };
                    let accounts_resp = if active_tab_value == TAB_ACCOUNTS {
                        let limit = ACCOUNT_PAGE_SIZE.max(1);
                        let offset = current_account_page.saturating_sub(1) * limit;
                        Some(
                            fetch_admin_llm_gateway_accounts_page_with_query(
                                limit,
                                offset,
                                &account_query,
                            )
                            .await?,
                        )
                    } else {
                        None
                    };
                    let codex_status_resp = if active_tab_value == TAB_ACCOUNTS {
                        Some(fetch_llm_gateway_status().await?)
                    } else {
                        None
                    };
                    let import_jobs = if should_load_llm_gateway_import_jobs(&active_tab_value) {
                        Some(
                            fetch_admin_llm_gateway_account_import_jobs(Some(
                                ADMIN_CODEX_IMPORT_JOB_LIST_LIMIT,
                            ))
                            .await?,
                        )
                    } else {
                        None
                    };
                    Ok::<_, String>((
                        base,
                        account_groups_page_resp,
                        accounts_resp,
                        codex_status_resp,
                        import_jobs,
                    ))
                }
                .await;

                match result {
                    Ok((
                        base,
                        account_groups_page_resp,
                        accounts_resp,
                        codex_status_resp,
                        import_jobs,
                    )) => {
                        if let Some((key_summary_resp, account_summary_resp, proxy_config_items)) =
                            base
                        {
                            keys_summary.set(key_summary_resp.summary);
                            accounts_summary.set(account_summary_resp.summary);
                            proxy_configs.set(proxy_config_items);
                            reload_base_loaded.set(true);
                        }
                        if let Some(account_groups_page_resp) = account_groups_page_resp {
                            let effective_limit = account_groups_page_resp.limit.max(1);
                            let total_pages = admin_group_total_pages(
                                account_groups_page_resp.total,
                                effective_limit,
                            );
                            account_groups_total.set(account_groups_page_resp.total);
                            account_groups_page_limit.set(effective_limit);
                            if current_group_page > total_pages {
                                account_groups_page.set(total_pages);
                            } else {
                                account_groups_page_items.set(account_groups_page_resp.groups);
                            }
                        }
                        if let Some(accounts_resp) = accounts_resp {
                            let next_proxy_inputs = accounts_resp
                                .accounts
                                .iter()
                                .map(|account| {
                                    (account.name.clone(), account_proxy_select_value(account))
                                })
                                .collect::<BTreeMap<_, _>>();
                            let next_route_weight_tier_inputs = accounts_resp
                                .accounts
                                .iter()
                                .map(|account| {
                                    (
                                        account.name.clone(),
                                        if account.route_weight_tier.trim().is_empty() {
                                            "auto".to_string()
                                        } else {
                                            account.route_weight_tier.clone()
                                        },
                                    )
                                })
                                .collect::<BTreeMap<_, _>>();
                            let next_request_max_inputs = accounts_resp
                                .accounts
                                .iter()
                                .map(|account| {
                                    (
                                        account.name.clone(),
                                        account
                                            .request_max_concurrency
                                            .map(|value| value.to_string())
                                            .unwrap_or_default(),
                                    )
                                })
                                .collect::<BTreeMap<_, _>>();
                            let next_request_min_inputs = accounts_resp
                                .accounts
                                .iter()
                                .map(|account| {
                                    (
                                        account.name.clone(),
                                        account
                                            .request_min_start_interval_ms
                                            .map(|value| value.to_string())
                                            .unwrap_or_default(),
                                    )
                                })
                                .collect::<BTreeMap<_, _>>();
                            let next_image_enabled_inputs = accounts_resp
                                .accounts
                                .iter()
                                .map(|account| {
                                    (account.name.clone(), account.codex_image_generation_enabled)
                                })
                                .collect::<BTreeMap<_, _>>();
                            let next_image_concurrency_inputs = accounts_resp
                                .accounts
                                .iter()
                                .map(|account| {
                                    (
                                        account.name.clone(),
                                        account.codex_image_generation_max_concurrency.to_string(),
                                    )
                                })
                                .collect::<BTreeMap<_, _>>();
                            accounts_total.set(accounts_resp.total);
                            account_page_limit.set(accounts_resp.limit.max(1));
                            accounts.set(accounts_resp.accounts);
                            account_proxy_inputs.set(next_proxy_inputs);
                            account_route_weight_tier_inputs.set(next_route_weight_tier_inputs);
                            account_request_max_inputs.set(next_request_max_inputs);
                            account_request_min_inputs.set(next_request_min_inputs);
                            account_image_enabled_inputs.set(next_image_enabled_inputs);
                            account_image_concurrency_inputs.set(next_image_concurrency_inputs);
                            codex_rate_limit_status.set(codex_status_resp);
                        } else if active_tab_value != TAB_GROUPS {
                            accounts_total.set(0);
                            accounts.set(Vec::new());
                            codex_rate_limit_status.set(None);
                            account_proxy_inputs.set(BTreeMap::new());
                            account_route_weight_tier_inputs.set(BTreeMap::new());
                            account_request_max_inputs.set(BTreeMap::new());
                            account_request_min_inputs.set(BTreeMap::new());
                            account_image_enabled_inputs.set(BTreeMap::new());
                            account_image_concurrency_inputs.set(BTreeMap::new());
                        }
                        if active_tab_value != TAB_GROUPS {
                            account_group_candidate_accounts.set(Vec::new());
                            account_group_candidate_loading.set(false);
                        }
                        if let Some(import_jobs) = import_jobs {
                            recent_import_jobs.set(import_jobs);
                        }
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                loading.set(false);
            });
        })
    };

    let load_account_group_candidates = {
        let account_group_candidate_accounts = account_group_candidate_accounts.clone();
        let account_group_candidate_loading = account_group_candidate_loading.clone();
        let load_error = load_error.clone();
        Callback::from(move |_| {
            account_group_candidate_loading.set(true);
            let account_group_candidate_accounts = account_group_candidate_accounts.clone();
            let account_group_candidate_loading = account_group_candidate_loading.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_llm_gateway_accounts().await {
                    Ok(resp) => {
                        account_group_candidate_accounts.set(resp.accounts);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                account_group_candidate_loading.set(false);
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
        let reload = reload.clone();
        let active_tab = active_tab.clone();
        let account_page = account_page.clone();
        let account_active_query = account_active_query.clone();
        let account_sort_mode = account_sort_mode.clone();
        let account_show_unhealthy = account_show_unhealthy.clone();
        let account_show_active_only = account_show_active_only.clone();
        use_effect_with(
            (
                *account_page,
                (*account_active_query).clone(),
                *account_sort_mode,
                *account_show_unhealthy,
                *account_show_active_only,
            ),
            move |_| {
                if active_tab == TAB_ACCOUNTS {
                    reload.emit(false);
                }
                || ()
            },
        );
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

    {
        let active_import_job = active_import_job.clone();
        let recent_import_jobs = recent_import_jobs.clone();
        let reload = reload.clone();
        let load_error = load_error.clone();
        use_effect_with((*active_import_job).clone(), move |job_detail| {
            let interval = job_detail.clone().and_then(|job_detail| {
                if codex_import_job_is_terminal(&job_detail.summary.status) {
                    return None;
                }
                let job_id = job_detail.summary.job_id.clone();
                Some(Interval::new(1500, move || {
                    let active_import_job = active_import_job.clone();
                    let recent_import_jobs = recent_import_jobs.clone();
                    let reload = reload.clone();
                    let load_error = load_error.clone();
                    let job_id = job_id.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        match fetch_admin_llm_gateway_account_import_job(&job_id).await {
                            Ok(detail) => {
                                let summary = detail.summary.clone();
                                let is_terminal = codex_import_job_is_terminal(&summary.status);
                                active_import_job.set(Some(detail));
                                recent_import_jobs.set(upsert_codex_import_job_summary(
                                    &recent_import_jobs,
                                    summary,
                                ));
                                if is_terminal {
                                    reload.emit(true);
                                }
                            },
                            Err(err) => load_error.set(Some(err)),
                        }
                    });
                }))
            });
            move || drop(interval)
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
        let load_account_group_candidates = load_account_group_candidates.clone();
        Callback::from(move |_| {
            let next_expanded = !*account_group_form_expanded;
            account_group_form_expanded.set(next_expanded);
            if next_expanded {
                load_account_group_candidates.emit(());
            }
        })
    };

    let on_create_account_group = {
        let create_account_group_name = create_account_group_name.clone();
        let create_account_group_account_names = create_account_group_account_names.clone();
        let creating_account_group = creating_account_group.clone();
        let flash = flash.clone();
        let load_error = load_error.clone();
        let reload = reload.clone();
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
            let reload = reload.clone();
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
                        reload.emit(true);
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

    let on_toggle_account_spark_mapping = {
        let account_action_inflight = account_action_inflight.clone();
        let accounts = accounts.clone();
        let load_error = load_error.clone();
        Callback::from(move |(account_name, enabled): (String, bool)| {
            let account_action_inflight = account_action_inflight.clone();
            let accounts = accounts.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match patch_admin_llm_gateway_account(
                    &account_name,
                    &PatchAdminLlmGatewayAccountInput {
                        status: None,
                        map_gpt53_codex_to_spark: Some(enabled),
                        auto_refresh_enabled: None,
                        route_weight_tier: None,
                        proxy_mode: None,
                        proxy_config_id: None,
                        request_max_concurrency: None,
                        request_min_start_interval_ms: None,
                        codex_image_generation_enabled: None,
                        codex_image_generation_max_concurrency: None,
                        request_max_concurrency_unlimited: false,
                        request_min_start_interval_ms_unlimited: false,
                    },
                )
                .await
                {
                    Ok(updated) => {
                        let mut items = (*accounts).clone();
                        if let Some(item) = items.iter_mut().find(|item| item.name == updated.name)
                        {
                            *item = updated;
                        }
                        accounts.set(items);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_action_inflight).clone();
                inflight.remove(&account_name);
                account_action_inflight.set(inflight);
            });
        })
    };

    let on_toggle_account_auto_refresh = {
        let account_action_inflight = account_action_inflight.clone();
        let accounts = accounts.clone();
        let load_error = load_error.clone();
        Callback::from(move |(account_name, enabled): (String, bool)| {
            let account_action_inflight = account_action_inflight.clone();
            let accounts = accounts.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match patch_admin_llm_gateway_account(
                    &account_name,
                    &PatchAdminLlmGatewayAccountInput {
                        status: None,
                        map_gpt53_codex_to_spark: None,
                        auto_refresh_enabled: Some(enabled),
                        route_weight_tier: None,
                        proxy_mode: None,
                        proxy_config_id: None,
                        request_max_concurrency: None,
                        request_min_start_interval_ms: None,
                        codex_image_generation_enabled: None,
                        codex_image_generation_max_concurrency: None,
                        request_max_concurrency_unlimited: false,
                        request_min_start_interval_ms_unlimited: false,
                    },
                )
                .await
                {
                    Ok(updated) => {
                        let mut items = (*accounts).clone();
                        if let Some(item) = items.iter_mut().find(|item| item.name == updated.name)
                        {
                            *item = updated;
                        }
                        accounts.set(items);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_action_inflight).clone();
                inflight.remove(&account_name);
                account_action_inflight.set(inflight);
            });
        })
    };

    let on_toggle_account_status = {
        let account_action_inflight = account_action_inflight.clone();
        let accounts = accounts.clone();
        let load_error = load_error.clone();
        Callback::from(move |(account_name, status): (String, String)| {
            let account_action_inflight = account_action_inflight.clone();
            let accounts = accounts.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match patch_admin_llm_gateway_account(
                    &account_name,
                    &PatchAdminLlmGatewayAccountInput {
                        status: Some(status),
                        map_gpt53_codex_to_spark: None,
                        auto_refresh_enabled: None,
                        route_weight_tier: None,
                        proxy_mode: None,
                        proxy_config_id: None,
                        request_max_concurrency: None,
                        request_min_start_interval_ms: None,
                        codex_image_generation_enabled: None,
                        codex_image_generation_max_concurrency: None,
                        request_max_concurrency_unlimited: false,
                        request_min_start_interval_ms_unlimited: false,
                    },
                )
                .await
                {
                    Ok(updated) => {
                        let mut items = (*accounts).clone();
                        if let Some(item) = items.iter_mut().find(|item| item.name == updated.name)
                        {
                            *item = updated;
                        }
                        accounts.set(items);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_action_inflight).clone();
                inflight.remove(&account_name);
                account_action_inflight.set(inflight);
            });
        })
    };

    let on_save_account_settings = {
        let account_action_inflight = account_action_inflight.clone();
        let account_proxy_inputs = account_proxy_inputs.clone();
        let account_route_weight_tier_inputs = account_route_weight_tier_inputs.clone();
        let account_request_max_inputs = account_request_max_inputs.clone();
        let account_request_min_inputs = account_request_min_inputs.clone();
        let account_image_enabled_inputs = account_image_enabled_inputs.clone();
        let account_image_concurrency_inputs = account_image_concurrency_inputs.clone();
        let accounts = accounts.clone();
        let load_error = load_error.clone();
        Callback::from(move |account_name: String| {
            let account_action_inflight = account_action_inflight.clone();
            let account_proxy_inputs = account_proxy_inputs.clone();
            let account_route_weight_tier_inputs = account_route_weight_tier_inputs.clone();
            let account_request_max_inputs = account_request_max_inputs.clone();
            let account_request_min_inputs = account_request_min_inputs.clone();
            let account_image_enabled_inputs = account_image_enabled_inputs.clone();
            let account_image_concurrency_inputs = account_image_concurrency_inputs.clone();
            let accounts = accounts.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let current_account = (*accounts)
                    .iter()
                    .find(|account| account.name == account_name)
                    .cloned();
                let selection = (*account_proxy_inputs)
                    .get(&account_name)
                    .cloned()
                    .unwrap_or_else(|| "inherit".to_string());
                let route_weight_tier = (*account_route_weight_tier_inputs)
                    .get(&account_name)
                    .cloned()
                    .unwrap_or_else(|| "auto".to_string());
                let request_max_raw = (*account_request_max_inputs)
                    .get(&account_name)
                    .cloned()
                    .unwrap_or_default();
                let request_min_raw = (*account_request_min_inputs)
                    .get(&account_name)
                    .cloned()
                    .unwrap_or_default();
                let image_enabled = (*account_image_enabled_inputs)
                    .get(&account_name)
                    .copied()
                    .or_else(|| {
                        current_account
                            .as_ref()
                            .map(|account| account.codex_image_generation_enabled)
                    })
                    .unwrap_or(false);
                let image_concurrency_raw = (*account_image_concurrency_inputs)
                    .get(&account_name)
                    .cloned()
                    .or_else(|| {
                        current_account.as_ref().map(|account| {
                            account.codex_image_generation_max_concurrency.to_string()
                        })
                    })
                    .unwrap_or_else(|| CODEX_IMAGE_DEFAULT_CONCURRENCY.to_string());
                let (proxy_mode, proxy_config_id) = if selection == "direct" {
                    (Some("direct".to_string()), None)
                } else if let Some(proxy_config_id) = selection.strip_prefix("fixed:") {
                    (Some("fixed".to_string()), Some(proxy_config_id.to_string()))
                } else {
                    (Some("inherit".to_string()), None)
                };
                let request_max_concurrency = if request_max_raw.trim().is_empty() {
                    None
                } else {
                    match request_max_raw.trim().parse::<u64>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            load_error
                                .set(Some("账号并发上限必须是整数，留空表示不限制".to_string()));
                            return;
                        },
                    }
                };
                let request_min_start_interval_ms = if request_min_raw.trim().is_empty() {
                    None
                } else {
                    match request_min_raw.trim().parse::<u64>() {
                        Ok(value) => Some(value),
                        Err(_) => {
                            load_error.set(Some(
                                "账号请求起始间隔必须是整数毫秒，留空表示不限制".to_string(),
                            ));
                            return;
                        },
                    }
                };
                let codex_image_generation_max_concurrency = if image_concurrency_raw
                    .trim()
                    .is_empty()
                {
                    CODEX_IMAGE_DEFAULT_CONCURRENCY
                } else {
                    match image_concurrency_raw.trim().parse::<u64>() {
                        Ok(value) if (1..=CODEX_IMAGE_MAX_CONCURRENCY).contains(&value) => value,
                        _ => {
                            load_error.set(Some(format!(
                                "生图并发必须是 1..={CODEX_IMAGE_MAX_CONCURRENCY} 的整数"
                            )));
                            return;
                        },
                    }
                };

                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match patch_admin_llm_gateway_account(
                    &account_name,
                    &PatchAdminLlmGatewayAccountInput {
                        status: None,
                        map_gpt53_codex_to_spark: None,
                        auto_refresh_enabled: None,
                        route_weight_tier: Some(route_weight_tier),
                        proxy_mode,
                        proxy_config_id,
                        request_max_concurrency,
                        request_min_start_interval_ms,
                        codex_image_generation_enabled: Some(image_enabled),
                        codex_image_generation_max_concurrency: Some(
                            codex_image_generation_max_concurrency,
                        ),
                        request_max_concurrency_unlimited: request_max_concurrency.is_none(),
                        request_min_start_interval_ms_unlimited: request_min_start_interval_ms
                            .is_none(),
                    },
                )
                .await
                {
                    Ok(updated) => {
                        let mut items = (*accounts).clone();
                        if let Some(item) = items.iter_mut().find(|item| item.name == updated.name)
                        {
                            *item = updated.clone();
                        }
                        accounts.set(items);

                        let mut next_inputs = (*account_proxy_inputs).clone();
                        next_inputs
                            .insert(updated.name.clone(), account_proxy_select_value(&updated));
                        account_proxy_inputs.set(next_inputs);
                        let mut next_route_weight_tier_inputs =
                            (*account_route_weight_tier_inputs).clone();
                        next_route_weight_tier_inputs
                            .insert(updated.name.clone(), updated.route_weight_tier.clone());
                        account_route_weight_tier_inputs.set(next_route_weight_tier_inputs);
                        let mut next_request_max_inputs = (*account_request_max_inputs).clone();
                        next_request_max_inputs.insert(
                            updated.name.clone(),
                            updated
                                .request_max_concurrency
                                .map(|value| value.to_string())
                                .unwrap_or_default(),
                        );
                        account_request_max_inputs.set(next_request_max_inputs);
                        let mut next_request_min_inputs = (*account_request_min_inputs).clone();
                        next_request_min_inputs.insert(
                            updated.name.clone(),
                            updated
                                .request_min_start_interval_ms
                                .map(|value| value.to_string())
                                .unwrap_or_default(),
                        );
                        account_request_min_inputs.set(next_request_min_inputs);
                        let mut next_image_enabled_inputs = (*account_image_enabled_inputs).clone();
                        next_image_enabled_inputs
                            .insert(updated.name.clone(), updated.codex_image_generation_enabled);
                        account_image_enabled_inputs.set(next_image_enabled_inputs);
                        let mut next_image_concurrency_inputs =
                            (*account_image_concurrency_inputs).clone();
                        next_image_concurrency_inputs.insert(
                            updated.name.clone(),
                            updated.codex_image_generation_max_concurrency.to_string(),
                        );
                        account_image_concurrency_inputs.set(next_image_concurrency_inputs);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_action_inflight).clone();
                inflight.remove(&account_name);
                account_action_inflight.set(inflight);
            });
        })
    };

    let on_refresh_account_auth = {
        let account_action_inflight = account_action_inflight.clone();
        let account_proxy_inputs = account_proxy_inputs.clone();
        let accounts = accounts.clone();
        let flash = flash.clone();
        let load_error = load_error.clone();
        Callback::from(move |account_name: String| {
            let account_action_inflight = account_action_inflight.clone();
            let account_proxy_inputs = account_proxy_inputs.clone();
            let accounts = accounts.clone();
            let flash = flash.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match refresh_admin_llm_gateway_account_auth(&account_name).await {
                    Ok(updated) => {
                        let mut items = (*accounts).clone();
                        if let Some(item) = items.iter_mut().find(|item| item.name == updated.name)
                        {
                            *item = updated.clone();
                        }
                        accounts.set(items);

                        let mut next_inputs = (*account_proxy_inputs).clone();
                        next_inputs
                            .insert(updated.name.clone(), account_proxy_select_value(&updated));
                        account_proxy_inputs.set(next_inputs);
                        load_error.set(None);
                        flash.emit((format!("已刷新账号 `{}` 的 token", updated.name), false));
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((
                            format!("刷新账号 `{}` 的 token 失败\n{err}", account_name),
                            true,
                        ));
                    },
                }

                let mut inflight = (*account_action_inflight).clone();
                inflight.remove(&account_name);
                account_action_inflight.set(inflight);
            });
        })
    };

    let on_refresh_account_usage = {
        let account_action_inflight = account_action_inflight.clone();
        let account_proxy_inputs = account_proxy_inputs.clone();
        let accounts = accounts.clone();
        let codex_rate_limit_status = codex_rate_limit_status.clone();
        let flash = flash.clone();
        let load_error = load_error.clone();
        Callback::from(move |account_name: String| {
            let account_action_inflight = account_action_inflight.clone();
            let account_proxy_inputs = account_proxy_inputs.clone();
            let accounts = accounts.clone();
            let codex_rate_limit_status = codex_rate_limit_status.clone();
            let flash = flash.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match refresh_admin_llm_gateway_account_usage(&account_name).await {
                    Ok(updated) => {
                        let mut items = (*accounts).clone();
                        if let Some(item) = items.iter_mut().find(|item| item.name == updated.name)
                        {
                            *item = updated.clone();
                        }
                        accounts.set(items);

                        let mut next_inputs = (*account_proxy_inputs).clone();
                        next_inputs
                            .insert(updated.name.clone(), account_proxy_select_value(&updated));
                        account_proxy_inputs.set(next_inputs);
                        if let Ok(status) = fetch_llm_gateway_status().await {
                            codex_rate_limit_status.set(Some(status));
                        }
                        load_error.set(None);
                        flash.emit((format!("已刷新账号 `{}` 的 usage", updated.name), false));
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((
                            format!("刷新账号 `{}` 的 usage 失败\n{err}", account_name),
                            true,
                        ));
                    },
                }

                let mut inflight = (*account_action_inflight).clone();
                inflight.remove(&account_name);
                account_action_inflight.set(inflight);
            });
        })
    };

    let on_consume_account_reset_credit = {
        let account_action_inflight = account_action_inflight.clone();
        let account_proxy_inputs = account_proxy_inputs.clone();
        let accounts = accounts.clone();
        let codex_rate_limit_status = codex_rate_limit_status.clone();
        let flash = flash.clone();
        let load_error = load_error.clone();
        Callback::from(move |account_name: String| {
            let account_action_inflight = account_action_inflight.clone();
            let account_proxy_inputs = account_proxy_inputs.clone();
            let accounts = accounts.clone();
            let codex_rate_limit_status = codex_rate_limit_status.clone();
            let flash = flash.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match consume_admin_llm_gateway_account_rate_limit_reset_credit(&account_name).await
                {
                    Ok(result) => {
                        let updated = result.account;
                        let mut items = (*accounts).clone();
                        if let Some(item) = items.iter_mut().find(|item| item.name == updated.name)
                        {
                            *item = updated.clone();
                        }
                        accounts.set(items);

                        let mut next_inputs = (*account_proxy_inputs).clone();
                        next_inputs
                            .insert(updated.name.clone(), account_proxy_select_value(&updated));
                        account_proxy_inputs.set(next_inputs);
                        if let Ok(status) = fetch_llm_gateway_status().await {
                            codex_rate_limit_status.set(Some(status));
                        }
                        load_error.set(None);
                        let message = match result.code.as_str() {
                            "reset" => format!(
                                "已使用账号 `{}` 的 reset credit，重置 {} 个窗口",
                                updated.name, result.windows_reset
                            ),
                            "nothing_to_reset" => {
                                format!("账号 `{}` 当前没有需要重置的限额窗口", updated.name)
                            },
                            "no_credit" => {
                                format!("账号 `{}` 当前没有可用 reset credit", updated.name)
                            },
                            "already_redeemed" => {
                                format!("账号 `{}` 的本次 reset 请求已处理过", updated.name)
                            },
                            other => {
                                format!("账号 `{}` reset credit 返回 `{}`", updated.name, other)
                            },
                        };
                        flash.emit((message, false));
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((
                            format!("使用账号 `{}` 的 reset credit 失败\n{err}", account_name),
                            true,
                        ));
                    },
                }

                let mut inflight = (*account_action_inflight).clone();
                inflight.remove(&account_name);
                account_action_inflight.set(inflight);
            });
        })
    };

    let on_probe_account_models = {
        let account_action_inflight = account_action_inflight.clone();
        let flash = flash.clone();
        let load_error = load_error.clone();
        Callback::from(move |account_name: String| {
            let account_action_inflight = account_action_inflight.clone();
            let flash = flash.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match probe_admin_llm_gateway_account_models(&account_name).await {
                    Ok(result) => {
                        load_error.set(None);
                        flash.emit((format!("账号 `{}` {}", account_name, result.message), false));
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((
                            format!("检查账号 `{}` 的 models 失败\n{err}", account_name),
                            true,
                        ));
                    },
                }

                let mut inflight = (*account_action_inflight).clone();
                inflight.remove(&account_name);
                account_action_inflight.set(inflight);
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

    let on_import_account = {
        let import_name = import_name.clone();
        let import_id_token = import_id_token.clone();
        let import_access_token = import_access_token.clone();
        let import_refresh_token = import_refresh_token.clone();
        let import_account_id = import_account_id.clone();
        let import_raw_auth_json = import_raw_auth_json.clone();
        let import_raw_auth_feedback = import_raw_auth_feedback.clone();
        let importing = importing.clone();
        let load_error = load_error.clone();
        let reload = reload.clone();
        Callback::from(move |_| {
            let name = (*import_name).trim().to_string();
            let id_token = (*import_id_token).trim().to_string();
            let access_token = (*import_access_token).trim().to_string();
            let refresh_token = (*import_refresh_token).trim().to_string();
            let raw_auth_json = (*import_raw_auth_json).trim().to_string();
            let account_id = {
                let v = (*import_account_id).trim().to_string();
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            };
            let importing = importing.clone();
            let load_error = load_error.clone();
            let reload = reload.clone();
            let import_name = import_name.clone();
            let import_id_token = import_id_token.clone();
            let import_access_token = import_access_token.clone();
            let import_refresh_token = import_refresh_token.clone();
            let import_account_id = import_account_id.clone();
            let import_raw_auth_json = import_raw_auth_json.clone();
            let import_raw_auth_feedback = import_raw_auth_feedback.clone();
            wasm_bindgen_futures::spawn_local(async move {
                importing.set(true);
                let raw_auth_json_ref =
                    (!raw_auth_json.is_empty()).then_some(raw_auth_json.as_str());
                match import_admin_llm_gateway_account(
                    &name,
                    &id_token,
                    &access_token,
                    &refresh_token,
                    account_id.as_deref(),
                    raw_auth_json_ref,
                )
                .await
                {
                    Ok(_) => {
                        import_name.set(String::new());
                        import_id_token.set(String::new());
                        import_access_token.set(String::new());
                        import_refresh_token.set(String::new());
                        import_account_id.set(String::new());
                        import_raw_auth_json.set(String::new());
                        import_raw_auth_feedback.set(None);
                        load_error.set(None);
                        reload.emit(true);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                importing.set(false);
            });
        })
    };

    let on_import_account_batch = {
        let batch_import_raw_json = batch_import_raw_json.clone();
        let batch_import_feedback = batch_import_feedback.clone();
        let batch_import_validate_before_import = batch_import_validate_before_import.clone();
        let batch_importing = batch_importing.clone();
        let recent_import_jobs = recent_import_jobs.clone();
        let active_import_job = active_import_job.clone();
        let load_error = load_error.clone();
        Callback::from(move |_| {
            let raw_json = (*batch_import_raw_json).trim().to_string();
            let items = match parse_admin_codex_batch_import_json(&raw_json) {
                Ok(items) => items,
                Err(err) => {
                    batch_import_feedback.set(Some((err, true)));
                    return;
                },
            };
            let validate_before_import = *batch_import_validate_before_import;
            let batch_import_raw_json = batch_import_raw_json.clone();
            let batch_import_feedback = batch_import_feedback.clone();
            let batch_importing = batch_importing.clone();
            let recent_import_jobs = recent_import_jobs.clone();
            let active_import_job = active_import_job.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                batch_importing.set(true);
                batch_import_feedback.set(None);
                match create_admin_llm_gateway_account_import_job(validate_before_import, &items)
                    .await
                {
                    Ok(detail) => {
                        let summary = detail.summary.clone();
                        let next_jobs =
                            upsert_codex_import_job_summary(&recent_import_jobs, summary.clone());
                        active_import_job.set(Some(detail));
                        recent_import_jobs.set(next_jobs);
                        batch_import_raw_json.set(String::new());
                        batch_import_feedback
                            .set(Some((format!("已创建批量导入作业 {}", summary.job_id), false)));
                        load_error.set(None);
                    },
                    Err(err) => {
                        batch_import_feedback.set(Some((err.clone(), true)));
                        load_error.set(Some(err));
                    },
                }
                batch_importing.set(false);
            });
        })
    };

    let on_load_import_job = {
        let active_import_job = active_import_job.clone();
        let load_error = load_error.clone();
        Callback::from(move |job_id: String| {
            let active_import_job = active_import_job.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_llm_gateway_account_import_job(&job_id).await {
                    Ok(detail) => {
                        active_import_job.set(Some(detail));
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
            });
        })
    };

    let on_delete_account = {
        let reload = reload.clone();
        let load_error = load_error.clone();
        Callback::from(move |name: String| {
            if !confirm_destructive(&format!("确认删除账号 {} ？", name)) {
                return;
            }
            let reload = reload.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match delete_admin_llm_gateway_account(&name).await {
                    Ok(_) => reload.emit(true),
                    Err(err) => load_error.set(Some(err)),
                }
            });
        })
    };

    let key_summary = *keys_summary;
    let account_summary = *accounts_summary;
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

    let account_total_pages = admin_group_total_pages(*accounts_total, *account_page_limit);
    let account_current_page = (*account_page).clamp(1, account_total_pages);
    let account_page_entries: Vec<&AccountSummaryView> = accounts.iter().collect();
    let on_account_page_change = {
        let account_page = account_page.clone();
        Callback::from(move |p: usize| account_page.set(p))
    };
    let on_account_search_submit = {
        let account_search = account_search.clone();
        let account_active_query = account_active_query.clone();
        let account_page = account_page.clone();
        Callback::from(move |_: ()| {
            account_active_query.set((*account_search).clone());
            account_page.set(1);
        })
    };
    let on_account_search_input = {
        let account_search = account_search.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<HtmlInputElement>() {
                account_search.set(target.value());
            }
        })
    };
    let on_account_search_keydown = {
        let on_account_search_submit = on_account_search_submit.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_account_search_submit.emit(());
            }
        })
    };
    let on_account_search_clear = {
        let account_search = account_search.clone();
        let account_active_query = account_active_query.clone();
        let account_page = account_page.clone();
        Callback::from(move |_: MouseEvent| {
            account_search.set(String::new());
            account_active_query.set(String::new());
            account_page.set(1);
        })
    };

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

                if active_tab == TAB_GROUPS {
                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Account Groups" }</h2>
                            <p class={classes!("mt-2", "mb-0", "text-sm", "text-[var(--muted)]")}>
                                { "先为账号分组，再让 key 选择组而不是直接勾账号。固定路由请选择单账号组；自动路由可以选任意组，留空则继续使用全账号池。" }
                            </p>
                        </div>
                        <button
                            class={classes!("btn-terminal")}
                            onclick={{
                                let reload = reload.clone();
                                Callback::from(move |_| reload.emit(true))
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
                                class={classes!("btn-terminal")}
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
                                    if *account_group_candidate_loading {
                                        <div class={classes!("rounded-lg", "border", "border-dashed", "border-[var(--border)]", "px-3", "py-3", "text-xs", "text-[var(--muted)]")}>
                                            { "正在加载账号候选..." }
                                        </div>
                                    } else if account_group_candidate_accounts.is_empty() {
                                        <div class={classes!("rounded-lg", "border", "border-dashed", "border-[var(--border)]", "px-3", "py-3", "text-xs", "text-[var(--muted)]")}>
                                            { "当前没有可加入账号组的账号。" }
                                        </div>
                                    } else {
                                        <div class={classes!("grid", "gap-2", "xl:grid-cols-2")}>
                                            { for account_group_candidate_accounts.iter().map(|account| {
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
                                                            type="checkbox"
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
                                        class={classes!("btn-terminal", "btn-terminal-primary")}
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
                                    on_changed={reload.reform(|_: ()| true)}
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
                } // end TAB_GROUPS

                // ── Accounts Tab ──
                if active_tab == TAB_ACCOUNTS {
                // === Codex Accounts ===
                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Codex Accounts" }</h2>
                            <p class={classes!("mt-1", "m-0", "text-xs", "text-[var(--muted)]")}>
                                { format!("已导入 {} 个账号。这里会显示账号状态、usage 刷新健康度和账号级 proxy 配置。", account_summary.total) }
                            </p>
                        </div>
                        <button
                            type="button"
                            class={classes!("btn-terminal")}
                            onclick={{
                                let reload = reload.clone();
                                Callback::from(move |_| reload.emit(true))
                            }}
                            disabled={*loading}
                        >
                            <i class={classes!("fas", if *loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                            { if *loading { "刷新中..." } else { "刷新列表" } }
                        </button>
                    </div>

                    <div class={classes!("mt-3", "flex", "gap-2", "flex-wrap")}>
                        <button
                            type="button"
                            class={classes!("btn-terminal")}
                            onclick={{
                                let show_import_form = show_import_form.clone();
                                Callback::from(move |_| show_import_form.set(!*show_import_form))
                            }}
                        >
                            <i class={classes!("fas", if *show_import_form { "fa-chevron-up" } else { "fa-plus" })}></i>
                            { if *show_import_form { "收起单账号导入" } else { "导入单账号" } }
                        </button>
                        <button
                            type="button"
                            class={classes!("btn-terminal")}
                            onclick={{
                                let show_batch_import_form = show_batch_import_form.clone();
                                Callback::from(move |_| show_batch_import_form.set(!*show_batch_import_form))
                            }}
                        >
                            <i class={classes!("fas", if *show_batch_import_form { "fa-chevron-up" } else { "fa-layer-group" })}></i>
                            { if *show_batch_import_form { "收起批量导入" } else { "批量导入" } }
                        </button>
                    </div>

                    if *show_import_form {
                    <div class={classes!("mt-3", "grid", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                        <div class={classes!("grid", "gap-3", "md:grid-cols-2")}>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "名称 (唯一)" }</span>
                                <input
                                    type="text"
                                    placeholder="my-pro-account"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*import_name).clone()}
                                    oninput={{
                                        let import_name = import_name.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                import_name.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "account_id (可选)" }</span>
                                <input
                                    type="text"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*import_account_id).clone()}
                                    oninput={{
                                        let import_account_id = import_account_id.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                import_account_id.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            </div>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "auth.json（可直接粘贴导入）" }</span>
                                <textarea
                                    rows="4"
                                    placeholder="{\"tokens\":{...}}"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-xs")}
                                    value={(*import_raw_auth_json).clone()}
                                    oninput={{
                                        let import_raw_auth_json = import_raw_auth_json.clone();
                                        let import_raw_auth_feedback = import_raw_auth_feedback.clone();
                                        let import_account_id = import_account_id.clone();
                                        let import_id_token = import_id_token.clone();
                                        let import_access_token = import_access_token.clone();
                                        let import_refresh_token = import_refresh_token.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                                                let raw = target.value();
                                                let trimmed = raw.trim().to_string();
                                                import_raw_auth_json.set(raw);
                                                if trimmed.is_empty() {
                                                    import_raw_auth_feedback.set(None);
                                                    return;
                                                }
                                                match parse_admin_codex_auth_json(&trimmed) {
                                                    Ok(parsed) => {
                                                        import_account_id.set(parsed.account_id.unwrap_or_default());
                                                        import_id_token.set(parsed.id_token);
                                                        import_access_token.set(parsed.access_token);
                                                        import_refresh_token.set(parsed.refresh_token);
                                                        import_raw_auth_feedback.set(Some(("已解析并回填可识别字段；提交时会保留完整 JSON".to_string(), false)));
                                                    },
                                                    Err(err) => {
                                                        if trimmed.ends_with('}') || trimmed.contains('\n') {
                                                            import_raw_auth_feedback.set(Some((err, true)));
                                                        } else {
                                                            import_raw_auth_feedback.set(None);
                                                        }
                                                    },
                                                }
                                            }
                                        })
                                    }}
                                />
                                if let Some((message, is_error)) = (*import_raw_auth_feedback).clone() {
                                    <div class={classes!("mt-1", "font-mono", "text-[11px]", if is_error { "text-red-600 dark:text-red-300" } else { "text-emerald-600 dark:text-emerald-300" })}>
                                        { message }
                                    </div>
                                }
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "access_token" }</span>
                            <textarea
                                rows="2"
                                class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-xs")}
                                value={(*import_access_token).clone()}
                                oninput={{
                                    let import_access_token = import_access_token.clone();
                                    Callback::from(move |event: InputEvent| {
                                        if let Some(target) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                                            import_access_token.set(target.value());
                                        }
                                    })
                                }}
                            />
                        </label>
                        <div class={classes!("grid", "gap-3", "md:grid-cols-2")}>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "id_token" }</span>
                                <textarea
                                    rows="2"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-xs")}
                                    value={(*import_id_token).clone()}
                                    oninput={{
                                        let import_id_token = import_id_token.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                                                import_id_token.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "refresh_token" }</span>
                                <textarea
                                    rows="2"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-xs")}
                                    value={(*import_refresh_token).clone()}
                                    oninput={{
                                        let import_refresh_token = import_refresh_token.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                                                import_refresh_token.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                        </div>
                        <div class={classes!("flex", "justify-end")}>
                            <button class={classes!("btn-terminal", "btn-terminal-primary")} onclick={on_import_account} disabled={*importing}>
                                { if *importing { "导入验证中..." } else { "导入账号" } }
                            </button>
                        </div>
                    </div>
                    } // end show_import_form

                    if *show_batch_import_form {
                    <div class={classes!("mt-3", "grid", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                            <div>
                                <h3 class={classes!("m-0", "text-sm", "font-semibold", "text-[var(--text)]")}>{ "本地 JSON 数组批量导入" }</h3>
                                <p class={classes!("mt-1", "mb-0", "text-xs", "text-[var(--muted)]")}>
                                    { "每项至少带 name 和 auth_json/tokens。开启验证后会先走默认 Codex 代理做 refresh 校验，再真正入库。" }
                                </p>
                            </div>
                            <label class={classes!("flex", "items-center", "gap-2", "text-xs", "text-[var(--muted)]")}>
                                <input
                                    type="checkbox"
                                    checked={*batch_import_validate_before_import}
                                    onchange={{
                                        let batch_import_validate_before_import =
                                            batch_import_validate_before_import.clone();
                                        Callback::from(move |event: Event| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                batch_import_validate_before_import.set(target.checked());
                                            }
                                        })
                                    }}
                                />
                                <span>{ "提交前 refresh 验证" }</span>
                            </label>
                        </div>
                        <textarea
                            rows="12"
                            placeholder={r#"[
  {
    "name": "codex-a",
    "auth_json": { "refresh_token": "rt-a", "account_id": "acct-a" }
  },
  {
    "name": "codex-b",
    "tokens": { "refresh_token": "rt-b" }
  }
]"#}
                            class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono", "text-xs")}
                            value={(*batch_import_raw_json).clone()}
                            oninput={{
                                let batch_import_raw_json = batch_import_raw_json.clone();
                                let batch_import_feedback = batch_import_feedback.clone();
                                Callback::from(move |event: InputEvent| {
                                    if let Some(target) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                                        batch_import_raw_json.set(target.value());
                                        batch_import_feedback.set(None);
                                    }
                                })
                            }}
                        />
                        if let Some((message, is_error)) = (*batch_import_feedback).clone() {
                            <div class={classes!("font-mono", "text-[11px]", if is_error { "text-red-600 dark:text-red-300" } else { "text-emerald-600 dark:text-emerald-300" })}>
                                { message }
                            </div>
                        }
                        <div class={classes!("flex", "justify-end")}>
                            <button
                                class={classes!("btn-terminal", "btn-terminal-primary")}
                                onclick={on_import_account_batch}
                                disabled={*batch_importing}
                            >
                                { if *batch_importing { "创建导入作业中..." } else { "开始批量导入" } }
                            </button>
                        </div>
                    </div>
                    }

                    if !recent_import_jobs.is_empty() || active_import_job.is_some() {
                        <div class={classes!("mt-4", "grid", "gap-4", "xl:grid-cols-2")}>
                            <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                                <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                    <div>
                                        <h3 class={classes!("m-0", "text-sm", "font-semibold", "text-[var(--text)]")}>{ "最近导入作业" }</h3>
                                        <p class={classes!("mt-1", "mb-0", "text-xs", "text-[var(--muted)]")}>
                                            { format!("最多展示最近 {} 个作业。", ADMIN_CODEX_IMPORT_JOB_LIST_LIMIT) }
                                        </p>
                                    </div>
                                    if let Some(active_detail) = (*active_import_job).clone() {
                                        <span class={classes!("font-mono", "text-[11px]", codex_import_status_tone(&active_detail.summary.status))}>
                                            { format!("当前查看: {}", active_detail.summary.job_id) }
                                        </span>
                                    }
                                </div>
                                <div class={classes!("mt-3", "space-y-2")}>
                                    { for recent_import_jobs.iter().map(|job| {
                                        let job_id = job.job_id.clone();
                                        let is_selected = (*active_import_job)
                                            .as_ref()
                                            .map(|detail| detail.summary.job_id == job.job_id)
                                            .unwrap_or(false);
                                        let progress = format!(
                                            "{}/{} done · ok {} · skipped {} · failed {}",
                                            job.completed_count,
                                            job.total_count,
                                            job.succeeded_count,
                                            job.skipped_count,
                                            job.failed_count
                                        );
                                        html! {
                                            <button
                                                type="button"
                                                class={classes!(
                                                    "w-full", "rounded-lg", "border", "px-3", "py-2.5", "text-left",
                                                    if is_selected {
                                                        "border-sky-500/30 bg-sky-500/8"
                                                    } else {
                                                        "border-[var(--border)] bg-[var(--surface)]"
                                                    }
                                                )}
                                                onclick={{
                                                    let on_load_import_job = on_load_import_job.clone();
                                                    Callback::from(move |_| on_load_import_job.emit(job_id.clone()))
                                                }}
                                            >
                                                <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                                    <span class={classes!("font-mono", "text-xs", "font-semibold", "text-[var(--text)]")}>{ job.job_id.clone() }</span>
                                                    <span class={classes!("font-mono", "text-[11px]", codex_import_status_tone(&job.status))}>{ job.status.clone() }</span>
                                                </div>
                                                <div class={classes!("mt-1", "text-xs", "text-[var(--muted)]")}>
                                                    { progress }
                                                </div>
                                                <div class={classes!("mt-1", "font-mono", "text-[11px]", "text-[var(--muted)]")}>
                                                    { format!("{} · {}", job.source_type, format_ms(job.created_at_ms)) }
                                                </div>
                                            </button>
                                        }
                                    }) }
                                </div>
                            </div>
                            <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                                if let Some(job_detail) = (*active_import_job).clone() {
                                    <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                                        <div>
                                            <h3 class={classes!("m-0", "text-sm", "font-semibold", "text-[var(--text)]")}>{ "导入作业详情" }</h3>
                                            <p class={classes!("mt-1", "mb-0", "font-mono", "text-[11px]", "text-[var(--muted)]")}>
                                                { format!("{} · {} · validate={}", job_detail.summary.job_id, job_detail.summary.source_type, job_detail.summary.validate_before_import) }
                                            </p>
                                        </div>
                                        <span class={classes!("font-mono", "text-[11px]", codex_import_status_tone(&job_detail.summary.status))}>
                                            { job_detail.summary.status.clone() }
                                        </span>
                                    </div>
                                    <div class={classes!("mt-3", "grid", "gap-2", "sm:grid-cols-2", "xl:grid-cols-4")}>
                                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}>
                                            <div class={classes!("text-[11px]", "text-[var(--muted)]")}>{ "总数" }</div>
                                            <div class={classes!("mt-1", "font-mono", "text-sm", "font-semibold")}>{ job_detail.summary.total_count }</div>
                                        </div>
                                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}>
                                            <div class={classes!("text-[11px]", "text-[var(--muted)]")}>{ "成功" }</div>
                                            <div class={classes!("mt-1", "font-mono", "text-sm", "font-semibold", "text-emerald-600", "dark:text-emerald-300")}>{ job_detail.summary.succeeded_count }</div>
                                        </div>
                                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}>
                                            <div class={classes!("text-[11px]", "text-[var(--muted)]")}>{ "跳过" }</div>
                                            <div class={classes!("mt-1", "font-mono", "text-sm", "font-semibold")}>{ job_detail.summary.skipped_count }</div>
                                        </div>
                                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}>
                                            <div class={classes!("text-[11px]", "text-[var(--muted)]")}>{ "失败/冲突" }</div>
                                            <div class={classes!("mt-1", "font-mono", "text-sm", "font-semibold", "text-red-600", "dark:text-red-300")}>{ job_detail.summary.failed_count }</div>
                                        </div>
                                    </div>
                                    if let Some(batch_error_message) = job_detail.summary.batch_error_message.clone() {
                                        <div class={classes!("mt-3", "rounded-lg", "border", "border-red-500/30", "bg-red-500/5", "px-3", "py-2", "font-mono", "text-[11px]", "text-red-600", "dark:text-red-300")}>
                                            { batch_error_message }
                                        </div>
                                    }
                                    <div class={classes!("mt-3", "overflow-x-auto")}>
                                        <table class={classes!("min-w-full", "text-sm")}>
                                            <thead class={classes!("text-left", "text-[11px]", "uppercase", "tracking-wide", "text-[var(--muted)]")}>
                                                <tr>
                                                    <th class={classes!("px-2", "py-2")}>{ "#" }</th>
                                                    <th class={classes!("px-2", "py-2")}>{ "name" }</th>
                                                    <th class={classes!("px-2", "py-2")}>{ "status" }</th>
                                                    <th class={classes!("px-2", "py-2")}>{ "account" }</th>
                                                    <th class={classes!("px-2", "py-2")}>{ "result" }</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                { for job_detail.items.iter().map(|item| {
                                                    let account_line = item
                                                        .final_account_id
                                                        .clone()
                                                        .or_else(|| item.requested_account_id.clone())
                                                        .unwrap_or_else(|| "-".to_string());
                                                    let result_line = item
                                                        .imported_account_name
                                                        .clone()
                                                        .or_else(|| item.error_message.clone())
                                                        .unwrap_or_else(|| "-".to_string());
                                                    html! {
                                                        <tr class={classes!("border-t", "border-[var(--border)]", "align-top")}>
                                                            <td class={classes!("px-2", "py-2", "font-mono", "text-[11px]", "text-[var(--muted)]")}>{ item.item_index }</td>
                                                            <td class={classes!("px-2", "py-2")}>
                                                                <div class={classes!("font-mono", "text-xs", "text-[var(--text)]")}>{ item.requested_name.clone() }</div>
                                                                <div class={classes!("mt-1", "text-[11px]", "text-[var(--muted)]")}>
                                                                    { item.validated_at_ms.map(format_ms).unwrap_or_else(|| "-".to_string()) }
                                                                </div>
                                                            </td>
                                                            <td class={classes!("px-2", "py-2", "font-mono", "text-[11px]", codex_import_status_tone(&item.status))}>{ item.status.clone() }</td>
                                                            <td class={classes!("px-2", "py-2", "font-mono", "text-[11px]", "text-[var(--muted)]")}>{ account_line }</td>
                                                            <td class={classes!("px-2", "py-2", "font-mono", "text-[11px]", "text-[var(--muted)]")}>{ result_line }</td>
                                                        </tr>
                                                    }
                                                }) }
                                            </tbody>
                                        </table>
                                    </div>
                                } else {
                                    <div class={classes!("rounded-lg", "border", "border-dashed", "border-[var(--border)]", "px-4", "py-10", "text-center", "text-[var(--muted)]")}>
                                        { "选择一个导入作业后，这里会显示逐条处理结果。" }
                                    </div>
                                }
                            </div>
                        </div>
                    }

                    // Account search + sort + filter toolbar
                    <div class={classes!("mt-4", "space-y-4")}>
                        // Search bar
                        <div class={classes!("flex", "items-center", "gap-2")}>
                            <input
                                type="text"
                                class={classes!("flex-1", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm", "placeholder:text-[var(--muted)]", "focus:outline-none", "focus:ring-2", "focus:ring-[var(--primary)]/40")}
                                placeholder="搜索账号名称、状态、plan、ID、权重..."
                                value={(*account_search).clone()}
                                oninput={on_account_search_input.clone()}
                                onkeydown={on_account_search_keydown.clone()}
                            />
                            if !(*account_search).is_empty() {
                                <button
                                    type="button"
                                    class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm", "text-[var(--muted)]", "hover:text-[var(--text)]", "transition-colors")}
                                    onclick={on_account_search_clear.clone()}
                                >
                                    { "清除" }
                                </button>
                            }
                            <button
                                type="button"
                                class={classes!("rounded-lg", "bg-[var(--primary)]", "px-4", "py-2", "text-sm", "font-medium", "text-white", "hover:opacity-90", "transition-opacity")}
                                onclick={Callback::from({
                                    let on_account_search_submit = on_account_search_submit.clone();
                                    move |_| on_account_search_submit.emit(())
                                })}
                            >
                                { "搜索" }
                            </button>
                        </div>
                        // Sort & filter toolbar
                        <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                            <button
                                type="button"
                                class={classes!(
                                    "rounded-full", "px-3", "py-1.5", "text-xs", "font-semibold", "border", "transition-colors",
                                    if *account_show_unhealthy {
                                        "bg-red-500/15 text-red-700 dark:text-red-300 border-red-400/50"
                                    } else {
                                        "bg-[var(--surface)] text-[var(--muted)] border-[var(--border)] hover:text-[var(--text)]"
                                    }
                                )}
                                onclick={{
                                    let account_show_unhealthy = account_show_unhealthy.clone();
                                    let account_page = account_page.clone();
                                    Callback::from(move |_| {
                                        account_show_unhealthy.set(!*account_show_unhealthy);
                                        account_page.set(1);
                                    })
                                }}
                            >
                                { "异常" }
                            </button>
                            <button
                                type="button"
                                class={classes!(
                                    "rounded-full", "px-3", "py-1.5", "text-xs", "font-semibold", "border", "transition-colors",
                                    if *account_show_active_only {
                                        "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-400/50"
                                    } else {
                                        "bg-[var(--surface)] text-[var(--muted)] border-[var(--border)] hover:text-[var(--text)]"
                                    }
                                )}
                                onclick={{
                                    let account_show_active_only = account_show_active_only.clone();
                                    let account_page = account_page.clone();
                                    Callback::from(move |_| {
                                        account_show_active_only.set(!*account_show_active_only);
                                        account_page.set(1);
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
                                    if matches!(*account_sort_mode, AccountSortMode::PrimaryAsc | AccountSortMode::PrimaryDesc) {
                                        "bg-teal-500/15 text-teal-700 dark:text-teal-300 border-teal-400/50"
                                    } else {
                                        "bg-[var(--surface)] text-[var(--muted)] border-[var(--border)] hover:text-[var(--text)]"
                                    }
                                )}
                                onclick={{
                                    let account_sort_mode = account_sort_mode.clone();
                                    let account_page = account_page.clone();
                                    Callback::from(move |_| {
                                        let next = match *account_sort_mode {
                                            AccountSortMode::PrimaryAsc => AccountSortMode::PrimaryDesc,
                                            AccountSortMode::PrimaryDesc => AccountSortMode::None,
                                            _ => AccountSortMode::PrimaryAsc,
                                        };
                                        account_sort_mode.set(next);
                                        account_page.set(1);
                                    })
                                }}
                            >
                                { match *account_sort_mode {
                                    AccountSortMode::PrimaryAsc => "5h ↑",
                                    AccountSortMode::PrimaryDesc => "5h ↓",
                                    _ => "5h",
                                }}
                            </button>
                            <button
                                type="button"
                                class={classes!(
                                    "rounded-full", "px-3", "py-1.5", "text-xs", "font-semibold", "border", "transition-colors",
                                    if matches!(*account_sort_mode, AccountSortMode::SecondaryAsc | AccountSortMode::SecondaryDesc) {
                                        "bg-violet-500/15 text-violet-700 dark:text-violet-300 border-violet-400/50"
                                    } else {
                                        "bg-[var(--surface)] text-[var(--muted)] border-[var(--border)] hover:text-[var(--text)]"
                                    }
                                )}
                                onclick={{
                                    let account_sort_mode = account_sort_mode.clone();
                                    let account_page = account_page.clone();
                                    Callback::from(move |_| {
                                        let next = match *account_sort_mode {
                                            AccountSortMode::SecondaryAsc => AccountSortMode::SecondaryDesc,
                                            AccountSortMode::SecondaryDesc => AccountSortMode::None,
                                            _ => AccountSortMode::SecondaryAsc,
                                        };
                                        account_sort_mode.set(next);
                                        account_page.set(1);
                                    })
                                }}
                            >
                                { match *account_sort_mode {
                                    AccountSortMode::SecondaryAsc => "周限额 ↑",
                                    AccountSortMode::SecondaryDesc => "周限额 ↓",
                                    _ => "周限额",
                                }}
                            </button>
                        </div>
                        // Summary line
                        <div class={classes!("flex", "items-center", "justify-between", "text-xs", "text-[var(--muted)]")}>
                            <span>{ format!("总数 {} · 当前筛选 {} · 本页 {}", account_summary.total, *accounts_total, accounts.len()) }</span>
                            if account_total_pages > 1 {
                                <span>{ format!("第 {} / {} 页", account_current_page, account_total_pages) }</span>
                            }
                        </div>
                    </div>
                    // Account card grid
                    if account_page_entries.is_empty() {
                        <div class={classes!("mt-4", "rounded-lg", "border", "border-dashed", "border-[var(--border)]", "px-4", "py-10", "text-center", "text-sm", "text-[var(--muted)]")}>
                            { if accounts.is_empty() {
                                "当前还没有导入任何 Codex 账号。可以先导入账号，或者点击上方「刷新列表」确认后端是否已加载本地账号文件。"
                            } else {
                                "没有匹配的账号。尝试调整搜索条件或清除筛选。"
                            }}
                        </div>
                    } else {
                        <div class={classes!("mt-4", "grid", "gap-4", "sm:grid-cols-2")}>
                            { for account_page_entries.iter().enumerate().map(|(idx, acc)| {
                                let acc_name_for_toggle = acc.name.clone();
                                let acc_name_for_auto_refresh_toggle = acc.name.clone();
                                let acc_name_for_status_toggle = acc.name.clone();
                                let acc_name_for_delete = acc.name.clone();
                                let acc_name_for_auth_refresh = acc.name.clone();
                                let acc_name_for_usage_refresh = acc.name.clone();
                                let acc_name_for_reset_credit_consume = acc.name.clone();
                                let acc_name_for_models_probe = acc.name.clone();
                                let acc_name_for_proxy_change = acc.name.clone();
                                let acc_name_for_route_weight_tier_change = acc.name.clone();
                                let acc_name_for_settings_save = acc.name.clone();
                                let acc_name_for_request_max_change = acc.name.clone();
                                let acc_name_for_request_min_change = acc.name.clone();
                                let acc_name_for_image_enabled_change = acc.name.clone();
                                let acc_name_for_image_concurrency_change = acc.name.clone();
                                let acc_name = acc.name.clone();
                                let acc_status = acc.status.clone();
                                let account_disabled = acc_status == "disabled";
                                let toggled_account_status = if account_disabled {
                                    "active".to_string()
                                } else {
                                    "disabled".to_string()
                                };
                                let acc_plan_type = acc.plan_type.clone();
                                let acc_account_id = acc.account_id.clone();
                                let acc_email = acc.email.clone();
                                let spark_mapping_enabled = acc.map_gpt53_codex_to_spark;
                                let auto_refresh_enabled = acc.auto_refresh_enabled;
                                let selected_proxy_value = (*account_proxy_inputs)
                                    .get(&acc_name)
                                    .cloned()
                                    .unwrap_or_else(|| account_proxy_select_value(acc));
                                let selected_route_weight_tier = (*account_route_weight_tier_inputs)
                                    .get(&acc_name)
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        if acc.route_weight_tier.trim().is_empty() {
                                            "auto".to_string()
                                        } else {
                                            acc.route_weight_tier.clone()
                                        }
                                    });
                                let selected_request_max_value = (*account_request_max_inputs)
                                    .get(&acc_name)
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        acc.request_max_concurrency
                                            .map(|value| value.to_string())
                                            .unwrap_or_default()
                                    });
                                let selected_request_min_value = (*account_request_min_inputs)
                                    .get(&acc_name)
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        acc.request_min_start_interval_ms
                                            .map(|value| value.to_string())
                                            .unwrap_or_default()
                                    });
                                let selected_image_enabled = (*account_image_enabled_inputs)
                                    .get(&acc_name)
                                    .copied()
                                    .unwrap_or(acc.codex_image_generation_enabled);
                                let selected_image_concurrency_value =
                                    (*account_image_concurrency_inputs)
                                        .get(&acc_name)
                                        .cloned()
                                        .unwrap_or_else(|| {
                                            acc.codex_image_generation_max_concurrency.to_string()
                                        });
                                let configured_proxy_line = account_configured_proxy_label(acc);
                                let effective_proxy_line = format!(
                                    "effective: {} · {}",
                                    acc.effective_proxy_source,
                                    acc.effective_proxy_url.clone().unwrap_or_else(|| "direct".to_string())
                                );
                                let scheduler_line = format!(
                                    "scheduler: concurrency {} · start interval {}",
                                    acc.request_max_concurrency
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "∞".to_string()),
                                    acc.request_min_start_interval_ms
                                        .map(|value| format!("{} ms", value))
                                        .unwrap_or_else(|| "∞".to_string())
                                );
                                let image_rate_limit_bucket = account_image_rate_limit_bucket(
                                    (*codex_rate_limit_status).as_ref(),
                                    &acc_name,
                                );
                                let image_quota_line = if acc.codex_image_generation_enabled {
                                    if let Some(bucket) = image_rate_limit_bucket {
                                        format!(
                                            "image on · concurrency {} · bucket {}",
                                            acc.codex_image_generation_max_concurrency,
                                            bucket.display_name
                                        )
                                    } else {
                                        format!(
                                            "image on · concurrency {} · common Codex quota",
                                            acc.codex_image_generation_max_concurrency
                                        )
                                    }
                                } else {
                                    "image off · common Codex quota".to_string()
                                };
                                let last_refresh_line = acc
                                    .last_refresh
                                    .map(format_ms)
                                    .unwrap_or_else(|| "-".to_string());
                                let access_token_expiry_line = format_access_token_expiry(
                                    Date::now() as i64,
                                    acc.access_token_expires_at,
                                );
                                let last_usage_checked_line = acc
                                    .last_usage_checked_at
                                    .map(format_ms)
                                    .unwrap_or_else(|| "-".to_string());
                                let last_usage_success_line = acc
                                    .last_usage_success_at
                                    .map(format_ms)
                                    .unwrap_or_else(|| "-".to_string());
                                let on_delete = on_delete_account.clone();
                                let on_probe_account_models = on_probe_account_models.clone();
                                let on_refresh_account_auth = on_refresh_account_auth.clone();
                                let on_refresh_account_usage = on_refresh_account_usage.clone();
                                let on_consume_account_reset_credit =
                                    on_consume_account_reset_credit.clone();
                                let on_toggle_account_status = on_toggle_account_status.clone();
                                let on_toggle_account_spark_mapping =
                                    on_toggle_account_spark_mapping.clone();
                                let on_toggle_account_auto_refresh =
                                    on_toggle_account_auto_refresh.clone();
                                let on_save_account_settings = on_save_account_settings.clone();
                                let rate_limit_bucket = account_rate_limit_bucket(
                                    (*codex_rate_limit_status).as_ref(),
                                    &acc_name,
                                );
                                let primary_window =
                                    rate_limit_bucket.and_then(|bucket| bucket.primary.as_ref());
                                let secondary_window =
                                    rate_limit_bucket.and_then(|bucket| bucket.secondary.as_ref());
                                let primary_pct = account_limit_percent_label(
                                    primary_window,
                                    acc.primary_remaining_percent,
                                );
                                let secondary_pct = account_limit_percent_label(
                                    secondary_window,
                                    acc.secondary_remaining_percent,
                                );
                                let primary_width = account_limit_width(
                                    primary_window,
                                    acc.primary_remaining_percent,
                                );
                                let secondary_width = account_limit_width(
                                    secondary_window,
                                    acc.secondary_remaining_percent,
                                );
                                let primary_used_label = account_limit_used_label(primary_window);
                                let secondary_used_label =
                                    account_limit_used_label(secondary_window);
                                let primary_reset_label =
                                    account_limit_reset_label(primary_window);
                                let secondary_reset_label =
                                    account_limit_reset_label(secondary_window);
                                let reset_credits_label = acc
                                    .rate_limit_reset_credits_available
                                    .map(|value| value.to_string())
                                    .unwrap_or_else(|| "-".to_string());
                                let reset_credit_available =
                                    acc.rate_limit_reset_credits_available.unwrap_or(0) > 0;
                                let is_pro = is_gpt_pro_account(acc_plan_type.as_deref());
                                let show_spark_toggle = is_pro || spark_mapping_enabled;
                                let account_busy =
                                    (*account_action_inflight).contains(&acc_name);
                                let accent = ACCOUNT_ACCENT_BORDERS[idx % ACCOUNT_ACCENT_BORDERS.len()];
                                html! {
                                    <div class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "overflow-hidden", "transition-all", "duration-200", "hover:shadow-lg", "hover:shadow-black/5", accent)}>
                                        // Card header
                                        <div class={classes!("p-5", "pb-3")}>
                                            <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                                <span class={classes!(
                                                    "inline-flex", "items-center", "gap-1.5", "shrink-0",
                                                    "rounded-full", "px-2", "py-0.5",
                                                    "font-mono", "text-[10px]", "font-semibold", "uppercase", "tracking-wider",
                                                    "bg-[var(--surface-alt)]",
                                                    match acc_status.as_str() {
                                                        "active" | "ready" => "text-emerald-600",
                                                        "disabled" => "text-red-600",
                                                        _ => "text-[var(--muted)]",
                                                    }
                                                )}>
                                                    <span class={classes!(
                                                        "inline-block", "h-1.5", "w-1.5", "rounded-full",
                                                        match acc_status.as_str() {
                                                            "active" | "ready" => "bg-emerald-500",
                                                            "disabled" => "bg-red-500",
                                                            _ => "bg-slate-400",
                                                        }
                                                    )} />
                                                    { acc_status.clone() }
                                                </span>
                                                <span class={classes!("font-bold", "text-sm", "break-all")}>{ acc_name.clone() }</span>
                                                if let Some(ref plan_type) = acc_plan_type {
                                                    <span class={classes!("rounded-full", "bg-[var(--surface-alt)]", "px-2.5", "py-0.5", "shrink-0", "font-mono", "text-[10px]", "font-medium", "text-[var(--muted)]")}>
                                                        { plan_type.clone() }
                                                    </span>
                                                }
                                            </div>
                                            if acc_status != "disabled" {
                                                <div class={classes!("mt-3", "grid", "gap-2.5", "sm:grid-cols-2")}>
                                                    { render_account_limit_tile(
                                                        "5H",
                                                        &primary_pct,
                                                        &primary_used_label,
                                                        &primary_reset_label,
                                                        primary_width,
                                                        "bg-[linear-gradient(90deg,#0f766e,#14b8a6)]",
                                                    ) }
                                                    { render_account_limit_tile(
                                                        "WEEK",
                                                        &secondary_pct,
                                                        &secondary_used_label,
                                                        &secondary_reset_label,
                                                        secondary_width,
                                                        "bg-[linear-gradient(90deg,#2563eb,#7c3aed)]",
                                                    ) }
                                                </div>
                                            }
                                            // Info section
                                            <div class={classes!("mt-2", "space-y-0.5", "text-xs", "font-mono", "text-[var(--muted)]")}>
                                                if let Some(ref aid) = acc_account_id {
                                                    <div class={classes!("break-all")}>{ format!("id: {}", aid) }</div>
                                                }
                                                if let Some(email) = acc_email.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
                                                    <div class={classes!("break-all")}>{ format!("email: {}", email) }</div>
                                                }
                                                <div>{ configured_proxy_line.clone() }</div>
                                                <div>
                                                    { effective_proxy_line.clone() }
                                                    if let Some(proxy_name) = acc.effective_proxy_config_name.as_deref() {
                                                        { format!(" · {}", proxy_name) }
                                                    }
                                                </div>
                                                <div>{ scheduler_line.clone() }</div>
                                                <div>{ image_quota_line.clone() }</div>
                                                <div>{ format!("route weight tier: {}", acc.route_weight_tier) }</div>
                                                <div>{ format!("reset credits available: {}", reset_credits_label) }</div>
                                                <div class={classes!("flex", "gap-3", "flex-wrap")}>
                                                    <span>{ if auto_refresh_enabled { "auto refresh on" } else { "auto refresh off" } }</span>
                                                    <span>{ format!("token refresh {}", last_refresh_line) }</span>
                                                    <span>{ access_token_expiry_line.clone() }</span>
                                                </div>
                                                <div class={classes!("flex", "gap-3", "flex-wrap")}>
                                                    <span>{ format!("usage checked {}", last_usage_checked_line) }</span>
                                                    <span>{ format!("usage success {}", last_usage_success_line) }</span>
                                                </div>
                                            </div>
                                            if let Some(auth_error) = acc.auth_refresh_error_message.as_deref() {
                                                <div class={classes!("mt-2", "text-xs", "leading-5", "text-amber-700", "dark:text-amber-300", "break-all")}>
                                                    { format!("auth refresh error: {}", auth_error) }
                                                </div>
                                            }
                                            if let Some(usage_error) = acc.usage_error_message.as_deref() {
                                                <div class={classes!("mt-2", "text-xs", "leading-5", "text-amber-700", "dark:text-amber-300", "break-all")}>
                                                    { format!("usage refresh error: {}", usage_error) }
                                                </div>
                                            }
                                        </div>
                                        // Controls section
                                        <div class={classes!("border-t", "border-[var(--border)]", "px-5", "py-3")}>
                                            <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                                <input
                                                    type="number"
                                                    class={classes!("w-24", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2", "py-1.5", "text-xs")}
                                                    placeholder="并发"
                                                    value={selected_request_max_value.clone()}
                                                    oninput={{
                                                        let account_request_max_inputs = account_request_max_inputs.clone();
                                                        Callback::from(move |event: InputEvent| {
                                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                                let mut next = (*account_request_max_inputs).clone();
                                                                next.insert(acc_name_for_request_max_change.clone(), target.value());
                                                                account_request_max_inputs.set(next);
                                                            }
                                                        })
                                                    }}
                                                />
                                                <input
                                                    type="number"
                                                    class={classes!("w-28", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2", "py-1.5", "text-xs")}
                                                    placeholder="间隔 ms"
                                                    value={selected_request_min_value.clone()}
                                                    oninput={{
                                                        let account_request_min_inputs = account_request_min_inputs.clone();
                                                        Callback::from(move |event: InputEvent| {
                                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                                let mut next = (*account_request_min_inputs).clone();
                                                                next.insert(acc_name_for_request_min_change.clone(), target.value());
                                                                account_request_min_inputs.set(next);
                                                            }
                                                        })
                                                    }}
                                                />
                                                <select
                                                    class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2", "py-1.5", "text-xs")}
                                                    value={selected_proxy_value.clone()}
                                                    onchange={{
                                                        let account_proxy_inputs = account_proxy_inputs.clone();
                                                        Callback::from(move |event: Event| {
                                                            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                                                let mut next = (*account_proxy_inputs).clone();
                                                                next.insert(acc_name_for_proxy_change.clone(), target.value());
                                                                account_proxy_inputs.set(next);
                                                            }
                                                        })
                                                    }}
                                                >
                                                    <option value="inherit" selected={selected_proxy_value == "inherit"}>{ "继承 Proxy" }</option>
                                                    <option value="direct" selected={selected_proxy_value == "direct"}>{ "Direct" }</option>
                                                    { for proxy_configs.iter().map(|proxy_config| {
                                                        let option_value = format!("fixed:{}", proxy_config.id);
                                                        html! {
                                                            <option value={option_value.clone()} selected={selected_proxy_value == option_value}>
                                                                { format!("{} · {}", proxy_config.name, proxy_config.proxy_url) }
                                                            </option>
                                                        }
                                                    }) }
                                                </select>
                                                <select
                                                    class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2", "py-1.5", "text-xs")}
                                                    value={selected_route_weight_tier.clone()}
                                                    onchange={{
                                                        let account_route_weight_tier_inputs = account_route_weight_tier_inputs.clone();
                                                        Callback::from(move |event: Event| {
                                                            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                                                let mut next = (*account_route_weight_tier_inputs).clone();
                                                                next.insert(acc_name_for_route_weight_tier_change.clone(), target.value());
                                                                account_route_weight_tier_inputs.set(next);
                                                            }
                                                        })
                                                    }}
                                                >
                                                    <option value="auto" selected={selected_route_weight_tier == "auto"}>{ "Auto" }</option>
                                                    <option value="free" selected={selected_route_weight_tier == "free"}>{ "Free" }</option>
                                                    <option value="plus" selected={selected_route_weight_tier == "plus"}>{ "Plus" }</option>
                                                    <option value="pro5x" selected={selected_route_weight_tier == "pro5x"}>{ "Pro5x" }</option>
                                                    <option value="pro20x" selected={selected_route_weight_tier == "pro20x"}>{ "Pro20x" }</option>
                                                </select>
                                                <label class={classes!("inline-flex", "items-center", "gap-1.5", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2", "py-1.5", "text-xs")}>
                                                    <input
                                                        type="checkbox"
                                                        checked={selected_image_enabled}
                                                        onchange={{
                                                            let account_image_enabled_inputs = account_image_enabled_inputs.clone();
                                                            Callback::from(move |event: Event| {
                                                                if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                                    let mut next = (*account_image_enabled_inputs).clone();
                                                                    next.insert(acc_name_for_image_enabled_change.clone(), target.checked());
                                                                    account_image_enabled_inputs.set(next);
                                                                }
                                                            })
                                                        }}
                                                    />
                                                    <span>{ "生图" }</span>
                                                </label>
                                                <input
                                                    type="number"
                                                    min="1"
                                                    max={CODEX_IMAGE_MAX_CONCURRENCY.to_string()}
                                                    class={classes!("w-24", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2", "py-1.5", "text-xs")}
                                                    placeholder="生图并发"
                                                    value={selected_image_concurrency_value.clone()}
                                                    oninput={{
                                                        let account_image_concurrency_inputs = account_image_concurrency_inputs.clone();
                                                        Callback::from(move |event: InputEvent| {
                                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                                let mut next = (*account_image_concurrency_inputs).clone();
                                                                next.insert(acc_name_for_image_concurrency_change.clone(), target.value());
                                                                account_image_concurrency_inputs.set(next);
                                                            }
                                                        })
                                                    }}
                                                />
                                                <button
                                                    class={classes!("btn-terminal")}
                                                    onclick={Callback::from(move |_| on_save_account_settings.emit(acc_name_for_settings_save.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "保存" } }
                                                </button>
                                            </div>
                                            <div class={classes!("mt-2", "flex", "items-center", "gap-2", "flex-wrap")}>
                                                <button
                                                    class={classes!("btn-terminal")}
                                                    onclick={Callback::from(move |_| on_refresh_account_auth.emit(acc_name_for_auth_refresh.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "刷新 Token" } }
                                                </button>
                                                <button
                                                    class={classes!("btn-terminal")}
                                                    onclick={Callback::from(move |_| on_refresh_account_usage.emit(acc_name_for_usage_refresh.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "刷新 Usage" } }
                                                </button>
                                                <button
                                                    class={classes!(
                                                        "btn-terminal",
                                                        if reset_credit_available { "btn-terminal-primary" } else { "" }
                                                    )}
                                                    onclick={Callback::from(move |_| on_consume_account_reset_credit.emit(acc_name_for_reset_credit_consume.clone()))}
                                                    disabled={account_busy || account_disabled || !reset_credit_available}
                                                    title="使用一个 Codex usage limit reset credit"
                                                >
                                                    { if account_busy { "..." } else if reset_credit_available { "重置限额" } else { "无 Reset" } }
                                                </button>
                                                <button
                                                    class={classes!("btn-terminal")}
                                                    onclick={Callback::from(move |_| on_probe_account_models.emit(acc_name_for_models_probe.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "测试 Models" } }
                                                </button>
                                                <button
                                                    class={classes!(
                                                        "btn-terminal",
                                                        if auto_refresh_enabled { "btn-terminal-primary" } else { "" }
                                                    )}
                                                    onclick={Callback::from(move |_| {
                                                        on_toggle_account_auto_refresh.emit((
                                                            acc_name_for_auto_refresh_toggle.clone(),
                                                            !auto_refresh_enabled,
                                                        ))
                                                    })}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else if auto_refresh_enabled { "Auto ✓" } else { "Auto ✗" } }
                                                </button>
                                                <button
                                                    class={classes!("btn-terminal")}
                                                    onclick={Callback::from(move |_| {
                                                        on_toggle_account_status.emit((
                                                            acc_name_for_status_toggle.clone(),
                                                            toggled_account_status.clone(),
                                                        ))
                                                    })}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else if account_disabled { "启用" } else { "禁用" } }
                                                </button>
                                                if show_spark_toggle {
                                                    <button
                                                        class={classes!(
                                                            "btn-terminal",
                                                            if spark_mapping_enabled { "btn-terminal-primary" } else { "" }
                                                        )}
                                                        onclick={Callback::from(move |_| {
                                                            on_toggle_account_spark_mapping.emit((
                                                                acc_name_for_toggle.clone(),
                                                                !spark_mapping_enabled,
                                                            ))
                                                        })}
                                                        disabled={account_busy}
                                                        title="把客户端请求的 gpt-5.3-codex 映射到该账号上游的 gpt-5.3-codex-spark"
                                                    >
                                                        { if account_busy { "..." } else if spark_mapping_enabled { "Spark ✓" } else { "Spark" } }
                                                    </button>
                                                }
                                                <button
                                                    class={classes!("btn-terminal", "btn-terminal-danger")}
                                                    onclick={Callback::from(move |_| on_delete.emit(acc_name_for_delete.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "删除" } }
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }) }
                        </div>
                        <div class={classes!("mt-4")}>
                            <Pagination
                                current_page={account_current_page}
                                total_pages={account_total_pages}
                                on_page_change={on_account_page_change.clone()}
                            />
                        </div>
                    }
                </section>
                } // end TAB_ACCOUNTS


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
    fn llm_inventory_load_helpers_follow_active_tab() {
        assert!(should_load_llm_gateway_import_jobs(TAB_ACCOUNTS));
        assert!(!should_load_llm_gateway_import_jobs(TAB_OVERVIEW));
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

    #[test]
    fn parse_admin_codex_batch_import_json_accepts_local_json_array() {
        let items = parse_admin_codex_batch_import_json(
            r#"[
                {
                    "name": "codex-a",
                    "auth_json": { "refresh_token": "rt-a", "account_id": "acct-a" }
                },
                {
                    "name": "codex-b",
                    "tokens": { "refresh_token": "rt-b" }
                }
            ]"#,
        )
        .expect("valid local batch import json");

        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["name"], "codex-a");
        assert!(items[0]["auth_json"].is_object());
        assert!(items[1]["tokens"].is_object());
    }

    #[test]
    fn parse_admin_codex_batch_import_json_rejects_missing_name() {
        let err = parse_admin_codex_batch_import_json(
            r#"[
                {
                    "auth_json": { "refresh_token": "rt-a" }
                }
            ]"#,
        )
        .expect_err("missing name must fail");

        assert!(err.contains("name"));
    }
}
