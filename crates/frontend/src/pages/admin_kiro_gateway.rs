//! Admin UI for managing Kiro accounts, keys, usage, and proxy bindings.

use std::collections::BTreeMap;

use gloo_timers::callback::Timeout;
use llm_access_core::store as llm_store;
use serde::{Deserialize, Serialize};
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_router::prelude::{use_navigator, Link};

use crate::{
    api::{
        delete_admin_kiro_account, delete_admin_kiro_key, fetch_admin_kiro_accounts_page,
        fetch_admin_kiro_cache_stats, fetch_admin_kiro_keys_page, fetch_admin_llm_gateway_config,
        fetch_admin_llm_gateway_proxy_bindings, fetch_admin_llm_gateway_proxy_configs,
        patch_admin_kiro_account, patch_admin_kiro_key, refresh_admin_kiro_account_balance,
        update_admin_llm_gateway_config, AdminAccountGroupOptionView, AdminAccountsSummaryView,
        AdminAnthropicUpstreamChannelView, AdminKiroCacheStatsResponse,
        AdminKiroKeyCandidateCreditSummaryView, AdminLlmGatewayKeyView,
        AdminLlmGatewayKeysSummaryView, AdminUpstreamProxyBindingView,
        AdminUpstreamProxyConfigView, KiroAccountView, KiroBalanceView, KiroModelView,
        LlmGatewayRuntimeConfig, PatchAdminLlmGatewayKeyRequest, PatchKiroAccountInput,
    },
    pages::llm_access_shared::{
        confirm_destructive, format_float2, format_kiro_disabled_reason, format_ms,
        format_number_i64, format_number_u64, format_reset_hint, format_timestamp_opt,
        kiro_credit_ratio, kiro_key_usage_ratio, MaskedSecretCode,
    },
    router::Route,
};

const TAB_ACCOUNTS: &str = "accounts";
const TAB_KEYS: &str = "keys";
const TAB_GROUPS: &str = "groups";
const TAB_USAGE: &str = "usage";
pub(crate) const DEFAULT_KIRO_KEY_PAGE_SIZE: usize = 24;

pub(crate) fn kiro_account_status_route() -> Route {
    Route::AdminKiroAccountStatus
}

pub(crate) fn kiro_account_status_cta_text() -> &'static str {
    "Open Account Status Page"
}

pub(crate) fn kiro_account_status_abnormal_href() -> &'static str {
    if cfg!(feature = "mock") {
        "/static_flow/admin/kiro-gateway/accounts?issue=abnormal"
    } else {
        "/admin/kiro-gateway/accounts?issue=abnormal"
    }
}

pub(crate) fn admin_kiro_key_total_pages(total: usize, page_size: usize) -> usize {
    total.max(1).div_ceil(page_size.max(1))
}

/// Shared Tailwind classes for the dark "Kiro" pill badge.
fn kiro_badge() -> Classes {
    classes!(
        "inline-flex",
        "items-center",
        "rounded-full",
        "bg-slate-900",
        "px-2.5",
        "py-1",
        "font-mono",
        "text-[11px]",
        "font-semibold",
        "uppercase",
        "tracking-[0.16em]",
        "text-emerald-300"
    )
}

fn format_float4(value: f64) -> String {
    format!("{value:.4}")
}

fn parse_manual_usage_limit_input(raw: &str) -> Result<Option<f64>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    match trimmed.parse::<f64>() {
        Ok(value) if value.is_finite() && value >= 0.0 => Ok(Some(value)),
        _ => Err("Manual usage limit must be a non-negative number.".to_string()),
    }
}

fn kiro_cache_token_percent(resident_tokens: u64, max_tokens: u64) -> f64 {
    if max_tokens == 0 {
        return 0.0;
    }
    (resident_tokens as f64 / max_tokens as f64 * 100.0).clamp(0.0, 100.0)
}

fn format_compact_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let value = bytes as f64;
    if value >= GIB {
        format!("{:.1} GiB", value / GIB)
    } else if value >= MIB {
        format!("{:.1} MiB", value / MIB)
    } else if value >= KIB {
        format!("{:.1} KiB", value / KIB)
    } else {
        format!("{bytes} B")
    }
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

pub(crate) fn format_optional_stream_bytes(bytes_streamed: Option<u64>) -> String {
    bytes_streamed
        .map(format_compact_bytes)
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn format_usage_stream_summary(
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
        "{} · final {} · {}",
        usage_stream_state_label(stream_completed_cleanly, downstream_disconnect),
        final_event_type,
        format_optional_stream_bytes(bytes_streamed),
    )
}

pub(crate) fn format_json_for_textarea(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .unwrap_or_else(|| raw.to_string())
}

fn routing_diagnostic_string(raw: Option<&str>, key: &str) -> Option<String> {
    raw.and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .and_then(|value| {
            value
                .get(key)
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
}

pub(crate) fn anthropic_routing_badge(raw: Option<&str>) -> Option<&'static str> {
    match routing_diagnostic_string(raw, "upstream_pool").as_deref() {
        Some("direct_anthropic_test") => Some("Anthropic 测试"),
        Some("direct_anthropic") => Some("Anthropic 直连"),
        _ => None,
    }
}

const PREFLIGHT_CHANGE_COUNT_KEYS: [&str; 3] =
    ["tool_use_id_rewrite_count", "normalization_event_count", "tool_normalization_event_count"];

fn routing_diagnostic_preflight_change_count(raw: Option<&str>) -> Option<u64> {
    let preflight = raw
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .and_then(|value| value.get("preflight").cloned())?;
    if !preflight
        .get("normalized")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }
    if let Some(count) = preflight
        .get("change_count")
        .and_then(serde_json::Value::as_u64)
    {
        return (count > 0).then_some(count);
    }
    let count = PREFLIGHT_CHANGE_COUNT_KEYS
        .into_iter()
        .filter_map(|key| preflight.get(key).and_then(serde_json::Value::as_u64))
        .sum::<u64>();
    (count > 0).then_some(count)
}

