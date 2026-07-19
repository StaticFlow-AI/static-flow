//! LLM gateway accounts page (`/admin/llm-gateway/accounts`).
//!
//! Owns the server-paginated Codex account inventory (search / sort /
//! unhealthy / active filters), per-account scheduler + proxy + image
//! settings, auth/usage refresh actions, single and batch account import
//! with import-job polling, and the codex rate-limit status strip. Extracted
//! from the mega llm gateway panel.

use std::collections::{BTreeMap, HashSet};

use gloo_timers::callback::{Interval, Timeout};
use js_sys::Date;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_llm_gateway::{admin_group_total_pages, format_optional_duration_ms};
use crate::{
    api::{
        consume_admin_llm_gateway_account_rate_limit_reset_credit,
        create_admin_llm_gateway_account_import_job, delete_admin_llm_gateway_account,
        fetch_admin_llm_gateway_account_import_job, fetch_admin_llm_gateway_account_import_jobs,
        fetch_admin_llm_gateway_account_rate_limit_reset_credits,
        fetch_admin_llm_gateway_accounts_page_with_query, fetch_admin_llm_gateway_proxy_configs,
        fetch_llm_gateway_status, import_admin_llm_gateway_account,
        patch_admin_llm_gateway_account, probe_admin_llm_gateway_account_models,
        refresh_admin_llm_gateway_account_auth, refresh_admin_llm_gateway_account_usage,
        AccountSummaryView, AdminAccountsSummaryView, AdminLlmGatewayAccountPageQuery,
        AdminUpstreamProxyConfigView, CodexAccountImportJobDetailView,
        CodexAccountImportJobSummaryView, CodexRateLimitResetCreditsDetails,
        ConsumeCodexRateLimitResetCreditRequest, LlmGatewayRateLimitBucketView,
        LlmGatewayRateLimitStatusResponse, LlmGatewayRateLimitWindowView,
        PatchAdminLlmGatewayAccountInput,
    },
    components::{modal::Modal, pagination::Pagination},
    pages::llm_access_shared::{confirm_destructive, format_ms, format_percent, format_reset_hint},
    router::Route,
};

const ACCOUNT_PAGE_SIZE: usize = 8;
const ADMIN_CODEX_IMPORT_JOB_LIST_LIMIT: usize = 10;

const CODEX_IMAGE_DEFAULT_CONCURRENCY: u64 = 3;

const CODEX_IMAGE_MAX_CONCURRENCY: u64 = 1024;

#[derive(Clone, PartialEq, Eq)]
struct ResetCreditPickerState {
    account_name: String,
    details: CodexRateLimitResetCreditsDetails,
    selected_credit_id: String,
    idempotency_key: String,
}

fn selected_reset_credit_id(
    details: &CodexRateLimitResetCreditsDetails,
    selected_credit_id: &str,
) -> Result<Option<String>, &'static str> {
    if details.available_count <= 0 {
        return Err("当前没有可用 reset credit");
    }
    if details.credits.is_empty() {
        return Ok(None);
    }
    details
        .credits
        .iter()
        .find(|credit| {
            credit.id == selected_credit_id && credit.status.eq_ignore_ascii_case("available")
        })
        .map(|credit| Some(credit.id.clone()))
        .ok_or("请先选择一个 reset credit")
}

fn new_reset_credit_idempotency_key() -> Result<String, String> {
    let window = web_sys::window().ok_or_else(|| "浏览器 window 不可用".to_string())?;
    let crypto = window
        .crypto()
        .map_err(|_| "浏览器安全随机数不可用".to_string())?;
    Ok(crypto.random_uuid())
}

const ACCOUNT_ACCENT_BORDERS: &[&str] = &[
    "border-l-4 border-l-teal-500/70",
    "border-l-4 border-l-violet-500/70",
    "border-l-4 border-l-amber-500/70",
    "border-l-4 border-l-sky-500/70",
    "border-l-4 border-l-rose-500/70",
];

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

fn is_gpt_pro_account(plan_type: Option<&str>) -> bool {
    plan_type.map(str::trim).is_some_and(|plan| {
        let normalized = plan.to_ascii_lowercase();
        normalized == "pro" || normalized == "gpt pro"
    })
}