pub(crate) fn anthropic_routing_summary(raw: Option<&str>) -> Option<String> {
    let badge = anthropic_routing_badge(raw)?;
    let channel = routing_diagnostic_string(raw, "channel_name");
    let probe_kind = routing_diagnostic_string(raw, "probe_kind");
    let preflight_changes = routing_diagnostic_preflight_change_count(raw);
    let mut parts = vec![badge.to_string()];
    if let Some(channel) = channel {
        parts.push(format!("channel {channel}"));
    }
    if let Some(probe_kind) = probe_kind {
        parts.push(probe_kind);
    }
    if let Some(count) = preflight_changes {
        let noun = if count == 1 { "change" } else { "changes" };
        parts.push(format!("preflight {count} {noun}"));
    }
    Some(parts.join(" · "))
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct KiroCachePolicyBandForm {
    credit_start: String,
    credit_end: String,
    cache_ratio_start: String,
    cache_ratio_end: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct KiroCachePolicyForm {
    target_input_tokens: String,
    credit_start: String,
    credit_end: String,
    high_credit_diagnostic_threshold: String,
    anthropic_cache_creation_input_ratio: String,
    bands: Vec<KiroCachePolicyBandForm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KiroCachePolicyJson {
    small_input_high_credit_boost: KiroSmallInputHighCreditBoostJson,
    prefix_tree_credit_ratio_bands: Vec<KiroCachePolicyBandJson>,
    high_credit_diagnostic_threshold: f64,
    #[serde(default)]
    anthropic_cache_creation_input_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KiroSmallInputHighCreditBoostJson {
    target_input_tokens: u64,
    credit_start: f64,
    credit_end: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KiroCachePolicyBandJson {
    credit_start: f64,
    credit_end: f64,
    cache_ratio_start: f64,
    cache_ratio_end: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct KiroCachePolicyOverrideJson {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    small_input_high_credit_boost: Option<KiroSmallInputHighCreditBoostOverrideJson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prefix_tree_credit_ratio_bands: Option<Vec<KiroCachePolicyBandJson>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    high_credit_diagnostic_threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    anthropic_cache_creation_input_ratio: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct KiroSmallInputHighCreditBoostOverrideJson {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    credit_start: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    credit_end: Option<f64>,
}

fn format_kiro_cache_policy_number(value: f64) -> String {
    let mut text = value.to_string();
    if !text.contains(['.', 'e', 'E']) {
        text.push_str(".0");
    }
    text
}

fn parse_kiro_cache_policy_u64(label: &str, raw: &str) -> Result<u64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} must not be empty."));
    }
    trimmed
        .parse::<u64>()
        .map_err(|_| format!("{label} must be a valid integer."))
}

fn parse_kiro_cache_policy_f64(label: &str, raw: &str) -> Result<f64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} must not be empty."));
    }
    let value = trimmed
        .parse::<f64>()
        .map_err(|_| format!("{label} must be a valid number."))?;
    if !value.is_finite() {
        return Err(format!("{label} must be finite."));
    }
    Ok(value)
}

fn kiro_cache_policy_form_from_json(policy: &KiroCachePolicyJson) -> KiroCachePolicyForm {
    KiroCachePolicyForm {
        target_input_tokens: policy
            .small_input_high_credit_boost
            .target_input_tokens
            .to_string(),
        credit_start: format_kiro_cache_policy_number(
            policy.small_input_high_credit_boost.credit_start,
        ),
        credit_end: format_kiro_cache_policy_number(
            policy.small_input_high_credit_boost.credit_end,
        ),
        high_credit_diagnostic_threshold: format_kiro_cache_policy_number(
            policy.high_credit_diagnostic_threshold,
        ),
        anthropic_cache_creation_input_ratio: format_kiro_cache_policy_number(
            policy.anthropic_cache_creation_input_ratio,
        ),
        bands: policy
            .prefix_tree_credit_ratio_bands
            .iter()
            .map(|band| KiroCachePolicyBandForm {
                credit_start: format_kiro_cache_policy_number(band.credit_start),
                credit_end: format_kiro_cache_policy_number(band.credit_end),
                cache_ratio_start: format_kiro_cache_policy_number(band.cache_ratio_start),
                cache_ratio_end: format_kiro_cache_policy_number(band.cache_ratio_end),
            })
            .collect(),
    }
}

fn kiro_cache_policy_json_from_form(
    form: &KiroCachePolicyForm,
) -> Result<KiroCachePolicyJson, String> {
    let bands = form
        .bands
        .iter()
        .enumerate()
        .map(|(index, band)| {
            Ok(KiroCachePolicyBandJson {
                credit_start: parse_kiro_cache_policy_f64(
                    &format!("Band {} credit start", index + 1),
                    &band.credit_start,
                )?,
                credit_end: parse_kiro_cache_policy_f64(
                    &format!("Band {} credit end", index + 1),
                    &band.credit_end,
                )?,
                cache_ratio_start: parse_kiro_cache_policy_f64(
                    &format!("Band {} cache ratio start", index + 1),
                    &band.cache_ratio_start,
                )?,
                cache_ratio_end: parse_kiro_cache_policy_f64(
                    &format!("Band {} cache ratio end", index + 1),
                    &band.cache_ratio_end,
                )?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let policy = KiroCachePolicyJson {
        small_input_high_credit_boost: KiroSmallInputHighCreditBoostJson {
            target_input_tokens: parse_kiro_cache_policy_u64(
                "Small-input target input tokens",
                &form.target_input_tokens,
            )?,
            credit_start: parse_kiro_cache_policy_f64(
                "Small-input credit start",
                &form.credit_start,
            )?,
            credit_end: parse_kiro_cache_policy_f64("Small-input credit end", &form.credit_end)?,
        },
        prefix_tree_credit_ratio_bands: bands,
        high_credit_diagnostic_threshold: parse_kiro_cache_policy_f64(
            "High-credit diagnostic threshold",
            &form.high_credit_diagnostic_threshold,
        )?,
        anthropic_cache_creation_input_ratio: parse_kiro_cache_policy_f64(
            "Anthropic cache creation input ratio",
            &form.anthropic_cache_creation_input_ratio,
        )?,
    };
    Ok(policy)
}

pub(crate) fn parse_kiro_cache_policy_form_json(raw: &str) -> Result<KiroCachePolicyForm, String> {
    let policy = serde_json::from_str::<KiroCachePolicyJson>(raw)
        .map_err(|err| format!("Failed to parse kiro cache policy JSON: {err}"))?;
    Ok(kiro_cache_policy_form_from_json(&policy))
}

fn serialize_kiro_cache_policy_form_json(form: &KiroCachePolicyForm) -> Result<String, String> {
    let policy = kiro_cache_policy_json_from_form(form)?;
    serde_json::to_string(&policy)
        .map_err(|err| format!("Failed to serialize kiro cache policy: {err}"))
}

fn build_kiro_cache_policy_override_json(
    global: &KiroCachePolicyForm,
    edited: &KiroCachePolicyForm,
) -> Result<Option<String>, String> {
    let global_policy = kiro_cache_policy_json_from_form(global)?;
    let edited_policy = kiro_cache_policy_json_from_form(edited)?;
    let mut override_policy = KiroCachePolicyOverrideJson::default();
    let mut boost_override = KiroSmallInputHighCreditBoostOverrideJson::default();

    if edited_policy
        .small_input_high_credit_boost
        .target_input_tokens
        != global_policy
            .small_input_high_credit_boost
            .target_input_tokens
    {
        boost_override.target_input_tokens = Some(
            edited_policy
                .small_input_high_credit_boost
                .target_input_tokens,
        );
    }
    if edited_policy.small_input_high_credit_boost.credit_start
        != global_policy.small_input_high_credit_boost.credit_start
    {
        boost_override.credit_start =
            Some(edited_policy.small_input_high_credit_boost.credit_start);
    }
    if edited_policy.small_input_high_credit_boost.credit_end
        != global_policy.small_input_high_credit_boost.credit_end
    {
        boost_override.credit_end = Some(edited_policy.small_input_high_credit_boost.credit_end);
    }
    if boost_override != KiroSmallInputHighCreditBoostOverrideJson::default() {
        override_policy.small_input_high_credit_boost = Some(boost_override);
    }
    if edited_policy.prefix_tree_credit_ratio_bands != global_policy.prefix_tree_credit_ratio_bands
    {
        override_policy.prefix_tree_credit_ratio_bands =
            Some(edited_policy.prefix_tree_credit_ratio_bands.clone());
    }
    if edited_policy.high_credit_diagnostic_threshold
        != global_policy.high_credit_diagnostic_threshold
    {
        override_policy.high_credit_diagnostic_threshold =
            Some(edited_policy.high_credit_diagnostic_threshold);
    }
    if edited_policy.anthropic_cache_creation_input_ratio
        != global_policy.anthropic_cache_creation_input_ratio
    {
        override_policy.anthropic_cache_creation_input_ratio =
            Some(edited_policy.anthropic_cache_creation_input_ratio);
    }
    if override_policy == KiroCachePolicyOverrideJson::default() {
        Ok(None)
    } else {
        serde_json::to_string(&override_policy)
            .map(Some)
            .map_err(|err| format!("Failed to serialize kiro cache policy override: {err}"))
    }
}

fn build_kiro_cache_policy_override_patch(
    persisted_global: &KiroCachePolicyForm,
    initial_override_enabled: bool,
    initial_effective: &KiroCachePolicyForm,
    edited_override_enabled: bool,
    edited_effective: &KiroCachePolicyForm,
) -> Result<Option<Option<String>>, String> {
    let editor_state_changed = edited_override_enabled != initial_override_enabled
        || (edited_override_enabled && edited_effective != initial_effective);
    if !editor_state_changed {
        return Ok(None);
    }

    if !edited_override_enabled {
        return Ok(initial_override_enabled.then_some(None));
    }

    match build_kiro_cache_policy_override_json(persisted_global, edited_effective)? {
        Some(json) => Ok(Some(Some(json))),
        None if initial_override_enabled => Ok(Some(None)),
        None => Ok(None),
    }
}

fn default_kiro_billable_multiplier_map() -> BTreeMap<String, f64> {
    llm_store::default_kiro_billable_model_multipliers()
}

fn parse_kiro_billable_multiplier_json_with_base(
    raw: &str,
    base: &BTreeMap<String, f64>,
) -> Result<BTreeMap<String, f64>, String> {
    let overrides = serde_json::from_str::<BTreeMap<String, f64>>(raw)
        .map_err(|err| format!("Failed to parse kiro billable multiplier JSON: {err}"))?;
    let mut merged = base.clone();
    for (family, multiplier) in overrides {
        if !llm_store::is_kiro_billable_model_family(&family) {
            return Err(format!(
                "Unsupported multiplier family `{family}`. Use only `fable`, `haiku`, `opus`, \
                 `sonnet`."
            ));
        }
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(format!("Multiplier `{family}` must be a positive finite number."));
        }
        merged.insert(family, multiplier);
    }
    Ok(merged)
}

fn build_kiro_billable_multiplier_override_json(
    global_raw: &str,
    edited_raw: &str,
) -> Result<Option<String>, String> {
    let defaults = default_kiro_billable_multiplier_map();
    let global = parse_kiro_billable_multiplier_json_with_base(global_raw, &defaults)?;
    let edited = parse_kiro_billable_multiplier_json_with_base(edited_raw, &global)?;
    let mut overrides = BTreeMap::new();
    for family in llm_store::KIRO_BILLABLE_MODEL_FAMILIES {
        if edited.get(family) != global.get(family) {
            overrides.insert(
                family.to_string(),
                *edited
                    .get(family)
                    .expect("billable multiplier family should always exist"),
            );
        }
    }
    if overrides.is_empty() {
        Ok(None)
    } else {
        serde_json::to_string(&overrides)
            .map(Some)
            .map_err(|err| format!("Failed to serialize kiro billable multiplier override: {err}"))
    }
}

fn build_kiro_billable_multiplier_override_patch(
    persisted_global_raw: &str,
    initial_override_enabled: bool,
    initial_effective_raw: &str,
    edited_override_enabled: bool,
    edited_effective_raw: &str,
) -> Result<Option<Option<String>>, String> {
    let initial_effective = parse_kiro_billable_multiplier_json_with_base(
        initial_effective_raw,
        &default_kiro_billable_multiplier_map(),
    )?;
    let edited_effective = parse_kiro_billable_multiplier_json_with_base(
        edited_effective_raw,
        &default_kiro_billable_multiplier_map(),
    )?;
    let editor_state_changed = edited_override_enabled != initial_override_enabled
        || (edited_override_enabled && edited_effective != initial_effective);
    if !editor_state_changed {
        return Ok(None);
    }
    if !edited_override_enabled {
        return Ok(initial_override_enabled.then_some(None));
    }
    match build_kiro_billable_multiplier_override_json(persisted_global_raw, edited_effective_raw)?
    {
        Some(json) => Ok(Some(Some(json))),
        None if initial_override_enabled => Ok(Some(None)),
        None => Ok(None),
    }
}

fn format_kiro_billable_multiplier_summary(uses_global: bool, effective_raw: &str) -> String {
    match parse_kiro_billable_multiplier_json_with_base(
        effective_raw,
        &default_kiro_billable_multiplier_map(),
    ) {
        Ok(effective) => format!(
            "{} · fable {} · opus {} · sonnet {} · haiku {}",
            if uses_global { "inherit global" } else { "override" },
            format_float4(*effective.get("fable").unwrap_or(&2.0)),
            format_float4(*effective.get("opus").unwrap_or(&1.0)),
            format_float4(*effective.get("sonnet").unwrap_or(&1.0)),
            format_float4(*effective.get("haiku").unwrap_or(&1.0)),
        ),
        Err(err) => format!("invalid effective multiplier json · {err}"),
    }
}

fn should_reset_kiro_cache_policy_editor(
    is_initial_load: bool,
    editable_form: &KiroCachePolicyForm,
    persisted_form: &KiroCachePolicyForm,
) -> bool {
    is_initial_load || editable_form == persisted_form
}

fn format_kiro_cache_policy_summary(
    global: &KiroCachePolicyForm,
    effective: &KiroCachePolicyForm,
) -> String {
    format_kiro_cache_policy_summary_with_scope(
        if global == effective { "inherit global" } else { "override" },
        effective,
    )
}

fn format_effective_kiro_cache_policy_summary(
    uses_global: bool,
    effective: &KiroCachePolicyForm,
) -> String {
    format_kiro_cache_policy_summary_with_scope(
        if uses_global { "inherit global" } else { "override" },
        effective,
    )
}

fn format_kiro_cache_policy_summary_with_scope(
    scope: &str,
    effective: &KiroCachePolicyForm,
) -> String {
    let scope = if scope.is_empty() { "inherit global" } else { scope };
    format!(
        "{scope} · boost {} -> {} => {} · diag {} · create {} · bands {}",
        effective.credit_start.trim(),
        effective.credit_end.trim(),
        effective.target_input_tokens.trim(),
        effective.high_credit_diagnostic_threshold.trim(),
        effective.anthropic_cache_creation_input_ratio.trim(),
        effective.bands.len(),
    )
}

fn format_cache_summary(account: &KiroAccountView) -> String {
    let status = account.cache.status.trim();
    if status.is_empty() {
        return "cache loading".to_string();
    }
    match account.cache.last_checked_at {
        Some(ts) => format!("cache {status} · checked {}", format_ms(ts)),
        None => format!("cache {status}"),
    }
}

fn kiro_account_proxy_select_value(account: &KiroAccountView) -> String {
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

fn sanitize_kiro_account_group_id(
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

fn kiro_group_name_for_id(groups: &[AdminAccountGroupOptionView], group_id: &str) -> String {
    groups
        .iter()
        .find(|group| group.id == group_id)
        .map(|group| group.name.clone())
        .unwrap_or_else(|| group_id.to_string())
}

fn kiro_key_route_summary(
    route_strategy: &str,
    account_group_id: &str,
    preferred_pool_strategy: &str,
    account_groups: &[AdminAccountGroupOptionView],
) -> String {
    if route_strategy == "fixed" {
        if account_group_id.is_empty() {
            "固定组：未选择".to_string()
        } else {
            format!("固定组：{}", kiro_group_name_for_id(account_groups, account_group_id))
        }
    } else if account_group_id.is_empty() {
        let preferred_pool_label = kiro_pool_strategy_label(preferred_pool_strategy);
        format!(
            "全账号池自动择优，优先使用标记为 `{preferred_pool_label}` \
             的账号池；若该池当前都不可用，会回退到其他池；若没有可用账号，请求会直接报错。"
        )
    } else {
        let preferred_pool_label = kiro_pool_strategy_label(preferred_pool_strategy);
        format!(
            "仅在账号组 `{}` 中自动择优，优先使用组内标记为 `{}` \
             的账号池；若优先池当前都不可用，会回退到组内其他池；若组内没有可用账号，\
             请求会直接报错。",
            kiro_group_name_for_id(account_groups, account_group_id),
            preferred_pool_label
        )
    }
}

fn kiro_model_group_preferences_summary(
    preferences: &BTreeMap<String, String>,
    account_groups: &[AdminAccountGroupOptionView],
) -> String {
    if preferences.is_empty() {
        return "0 rules".to_string();
    }
    let mut entries = preferences
        .iter()
        .take(3)
        .map(|(model, group_id)| {
            format!("{model} -> {}", kiro_group_name_for_id(account_groups, group_id))
        })
        .collect::<Vec<_>>();
    if preferences.len() > entries.len() {
        entries.push(format!("+{}", preferences.len() - entries.len()));
    }
    format!("{} rules · {}", preferences.len(), entries.join(" · "))
}

fn kiro_model_channel_preferences_summary(preferences: &BTreeMap<String, String>) -> String {
    if preferences.is_empty() {
        return "0 channels".to_string();
    }
    let mut entries = preferences
        .iter()
        .take(3)
        .map(|(model, channel)| format!("{model} -> {channel}"))
        .collect::<Vec<_>>();
    if preferences.len() > entries.len() {
        entries.push(format!("+{}", preferences.len() - entries.len()));
    }
    format!("{} channels · {}", preferences.len(), entries.join(" · "))
}

fn kiro_model_routing_summary(
    group_preferences: &BTreeMap<String, String>,
    channel_preferences: &BTreeMap<String, String>,
    account_groups: &[AdminAccountGroupOptionView],
) -> String {
    format!(
        "groups: {}; direct channels: {}",
        kiro_model_group_preferences_summary(group_preferences, account_groups),
        kiro_model_channel_preferences_summary(channel_preferences)
    )
}

fn kiro_pool_strategy_label(strategy: &str) -> &'static str {
    match llm_store::normalize_kiro_pool_strategy(strategy) {
        Some(llm_store::KIRO_POOL_STRATEGY_BALANCED) => "亲和 + 动态",
        Some(llm_store::KIRO_POOL_STRATEGY_CREDIT_FIRST) => "剩余额度优先",
        _ => "未知策略",
    }
}

pub(crate) fn kiro_pool_strategy_description(strategy: &str) -> &'static str {
    match llm_store::normalize_kiro_pool_strategy(strategy) {
        Some(llm_store::KIRO_POOL_STRATEGY_CREDIT_FIRST) => {
            "池内优先消耗剩余额度最高的账号，其次才参考首字延迟与轮转。"
        },
        _ => "按会话亲和、首字延迟与轮转均衡调度（历史默认行为）。",
    }
}

pub(crate) fn kiro_pool_strategy_options(current: &str) -> Html {
    html! {
        <>
            { for llm_store::KIRO_POOL_STRATEGIES.iter().map(|value| html! {
                <option value={*value} selected={*value == current} title={kiro_pool_strategy_description(value)}>
                    { kiro_pool_strategy_label(value) }
                </option>
            }) }
        </>
    }
}

fn anthropic_upstream_pool_mode_label(mode: &str) -> &'static str {
    match llm_store::normalize_anthropic_upstream_pool_mode(mode) {
        Some(llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_PREFERRED_BEFORE_KIRO) => "优先直连 Anthropic",
        Some(llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_KIRO_BEFORE_ANTHROPIC) => {
            "Kiro 优先，不支持模型时直连"
        },
        Some(llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_ONLY) => "仅直连 Anthropic",
        _ => "关闭",
    }
}

fn anthropic_upstream_pool_mode_description(mode: &str) -> &'static str {
    match llm_store::normalize_anthropic_upstream_pool_mode(mode) {
        Some(llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_PREFERRED_BEFORE_KIRO) => {
            "先使用标准 Anthropic upstream channel；可重试上游失败时回落 Kiro。"
        },
        Some(llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_KIRO_BEFORE_ANTHROPIC) => {
            "先使用 Kiro 账号池；仅当 Kiro 本地模型转换不支持该模型时，转发到标准 Anthropic \
             upstream channel。"
        },
        Some(llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_ONLY) => {
            "只使用标准 Anthropic upstream channel；无可用 channel 时直接报错。"
        },
        _ => "默认关闭，完全走现有 Kiro 账号池。",
    }
}

fn anthropic_upstream_pool_mode_options(current: &str) -> Html {
    const MODES: [&str; 4] = [
        llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_DISABLED,
        llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_PREFERRED_BEFORE_KIRO,
        llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_KIRO_BEFORE_ANTHROPIC,
        llm_store::ANTHROPIC_UPSTREAM_POOL_MODE_ONLY,
    ];
    html! {
        <>
            { for MODES.iter().map(|value| html! {
                <option value={*value} selected={*value == current} title={anthropic_upstream_pool_mode_description(value)}>
                    { anthropic_upstream_pool_mode_label(value) }
                </option>
            }) }
        </>
    }
}

fn format_kiro_key_candidate_credit_summary(
    summary: &AdminKiroKeyCandidateCreditSummaryView,
) -> String {
    if summary.candidate_count == 0 {
        return "候选账号额度: 当前没有命中任何账号。".to_string();
    }

    format!(
        "候选账号额度: 剩余 {} / 总额 {} · {}/{} 已加载{}",
        format_float4(summary.total_remaining),
        format_float4(summary.total_limit),
        summary.loaded_balance_count,
        summary.candidate_count,
        if summary.missing_balance_count == 0 {
            String::new()
        } else {
            format!(" · {} 个账号余额未加载", summary.missing_balance_count)
        }
    )
}

fn kiro_preferred_pool_warning(
    route_strategy: &str,
    preferred_pool_strategy: &str,
    summary: &AdminKiroKeyCandidateCreditSummaryView,
) -> Option<String> {
    if route_strategy == "fixed"
        || summary.candidate_count == 0
        || summary.preferred_pool_candidate_count != Some(0)
    {
        return None;
    }
    let preferred_pool_label = kiro_pool_strategy_label(preferred_pool_strategy);
    Some(format!(
        "当前候选集中没有标记为 `{preferred_pool_label}` \
         的账号，优先池设置不会生效；请求会回退到其他池。"
    ))
}

/// Positive companion to [`kiro_preferred_pool_warning`]: when the preferred
/// pool does match candidates, surface how many so admins can confirm the
/// preference is active without reading per-account settings.
fn kiro_preferred_pool_candidate_note(
    route_strategy: &str,
    preferred_pool_strategy: &str,
    summary: &AdminKiroKeyCandidateCreditSummaryView,
) -> Option<String> {
    if route_strategy == "fixed" || summary.candidate_count == 0 {
        return None;
    }
    let preferred_pool_candidate_count = summary.preferred_pool_candidate_count?;
    if preferred_pool_candidate_count == 0 {
        return None;
    }
    Some(format!(
        "优先池 `{}` 命中 {}/{} 个候选账号",
        kiro_pool_strategy_label(preferred_pool_strategy),
        preferred_pool_candidate_count,
        summary.candidate_count
    ))
}

#[derive(Properties, PartialEq)]
struct KiroCachePolicyEditorProps {
    form: UseStateHandle<KiroCachePolicyForm>,
}

#[function_component(KiroCachePolicyEditor)]
fn kiro_cache_policy_editor(props: &KiroCachePolicyEditorProps) -> Html {
    let on_add_band = {
        let form = props.form.clone();
        Callback::from(move |_| {
            let mut next = (*form).clone();
            next.bands.push(KiroCachePolicyBandForm::default());
            form.set(next);
        })
    };

    html! {
        <div class={classes!("space-y-3")}>
            <div class={classes!("grid", "gap-3", "md:grid-cols-2")}>
                <label class={classes!("block", "text-sm")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ "Boost Target Tokens" }</div>
                    <input
                        class={classes!("mono")}
                        value={props.form.target_input_tokens.clone()}
                        oninput={{
                            let form = props.form.clone();
                            Callback::from(move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                let mut next = (*form).clone();
                                next.target_input_tokens = input.value();
                                form.set(next);
                            })
                        }}
                    />
                </label>
                <label class={classes!("block", "text-sm")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ "Diagnostic Threshold" }</div>
                    <input
                        class={classes!("mono")}
                        value={props.form.high_credit_diagnostic_threshold.clone()}
                        oninput={{
                            let form = props.form.clone();
                            Callback::from(move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                let mut next = (*form).clone();
                                next.high_credit_diagnostic_threshold = input.value();
                                form.set(next);
                            })
                        }}
                    />
                </label>
                <label class={classes!("block", "text-sm")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ "Anthropic Creation Ratio" }</div>
                    <input
                        class={classes!("mono")}
                        value={props.form.anthropic_cache_creation_input_ratio.clone()}
                        oninput={{
                            let form = props.form.clone();
                            Callback::from(move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                let mut next = (*form).clone();
                                next.anthropic_cache_creation_input_ratio = input.value();
                                form.set(next);
                            })
                        }}
                    />
                </label>
                <label class={classes!("block", "text-sm")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ "Boost Credit Start" }</div>
                    <input
                        class={classes!("mono")}
                        value={props.form.credit_start.clone()}
                        oninput={{
                            let form = props.form.clone();
                            Callback::from(move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                let mut next = (*form).clone();
                                next.credit_start = input.value();
                                form.set(next);
                            })
                        }}
                    />
                </label>
                <label class={classes!("block", "text-sm")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ "Boost Credit End" }</div>
                    <input
                        class={classes!("mono")}
                        value={props.form.credit_end.clone()}
                        oninput={{
                            let form = props.form.clone();
                            Callback::from(move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                let mut next = (*form).clone();
                                next.credit_end = input.value();
                                form.set(next);
                            })
                        }}
                    />
                </label>
            </div>

            <div class={classes!("rounded-[var(--r-field)]", "border", "border-[var(--border)]", "bg-[var(--card-2)]", "px-3", "py-3")}>
                <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                    <div>
                        <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ "Prefix Tree Bands" }</div>
                        <div class={classes!("mt-1", "text-xs", "text-[var(--muted-foreground)]")}>
                            { "Bands replace as a whole when any row changes. Keep credit/rate continuity between adjacent rows." }
                        </div>
                    </div>
                    <button type="button" class={classes!("text-xs")} onclick={on_add_band}>
                        { "Add Band" }
                    </button>
                </div>
                <div class={classes!("mt-3", "space-y-2")}>
                    { for props.form.bands.iter().enumerate().map(|(index, band)| {
                        let remove_disabled = props.form.bands.len() <= 1;
                        html! {
                            <div class={classes!("grid", "gap-2", "md:grid-cols-[repeat(4,minmax(0,1fr))_auto]", "items-end")}>
                                <label class={classes!("block", "text-sm")}>
                                    <div class={classes!("mb-1", "text-[11px]", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ format!("Band {} Credit Start", index + 1) }</div>
                                    <input
                                        class={classes!("mono")}
                                        value={band.credit_start.clone()}
                                        oninput={{
                                            let form = props.form.clone();
                                            Callback::from(move |event: InputEvent| {
                                                let input: HtmlInputElement = event.target_unchecked_into();
                                                let mut next = (*form).clone();
                                                if let Some(target_band) = next.bands.get_mut(index) {
                                                    target_band.credit_start = input.value();
                                                }
                                                form.set(next);
                                            })
                                        }}
                                    />
                                </label>
                                <label class={classes!("block", "text-sm")}>
                                    <div class={classes!("mb-1", "text-[11px]", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ format!("Band {} Credit End", index + 1) }</div>
                                    <input
                                        class={classes!("mono")}
                                        value={band.credit_end.clone()}
                                        oninput={{
                                            let form = props.form.clone();
                                            Callback::from(move |event: InputEvent| {
                                                let input: HtmlInputElement = event.target_unchecked_into();
                                                let mut next = (*form).clone();
                                                if let Some(target_band) = next.bands.get_mut(index) {
                                                    target_band.credit_end = input.value();
                                                }
                                                form.set(next);
                                            })
                                        }}
                                    />
                                </label>
                                <label class={classes!("block", "text-sm")}>
                                    <div class={classes!("mb-1", "text-[11px]", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ format!("Band {} Ratio Start", index + 1) }</div>
                                    <input
                                        class={classes!("mono")}
                                        value={band.cache_ratio_start.clone()}
                                        oninput={{
                                            let form = props.form.clone();
                                            Callback::from(move |event: InputEvent| {
                                                let input: HtmlInputElement = event.target_unchecked_into();
                                                let mut next = (*form).clone();
                                                if let Some(target_band) = next.bands.get_mut(index) {
                                                    target_band.cache_ratio_start = input.value();
                                                }
                                                form.set(next);
                                            })
                                        }}
                                    />
                                </label>
                                <label class={classes!("block", "text-sm")}>
                                    <div class={classes!("mb-1", "text-[11px]", "uppercase", "tracking-[0.16em]", "text-[var(--muted-foreground)]")}>{ format!("Band {} Ratio End", index + 1) }</div>
                                    <input
                                        class={classes!("mono")}
                                        value={band.cache_ratio_end.clone()}
                                        oninput={{
                                            let form = props.form.clone();
                                            Callback::from(move |event: InputEvent| {
                                                let input: HtmlInputElement = event.target_unchecked_into();
                                                let mut next = (*form).clone();
                                                if let Some(target_band) = next.bands.get_mut(index) {
                                                    target_band.cache_ratio_end = input.value();
                                                }
                                                form.set(next);
                                            })
                                        }}
                                    />
                                </label>
                                <button
                                    type="button"
                                    class={classes!("danger", "text-xs")}
                                    disabled={remove_disabled}
                                    onclick={{
                                        let form = props.form.clone();
                                        Callback::from(move |_| {
                                            let mut next = (*form).clone();
                                            if next.bands.len() > 1 {
                                                next.bands.remove(index);
                                                form.set(next);
                                            }
                                        })
                                    }}
                                >
                                    { "Remove" }
                                </button>
                            </div>
                        }
                    }) }
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct KiroAccountCardProps {
    pub(crate) account: KiroAccountView,
    pub(crate) proxy_configs: Vec<AdminUpstreamProxyConfigView>,
    pub(crate) on_reload: Callback<()>,
    pub(crate) flash: UseStateHandle<Option<String>>,
    pub(crate) notify: Callback<(String, bool)>,
    pub(crate) error: UseStateHandle<Option<String>>,
}

#[function_component(KiroAccountCard)]
pub(crate) fn kiro_account_card(props: &KiroAccountCardProps) -> Html {
    let expanded = use_state(|| false);
    let scheduler_max = use_state(|| props.account.kiro_channel_max_concurrency.to_string());
    let scheduler_rpm = use_state(|| props.account.kiro_channel_rpm_limit.to_string());
    let scheduler_min = use_state(|| props.account.kiro_channel_min_start_interval_ms.to_string());
    let minimum_remaining_credits_before_block =
        use_state(|| format_float4(props.account.minimum_remaining_credits_before_block));
    let manual_usage_limit = use_state(|| {
        props
            .account
            .manual_usage_limit
            .map(format_float4)
            .unwrap_or_default()
    });
    let pool_strategy = use_state(|| props.account.pool_strategy.clone());
    let selected_proxy = use_state(|| kiro_account_proxy_select_value(&props.account));
    let feedback = use_state(|| None::<String>);
    let busy = use_state(|| false);

    {
        let account = props.account.clone();
        let scheduler_max = scheduler_max.clone();
        let scheduler_rpm = scheduler_rpm.clone();
        let scheduler_min = scheduler_min.clone();
        let minimum_remaining_credits_before_block = minimum_remaining_credits_before_block.clone();
        let manual_usage_limit = manual_usage_limit.clone();
        let pool_strategy = pool_strategy.clone();
        let selected_proxy = selected_proxy.clone();
        use_effect_with(props.account.clone(), move |_| {
            scheduler_max.set(account.kiro_channel_max_concurrency.to_string());
            scheduler_rpm.set(account.kiro_channel_rpm_limit.to_string());
            scheduler_min.set(account.kiro_channel_min_start_interval_ms.to_string());
            minimum_remaining_credits_before_block
                .set(format_float4(account.minimum_remaining_credits_before_block));
            manual_usage_limit.set(
                account
                    .manual_usage_limit
                    .map(format_float4)
                    .unwrap_or_default(),
            );
            pool_strategy.set(account.pool_strategy.clone());
            selected_proxy.set(kiro_account_proxy_select_value(&account));
            || ()
        });
    }

    let on_refresh_cache = {
        let account_name = props.account.name.clone();
        let flash = props.flash.clone();
        let notify = props.notify.clone();
        let error = props.error.clone();
        let feedback = feedback.clone();
        let busy = busy.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            let account_name = account_name.clone();
            let flash = flash.clone();
            let notify = notify.clone();
            let error = error.clone();
            let feedback = feedback.clone();
            let busy = busy.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                busy.set(true);
                error.set(None);
                match refresh_admin_kiro_account_balance(&account_name).await {
                    Ok(_) => {
                        feedback.set(Some("Cache refreshed.".to_string()));
                        let message = format!("Refreshed cached balance for `{account_name}`.");
                        flash.set(Some(message.clone()));
                        notify.emit((message, false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        error.set(Some(err.clone()));
                        notify.emit((
                            format!(
                                "Failed to refresh cached balance for `{account_name}`.\n{err}"
                            ),
                            true,
                        ));
                    },
                }
                busy.set(false);
            });
        })
    };

    let on_delete_account = {
        let account_name = props.account.name.clone();
        let flash = props.flash.clone();
        let notify = props.notify.clone();
        let error = props.error.clone();
        let feedback = feedback.clone();
        let busy = busy.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            if !confirm_destructive(&format!(
                "确认删除 Kiro 账号 `{}` ？此操作不可撤销。",
                account_name
            )) {
                return;
            }
            let account_name = account_name.clone();
            let flash = flash.clone();
            let notify = notify.clone();
            let error = error.clone();
            let feedback = feedback.clone();
            let busy = busy.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                busy.set(true);
                error.set(None);
                match delete_admin_kiro_account(&account_name).await {
                    Ok(_) => {
                        feedback.set(Some("Deleted.".to_string()));
                        let message = format!("Deleted `{account_name}`.");
                        flash.set(Some(message.clone()));
                        notify.emit((message, false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        error.set(Some(err.clone()));
                        notify.emit((format!("Failed to delete `{account_name}`.\n{err}"), true));
                    },
                }
                busy.set(false);
            });
        })
    };

    let on_save_scheduler = {
        let account_name = props.account.name.clone();
        let scheduler_max = scheduler_max.clone();
        let scheduler_rpm = scheduler_rpm.clone();
        let scheduler_min = scheduler_min.clone();
        let minimum_remaining_credits_before_block = minimum_remaining_credits_before_block.clone();
        let manual_usage_limit = manual_usage_limit.clone();
        let pool_strategy = pool_strategy.clone();
        let selected_proxy = selected_proxy.clone();
        let flash = props.flash.clone();
        let notify = props.notify.clone();
        let error = props.error.clone();
        let feedback = feedback.clone();
        let busy = busy.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            let account_name = account_name.clone();
            let scheduler_max = scheduler_max.clone();
            let scheduler_rpm = scheduler_rpm.clone();
            let scheduler_min = scheduler_min.clone();
            let minimum_remaining_credits_before_block =
                minimum_remaining_credits_before_block.clone();
            let manual_usage_limit = manual_usage_limit.clone();
            let pool_strategy = pool_strategy.clone();
            let selected_proxy = selected_proxy.clone();
            let flash = flash.clone();
            let notify = notify.clone();
            let error = error.clone();
            let feedback = feedback.clone();
            let busy = busy.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let parsed_max = match (*scheduler_max).trim().parse::<u64>() {
                    Ok(value) => value,
                    Err(_) => {
                        let message = "Max concurrency must be a valid integer.".to_string();
                        error.set(Some(message.clone()));
                        notify.emit((message, true));
                        return;
                    },
                };
                let parsed_min = match (*scheduler_min).trim().parse::<u64>() {
                    Ok(value) => value,
                    Err(_) => {
                        let message = "Min start interval must be a valid integer.".to_string();
                        error.set(Some(message.clone()));
                        notify.emit((message, true));
                        return;
                    },
                };
                let parsed_rpm = match (*scheduler_rpm).trim().parse::<u64>() {
                    Ok(value) if value > 0 => value,
                    _ => {
                        let message = "RPM must be an integer greater than zero.".to_string();
                        error.set(Some(message.clone()));
                        notify.emit((message, true));
                        return;
                    },
                };
                let parsed_minimum_remaining_credits_before_block =
                    match (*minimum_remaining_credits_before_block)
                        .trim()
                        .parse::<f64>()
                    {
                        Ok(value) if value.is_finite() && value >= 0.0 => value,
                        _ => {
                            let message = "Minimum remaining credits must be a non-negative \
                                           number."
                                .to_string();
                            error.set(Some(message.clone()));
                            notify.emit((message, true));
                            return;
                        },
                    };
                let parsed_manual_usage_limit =
                    match parse_manual_usage_limit_input((*manual_usage_limit).as_str()) {
                        Ok(value) => value,
                        Err(message) => {
                            error.set(Some(message.clone()));
                            notify.emit((message, true));
                            return;
                        },
                    };
                busy.set(true);
                error.set(None);
                let (proxy_mode, proxy_config_id) = if *selected_proxy == "direct" {
                    (Some("direct".to_string()), None)
                } else if let Some(proxy_config_id) = (*selected_proxy).strip_prefix("fixed:") {
                    (Some("fixed".to_string()), Some(proxy_config_id.to_string()))
                } else {
                    (Some("inherit".to_string()), None)
                };
                match patch_admin_kiro_account(&account_name, &PatchKiroAccountInput {
                    status: None,
                    kiro_channel_max_concurrency: Some(parsed_max),
                    kiro_channel_rpm_limit: Some(parsed_rpm),
                    kiro_channel_min_start_interval_ms: Some(parsed_min),
                    minimum_remaining_credits_before_block: Some(
                        parsed_minimum_remaining_credits_before_block,
                    ),
                    manual_usage_limit: Some(parsed_manual_usage_limit),
                    pool_strategy: Some((*pool_strategy).clone()),
                    proxy_mode,
                    proxy_config_id,
                })
                .await
                {
                    Ok(_) => {
                        feedback.set(Some("Account settings saved.".to_string()));
                        let message = format!("Updated account settings for `{account_name}`.");
                        flash.set(Some(message.clone()));
                        notify.emit((message, false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        error.set(Some(err.clone()));
                        notify.emit((
                            format!(
                                "Failed to update account settings for `{account_name}`.\n{err}"
                            ),
                            true,
                        ));
                    },
                }
                busy.set(false);
            });
        })
    };

    let on_toggle_disabled = {
        let account_name = props.account.name.clone();
        let currently_disabled = props.account.disabled;
        let flash = props.flash.clone();
        let notify = props.notify.clone();
        let error = props.error.clone();
        let feedback = feedback.clone();
        let busy = busy.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            let account_name = account_name.clone();
            let flash = flash.clone();
            let notify = notify.clone();
            let error = error.clone();
            let feedback = feedback.clone();
            let busy = busy.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                busy.set(true);
                error.set(None);
                let next_status = if currently_disabled { "active" } else { "disabled" };
                match patch_admin_kiro_account(&account_name, &PatchKiroAccountInput {
                    status: Some(next_status.to_string()),
                    kiro_channel_max_concurrency: None,
                    kiro_channel_rpm_limit: None,
                    kiro_channel_min_start_interval_ms: None,
                    minimum_remaining_credits_before_block: None,
                    manual_usage_limit: None,
                    pool_strategy: None,
                    proxy_mode: None,
                    proxy_config_id: None,
                })
                .await
                {
                    Ok(_) => {
                        let action = if currently_disabled { "enabled" } else { "disabled" };
                        feedback.set(Some(format!("Account {action}.")));
                        let message = format!("Updated `{account_name}` to {next_status}.");
                        flash.set(Some(message.clone()));
                        notify.emit((message, false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        error.set(Some(err.clone()));
                        notify.emit((
                            format!("Failed to update `{account_name}` status.\n{err}"),
                            true,
                        ));
                    },
                }
                busy.set(false);
            });
        })
    };

    let toggle_expanded = {
        let expanded = expanded.clone();
        Callback::from(move |_| expanded.set(!*expanded))
    };

    let account = props.account.clone();
    let email = account.email.clone().unwrap_or_else(|| "-".to_string());
    let expires_at = account
        .expires_at
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let profile_arn = account
        .profile_arn
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let machine_id = account
        .machine_id
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let region = account.region.clone().unwrap_or_else(|| "-".to_string());
    let auth_region = account
        .auth_region
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let api_region = account
        .api_region
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let proxy_url = account.proxy_url.clone().unwrap_or_else(|| "-".to_string());
    let effective_proxy_url = account
        .effective_proxy_url
        .clone()
        .unwrap_or_else(|| "direct".to_string());
    let source = account.source.clone().unwrap_or_else(|| "-".to_string());
    let source_db_path = account
        .source_db_path
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let last_imported = format_timestamp_opt(account.last_imported_at);
    let disabled_reason = format_kiro_disabled_reason(account.disabled_reason.as_deref());
    let issue_label = account.issue_kind.as_deref().map(|kind| match kind {
        "auth_401" => "401 auth".to_string(),
        "error" => "error".to_string(),
        "disabled" => "disabled issue".to_string(),
        _ => kind.to_string(),
    });
    let issue_at = format_timestamp_opt(account.issue_at_ms);
    let manual_limit_label = account
        .manual_usage_limit
        .map(format_float4)
        .unwrap_or_else(|| "upstream".to_string());

    html! {
        <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
            <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                <div>
                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                        <span class={kiro_badge()}>
                            { "Kiro" }
                        </span>
                        <h3 class={classes!("m-0", "text-lg", "font-semibold")}>{ account.name.clone() }</h3>
                        if account.disabled {
                            <span class={classes!("inline-flex", "items-center", "rounded-full", "border", "border-amber-500/20", "bg-amber-500/10", "px-2.5", "py-1", "text-[11px]", "font-semibold", "uppercase", "tracking-[0.16em]", "text-amber-700", "dark:text-amber-200")}>
                                { "disabled" }
                            </span>
                        }
                        if let Some(issue_label) = issue_label.clone() {
                            <span class={classes!("inline-flex", "items-center", "rounded-full", "border", "border-red-500/25", "bg-red-500/10", "px-2.5", "py-1", "text-[11px]", "font-semibold", "uppercase", "tracking-[0.16em]", "text-red-700", "dark:text-red-200")}>
                                { issue_label }
                            </span>
                        }
                    </div>
                    <p class={classes!("mt-2", "mb-0", "text-sm", "text-[var(--muted)]")}>
                        { format!("{} · provider {} · refresh {}", account.auth_method, account.provider.clone().unwrap_or_else(|| "-".to_string()), if account.has_refresh_token { "present" } else { "missing" }) }
                    </p>
                    <p class={classes!("mt-1", "mb-0", "text-xs", "font-mono", "text-[var(--muted)]")}>
                        { format_cache_summary(&account) }
                    </p>
                    <p class={classes!("mt-1", "mb-0", "text-xs", "font-mono", "text-[var(--muted)]")}>
                        { format!(
                            "scheduler {} in-flight · {} RPM · {} ms spacing · credit floor {} · manual limit {} · pool {}",
                            account.kiro_channel_max_concurrency,
                            account.kiro_channel_rpm_limit,
                            account.kiro_channel_min_start_interval_ms,
                            format_float4(account.minimum_remaining_credits_before_block),
                            manual_limit_label,
                            kiro_pool_strategy_label(&account.pool_strategy)
                        ) }
                    </p>
                    if let Some(issue_summary) = account.issue_summary.clone() {
                        <p class={classes!("mt-1", "mb-0", "text-xs", "font-mono", "text-red-700", "dark:text-red-200", "break-all")}>
                            { format!("issue {issue_at}: {issue_summary}") }
                        </p>
                    }
                    if let Some(cache_error) = account.cache.error_message.clone() {
                        <p class={classes!("mt-1", "mb-0", "text-xs", "font-mono", "text-amber-700", "dark:text-amber-200")}>
                            { cache_error }
                        </p>
                    }
                    if let Some(disabled_reason) = disabled_reason {
                        <p class={classes!("mt-1", "mb-0", "text-xs", "font-mono", "text-amber-700", "dark:text-amber-200")}>
                            { disabled_reason }
                        </p>
                    }
                </div>
                <div class={classes!("flex", "gap-2", "flex-wrap")}>
                    <button type="button" class={classes!("btn-terminal")} onclick={on_refresh_cache.clone()} disabled={*busy}>
                        { "Refresh Cache" }
                    </button>
                    <button type="button" class={classes!("btn-terminal")} onclick={on_toggle_disabled.clone()} disabled={*busy}>
                        { if account.disabled { "Enable" } else { "Disable" } }
                    </button>
                    <button type="button" class={classes!("btn-terminal", "btn-terminal-danger")} onclick={on_delete_account.clone()} disabled={*busy}>
                        { "Delete" }
                    </button>
                </div>
            </div>

            if !account.disabled {
                <div class={classes!("mt-4", "rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                    <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Quota Snapshot" }</div>
                    if let Some(balance) = account.balance.clone() {
                        { quota_progress_bar(&balance, account.subscription_title.clone()) }
                    } else {
                        <p class={classes!("mt-3", "mb-0", "text-sm", "text-[var(--muted)]")}>{ "Balance not loaded yet." }</p>
                    }
                </div>
            }

            <button
                type="button"
                class={classes!("mt-3", "btn-terminal", "text-xs")}
                onclick={toggle_expanded}
            >
                { if *expanded { "收起详情 ▲" } else { "展开详情 ▼" } }
            </button>

            if *expanded {
                <div class={classes!("mt-3", "grid", "gap-4", "lg:grid-cols-3")}>
                    <div class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                        <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Identity" }</div>
                        <dl class={classes!("mt-3", "space-y-2", "text-sm")}>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "email: " }</dt><dd class={classes!("inline", "font-mono", "break-all")}>{ email }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "expires_at: " }</dt><dd class={classes!("inline", "font-mono", "break-all")}>{ expires_at }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "profileArn: " }</dt><dd class={classes!("inline", "font-mono", "break-all")}>{ profile_arn }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "machineId: " }</dt><dd class={classes!("inline", "font-mono", "break-all")}>{ machine_id }</dd></div>
                        </dl>
                    </div>
                    <div class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                        <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Regions / Proxy" }</div>
                        <dl class={classes!("mt-3", "space-y-2", "text-sm")}>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "region: " }</dt><dd class={classes!("inline", "font-mono")}>{ region }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "auth_region: " }</dt><dd class={classes!("inline", "font-mono")}>{ auth_region }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "api_region: " }</dt><dd class={classes!("inline", "font-mono")}>{ api_region }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "effective_proxy: " }</dt><dd class={classes!("inline", "font-mono", "break-all")}>{ format!("{} · {}", account.effective_proxy_source, effective_proxy_url) }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "effective_proxy_config: " }</dt><dd class={classes!("inline", "font-mono", "break-all")}>{ account.effective_proxy_config_name.clone().unwrap_or_else(|| "-".to_string()) }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "legacy_proxy_url: " }</dt><dd class={classes!("inline", "font-mono", "break-all")}>{ proxy_url }</dd></div>
                        </dl>
                    </div>
                    <div class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                        <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Source" }</div>
                        <dl class={classes!("mt-3", "space-y-2", "text-sm")}>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "source: " }</dt><dd class={classes!("inline", "font-mono")}>{ source }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "source_db_path: " }</dt><dd class={classes!("inline", "font-mono", "break-all")}>{ source_db_path }</dd></div>
                            <div><dt class={classes!("inline", "text-[var(--muted)]")}>{ "last_imported_at: " }</dt><dd class={classes!("inline", "font-mono")}>{ last_imported }</dd></div>
                        </dl>
                    </div>
                </div>
                <div class={classes!("mt-4", "rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                    <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Scheduler / Proxy" }</div>
                    <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-7")}>
                        <label class={classes!("text-sm")}>
                            <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Max Concurrency" }</div>
                            <input
                                class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm", "font-mono")}
                                value={(*scheduler_max).clone()}
                                oninput={{
                                    let scheduler_max = scheduler_max.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        scheduler_max.set(input.value());
                                    })
                                }}
                            />
                        </label>
                        <label class={classes!("text-sm")}>
                            <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "RPM" }</div>
                            <input
                                type="number"
                                min="1"
                                class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm", "font-mono")}
                                value={(*scheduler_rpm).clone()}
                                oninput={{
                                    let scheduler_rpm = scheduler_rpm.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        scheduler_rpm.set(input.value());
                                    })
                                }}
                            />
                        </label>
                        <label class={classes!("text-sm")}>
                            <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Min Start Interval Ms" }</div>
                            <input
                                class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm", "font-mono")}
                                value={(*scheduler_min).clone()}
                                oninput={{
                                    let scheduler_min = scheduler_min.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        scheduler_min.set(input.value());
                                    })
                                }}
                            />
                        </label>
                        <label class={classes!("text-sm")}>
                            <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Min Remaining Credits" }</div>
                            <input
                                class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm", "font-mono")}
                                value={(*minimum_remaining_credits_before_block).clone()}
                                oninput={{
                                    let minimum_remaining_credits_before_block =
                                        minimum_remaining_credits_before_block.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        minimum_remaining_credits_before_block.set(input.value());
                                    })
                                }}
                            />
                        </label>
                        <label class={classes!("text-sm")}>
                            <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Manual Usage Limit" }</div>
                            <input
                                class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm", "font-mono")}
                                value={(*manual_usage_limit).clone()}
                                placeholder={"upstream"}
                                oninput={{
                                    let manual_usage_limit = manual_usage_limit.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        manual_usage_limit.set(input.value());
                                    })
                                }}
                            />
                        </label>
                        <label class={classes!("text-sm")}>
                            <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Pool Strategy" }</div>
                            <select
                                class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm")}
                                value={(*pool_strategy).clone()}
                                onchange={{
                                    let pool_strategy = pool_strategy.clone();
                                    Callback::from(move |event: Event| {
                                        if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                            pool_strategy.set(target.value());
                                        }
                                    })
                                }}
                            >
                                { kiro_pool_strategy_options((*pool_strategy).as_str()) }
                            </select>
                            <div class={classes!("mt-1", "text-[11px]", "text-[var(--muted)]")}>
                                { kiro_pool_strategy_description((*pool_strategy).as_str()) }
                            </div>
                        </label>
                        <label class={classes!("text-sm")}>
                            <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Proxy Mode" }</div>
                            <select
                                class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm")}
                                value={(*selected_proxy).clone()}
                                onchange={{
                                    let selected_proxy = selected_proxy.clone();
                                    Callback::from(move |event: Event| {
                                        let input: HtmlSelectElement = event.target_unchecked_into();
                                        selected_proxy.set(input.value());
                                    })
                                }}
                            >
                                <option value="inherit" selected={*selected_proxy == "inherit"}>{ "Inherit Provider Proxy" }</option>
                                <option value="direct" selected={*selected_proxy == "direct"}>{ "Direct / No Proxy" }</option>
                                { for props.proxy_configs.iter().map(|proxy_config| {
                                    let option_value = format!("fixed:{}", proxy_config.id);
                                    html! {
                                        <option value={option_value.clone()} selected={*selected_proxy == option_value}>
                                            { format!("Fixed · {} · {}", proxy_config.name, proxy_config.proxy_url) }
                                        </option>
                                    }
                                }) }
                            </select>
                        </label>
                    </div>
                    <div class={classes!("mt-3", "flex", "items-center", "gap-3", "flex-wrap")}>
                        <button type="button" class={classes!("btn-terminal", "btn-terminal-primary")} onclick={on_save_scheduler} disabled={*busy}>
                            { if *busy { "Saving..." } else { "Save Account Settings" } }
                        </button>
                        <span class={classes!("text-xs", "text-[var(--muted)]")}>
                            { "并发、起步间隔、剩余积分阈值和账号级 proxy 选择一起保存。阈值为 0 表示保持原行为，只在 remaining <= 0 时停用账号。" }
                        </span>
                    </div>
                </div>
            }

            if let Some(message) = (*feedback).clone() {
                <div class={classes!("mt-3", "text-sm", "text-[var(--muted)]")}>{ message }</div>
            }
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct KiroKeyEditorCardProps {
    pub(crate) key_item: AdminLlmGatewayKeyView,
    pub(crate) persisted_global_policy_form: KiroCachePolicyForm,
    pub(crate) persisted_global_billable_multiplier_json: String,
    pub(crate) available_models: Vec<KiroModelView>,
    pub(crate) account_groups: Vec<AdminAccountGroupOptionView>,
    pub(crate) anthropic_channels: Vec<AdminAnthropicUpstreamChannelView>,
    pub(crate) on_reload: Callback<()>,
    pub(crate) on_copy: Callback<(String, String)>,
    pub(crate) on_flash: Callback<(String, bool)>,
}