#[function_component(AdminLlmGatewayAccountsPage)]
pub fn admin_llm_gateway_accounts_page() -> Html {
    let accounts = use_state(Vec::<AccountSummaryView>::new);
    let accounts_summary = use_state(AdminAccountsSummaryView::default);
    let codex_rate_limit_status = use_state(|| None::<LlmGatewayRateLimitStatusResponse>);
    let proxy_configs = use_state(Vec::<AdminUpstreamProxyConfigView>::new);
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
    let account_request_rpm_inputs = use_state(BTreeMap::<String, String>::new);
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
    let loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0_u32);
    let toast = use_state(|| None::<(String, bool)>);
    let reset_credit_picker = use_state(|| None::<ResetCreditPickerState>);
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

    // The per-account proxy dropdown lists the shared proxy-config slots;
    // account paging never re-fetches them.
    {
        let proxy_configs = proxy_configs.clone();
        let load_error = load_error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let proxy_configs = proxy_configs.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_llm_gateway_proxy_configs().await {
                    Ok(resp) => proxy_configs.set(resp.proxy_configs),
                    Err(err) => load_error.set(Some(err)),
                }
            });
            || ()
        });
    }

    // Server-paginated account inventory plus the codex rate-limit status and
    // the recent import jobs; every filter change re-fetches all three.
    {
        let accounts = accounts.clone();
        let accounts_summary = accounts_summary.clone();
        let codex_rate_limit_status = codex_rate_limit_status.clone();
        let recent_import_jobs = recent_import_jobs.clone();
        let account_proxy_inputs = account_proxy_inputs.clone();
        let account_route_weight_tier_inputs = account_route_weight_tier_inputs.clone();
        let account_request_max_inputs = account_request_max_inputs.clone();
        let account_request_rpm_inputs = account_request_rpm_inputs.clone();
        let account_request_min_inputs = account_request_min_inputs.clone();
        let account_image_enabled_inputs = account_image_enabled_inputs.clone();
        let account_image_concurrency_inputs = account_image_concurrency_inputs.clone();
        let accounts_total = accounts_total.clone();
        let account_page_limit = account_page_limit.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        use_effect_with(
            (
                *account_page,
                (*account_active_query).clone(),
                *account_sort_mode,
                *account_show_unhealthy,
                *account_show_active_only,
                *refresh_tick,
            ),
            move |(
                requested_page,
                active_query,
                sort_mode,
                show_unhealthy,
                show_active_only,
                _,
            )| {
                let requested_page = (*requested_page).max(1);
                let account_query = AdminLlmGatewayAccountPageQuery {
                    q: Some(active_query.clone()),
                    active_only: *show_active_only,
                    unhealthy_only: *show_unhealthy,
                    sort: Some(
                        match sort_mode {
                            AccountSortMode::PrimaryAsc => "primary_asc",
                            AccountSortMode::PrimaryDesc => "primary_desc",
                            AccountSortMode::SecondaryAsc => "secondary_asc",
                            AccountSortMode::SecondaryDesc => "secondary_desc",
                            AccountSortMode::None => "",
                        }
                        .to_string(),
                    ),
                };
                let accounts = accounts.clone();
                let accounts_summary = accounts_summary.clone();
                let codex_rate_limit_status = codex_rate_limit_status.clone();
                let recent_import_jobs = recent_import_jobs.clone();
                let account_proxy_inputs = account_proxy_inputs.clone();
                let account_route_weight_tier_inputs = account_route_weight_tier_inputs.clone();
                let account_request_max_inputs = account_request_max_inputs.clone();
                let account_request_rpm_inputs = account_request_rpm_inputs.clone();
                let account_request_min_inputs = account_request_min_inputs.clone();
                let account_image_enabled_inputs = account_image_enabled_inputs.clone();
                let account_image_concurrency_inputs = account_image_concurrency_inputs.clone();
                let accounts_total = accounts_total.clone();
                let account_page_limit = account_page_limit.clone();
                let loading = loading.clone();
                let load_error = load_error.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    loading.set(true);
                    let result = async {
                        let limit = ACCOUNT_PAGE_SIZE.max(1);
                        let offset = requested_page.saturating_sub(1) * limit;
                        let (accounts_result, codex_status_result, import_jobs_result) = futures::join!(
                            fetch_admin_llm_gateway_accounts_page_with_query(
                                limit,
                                offset,
                                &account_query,
                            ),
                            fetch_llm_gateway_status(),
                            fetch_admin_llm_gateway_account_import_jobs(Some(
                                ADMIN_CODEX_IMPORT_JOB_LIST_LIMIT,
                            )),
                        );
                        Ok::<_, String>((accounts_result?, codex_status_result?, import_jobs_result?))
                    }
                    .await;
                    match result {
                        Ok((accounts_resp, codex_status_resp, import_jobs)) => {
                            accounts_summary.set(accounts_resp.summary);
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
                            let next_request_rpm_inputs = accounts_resp
                                .accounts
                                .iter()
                                .map(|account| {
                                    (account.name.clone(), account.request_rpm_limit.to_string())
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
                            account_request_rpm_inputs.set(next_request_rpm_inputs);
                            account_request_min_inputs.set(next_request_min_inputs);
                            account_image_enabled_inputs.set(next_image_enabled_inputs);
                            account_image_concurrency_inputs.set(next_image_concurrency_inputs);
                            codex_rate_limit_status.set(Some(codex_status_resp));
                            recent_import_jobs.set(import_jobs);
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

    {
        let active_import_job = active_import_job.clone();
        let recent_import_jobs = recent_import_jobs.clone();
        let on_reload = on_reload.clone();
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
                    let on_reload = on_reload.clone();
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
                                    on_reload.emit(());
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
                        request_rpm_limit: None,
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
                        request_rpm_limit: None,
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
                        request_rpm_limit: None,
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
        let account_request_rpm_inputs = account_request_rpm_inputs.clone();
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
            let account_request_rpm_inputs = account_request_rpm_inputs.clone();
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
                let request_rpm_raw = (*account_request_rpm_inputs)
                    .get(&account_name)
                    .cloned()
                    .or_else(|| {
                        current_account
                            .as_ref()
                            .map(|account| account.request_rpm_limit.to_string())
                    })
                    .unwrap_or_else(|| "20".to_string());
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
                let request_rpm_limit = match request_rpm_raw.trim().parse::<u64>() {
                    Ok(value) if value > 0 => value,
                    _ => {
                        load_error.set(Some("账号 RPM 必须是大于 0 的整数".to_string()));
                        return;
                    },
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
                        request_rpm_limit: Some(request_rpm_limit),
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
                        let mut next_request_rpm_inputs = (*account_request_rpm_inputs).clone();
                        next_request_rpm_inputs
                            .insert(updated.name.clone(), updated.request_rpm_limit.to_string());
                        account_request_rpm_inputs.set(next_request_rpm_inputs);
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

    let on_open_account_reset_credit = {
        let account_action_inflight = account_action_inflight.clone();
        let reset_credit_picker = reset_credit_picker.clone();
        let flash = flash.clone();
        let load_error = load_error.clone();
        Callback::from(move |account_name: String| {
            let account_action_inflight = account_action_inflight.clone();
            let reset_credit_picker = reset_credit_picker.clone();
            let flash = flash.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                let result = match new_reset_credit_idempotency_key() {
                    Ok(idempotency_key) => {
                        fetch_admin_llm_gateway_account_rate_limit_reset_credits(&account_name)
                            .await
                            .map(|details| ResetCreditPickerState {
                                account_name: account_name.clone(),
                                details,
                                selected_credit_id: String::new(),
                                idempotency_key,
                            })
                    },
                    Err(error) => Err(error),
                };
                match result {
                    Ok(picker) => {
                        load_error.set(None);
                        reset_credit_picker.set(Some(picker));
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((
                            format!("加载账号 `{}` 的 reset credits 失败\n{err}", account_name),
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

    let on_confirm_account_reset_credit = {
        let account_action_inflight = account_action_inflight.clone();
        let account_proxy_inputs = account_proxy_inputs.clone();
        let accounts = accounts.clone();
        let codex_rate_limit_status = codex_rate_limit_status.clone();
        let reset_credit_picker = reset_credit_picker.clone();
        let flash = flash.clone();
        let load_error = load_error.clone();
        Callback::from(move |_: ()| {
            let Some(picker) = (*reset_credit_picker).clone() else {
                return;
            };
            let credit_id =
                match selected_reset_credit_id(&picker.details, &picker.selected_credit_id) {
                    Ok(credit_id) => credit_id,
                    Err(message) => {
                        flash.emit((message.to_string(), true));
                        return;
                    },
                };
            let request = ConsumeCodexRateLimitResetCreditRequest {
                idempotency_key: picker.idempotency_key.clone(),
                credit_id,
            };
            let account_name = picker.account_name.clone();
            let account_action_inflight = account_action_inflight.clone();
            let account_proxy_inputs = account_proxy_inputs.clone();
            let accounts = accounts.clone();
            let codex_rate_limit_status = codex_rate_limit_status.clone();
            let reset_credit_picker = reset_credit_picker.clone();
            let flash = flash.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_action_inflight).clone();
                inflight.insert(account_name.clone());
                account_action_inflight.set(inflight);

                match consume_admin_llm_gateway_account_rate_limit_reset_credit(
                    &account_name,
                    &request,
                )
                .await
                {
                    Ok(result) => {
                        let code = result.code.clone();
                        let windows_reset = result.windows_reset;
                        let replayed = result.replayed;
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
                        reset_credit_picker.set(None);
                        let mut message = match code.as_str() {
                            "reset" => format!(
                                "已使用账号 `{}` 的 reset credit，重置 {} 个窗口",
                                updated.name, windows_reset
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
                        if replayed {
                            message.push_str("（幂等重放，未再次调用上游）");
                        }
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

    let on_cancel_account_reset_credit = {
        let account_action_inflight = account_action_inflight.clone();
        let reset_credit_picker = reset_credit_picker.clone();
        Callback::from(move |_: ()| {
            let busy = (*reset_credit_picker)
                .as_ref()
                .is_some_and(|picker| account_action_inflight.contains(&picker.account_name));
            if !busy {
                reset_credit_picker.set(None);
            }
        })
    };

    let on_select_account_reset_credit = {
        let reset_credit_picker = reset_credit_picker.clone();
        Callback::from(move |selected_credit_id: String| {
            if let Some(mut picker) = (*reset_credit_picker).clone() {
                picker.selected_credit_id = selected_credit_id;
                reset_credit_picker.set(Some(picker));
            }
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
        let on_reload = on_reload.clone();
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
            let on_reload = on_reload.clone();
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
                        on_reload.emit(());
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
        let on_reload = on_reload.clone();
        let load_error = load_error.clone();
        Callback::from(move |name: String| {
            if !confirm_destructive(&format!("确认删除账号 {} ？", name)) {
                return;
            }
            let on_reload = on_reload.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match delete_admin_llm_gateway_account(&name).await {
                    Ok(_) => on_reload.emit(()),
                    Err(err) => load_error.set(Some(err)),
                }
            });
        })
    };

    let account_summary = *accounts_summary;
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
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "LLM Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Accounts" }</h1>
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
                            class={classes!("ghost")}
                            onclick={{
                                let on_reload = on_reload.clone();
                                Callback::from(move |_| on_reload.emit(()))
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
                            class={classes!("ghost")}
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
                            class={classes!("ghost")}
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
                            <button class={classes!("primary")} onclick={on_import_account} disabled={*importing}>
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
                                    type="checkbox" class={classes!("min-h-0", "w-auto")}
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
                                class={classes!("primary")}
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
                                let acc_name_for_reset_credit_open = acc.name.clone();
                                let acc_name_for_models_probe = acc.name.clone();
                                let acc_name_for_proxy_change = acc.name.clone();
                                let acc_name_for_route_weight_tier_change = acc.name.clone();
                                let acc_name_for_settings_save = acc.name.clone();
                                let acc_name_for_request_max_change = acc.name.clone();
                                let acc_name_for_request_rpm_change = acc.name.clone();
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
                                let selected_request_rpm_value = (*account_request_rpm_inputs)
                                    .get(&acc_name)
                                    .cloned()
                                    .unwrap_or_else(|| acc.request_rpm_limit.to_string());
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
                                    "scheduler: concurrency {} · RPM {} · start interval {}",
                                    acc.request_max_concurrency
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "∞".to_string()),
                                    acc.request_rpm_limit,
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
                                let on_open_account_reset_credit =
                                    on_open_account_reset_credit.clone();
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
                                                    <span>{ if auto_refresh_enabled { "Token 自动刷新：已开启" } else { "Token 自动刷新：已关闭" } }</span>
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
                                                    min="1"
                                                    class={classes!("w-24", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2", "py-1.5", "text-xs")}
                                                    placeholder="RPM"
                                                    value={selected_request_rpm_value.clone()}
                                                    oninput={{
                                                        let account_request_rpm_inputs = account_request_rpm_inputs.clone();
                                                        Callback::from(move |event: InputEvent| {
                                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                                let mut next = (*account_request_rpm_inputs).clone();
                                                                next.insert(acc_name_for_request_rpm_change.clone(), target.value());
                                                                account_request_rpm_inputs.set(next);
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
                                                        type="checkbox" class={classes!("min-h-0", "w-auto")}
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
                                                    class={classes!("ghost")}
                                                    onclick={Callback::from(move |_| on_save_account_settings.emit(acc_name_for_settings_save.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "保存" } }
                                                </button>
                                            </div>
                                            <div class={classes!("mt-2", "flex", "items-center", "gap-2", "flex-wrap")}>
                                                <button
                                                    class={classes!("ghost")}
                                                    onclick={Callback::from(move |_| on_refresh_account_auth.emit(acc_name_for_auth_refresh.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "刷新 Token" } }
                                                </button>
                                                <button
                                                    type="button"
                                                    class={classes!(
                                                        "ghost",
                                                        if auto_refresh_enabled { "btn-terminal-primary" } else { "" }
                                                    )}
                                                    aria-pressed={auto_refresh_enabled.to_string()}
                                                    aria-label={if auto_refresh_enabled { "关闭 Token 自动刷新" } else { "开启 Token 自动刷新" }}
                                                    title={if auto_refresh_enabled { "Token 自动刷新已开启，点击关闭" } else { "Token 自动刷新已关闭，点击开启" }}
                                                    onclick={Callback::from(move |_| {
                                                        on_toggle_account_auto_refresh.emit((
                                                            acc_name_for_auto_refresh_toggle.clone(),
                                                            !auto_refresh_enabled,
                                                        ))
                                                    })}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else if auto_refresh_enabled { "Token 自动刷新：开" } else { "Token 自动刷新：关" } }
                                                </button>
                                                <button
                                                    class={classes!("ghost")}
                                                    onclick={Callback::from(move |_| on_refresh_account_usage.emit(acc_name_for_usage_refresh.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "刷新 Usage" } }
                                                </button>
                                                <button
                                                    class={classes!(
                                                        "ghost",
                                                        if reset_credit_available { "btn-terminal-primary" } else { "" }
                                                    )}
                                                    onclick={Callback::from(move |_| on_open_account_reset_credit.emit(acc_name_for_reset_credit_open.clone()))}
                                                    disabled={account_busy || account_disabled || !reset_credit_available}
                                                    title="使用一个 Codex usage limit reset credit"
                                                >
                                                    { if account_busy { "..." } else if reset_credit_available { "重置限额" } else { "无 Reset" } }
                                                </button>
                                                <button
                                                    class={classes!("ghost")}
                                                    onclick={Callback::from(move |_| on_probe_account_models.emit(acc_name_for_models_probe.clone()))}
                                                    disabled={account_busy}
                                                >
                                                    { if account_busy { "..." } else { "测试 Models" } }
                                                </button>
                                                <button
                                                    class={classes!("ghost")}
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
                                                            "ghost",
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
                                                    class={classes!("ghost", "btn-terminal-danger")}
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
            </div>
            if let Some(picker) = (*reset_credit_picker).clone() {
                <Modal
                    open={true}
                    title={format!("使用 {} 的 Reset Credit", picker.account_name)}
                    on_close={on_cancel_account_reset_credit.clone()}
                >
                    <div class={classes!("space-y-4")}>
                        <p class={classes!("m-0", "text-sm", "text-[var(--muted)]")}>
                            { format!("当前可用 {} 个。此操作会立即消耗一个 credit；取消按钮是默认安全动作。", picker.details.available_count) }
                        </p>
                        if picker.details.credits.is_empty() {
                            <div class={classes!("rounded-lg", "border", "border-amber-500/40", "bg-amber-500/10", "p-3", "text-sm")}>
                                { "上游没有返回可选择的 credit 明细，将使用兼容的无 credit_id 请求。" }
                            </div>
                        } else {
                            <label class={classes!("grid", "gap-2", "text-sm")}>
                                <span class={classes!("font-semibold")}>{ "选择要使用的 credit" }</span>
                                <select
                                    class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={picker.selected_credit_id.clone()}
                                    onchange={{
                                        let on_select_account_reset_credit = on_select_account_reset_credit.clone();
                                        Callback::from(move |event: Event| {
                                            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                                on_select_account_reset_credit.emit(target.value());
                                            }
                                        })
                                    }}
                                >
                                    <option value="">{ "取消 / 请选择" }</option>
                                    { for picker.details.credits.iter().filter(|credit| credit.status.eq_ignore_ascii_case("available")).map(|credit| {
                                        let label = credit.title.as_deref().unwrap_or(&credit.id);
                                        let expiry = credit.expires_at.as_deref().unwrap_or("无到期时间");
                                        html! {
                                            <option value={credit.id.clone()}>
                                                { format!("{} · {} · {}", label, credit.status, expiry) }
                                            </option>
                                        }
                                    }) }
                                </select>
                            </label>
                            if let Some(selected) = picker
                                .details
                                .credits
                                .iter()
                                .find(|credit| {
                                    credit.id == picker.selected_credit_id
                                        && credit.status.eq_ignore_ascii_case("available")
                                })
                            {
                                <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-3", "text-sm", "space-y-1")}>
                                    <div>{ format!("ID: {}", selected.id) }</div>
                                    <div>{ format!("类型: {}", selected.reset_type) }</div>
                                    <div>{ format!("授予: {}", selected.granted_at) }</div>
                                    <div>{ format!("到期: {}", selected.expires_at.as_deref().unwrap_or("无")) }</div>
                                    if let Some(description) = selected.description.as_deref() {
                                        <div class={classes!("text-[var(--muted)]")}>{ description }</div>
                                    }
                                </div>
                            }
                        }
                        <div class={classes!("modal-actions")}>
                            <button
                                type="button"
                                class={classes!("modal-btn", "modal-btn--ghost")}
                                disabled={account_action_inflight.contains(&picker.account_name)}
                                onclick={{
                                    let on_cancel_account_reset_credit = on_cancel_account_reset_credit.clone();
                                    Callback::from(move |_| on_cancel_account_reset_credit.emit(()))
                                }}
                            >
                                { "取消" }
                            </button>
                            <button
                                type="button"
                                class={classes!("modal-btn", "modal-btn--danger")}
                                disabled={
                                    account_action_inflight.contains(&picker.account_name)
                                        || selected_reset_credit_id(
                                            &picker.details,
                                            &picker.selected_credit_id,
                                        )
                                        .is_err()
                                }
                                onclick={{
                                    let on_confirm_account_reset_credit = on_confirm_account_reset_credit.clone();
                                    Callback::from(move |_| on_confirm_account_reset_credit.emit(()))
                                }}
                            >
                                { if account_action_inflight.contains(&picker.account_name) { "处理中..." } else { "确认使用" } }
                            </button>
                        </div>
                    </div>
                </Modal>
            }
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

#[cfg(test)]
mod tests {
    use super::{parse_admin_codex_batch_import_json, selected_reset_credit_id};
    use crate::api::{CodexRateLimitResetCreditDetails, CodexRateLimitResetCreditsDetails};

    fn reset_credit_details(
        credits: Vec<CodexRateLimitResetCreditDetails>,
    ) -> CodexRateLimitResetCreditsDetails {
        CodexRateLimitResetCreditsDetails {
            available_count: 1,
            credits,
        }
    }

    #[test]
    fn reset_credit_picker_requires_an_explicit_valid_selection_when_details_exist() {
        let details = reset_credit_details(vec![CodexRateLimitResetCreditDetails {
            id: "credit-1".to_string(),
            reset_type: "codex_rate_limits".to_string(),
            status: "available".to_string(),
            granted_at: "2026-07-01T00:00:00Z".to_string(),
            expires_at: None,
            title: None,
            description: None,
        }]);

        assert!(selected_reset_credit_id(&details, "").is_err());
        assert!(selected_reset_credit_id(&details, "missing").is_err());
        assert_eq!(
            selected_reset_credit_id(&details, "credit-1").expect("selected credit"),
            Some("credit-1".to_string())
        );
    }

    #[test]
    fn reset_credit_picker_uses_legacy_no_id_only_when_no_details_exist() {
        let details = reset_credit_details(Vec::new());

        assert_eq!(selected_reset_credit_id(&details, "").expect("legacy reset"), None);
    }

    #[test]
    fn reset_credit_picker_rejects_unavailable_credit_details() {
        let details = reset_credit_details(vec![CodexRateLimitResetCreditDetails {
            id: "credit-used".to_string(),
            reset_type: "codex_rate_limits".to_string(),
            status: "consumed".to_string(),
            granted_at: "2026-07-01T00:00:00Z".to_string(),
            expires_at: None,
            title: None,
            description: None,
        }]);

        assert!(selected_reset_credit_id(&details, "credit-used").is_err());
        assert!(selected_reset_credit_id(&details, "").is_err());
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