#[function_component(KiroKeyEditorCard)]
pub(crate) fn kiro_key_editor_card(props: &KiroKeyEditorCardProps) -> Html {
    let initial_effective_policy_form_result =
        parse_kiro_cache_policy_form_json(&props.key_item.effective_kiro_cache_policy_json);
    let effective_policy_parse_error = initial_effective_policy_form_result.as_ref().err().cloned();
    let initial_effective_policy_form = initial_effective_policy_form_result
        .clone()
        .unwrap_or_else(|_| KiroCachePolicyForm::default());
    let initial_override_enabled = !props.key_item.uses_global_kiro_cache_policy;
    let initial_effective_billable_multiplier_json = format_json_for_textarea(
        &props
            .key_item
            .effective_kiro_billable_model_multipliers_json,
    );
    let effective_billable_multiplier_parse_error = parse_kiro_billable_multiplier_json_with_base(
        &props
            .key_item
            .effective_kiro_billable_model_multipliers_json,
        &default_kiro_billable_multiplier_map(),
    )
    .err();
    let initial_billable_multiplier_override_enabled =
        !props.key_item.uses_global_kiro_billable_model_multipliers;
    let name = use_state(|| props.key_item.name.clone());
    let quota = use_state(|| props.key_item.quota_billable_limit.to_string());
    let status = use_state(|| props.key_item.status.clone());
    let route_strategy = use_state(|| {
        props
            .key_item
            .route_strategy
            .clone()
            .unwrap_or_else(|| "auto".to_string())
    });
    let account_group_id = use_state(|| {
        sanitize_kiro_account_group_id(
            props.key_item.account_group_id.as_deref(),
            &props.account_groups,
            true,
        )
    });
    let preferred_pool_strategy = use_state(|| props.key_item.preferred_pool_strategy.clone());
    let anthropic_upstream_pool_mode =
        use_state(|| props.key_item.kiro_anthropic_upstream_pool_mode.clone());
    let model_name_map = use_state(|| props.key_item.model_name_map.clone().unwrap_or_default());
    let model_group_preferences = use_state(|| props.key_item.kiro_model_group_preferences.clone());
    let model_channel_preferences =
        use_state(|| props.key_item.kiro_model_channel_preferences.clone());
    let moderation_enabled = use_state(|| props.key_item.moderation_enabled);
    let kiro_request_validation_enabled =
        use_state(|| props.key_item.kiro_request_validation_enabled);
    let kiro_cache_estimation_enabled = use_state(|| props.key_item.kiro_cache_estimation_enabled);
    let kiro_zero_cache_debug_enabled = use_state(|| props.key_item.kiro_zero_cache_debug_enabled);
    let kiro_full_request_logging_enabled =
        use_state(|| props.key_item.kiro_full_request_logging_enabled);
    let kiro_remote_media_resolution_enabled =
        use_state(|| props.key_item.kiro_remote_media_resolution_enabled);
    let kiro_latency_routing_enabled = use_state(|| props.key_item.kiro_latency_routing_enabled);
    let kiro_protected_content_validation_enabled =
        use_state(|| props.key_item.kiro_protected_content_validation_enabled);
    let kiro_cctest_text_handling_enabled =
        use_state(|| props.key_item.kiro_cctest_text_handling_enabled);
    let policy_override_enabled = use_state(|| initial_override_enabled);
    let key_policy_form = use_state(|| initial_effective_policy_form.clone());
    let key_policy_effective_baseline = use_state(|| initial_effective_policy_form.clone());
    let billable_multiplier_override_enabled =
        use_state(|| initial_billable_multiplier_override_enabled);
    let key_billable_multiplier_json =
        use_state(|| initial_effective_billable_multiplier_json.clone());
    let key_billable_multiplier_effective_baseline =
        use_state(|| initial_effective_billable_multiplier_json.clone());
    let billable_multiplier_settings_expanded = use_state(|| false);
    let route_settings_expanded = use_state(|| false);
    let model_mapping_expanded = use_state(|| false);
    let model_group_routing_open = use_state(|| false);
    let saving = use_state(|| false);
    let feedback = use_state(|| None::<String>);

    {
        let key_item = props.key_item.clone();
        let account_groups = props.account_groups.clone();
        let initial_effective_policy_form = initial_effective_policy_form.clone();
        let name = name.clone();
        let quota = quota.clone();
        let status = status.clone();
        let route_strategy = route_strategy.clone();
        let account_group_id = account_group_id.clone();
        let preferred_pool_strategy = preferred_pool_strategy.clone();
        let anthropic_upstream_pool_mode = anthropic_upstream_pool_mode.clone();
        let model_name_map = model_name_map.clone();
        let model_group_preferences = model_group_preferences.clone();
        let model_channel_preferences = model_channel_preferences.clone();
        let moderation_enabled = moderation_enabled.clone();
        let kiro_request_validation_enabled = kiro_request_validation_enabled.clone();
        let kiro_cache_estimation_enabled = kiro_cache_estimation_enabled.clone();
        let kiro_zero_cache_debug_enabled = kiro_zero_cache_debug_enabled.clone();
        let kiro_full_request_logging_enabled = kiro_full_request_logging_enabled.clone();
        let kiro_remote_media_resolution_enabled = kiro_remote_media_resolution_enabled.clone();
        let kiro_latency_routing_enabled = kiro_latency_routing_enabled.clone();
        let kiro_protected_content_validation_enabled =
            kiro_protected_content_validation_enabled.clone();
        let kiro_cctest_text_handling_enabled = kiro_cctest_text_handling_enabled.clone();
        let policy_override_enabled = policy_override_enabled.clone();
        let key_policy_form = key_policy_form.clone();
        let key_policy_effective_baseline = key_policy_effective_baseline.clone();
        let billable_multiplier_override_enabled = billable_multiplier_override_enabled.clone();
        let key_billable_multiplier_json = key_billable_multiplier_json.clone();
        let key_billable_multiplier_effective_baseline =
            key_billable_multiplier_effective_baseline.clone();
        let billable_multiplier_settings_expanded = billable_multiplier_settings_expanded.clone();
        let model_group_routing_open = model_group_routing_open.clone();
        let initial_effective_billable_multiplier_json_for_effect =
            initial_effective_billable_multiplier_json.clone();
        use_effect_with((props.key_item.clone(), props.account_groups.clone()), move |_| {
            name.set(key_item.name.clone());
            quota.set(key_item.quota_billable_limit.to_string());
            status.set(key_item.status.clone());
            route_strategy.set(
                key_item
                    .route_strategy
                    .clone()
                    .unwrap_or_else(|| "auto".to_string()),
            );
            account_group_id.set(sanitize_kiro_account_group_id(
                key_item.account_group_id.as_deref(),
                &account_groups,
                true,
            ));
            preferred_pool_strategy.set(key_item.preferred_pool_strategy.clone());
            anthropic_upstream_pool_mode.set(key_item.kiro_anthropic_upstream_pool_mode.clone());
            model_name_map.set(key_item.model_name_map.clone().unwrap_or_default());
            model_group_preferences.set(key_item.kiro_model_group_preferences.clone());
            model_channel_preferences.set(key_item.kiro_model_channel_preferences.clone());
            moderation_enabled.set(key_item.moderation_enabled);
            kiro_request_validation_enabled.set(key_item.kiro_request_validation_enabled);
            kiro_cache_estimation_enabled.set(key_item.kiro_cache_estimation_enabled);
            kiro_zero_cache_debug_enabled.set(key_item.kiro_zero_cache_debug_enabled);
            kiro_full_request_logging_enabled.set(key_item.kiro_full_request_logging_enabled);
            kiro_remote_media_resolution_enabled.set(key_item.kiro_remote_media_resolution_enabled);
            kiro_latency_routing_enabled.set(key_item.kiro_latency_routing_enabled);
            kiro_protected_content_validation_enabled
                .set(key_item.kiro_protected_content_validation_enabled);
            kiro_cctest_text_handling_enabled.set(key_item.kiro_cctest_text_handling_enabled);
            policy_override_enabled.set(initial_override_enabled);
            key_policy_form.set(initial_effective_policy_form.clone());
            key_policy_effective_baseline.set(initial_effective_policy_form.clone());
            billable_multiplier_override_enabled.set(initial_billable_multiplier_override_enabled);
            key_billable_multiplier_json
                .set(initial_effective_billable_multiplier_json_for_effect.clone());
            key_billable_multiplier_effective_baseline
                .set(initial_effective_billable_multiplier_json_for_effect.clone());
            billable_multiplier_settings_expanded.set(false);
            model_group_routing_open.set(false);
            || ()
        });
    }

    let on_save = {
        let key_id = props.key_item.id.clone();
        let key_name = props.key_item.name.clone();
        let persisted_global_policy_form = props.persisted_global_policy_form.clone();
        let persisted_global_billable_multiplier_json =
            props.persisted_global_billable_multiplier_json.clone();
        let initial_effective_policy_form = initial_effective_policy_form.clone();
        let has_effective_policy_parse_error = effective_policy_parse_error.is_some();
        let initial_effective_billable_multiplier_json =
            initial_effective_billable_multiplier_json.clone();
        let has_effective_billable_multiplier_parse_error =
            effective_billable_multiplier_parse_error.is_some();
        let name = name.clone();
        let quota = quota.clone();
        let status = status.clone();
        let route_strategy = route_strategy.clone();
        let account_group_id = account_group_id.clone();
        let preferred_pool_strategy = preferred_pool_strategy.clone();
        let anthropic_upstream_pool_mode = anthropic_upstream_pool_mode.clone();
        let model_name_map = model_name_map.clone();
        let model_group_preferences = model_group_preferences.clone();
        let model_channel_preferences = model_channel_preferences.clone();
        let moderation_enabled = moderation_enabled.clone();
        let kiro_request_validation_enabled = kiro_request_validation_enabled.clone();
        let kiro_cache_estimation_enabled = kiro_cache_estimation_enabled.clone();
        let kiro_zero_cache_debug_enabled = kiro_zero_cache_debug_enabled.clone();
        let kiro_full_request_logging_enabled = kiro_full_request_logging_enabled.clone();
        let kiro_remote_media_resolution_enabled = kiro_remote_media_resolution_enabled.clone();
        let kiro_latency_routing_enabled = kiro_latency_routing_enabled.clone();
        let kiro_protected_content_validation_enabled =
            kiro_protected_content_validation_enabled.clone();
        let kiro_cctest_text_handling_enabled = kiro_cctest_text_handling_enabled.clone();
        let policy_override_enabled = policy_override_enabled.clone();
        let key_policy_form = key_policy_form.clone();
        let billable_multiplier_override_enabled = billable_multiplier_override_enabled.clone();
        let key_billable_multiplier_json = key_billable_multiplier_json.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            let key_id = key_id.clone();
            let key_name = key_name.clone();
            let persisted_global_policy_form = persisted_global_policy_form.clone();
            let persisted_global_billable_multiplier_json =
                persisted_global_billable_multiplier_json.clone();
            let initial_effective_policy_form = initial_effective_policy_form.clone();
            let initial_effective_billable_multiplier_json =
                initial_effective_billable_multiplier_json.clone();
            let name_value = (*name).clone();
            let quota_value = (*quota).clone();
            let status_value = (*status).clone();
            let route_strategy_value = (*route_strategy).clone();
            let account_group_id_value = (*account_group_id).clone();
            let preferred_pool_strategy_value = (*preferred_pool_strategy).clone();
            let anthropic_upstream_pool_mode_value = (*anthropic_upstream_pool_mode).clone();
            let model_name_map_value = (*model_name_map).clone();
            let model_group_preferences_value = (*model_group_preferences).clone();
            let model_channel_preferences_value = (*model_channel_preferences).clone();
            let moderation_enabled_value = *moderation_enabled;
            let kiro_request_validation_enabled_value = *kiro_request_validation_enabled;
            let kiro_cache_estimation_enabled_value = *kiro_cache_estimation_enabled;
            let kiro_zero_cache_debug_enabled_value = *kiro_zero_cache_debug_enabled;
            let kiro_full_request_logging_enabled_value = *kiro_full_request_logging_enabled;
            let kiro_remote_media_resolution_enabled_value = *kiro_remote_media_resolution_enabled;
            let kiro_latency_routing_enabled_value = *kiro_latency_routing_enabled;
            let kiro_protected_content_validation_enabled_value =
                *kiro_protected_content_validation_enabled;
            let kiro_cctest_text_handling_enabled_value = *kiro_cctest_text_handling_enabled;
            let policy_override_enabled_value = *policy_override_enabled;
            let key_policy_form_value = (*key_policy_form).clone();
            let billable_multiplier_override_enabled_value = *billable_multiplier_override_enabled;
            let key_billable_multiplier_json_value = (*key_billable_multiplier_json).clone();
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_flash = on_flash.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let parsed_quota = match quota_value.trim().parse::<u64>() {
                    Ok(value) => value,
                    Err(_) => {
                        let message = "Quota must be a valid integer.".to_string();
                        feedback.set(Some(message.clone()));
                        on_flash.emit((message, true));
                        return;
                    },
                };
                let policy_override_json = if has_effective_policy_parse_error {
                    None
                } else {
                    match build_kiro_cache_policy_override_patch(
                        &persisted_global_policy_form,
                        initial_override_enabled,
                        &initial_effective_policy_form,
                        policy_override_enabled_value,
                        &key_policy_form_value,
                    ) {
                        Ok(value) => value,
                        Err(err) => {
                            feedback.set(Some(err.clone()));
                            on_flash.emit((err, true));
                            return;
                        },
                    }
                };
                let billable_multiplier_override_json =
                    if has_effective_billable_multiplier_parse_error {
                        None
                    } else {
                        match build_kiro_billable_multiplier_override_patch(
                            &persisted_global_billable_multiplier_json,
                            initial_billable_multiplier_override_enabled,
                            &initial_effective_billable_multiplier_json,
                            billable_multiplier_override_enabled_value,
                            &key_billable_multiplier_json_value,
                        ) {
                            Ok(value) => value,
                            Err(err) => {
                                feedback.set(Some(err.clone()));
                                on_flash.emit((err, true));
                                return;
                            },
                        }
                    };
                saving.set(true);
                feedback.set(None);
                match patch_admin_kiro_key(&key_id, PatchAdminLlmGatewayKeyRequest {
                    name: Some(name_value.trim()),
                    status: Some(status_value.trim()),
                    public_visible: None,
                    quota_billable_limit: Some(parsed_quota),
                    route_strategy: Some(route_strategy_value.as_str()),
                    account_group_id: Some(account_group_id_value.as_str()),
                    fixed_account_name: None,
                    auto_account_names: None,
                    preferred_pool_strategy: Some(preferred_pool_strategy_value.as_str()),
                    kiro_anthropic_upstream_pool_mode: Some(
                        anthropic_upstream_pool_mode_value.as_str(),
                    ),
                    model_name_map: Some(&model_name_map_value),
                    kiro_model_group_preferences: Some(&model_group_preferences_value),
                    kiro_model_channel_preferences: Some(&model_channel_preferences_value),
                    request_max_concurrency: None,
                    request_min_start_interval_ms: None,
                    moderation_enabled: Some(moderation_enabled_value),
                    codex_fast_enabled: None,
                    codex_responses_lite_enabled: None,
                    codex_strict_session_rejection_enabled: None,
                    codex_image_generation_enabled: None,
                    codex_image_standalone_generation_enabled: None,
                    codex_image_direct_generation_enabled: None,
                    kiro_request_validation_enabled: Some(kiro_request_validation_enabled_value),
                    kiro_cache_estimation_enabled: Some(kiro_cache_estimation_enabled_value),
                    kiro_zero_cache_debug_enabled: Some(kiro_zero_cache_debug_enabled_value),
                    kiro_full_request_logging_enabled: Some(
                        kiro_full_request_logging_enabled_value,
                    ),
                    kiro_remote_media_resolution_enabled: Some(
                        kiro_remote_media_resolution_enabled_value,
                    ),
                    kiro_latency_routing_enabled: Some(kiro_latency_routing_enabled_value),
                    kiro_protected_content_validation_enabled: Some(
                        kiro_protected_content_validation_enabled_value,
                    ),
                    kiro_cctest_text_handling_enabled: Some(
                        kiro_cctest_text_handling_enabled_value,
                    ),
                    kiro_cache_policy_override_json: policy_override_json
                        .as_ref()
                        .map(|value| value.as_deref()),
                    kiro_billable_model_multipliers_override_json:
                        billable_multiplier_override_json
                            .as_ref()
                            .map(|value| value.as_deref()),
                    request_max_concurrency_unlimited: false,
                    request_min_start_interval_ms_unlimited: false,
                })
                .await
                {
                    Ok(_) => {
                        feedback.set(Some("Saved.".to_string()));
                        on_flash.emit((format!("Saved Kiro key `{key_name}`."), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash
                            .emit((format!("Failed to save Kiro key `{key_name}`.\n{err}"), true));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_disable = {
        let key_id = props.key_item.id.clone();
        let key_name = props.key_item.name.clone();
        let name = name.clone();
        let quota = quota.clone();
        let route_strategy = route_strategy.clone();
        let account_group_id = account_group_id.clone();
        let preferred_pool_strategy = preferred_pool_strategy.clone();
        let anthropic_upstream_pool_mode = anthropic_upstream_pool_mode.clone();
        let model_name_map = model_name_map.clone();
        let model_group_preferences = model_group_preferences.clone();
        let model_channel_preferences = model_channel_preferences.clone();
        let kiro_cache_estimation_enabled = kiro_cache_estimation_enabled.clone();
        let kiro_zero_cache_debug_enabled = kiro_zero_cache_debug_enabled.clone();
        let kiro_full_request_logging_enabled = kiro_full_request_logging_enabled.clone();
        let kiro_remote_media_resolution_enabled = kiro_remote_media_resolution_enabled.clone();
        let kiro_latency_routing_enabled = kiro_latency_routing_enabled.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            let key_id = key_id.clone();
            let key_name = key_name.clone();
            let name_value = (*name).clone();
            let quota_value = (*quota).clone();
            let route_strategy_value = (*route_strategy).clone();
            let account_group_id_value = (*account_group_id).clone();
            let preferred_pool_strategy_value = (*preferred_pool_strategy).clone();
            let anthropic_upstream_pool_mode_value = (*anthropic_upstream_pool_mode).clone();
            let model_name_map_value = (*model_name_map).clone();
            let model_group_preferences_value = (*model_group_preferences).clone();
            let model_channel_preferences_value = (*model_channel_preferences).clone();
            let kiro_cache_estimation_enabled_value = *kiro_cache_estimation_enabled;
            let kiro_zero_cache_debug_enabled_value = *kiro_zero_cache_debug_enabled;
            let kiro_full_request_logging_enabled_value = *kiro_full_request_logging_enabled;
            let kiro_remote_media_resolution_enabled_value = *kiro_remote_media_resolution_enabled;
            let kiro_latency_routing_enabled_value = *kiro_latency_routing_enabled;
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_flash = on_flash.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let parsed_quota = match quota_value.trim().parse::<u64>() {
                    Ok(value) => value,
                    Err(_) => {
                        let message = "Quota must be a valid integer.".to_string();
                        feedback.set(Some(message.clone()));
                        on_flash.emit((message, true));
                        return;
                    },
                };
                saving.set(true);
                feedback.set(None);
                match patch_admin_kiro_key(&key_id, PatchAdminLlmGatewayKeyRequest {
                    name: Some(name_value.trim()),
                    status: Some("disabled"),
                    public_visible: None,
                    quota_billable_limit: Some(parsed_quota),
                    route_strategy: Some(route_strategy_value.as_str()),
                    account_group_id: Some(account_group_id_value.as_str()),
                    fixed_account_name: None,
                    auto_account_names: None,
                    preferred_pool_strategy: Some(preferred_pool_strategy_value.as_str()),
                    kiro_anthropic_upstream_pool_mode: Some(
                        anthropic_upstream_pool_mode_value.as_str(),
                    ),
                    model_name_map: Some(&model_name_map_value),
                    kiro_model_group_preferences: Some(&model_group_preferences_value),
                    kiro_model_channel_preferences: Some(&model_channel_preferences_value),
                    request_max_concurrency: None,
                    request_min_start_interval_ms: None,
                    moderation_enabled: None,
                    codex_fast_enabled: None,
                    codex_responses_lite_enabled: None,
                    codex_strict_session_rejection_enabled: None,
                    codex_image_generation_enabled: None,
                    codex_image_standalone_generation_enabled: None,
                    codex_image_direct_generation_enabled: None,
                    kiro_request_validation_enabled: None,
                    kiro_cache_estimation_enabled: Some(kiro_cache_estimation_enabled_value),
                    kiro_zero_cache_debug_enabled: Some(kiro_zero_cache_debug_enabled_value),
                    kiro_full_request_logging_enabled: Some(
                        kiro_full_request_logging_enabled_value,
                    ),
                    kiro_remote_media_resolution_enabled: Some(
                        kiro_remote_media_resolution_enabled_value,
                    ),
                    kiro_latency_routing_enabled: Some(kiro_latency_routing_enabled_value),
                    kiro_protected_content_validation_enabled: None,
                    kiro_cctest_text_handling_enabled: None,
                    kiro_cache_policy_override_json: None,
                    kiro_billable_model_multipliers_override_json: None,
                    request_max_concurrency_unlimited: false,
                    request_min_start_interval_ms_unlimited: false,
                })
                .await
                {
                    Ok(_) => {
                        feedback.set(Some("Disabled.".to_string()));
                        on_flash.emit((format!("Disabled Kiro key `{key_name}`."), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((
                            format!("Failed to disable Kiro key `{key_name}`.\n{err}"),
                            true,
                        ));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_delete = {
        let key_id = props.key_item.id.clone();
        let key_name = props.key_item.name.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            if !confirm_destructive(&format!("确认删除 Kiro key `{}` ？此操作不可撤销。", key_name))
            {
                return;
            }
            let key_id = key_id.clone();
            let key_name = key_name.clone();
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_flash = on_flash.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                feedback.set(None);
                match delete_admin_kiro_key(&key_id).await {
                    Ok(_) => {
                        feedback.set(Some("Deleted.".to_string()));
                        on_flash.emit((format!("Deleted Kiro key `{key_name}`."), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((
                            format!("Failed to delete Kiro key `{key_name}`.\n{err}"),
                            true,
                        ));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_reset_model_map = {
        let model_name_map = model_name_map.clone();
        Callback::from(move |_| model_name_map.set(BTreeMap::new()))
    };
    let on_reset_model_routing_preferences = {
        let model_group_preferences = model_group_preferences.clone();
        let model_channel_preferences = model_channel_preferences.clone();
        Callback::from(move |_| {
            model_group_preferences.set(BTreeMap::new());
            model_channel_preferences.set(BTreeMap::new());
        })
    };
    let on_save_model_routing_preferences = {
        let key_id = props.key_item.id.clone();
        let key_name = props.key_item.name.clone();
        let model_group_preferences = model_group_preferences.clone();
        let model_channel_preferences = model_channel_preferences.clone();
        let model_group_routing_open = model_group_routing_open.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |_| {
            if *saving {
                return;
            }
            let key_id = key_id.clone();
            let key_name = key_name.clone();
            let group_preferences_value = (*model_group_preferences).clone();
            let channel_preferences_value = (*model_channel_preferences).clone();
            let model_group_routing_open = model_group_routing_open.clone();
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_flash = on_flash.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                feedback.set(None);
                match patch_admin_kiro_key(&key_id, PatchAdminLlmGatewayKeyRequest {
                    kiro_model_group_preferences: Some(&group_preferences_value),
                    kiro_model_channel_preferences: Some(&channel_preferences_value),
                    ..PatchAdminLlmGatewayKeyRequest::default()
                })
                .await
                {
                    Ok(_) => {
                        feedback.set(Some("Saved model routing.".to_string()));
                        model_group_routing_open.set(false);
                        on_flash.emit((format!("Saved model routing for `{key_name}`."), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((
                            format!("Failed to save model routing for `{key_name}`.\n{err}"),
                            true,
                        ));
                    },
                }
                saving.set(false);
            });
        })
    };
    let on_restore_inherit = {
        let key_id = props.key_item.id.clone();
        let key_name = props.key_item.name.clone();
        let key_policy_form = key_policy_form.clone();
        let key_policy_effective_baseline = key_policy_effective_baseline.clone();
        let policy_override_enabled = policy_override_enabled.clone();
        let saving = saving.clone();
        let feedback = feedback.clone();
        let on_flash = props.on_flash.clone();
        let on_reload = props.on_reload.clone();
        let persisted_global_policy_form = props.persisted_global_policy_form.clone();
        Callback::from(move |_| {
            if *saving {
                return;
            }
            let key_id = key_id.clone();
            let key_name = key_name.clone();
            let key_policy_form = key_policy_form.clone();
            let key_policy_effective_baseline = key_policy_effective_baseline.clone();
            let policy_override_enabled = policy_override_enabled.clone();
            let saving = saving.clone();
            let feedback = feedback.clone();
            let on_flash = on_flash.clone();
            let on_reload = on_reload.clone();
            let persisted_global_policy_form = persisted_global_policy_form.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving.set(true);
                feedback.set(None);
                match patch_admin_kiro_key(&key_id, PatchAdminLlmGatewayKeyRequest {
                    kiro_cache_policy_override_json: Some(None),
                    ..PatchAdminLlmGatewayKeyRequest::default()
                })
                .await
                {
                    Ok(_) => {
                        key_policy_effective_baseline.set(persisted_global_policy_form.clone());
                        policy_override_enabled.set(false);
                        key_policy_form.set(persisted_global_policy_form.clone());
                        feedback.set(Some("Restored inherit.".to_string()));
                        on_flash
                            .emit((format!("Restored inherit for Kiro key `{key_name}`."), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        feedback.set(Some(err.clone()));
                        on_flash.emit((
                            format!("Failed to restore inherit for Kiro key `{key_name}`.\n{err}"),
                            true,
                        ));
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
    let route_summary = kiro_key_route_summary(
        (*route_strategy).as_str(),
        (*account_group_id).as_str(),
        (*preferred_pool_strategy).as_str(),
        props.account_groups.as_slice(),
    );
    let candidate_credit_summary = props
        .key_item
        .kiro_candidate_credit_summary
        .unwrap_or_default();
    let candidate_credit_summary_text = {
        let base = format_kiro_key_candidate_credit_summary(&candidate_credit_summary);
        match kiro_preferred_pool_candidate_note(
            (*route_strategy).as_str(),
            (*preferred_pool_strategy).as_str(),
            &candidate_credit_summary,
        ) {
            Some(note) => format!("{base} · {note}"),
            None => base,
        }
    };
    let preferred_pool_warning = kiro_preferred_pool_warning(
        (*route_strategy).as_str(),
        (*preferred_pool_strategy).as_str(),
        &candidate_credit_summary,
    );
    let global_policy_summary = format_kiro_cache_policy_summary(
        &props.persisted_global_policy_form,
        &props.persisted_global_policy_form,
    );
    let displayed_effective_policy_form = if *policy_override_enabled {
        (*key_policy_form).clone()
    } else {
        props.persisted_global_policy_form.clone()
    };
    let effective_policy_summary = effective_policy_parse_error
        .as_ref()
        .map(|err| format!("invalid backend policy json · {err}"))
        .unwrap_or_else(|| {
            format_effective_kiro_cache_policy_summary(
                !*policy_override_enabled,
                &displayed_effective_policy_form,
            )
        });
    let policy_controls_disabled = effective_policy_parse_error.is_some() || *saving;
    let global_billable_multiplier_summary = format_kiro_billable_multiplier_summary(
        true,
        &props.persisted_global_billable_multiplier_json,
    );
    let displayed_effective_billable_multiplier_json = if *billable_multiplier_override_enabled {
        (*key_billable_multiplier_json).clone()
    } else {
        props.persisted_global_billable_multiplier_json.clone()
    };
    let effective_billable_multiplier_summary = effective_billable_multiplier_parse_error
        .as_ref()
        .map(|err| format!("invalid backend multiplier json · {err}"))
        .unwrap_or_else(|| {
            format_kiro_billable_multiplier_summary(
                !*billable_multiplier_override_enabled,
                &displayed_effective_billable_multiplier_json,
            )
        });
    let billable_multiplier_controls_disabled =
        effective_billable_multiplier_parse_error.is_some() || *saving;

    let key_ratio = kiro_key_usage_ratio(
        props.key_item.remaining_billable,
        props.key_item.quota_billable_limit,
    );
    let key_pct = (key_ratio * 100.0).round() as i32;
    let model_override_count = (*model_name_map).len();
    let mapping_overrides_preview = if (*model_name_map).is_empty() {
        "identity map".to_string()
    } else {
        (*model_name_map)
            .iter()
            .map(|(source, target)| format!("{source} -> {target}"))
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let model_routing_summary = kiro_model_routing_summary(
        &model_group_preferences,
        &model_channel_preferences,
        props.account_groups.as_slice(),
    );
    html! {
        <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
            <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                <div>
                    <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                        <span class={kiro_badge()}>
                            { "Kiro" }
                        </span>
                        <h3 class={classes!("m-0", "text-base", "font-semibold")}>{ props.key_item.name.clone() }</h3>
                    </div>
                    <p class={classes!("mt-2", "mb-0", "text-xs", "font-mono", "text-[var(--muted)]")}>
                        { format!("{} · remaining {}", props.key_item.status, format_number_i64(props.key_item.remaining_billable)) }
                    </p>
                    <p class={classes!("mt-1", "mb-0", "text-xs", "font-mono", "text-[var(--muted)]")}>
                        { format!("credits {}", format_float4(props.key_item.usage_credit_total)) }
                        if props.key_item.usage_credit_missing_events > 0 {
                            { format!(" · partial ({} missing)", props.key_item.usage_credit_missing_events) }
                        }
                    </p>
                </div>
                <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                    <span class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>
                        { format!("created {} · used {}", format_ms(props.key_item.created_at), format_timestamp_opt(props.key_item.last_used_at)) }
                    </span>
                    if *route_strategy != "fixed" {
                        <span class={classes!("rounded-full", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-1.5", "text-xs", "font-mono", "text-[var(--text)]")}>
                            { format!("优先池 {}", kiro_pool_strategy_label((*preferred_pool_strategy).as_str())) }
                        </span>
                    }
                    <span
                        class={classes!("rounded-full", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-1.5", "text-xs", "font-mono", "text-[var(--text)]")}
                        title={anthropic_upstream_pool_mode_description((*anthropic_upstream_pool_mode).as_str())}
                    >
                        { format!("Anthropic 直连 {}", anthropic_upstream_pool_mode_label((*anthropic_upstream_pool_mode).as_str())) }
                    </span>
                    <button
                        type="button"
                        class={classes!("btn-terminal", "text-xs")}
                        onclick={{
                            let on_reload = props.on_reload.clone();
                            Callback::from(move |_| on_reload.emit(()))
                        }}
                    >
                        { "Refresh" }
                    </button>
                </div>
            </div>

            <div class={classes!("mt-3")}>
                <div class={classes!("flex", "items-center", "justify-between", "font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>
                    <span>{ "用量" }</span>
                    <span>{ format!("{key_pct}%") }</span>
                </div>
                <div class={classes!("mt-1.5", "h-2", "overflow-hidden", "rounded-full", "bg-[var(--surface-alt)]")}>
                    <div class={classes!("h-full", "rounded-full", "bg-[linear-gradient(90deg,#0f766e,#2563eb)]", "transition-[width]", "duration-300")}
                         style={format!("width: {}%;", key_pct.clamp(0, 100))} />
                </div>
                <div class={classes!("mt-2", "flex", "items-center", "gap-4", "font-mono", "text-[11px]", "text-[var(--muted)]")}>
                    <span>{ format!("remaining {}", format_number_i64(props.key_item.remaining_billable)) }</span>
                    <span>{ format!("limit {}", format_number_u64(props.key_item.quota_billable_limit)) }</span>
                    <span>{ format!("输入 {}", format_number_u64(props.key_item.usage_input_uncached_tokens)) }</span>
                    <span>{ format!("缓存 {}", format_number_u64(props.key_item.usage_input_cached_tokens)) }</span>
                    <span>{ format!("输出 {}", format_number_u64(props.key_item.usage_output_tokens)) }</span>
                </div>
                <div class={classes!("mt-2", "font-mono", "text-[11px]", "text-[var(--muted)]")}>
                    { candidate_credit_summary_text }
                </div>
                if let Some(message) = preferred_pool_warning.clone() {
                    <div class={classes!("mt-2", "rounded-lg", "border", "border-amber-300/40", "bg-amber-500/10", "px-3", "py-2", "text-xs", "text-amber-200")}>
                        { message }
                    </div>
                }
            </div>

            <div class={classes!("mt-4", "grid", "gap-3", "md:grid-cols-2")}>
                <div class={classes!("md:col-span-2", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Secret" }</div>
                    <MaskedSecretCode
                        value={props.key_item.secret.clone()}
                        copy_label={"Kiro Key"}
                        on_copy={props.on_copy.clone()}
                        code_class={classes!("leading-6", "text-[var(--text)]")}
                    />
                </div>
                <label class={classes!("text-sm")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Name" }</div>
                    <input
                        class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "text-sm")}
                        value={(*name).clone()}
                        oninput={{
                            let name = name.clone();
                            Callback::from(move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                name.set(input.value());
                            })
                        }}
                    />
                </label>
                <label class={classes!("text-sm")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Quota" }</div>
                    <input
                        class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "text-sm", "font-mono")}
                        value={(*quota).clone()}
                        oninput={{
                            let quota = quota.clone();
                            Callback::from(move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                quota.set(input.value());
                            })
                        }}
                    />
                </label>
                <label class={classes!("text-sm")}>
                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Status" }</div>
                    <select
                        key={format!("kiro-key-status-{}-{}", props.key_item.id, (*status).clone())}
                        class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "text-sm")}
                        value={(*status).clone()}
                        onchange={{
                            let status = status.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlSelectElement = event.target_unchecked_into();
                                status.set(input.value());
                            })
                        }}
                    >
                        <option value="active" selected={(*status).as_str() == "active"}>{ "active" }</option>
                        <option value="disabled" selected={(*status).as_str() == "disabled"}>{ "disabled" }</option>
                    </select>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*moderation_enabled}
                        onchange={{
                            let moderation_enabled = moderation_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                moderation_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "审核拦截" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            {
                                if *moderation_enabled {
                                    "ON · 命中关键词会封禁该 key 的当前 session，并返回可定位的 review id；解封后同 session 的同一匹配内容不会再次封禁。"
                                } else {
                                    "OFF · 仅此 key 跳过审核拦截；历史记录保留，重新开启后已有 active ban 继续生效。"
                                }
                            }
                        </span>
                    </span>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*kiro_request_validation_enabled}
                        onchange={{
                            let kiro_request_validation_enabled =
                                kiro_request_validation_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                kiro_request_validation_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "请求合法性校验" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            { "开启时会在转发前拦截明显坏掉的 Anthropic message 结构。空文本占位块现在会自动忽略；如果某个客户端仍被误伤，可以按 key 关闭这层校验。" }
                        </span>
                    </span>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*kiro_remote_media_resolution_enabled}
                        onchange={{
                            let kiro_remote_media_resolution_enabled =
                                kiro_remote_media_resolution_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                kiro_remote_media_resolution_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "URL 媒体代拉" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            { "默认关闭，保持 kiro-gateway 行为。开启后才会把 Anthropic image/document 的 source.type=url 在服务端拉取并转成 base64；关闭时 URL 媒体不会作为附件进入 Kiro，需要图片内容请传 base64/data URL。" }
                        </span>
                    </span>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*kiro_latency_routing_enabled}
                        onchange={{
                            let kiro_latency_routing_enabled =
                                kiro_latency_routing_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                kiro_latency_routing_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "首字延迟自适应选号" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            { "开启后，账号池内的亲和+动态策略会使用近期首字延迟快照作为排序信号；关闭后该 key 回到无延迟快照的轮转顺序。" }
                        </span>
                    </span>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*kiro_protected_content_validation_enabled}
                        onchange={{
                            let kiro_protected_content_validation_enabled =
                                kiro_protected_content_validation_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                kiro_protected_content_validation_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "Thinking 防篡改校验" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            { "开启后，会拒绝无法校验的 encrypted_content、非 thinking 块上的 signature，以及被改动过的 thinking/signature 历史内容。" }
                        </span>
                    </span>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*kiro_cctest_text_handling_enabled}
                        onchange={{
                            let kiro_cctest_text_handling_enabled =
                                kiro_cctest_text_handling_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                kiro_cctest_text_handling_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "cctest 文本专门处理" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            { "默认关闭。开启后只会识别已知 cctest 纯文本探针；多模态和 websearch 请求仍走正常 Kiro 路径，signature 题会转发到配置的专用 Anthropic 上游。" }
                        </span>
                    </span>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*kiro_cache_estimation_enabled}
                        onchange={{
                            let kiro_cache_estimation_enabled =
                                kiro_cache_estimation_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                kiro_cache_estimation_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "Cache Token 估算" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            { "开启时，这个 key 对外返回的 Anthropic usage 会暴露保守估算的 cache_read_input_tokens，同时 usage event 也会记入估算后的 cached / uncached split。关闭后这两处都会回到 cache=0。" }
                        </span>
                    </span>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*kiro_zero_cache_debug_enabled}
                        onchange={{
                            let kiro_zero_cache_debug_enabled =
                                kiro_zero_cache_debug_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                kiro_zero_cache_debug_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "0 Cache 诊断" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            { "开启后，这个 key 的成功请求如果最终 cache_read_input_tokens 为 0，会把完整 client/upstream request 写入 usage event 详情，并在 0-cache 日志里带上完整请求数据。关闭时仍会记录 0-cache 元信息日志，但不会保存完整请求体。" }
                        </span>
                    </span>
                </label>
                <label class={classes!("md:col-span-2", "flex", "cursor-pointer", "items-start", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "toggle-card")}>
                    <input
                        type="checkbox"
                        class={classes!("switch")}
                        checked={*kiro_full_request_logging_enabled}
                        onchange={{
                            let kiro_full_request_logging_enabled =
                                kiro_full_request_logging_enabled.clone();
                            Callback::from(move |event: Event| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                kiro_full_request_logging_enabled.set(input.checked());
                            })
                        }}
                    />
                    <span>
                        <strong>{ "完整请求记录" }</strong>
                        <span class={classes!("block", "mt-1", "text-xs", "text-[var(--muted)]")}>
                            { "默认关闭。开启后，这个 key 的每次 Kiro 请求都会在 usage event 详情里保留完整 client/upstream request，用于协议兼容和异常排查。" }
                        </span>
                    </span>
                </label>
                <div class={classes!("md:col-span-2", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "space-y-3")}>
                    <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                        <div class={classes!("space-y-1")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Cache Policy" }</div>
                            <div class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>
                                { format!("global: {global_policy_summary}") }
                            </div>
                            <div class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>
                                { format!("effective: {effective_policy_summary}") }
                            </div>
                        </div>
                        <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                            <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                                <input
                                    type="checkbox"
                                    class={classes!("switch")}
                                    checked={*policy_override_enabled}
                                    disabled={policy_controls_disabled}
                                    onchange={{
                                        let policy_override_enabled = policy_override_enabled.clone();
                                        let key_policy_form = key_policy_form.clone();
                                        let key_policy_effective_baseline =
                                            key_policy_effective_baseline.clone();
                                        Callback::from(move |event: Event| {
                                            let input: HtmlInputElement = event.target_unchecked_into();
                                            let checked = input.checked();
                                            policy_override_enabled.set(checked);
                                            if checked {
                                                key_policy_form
                                                    .set((*key_policy_effective_baseline).clone());
                                            }
                                        })
                                    }}
                                />
                                <span>{ "Override Global Policy" }</span>
                            </label>
                            <button
                                type="button"
                                class={classes!("btn-terminal", "text-xs")}
                                disabled={!initial_override_enabled || *saving}
                                onclick={on_restore_inherit}
                            >
                                { "Restore Inherit" }
                            </button>
                        </div>
                    </div>
                    if let Some(policy_error) = effective_policy_parse_error.clone() {
                        <p class={classes!("m-0", "text-xs", "text-red-600", "dark:text-red-300")}>
                            { format!("Backend effective policy JSON is invalid: {policy_error}") }
                        </p>
                    }
                    if *policy_override_enabled {
                        if effective_policy_parse_error.is_none() {
                            <KiroCachePolicyEditor form={key_policy_form.clone()} />
                        }
                    } else {
                        <p class={classes!("m-0", "text-xs", "text-[var(--muted)]")}>
                            { "This key inherits the global cache policy. Enable override to replace only changed scalar fields or the full bands block." }
                        </p>
                    }
                </div>
                <div class={classes!("md:col-span-2", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "space-y-3")}>
                    <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                        <div class={classes!("space-y-1")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Billable Multipliers" }</div>
                            <div class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>
                                { format!("global: {global_billable_multiplier_summary}") }
                            </div>
                            <div class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>
                                { format!("effective: {effective_billable_multiplier_summary}") }
                            </div>
                        </div>
                        <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                            <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                                <input
                                    type="checkbox"
                                    class={classes!("switch")}
                                    checked={*billable_multiplier_override_enabled}
                                    disabled={billable_multiplier_controls_disabled}
                                    onchange={{
                                        let billable_multiplier_override_enabled =
                                            billable_multiplier_override_enabled.clone();
                                        let key_billable_multiplier_json =
                                            key_billable_multiplier_json.clone();
                                        let key_billable_multiplier_effective_baseline =
                                            key_billable_multiplier_effective_baseline.clone();
                                        Callback::from(move |event: Event| {
                                            let input: HtmlInputElement =
                                                event.target_unchecked_into();
                                            let checked = input.checked();
                                            billable_multiplier_override_enabled.set(checked);
                                            if checked {
                                                key_billable_multiplier_json.set(
                                                    (*key_billable_multiplier_effective_baseline)
                                                        .clone(),
                                                );
                                            }
                                        })
                                    }}
                                />
                                <span>{ "Override Global Multipliers" }</span>
                            </label>
                            <button
                                type="button"
                                class={classes!("btn-terminal", "text-xs")}
                                onclick={{
                                    let billable_multiplier_settings_expanded =
                                        billable_multiplier_settings_expanded.clone();
                                    Callback::from(move |_| {
                                        billable_multiplier_settings_expanded
                                            .set(!*billable_multiplier_settings_expanded)
                                    })
                                }}
                            >
                                { if *billable_multiplier_settings_expanded {
                                    "Hide Multiplier Settings"
                                } else {
                                    "Show Multiplier Settings"
                                } }
                            </button>
                        </div>
                    </div>
                    if let Some(multiplier_error) = effective_billable_multiplier_parse_error.clone() {
                        <p class={classes!("m-0", "text-xs", "text-red-600", "dark:text-red-300")}>
                            { format!("Backend effective multiplier JSON is invalid: {multiplier_error}") }
                        </p>
                    }
                    if *billable_multiplier_settings_expanded {
                        if *billable_multiplier_override_enabled {
                            <div class={classes!("space-y-2")}>
                                <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                    { "只支持 `opus` / `sonnet` / `haiku`。这里编辑的是 key 的有效倍率，保存时只会写入相对全局发生变化的 key。" }
                                </div>
                                <textarea
                                    class={classes!("min-h-[10rem]", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-3", "font-mono", "text-xs", "leading-6")}
                                    value={(*key_billable_multiplier_json).clone()}
                                    oninput={{
                                        let key_billable_multiplier_json =
                                            key_billable_multiplier_json.clone();
                                        Callback::from(move |event: InputEvent| {
                                            let input: HtmlTextAreaElement =
                                                event.target_unchecked_into();
                                            key_billable_multiplier_json.set(input.value());
                                        })
                                    }}
                                />
                            </div>
                        } else {
                            <p class={classes!("m-0", "text-xs", "text-[var(--muted)]")}>
                                { "This key inherits the global billable multipliers. Enable override to change only the model families that differ from the global defaults." }
                            </p>
                        }
                    }
                </div>
                <div class={classes!("flex", "items-center", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "text-sm")}>
                    <span class={classes!("inline-flex", "items-center", "rounded-full", "bg-slate-900", "px-2", "py-1", "font-mono", "text-[11px]", "font-semibold", "uppercase", "tracking-[0.16em]", "text-emerald-300")}>
                        { "private" }
                    </span>
                    <span>{ "Kiro key 不会在公开页面暴露。" }</span>
                </div>
                <div class={classes!("md:col-span-2", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "text-sm", "text-[var(--muted)]", "space-y-2")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Route Settings" }</div>
                            <div class={classes!("mt-1", "text-xs", "text-[var(--muted)]")}>{ route_summary.clone() }</div>
                        </div>
                        <button
                            type="button"
                            class={classes!("btn-terminal", "text-xs")}
                            onclick={{
                                let route_settings_expanded = route_settings_expanded.clone();
                                Callback::from(move |_| route_settings_expanded.set(!*route_settings_expanded))
                            }}
                        >
                            { if *route_settings_expanded { "Hide Route Settings" } else { "Show Route Settings" } }
                        </button>
                    </div>
                    if *route_settings_expanded {
                        <div class={classes!("flex", "items-center", "gap-3", "flex-wrap")}>
                            <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                                <span>{ "路由" }</span>
                                    <select
                                        key={format!("{}-route-{}", props.key_item.id, (*route_strategy).clone())}
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
                                    <option value="auto" selected={*route_strategy == "auto"}>{ "自动 (按池策略)" }</option>
                                    <option value="fixed" selected={*route_strategy == "fixed"}>{ "绑定账号" }</option>
                                </select>
                            </label>
                            if *route_strategy == "fixed" {
                                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                                    <span>{ "单账号组" }</span>
                                    <select
                                        key={format!("{}-group-fixed-{}", props.key_item.id, (*account_group_id).clone())}
                                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-1.5", "text-sm")}
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
                                <label
                                    class={classes!("flex", "items-center", "gap-2", "text-sm")}
                                    title={kiro_pool_strategy_description((*preferred_pool_strategy).as_str())}
                                >
                                    <span>{ "优先池" }</span>
                                    <select
                                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-1.5", "text-sm")}
                                        value={(*preferred_pool_strategy).clone()}
                                        onchange={{
                                            let preferred_pool_strategy = preferred_pool_strategy.clone();
                                            Callback::from(move |event: Event| {
                                                if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                                    preferred_pool_strategy.set(target.value());
                                                }
                                            })
                                        }}
                                    >
                                        { kiro_pool_strategy_options((*preferred_pool_strategy).as_str()) }
                                    </select>
                                </label>
                                <label
                                    class={classes!("flex", "items-center", "gap-2", "text-sm")}
                                    title={anthropic_upstream_pool_mode_description((*anthropic_upstream_pool_mode).as_str())}
                                >
                                    <span>{ "Anthropic 直连" }</span>
                                    <select
                                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-1.5", "text-sm")}
                                        value={(*anthropic_upstream_pool_mode).clone()}
                                        onchange={{
                                            let anthropic_upstream_pool_mode = anthropic_upstream_pool_mode.clone();
                                            Callback::from(move |event: Event| {
                                                if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                                    anthropic_upstream_pool_mode.set(target.value());
                                                }
                                            })
                                        }}
                                    >
                                        { anthropic_upstream_pool_mode_options((*anthropic_upstream_pool_mode).as_str()) }
                                    </select>
                                </label>
                                <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                                    <span>{ "账号组" }</span>
                                    <select
                                        key={format!("{}-group-auto-{}", props.key_item.id, (*account_group_id).clone())}
                                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-1.5", "text-sm")}
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
                        </div>
                    }
                </div>
                <div class={classes!("md:col-span-2", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div class={classes!("min-w-0")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Model Routing" }</div>
                            <div class={classes!("mt-1", "text-xs", "text-[var(--muted)]", "break-words")}>
                                { model_routing_summary.clone() }
                            </div>
                        </div>
                        <button
                            type="button"
                            class={classes!("btn-terminal", "text-xs")}
                            onclick={{
                                let model_group_routing_open = model_group_routing_open.clone();
                                Callback::from(move |_| model_group_routing_open.set(true))
                            }}
                        >
                            { "Configure" }
                        </button>
                    </div>
                </div>
                <div class={classes!("md:col-span-2", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Model Mapping" }</div>
                            <div class={classes!("mt-1", "text-xs", "text-[var(--muted)]")}>
                                { format!("{} · overrides {}", mapping_overrides_preview, model_override_count) }
                            </div>
                        </div>
                        <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                            <button
                                type="button"
                                class={classes!("btn-terminal", "text-xs")}
                                onclick={{
                                    let model_mapping_expanded = model_mapping_expanded.clone();
                                    Callback::from(move |_| model_mapping_expanded.set(!*model_mapping_expanded))
                                }}
                            >
                                { if *model_mapping_expanded { "Hide Model Mapping" } else { "Show Model Mapping" } }
                            </button>
                            if *model_mapping_expanded {
                                <button type="button" class={classes!("btn-terminal", "text-xs")} onclick={on_reset_model_map}>
                                    { "Reset To Identity" }
                                </button>
                            }
                        </div>
                    </div>
                    if *model_mapping_expanded {
                        if props.available_models.is_empty() {
                            <div class={classes!("mt-3", "text-sm", "text-[var(--muted)]")}>{ "当前没有加载到可用模型目录。" }</div>
                        } else {
                            <div class={classes!("mt-3", "space-y-2")}>
                                { for props.available_models.iter().map(|source_model| {
                                    let source_id = source_model.id.clone();
                                    let current_target = (*model_name_map)
                                        .get(&source_id)
                                        .cloned()
                                        .unwrap_or_else(|| source_id.clone());
                                    let model_name_map = model_name_map.clone();
                                    let target_models = props.available_models.clone();
                                    html! {
                                        <div class={classes!("grid", "gap-2", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-3", "lg:grid-cols-[minmax(0,1fr)_minmax(18rem,24rem)]")}>
                                            <div>
                                                <div class={classes!("text-sm", "font-semibold", "text-[var(--text)]")}>{ source_model.display_name.clone() }</div>
                                                <div class={classes!("mt-1", "font-mono", "text-[11px]", "break-all", "text-[var(--muted)]")}>{ source_model.id.clone() }</div>
                                            </div>
                                            <label class={classes!("text-sm")}>
                                                <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Map To" }</div>
                                                <select
                                                    class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "text-sm")}
                                                    value={current_target.clone()}
                                                    onchange={Callback::from(move |event: Event| {
                                                        let input: HtmlSelectElement = event.target_unchecked_into();
                                                        let selected = input.value();
                                                        let mut next = (*model_name_map).clone();
                                                        if selected == source_id {
                                                            next.remove(&source_id);
                                                        } else {
                                                            next.insert(source_id.clone(), selected);
                                                        }
                                                        model_name_map.set(next);
                                                    })}
                                                >
                                                    { for target_models.iter().map(|target_model| html! {
                                                        <option value={target_model.id.clone()} selected={target_model.id == current_target}>
                                                            { format!("{} · {}", target_model.display_name, target_model.id) }
                                                        </option>
                                                    }) }
                                                </select>
                                            </label>
                                        </div>
                                    }
                                }) }
                            </div>
                            <div class={classes!("mt-3", "font-mono", "text-[11px]", "text-[var(--muted)]", "break-words")}>
                                { format!("overrides: {}", mapping_overrides_preview) }
                            </div>
                        }
                    }
                </div>
            </div>

            if *model_group_routing_open {
                <div class={classes!("fixed", "inset-0", "z-50")}>
                    <button
                        type="button"
                        class={classes!("absolute", "inset-0", "h-full", "w-full", "border-0", "bg-black/45", "p-0")}
                        aria-label="Close model routing"
                        onclick={{
                            let model_group_routing_open = model_group_routing_open.clone();
                            Callback::from(move |_| model_group_routing_open.set(false))
                        }}
                    />
                    <section class={classes!("absolute", "right-0", "top-0", "flex", "h-full", "w-full", "max-w-3xl", "flex-col", "border-l", "border-[var(--border)]", "bg-[var(--surface)]", "shadow-2xl")}>
                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "border-b", "border-[var(--border)]", "px-5", "py-4")}>
                            <div class={classes!("min-w-0")}>
                                <div class={classes!("text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Model Routing" }</div>
                                <h4 class={classes!("m-0", "mt-1", "text-base", "font-semibold", "text-[var(--text)]")}>
                                    { props.key_item.name.clone() }
                                </h4>
                                <div class={classes!("mt-1", "text-xs", "text-[var(--muted)]", "break-words")}>
                                    { model_routing_summary.clone() }
                                </div>
                            </div>
                            <button
                                type="button"
                                class={classes!("btn-terminal", "text-xs")}
                                onclick={{
                                    let model_group_routing_open = model_group_routing_open.clone();
                                    Callback::from(move |_| model_group_routing_open.set(false))
                                }}
                            >
                                { "Close" }
                            </button>
                        </div>
                        <div class={classes!("min-h-0", "flex-1", "overflow-y-auto", "px-5", "py-4")}>
                            if props.available_models.is_empty() {
                                <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-4", "py-4", "text-sm", "text-[var(--muted)]")}>
                                    { "No model inventory loaded." }
                                </div>
                            } else {
                                <div class={classes!("grid", "gap-2")}>
                                    { for props.available_models.iter().map(|model| {
                                        let model_id = model.id.clone();
                                        let selected_group_id = (*model_group_preferences)
                                            .get(&model_id)
                                            .cloned()
                                            .unwrap_or_default();
                                        let selected_channel_name = (*model_channel_preferences)
                                            .get(&model_id)
                                            .cloned()
                                            .unwrap_or_default();
                                        let selected_channel_missing = !selected_channel_name.is_empty()
                                            && !props
                                                .anthropic_channels
                                                .iter()
                                                .any(|channel| channel.name == selected_channel_name);
                                        let model_id_for_group = model_id.clone();
                                        let model_id_for_channel = model_id.clone();
                                        let model_group_preferences = model_group_preferences.clone();
                                        let model_channel_preferences = model_channel_preferences.clone();
                                        let account_groups = props.account_groups.clone();
                                        let anthropic_channels = props.anthropic_channels.clone();
                                        html! {
                                            <div class={classes!("grid", "gap-3", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-3", "lg:grid-cols-[minmax(0,1fr)_minmax(14rem,18rem)_minmax(14rem,18rem)]")}>
                                                <div class={classes!("min-w-0")}>
                                                    <div class={classes!("text-sm", "font-semibold", "text-[var(--text)]")}>{ model.display_name.clone() }</div>
                                                    <div class={classes!("mt-1", "break-all", "font-mono", "text-[11px]", "text-[var(--muted)]")}>{ model.id.clone() }</div>
                                                </div>
                                                <label class={classes!("text-sm")}>
                                                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Preferred Group" }</div>
                                                    <select
                                                        class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm")}
                                                        value={selected_group_id.clone()}
                                                        onchange={Callback::from(move |event: Event| {
                                                            let input: HtmlSelectElement = event.target_unchecked_into();
                                                            let selected = input.value();
                                                            let mut next = (*model_group_preferences).clone();
                                                            if selected.trim().is_empty() {
                                                                next.remove(&model_id_for_group);
                                                            } else {
                                                                next.insert(model_id_for_group.clone(), selected);
                                                            }
                                                            model_group_preferences.set(next);
                                                        })}
                                                    >
                                                        <option value="" selected={selected_group_id.is_empty()}>{ "Default routing" }</option>
                                                        { for account_groups.iter().map(|group| html! {
                                                            <option value={group.id.clone()} selected={group.id == selected_group_id}>
                                                                { format!("{} · {} accounts", group.name, group.account_count) }
                                                            </option>
                                                        }) }
                                                    </select>
                                                </label>
                                                <label class={classes!("text-sm")}>
                                                    <div class={classes!("mb-1", "text-xs", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ "Preferred Channel" }</div>
                                                    <select
                                                        class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "text-sm")}
                                                        value={selected_channel_name.clone()}
                                                        onchange={Callback::from(move |event: Event| {
                                                            let input: HtmlSelectElement = event.target_unchecked_into();
                                                            let selected = input.value();
                                                            let mut next = (*model_channel_preferences).clone();
                                                            if selected.trim().is_empty() {
                                                                next.remove(&model_id_for_channel);
                                                            } else {
                                                                next.insert(model_id_for_channel.clone(), selected);
                                                            }
                                                            model_channel_preferences.set(next);
                                                        })}
                                                    >
                                                        <option value="" selected={selected_channel_name.is_empty()}>{ "Default channel order" }</option>
                                                        if selected_channel_missing {
                                                            <option value={selected_channel_name.clone()} selected={selected_channel_missing}>
                                                                { format!("{} · missing", selected_channel_name) }
                                                            </option>
                                                        }
                                                        { for anthropic_channels.iter().map(|channel| html! {
                                                            <option value={channel.name.clone()} selected={channel.name == selected_channel_name}>
                                                                { format!("{} · {}", channel.name, channel.status) }
                                                            </option>
                                                        }) }
                                                    </select>
                                                </label>
                                            </div>
                                        }
                                    }) }
                                </div>
                            }
                        </div>
                        <div class={classes!("flex", "items-center", "justify-between", "gap-3", "border-t", "border-[var(--border)]", "px-5", "py-4", "flex-wrap")}>
                            <button
                                type="button"
                                class={classes!("btn-terminal", "text-xs")}
                                onclick={on_reset_model_routing_preferences.clone()}
                                disabled={*saving}
                            >
                                { "Reset" }
                            </button>
                            <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                <button
                                    type="button"
                                    class={classes!("btn-terminal", "text-xs")}
                                    onclick={{
                                        let model_group_routing_open = model_group_routing_open.clone();
                                        Callback::from(move |_| model_group_routing_open.set(false))
                                    }}
                                    disabled={*saving}
                                >
                                    { "Cancel" }
                                </button>
                                <button
                                    type="button"
                                    class={classes!("btn-terminal", "btn-terminal-primary", "text-xs")}
                                    onclick={on_save_model_routing_preferences.clone()}
                                    disabled={*saving}
                                >
                                    { if *saving { "Saving..." } else { "Save Routing" } }
                                </button>
                            </div>
                        </div>
                    </section>
                </div>
            }

            <div class={classes!("mt-4", "flex", "items-center", "gap-2", "flex-wrap")}>
                <button type="button" class={classes!("btn-terminal", "btn-terminal-primary")} onclick={on_save} disabled={*saving}>
                    { if *saving { "Saving..." } else { "Save" } }
                </button>
                <button type="button" class={classes!("btn-terminal")} onclick={on_disable} disabled={*saving}>
                    { if *saving { "Disabling..." } else { "Disable" } }
                </button>
                <button
                    type="button"
                    class={classes!("btn-terminal", "btn-terminal-danger")}
                    onclick={on_delete}
                    disabled={*saving}
                >
                    { "Delete" }
                </button>
            </div>

            if let Some(message) = (*feedback).clone() {
                <div class={classes!("mt-3", "text-sm", "text-[var(--muted)]")}>{ message }</div>
            }
        </article>
    }
}

/// One navigation card on the Kiro overview linking to a dedicated section.
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

/// A labeled numeric/text config field bound to a string state handle, in the
/// admin-shell form style.
fn cache_cfg_field(label: &str, state: &UseStateHandle<String>) -> Html {
    let handle = state.clone();
    html! {
        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
            { label.to_string() }
            <input
                class={classes!("mono")}
                value={(**state).clone()}
                oninput={Callback::from(move |event: InputEvent| {
                    let input: HtmlInputElement = event.target_unchecked_into();
                    handle.set(input.value());
                })}
            />
        </label>
    }
}

/// The route for one Kiro admin section id (used to forward legacy `?tab=`
/// deep links onto the dedicated per-section routes).
fn kiro_tab_route(tab: &str) -> Route {
    match tab {
        TAB_ACCOUNTS => Route::AdminKiroGatewayAccountsManage,
        TAB_KEYS => Route::AdminKiroGatewayKeys,
        TAB_GROUPS => Route::AdminKiroGatewayGroups,
        TAB_USAGE => Route::AdminKiroGatewayUsage,
        _ => Route::AdminKiroGateway,
    }
}

#[function_component(AdminKiroGatewayPage)]
/// Render the Kiro-specific admin surface.
///
/// This page owns the full CRUD workflow for Kiro accounts and private keys,
/// plus usage inspection and provider-level proxy context.
pub fn admin_kiro_gateway_page() -> Html {
    let accounts_summary = use_state(AdminAccountsSummaryView::default);
    let keys_summary = use_state(AdminLlmGatewayKeysSummaryView::default);
    let proxy_configs = use_state(Vec::<AdminUpstreamProxyConfigView>::new);
    let proxy_bindings = use_state(Vec::<AdminUpstreamProxyBindingView>::new);
    let runtime_config = use_state(|| None::<LlmGatewayRuntimeConfig>);
    let kiro_cache_stats = use_state(|| None::<AdminKiroCacheStatsResponse>);
    let kiro_cache_stats_error = use_state(|| None::<String>);
    let kiro_cache_policy_form = use_state(KiroCachePolicyForm::default);
    let persisted_kiro_cache_policy_form = use_state(KiroCachePolicyForm::default);
    let kiro_cache_kmodels_json = use_state(String::new);
    let kiro_billable_model_multipliers_json = use_state(String::new);
    let persisted_kiro_billable_model_multipliers_json = use_state(String::new);
    let saving_kmodel_config = use_state(|| false);
    let kiro_context_usage_min_request_tokens = use_state(String::new);
    let kiro_compact_trigger_tokens = use_state(String::new);
    let kiro_prefix_cache_mode = use_state(String::new);
    let kiro_prefix_cache_max_tokens = use_state(String::new);
    let kiro_prefix_cache_entry_ttl_seconds = use_state(String::new);
    let kiro_conversation_anchor_max_entries = use_state(String::new);
    let kiro_conversation_anchor_ttl_seconds = use_state(String::new);
    let kiro_cache_snapshot_enabled = use_state(|| false);
    let kiro_cache_snapshot_interval_seconds = use_state(String::new);
    let kiro_cache_snapshot_ttl_seconds = use_state(String::new);
    let kiro_cache_snapshot_max_tokens = use_state(String::new);
    let kiro_cache_snapshot_max_anchor_entries = use_state(String::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let toast = use_state(|| None::<(String, bool)>);
    let toast_timeout = use_mut_ref(|| None::<Timeout>);
    let notify = {
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
    let refresh_tick = use_state(|| 0u32);
    let cache_config_expanded = use_state(|| false);
    // Legacy deep links used `?tab=`; forward them once onto the dedicated
    // per-section routes so old bookmarks keep working.
    {
        let navigator = use_navigator();
        use_effect_with((), move |_| {
            let legacy = crate::pages::llm_access_shared::initial_tab_from_url(
                &[TAB_ACCOUNTS, TAB_KEYS, TAB_GROUPS, TAB_USAGE],
                "",
            );
            if !legacy.is_empty() {
                if let Some(navigator) = navigator {
                    navigator.replace(&kiro_tab_route(&legacy));
                }
            }
            || ()
        });
    }


    {
        let proxy_configs = proxy_configs.clone();
        let proxy_bindings = proxy_bindings.clone();
        let runtime_config = runtime_config.clone();
        let accounts_summary = accounts_summary.clone();
        let keys_summary = keys_summary.clone();
        let kiro_cache_stats = kiro_cache_stats.clone();
        let kiro_cache_stats_error = kiro_cache_stats_error.clone();
        let kiro_cache_policy_form = kiro_cache_policy_form.clone();
        let persisted_kiro_cache_policy_form = persisted_kiro_cache_policy_form.clone();
        let kiro_cache_kmodels_json = kiro_cache_kmodels_json.clone();
        let kiro_billable_model_multipliers_json = kiro_billable_model_multipliers_json.clone();
        let persisted_kiro_billable_model_multipliers_json =
            persisted_kiro_billable_model_multipliers_json.clone();
        let kiro_context_usage_min_request_tokens = kiro_context_usage_min_request_tokens.clone();
        let kiro_compact_trigger_tokens = kiro_compact_trigger_tokens.clone();
        let kiro_prefix_cache_mode = kiro_prefix_cache_mode.clone();
        let kiro_prefix_cache_max_tokens = kiro_prefix_cache_max_tokens.clone();
        let kiro_prefix_cache_entry_ttl_seconds = kiro_prefix_cache_entry_ttl_seconds.clone();
        let kiro_conversation_anchor_max_entries = kiro_conversation_anchor_max_entries.clone();
        let kiro_conversation_anchor_ttl_seconds = kiro_conversation_anchor_ttl_seconds.clone();
        let kiro_cache_snapshot_enabled = kiro_cache_snapshot_enabled.clone();
        let kiro_cache_snapshot_interval_seconds = kiro_cache_snapshot_interval_seconds.clone();
        let kiro_cache_snapshot_ttl_seconds = kiro_cache_snapshot_ttl_seconds.clone();
        let kiro_cache_snapshot_max_tokens = kiro_cache_snapshot_max_tokens.clone();
        let kiro_cache_snapshot_max_anchor_entries = kiro_cache_snapshot_max_anchor_entries.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let proxy_configs = proxy_configs.clone();
            let proxy_bindings = proxy_bindings.clone();
            let runtime_config = runtime_config.clone();
            let accounts_summary = accounts_summary.clone();
            let keys_summary = keys_summary.clone();
            let kiro_cache_stats = kiro_cache_stats.clone();
            let kiro_cache_stats_error = kiro_cache_stats_error.clone();
            let kiro_cache_policy_form = kiro_cache_policy_form.clone();
            let persisted_kiro_cache_policy_form = persisted_kiro_cache_policy_form.clone();
            let kiro_cache_kmodels_json = kiro_cache_kmodels_json.clone();
            let kiro_billable_model_multipliers_json = kiro_billable_model_multipliers_json.clone();
            let persisted_kiro_billable_model_multipliers_json =
                persisted_kiro_billable_model_multipliers_json.clone();
            let kiro_context_usage_min_request_tokens =
                kiro_context_usage_min_request_tokens.clone();
            let kiro_prefix_cache_mode = kiro_prefix_cache_mode.clone();
            let kiro_prefix_cache_max_tokens = kiro_prefix_cache_max_tokens.clone();
            let kiro_prefix_cache_entry_ttl_seconds = kiro_prefix_cache_entry_ttl_seconds.clone();
            let kiro_conversation_anchor_max_entries = kiro_conversation_anchor_max_entries.clone();
            let kiro_conversation_anchor_ttl_seconds = kiro_conversation_anchor_ttl_seconds.clone();
            let loading = loading.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);
                let (
                    config_result,
                    accounts_summary_result,
                    keys_summary_result,
                    proxy_configs_result,
                    proxy_bindings_result,
                    cache_stats_result,
                ) = futures::join!(
                    fetch_admin_llm_gateway_config(),
                    fetch_admin_kiro_accounts_page(1, 0),
                    fetch_admin_kiro_keys_page(1, 0),
                    fetch_admin_llm_gateway_proxy_configs(),
                    fetch_admin_llm_gateway_proxy_bindings(),
                    fetch_admin_kiro_cache_stats(),
                );
                match (
                    config_result,
                    accounts_summary_result,
                    keys_summary_result,
                    proxy_configs_result,
                    proxy_bindings_result,
                ) {
                    (
                        Ok(config_resp),
                        Ok(accounts_summary_resp),
                        Ok(keys_summary_resp),
                        Ok(proxy_configs_resp),
                        Ok(proxy_bindings_resp),
                    ) => {
                        let policy_form = match parse_kiro_cache_policy_form_json(
                            &config_resp.kiro_cache_policy_json,
                        ) {
                            Ok(policy_form) => policy_form,
                            Err(err) => {
                                error.set(Some(err));
                                loading.set(false);
                                return;
                            },
                        };
                        let should_reset_policy_editor = should_reset_kiro_cache_policy_editor(
                            (*runtime_config).is_none(),
                            &kiro_cache_policy_form,
                            &persisted_kiro_cache_policy_form,
                        );
                        if should_reset_policy_editor {
                            kiro_cache_policy_form.set(policy_form.clone());
                        }
                        persisted_kiro_cache_policy_form.set(policy_form);
                        kiro_cache_kmodels_json
                            .set(format_json_for_textarea(&config_resp.kiro_cache_kmodels_json));
                        kiro_billable_model_multipliers_json.set(format_json_for_textarea(
                            &config_resp.kiro_billable_model_multipliers_json,
                        ));
                        persisted_kiro_billable_model_multipliers_json.set(
                            format_json_for_textarea(
                                &config_resp.kiro_billable_model_multipliers_json,
                            ),
                        );
                        kiro_context_usage_min_request_tokens.set(
                            config_resp
                                .kiro_context_usage_min_request_tokens
                                .to_string(),
                        );
                        kiro_compact_trigger_tokens
                            .set(config_resp.kiro_compact_trigger_tokens.to_string());
                        kiro_prefix_cache_mode.set(config_resp.kiro_prefix_cache_mode.clone());
                        kiro_prefix_cache_max_tokens
                            .set(config_resp.kiro_prefix_cache_max_tokens.to_string());
                        kiro_prefix_cache_entry_ttl_seconds
                            .set(config_resp.kiro_prefix_cache_entry_ttl_seconds.to_string());
                        kiro_conversation_anchor_max_entries
                            .set(config_resp.kiro_conversation_anchor_max_entries.to_string());
                        kiro_conversation_anchor_ttl_seconds
                            .set(config_resp.kiro_conversation_anchor_ttl_seconds.to_string());
                        kiro_cache_snapshot_enabled.set(config_resp.kiro_cache_snapshot_enabled);
                        kiro_cache_snapshot_interval_seconds
                            .set(config_resp.kiro_cache_snapshot_interval_seconds.to_string());
                        kiro_cache_snapshot_ttl_seconds
                            .set(config_resp.kiro_cache_snapshot_ttl_seconds.to_string());
                        kiro_cache_snapshot_max_tokens
                            .set(config_resp.kiro_cache_snapshot_max_tokens.to_string());
                        kiro_cache_snapshot_max_anchor_entries.set(
                            config_resp
                                .kiro_cache_snapshot_max_anchor_entries
                                .to_string(),
                        );
                        runtime_config.set(Some(config_resp));
                        accounts_summary.set(accounts_summary_resp.summary);
                        keys_summary.set(keys_summary_resp.summary);
                        proxy_configs.set(proxy_configs_resp.proxy_configs);
                        proxy_bindings.set(proxy_bindings_resp.bindings);
                        match cache_stats_result {
                            Ok(cache_stats_resp) => {
                                kiro_cache_stats.set(Some(cache_stats_resp));
                                kiro_cache_stats_error.set(None);
                            },
                            Err(err) => {
                                kiro_cache_stats.set(None);
                                kiro_cache_stats_error.set(Some(err));
                            },
                        }
                    },
                    (Err(err), _, _, _, _)
                    | (_, Err(err), _, _, _)
                    | (_, _, Err(err), _, _)
                    | (_, _, _, Err(err), _)
                    | (_, _, _, _, Err(err)) => {
                        error.set(Some(err));
                    },
                }
                loading.set(false);
            });
            || ()
        });
    }


    let on_reload = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_| refresh_tick.set(refresh_tick.wrapping_add(1)))
    };


    let on_save_kiro_cache_kmodels = {
        let runtime_config = runtime_config.clone();
        let kiro_cache_policy_form_input = kiro_cache_policy_form.clone();
        let persisted_kiro_cache_policy_form_input = persisted_kiro_cache_policy_form.clone();
        let kiro_cache_kmodels_json_input = kiro_cache_kmodels_json.clone();
        let kiro_cache_kmodels_json = kiro_cache_kmodels_json.clone();
        let kiro_billable_model_multipliers_json_input =
            kiro_billable_model_multipliers_json.clone();
        let kiro_billable_model_multipliers_json = kiro_billable_model_multipliers_json.clone();
        let persisted_kiro_billable_model_multipliers_json_input =
            persisted_kiro_billable_model_multipliers_json.clone();
        let kiro_context_usage_min_request_tokens_input =
            kiro_context_usage_min_request_tokens.clone();
        let kiro_compact_trigger_tokens_input = kiro_compact_trigger_tokens.clone();
        let kiro_prefix_cache_mode_input = kiro_prefix_cache_mode.clone();
        let kiro_prefix_cache_max_tokens_input = kiro_prefix_cache_max_tokens.clone();
        let kiro_prefix_cache_entry_ttl_seconds_input = kiro_prefix_cache_entry_ttl_seconds.clone();
        let kiro_conversation_anchor_max_entries_input =
            kiro_conversation_anchor_max_entries.clone();
        let kiro_conversation_anchor_ttl_seconds_input =
            kiro_conversation_anchor_ttl_seconds.clone();
        let kiro_cache_snapshot_enabled_input = kiro_cache_snapshot_enabled.clone();
        let kiro_cache_snapshot_interval_seconds_input =
            kiro_cache_snapshot_interval_seconds.clone();
        let kiro_cache_snapshot_ttl_seconds_input = kiro_cache_snapshot_ttl_seconds.clone();
        let kiro_cache_snapshot_max_tokens_input = kiro_cache_snapshot_max_tokens.clone();
        let kiro_cache_snapshot_max_anchor_entries_input =
            kiro_cache_snapshot_max_anchor_entries.clone();
        let saving_kmodel_config = saving_kmodel_config.clone();
        let notify = notify.clone();
        let error = error.clone();
        let on_reload = on_reload.clone();
        Callback::from(move |_| {
            let runtime_config = runtime_config.clone();
            let kiro_cache_policy_form_value = (*kiro_cache_policy_form_input).clone();
            let kiro_cache_policy_form_input = kiro_cache_policy_form_input.clone();
            let persisted_kiro_cache_policy_form_input =
                persisted_kiro_cache_policy_form_input.clone();
            let kiro_cache_kmodels_json = (*kiro_cache_kmodels_json).clone();
            let kiro_cache_kmodels_json_input = kiro_cache_kmodels_json_input.clone();
            let kiro_billable_model_multipliers_json =
                (*kiro_billable_model_multipliers_json).clone();
            let kiro_billable_model_multipliers_json_input =
                kiro_billable_model_multipliers_json_input.clone();
            let persisted_kiro_billable_model_multipliers_json_input =
                persisted_kiro_billable_model_multipliers_json_input.clone();
            let kiro_context_usage_min_request_tokens_value =
                (*kiro_context_usage_min_request_tokens_input).clone();
            let kiro_compact_trigger_tokens_value = (*kiro_compact_trigger_tokens_input).clone();
            let kiro_prefix_cache_mode_value = (*kiro_prefix_cache_mode_input).clone();
            let kiro_prefix_cache_max_tokens_value = (*kiro_prefix_cache_max_tokens_input).clone();
            let kiro_prefix_cache_entry_ttl_seconds_value =
                (*kiro_prefix_cache_entry_ttl_seconds_input).clone();
            let kiro_conversation_anchor_max_entries_value =
                (*kiro_conversation_anchor_max_entries_input).clone();
            let kiro_conversation_anchor_ttl_seconds_value =
                (*kiro_conversation_anchor_ttl_seconds_input).clone();
            let kiro_cache_snapshot_enabled_value = *kiro_cache_snapshot_enabled_input;
            let kiro_cache_snapshot_interval_seconds_value =
                (*kiro_cache_snapshot_interval_seconds_input).clone();
            let kiro_cache_snapshot_ttl_seconds_value =
                (*kiro_cache_snapshot_ttl_seconds_input).clone();
            let kiro_cache_snapshot_max_tokens_value =
                (*kiro_cache_snapshot_max_tokens_input).clone();
            let kiro_cache_snapshot_max_anchor_entries_value =
                (*kiro_cache_snapshot_max_anchor_entries_input).clone();
            let kiro_prefix_cache_mode_input = kiro_prefix_cache_mode_input.clone();
            let kiro_context_usage_min_request_tokens_input =
                kiro_context_usage_min_request_tokens_input.clone();
            let kiro_compact_trigger_tokens_input = kiro_compact_trigger_tokens_input.clone();
            let kiro_prefix_cache_max_tokens_input = kiro_prefix_cache_max_tokens_input.clone();
            let kiro_prefix_cache_entry_ttl_seconds_input =
                kiro_prefix_cache_entry_ttl_seconds_input.clone();
            let kiro_conversation_anchor_max_entries_input =
                kiro_conversation_anchor_max_entries_input.clone();
            let kiro_conversation_anchor_ttl_seconds_input =
                kiro_conversation_anchor_ttl_seconds_input.clone();
            let saving_kmodel_config = saving_kmodel_config.clone();
            let notify = notify.clone();
            let error = error.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Some(mut next_config) = (*runtime_config).clone() else {
                    let message = "Kiro runtime config is not loaded yet.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let mode = kiro_prefix_cache_mode_value.trim();
                if mode != "formula" && mode != "prefix_tree" {
                    let message =
                        "Kiro prefix cache mode must be `formula` or `prefix_tree`.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                }
                let Ok(context_usage_min_request_tokens) =
                    kiro_context_usage_min_request_tokens_value
                        .trim()
                        .parse::<u64>()
                else {
                    let message =
                        "Kiro contextUsage min request tokens must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                if context_usage_min_request_tokens == 0 {
                    let message =
                        "Kiro contextUsage min request tokens must be positive.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                }
                // `0` disables the proactive compaction gate, so it is allowed.
                let Ok(compact_trigger_tokens) =
                    kiro_compact_trigger_tokens_value.trim().parse::<u64>()
                else {
                    let message =
                        "Kiro compaction trigger tokens must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let Ok(prefix_cache_max_tokens) =
                    kiro_prefix_cache_max_tokens_value.trim().parse::<u64>()
                else {
                    let message =
                        "Kiro prefix cache max tokens must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let Ok(prefix_cache_entry_ttl_seconds) = kiro_prefix_cache_entry_ttl_seconds_value
                    .trim()
                    .parse::<u64>()
                else {
                    let message =
                        "Kiro prefix cache TTL seconds must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let Ok(conversation_anchor_max_entries) =
                    kiro_conversation_anchor_max_entries_value
                        .trim()
                        .parse::<u64>()
                else {
                    let message =
                        "Kiro conversation anchor max entries must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let Ok(conversation_anchor_ttl_seconds) =
                    kiro_conversation_anchor_ttl_seconds_value
                        .trim()
                        .parse::<u64>()
                else {
                    let message =
                        "Kiro conversation anchor TTL seconds must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let kiro_cache_policy_json =
                    match serialize_kiro_cache_policy_form_json(&kiro_cache_policy_form_value) {
                        Ok(value) => value,
                        Err(err) => {
                            error.set(Some(err.clone()));
                            notify.emit((err, true));
                            return;
                        },
                    };
                next_config.kiro_cache_kmodels_json = kiro_cache_kmodels_json;
                next_config.kiro_billable_model_multipliers_json =
                    kiro_billable_model_multipliers_json;
                next_config.kiro_cache_policy_json = kiro_cache_policy_json;
                next_config.kiro_context_usage_min_request_tokens =
                    context_usage_min_request_tokens;
                next_config.kiro_compact_trigger_tokens = compact_trigger_tokens;
                next_config.kiro_prefix_cache_mode = mode.to_string();
                next_config.kiro_prefix_cache_max_tokens = prefix_cache_max_tokens;
                next_config.kiro_prefix_cache_entry_ttl_seconds = prefix_cache_entry_ttl_seconds;
                next_config.kiro_conversation_anchor_max_entries = conversation_anchor_max_entries;
                next_config.kiro_conversation_anchor_ttl_seconds = conversation_anchor_ttl_seconds;
                let Ok(cache_snapshot_interval_seconds) =
                    kiro_cache_snapshot_interval_seconds_value
                        .trim()
                        .parse::<u64>()
                else {
                    let message =
                        "Kiro cache snapshot interval seconds must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let Ok(cache_snapshot_ttl_seconds) =
                    kiro_cache_snapshot_ttl_seconds_value.trim().parse::<u64>()
                else {
                    let message =
                        "Kiro cache snapshot TTL seconds must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let Ok(cache_snapshot_max_tokens) =
                    kiro_cache_snapshot_max_tokens_value.trim().parse::<u64>()
                else {
                    let message =
                        "Kiro cache snapshot max tokens must be a valid integer.".to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                let Ok(cache_snapshot_max_anchor_entries) =
                    kiro_cache_snapshot_max_anchor_entries_value
                        .trim()
                        .parse::<u64>()
                else {
                    let message = "Kiro cache snapshot max anchor entries must be a valid integer."
                        .to_string();
                    error.set(Some(message.clone()));
                    notify.emit((message, true));
                    return;
                };
                next_config.kiro_cache_snapshot_enabled = kiro_cache_snapshot_enabled_value;
                next_config.kiro_cache_snapshot_interval_seconds = cache_snapshot_interval_seconds;
                next_config.kiro_cache_snapshot_ttl_seconds = cache_snapshot_ttl_seconds;
                next_config.kiro_cache_snapshot_max_tokens = cache_snapshot_max_tokens;
                next_config.kiro_cache_snapshot_max_anchor_entries =
                    cache_snapshot_max_anchor_entries;
                saving_kmodel_config.set(true);
                match update_admin_llm_gateway_config(&next_config).await {
                    Ok(saved) => {
                        let saved_policy_form = match parse_kiro_cache_policy_form_json(
                            &saved.kiro_cache_policy_json,
                        ) {
                            Ok(value) => value,
                            Err(err) => {
                                error.set(Some(err.clone()));
                                notify.emit((err, true));
                                saving_kmodel_config.set(false);
                                return;
                            },
                        };
                        error.set(None);
                        kiro_cache_policy_form_input.set(saved_policy_form.clone());
                        persisted_kiro_cache_policy_form_input.set(saved_policy_form);
                        kiro_cache_kmodels_json_input
                            .set(format_json_for_textarea(&saved.kiro_cache_kmodels_json));
                        kiro_billable_model_multipliers_json_input.set(format_json_for_textarea(
                            &saved.kiro_billable_model_multipliers_json,
                        ));
                        persisted_kiro_billable_model_multipliers_json_input.set(
                            format_json_for_textarea(&saved.kiro_billable_model_multipliers_json),
                        );
                        kiro_context_usage_min_request_tokens_input
                            .set(saved.kiro_context_usage_min_request_tokens.to_string());
                        kiro_compact_trigger_tokens_input
                            .set(saved.kiro_compact_trigger_tokens.to_string());
                        kiro_prefix_cache_mode_input.set(saved.kiro_prefix_cache_mode.clone());
                        kiro_prefix_cache_max_tokens_input
                            .set(saved.kiro_prefix_cache_max_tokens.to_string());
                        kiro_prefix_cache_entry_ttl_seconds_input
                            .set(saved.kiro_prefix_cache_entry_ttl_seconds.to_string());
                        kiro_conversation_anchor_max_entries_input
                            .set(saved.kiro_conversation_anchor_max_entries.to_string());
                        kiro_conversation_anchor_ttl_seconds_input
                            .set(saved.kiro_conversation_anchor_ttl_seconds.to_string());
                        runtime_config.set(Some(saved.clone()));
                        notify.emit(("Saved Kiro cache config.".to_string(), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        error.set(Some(err.clone()));
                        notify.emit((format!("Failed to save Kiro cache config.\n{err}"), true));
                    },
                }
                saving_kmodel_config.set(false);
            });
        })
    };

    let account_summary = *accounts_summary;
    let key_summary = *keys_summary;
    let disabled_account_count = account_summary.disabled_count;
    let active_key_count = key_summary.active_count;


    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "Kiro Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Overview" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::KiroAccess} classes={classes!("linkbtn")}>{ "Kiro Access" }</Link<Route>>
                        <Link<Route> to={Route::LlmAccess} classes={classes!("linkbtn")}>{ "LLM Access" }</Link<Route>>
                        <button type="button" class={classes!("primary")} disabled={*loading} onclick={{
                            let on_reload = on_reload.clone();
                            Callback::from(move |_| on_reload.emit(()))
                        }}>
                            { if *loading { "Loading..." } else { "Refresh" } }
                        </button>
                    </div>
                </header>

                if let Some(err) = (*error).clone() {
                    <div class={classes!("errorline", "text-sm")}>{ err }</div>
                }

                <section class={classes!("panel")}>
                    <div class={classes!("stat-strip")}>
                        <div class={classes!("stat")}>
                            <span>{ "Accounts" }</span>
                            <b>{ account_summary.total }</b>
                        </div>
                        <div class={classes!("stat", (disabled_account_count > 0).then_some("warn"))}>
                            <span>{ "Disabled" }</span>
                            <b>{ disabled_account_count }</b>
                        </div>
                        <div class={classes!("stat")}>
                            <span>{ "Keys" }</span>
                            <b>{ key_summary.total }</b>
                        </div>
                        <div class={classes!("stat")}>
                            <span>{ "Active Keys" }</span>
                            <b>{ active_key_count }</b>
                        </div>
                    </div>
                </section>

                <section class={classes!("grid", "gap-3", "sm:grid-cols-2", "xl:grid-cols-3")}>
                    { overview_nav_card("Accounts", "导入 / 手动创建 Kiro 账号", Route::AdminKiroGatewayAccountsManage) }
                    { overview_nav_card("Account Status", "分页浏览账号余额与异常状态", Route::AdminKiroAccountStatus) }
                    { overview_nav_card("Keys", "创建与管理私钥、路由与缓存策略", Route::AdminKiroGatewayKeys) }
                    { overview_nav_card("Groups", "维护账号组，供 key 选择固定/自动路由", Route::AdminKiroGatewayGroups) }
                    { overview_nav_card("Usage", "分页查看用量事件与详情", Route::AdminKiroGatewayUsage) }
                    { overview_nav_card("Upstream Channels", "直连 Anthropic 渠道、模型测试与冷却", Route::AdminKiroAnthropicUpstreams) }
                </section>

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <h2>{ "Effective Upstream Proxy" }</h2>
                    </div>
                    <div class={classes!("panel-body")}>
                        {
                            if let Some(binding) = proxy_bindings.iter().find(|item| item.provider_type == "kiro") {
                                html! {
                                    <div class={classes!("space-y-2", "text-sm")}>
                                        <div class={classes!("eyebrow")}>{ format!("source: {}", binding.effective_source) }</div>
                                        <div class={classes!("rounded-[var(--r-field)]", "border", "border-[var(--border)]", "bg-[var(--card-2)]", "px-3", "py-3")}>
                                            <div class={classes!("mono", "break-all")}>
                                                { binding.effective_proxy_url.clone().unwrap_or_else(|| "-".to_string()) }
                                            </div>
                                            if let Some(name) = binding.effective_proxy_config_name.as_deref() {
                                                <div class={classes!("mt-2", "text-xs", "text-[var(--muted-foreground)]")}>{ format!("config: {}", name) }</div>
                                            }
                                            if let Some(error_message) = binding.error_message.as_deref() {
                                                <div class={classes!("mt-2", "text-xs", "text-[var(--destructive)]")}>{ error_message }</div>
                                            }
                                        </div>
                                        <p class={classes!("m-0", "text-xs", "text-[var(--muted-foreground)]")}>
                                            { "Kiro 的默认 provider 级代理。账号未单独指定时继承它；账号改成 direct/fixed 会覆盖这里的默认值。" }
                                        </p>
                                    </div>
                                }
                            } else {
                                html! {
                                    <p class={classes!("m-0", "text-sm", "text-[var(--muted-foreground)]")}>
                                        { "当前还没有拿到 Kiro provider 代理绑定状态。" }
                                    </p>
                                }
                            }
                        }
                    </div>
                </section>

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <div>
                            <h2>{ "Kiro Cache Simulation" }</h2>
                            <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Kiro cache 全局参数的唯一前端编辑面：cache policy、模拟模式、prefix tree 容量/TTL、按模型 Kmodel 系数与 billable 倍率。" }
                            </p>
                        </div>
                        <button
                            type="button"
                            class={classes!("ghost")}
                            onclick={{
                                let cache_config_expanded = cache_config_expanded.clone();
                                Callback::from(move |_| cache_config_expanded.set(!*cache_config_expanded))
                            }}
                        >
                            { if *cache_config_expanded { "收起配置" } else { "展开配置" } }
                        </button>
                    </div>
                    <div class={classes!("panel-body", "space-y-4")}>
                        {
                            if let Some(stats) = (*kiro_cache_stats).clone() {
                                let token_percent = kiro_cache_token_percent(
                                    stats.prefix_tree.resident_tokens,
                                    stats.prefix_tree.max_tokens,
                                );
                                let estimated_bytes = stats
                                    .prefix_tree
                                    .estimated_memory_bytes
                                    .saturating_add(stats.conversation_anchors.estimated_memory_bytes);
                                html! {
                                    <div class={classes!("rounded-[var(--r-field)]", "border", "border-[var(--border)]", "bg-[var(--card-2)]", "px-3", "py-3")}>
                                        <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                            <div>
                                                <div class={classes!("eyebrow")}>{ "Runtime Cache Footprint" }</div>
                                                <div class={classes!("mono", "mt-1", "text-sm")}>
                                                    { format!("mode={} · page={} tokens", stats.mode, stats.page_size_tokens) }
                                                </div>
                                            </div>
                                            <div class={classes!("mono", "text-sm", "font-semibold")}>
                                                { format!("{:.2}% tokens", token_percent) }
                                            </div>
                                        </div>
                                        <div class={classes!("mt-3", "h-2", "overflow-hidden", "rounded-full", "bg-[var(--secondary)]")}>
                                            <div class={classes!("h-full", "bg-[var(--success)]")} style={format!("width: {:.2}%;", token_percent)} />
                                        </div>
                                        <div class={classes!("mono", "mt-3", "grid", "gap-2", "text-xs", "sm:grid-cols-2", "xl:grid-cols-5")}>
                                            <div>
                                                <div class={classes!("text-[var(--muted-foreground)]")}>{ "resident / max" }</div>
                                                <div>{ format!("{} / {}", format_number_u64(stats.prefix_tree.resident_tokens), format_number_u64(stats.prefix_tree.max_tokens)) }</div>
                                            </div>
                                            <div>
                                                <div class={classes!("text-[var(--muted-foreground)]")}>{ "estimated memory" }</div>
                                                <div>{ format_compact_bytes(estimated_bytes) }</div>
                                            </div>
                                            <div>
                                                <div class={classes!("text-[var(--muted-foreground)]")}>{ "nodes / leaves" }</div>
                                                <div>{ format!("{} / {}", format_number_u64(stats.prefix_tree.node_count as u64), format_number_u64(stats.prefix_tree.leaf_count as u64)) }</div>
                                            </div>
                                            <div>
                                                <div class={classes!("text-[var(--muted-foreground)]")}>{ "anchors" }</div>
                                                <div>{ format!("{} / {}", format_number_u64(stats.conversation_anchors.entries as u64), format_number_u64(stats.conversation_anchors.max_entries as u64)) }</div>
                                            </div>
                                            <div>
                                                <div class={classes!("text-[var(--muted-foreground)]")}>{ "process rss" }</div>
                                                <div>{ stats.process_memory.rss_bytes.map(format_compact_bytes).unwrap_or_else(|| "-".to_string()) }</div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else if let Some(err) = (*kiro_cache_stats_error).clone() {
                                html! {
                                    <div class={classes!("errorline", "mono", "text-xs")}>
                                        { format!("Runtime cache stats unavailable: {err}") }
                                    </div>
                                }
                            } else {
                                html! { <div class={classes!("skeleton")}><i></i><i></i></div> }
                            }
                        }
                        if *cache_config_expanded {
                            <div class={classes!("rounded-[var(--r-field)]", "border", "border-[var(--border)]", "bg-[var(--card-2)]", "px-3", "py-3", "space-y-2")}>
                                <div class={classes!("eyebrow")}>{ "Global Cache Policy" }</div>
                                <div class={classes!("text-xs", "text-[var(--muted-foreground)]")}>
                                    { "Key 级 override 只覆盖变化过的标量字段，bands 则整段替换。未覆盖的字段继续继承全局默认。" }
                                </div>
                                <div class={classes!("mono", "text-xs", "text-[var(--muted-foreground)]")}>
                                    { format_kiro_cache_policy_summary(&kiro_cache_policy_form, &kiro_cache_policy_form) }
                                </div>
                                <KiroCachePolicyEditor form={kiro_cache_policy_form.clone()} />
                            </div>
                            <div class={classes!("grid", "gap-3", "sm:grid-cols-2", "xl:grid-cols-3")}>
                                { cache_cfg_field("contextUsage Min Request Tokens", &kiro_context_usage_min_request_tokens) }
                                { cache_cfg_field("Compaction Trigger Tokens (0 disables)", &kiro_compact_trigger_tokens) }
                                <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                    { "Simulation Mode" }
                                    <select value={(*kiro_prefix_cache_mode).clone()} onchange={{
                                        let kiro_prefix_cache_mode = kiro_prefix_cache_mode.clone();
                                        Callback::from(move |event: Event| {
                                            let input: HtmlSelectElement = event.target_unchecked_into();
                                            kiro_prefix_cache_mode.set(input.value());
                                        })
                                    }}>
                                        <option value="formula" selected={*kiro_prefix_cache_mode == "formula"}>{ "formula" }</option>
                                        <option value="prefix_tree" selected={*kiro_prefix_cache_mode == "prefix_tree"}>{ "prefix_tree" }</option>
                                    </select>
                                </label>
                                { cache_cfg_field("Prefix Tree Max Tokens", &kiro_prefix_cache_max_tokens) }
                                { cache_cfg_field("Prefix Tree TTL Seconds", &kiro_prefix_cache_entry_ttl_seconds) }
                                { cache_cfg_field("Anchor Max Entries", &kiro_conversation_anchor_max_entries) }
                                { cache_cfg_field("Anchor TTL Seconds", &kiro_conversation_anchor_ttl_seconds) }
                                { cache_cfg_field("Snapshot Interval Seconds", &kiro_cache_snapshot_interval_seconds) }
                                { cache_cfg_field("Snapshot TTL Seconds", &kiro_cache_snapshot_ttl_seconds) }
                                { cache_cfg_field("Snapshot Max Tokens (0 = live)", &kiro_cache_snapshot_max_tokens) }
                                { cache_cfg_field("Snapshot Max Anchor Entries (0 = live)", &kiro_cache_snapshot_max_anchor_entries) }
                                <label class={classes!("flex", "items-center", "gap-2", "text-xs", "text-[var(--muted-foreground)]")}>
                                    <input
                                        type="checkbox"
                                        class={classes!("min-h-0", "w-auto")}
                                        checked={*kiro_cache_snapshot_enabled}
                                        oninput={{
                                            let kiro_cache_snapshot_enabled = kiro_cache_snapshot_enabled.clone();
                                            Callback::from(move |event: InputEvent| {
                                                let input: HtmlInputElement = event.target_unchecked_into();
                                                kiro_cache_snapshot_enabled.set(input.checked());
                                            })
                                        }}
                                    />
                                    { "Snapshot Persistence (Valkey)" }
                                </label>
                            </div>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Kmodel JSON" }
                                <textarea
                                    class={classes!("mono", "min-h-[18rem]", "leading-6")}
                                    value={(*kiro_cache_kmodels_json).clone()}
                                    oninput={{
                                        let kiro_cache_kmodels_json = kiro_cache_kmodels_json.clone();
                                        Callback::from(move |event: InputEvent| {
                                            let input: HtmlTextAreaElement = event.target_unchecked_into();
                                            kiro_cache_kmodels_json.set(input.value());
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                { "Billable Multiplier JSON" }
                                <span class={classes!("text-[11px]", "text-[var(--faint)]")}>
                                    { "识别 `fable` / `opus` / `sonnet` / `haiku` 四个 key。fable 默认 2.0（opus 两倍），其余 1.0。" }
                                </span>
                                <textarea
                                    class={classes!("mono", "min-h-[10rem]", "leading-6")}
                                    value={(*kiro_billable_model_multipliers_json).clone()}
                                    oninput={{
                                        let kiro_billable_model_multipliers_json = kiro_billable_model_multipliers_json.clone();
                                        Callback::from(move |event: InputEvent| {
                                            let input: HtmlTextAreaElement = event.target_unchecked_into();
                                            kiro_billable_model_multipliers_json.set(input.value());
                                        })
                                    }}
                                />
                            </label>
                            if let Some(config) = (*runtime_config).clone() {
                                <div class={classes!("mono", "rounded-[var(--r-field)]", "border", "border-[var(--border)]", "bg-[var(--card-2)]", "px-3", "py-3", "text-xs", "text-[var(--muted-foreground)]", "space-y-1")}>
                                    <div>{ format!("stored kmodel bytes: {} · billable bytes: {} · policy bytes: {}", config.kiro_cache_kmodels_json.len(), config.kiro_billable_model_multipliers_json.len(), config.kiro_cache_policy_json.len()) }</div>
                                    <div>{ format!("context_min={}, mode={}, prefix_max={}, prefix_ttl={}, anchor_max={}, anchor_ttl={}", config.kiro_context_usage_min_request_tokens, config.kiro_prefix_cache_mode, config.kiro_prefix_cache_max_tokens, config.kiro_prefix_cache_entry_ttl_seconds, config.kiro_conversation_anchor_max_entries, config.kiro_conversation_anchor_ttl_seconds) }</div>
                                </div>
                            }
                            <div class={classes!("flex", "justify-end")}>
                                <button
                                    type="button"
                                    class={classes!("primary")}
                                    disabled={*saving_kmodel_config}
                                    onclick={on_save_kiro_cache_kmodels}
                                >
                                    { if *saving_kmodel_config { "Saving..." } else { "Save Cache Settings" } }
                                </button>
                            </div>
                        }
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

pub(crate) fn normalized_str_option(state: &UseStateHandle<String>) -> Option<String> {
    let value = (**state).trim();
    (!value.is_empty()).then_some(value.to_string())
}


fn quota_progress_bar(balance: &KiroBalanceView, account_sub_title: Option<String>) -> Html {
    let subscription_title = balance
        .subscription_title
        .clone()
        .unwrap_or_else(|| account_sub_title.unwrap_or_else(|| "-".to_string()));
    let ratio = kiro_credit_ratio(Some(balance.current_usage), Some(balance.usage_limit));
    let pct = (ratio * 100.0).round() as i32;
    let upstream_limit_label = balance
        .upstream_usage_limit
        .filter(|value| *value != balance.usage_limit)
        .map(|value| format!("upstream {}", format_float2(value)));
    let manual_limit_label = balance
        .manual_usage_limit
        .map(|value| format!("manual {}", format_float2(value)));
    html! { <>
        <div class={classes!("mt-3", "grid", "gap-3", "grid-cols-2")}>
            <div>
                <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "剩余" }</div>
                <div class={classes!("mt-1", "font-mono", "text-xl", "font-black", "text-[var(--text)]")}>
                    { format_float2(balance.remaining) }
                </div>
            </div>
            <div>
                <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "总额度" }</div>
                <div class={classes!("mt-1", "font-mono", "text-xl", "font-black", "text-[var(--text)]")}>
                    { format_float2(balance.usage_limit) }
                </div>
            </div>
        </div>
        <div class={classes!("mt-3")}>
            <div class={classes!("flex", "items-center", "justify-between", "font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>
                <span>{ "用量" }</span>
                <span>{ format!("{pct}%") }</span>
            </div>
            <div class={classes!("mt-1.5", "h-2", "overflow-hidden", "rounded-full", "bg-[var(--surface)]")}>
                <div class={classes!("h-full", "rounded-full", "bg-[linear-gradient(90deg,#0f766e,#2563eb)]", "transition-[width]", "duration-300")}
                     style={format!("width: {}%;", pct.clamp(0, 100))} />
            </div>
            <div class={classes!("mt-2", "flex", "items-center", "gap-4", "font-mono", "text-[11px]", "text-[var(--muted)]")}>
                <span>{ subscription_title }</span>
                if let Some(label) = manual_limit_label {
                    <span>{ label }</span>
                }
                if let Some(label) = upstream_limit_label {
                    <span>{ label }</span>
                }
                <span class={classes!("ml-auto")}>{ format_reset_hint(balance.next_reset_at) }</span>
            </div>
        </div>
    </> }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        anthropic_routing_summary, build_kiro_billable_multiplier_override_json,
        build_kiro_billable_multiplier_override_patch, build_kiro_cache_policy_override_json,
        build_kiro_cache_policy_override_patch, default_kiro_billable_multiplier_map,
        format_compact_bytes, format_kiro_billable_multiplier_summary,
        format_kiro_cache_policy_summary, format_kiro_key_candidate_credit_summary,
        kiro_account_status_abnormal_href, kiro_account_status_cta_text, kiro_account_status_route,
        kiro_cache_token_percent, kiro_key_route_summary, kiro_preferred_pool_candidate_note,
        kiro_preferred_pool_warning, kiro_tab_route, parse_kiro_cache_policy_form_json,
        parse_manual_usage_limit_input, sanitize_kiro_account_group_id,
        should_reset_kiro_cache_policy_editor, TAB_ACCOUNTS, TAB_GROUPS, TAB_KEYS, TAB_USAGE,
    };
    use crate::{
        api::{
            AdminAccountGroupOptionView, AdminKiroKeyCandidateCreditSummaryView,
            PatchKiroAccountInput,
        },
        router::Route,
    };

    #[test]
    fn sanitize_kiro_account_group_id_drops_unknown_value() {
        let groups =
            vec![test_group("group-alpha", &["alpha"]), test_group("group-beta", &["beta"])];

        assert_eq!(sanitize_kiro_account_group_id(Some("missing"), &groups, true), "");
        assert_eq!(
            sanitize_kiro_account_group_id(Some(" group-beta "), &groups, true),
            "group-beta"
        );
    }

    #[test]
    fn anthropic_routing_summary_includes_preflight_change_count() {
        let diagnostics = json!({
            "upstream_pool": "direct_anthropic",
            "channel_name": "channel-a",
            "preflight": {
                "normalized": true,
                "change_count": 2,
                "tool_use_id_rewrite_count": 1,
                "normalization_event_count": 1,
                "tool_normalization_event_count": 0,
                "tool_schema_keyword_count": 0
            }
        })
        .to_string();

        assert_eq!(
            anthropic_routing_summary(Some(&diagnostics)),
            Some("Anthropic 直连 · channel channel-a · preflight 2 changes".to_string())
        );
    }

    #[test]
    fn anthropic_routing_summary_omits_schema_observation_only_preflight() {
        let diagnostics = json!({
            "upstream_pool": "direct_anthropic",
            "channel_name": "channel-a",
            "preflight": {
                "normalized": false,
                "change_count": 0,
                "tool_use_id_rewrite_count": 0,
                "normalization_event_count": 0,
                "tool_normalization_event_count": 0,
                "tool_schema_keyword_count": 2
            }
        })
        .to_string();

        assert_eq!(
            anthropic_routing_summary(Some(&diagnostics)),
            Some("Anthropic 直连 · channel channel-a".to_string())
        );
    }

    #[test]
    fn anthropic_routing_summary_omits_legacy_schema_only_preflight() {
        let diagnostics = json!({
            "upstream_pool": "direct_anthropic",
            "channel_name": "channel-a",
            "preflight": {
                "normalized": true,
                "tool_use_id_rewrite_count": 0,
                "normalization_event_count": 0,
                "tool_normalization_event_count": 0,
                "tool_schema_keyword_count": 2
            }
        })
        .to_string();

        assert_eq!(
            anthropic_routing_summary(Some(&diagnostics)),
            Some("Anthropic 直连 · channel channel-a".to_string())
        );
    }

    #[test]
    fn kiro_key_route_summary_uses_full_pool_text_when_group_is_empty() {
        let summary = kiro_key_route_summary(
            "auto",
            "",
            llm_access_core::store::KIRO_POOL_STRATEGY_BALANCED,
            &[],
        );
        assert!(summary.contains("全账号池自动择优"));
        assert!(summary.contains("标记为"));
        assert!(summary.contains("亲和 + 动态"));
    }

    #[test]
    fn kiro_key_route_summary_keeps_no_available_account_warning() {
        let groups = vec![test_group("group-alpha", &["alpha", "beta"])];
        let summary = kiro_key_route_summary(
            "auto",
            "group-alpha",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &groups,
        );

        assert!(summary.contains("回退到组内其他池"));
        assert!(summary.contains("若组内没有可用账号，请求会直接报错"));
    }

    #[test]
    fn kiro_key_route_summary_ignores_pool_for_fixed_route() {
        let groups = vec![test_group("group-alpha", &["alpha"])];
        let summary = kiro_key_route_summary(
            "fixed",
            "group-alpha",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &groups,
        );

        assert_eq!(summary, "固定组：group-alpha");
    }

    #[test]
    fn format_kiro_key_candidate_credit_summary_counts_missing_balances() {
        let text =
            format_kiro_key_candidate_credit_summary(&AdminKiroKeyCandidateCreditSummaryView {
                candidate_count: 3,
                preferred_pool_candidate_count: Some(1),
                loaded_balance_count: 2,
                missing_balance_count: 1,
                total_limit: 160.0,
                total_remaining: 50.0,
            });

        assert_eq!(
            text,
            "候选账号额度: 剩余 50.0000 / 总额 160.0000 · 2/3 已加载 · 1 个账号余额未加载"
        );
    }

    #[test]
    fn kiro_preferred_pool_warning_reports_empty_preferred_pool() {
        let warning = kiro_preferred_pool_warning(
            "auto",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &AdminKiroKeyCandidateCreditSummaryView {
                candidate_count: 2,
                preferred_pool_candidate_count: Some(0),
                loaded_balance_count: 2,
                missing_balance_count: 0,
                total_limit: 100.0,
                total_remaining: 40.0,
            },
        )
        .expect("empty preferred pool should warn");

        assert!(warning.contains("没有标记为 `剩余额度优先` 的账号"));
        assert!(warning.contains("优先池设置不会生效"));
    }

    #[test]
    fn kiro_preferred_pool_warning_ignores_fixed_or_unknown_backend_count() {
        let summary = AdminKiroKeyCandidateCreditSummaryView {
            candidate_count: 2,
            preferred_pool_candidate_count: Some(0),
            loaded_balance_count: 2,
            missing_balance_count: 0,
            total_limit: 100.0,
            total_remaining: 40.0,
        };
        assert!(kiro_preferred_pool_warning(
            "fixed",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &summary
        )
        .is_none());

        let missing_new_backend_field = AdminKiroKeyCandidateCreditSummaryView {
            preferred_pool_candidate_count: None,
            ..summary
        };
        assert!(kiro_preferred_pool_warning(
            "auto",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &missing_new_backend_field
        )
        .is_none());
    }

    #[test]
    fn kiro_preferred_pool_candidate_note_reports_matched_count() {
        let note = kiro_preferred_pool_candidate_note(
            "auto",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &AdminKiroKeyCandidateCreditSummaryView {
                candidate_count: 5,
                preferred_pool_candidate_count: Some(2),
                loaded_balance_count: 5,
                missing_balance_count: 0,
                total_limit: 500.0,
                total_remaining: 320.0,
            },
        )
        .expect("matched preferred pool should produce a note");

        assert_eq!(note, "优先池 `剩余额度优先` 命中 2/5 个候选账号");
    }

    #[test]
    fn kiro_preferred_pool_candidate_note_silent_for_fixed_empty_or_unknown() {
        let matched = AdminKiroKeyCandidateCreditSummaryView {
            candidate_count: 5,
            preferred_pool_candidate_count: Some(2),
            loaded_balance_count: 5,
            missing_balance_count: 0,
            total_limit: 500.0,
            total_remaining: 320.0,
        };
        assert!(kiro_preferred_pool_candidate_note(
            "fixed",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &matched
        )
        .is_none());

        // The empty-pool case is owned by kiro_preferred_pool_warning.
        let empty_pool = AdminKiroKeyCandidateCreditSummaryView {
            preferred_pool_candidate_count: Some(0),
            ..matched
        };
        assert!(kiro_preferred_pool_candidate_note(
            "auto",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &empty_pool
        )
        .is_none());

        let missing_new_backend_field = AdminKiroKeyCandidateCreditSummaryView {
            preferred_pool_candidate_count: None,
            ..matched
        };
        assert!(kiro_preferred_pool_candidate_note(
            "auto",
            llm_access_core::store::KIRO_POOL_STRATEGY_CREDIT_FIRST,
            &missing_new_backend_field
        )
        .is_none());
    }

    #[test]
    fn format_kiro_cache_policy_summary_reports_inherit_global() {
        let global = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    },
                    {
                        "credit_start": 1.0,
                        "credit_end": 2.5,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.0
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0
            }"#,
        )
        .expect("global policy should parse");

        let summary = format_kiro_cache_policy_summary(&global, &global);

        assert_eq!(
            summary,
            "inherit global · boost 1.0 -> 1.8 => 100000 · diag 2.0 · create 0.0 · bands 2"
        );
    }

    #[test]
    fn kiro_account_status_cta_text_is_stable() {
        assert_eq!(kiro_account_status_cta_text(), "Open Account Status Page");
    }

    #[test]
    fn kiro_account_status_route_points_to_admin_page() {
        assert_eq!(kiro_account_status_route(), Route::AdminKiroAccountStatus);
    }

    #[test]
    fn kiro_account_status_abnormal_href_opens_abnormal_filter() {
        assert!(kiro_account_status_abnormal_href().ends_with("/accounts?issue=abnormal"));
    }

    #[test]
    fn build_kiro_cache_policy_override_json_only_emits_changed_scalar_field() {
        let global = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    },
                    {
                        "credit_start": 1.0,
                        "credit_end": 2.5,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.0
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0
            }"#,
        )
        .expect("global policy should parse");
        let mut edited = global.clone();
        edited.high_credit_diagnostic_threshold = "1.5".to_string();

        let override_json = build_kiro_cache_policy_override_json(&global, &edited)
            .expect("override json")
            .expect("changed policy should emit override json");
        let override_value: serde_json::Value =
            serde_json::from_str(&override_json).expect("override json should parse");

        assert_eq!(
            override_value,
            json!({
                "high_credit_diagnostic_threshold": 1.5
            })
        );
    }

    #[test]
    fn build_kiro_cache_policy_override_json_only_emits_changed_creation_ratio() {
        let global = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0,
                "anthropic_cache_creation_input_ratio": 0.0
            }"#,
        )
        .expect("global policy should parse");
        let mut edited = global.clone();
        edited.anthropic_cache_creation_input_ratio = "0.25".to_string();

        let override_json = build_kiro_cache_policy_override_json(&global, &edited)
            .expect("override json")
            .expect("changed policy should emit override json");
        let override_value: serde_json::Value =
            serde_json::from_str(&override_json).expect("override json should parse");

        assert_eq!(
            override_value,
            json!({
                "anthropic_cache_creation_input_ratio": 0.25
            })
        );
    }

    #[test]
    fn build_kiro_cache_policy_override_json_only_emits_changed_bands_block() {
        let global = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    },
                    {
                        "credit_start": 1.0,
                        "credit_end": 2.5,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.0
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0
            }"#,
        )
        .expect("global policy should parse");
        let edited = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.5,
                        "credit_end": 1.5,
                        "cache_ratio_start": 0.6,
                        "cache_ratio_end": 0.25
                    },
                    {
                        "credit_start": 1.8,
                        "credit_end": 2.8,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.05
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0
            }"#,
        )
        .expect("edited policy should parse");

        let override_json = build_kiro_cache_policy_override_json(&global, &edited)
            .expect("override json")
            .expect("changed policy should emit override json");
        let override_value: serde_json::Value =
            serde_json::from_str(&override_json).expect("override json should parse");

        assert_eq!(
            override_value,
            json!({
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.5,
                        "credit_end": 1.5,
                        "cache_ratio_start": 0.6,
                        "cache_ratio_end": 0.25
                    },
                    {
                        "credit_start": 1.8,
                        "credit_end": 2.8,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.05
                    }
                ]
            })
        );
    }

    #[test]
    fn build_kiro_cache_policy_override_patch_keeps_existing_override_when_unchanged() {
        let global = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    },
                    {
                        "credit_start": 1.0,
                        "credit_end": 2.5,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.0
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0
            }"#,
        )
        .expect("global policy should parse");

        let patch = build_kiro_cache_policy_override_patch(&global, true, &global, true, &global)
            .expect("patch should build");

        assert_eq!(patch, None);
    }

    #[test]
    fn build_kiro_cache_policy_override_patch_clears_override_when_restoring_inherit() {
        let global = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    },
                    {
                        "credit_start": 1.0,
                        "credit_end": 2.5,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.0
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0
            }"#,
        )
        .expect("global policy should parse");

        let patch = build_kiro_cache_policy_override_patch(&global, true, &global, false, &global)
            .expect("patch should build");

        assert_eq!(patch, Some(None));
    }

    #[test]
    fn kiro_billable_multiplier_defaults_include_fable_at_twice_opus() {
        let defaults = default_kiro_billable_multiplier_map();

        assert_eq!(defaults.get("fable"), Some(&2.0));
        assert_eq!(defaults.get("opus"), Some(&1.0));
    }

    #[test]
    fn build_kiro_billable_multiplier_override_json_emits_fable_changes() {
        let override_json = build_kiro_billable_multiplier_override_json(
            r#"{"haiku":1.0,"opus":1.0,"sonnet":1.0}"#,
            r#"{"fable":3.0,"haiku":1.0,"opus":1.0,"sonnet":1.0}"#,
        )
        .expect("override json should build")
        .expect("changed fable multiplier should emit override json");
        let override_value: serde_json::Value =
            serde_json::from_str(&override_json).expect("override json should parse");

        assert_eq!(override_value, json!({ "fable": 3.0 }));
    }

    #[test]
    fn format_kiro_billable_multiplier_summary_includes_fable() {
        let summary = format_kiro_billable_multiplier_summary(
            true,
            r#"{"haiku":1.0,"opus":1.0,"sonnet":1.0}"#,
        );

        assert!(summary.contains("fable 2"), "summary: {summary}");
    }

    #[test]
    fn build_kiro_billable_multiplier_override_json_only_emits_changed_families() {
        let override_json = build_kiro_billable_multiplier_override_json(
            r#"{"haiku":1.0,"opus":2.0,"sonnet":1.0}"#,
            r#"{"haiku":0.8,"opus":2.0,"sonnet":1.3}"#,
        )
        .expect("override json should build")
        .expect("changed families should emit override json");
        let override_value: serde_json::Value =
            serde_json::from_str(&override_json).expect("override json should parse");

        assert_eq!(
            override_value,
            json!({
                "haiku": 0.8,
                "sonnet": 1.3
            })
        );
    }

    #[test]
    fn build_kiro_billable_multiplier_override_patch_clears_when_restoring_inherit() {
        let patch = build_kiro_billable_multiplier_override_patch(
            r#"{"haiku":1.0,"opus":2.0,"sonnet":1.0}"#,
            true,
            r#"{"haiku":1.0,"opus":1.5,"sonnet":1.0}"#,
            false,
            r#"{"haiku":1.0,"opus":1.5,"sonnet":1.0}"#,
        )
        .expect("patch should build");

        assert_eq!(patch, Some(None));
    }

    #[test]
    fn should_reset_kiro_cache_policy_editor_resets_on_initial_load() {
        let persisted = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    },
                    {
                        "credit_start": 1.0,
                        "credit_end": 2.5,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.0
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0
            }"#,
        )
        .expect("policy should parse");
        let mut draft = persisted.clone();
        draft.credit_end = "2.2".to_string();

        assert!(should_reset_kiro_cache_policy_editor(true, &draft, &persisted));
    }

    #[test]
    fn should_reset_kiro_cache_policy_editor_preserves_unsaved_draft() {
        let persisted = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    },
                    {
                        "credit_start": 1.0,
                        "credit_end": 2.5,
                        "cache_ratio_start": 0.2,
                        "cache_ratio_end": 0.0
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0
            }"#,
        )
        .expect("policy should parse");
        let mut draft = persisted.clone();
        draft.credit_end = "2.2".to_string();

        assert!(!should_reset_kiro_cache_policy_editor(false, &draft, &persisted,));
    }

    #[test]
    fn parse_kiro_cache_policy_form_json_allows_backend_validated_policy_rules() {
        let form = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 0,
                    "credit_start": 2.0,
                    "credit_end": 1.0
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 1.5,
                        "credit_end": 1.0,
                        "cache_ratio_start": 1.2,
                        "cache_ratio_end": -0.1
                    }
                ],
                "high_credit_diagnostic_threshold": -3.0
            }"#,
        )
        .expect("frontend should only require parseable numeric fields");

        assert_eq!(form.target_input_tokens, "0");
        assert_eq!(form.credit_start, "2.0");
        assert_eq!(form.credit_end, "1.0");
        assert_eq!(form.high_credit_diagnostic_threshold, "-3.0");
        assert_eq!(form.bands.len(), 1);
    }

    #[test]
    fn parse_kiro_cache_policy_form_json_accepts_anthropic_cache_creation_input_ratio() {
        let form = parse_kiro_cache_policy_form_json(
            r#"{
                "small_input_high_credit_boost": {
                    "target_input_tokens": 100000,
                    "credit_start": 1.0,
                    "credit_end": 1.8
                },
                "prefix_tree_credit_ratio_bands": [
                    {
                        "credit_start": 0.3,
                        "credit_end": 1.0,
                        "cache_ratio_start": 0.7,
                        "cache_ratio_end": 0.2
                    }
                ],
                "high_credit_diagnostic_threshold": 2.0,
                "anthropic_cache_creation_input_ratio": 0.25
            }"#,
        )
        .expect("policy should parse");

        assert_eq!(form.anthropic_cache_creation_input_ratio, "0.25");
    }

    #[test]
    fn kiro_tab_route_round_trips_section_ids() {
        use crate::router::Route;

        assert_eq!(kiro_tab_route("overview"), Route::AdminKiroGateway);
        assert_eq!(kiro_tab_route(TAB_ACCOUNTS), Route::AdminKiroGatewayAccountsManage);
        assert_eq!(kiro_tab_route(TAB_KEYS), Route::AdminKiroGatewayKeys);
        assert_eq!(kiro_tab_route(TAB_GROUPS), Route::AdminKiroGatewayGroups);
        assert_eq!(kiro_tab_route(TAB_USAGE), Route::AdminKiroGatewayUsage);
        assert_eq!(kiro_tab_route("unknown"), Route::AdminKiroGateway);
    }

    #[test]
    fn kiro_cache_token_percent_handles_empty_limit() {
        assert_eq!(kiro_cache_token_percent(12, 0), 0.0);
    }

    #[test]
    fn format_compact_bytes_formats_runtime_cache_memory() {
        assert_eq!(format_compact_bytes(512), "512 B");
        assert_eq!(format_compact_bytes(1536), "1.5 KiB");
        assert_eq!(format_compact_bytes(2 * 1024 * 1024), "2.0 MiB");
    }

    #[test]
    fn parse_manual_usage_limit_input_distinguishes_clear_from_zero() {
        assert_eq!(parse_manual_usage_limit_input("   ").expect("blank clears"), None);
        assert_eq!(parse_manual_usage_limit_input("0").expect("zero is valid"), Some(0.0));
        assert_eq!(
            parse_manual_usage_limit_input("500.25").expect("positive value is valid"),
            Some(500.25)
        );
        assert!(parse_manual_usage_limit_input("-1").is_err());
        assert!(parse_manual_usage_limit_input("NaN").is_err());
    }

    #[test]
    fn patch_kiro_account_input_serializes_manual_usage_limit_patch_semantics() {
        let body = serde_json::to_value(PatchKiroAccountInput {
            manual_usage_limit: None,
            ..PatchKiroAccountInput::default()
        })
        .expect("patch body should serialize");
        assert!(!body
            .as_object()
            .expect("patch body should be object")
            .contains_key("manual_usage_limit"));

        let body = serde_json::to_value(PatchKiroAccountInput {
            manual_usage_limit: Some(None),
            ..PatchKiroAccountInput::default()
        })
        .expect("patch body should serialize");
        assert_eq!(body.get("manual_usage_limit"), Some(&serde_json::Value::Null));
    }

    fn test_group(id: &str, account_names: &[&str]) -> AdminAccountGroupOptionView {
        AdminAccountGroupOptionView {
            id: id.to_string(),
            provider_type: "kiro".to_string(),
            name: id.to_string(),
            account_count: account_names.len(),
            single_account_name: (account_names.len() == 1).then(|| account_names[0].to_string()),
        }
    }
}
