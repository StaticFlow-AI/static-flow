use super::*;

/// Public key metadata exposed on the read-only LLM access page.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LlmGatewayPublicKeyView {
    pub id: String,
    pub name: String,
    pub secret: String,
    pub quota_billable_limit: u64,
    pub usage_input_uncached_tokens: u64,
    pub usage_input_cached_tokens: u64,
    pub usage_output_tokens: u64,
    pub remaining_billable: i64,
    pub last_used_at: Option<i64>,
}

/// Public payload returned by `/api/llm-gateway/access`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct LlmGatewayAccessResponse {
    pub base_url: String,
    pub gateway_path: String,
    pub model_catalog_path: String,
    pub auth_cache_ttl_seconds: u64,
    pub keys: Vec<LlmGatewayPublicKeyView>,
    pub generated_at: i64,
}

/// Cached public payload used for the first paint of `/llm-access`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct LlmGatewayPublicPageResponse {
    pub access: LlmGatewayAccessResponse,
    pub account_contributions: PublicLlmGatewayAccountContributionsResponse,
    pub support_config: LlmGatewaySupportConfigView,
    pub sponsors: PublicLlmGatewaySponsorsResponse,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct PublicLlmGatewayUsageLookupRequest {
    pub api_key: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct PublicLlmGatewayUsageKeyView {
    pub name: String,
    pub provider_type: String,
    pub quota_billable_limit: u64,
    pub usage_input_uncached_tokens: u64,
    pub usage_input_cached_tokens: u64,
    pub usage_output_tokens: u64,
    pub usage_billable_tokens: u64,
    pub usage_credit_total: f64,
    pub usage_credit_missing_events: u64,
    pub remaining_billable: i64,
    pub last_used_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct PublicLlmGatewayUsageEventView {
    pub id: String,
    pub key_name: String,
    pub account_name: Option<String>,
    pub request_method: String,
    pub request_url: String,
    pub latency_ms: i32,
    pub routing_wait_ms: Option<i32>,
    pub upstream_headers_ms: Option<i32>,
    pub post_headers_body_ms: Option<i32>,
    pub request_body_bytes: Option<u64>,
    pub request_body_read_ms: Option<i32>,
    pub request_json_parse_ms: Option<i32>,
    pub pre_handler_ms: Option<i32>,
    pub first_sse_write_ms: Option<i32>,
    pub stream_finish_ms: Option<i32>,
    pub stream_completed_cleanly: Option<bool>,
    pub downstream_disconnect: Option<bool>,
    pub final_event_type: Option<String>,
    pub bytes_streamed: Option<u64>,
    pub other_latency_ms: Option<i32>,
    pub quota_failover_count: u64,
    #[serde(default)]
    pub same_account_retry_count: u64,
    #[serde(default)]
    pub same_account_retry_delay_ms: i64,
    #[serde(default)]
    pub same_account_retry_reasons: Vec<String>,
    pub endpoint: String,
    pub model: Option<String>,
    pub status_code: i32,
    pub input_uncached_tokens: u64,
    pub input_cached_tokens: u64,
    pub output_tokens: u64,
    pub billable_tokens: u64,
    pub usage_missing: bool,
    pub credit_usage: Option<f64>,
    pub credit_usage_missing: bool,
    pub client_ip: String,
    pub ip_region: String,
    /// Inline error message surfaced directly in the list (no detail view
    /// needed).
    #[serde(default)]
    pub error_message: Option<String>,
    /// Stable upstream error class for failed requests, when classified.
    #[serde(default)]
    pub error_class: Option<String>,
    /// Whether this event belongs to a permanently rejected Codex session.
    #[serde(default)]
    pub session_blocked: bool,
    /// Number of images returned by a Codex image generation/edit request.
    #[serde(default)]
    pub response_image_count: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct PublicLlmGatewayUsageChartPointView {
    pub bucket_start_ms: i64,
    pub tokens: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct PublicLlmGatewayUsageLookupResponse {
    pub key: PublicLlmGatewayUsageKeyView,
    pub chart_points: Vec<PublicLlmGatewayUsageChartPointView>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub totals: AdminUsageTotalsView,
    pub events: Vec<PublicLlmGatewayUsageEventView>,
    pub generated_at: i64,
}

/// One public usage window from the cached Codex limit snapshot.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LlmGatewayRateLimitWindowView {
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub window_duration_mins: Option<i64>,
    pub resets_at: Option<i64>,
}

/// Optional credits metadata included in the cached status payload.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LlmGatewayCreditsView {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

/// One public rate-limit bucket rendered on `/llm-access`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LlmGatewayRateLimitBucketView {
    pub limit_id: String,
    pub limit_name: Option<String>,
    pub display_name: String,
    pub is_primary: bool,
    pub plan_type: Option<String>,
    pub primary: Option<LlmGatewayRateLimitWindowView>,
    pub secondary: Option<LlmGatewayRateLimitWindowView>,
    pub credits: Option<LlmGatewayCreditsView>,
    pub account_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LlmGatewayPublicAccountStatusView {
    pub name: String,
    pub status: String,
    pub plan_type: Option<String>,
    pub primary_remaining_percent: Option<f64>,
    pub secondary_remaining_percent: Option<f64>,
    pub last_usage_checked_at: Option<i64>,
    pub last_usage_success_at: Option<i64>,
    pub usage_error_message: Option<String>,
}

/// Cached public rate-limit status for the upstream Codex account.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LlmGatewayRateLimitStatusResponse {
    pub status: String,
    pub refresh_interval_seconds: u64,
    pub last_checked_at: Option<i64>,
    pub last_success_at: Option<i64>,
    pub source_url: String,
    pub error_message: Option<String>,
    #[serde(default)]
    pub accounts: Vec<LlmGatewayPublicAccountStatusView>,
    pub buckets: Vec<LlmGatewayRateLimitBucketView>,
}

const fn default_true() -> bool {
    true
}

const fn default_codex_image_generation_max_concurrency() -> u64 {
    3
}

const fn default_codex_account_rpm_limit() -> u64 {
    llm_store::DEFAULT_CODEX_ACCOUNT_RPM_LIMIT
}

const fn default_kiro_channel_rpm_limit() -> u64 {
    llm_store::DEFAULT_KIRO_CHANNEL_RPM_LIMIT
}

const fn default_anthropic_upstream_rpm_limit() -> u64 {
    llm_store::DEFAULT_ANTHROPIC_UPSTREAM_RPM_LIMIT
}

fn default_kiro_pool_strategy() -> String {
    llm_store::default_kiro_pool_strategy()
}

fn default_anthropic_upstream_pool_mode() -> String {
    llm_store::default_anthropic_upstream_pool_mode()
}

fn default_kiro_cache_policy_json() -> String {
    r#"{"small_input_high_credit_boost":{"target_input_tokens":100000,"credit_start":1.0,"credit_end":1.8},"prefix_tree_credit_ratio_bands":[{"credit_start":0.3,"credit_end":1.0,"cache_ratio_start":0.7,"cache_ratio_end":0.2},{"credit_start":1.0,"credit_end":2.5,"cache_ratio_start":0.2,"cache_ratio_end":0.0}],"high_credit_diagnostic_threshold":2.0}"#.to_string()
}

fn default_kiro_billable_model_multipliers_json() -> String {
    llm_store::default_kiro_billable_model_multipliers_json()
}

/// Admin-only editable representation of a gateway key.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayKeyView {
    pub id: String,
    pub name: String,
    pub secret: String,
    pub key_hash: String,
    pub status: String,
    pub provider_type: String,
    pub public_visible: bool,
    pub quota_billable_limit: u64,
    pub usage_input_uncached_tokens: u64,
    pub usage_input_cached_tokens: u64,
    pub usage_output_tokens: u64,
    pub usage_credit_total: f64,
    pub usage_credit_missing_events: u64,
    #[serde(default)]
    pub codex_image_usage_tokens: u64,
    #[serde(default)]
    pub codex_image_usage_missing_events: u64,
    #[serde(default)]
    pub codex_image_last_used_at: Option<i64>,
    pub remaining_billable: i64,
    pub last_used_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub route_strategy: Option<String>,
    pub account_group_id: Option<String>,
    pub fixed_account_name: Option<String>,
    pub auto_account_names: Option<Vec<String>>,
    #[serde(default = "default_kiro_pool_strategy")]
    pub preferred_pool_strategy: String,
    #[serde(default = "default_anthropic_upstream_pool_mode")]
    pub kiro_anthropic_upstream_pool_mode: String,
    pub model_name_map: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub kiro_model_group_preferences: BTreeMap<String, String>,
    #[serde(default)]
    pub kiro_model_channel_preferences: BTreeMap<String, String>,
    pub request_max_concurrency: Option<u64>,
    pub request_min_start_interval_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub moderation_enabled: bool,
    #[serde(default = "default_true")]
    pub codex_fast_enabled: bool,
    #[serde(default = "default_true")]
    pub codex_responses_lite_enabled: bool,
    #[serde(default)]
    pub codex_strict_session_rejection_enabled: bool,
    #[serde(default = "default_true")]
    pub codex_image_generation_enabled: bool,
    #[serde(default = "default_true")]
    pub codex_image_standalone_generation_enabled: bool,
    #[serde(default)]
    pub codex_image_direct_generation_enabled: bool,
    #[serde(default = "default_true")]
    pub kiro_request_validation_enabled: bool,
    #[serde(default = "default_true")]
    pub kiro_cache_estimation_enabled: bool,
    #[serde(default)]
    pub kiro_zero_cache_debug_enabled: bool,
    #[serde(default)]
    pub kiro_full_request_logging_enabled: bool,
    #[serde(default)]
    pub kiro_remote_media_resolution_enabled: bool,
    #[serde(default = "default_true")]
    pub kiro_latency_routing_enabled: bool,
    #[serde(default)]
    pub kiro_protected_content_validation_enabled: bool,
    #[serde(default)]
    pub kiro_cctest_text_handling_enabled: bool,
    #[serde(default)]
    pub kiro_cache_policy_override_json: Option<String>,
    #[serde(default)]
    pub kiro_billable_model_multipliers_override_json: Option<String>,
    #[serde(default = "default_kiro_cache_policy_json")]
    pub effective_kiro_cache_policy_json: String,
    #[serde(default = "default_true")]
    pub uses_global_kiro_cache_policy: bool,
    #[serde(default = "default_kiro_billable_model_multipliers_json")]
    pub effective_kiro_billable_model_multipliers_json: String,
    #[serde(default = "default_true")]
    pub uses_global_kiro_billable_model_multipliers: bool,
    #[serde(default)]
    pub kiro_candidate_credit_summary: Option<AdminKiroKeyCandidateCreditSummaryView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(default)]
pub struct AdminKiroKeyCandidateCreditSummaryView {
    pub candidate_count: usize,
    pub preferred_pool_candidate_count: Option<usize>,
    pub loaded_balance_count: usize,
    pub missing_balance_count: usize,
    pub total_limit: f64,
    pub total_remaining: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayKeysSummaryView {
    pub total: usize,
    pub public_visible_count: usize,
    pub active_count: usize,
    pub disabled_count: usize,
    pub quota_billable_limit_sum: u64,
    pub remaining_billable_sum: i64,
    pub usage_input_uncached_tokens_sum: u64,
    pub usage_input_cached_tokens_sum: u64,
    pub usage_output_tokens_sum: u64,
    pub usage_billable_tokens_sum: u64,
    pub usage_credit_total: f64,
    pub usage_credit_missing_events: u64,
    #[serde(default)]
    pub codex_image_usage_tokens_sum: u64,
    #[serde(default)]
    pub codex_image_usage_missing_events: u64,
}

/// Combined admin payload for the key inventory screen.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayKeysResponse {
    pub keys: Vec<AdminLlmGatewayKeyView>,
    pub summary: AdminLlmGatewayKeysSummaryView,
    pub auth_cache_ttl_seconds: u64,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
    pub generated_at: i64,
}

const ADMIN_GATEWAY_INVENTORY_PAGE_LIMIT: usize = 200;

pub(super) fn merge_admin_codex_account_pages(
    mut first: AccountListResponse,
    mut next: AccountListResponse,
) -> AccountListResponse {
    first.accounts.append(&mut next.accounts);
    first.summary = next.summary;
    first.total = next.total;
    first.generated_at = next.generated_at;
    first.has_more = next.has_more;
    first.offset = 0;
    first.limit = first.accounts.len();
    first
}

fn merge_admin_kiro_account_pages(
    mut first: AdminKiroAccountsResponse,
    mut next: AdminKiroAccountsResponse,
) -> AdminKiroAccountsResponse {
    first.accounts.append(&mut next.accounts);
    first.summary = next.summary;
    first.total = next.total;
    first.generated_at = next.generated_at;
    first.has_more = next.has_more;
    first.offset = 0;
    first.limit = first.accounts.len();
    first
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminAccountGroupView {
    pub id: String,
    pub provider_type: String,
    pub name: String,
    pub account_names: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminAccountGroupsResponse {
    pub groups: Vec<AdminAccountGroupView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminAccountGroupOptionView {
    pub id: String,
    pub provider_type: String,
    pub name: String,
    pub account_count: usize,
    pub single_account_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
struct AdminAccountGroupOptionsResponse {
    pub options: Vec<AdminAccountGroupOptionView>,
    pub generated_at: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdminLlmGatewayKeyPageQuery {
    pub q: Option<String>,
    pub active_only: bool,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdminLlmGatewayAccountPageQuery {
    pub q: Option<String>,
    pub active_only: bool,
    pub unhealthy_only: bool,
    pub sort: Option<String>,
}

/// Summary usage event used by admin paging and filtering views.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageEventView {
    pub id: String,
    pub key_id: String,
    pub key_name: String,
    pub account_name: Option<String>,
    pub request_method: String,
    pub request_url: String,
    pub latency_ms: i32,
    pub routing_wait_ms: Option<i32>,
    pub upstream_headers_ms: Option<i32>,
    pub post_headers_body_ms: Option<i32>,
    pub request_body_bytes: Option<u64>,
    pub request_body_read_ms: Option<i32>,
    pub request_json_parse_ms: Option<i32>,
    pub pre_handler_ms: Option<i32>,
    pub first_sse_write_ms: Option<i32>,
    pub stream_finish_ms: Option<i32>,
    pub stream_completed_cleanly: Option<bool>,
    pub downstream_disconnect: Option<bool>,
    pub final_event_type: Option<String>,
    pub bytes_streamed: Option<u64>,
    pub other_latency_ms: Option<i32>,
    pub quota_failover_count: u64,
    #[serde(default)]
    pub same_account_retry_count: u64,
    #[serde(default)]
    pub same_account_retry_delay_ms: i64,
    #[serde(default)]
    pub same_account_retry_reasons: Vec<String>,
    pub routing_diagnostics_json: Option<String>,
    pub endpoint: String,
    pub model: Option<String>,
    pub status_code: i32,
    pub input_uncached_tokens: u64,
    pub input_cached_tokens: u64,
    pub output_tokens: u64,
    pub billable_tokens: u64,
    pub usage_missing: bool,
    pub credit_usage: Option<f64>,
    pub credit_usage_missing: bool,
    pub client_ip: String,
    pub ip_region: String,
    pub last_message_content: Option<String>,
    /// Inline error message surfaced directly in the list (no detail view
    /// needed).
    #[serde(default)]
    pub error_message: Option<String>,
    /// Stable upstream error class for failed requests, when classified.
    #[serde(default)]
    pub error_class: Option<String>,
    /// Whether this event belongs to a permanently rejected Codex session.
    #[serde(default)]
    pub session_blocked: bool,
    /// Number of images returned by a Codex image generation/edit request.
    #[serde(default)]
    pub response_image_count: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageEventDetailView {
    pub id: String,
    pub key_id: String,
    pub key_name: String,
    pub account_name: Option<String>,
    pub request_method: String,
    pub request_url: String,
    pub latency_ms: i32,
    pub routing_wait_ms: Option<i32>,
    pub upstream_headers_ms: Option<i32>,
    pub post_headers_body_ms: Option<i32>,
    pub request_body_bytes: Option<u64>,
    pub request_body_read_ms: Option<i32>,
    pub request_json_parse_ms: Option<i32>,
    pub pre_handler_ms: Option<i32>,
    pub first_sse_write_ms: Option<i32>,
    pub stream_finish_ms: Option<i32>,
    pub stream_completed_cleanly: Option<bool>,
    pub downstream_disconnect: Option<bool>,
    pub final_event_type: Option<String>,
    pub bytes_streamed: Option<u64>,
    pub other_latency_ms: Option<i32>,
    pub quota_failover_count: u64,
    #[serde(default)]
    pub same_account_retry_count: u64,
    #[serde(default)]
    pub same_account_retry_delay_ms: i64,
    #[serde(default)]
    pub same_account_retry_reasons: Vec<String>,
    pub routing_diagnostics_json: Option<String>,
    pub endpoint: String,
    pub model: Option<String>,
    pub status_code: i32,
    pub input_uncached_tokens: u64,
    pub input_cached_tokens: u64,
    pub output_tokens: u64,
    pub billable_tokens: u64,
    pub usage_missing: bool,
    pub credit_usage: Option<f64>,
    pub credit_usage_missing: bool,
    pub client_ip: String,
    pub ip_region: String,
    pub request_headers_json: String,
    pub upstream_request: AdminUsageUpstreamRequestView,
    pub last_message_content: Option<String>,
    pub client_request_body_json: Option<String>,
    pub upstream_request_body_json: Option<String>,
    pub full_request_json: Option<String>,
    pub error_message: Option<String>,
    pub error_class: Option<String>,
    pub session_blocked: bool,
    #[serde(default)]
    pub response_image_count: Option<i64>,
    pub error_body: Option<String>,
    pub response_body: Option<String>,
    pub created_at: i64,
}

/// Lightweight metadata for the final request sent to the provider.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageUpstreamRequestView {
    pub method: Option<String>,
    pub url: Option<String>,
    pub headers_json: Option<String>,
}

/// Paginated usage-event response from the admin diagnostics endpoint.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageEventsResponse {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub current_rpm: u32,
    pub current_in_flight: u32,
    #[serde(default = "default_usage_analytics_retention_days")]
    pub retention_days: u64,
    pub totals: AdminUsageTotalsView,
    pub events: Vec<AdminLlmGatewayUsageEventView>,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageFilterOptionsResponse {
    pub models: Vec<String>,
    pub accounts: Vec<String>,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageMetricsSummaryView {
    pub total_requests: u64,
    pub ok_requests: u64,
    pub non_ok_requests: u64,
    pub distinct_accounts: usize,
    pub distinct_proxies: usize,
    pub first_token_samples: u64,
    pub avg_first_token_ms: Option<f64>,
    pub max_first_token_ms: Option<i64>,
    pub avg_latency_ms: Option<f64>,
    pub avg_routing_wait_ms: Option<f64>,
    pub failover_request_count: u64,
    pub total_quota_failovers: u64,
    pub downstream_disconnect_count: u64,
    pub usage_missing_count: u64,
    pub credit_usage_missing_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageMetricsDimensionView {
    pub key: String,
    pub label: String,
    pub account_name: Option<String>,
    pub proxy_config_id: Option<String>,
    pub proxy_config_name: Option<String>,
    pub proxy_url: Option<String>,
    pub proxy_source: Option<String>,
    pub request_count: u64,
    pub ok_count: u64,
    pub non_ok_count: u64,
    pub first_token_samples: u64,
    pub avg_first_token_ms: Option<f64>,
    pub max_first_token_ms: Option<i64>,
    pub routing_wait_samples: u64,
    pub avg_routing_wait_ms: Option<f64>,
    pub max_routing_wait_ms: Option<i64>,
    pub failover_request_count: u64,
    pub total_quota_failovers: u64,
    pub downstream_disconnect_count: u64,
    pub usage_missing_count: u64,
    pub credit_usage_missing_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageMetricsStatusCodeView {
    pub status_code: i32,
    pub request_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageMetricsResponse {
    pub generated_at_ms: i64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub provider_type: Option<String>,
    pub source: String,
    pub summary: AdminLlmGatewayUsageMetricsSummaryView,
    pub top_first_token_accounts: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_first_token_proxies: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_non_ok_accounts: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_non_ok_proxies: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_routing_wait_accounts: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_routing_wait_proxies: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_failover_accounts: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_failover_proxies: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_disconnect_accounts: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub top_disconnect_proxies: Vec<AdminLlmGatewayUsageMetricsDimensionView>,
    pub non_ok_status_codes: Vec<AdminLlmGatewayUsageMetricsStatusCodeView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayProxyTrafficTotalsView {
    pub event_count: u64,
    pub request_bytes: u64,
    pub response_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminProxyTrafficSnapshotView {
    pub refreshed_at_ms: i64,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
    pub retention_days: u64,
    pub totals: AdminLlmGatewayProxyTrafficTotalsView,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayProxyTrafficPointView {
    pub bucket_start_ms: i64,
    pub event_count: u64,
    pub request_bytes: u64,
    pub response_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayProxyTrafficProxySummaryView {
    pub proxy_key: String,
    pub proxy_config_id: Option<String>,
    pub proxy_config_name: Option<String>,
    pub proxy_url: Option<String>,
    pub proxy_source: Option<String>,
    pub totals: AdminLlmGatewayProxyTrafficTotalsView,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayProxyTrafficResponse {
    pub generated_at_ms: i64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub provider_type: Option<String>,
    pub source: String,
    pub proxy_config_id: Option<String>,
    pub bucket_ms: i64,
    pub totals: AdminLlmGatewayProxyTrafficTotalsView,
    pub points: Vec<AdminLlmGatewayProxyTrafficPointView>,
    pub proxies: Vec<AdminLlmGatewayProxyTrafficProxySummaryView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayProxyTrafficQuery {
    pub proxy_config_id: Option<String>,
    pub provider_type: Option<String>,
    pub source: Option<String>,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub bucket_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageTotalsView {
    pub event_count: usize,
    pub input_uncached_tokens: u64,
    pub input_cached_tokens: u64,
    pub output_tokens: u64,
    pub billable_tokens: u64,
    /// Sum of credit across all matches whose credit could be aggregated.
    /// Zero when talking to a server predating credit aggregation.
    pub credit_total: f64,
    /// Matches whose credit is not included in `credit_total`.
    pub credit_missing_events: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct ProcessMemoryRuntimeStats {
    pub rss_bytes: Option<u64>,
    pub virtual_bytes: Option<u64>,
    pub cgroup_current_bytes: Option<u64>,
    pub cgroup_peak_bytes: Option<u64>,
    pub cgroup_high_bytes: Option<u64>,
    pub cgroup_max_bytes: Option<u64>,
    pub cgroup_swap_current_bytes: Option<u64>,
    pub cgroup_swap_max_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageJournalFileView {
    pub file_name: String,
    pub path: String,
    pub sequence: Option<u64>,
    pub bytes: u64,
    pub age_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageWorkerProgressView {
    pub state: String,
    pub current_file_path: Option<String>,
    pub current_file_sequence: Option<u64>,
    pub processed_blocks: u64,
    pub total_blocks: u64,
    pub processed_events: u64,
    pub total_events: u64,
    pub processed_compressed_bytes: u64,
    pub total_compressed_bytes: u64,
    pub progress_percent: f64,
    pub import_rate_events_per_second: f64,
    pub heartbeat_age_ms: Option<i64>,
    pub last_successful_file_sequence: Option<u64>,
    pub last_successful_import_at_ms: Option<i64>,
    pub last_error: Option<String>,
    pub last_error_at_ms: Option<i64>,
    pub process_memory: ProcessMemoryRuntimeStats,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageJournalClusterView {
    pub node_id: String,
    pub node_class: String,
    pub runtime_role: String,
    pub primary_node_id: Option<String>,
    pub usage_query_mode: String,
    pub primary_worker_base_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageJournalStatusView {
    pub cluster: Option<AdminUsageJournalClusterView>,
    pub journal_enabled: bool,
    pub journal_root: String,
    pub current_rpm: u32,
    pub current_in_flight: u32,
    pub active_file_sequence: Option<u64>,
    pub active_file_bytes: u64,
    pub sealed_file_count: u64,
    pub sealed_bytes: u64,
    pub oldest_sealed_age_ms: Option<i64>,
    pub dropped_files_total: u64,
    pub dropped_unconsumed_files_total: u64,
    pub write_failures_total: u64,
    pub usage_query_base_url: String,
    pub producer_current_file: Option<AdminUsageJournalFileView>,
    pub orphan_active_files: Vec<AdminUsageJournalFileView>,
    pub current_consuming_file: Option<AdminUsageJournalFileView>,
    pub orphan_consuming_files: Vec<AdminUsageJournalFileView>,
    pub active_files: Vec<AdminUsageJournalFileView>,
    pub sealed_files: Vec<AdminUsageJournalFileView>,
    pub consuming_files: Vec<AdminUsageJournalFileView>,
    pub bad_files: Vec<AdminUsageJournalFileView>,
    pub worker: AdminUsageWorkerProgressView,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageJournalPreviewEventView {
    pub event_id: String,
    pub created_at_ms: i64,
    pub provider_type: String,
    pub protocol_family: String,
    pub key_id: String,
    pub key_name: String,
    pub account_name: Option<String>,
    pub request_method: String,
    pub endpoint: String,
    pub model: Option<String>,
    pub mapped_model: Option<String>,
    pub status_code: i32,
    pub input_uncached_tokens: u64,
    pub input_cached_tokens: u64,
    pub output_tokens: u64,
    pub billable_tokens: u64,
    pub usage_missing: bool,
    pub credit_usage_missing: bool,
    pub last_message_content: Option<String>,
    pub final_event_type: Option<String>,
    pub stream_completed_cleanly: Option<bool>,
    pub downstream_disconnect: Option<bool>,
    pub bytes_streamed: Option<i64>,
    pub latency_ms: Option<i64>,
    pub first_sse_write_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageJournalPreviewFileView {
    pub path: String,
    pub file_sequence: u64,
    pub bytes_scanned: u64,
    pub complete_blocks: u64,
    pub truncated_tail: bool,
    pub total_events: usize,
    pub events: Vec<AdminUsageJournalPreviewEventView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUsageJournalPreviewResponse {
    pub journal_root: String,
    pub producer_current_file: Option<AdminUsageJournalFileView>,
    pub preview: Option<AdminUsageJournalPreviewFileView>,
    pub limit: usize,
    pub offset: usize,
    pub total: usize,
    pub has_more: bool,
    pub generated_at: i64,
}

/// Query options for paginating and filtering LLM gateway usage events.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AdminLlmGatewayUsageEventsQuery {
    pub key_id: Option<String>,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub source: Option<String>,
    pub model: Option<String>,
    pub account_name: Option<String>,
    pub endpoint: Option<String>,
    pub status_code: Option<i32>,
    pub status_kind: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayUsageMetricsQuery {
    pub provider_type: Option<String>,
    pub source: Option<String>,
    pub window: Option<String>,
    pub top_limit: Option<usize>,
}

/// Public acknowledgement returned after a token wish is queued.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubmitLlmGatewayTokenRequestResponse {
    pub request_id: String,
    pub status: String,
}

/// Public acknowledgement returned after an account contribution is queued.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubmitLlmGatewayAccountContributionRequestResponse {
    pub request_id: String,
    pub status: String,
}

/// Public thank-you card item for approved account contributions.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicLlmGatewayAccountContributionView {
    pub request_id: String,
    pub account_name: String,
    pub contributor_message: String,
    pub github_id: Option<String>,
    pub processed_at: Option<i64>,
}

/// Public response for approved account contribution cards.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PublicLlmGatewayAccountContributionsResponse {
    pub contributions: Vec<PublicLlmGatewayAccountContributionView>,
    pub generated_at: i64,
}

/// Public support/community config rendered on `/llm-access`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct LlmGatewaySupportConfigView {
    pub sponsor_title: String,
    pub sponsor_intro: String,
    pub group_name: String,
    pub qq_group_number: String,
    pub group_invite_text: String,
    pub alipay_qr_url: String,
    pub wechat_qr_url: String,
    pub qq_group_qr_url: Option<String>,
    pub generated_at: i64,
}

/// Public form payload for contributing a Codex account.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubmitLlmGatewayAccountContributionInput {
    pub account_name: String,
    pub account_id: Option<String>,
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub requester_email: Option<String>,
    pub contributor_message: String,
    pub github_id: Option<String>,
    pub frontend_page_url: Option<String>,
}

/// Public form payload for contributing a GPT image account.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubmitGpt2ApiAccountContributionInput {
    pub account_name: String,
    pub access_token: Option<String>,
    pub session_json: Option<String>,
    pub requester_email: String,
    pub contributor_message: String,
    pub github_id: Option<String>,
    pub frontend_page_url: Option<String>,
}

/// Public acknowledgement returned after a GPT contribution is queued.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubmitGpt2ApiAccountContributionRequestResponse {
    pub request_id: String,
    pub status: String,
}

/// Public form payload for requesting to become a sponsor.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubmitLlmGatewaySponsorInput {
    pub requester_email: String,
    pub sponsor_message: String,
    pub display_name: Option<String>,
    pub github_id: Option<String>,
    pub frontend_page_url: Option<String>,
}

/// Public acknowledgement returned after a sponsor request is queued.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubmitLlmGatewaySponsorRequestResponse {
    pub request_id: String,
    pub status: String,
    pub payment_email_sent: bool,
}

/// Public thank-you card item for approved sponsors.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PublicLlmGatewaySponsorView {
    pub request_id: String,
    pub display_name: Option<String>,
    pub sponsor_message: String,
    pub github_id: Option<String>,
    pub processed_at: Option<i64>,
}

/// Public response for approved sponsor cards.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct PublicLlmGatewaySponsorsResponse {
    pub sponsors: Vec<PublicLlmGatewaySponsorView>,
    pub generated_at: i64,
}

/// Admin-only view of one token wish / issuance task.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminLlmGatewayTokenRequestView {
    pub request_id: String,
    pub requester_email: String,
    pub requested_quota_billable_limit: u64,
    pub request_reason: String,
    pub frontend_page_url: Option<String>,
    pub status: String,
    pub client_ip: String,
    pub ip_region: String,
    pub admin_note: Option<String>,
    pub failure_reason: Option<String>,
    pub issued_key_id: Option<String>,
    pub issued_key_name: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub processed_at: Option<i64>,
}

/// Paginated admin response for token wishes.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminLlmGatewayTokenRequestsResponse {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub requests: Vec<AdminLlmGatewayTokenRequestView>,
    pub generated_at: i64,
}

/// Admin-only view of one Codex account contribution request.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminLlmGatewayAccountContributionRequestView {
    pub request_id: String,
    pub account_name: String,
    pub account_id: Option<String>,
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub requester_email: String,
    pub contributor_message: String,
    pub github_id: Option<String>,
    pub frontend_page_url: Option<String>,
    pub status: String,
    pub client_ip: String,
    pub ip_region: String,
    pub admin_note: Option<String>,
    pub failure_reason: Option<String>,
    pub imported_account_name: Option<String>,
    pub issued_key_id: Option<String>,
    pub issued_key_name: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub processed_at: Option<i64>,
}

/// Paginated admin response for account contribution requests.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminLlmGatewayAccountContributionRequestsResponse {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub requests: Vec<AdminLlmGatewayAccountContributionRequestView>,
    pub generated_at: i64,
}

/// Query options for admin account contribution request listing.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AdminLlmGatewayAccountContributionRequestsQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Admin-only view of one GPT account contribution request.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminGpt2ApiAccountContributionRequestView {
    pub request_id: String,
    pub account_name: String,
    pub access_token: Option<String>,
    pub session_json: Option<String>,
    pub requester_email: String,
    pub contributor_message: String,
    pub github_id: Option<String>,
    pub frontend_page_url: Option<String>,
    pub status: String,
    pub client_ip: String,
    pub ip_region: String,
    pub admin_note: Option<String>,
    pub failure_reason: Option<String>,
    pub imported_account_name: Option<String>,
    pub issued_key_id: Option<String>,
    pub issued_key_name: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub processed_at: Option<i64>,
}

/// Paginated admin response for GPT account contribution requests.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminGpt2ApiAccountContributionRequestsResponse {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub requests: Vec<AdminGpt2ApiAccountContributionRequestView>,
    pub generated_at: i64,
}

/// Query options for admin GPT account contribution request listing.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AdminGpt2ApiAccountContributionRequestsQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Admin-only view of one sponsor request.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminLlmGatewaySponsorRequestView {
    pub request_id: String,
    pub requester_email: String,
    pub sponsor_message: String,
    pub display_name: Option<String>,
    pub github_id: Option<String>,
    pub frontend_page_url: Option<String>,
    pub status: String,
    pub client_ip: String,
    pub ip_region: String,
    pub admin_note: Option<String>,
    pub failure_reason: Option<String>,
    pub payment_email_sent_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub processed_at: Option<i64>,
}

/// Paginated admin response for sponsor requests.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminLlmGatewaySponsorRequestsResponse {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
    pub requests: Vec<AdminLlmGatewaySponsorRequestView>,
    pub generated_at: i64,
}

/// Query options for admin sponsor request listing.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AdminLlmGatewaySponsorRequestsQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Query options for admin token-wish listing.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct AdminLlmGatewayTokenRequestsQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Editable LLM gateway runtime settings exposed to the admin UI.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LlmGatewayRuntimeConfig {
    pub auth_cache_ttl_seconds: u64,
    pub max_request_body_bytes: u64,
    pub account_failure_retry_limit: u64,
    #[serde(default = "default_codex_client_version")]
    pub codex_client_version: String,
    pub codex_status_refresh_min_interval_seconds: u64,
    pub codex_status_refresh_max_interval_seconds: u64,
    pub codex_status_account_jitter_max_seconds: u64,
    #[serde(default = "default_codex_weight_free")]
    pub codex_weight_free: u64,
    #[serde(default = "default_codex_weight_plus")]
    pub codex_weight_plus: u64,
    #[serde(default = "default_codex_weight_pro5x")]
    pub codex_weight_pro5x: u64,
    #[serde(default = "default_codex_weight_pro20x")]
    pub codex_weight_pro20x: u64,
    #[serde(default = "default_codex_session_affinity_enabled")]
    pub codex_session_affinity_enabled: bool,
    #[serde(default = "default_codex_session_affinity_max_entries")]
    pub codex_session_affinity_max_entries: u64,
    #[serde(default = "default_codex_session_affinity_ttl_seconds")]
    pub codex_session_affinity_ttl_seconds: u64,
    #[serde(default = "default_codex_fallback_affinity_enabled")]
    pub codex_fallback_affinity_enabled: bool,
    #[serde(default = "default_codex_fallback_affinity_ttl_seconds")]
    pub codex_fallback_affinity_ttl_seconds: u64,
    #[serde(default = "default_codex_fallback_affinity_prefix_bytes")]
    pub codex_fallback_affinity_prefix_bytes: u64,
    #[serde(default = "default_codex_fallback_affinity_min_body_bytes")]
    pub codex_fallback_affinity_min_body_bytes: u64,
    pub kiro_status_refresh_min_interval_seconds: u64,
    pub kiro_status_refresh_max_interval_seconds: u64,
    pub kiro_status_account_jitter_max_seconds: u64,
    pub usage_event_flush_batch_size: u64,
    pub usage_event_flush_interval_seconds: u64,
    pub usage_event_flush_max_buffer_bytes: u64,
    #[serde(default = "default_duckdb_usage_memory_limit_mib")]
    pub duckdb_usage_memory_limit_mib: u64,
    #[serde(default = "default_duckdb_usage_checkpoint_threshold_mib")]
    pub duckdb_usage_checkpoint_threshold_mib: u64,
    #[serde(default = "default_usage_analytics_retention_days")]
    pub usage_analytics_retention_days: u64,
    #[serde(default = "default_true")]
    pub usage_journal_enabled: bool,
    #[serde(default = "default_usage_journal_max_file_bytes")]
    pub usage_journal_max_file_bytes: u64,
    #[serde(default = "default_usage_journal_max_file_age_ms")]
    pub usage_journal_max_file_age_ms: u64,
    #[serde(default = "default_usage_journal_max_files")]
    pub usage_journal_max_files: u64,
    #[serde(default = "default_usage_journal_block_target_uncompressed_bytes")]
    pub usage_journal_block_target_uncompressed_bytes: u64,
    #[serde(default = "default_usage_journal_block_max_events")]
    pub usage_journal_block_max_events: u64,
    #[serde(default = "default_usage_journal_fsync_interval_ms")]
    pub usage_journal_fsync_interval_ms: u64,
    #[serde(default = "default_usage_journal_zstd_level")]
    pub usage_journal_zstd_level: i64,
    #[serde(default = "default_usage_journal_consumer_lease_ms")]
    pub usage_journal_consumer_lease_ms: u64,
    #[serde(default)]
    pub usage_journal_delete_bad_files: bool,
    #[serde(default = "default_usage_query_bind_addr")]
    pub usage_query_bind_addr: String,
    #[serde(default = "default_usage_query_base_url")]
    pub usage_query_base_url: String,
    pub kiro_cache_kmodels_json: String,
    #[serde(default = "default_kiro_billable_model_multipliers_json")]
    pub kiro_billable_model_multipliers_json: String,
    #[serde(default = "default_kiro_cache_policy_json")]
    pub kiro_cache_policy_json: String,
    #[serde(default = "default_kiro_context_usage_min_request_tokens")]
    pub kiro_context_usage_min_request_tokens: u64,
    #[serde(default = "default_kiro_compact_trigger_tokens")]
    pub kiro_compact_trigger_tokens: u64,
    pub kiro_prefix_cache_mode: String,
    pub kiro_prefix_cache_max_tokens: u64,
    pub kiro_prefix_cache_entry_ttl_seconds: u64,
    pub kiro_conversation_anchor_max_entries: u64,
    pub kiro_conversation_anchor_ttl_seconds: u64,
    #[serde(default = "default_kiro_cache_snapshot_enabled")]
    pub kiro_cache_snapshot_enabled: bool,
    #[serde(default = "default_kiro_cache_snapshot_interval_seconds")]
    pub kiro_cache_snapshot_interval_seconds: u64,
    #[serde(default = "default_kiro_cache_snapshot_ttl_seconds")]
    pub kiro_cache_snapshot_ttl_seconds: u64,
    #[serde(default)]
    pub kiro_cache_snapshot_max_tokens: u64,
    #[serde(default)]
    pub kiro_cache_snapshot_max_anchor_entries: u64,
    #[serde(default)]
    pub kiro_cctest_proxy_base_url: Option<String>,
    #[serde(default)]
    pub kiro_cctest_proxy_api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUpstreamProxyConfigView {
    pub id: String,
    pub name: String,
    pub proxy_url: String,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub scope_node_id: Option<String>,
    pub effective_source: String,
    pub has_node_override: bool,
    pub can_edit_slot_metadata: bool,
    pub latest_codex_check: Option<AdminUpstreamProxyEndpointCheckView>,
    pub latest_kiro_check: Option<AdminUpstreamProxyEndpointCheckView>,
    pub traffic_snapshot: Option<AdminProxyTrafficSnapshotView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUpstreamProxyEndpointCheckView {
    pub target_url: String,
    pub reachable: bool,
    pub status_code: Option<u16>,
    pub latency_ms: i64,
    pub error_message: Option<String>,
    pub checked_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUpstreamProxyConfigsResponse {
    pub proxy_config_scope: AdminUpstreamProxyConfigScopeView,
    pub proxy_configs: Vec<AdminUpstreamProxyConfigView>,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminProxyTrafficRefreshResponse {
    pub proxy_config_id: String,
    pub traffic_snapshot: AdminProxyTrafficSnapshotView,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUpstreamProxyConfigScopeView {
    pub node_id: String,
    pub is_core: bool,
    pub can_edit_slot_metadata: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUpstreamProxyCheckTargetView {
    pub target: String,
    pub url: String,
    pub reachable: bool,
    pub status_code: Option<u16>,
    pub latency_ms: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUpstreamProxyCheckResponse {
    pub proxy_config_id: String,
    pub proxy_config_name: String,
    pub provider_type: String,
    pub auth_label: String,
    pub ok: bool,
    pub targets: Vec<AdminUpstreamProxyCheckTargetView>,
    pub checked_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUpstreamProxyBindingView {
    pub provider_type: String,
    pub effective_source: String,
    pub bound_proxy_config_id: Option<String>,
    pub effective_proxy_config_name: Option<String>,
    pub effective_proxy_url: Option<String>,
    pub effective_proxy_username: Option<String>,
    pub effective_proxy_password: Option<String>,
    pub binding_updated_at: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminUpstreamProxyBindingsResponse {
    pub bindings: Vec<AdminUpstreamProxyBindingView>,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLegacyKiroProxyMigrationResponse {
    pub created_configs: Vec<AdminUpstreamProxyConfigView>,
    pub reused_configs: Vec<AdminUpstreamProxyConfigView>,
    pub migrated_account_names: Vec<String>,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct CreateAdminUpstreamProxyConfigInput {
    pub name: String,
    pub proxy_url: String,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct PatchAdminUpstreamProxyConfigInput {
    pub name: Option<String>,
    pub proxy_url: Option<String>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub status: Option<String>,
}

/// Fetch the read-only public gateway access bundle used by `/llm-access`.
pub async fn fetch_llm_gateway_access() -> Result<LlmGatewayAccessResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(LlmGatewayAccessResponse {
            base_url: "http://localhost:3000/api/llm-gateway/v1".to_string(),
            gateway_path: "/api/llm-gateway/v1".to_string(),
            model_catalog_path: "/api/llm-gateway/model-catalog.json".to_string(),
            auth_cache_ttl_seconds: 60,
            keys: vec![],
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/llm-gateway/access?_ts={}", API_BASE, Date::now() as u64);
        let response = api_get(&url)
            .header("Cache-Control", "no-cache, no-store, max-age=0")
            .header("Pragma", "no-cache")
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Fetch the cached first-paint bundle for `/llm-access`.
pub async fn fetch_llm_gateway_public_page() -> Result<LlmGatewayPublicPageResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(LlmGatewayPublicPageResponse {
            access: LlmGatewayAccessResponse {
                base_url: "http://localhost:3000/api/llm-gateway/v1".to_string(),
                gateway_path: "/api/llm-gateway/v1".to_string(),
                model_catalog_path: "/api/llm-gateway/model-catalog.json".to_string(),
                auth_cache_ttl_seconds: 60,
                keys: vec![],
                generated_at: 0,
            },
            account_contributions: PublicLlmGatewayAccountContributionsResponse {
                contributions: vec![],
                generated_at: 0,
            },
            support_config: LlmGatewaySupportConfigView {
                sponsor_title: "请作者喝杯咖啡".to_string(),
                sponsor_intro: "填写邮箱后，系统会把赞助说明和收款码发给你。".to_string(),
                group_name: "美区词元魔盗团".to_string(),
                qq_group_number: "1092356490".to_string(),
                group_invite_text: "遇到 token、贡献或使用问题都可以进群交流。".to_string(),
                alipay_qr_url: "/api/llm-gateway/support-assets/alipay_qr.png".to_string(),
                wechat_qr_url: "/api/llm-gateway/support-assets/wechat_qr.png".to_string(),
                qq_group_qr_url: Some(
                    "/api/llm-gateway/support-assets/qq_group_qr.png".to_string(),
                ),
                generated_at: 0,
            },
            sponsors: PublicLlmGatewaySponsorsResponse {
                sponsors: vec![],
                generated_at: 0,
            },
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/llm-gateway/public-page", API_BASE);
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

#[cfg(any(not(feature = "mock"), test))]
pub(super) fn build_llm_gateway_model_catalog_url_for_ts(path: Option<&str>, ts: u64) -> String {
    let path = path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("/api/llm-gateway/model-catalog.json");
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("/api/") {
        format!("{path}?_ts={ts}")
    } else {
        format!("{API_BASE}{path}?_ts={ts}")
    }
}

#[cfg(not(feature = "mock"))]
pub fn build_llm_gateway_model_catalog_url(path: Option<&str>) -> String {
    build_llm_gateway_model_catalog_url_for_ts(path, Date::now() as u64)
}

pub async fn fetch_llm_gateway_model_catalog_json(
    model_catalog_path: Option<&str>,
) -> Result<String, String> {
    #[cfg(feature = "mock")]
    {
        let _ = model_catalog_path;
        Ok(r#"{"models":[{"slug":"gpt-5.5","visibility":"list","supported_in_api":true}]}"#
            .to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = build_llm_gateway_model_catalog_url(model_catalog_path);
        let response = api_get(&url)
            .header("Cache-Control", "no-cache, no-store, max-age=0")
            .header("Pragma", "no-cache")
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .text()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_public_llm_gateway_usage(
    request: &PublicLlmGatewayUsageLookupRequest,
) -> Result<PublicLlmGatewayUsageLookupResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = request;
        Ok(PublicLlmGatewayUsageLookupResponse {
            key: PublicLlmGatewayUsageKeyView {
                name: "mock-public-key".to_string(),
                provider_type: "codex".to_string(),
                quota_billable_limit: 10_000,
                usage_input_uncached_tokens: 2_500,
                usage_input_cached_tokens: 800,
                usage_output_tokens: 1_700,
                usage_billable_tokens: 4_200,
                usage_credit_total: 0.0,
                usage_credit_missing_events: 0,
                remaining_billable: 5_800,
                last_used_at: Some(1_775_000_000_000),
            },
            chart_points: (0..24)
                .map(|index| PublicLlmGatewayUsageChartPointView {
                    bucket_start_ms: 1_775_000_000_000 - ((23 - index) as i64 * 3_600_000),
                    tokens: if index % 4 == 0 { 480 } else { 120 + (index as u64 * 13) },
                })
                .collect(),
            total: 2,
            offset: request.offset.unwrap_or(0),
            limit: request.limit.unwrap_or(50),
            has_more: false,
            totals: AdminUsageTotalsView {
                event_count: 2,
                input_uncached_tokens: 730,
                input_cached_tokens: 64,
                output_tokens: 364,
                billable_tokens: 1_094,
            },
            events: vec![
                PublicLlmGatewayUsageEventView {
                    id: "mock-usage-2".to_string(),
                    key_name: "mock-public-key".to_string(),
                    account_name: Some("default".to_string()),
                    request_method: "POST".to_string(),
                    request_url: "/api/llm-gateway/v1/responses".to_string(),
                    latency_ms: 842,
                    routing_wait_ms: None,
                    upstream_headers_ms: None,
                    post_headers_body_ms: None,
                    request_body_bytes: None,
                    request_body_read_ms: None,
                    request_json_parse_ms: None,
                    pre_handler_ms: None,
                    first_sse_write_ms: None,
                    stream_finish_ms: None,
                    stream_completed_cleanly: None,
                    downstream_disconnect: None,
                    final_event_type: None,
                    bytes_streamed: None,
                    other_latency_ms: None,
                    quota_failover_count: 0,
                    same_account_retry_count: 0,
                    same_account_retry_delay_ms: 0,
                    same_account_retry_reasons: Vec::new(),
                    endpoint: "/responses".to_string(),
                    model: Some("gpt-5.3-codex".to_string()),
                    status_code: 200,
                    input_uncached_tokens: 420,
                    input_cached_tokens: 0,
                    output_tokens: 156,
                    billable_tokens: 576,
                    usage_missing: false,
                    credit_usage: None,
                    credit_usage_missing: false,
                    client_ip: "203.0.113.8".to_string(),
                    ip_region: "US".to_string(),
                    created_at: 1_775_000_000_000,
                },
                PublicLlmGatewayUsageEventView {
                    id: "mock-usage-1".to_string(),
                    key_name: "mock-public-key".to_string(),
                    account_name: Some("backup".to_string()),
                    request_method: "POST".to_string(),
                    request_url: "/api/llm-gateway/v1/responses".to_string(),
                    latency_ms: 1_204,
                    routing_wait_ms: None,
                    upstream_headers_ms: None,
                    post_headers_body_ms: None,
                    request_body_bytes: None,
                    request_body_read_ms: None,
                    request_json_parse_ms: None,
                    pre_handler_ms: None,
                    first_sse_write_ms: None,
                    stream_finish_ms: None,
                    stream_completed_cleanly: None,
                    downstream_disconnect: None,
                    final_event_type: None,
                    bytes_streamed: None,
                    other_latency_ms: None,
                    quota_failover_count: 0,
                    same_account_retry_count: 0,
                    same_account_retry_delay_ms: 0,
                    same_account_retry_reasons: Vec::new(),
                    endpoint: "/responses".to_string(),
                    model: Some("gpt-5.3-codex".to_string()),
                    status_code: 200,
                    input_uncached_tokens: 310,
                    input_cached_tokens: 64,
                    output_tokens: 208,
                    billable_tokens: 518,
                    usage_missing: false,
                    credit_usage: None,
                    credit_usage_missing: false,
                    client_ip: "203.0.113.8".to_string(),
                    ip_region: "US".to_string(),
                    created_at: 1_774_996_400_000,
                },
            ],
            generated_at: 1_775_000_000_000,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/llm-gateway/public-usage/query", API_BASE);
        let response = api_post(&url)
            .header("Cache-Control", "no-cache, no-store, max-age=0")
            .json(request)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Fetch the cached public Codex rate-limit snapshot used by `/llm-access`.
pub async fn fetch_llm_gateway_status() -> Result<LlmGatewayRateLimitStatusResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(LlmGatewayRateLimitStatusResponse {
            status: "ready".to_string(),
            refresh_interval_seconds: 60,
            last_checked_at: Some(0),
            last_success_at: Some(0),
            source_url: "https://chatgpt.com/backend-api/wham/usage".to_string(),
            error_message: None,
            accounts: vec![
                LlmGatewayPublicAccountStatusView {
                    name: "default".to_string(),
                    status: "active".to_string(),
                    plan_type: Some("Pro".to_string()),
                    primary_remaining_percent: Some(62.0),
                    secondary_remaining_percent: Some(39.0),
                    last_usage_checked_at: Some(0),
                    last_usage_success_at: Some(0),
                    usage_error_message: None,
                },
                LlmGatewayPublicAccountStatusView {
                    name: "backup".to_string(),
                    status: "unavailable".to_string(),
                    plan_type: Some("Pro".to_string()),
                    primary_remaining_percent: Some(17.0),
                    secondary_remaining_percent: Some(5.0),
                    last_usage_checked_at: Some(0),
                    last_usage_success_at: Some(0),
                    usage_error_message: Some("upstream 503".to_string()),
                },
            ],
            buckets: vec![LlmGatewayRateLimitBucketView {
                limit_id: "codex".to_string(),
                limit_name: None,
                display_name: "codex".to_string(),
                is_primary: true,
                plan_type: Some("Pro".to_string()),
                primary: Some(LlmGatewayRateLimitWindowView {
                    used_percent: 38.0,
                    remaining_percent: 62.0,
                    window_duration_mins: Some(300),
                    resets_at: Some(0),
                }),
                secondary: Some(LlmGatewayRateLimitWindowView {
                    used_percent: 61.0,
                    remaining_percent: 39.0,
                    window_duration_mins: Some(10080),
                    resets_at: Some(0),
                }),
                credits: Some(LlmGatewayCreditsView {
                    has_credits: true,
                    unlimited: false,
                    balance: Some("24".to_string()),
                }),
                account_name: Some("default".to_string()),
            }],
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/llm-gateway/status?_ts={}", API_BASE, Date::now() as u64);
        let response = api_get(&url)
            .header("Cache-Control", "no-cache, no-store, max-age=0")
            .header("Pragma", "no-cache")
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Submit a public token wish from `/llm-access`.
pub async fn submit_llm_gateway_token_request(
    requested_quota_billable_limit: u64,
    request_reason: &str,
    requester_email: &str,
    frontend_page_url: Option<&str>,
) -> Result<SubmitLlmGatewayTokenRequestResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ =
            (requested_quota_billable_limit, request_reason, requester_email, frontend_page_url);
        Ok(SubmitLlmGatewayTokenRequestResponse {
            request_id: "mock-llm-wish-1".to_string(),
            status: "pending".to_string(),
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/llm-gateway/token-requests/submit", API_BASE);
        let mut body = serde_json::json!({
            "requested_quota_billable_limit": requested_quota_billable_limit,
            "request_reason": request_reason,
            "requester_email": requester_email,
        });
        if let Some(page_url) = frontend_page_url {
            body["frontend_page_url"] = serde_json::Value::String(page_url.to_string());
        }
        let response = api_post(&url)
            .json(&body)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Submit a public Codex account contribution request from `/llm-access`.
pub async fn submit_llm_gateway_account_contribution_request(
    input: &SubmitLlmGatewayAccountContributionInput,
) -> Result<SubmitLlmGatewayAccountContributionRequestResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = input;
        Ok(SubmitLlmGatewayAccountContributionRequestResponse {
            request_id: "mock-llm-account-contribution-1".to_string(),
            status: "pending".to_string(),
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/llm-gateway/account-contribution-requests/submit", API_BASE);
        let mut body = serde_json::json!({
            "account_name": input.account_name,
            "refresh_token": input.refresh_token,
            "contributor_message": input.contributor_message,
        });
        if !input.id_token.trim().is_empty() {
            body["id_token"] = serde_json::Value::String(input.id_token.trim().to_string());
        }
        if !input.access_token.trim().is_empty() {
            body["access_token"] = serde_json::Value::String(input.access_token.trim().to_string());
        }
        if let Some(email) = input
            .requester_email
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            body["requester_email"] = serde_json::Value::String(email.trim().to_string());
        }
        if let Some(account_id) = input
            .account_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            body["account_id"] = serde_json::Value::String(account_id.trim().to_string());
        }
        if let Some(github_id) = input
            .github_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            body["github_id"] = serde_json::Value::String(github_id.trim().to_string());
        }
        if let Some(page_url) = input.frontend_page_url.as_deref() {
            body["frontend_page_url"] = serde_json::Value::String(page_url.to_string());
        }
        let response = api_post(&url)
            .json(&body)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Submit a public GPT account contribution request from `/llm-access`.
pub async fn submit_gpt2api_account_contribution_request(
    input: &SubmitGpt2ApiAccountContributionInput,
) -> Result<SubmitGpt2ApiAccountContributionRequestResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = input;
        Ok(SubmitGpt2ApiAccountContributionRequestResponse {
            request_id: "mock-gpt2api-account-contribution-1".to_string(),
            status: "pending".to_string(),
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/gpt2api/account-contribution-requests/submit", API_BASE);
        let mut body = serde_json::json!({
            "account_name": input.account_name,
            "requester_email": input.requester_email,
            "contributor_message": input.contributor_message,
        });
        if let Some(access_token) = input
            .access_token
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            body["access_token"] = serde_json::Value::String(access_token.trim().to_string());
        }
        if let Some(session_json) = input
            .session_json
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            body["session_json"] = serde_json::Value::String(session_json.trim().to_string());
        }
        if let Some(github_id) = input
            .github_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            body["github_id"] = serde_json::Value::String(github_id.trim().to_string());
        }
        if let Some(page_url) = input.frontend_page_url.as_deref() {
            body["frontend_page_url"] = serde_json::Value::String(page_url.to_string());
        }
        let response = api_post(&url)
            .json(&body)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Submit a public sponsor request from `/llm-access`.
pub async fn submit_llm_gateway_sponsor_request(
    input: &SubmitLlmGatewaySponsorInput,
) -> Result<SubmitLlmGatewaySponsorRequestResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = input;
        Ok(SubmitLlmGatewaySponsorRequestResponse {
            request_id: "mock-llm-sponsor-1".to_string(),
            status: "payment_email_sent".to_string(),
            payment_email_sent: true,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/llm-gateway/sponsor-requests/submit", API_BASE);
        let mut body = serde_json::json!({
            "requester_email": input.requester_email,
            "sponsor_message": input.sponsor_message,
        });
        if let Some(display_name) = input
            .display_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            body["display_name"] = serde_json::Value::String(display_name.trim().to_string());
        }
        if let Some(github_id) = input
            .github_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            body["github_id"] = serde_json::Value::String(github_id.trim().to_string());
        }
        if let Some(page_url) = input.frontend_page_url.as_deref() {
            body["frontend_page_url"] = serde_json::Value::String(page_url.to_string());
        }
        let response = api_post(&url)
            .json(&body)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_music_runtime_config() -> Result<MusicRuntimeConfig, String> {
    #[cfg(feature = "mock")]
    {
        Ok(MusicRuntimeConfig {
            play_dedupe_window_seconds: 60,
            comment_rate_limit_seconds: 60,
            list_default_limit: 20,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/music-config", admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn update_admin_music_runtime_config(
    config: &MusicRuntimeConfig,
) -> Result<MusicRuntimeConfig, String> {
    #[cfg(feature = "mock")]
    {
        Ok(config.clone())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/music-config", admin_base());
        let response = api_post(&url)
            .header("Content-Type", "application/json")
            .json(config)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Fetch the current admin runtime configuration for the gateway cache.
pub async fn fetch_admin_llm_gateway_config() -> Result<LlmGatewayRuntimeConfig, String> {
    #[cfg(feature = "mock")]
    {
        Ok(LlmGatewayRuntimeConfig {
            auth_cache_ttl_seconds: 60,
            max_request_body_bytes: 8 * 1024 * 1024,
            account_failure_retry_limit: 10,
            codex_client_version: default_codex_client_version(),
            codex_status_refresh_min_interval_seconds: 240,
            codex_status_refresh_max_interval_seconds: 300,
            codex_status_account_jitter_max_seconds: 10,
            codex_weight_free: default_codex_weight_free(),
            codex_weight_plus: default_codex_weight_plus(),
            codex_weight_pro5x: default_codex_weight_pro5x(),
            codex_weight_pro20x: default_codex_weight_pro20x(),
            codex_session_affinity_enabled: default_codex_session_affinity_enabled(),
            codex_session_affinity_max_entries: default_codex_session_affinity_max_entries(),
            codex_session_affinity_ttl_seconds: default_codex_session_affinity_ttl_seconds(),
            codex_fallback_affinity_enabled: default_codex_fallback_affinity_enabled(),
            codex_fallback_affinity_ttl_seconds: default_codex_fallback_affinity_ttl_seconds(),
            codex_fallback_affinity_prefix_bytes: default_codex_fallback_affinity_prefix_bytes(),
            codex_fallback_affinity_min_body_bytes:
                default_codex_fallback_affinity_min_body_bytes(),
            kiro_status_refresh_min_interval_seconds: 240,
            kiro_status_refresh_max_interval_seconds: 300,
            kiro_status_account_jitter_max_seconds: 10,
            usage_event_flush_batch_size: 256,
            usage_event_flush_interval_seconds: 15,
            usage_event_flush_max_buffer_bytes: 8 * 1024 * 1024,
            duckdb_usage_memory_limit_mib: default_duckdb_usage_memory_limit_mib(),
            duckdb_usage_checkpoint_threshold_mib:
                default_duckdb_usage_checkpoint_threshold_mib(),
            usage_analytics_retention_days: default_usage_analytics_retention_days(),
            usage_journal_enabled: true,
            usage_journal_max_file_bytes: default_usage_journal_max_file_bytes(),
            usage_journal_max_file_age_ms: default_usage_journal_max_file_age_ms(),
            usage_journal_max_files: default_usage_journal_max_files(),
            usage_journal_block_target_uncompressed_bytes:
                default_usage_journal_block_target_uncompressed_bytes(),
            usage_journal_block_max_events: default_usage_journal_block_max_events(),
            usage_journal_fsync_interval_ms: default_usage_journal_fsync_interval_ms(),
            usage_journal_zstd_level: default_usage_journal_zstd_level(),
            usage_journal_consumer_lease_ms: default_usage_journal_consumer_lease_ms(),
            usage_journal_delete_bad_files: false,
            usage_query_bind_addr: default_usage_query_bind_addr(),
            usage_query_base_url: default_usage_query_base_url(),
            kiro_cache_kmodels_json: r#"{"claude-haiku-4-5-20251001":2.3681034438052206e-06,"claude-opus-4-6":8.061927916785985e-06,"claude-sonnet-4-6":5.055065250835128e-06}"#.to_string(),
            kiro_billable_model_multipliers_json: default_kiro_billable_model_multipliers_json(),
            kiro_cache_policy_json: r#"{"small_input_high_credit_boost":{"target_input_tokens":100000,"credit_start":1.0,"credit_end":1.8},"prefix_tree_credit_ratio_bands":[{"credit_start":0.3,"credit_end":1.0,"cache_ratio_start":0.7,"cache_ratio_end":0.2},{"credit_start":1.0,"credit_end":2.5,"cache_ratio_start":0.2,"cache_ratio_end":0.0}],"high_credit_diagnostic_threshold":2.0}"#.to_string(),
            kiro_context_usage_min_request_tokens: default_kiro_context_usage_min_request_tokens(),
            kiro_compact_trigger_tokens: default_kiro_compact_trigger_tokens(),
            kiro_prefix_cache_mode: "prefix_tree".to_string(),
            kiro_prefix_cache_max_tokens: 4_000_000,
            kiro_prefix_cache_entry_ttl_seconds: 21_600,
            kiro_conversation_anchor_max_entries: 20_000,
            kiro_conversation_anchor_ttl_seconds: 86_400,
            kiro_cache_snapshot_enabled: default_kiro_cache_snapshot_enabled(),
            kiro_cache_snapshot_interval_seconds: default_kiro_cache_snapshot_interval_seconds(),
            kiro_cache_snapshot_ttl_seconds: default_kiro_cache_snapshot_ttl_seconds(),
            kiro_cache_snapshot_max_tokens: 0,
            kiro_cache_snapshot_max_anchor_entries: 0,
            kiro_cctest_proxy_base_url: None,
            kiro_cctest_proxy_api_key: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/config", llm_access_admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Persist a new admin-selected auth cache TTL for gateway key validation.
pub async fn update_admin_llm_gateway_config(
    config: &LlmGatewayRuntimeConfig,
) -> Result<LlmGatewayRuntimeConfig, String> {
    #[cfg(feature = "mock")]
    {
        Ok(config.clone())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/config", llm_access_admin_base());
        let response = api_post(&url)
            .json(config)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_usage_journal_status() -> Result<AdminUsageJournalStatusView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminUsageJournalStatusView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-access/usage-journal/status", llm_access_admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        let content_type = response.headers().get("content-type").unwrap_or_default();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Read error: {:?}", e))?;
        if !content_type.contains("json") {
            let preview = body.chars().take(120).collect::<String>();
            return Err(format!(
                "Unexpected response content-type `{content_type}` while loading Usage Journal \
                 Worker: {preview}"
            ));
        }
        serde_json::from_str(&body).map_err(|e| {
            format!("Parse error: {:?}; body: {}", e, body.chars().take(120).collect::<String>())
        })
    }
}

pub async fn fetch_admin_usage_journal_preview(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<AdminUsageJournalPreviewResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminUsageJournalPreviewResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!("{}/admin/llm-access/usage-journal/preview", llm_access_admin_base());
        let mut query = Vec::new();
        if let Some(limit) = limit {
            query.push(format!("limit={limit}"));
        }
        if let Some(offset) = offset {
            query.push(format!("offset={offset}"));
        }
        if !query.is_empty() {
            url.push('?');
            url.push_str(&query.join("&"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        let content_type = response.headers().get("content-type").unwrap_or_default();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Read error: {:?}", e))?;
        if !content_type.contains("json") {
            let preview = body.chars().take(120).collect::<String>();
            return Err(format!(
                "Unexpected response content-type `{content_type}` while loading Usage Journal \
                 Preview: {preview}"
            ));
        }
        serde_json::from_str(&body).map_err(|e| {
            format!("Parse error: {:?}; body: {}", e, body.chars().take(120).collect::<String>())
        })
    }
}

pub async fn fetch_admin_llm_gateway_proxy_configs(
) -> Result<AdminUpstreamProxyConfigsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminUpstreamProxyConfigsResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/proxy-configs", llm_access_admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn create_admin_llm_gateway_proxy_config(
    input: &CreateAdminUpstreamProxyConfigInput,
) -> Result<AdminUpstreamProxyConfigView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminUpstreamProxyConfigView {
            id: "mock-proxy".to_string(),
            name: input.name.clone(),
            proxy_url: input.proxy_url.clone(),
            proxy_username: input.proxy_username.clone(),
            proxy_password: input.proxy_password.clone(),
            status: "active".to_string(),
            created_at: 0,
            updated_at: 0,
            scope_node_id: Some("core".to_string()),
            effective_source: "core".to_string(),
            has_node_override: false,
            can_edit_slot_metadata: true,
            latest_codex_check: None,
            latest_kiro_check: None,
            traffic_snapshot: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/proxy-configs", llm_access_admin_base());
        let response = api_post(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn patch_admin_llm_gateway_proxy_config(
    proxy_id: &str,
    input: &PatchAdminUpstreamProxyConfigInput,
) -> Result<AdminUpstreamProxyConfigView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = proxy_id;
        Ok(AdminUpstreamProxyConfigView {
            id: "mock-proxy".to_string(),
            name: input.name.clone().unwrap_or_else(|| "mock".to_string()),
            proxy_url: input
                .proxy_url
                .clone()
                .unwrap_or_else(|| "http://127.0.0.1:11111".to_string()),
            proxy_username: input.proxy_username.clone(),
            proxy_password: input.proxy_password.clone(),
            status: input.status.clone().unwrap_or_else(|| "active".to_string()),
            created_at: 0,
            updated_at: 0,
            scope_node_id: Some("core".to_string()),
            effective_source: "core".to_string(),
            has_node_override: false,
            can_edit_slot_metadata: true,
            latest_codex_check: None,
            latest_kiro_check: None,
            traffic_snapshot: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/proxy-configs/{}",
            llm_access_admin_base(),
            urlencoding::encode(proxy_id)
        );
        let response = api_patch(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn delete_admin_llm_gateway_proxy_config(proxy_id: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = proxy_id;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/proxy-configs/{}",
            llm_access_admin_base(),
            urlencoding::encode(proxy_id)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

pub async fn refresh_admin_llm_gateway_proxy_traffic(
    proxy_id: &str,
) -> Result<AdminProxyTrafficRefreshResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminProxyTrafficRefreshResponse {
            proxy_config_id: proxy_id.to_string(),
            traffic_snapshot: AdminProxyTrafficSnapshotView {
                retention_days: 7,
                totals: AdminLlmGatewayProxyTrafficTotalsView {
                    event_count: 12,
                    request_bytes: 1024 * 1024,
                    response_bytes: 3 * 1024 * 1024,
                    total_bytes: 4 * 1024 * 1024,
                },
                ..AdminProxyTrafficSnapshotView::default()
            },
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/proxy-configs/{}/traffic-refresh",
            llm_access_admin_base(),
            urlencoding::encode(proxy_id)
        );
        let response = api_post(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn reset_admin_llm_gateway_proxy_config_override(
    proxy_id: &str,
) -> Result<AdminUpstreamProxyConfigView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminUpstreamProxyConfigView {
            id: proxy_id.to_string(),
            name: "mock-proxy".to_string(),
            proxy_url: "http://127.0.0.1:11111".to_string(),
            status: "active".to_string(),
            scope_node_id: Some("edge-a".to_string()),
            effective_source: "core".to_string(),
            has_node_override: false,
            can_edit_slot_metadata: false,
            ..AdminUpstreamProxyConfigView::default()
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/proxy-configs/{}/override",
            llm_access_admin_base(),
            urlencoding::encode(proxy_id)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn check_admin_llm_gateway_proxy_config(
    proxy_id: &str,
    provider_type: &str,
) -> Result<AdminUpstreamProxyCheckResponse, String> {
    check_admin_llm_gateway_proxy_config_with_mode(proxy_id, provider_type, None).await
}

pub async fn check_admin_llm_gateway_proxy_config_full_chain(
    proxy_id: &str,
    provider_type: &str,
) -> Result<AdminUpstreamProxyCheckResponse, String> {
    check_admin_llm_gateway_proxy_config_with_mode(proxy_id, provider_type, Some("full_chain"))
        .await
}

async fn check_admin_llm_gateway_proxy_config_with_mode(
    proxy_id: &str,
    provider_type: &str,
    mode: Option<&str>,
) -> Result<AdminUpstreamProxyCheckResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminUpstreamProxyCheckResponse {
            proxy_config_id: proxy_id.to_string(),
            proxy_config_name: "mock-proxy".to_string(),
            provider_type: provider_type.to_string(),
            auth_label: mode
                .map(|mode| format!("{provider_type} {mode} probe `mock`"))
                .unwrap_or_else(|| format!("{provider_type} auth `mock`")),
            ok: true,
            targets: vec![AdminUpstreamProxyCheckTargetView {
                target: provider_type.to_string(),
                url: if mode == Some("full_chain") && provider_type == "kiro" {
                    "/api/kiro-gateway/v1/messages".to_string()
                } else if mode == Some("full_chain") {
                    "/api/codex-gateway/v1/responses".to_string()
                } else if provider_type == "kiro" {
                    "https://management.us-east-1.kiro.dev/getUsageLimits".to_string()
                } else {
                    "https://chatgpt.com/backend-api/codex/v1/models".to_string()
                },
                reachable: true,
                status_code: Some(200),
                latency_ms: if mode == Some("full_chain") { 842 } else { 120 },
                error_message: None,
            }],
            checked_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/proxy-configs/{}/check/{}",
            llm_access_admin_base(),
            urlencoding::encode(proxy_id),
            urlencoding::encode(provider_type)
        );
        let response = if let Some(mode) = mode {
            api_post(&url)
                .json(&serde_json::json!({ "mode": mode }))
                .map_err(|e| format!("Serialize error: {:?}", e))?
                .send()
                .await
        } else {
            api_post(&url).send().await
        }
        .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_llm_gateway_proxy_bindings(
) -> Result<AdminUpstreamProxyBindingsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminUpstreamProxyBindingsResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/proxy-bindings", llm_access_admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn update_admin_llm_gateway_proxy_binding(
    provider_type: &str,
    proxy_config_id: Option<&str>,
) -> Result<AdminUpstreamProxyBindingView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminUpstreamProxyBindingView {
            provider_type: provider_type.to_string(),
            effective_source: if proxy_config_id.is_some() {
                "binding".to_string()
            } else {
                "env_fallback".to_string()
            },
            bound_proxy_config_id: proxy_config_id.map(ToString::to_string),
            effective_proxy_config_name: Some("mock".to_string()),
            effective_proxy_url: Some("http://127.0.0.1:11111".to_string()),
            effective_proxy_username: None,
            effective_proxy_password: None,
            binding_updated_at: Some(0),
            error_message: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/proxy-bindings/{}",
            llm_access_admin_base(),
            urlencoding::encode(provider_type)
        );
        let response = api_post(&url)
            .json(&serde_json::json!({ "proxy_config_id": proxy_config_id }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn import_admin_legacy_kiro_proxy_configs(
) -> Result<AdminLegacyKiroProxyMigrationResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminLegacyKiroProxyMigrationResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/proxy-configs/import-legacy-kiro",
            llm_access_admin_base()
        );
        let response = api_post(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Fetch all admin key pages, including secrets and current counters.
pub async fn fetch_admin_llm_gateway_keys_page(
    limit: usize,
    offset: usize,
) -> Result<AdminLlmGatewayKeysResponse, String> {
    fetch_admin_llm_gateway_keys_page_with_query(
        limit,
        offset,
        &AdminLlmGatewayKeyPageQuery::default(),
    )
    .await
}

pub async fn fetch_admin_llm_gateway_keys_page_with_query(
    limit: usize,
    offset: usize,
    query: &AdminLlmGatewayKeyPageQuery,
) -> Result<AdminLlmGatewayKeysResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminLlmGatewayKeysResponse {
            keys: vec![],
            summary: AdminLlmGatewayKeysSummaryView::default(),
            auth_cache_ttl_seconds: 60,
            total: 0,
            limit,
            offset,
            has_more: false,
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut params = vec![format!("limit={limit}"), format!("offset={offset}")];
        if let Some(q) = query
            .q
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if query.active_only {
            params.push("active_only=true".to_string());
        }
        if let Some(sort) = query
            .sort
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("sort={}", urlencoding::encode(sort)));
        }
        let url =
            format!("{}/admin/llm-gateway/keys?{}", llm_access_admin_base(), params.join("&"));
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

#[derive(Debug, Clone, Default)]
pub struct CreateAdminAccountGroupInput<'a> {
    pub name: &'a str,
    pub account_names: &'a [String],
}

#[derive(Debug, Clone, Default)]
pub struct PatchAdminAccountGroupInput<'a> {
    pub name: Option<&'a str>,
    pub account_names: Option<&'a [String]>,
}

pub async fn fetch_admin_llm_gateway_account_groups_page(
    limit: usize,
    offset: usize,
) -> Result<AdminAccountGroupsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminAccountGroupsResponse {
            groups: vec![],
            total: 0,
            limit,
            offset,
            has_more: false,
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/account-groups?limit={limit}&offset={offset}",
            llm_access_admin_base()
        );
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_llm_gateway_account_group_options(
) -> Result<Vec<AdminAccountGroupOptionView>, String> {
    #[cfg(feature = "mock")]
    {
        Ok(Vec::new())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/account-group-options", llm_access_admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json::<AdminAccountGroupOptionsResponse>()
            .await
            .map(|resp| resp.options)
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn create_admin_llm_gateway_account_group(
    input: CreateAdminAccountGroupInput<'_>,
) -> Result<AdminAccountGroupView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminAccountGroupView {
            id: "mock-group".to_string(),
            provider_type: "codex".to_string(),
            name: input.name.to_string(),
            account_names: input.account_names.to_vec(),
            created_at: 0,
            updated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/account-groups", llm_access_admin_base());
        let response = api_post(&url)
            .json(&serde_json::json!({
                "name": input.name,
                "account_names": input.account_names,
            }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn patch_admin_llm_gateway_account_group(
    group_id: &str,
    input: PatchAdminAccountGroupInput<'_>,
) -> Result<AdminAccountGroupView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminAccountGroupView {
            id: group_id.to_string(),
            provider_type: "codex".to_string(),
            name: input.name.unwrap_or("mock").to_string(),
            account_names: input
                .account_names
                .map(|value| value.to_vec())
                .unwrap_or_default(),
            created_at: 0,
            updated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/account-groups/{}",
            llm_access_admin_base(),
            urlencoding::encode(group_id)
        );
        let mut body = serde_json::Map::new();
        if let Some(name) = input.name.map(str::trim).filter(|value| !value.is_empty()) {
            body.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        }
        if let Some(account_names) = input.account_names {
            body.insert(
                "account_names".to_string(),
                serde_json::Value::Array(
                    account_names
                        .iter()
                        .map(|value| serde_json::Value::String(value.clone()))
                        .collect(),
                ),
            );
        }
        let response = api_patch(&url)
            .json(&serde_json::Value::Object(body))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn delete_admin_llm_gateway_account_group(group_id: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = group_id;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/account-groups/{}",
            llm_access_admin_base(),
            urlencoding::encode(group_id)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

/// Fetch admin token wishes for review / issuance.
pub async fn fetch_admin_llm_gateway_token_requests(
    query: &AdminLlmGatewayTokenRequestsQuery,
) -> Result<AdminLlmGatewayTokenRequestsResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = query;
        Ok(AdminLlmGatewayTokenRequestsResponse {
            total: 0,
            offset: 0,
            limit: 20,
            has_more: false,
            requests: vec![],
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!("{}/admin/llm-gateway/token-requests", llm_access_admin_base());
        let mut params = Vec::new();
        if let Some(status) = query.status.as_deref() {
            params.push(format!("status={}", urlencoding::encode(status)));
        }
        if let Some(limit) = query.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(offset) = query.offset {
            params.push(format!("offset={offset}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Fetch admin account contribution requests for review / issuance.
pub async fn fetch_admin_llm_gateway_account_contribution_requests(
    query: &AdminLlmGatewayAccountContributionRequestsQuery,
) -> Result<AdminLlmGatewayAccountContributionRequestsResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = query;
        Ok(AdminLlmGatewayAccountContributionRequestsResponse {
            total: 0,
            offset: 0,
            limit: 20,
            has_more: false,
            requests: vec![],
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url =
            format!("{}/admin/llm-gateway/account-contribution-requests", llm_access_admin_base());
        let mut params = Vec::new();
        if let Some(status) = query.status.as_deref() {
            params.push(format!("status={}", urlencoding::encode(status)));
        }
        if let Some(limit) = query.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(offset) = query.offset {
            params.push(format!("offset={offset}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Fetch admin sponsor requests for manual review.
pub async fn fetch_admin_llm_gateway_sponsor_requests(
    query: &AdminLlmGatewaySponsorRequestsQuery,
) -> Result<AdminLlmGatewaySponsorRequestsResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = query;
        Ok(AdminLlmGatewaySponsorRequestsResponse {
            total: 0,
            offset: 0,
            limit: 20,
            has_more: false,
            requests: vec![],
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!("{}/admin/llm-gateway/sponsor-requests", llm_access_admin_base());
        let mut params = Vec::new();
        if let Some(status) = query.status.as_deref() {
            params.push(format!("status={}", urlencoding::encode(status)));
        }
        if let Some(limit) = query.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(offset) = query.offset {
            params.push(format!("offset={offset}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Create a new gateway key that can later be exposed on the public page.
pub async fn create_admin_llm_gateway_key(
    name: &str,
    quota_billable_limit: u64,
    public_visible: bool,
    request_max_concurrency: Option<u64>,
    request_min_start_interval_ms: Option<u64>,
) -> Result<AdminLlmGatewayKeyView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminLlmGatewayKeyView {
            id: "mock".to_string(),
            name: name.to_string(),
            secret: "sfk_mock".to_string(),
            key_hash: "hash".to_string(),
            status: "active".to_string(),
            provider_type: "codex".to_string(),
            public_visible,
            quota_billable_limit,
            usage_input_uncached_tokens: 0,
            usage_input_cached_tokens: 0,
            usage_output_tokens: 0,
            usage_credit_total: 0.0,
            usage_credit_missing_events: 0,
            codex_image_usage_tokens: 0,
            codex_image_usage_missing_events: 0,
            codex_image_last_used_at: None,
            remaining_billable: quota_billable_limit as i64,
            last_used_at: None,
            created_at: 0,
            updated_at: 0,
            route_strategy: None,
            account_group_id: None,
            fixed_account_name: None,
            auto_account_names: None,
            preferred_pool_strategy: default_kiro_pool_strategy(),
            kiro_anthropic_upstream_pool_mode: default_anthropic_upstream_pool_mode(),
            model_name_map: None,
            kiro_model_group_preferences: BTreeMap::new(),
            kiro_model_channel_preferences: BTreeMap::new(),
            request_max_concurrency,
            request_min_start_interval_ms,
            moderation_enabled: true,
            kiro_request_validation_enabled: true,
            kiro_cache_estimation_enabled: true,
            kiro_zero_cache_debug_enabled: false,
            kiro_full_request_logging_enabled: false,
            kiro_remote_media_resolution_enabled: false,
            kiro_latency_routing_enabled: true,
            kiro_protected_content_validation_enabled: false,
            kiro_cctest_text_handling_enabled: false,
            kiro_cache_policy_override_json: None,
            kiro_billable_model_multipliers_override_json: None,
            effective_kiro_cache_policy_json: String::new(),
            uses_global_kiro_cache_policy: true,
            effective_kiro_billable_model_multipliers_json:
                default_kiro_billable_model_multipliers_json(),
            uses_global_kiro_billable_model_multipliers: true,
            codex_fast_enabled: true,
            codex_responses_lite_enabled: true,
            codex_strict_session_rejection_enabled: false,
            codex_image_generation_enabled: true,
            codex_image_standalone_generation_enabled: true,
            codex_image_direct_generation_enabled: false,
            kiro_candidate_credit_summary: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/keys", llm_access_admin_base());
        let response = api_post(&url)
            .json(&serde_json::json!({
                "name": name,
                "quota_billable_limit": quota_billable_limit,
                "public_visible": public_visible,
                "request_max_concurrency": request_max_concurrency,
                "request_min_start_interval_ms": request_min_start_interval_ms
            }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Patch editable fields on a gateway key from the admin UI.
#[derive(Clone, Debug, Default)]
pub struct PatchAdminLlmGatewayKeyRequest<'a> {
    pub name: Option<&'a str>,
    pub status: Option<&'a str>,
    pub public_visible: Option<bool>,
    pub quota_billable_limit: Option<u64>,
    pub route_strategy: Option<&'a str>,
    pub account_group_id: Option<&'a str>,
    pub fixed_account_name: Option<&'a str>,
    pub auto_account_names: Option<&'a [String]>,
    pub preferred_pool_strategy: Option<&'a str>,
    pub kiro_anthropic_upstream_pool_mode: Option<&'a str>,
    pub model_name_map: Option<&'a BTreeMap<String, String>>,
    pub kiro_model_group_preferences: Option<&'a BTreeMap<String, String>>,
    pub kiro_model_channel_preferences: Option<&'a BTreeMap<String, String>>,
    pub request_max_concurrency: Option<u64>,
    pub request_min_start_interval_ms: Option<u64>,
    pub moderation_enabled: Option<bool>,
    pub codex_fast_enabled: Option<bool>,
    pub codex_responses_lite_enabled: Option<bool>,
    pub codex_strict_session_rejection_enabled: Option<bool>,
    pub codex_image_generation_enabled: Option<bool>,
    pub codex_image_standalone_generation_enabled: Option<bool>,
    pub codex_image_direct_generation_enabled: Option<bool>,
    pub kiro_request_validation_enabled: Option<bool>,
    pub kiro_cache_estimation_enabled: Option<bool>,
    pub kiro_zero_cache_debug_enabled: Option<bool>,
    pub kiro_full_request_logging_enabled: Option<bool>,
    pub kiro_remote_media_resolution_enabled: Option<bool>,
    pub kiro_latency_routing_enabled: Option<bool>,
    pub kiro_protected_content_validation_enabled: Option<bool>,
    pub kiro_cctest_text_handling_enabled: Option<bool>,
    pub kiro_cache_policy_override_json: Option<Option<&'a str>>,
    pub kiro_billable_model_multipliers_override_json: Option<Option<&'a str>>,
    pub request_max_concurrency_unlimited: bool,
    pub request_min_start_interval_ms_unlimited: bool,
}

pub async fn patch_admin_llm_gateway_key(
    key_id: &str,
    request: PatchAdminLlmGatewayKeyRequest<'_>,
) -> Result<AdminLlmGatewayKeyView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (
            key_id,
            request.name,
            request.status,
            request.public_visible,
            request.quota_billable_limit,
            request.route_strategy,
            request.account_group_id,
            request.fixed_account_name,
            request.auto_account_names,
            request.preferred_pool_strategy,
            request.kiro_anthropic_upstream_pool_mode,
            request.model_name_map,
            request.kiro_model_group_preferences,
            request.kiro_model_channel_preferences,
            request.request_max_concurrency,
            request.request_min_start_interval_ms,
            request.moderation_enabled,
            request.codex_fast_enabled,
            request.codex_responses_lite_enabled,
            request.codex_strict_session_rejection_enabled,
            request.codex_image_generation_enabled,
            request.codex_image_standalone_generation_enabled,
            request.codex_image_direct_generation_enabled,
            request.kiro_request_validation_enabled,
            request.kiro_cache_estimation_enabled,
            request.kiro_zero_cache_debug_enabled,
            request.kiro_full_request_logging_enabled,
            request.kiro_remote_media_resolution_enabled,
            request.kiro_latency_routing_enabled,
            request.kiro_protected_content_validation_enabled,
            request.kiro_cctest_text_handling_enabled,
            request.kiro_cache_policy_override_json,
            request.kiro_billable_model_multipliers_override_json,
            request.request_max_concurrency_unlimited,
            request.request_min_start_interval_ms_unlimited,
        );
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/keys/{}",
            llm_access_admin_base(),
            urlencoding::encode(key_id)
        );
        let mut body = serde_json::Map::new();
        if let Some(name) = request
            .name
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            body.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        }
        if let Some(status) = request
            .status
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            body.insert("status".to_string(), serde_json::Value::String(status.to_string()));
        }
        if let Some(public_visible) = request.public_visible {
            body.insert("public_visible".to_string(), serde_json::Value::Bool(public_visible));
        }
        if let Some(quota_billable_limit) = request.quota_billable_limit {
            body.insert(
                "quota_billable_limit".to_string(),
                serde_json::Value::Number(quota_billable_limit.into()),
            );
        }
        if let Some(strategy) = request.route_strategy {
            body.insert(
                "route_strategy".to_string(),
                serde_json::Value::String(strategy.to_string()),
            );
        }
        if let Some(group_id) = request.account_group_id {
            body.insert(
                "account_group_id".to_string(),
                serde_json::Value::String(group_id.to_string()),
            );
        }
        if let Some(account_name) = request.fixed_account_name {
            body.insert(
                "fixed_account_name".to_string(),
                serde_json::Value::String(account_name.to_string()),
            );
        }
        if let Some(account_names) = request.auto_account_names {
            body.insert(
                "auto_account_names".to_string(),
                serde_json::Value::Array(
                    account_names
                        .iter()
                        .map(|value| serde_json::Value::String(value.clone()))
                        .collect(),
                ),
            );
        }
        if let Some(preferred_pool_strategy) = request.preferred_pool_strategy {
            body.insert(
                "preferred_pool_strategy".to_string(),
                serde_json::Value::String(preferred_pool_strategy.to_string()),
            );
        }
        if let Some(mode) = request.kiro_anthropic_upstream_pool_mode {
            body.insert(
                "kiro_anthropic_upstream_pool_mode".to_string(),
                serde_json::Value::String(mode.to_string()),
            );
        }
        if let Some(model_name_map) = request.model_name_map {
            let value = serde_json::to_value(model_name_map)
                .map_err(|e| format!("Serialize error: {:?}", e))?;
            body.insert("model_name_map".to_string(), value);
        }
        if let Some(preferences) = request.kiro_model_group_preferences {
            let value = serde_json::to_value(preferences)
                .map_err(|e| format!("Serialize error: {:?}", e))?;
            body.insert("kiro_model_group_preferences".to_string(), value);
        }
        if let Some(preferences) = request.kiro_model_channel_preferences {
            let value = serde_json::to_value(preferences)
                .map_err(|e| format!("Serialize error: {:?}", e))?;
            body.insert("kiro_model_channel_preferences".to_string(), value);
        }
        if let Some(request_max_concurrency) = request.request_max_concurrency {
            body.insert(
                "request_max_concurrency".to_string(),
                serde_json::Value::Number(request_max_concurrency.into()),
            );
        }
        if let Some(request_min_start_interval_ms) = request.request_min_start_interval_ms {
            body.insert(
                "request_min_start_interval_ms".to_string(),
                serde_json::Value::Number(request_min_start_interval_ms.into()),
            );
        }
        if let Some(moderation_enabled) = request.moderation_enabled {
            body.insert(
                "moderation_enabled".to_string(),
                serde_json::Value::Bool(moderation_enabled),
            );
        }
        if let Some(codex_fast_enabled) = request.codex_fast_enabled {
            body.insert(
                "codex_fast_enabled".to_string(),
                serde_json::Value::Bool(codex_fast_enabled),
            );
        }
        if let Some(enabled) = request.codex_responses_lite_enabled {
            body.insert(
                "codex_responses_lite_enabled".to_string(),
                serde_json::Value::Bool(enabled),
            );
        }
        if let Some(enabled) = request.codex_strict_session_rejection_enabled {
            body.insert(
                "codex_strict_session_rejection_enabled".to_string(),
                serde_json::Value::Bool(enabled),
            );
        }
        if let Some(enabled) = request.codex_image_generation_enabled {
            body.insert(
                "codex_image_generation_enabled".to_string(),
                serde_json::Value::Bool(enabled),
            );
        }
        if let Some(enabled) = request.codex_image_standalone_generation_enabled {
            body.insert(
                "codex_image_standalone_generation_enabled".to_string(),
                serde_json::Value::Bool(enabled),
            );
        }
        if let Some(enabled) = request.codex_image_direct_generation_enabled {
            body.insert(
                "codex_image_direct_generation_enabled".to_string(),
                serde_json::Value::Bool(enabled),
            );
        }
        if let Some(kiro_request_validation_enabled) = request.kiro_request_validation_enabled {
            body.insert(
                "kiro_request_validation_enabled".to_string(),
                serde_json::Value::Bool(kiro_request_validation_enabled),
            );
        }
        if let Some(kiro_cache_estimation_enabled) = request.kiro_cache_estimation_enabled {
            body.insert(
                "kiro_cache_estimation_enabled".to_string(),
                serde_json::Value::Bool(kiro_cache_estimation_enabled),
            );
        }
        if let Some(kiro_zero_cache_debug_enabled) = request.kiro_zero_cache_debug_enabled {
            body.insert(
                "kiro_zero_cache_debug_enabled".to_string(),
                serde_json::Value::Bool(kiro_zero_cache_debug_enabled),
            );
        }
        if let Some(kiro_full_request_logging_enabled) = request.kiro_full_request_logging_enabled {
            body.insert(
                "kiro_full_request_logging_enabled".to_string(),
                serde_json::Value::Bool(kiro_full_request_logging_enabled),
            );
        }
        if let Some(kiro_remote_media_resolution_enabled) =
            request.kiro_remote_media_resolution_enabled
        {
            body.insert(
                "kiro_remote_media_resolution_enabled".to_string(),
                serde_json::Value::Bool(kiro_remote_media_resolution_enabled),
            );
        }
        if let Some(kiro_latency_routing_enabled) = request.kiro_latency_routing_enabled {
            body.insert(
                "kiro_latency_routing_enabled".to_string(),
                serde_json::Value::Bool(kiro_latency_routing_enabled),
            );
        }
        if let Some(kiro_protected_content_validation_enabled) =
            request.kiro_protected_content_validation_enabled
        {
            body.insert(
                "kiro_protected_content_validation_enabled".to_string(),
                serde_json::Value::Bool(kiro_protected_content_validation_enabled),
            );
        }
        if let Some(kiro_cctest_text_handling_enabled) = request.kiro_cctest_text_handling_enabled {
            body.insert(
                "kiro_cctest_text_handling_enabled".to_string(),
                serde_json::Value::Bool(kiro_cctest_text_handling_enabled),
            );
        }
        if let Some(kiro_cache_policy_override_json) = request.kiro_cache_policy_override_json {
            body.insert(
                "kiro_cache_policy_override_json".to_string(),
                kiro_cache_policy_override_json
                    .map(|raw| serde_json::Value::String(raw.to_string()))
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        if let Some(kiro_billable_model_multipliers_override_json) =
            request.kiro_billable_model_multipliers_override_json
        {
            body.insert(
                "kiro_billable_model_multipliers_override_json".to_string(),
                kiro_billable_model_multipliers_override_json
                    .map(|raw| serde_json::Value::String(raw.to_string()))
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        if request.request_max_concurrency_unlimited {
            body.insert(
                "request_max_concurrency_unlimited".to_string(),
                serde_json::Value::Bool(true),
            );
        }
        if request.request_min_start_interval_ms_unlimited {
            body.insert(
                "request_min_start_interval_ms_unlimited".to_string(),
                serde_json::Value::Bool(true),
            );
        }
        let response = api_patch(&url)
            .json(&serde_json::Value::Object(body))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Approve a token wish, issue the key, and email it to the requester.
pub async fn admin_approve_and_issue_llm_gateway_token_request(
    request_id: &str,
    admin_note: Option<&str>,
) -> Result<AdminLlmGatewayTokenRequestView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (request_id, admin_note);
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/token-requests/{}/approve-and-issue",
            llm_access_admin_base(),
            urlencoding::encode(request_id)
        );
        let response = api_post(&url)
            .json(&serde_json::json!({ "admin_note": admin_note }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Reject a token wish from the admin UI.
pub async fn admin_reject_llm_gateway_token_request(
    request_id: &str,
    admin_note: Option<&str>,
) -> Result<AdminLlmGatewayTokenRequestView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (request_id, admin_note);
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/token-requests/{}/reject",
            llm_access_admin_base(),
            urlencoding::encode(request_id)
        );
        let response = api_post(&url)
            .json(&serde_json::json!({ "admin_note": admin_note }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Approve an account contribution, import the account, issue a bound key,
/// and email it to the contributor.
pub async fn admin_approve_and_issue_llm_gateway_account_contribution_request(
    request_id: &str,
    admin_note: Option<&str>,
) -> Result<AdminLlmGatewayAccountContributionRequestView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (request_id, admin_note);
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/account-contribution-requests/{}/approve-and-issue",
            llm_access_admin_base(),
            urlencoding::encode(request_id)
        );
        let response = api_post(&url)
            .json(&serde_json::json!({ "admin_note": admin_note }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Validate a Codex account contribution by refreshing its auth before import.
pub async fn admin_validate_llm_gateway_account_contribution_request(
    request_id: &str,
    admin_note: Option<&str>,
) -> Result<AdminLlmGatewayAccountContributionRequestView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (request_id, admin_note);
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/account-contribution-requests/{}/validate",
            llm_access_admin_base(),
            urlencoding::encode(request_id)
        );
        let response = api_post(&url)
            .json(&serde_json::json!({ "admin_note": admin_note }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Reject an account contribution request from the admin UI.
pub async fn admin_reject_llm_gateway_account_contribution_request(
    request_id: &str,
    admin_note: Option<&str>,
) -> Result<AdminLlmGatewayAccountContributionRequestView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (request_id, admin_note);
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/account-contribution-requests/{}/reject",
            llm_access_admin_base(),
            urlencoding::encode(request_id)
        );
        let response = api_post(&url)
            .json(&serde_json::json!({ "admin_note": admin_note }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Mark a sponsor request as manually confirmed from the admin UI.
pub async fn admin_approve_llm_gateway_sponsor_request(
    request_id: &str,
    admin_note: Option<&str>,
) -> Result<AdminLlmGatewaySponsorRequestView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (request_id, admin_note);
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/sponsor-requests/{}/approve",
            llm_access_admin_base(),
            urlencoding::encode(request_id)
        );
        let response = api_post(&url)
            .json(&serde_json::json!({ "admin_note": admin_note }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

/// Delete one sponsor request from the admin UI.
pub async fn delete_admin_llm_gateway_sponsor_request(request_id: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = request_id;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/sponsor-requests/{}",
            llm_access_admin_base(),
            urlencoding::encode(request_id)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

/// Delete a gateway key from the admin UI.
pub async fn delete_admin_llm_gateway_key(key_id: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = key_id;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/keys/{}",
            llm_access_admin_base(),
            urlencoding::encode(key_id)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

/// Fetch a paginated slice of admin usage events with an optional key filter.
pub async fn fetch_admin_llm_gateway_usage_events(
    query: &AdminLlmGatewayUsageEventsQuery,
) -> Result<AdminLlmGatewayUsageEventsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminLlmGatewayUsageEventsResponse {
            total: 0,
            offset: query.offset.unwrap_or(0),
            limit: query.limit.unwrap_or(50),
            has_more: false,
            current_rpm: 0,
            current_in_flight: 0,
            retention_days: default_usage_analytics_retention_days(),
            totals: AdminUsageTotalsView::default(),
            events: vec![],
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!("{}/admin/llm-gateway/usage", llm_access_admin_base());
        let params = admin_usage_query_params(query);
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

fn admin_usage_query_params(query: &AdminLlmGatewayUsageEventsQuery) -> Vec<String> {
    let mut params = Vec::new();
    if let Some(key_id) = query
        .key_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        params.push(format!("key_id={}", urlencoding::encode(key_id)));
    }
    if let Some(start_ms) = query.start_ms {
        params.push(format!("start_ms={start_ms}"));
    }
    if let Some(end_ms) = query.end_ms {
        params.push(format!("end_ms={end_ms}"));
    }
    if let Some(source) = query
        .source
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        params.push(format!("source={}", urlencoding::encode(source)));
    }
    if let Some(model) = query
        .model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        params.push(format!("model={}", urlencoding::encode(model)));
    }
    if let Some(account_name) = query
        .account_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        params.push(format!("account_name={}", urlencoding::encode(account_name)));
    }
    if let Some(endpoint) = query
        .endpoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        params.push(format!("endpoint={}", urlencoding::encode(endpoint)));
    }
    if let Some(status_code) = query.status_code {
        params.push(format!("status_code={status_code}"));
    }
    if let Some(status_kind) = query
        .status_kind
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        params.push(format!("status_kind={}", urlencoding::encode(status_kind)));
    }
    if let Some(limit) = query.limit {
        params.push(format!("limit={limit}"));
    }
    if let Some(offset) = query.offset {
        params.push(format!("offset={offset}"));
    }
    params
}

pub async fn fetch_admin_llm_gateway_usage_event_detail(
    event_id: &str,
) -> Result<AdminLlmGatewayUsageEventDetailView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = event_id;
        Ok(AdminLlmGatewayUsageEventDetailView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let encoded = urlencoding::encode(event_id);
        let url = format!("{}/admin/llm-gateway/usage/{}", llm_access_admin_base(), encoded);
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_llm_gateway_usage_filter_options(
    query: &AdminLlmGatewayUsageEventsQuery,
) -> Result<AdminLlmGatewayUsageFilterOptionsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminLlmGatewayUsageFilterOptionsResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/usage/filter-options", llm_access_admin_base());
        let params = admin_usage_query_params(query);
        let request = if params.is_empty() {
            api_get(&url)
        } else {
            api_get(&format!("{url}?{}", params.join("&")))
        };
        let response = request
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_llm_gateway_usage_metrics(
    query: &AdminLlmGatewayUsageMetricsQuery,
) -> Result<AdminLlmGatewayUsageMetricsResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = query;
        Ok(AdminLlmGatewayUsageMetricsResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!("{}/admin/llm-gateway/usage/metrics", llm_access_admin_base());
        let mut params = Vec::new();
        if let Some(provider_type) = query
            .provider_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("provider_type={}", urlencoding::encode(provider_type)));
        }
        if let Some(source) = query
            .source
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("source={}", urlencoding::encode(source)));
        }
        if let Some(window) = query
            .window
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("window={}", urlencoding::encode(window)));
        }
        if let Some(top_limit) = query.top_limit {
            params.push(format!("top_limit={top_limit}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_llm_gateway_proxy_traffic(
    query: &AdminLlmGatewayProxyTrafficQuery,
) -> Result<AdminLlmGatewayProxyTrafficResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = query;
        Ok(AdminLlmGatewayProxyTrafficResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!("{}/admin/llm-gateway/usage/proxy-traffic", llm_access_admin_base());
        let mut params = Vec::new();
        if let Some(proxy_config_id) = query
            .proxy_config_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("proxy_config_id={}", urlencoding::encode(proxy_config_id)));
        }
        if let Some(provider_type) = query
            .provider_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("provider_type={}", urlencoding::encode(provider_type)));
        }
        if let Some(source) = query
            .source
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("source={}", urlencoding::encode(source)));
        }
        if let Some(start_ms) = query.start_ms {
            params.push(format!("start_ms={start_ms}"));
        }
        if let Some(end_ms) = query.end_ms {
            params.push(format!("end_ms={end_ms}"));
        }
        if let Some(bucket_ms) = query.bucket_ms {
            params.push(format!("bucket_ms={bucket_ms}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

// === Account pool management ===

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct AccountSummaryView {
    pub name: String,
    pub status: String,
    pub account_id: Option<String>,
    pub email: Option<String>,
    pub plan_type: Option<String>,
    pub route_weight_tier: String,
    pub primary_remaining_percent: Option<f64>,
    pub secondary_remaining_percent: Option<f64>,
    pub rate_limit_reset_credits_available: Option<i64>,
    pub map_gpt53_codex_to_spark: bool,
    pub auto_refresh_enabled: bool,
    pub request_max_concurrency: Option<u64>,
    #[serde(default = "default_codex_account_rpm_limit")]
    pub request_rpm_limit: u64,
    pub request_min_start_interval_ms: Option<u64>,
    #[serde(default)]
    pub codex_image_generation_enabled: bool,
    #[serde(default = "default_codex_image_generation_max_concurrency")]
    pub codex_image_generation_max_concurrency: u64,
    pub proxy_mode: String,
    pub proxy_config_id: Option<String>,
    pub effective_proxy_source: String,
    pub effective_proxy_url: Option<String>,
    pub effective_proxy_config_name: Option<String>,
    pub last_refresh: Option<i64>,
    pub access_token_expires_at: Option<i64>,
    pub auth_refresh_error_message: Option<String>,
    pub last_usage_checked_at: Option<i64>,
    pub last_usage_success_at: Option<i64>,
    pub usage_error_message: Option<String>,
}

impl Default for AccountSummaryView {
    fn default() -> Self {
        Self {
            name: String::new(),
            status: String::new(),
            account_id: None,
            email: None,
            plan_type: None,
            route_weight_tier: "auto".to_string(),
            primary_remaining_percent: None,
            secondary_remaining_percent: None,
            rate_limit_reset_credits_available: None,
            map_gpt53_codex_to_spark: false,
            auto_refresh_enabled: true,
            request_max_concurrency: None,
            request_rpm_limit: default_codex_account_rpm_limit(),
            request_min_start_interval_ms: None,
            codex_image_generation_enabled: false,
            codex_image_generation_max_concurrency: default_codex_image_generation_max_concurrency(
            ),
            proxy_mode: "inherit".to_string(),
            proxy_config_id: None,
            effective_proxy_source: "binding".to_string(),
            effective_proxy_url: None,
            effective_proxy_config_name: None,
            last_refresh: None,
            access_token_expires_at: None,
            auth_refresh_error_message: None,
            last_usage_checked_at: None,
            last_usage_success_at: None,
            usage_error_message: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AccountListResponse {
    pub accounts: Vec<AccountSummaryView>,
    pub summary: AdminAccountsSummaryView,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(default)]
pub struct AdminAccountsSummaryView {
    pub total: usize,
    pub active_count: usize,
    pub disabled_count: usize,
    pub unavailable_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct CodexAccountImportJobSummaryView {
    pub job_id: String,
    pub provider_type: String,
    pub source_type: String,
    pub validate_before_import: bool,
    pub status: String,
    pub total_count: usize,
    pub completed_count: usize,
    pub succeeded_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub batch_error_message: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub finished_at_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct CodexAccountImportJobItemView {
    pub item_index: usize,
    pub requested_name: String,
    pub requested_account_id: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub imported_account_name: Option<String>,
    pub final_account_id: Option<String>,
    pub validated_at_ms: Option<i64>,
    pub imported_at_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct CodexAccountImportJobDetailView {
    pub summary: CodexAccountImportJobSummaryView,
    pub items: Vec<CodexAccountImportJobItemView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
struct CodexAccountImportJobsResponse {
    pub jobs: Vec<CodexAccountImportJobSummaryView>,
    pub generated_at: i64,
}

pub async fn fetch_admin_llm_gateway_accounts() -> Result<AccountListResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AccountListResponse {
            accounts: vec![],
            summary: AdminAccountsSummaryView::default(),
            total: 0,
            limit: 0,
            offset: 0,
            has_more: false,
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut offset = 0;
        let mut result =
            fetch_admin_llm_gateway_accounts_page(ADMIN_GATEWAY_INVENTORY_PAGE_LIMIT, offset)
                .await?;
        while result.has_more {
            offset = result.accounts.len();
            let next =
                fetch_admin_llm_gateway_accounts_page(ADMIN_GATEWAY_INVENTORY_PAGE_LIMIT, offset)
                    .await?;
            let returned = next.accounts.len();
            result = merge_admin_codex_account_pages(result, next);
            if returned == 0 {
                break;
            }
        }
        result.has_more = false;
        result.limit = result.accounts.len();
        Ok(result)
    }
}

pub async fn fetch_admin_llm_gateway_accounts_page(
    limit: usize,
    offset: usize,
) -> Result<AccountListResponse, String> {
    fetch_admin_llm_gateway_accounts_page_with_query(
        limit,
        offset,
        &AdminLlmGatewayAccountPageQuery::default(),
    )
    .await
}

pub async fn fetch_admin_llm_gateway_accounts_page_with_query(
    limit: usize,
    offset: usize,
    query: &AdminLlmGatewayAccountPageQuery,
) -> Result<AccountListResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AccountListResponse {
            accounts: vec![],
            summary: AdminAccountsSummaryView::default(),
            total: 0,
            limit,
            offset,
            has_more: false,
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut params = vec![format!("limit={limit}"), format!("offset={offset}")];
        if let Some(q) = query
            .q
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if query.active_only {
            params.push("active_only=true".to_string());
        }
        if query.unhealthy_only {
            params.push("unhealthy_only=true".to_string());
        }
        if let Some(sort) = query
            .sort
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("sort={}", urlencoding::encode(sort)));
        }
        let url =
            format!("{}/admin/llm-gateway/accounts?{}", llm_access_admin_base(), params.join("&"));
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn create_admin_llm_gateway_account_import_job(
    validate_before_import: bool,
    items: &[serde_json::Value],
) -> Result<CodexAccountImportJobDetailView, String> {
    #[cfg(feature = "mock")]
    {
        let now_ms = Date::now() as i64;
        let item_views = items
            .iter()
            .enumerate()
            .map(|(item_index, item)| CodexAccountImportJobItemView {
                item_index,
                requested_name: item
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                requested_account_id: item
                    .get("auth_json")
                    .and_then(|value| value.get("account_id"))
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string),
                status: "imported".to_string(),
                error_message: None,
                imported_account_name: item
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string),
                final_account_id: item
                    .get("auth_json")
                    .and_then(|value| value.get("account_id"))
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string),
                validated_at_ms: validate_before_import.then_some(now_ms),
                imported_at_ms: Some(now_ms),
            })
            .collect::<Vec<_>>();
        Ok(CodexAccountImportJobDetailView {
            summary: CodexAccountImportJobSummaryView {
                job_id: "llm-import-mock".to_string(),
                provider_type: "codex".to_string(),
                source_type: "local_json".to_string(),
                validate_before_import,
                status: "completed".to_string(),
                total_count: item_views.len(),
                completed_count: item_views.len(),
                succeeded_count: item_views.len(),
                skipped_count: 0,
                failed_count: 0,
                batch_error_message: None,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
                finished_at_ms: Some(now_ms),
            },
            items: item_views,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        if items.is_empty() {
            return Err("批量导入内容不能为空".to_string());
        }
        let url = format!("{}/admin/llm-gateway/accounts/import-jobs", llm_access_admin_base());
        let payload = serde_json::json!({
            "provider_type": "codex",
            "source_type": "local_json",
            "validate_before_import": validate_before_import,
            "items": items,
        });
        let response = api_post(&url)
            .json(&payload)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_llm_gateway_account_import_jobs(
    limit: Option<usize>,
) -> Result<Vec<CodexAccountImportJobSummaryView>, String> {
    #[cfg(feature = "mock")]
    {
        let _ = limit;
        Ok(vec![])
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!("{}/admin/llm-gateway/accounts/import-jobs", llm_access_admin_base());
        if let Some(limit) = limit {
            url.push_str(&format!("?limit={limit}"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        let body: CodexAccountImportJobsResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))?;
        Ok(body.jobs)
    }
}

pub async fn fetch_admin_llm_gateway_account_import_job(
    job_id: &str,
) -> Result<CodexAccountImportJobDetailView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = job_id;
        Ok(CodexAccountImportJobDetailView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/accounts/import-jobs/{}",
            llm_access_admin_base(),
            urlencoding::encode(job_id)
        );
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn import_admin_llm_gateway_account(
    name: &str,
    id_token: &str,
    access_token: &str,
    refresh_token: &str,
    account_id: Option<&str>,
    auth_json: Option<&str>,
) -> Result<AccountSummaryView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (id_token, access_token, refresh_token, auth_json);
        Ok(AccountSummaryView {
            name: name.to_string(),
            status: "active".to_string(),
            account_id: account_id.map(str::to_string),
            email: None,
            plan_type: Some("Pro".to_string()),
            route_weight_tier: "auto".to_string(),
            primary_remaining_percent: Some(100.0),
            secondary_remaining_percent: Some(100.0),
            rate_limit_reset_credits_available: Some(1),
            map_gpt53_codex_to_spark: false,
            auto_refresh_enabled: true,
            request_max_concurrency: None,
            request_rpm_limit: default_codex_account_rpm_limit(),
            request_min_start_interval_ms: None,
            codex_image_generation_enabled: false,
            codex_image_generation_max_concurrency: default_codex_image_generation_max_concurrency(
            ),
            proxy_mode: "inherit".to_string(),
            proxy_config_id: None,
            effective_proxy_source: "binding".to_string(),
            effective_proxy_url: Some("http://127.0.0.1:11111".to_string()),
            effective_proxy_config_name: None,
            last_refresh: None,
            access_token_expires_at: None,
            auth_refresh_error_message: None,
            last_usage_checked_at: None,
            last_usage_success_at: None,
            usage_error_message: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/accounts", llm_access_admin_base());
        let mut payload = serde_json::json!({ "name": name });
        if let Some(raw_auth_json) = auth_json.map(str::trim).filter(|value| !value.is_empty()) {
            payload["auth_json"] = serde_json::from_str(raw_auth_json)
                .map_err(|_| "auth.json 不是合法 JSON".to_string())?;
        } else {
            let mut tokens = serde_json::Map::new();
            if !id_token.trim().is_empty() {
                tokens.insert(
                    "id_token".to_string(),
                    serde_json::Value::String(id_token.trim().to_string()),
                );
            }
            if !access_token.trim().is_empty() {
                tokens.insert(
                    "access_token".to_string(),
                    serde_json::Value::String(access_token.trim().to_string()),
                );
            }
            if !refresh_token.trim().is_empty() {
                tokens.insert(
                    "refresh_token".to_string(),
                    serde_json::Value::String(refresh_token.trim().to_string()),
                );
            }
            if let Some(aid) = account_id.map(str::trim).filter(|value| !value.is_empty()) {
                tokens.insert("account_id".to_string(), serde_json::Value::String(aid.to_string()));
            }
            payload["tokens"] = serde_json::Value::Object(tokens);
        }
        let response = api_post(&url)
            .json(&payload)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn delete_admin_llm_gateway_account(name: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = name;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/accounts/{}",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct PatchAdminLlmGatewayAccountInput {
    pub status: Option<String>,
    pub map_gpt53_codex_to_spark: Option<bool>,
    pub auto_refresh_enabled: Option<bool>,
    pub route_weight_tier: Option<String>,
    pub proxy_mode: Option<String>,
    pub proxy_config_id: Option<String>,
    pub request_max_concurrency: Option<u64>,
    pub request_rpm_limit: Option<u64>,
    pub request_min_start_interval_ms: Option<u64>,
    pub codex_image_generation_enabled: Option<bool>,
    pub codex_image_generation_max_concurrency: Option<u64>,
    pub request_max_concurrency_unlimited: bool,
    pub request_min_start_interval_ms_unlimited: bool,
}

pub async fn patch_admin_llm_gateway_account(
    name: &str,
    input: &PatchAdminLlmGatewayAccountInput,
) -> Result<AccountSummaryView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AccountSummaryView {
            name: name.to_string(),
            status: input.status.clone().unwrap_or_else(|| "active".to_string()),
            account_id: None,
            email: None,
            plan_type: Some("Pro".to_string()),
            route_weight_tier: input
                .route_weight_tier
                .clone()
                .unwrap_or_else(|| "auto".to_string()),
            primary_remaining_percent: Some(100.0),
            secondary_remaining_percent: Some(100.0),
            rate_limit_reset_credits_available: Some(1),
            map_gpt53_codex_to_spark: input.map_gpt53_codex_to_spark.unwrap_or(false),
            auto_refresh_enabled: input.auto_refresh_enabled.unwrap_or(true),
            request_max_concurrency: input.request_max_concurrency,
            request_rpm_limit: input
                .request_rpm_limit
                .unwrap_or_else(default_codex_account_rpm_limit),
            request_min_start_interval_ms: input.request_min_start_interval_ms,
            codex_image_generation_enabled: input.codex_image_generation_enabled.unwrap_or(false),
            codex_image_generation_max_concurrency: input
                .codex_image_generation_max_concurrency
                .unwrap_or_else(default_codex_image_generation_max_concurrency),
            proxy_mode: input
                .proxy_mode
                .clone()
                .unwrap_or_else(|| "inherit".to_string()),
            proxy_config_id: input.proxy_config_id.clone(),
            effective_proxy_source: "binding".to_string(),
            effective_proxy_url: Some("http://127.0.0.1:11111".to_string()),
            effective_proxy_config_name: None,
            last_refresh: None,
            access_token_expires_at: None,
            auth_refresh_error_message: None,
            last_usage_checked_at: None,
            last_usage_success_at: None,
            usage_error_message: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/accounts/{}",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_patch(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn refresh_admin_llm_gateway_account(name: &str) -> Result<AccountSummaryView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AccountSummaryView {
            name: name.to_string(),
            status: "active".to_string(),
            account_id: None,
            email: None,
            plan_type: Some("Pro".to_string()),
            route_weight_tier: "auto".to_string(),
            primary_remaining_percent: Some(100.0),
            secondary_remaining_percent: Some(100.0),
            rate_limit_reset_credits_available: Some(1),
            map_gpt53_codex_to_spark: false,
            auto_refresh_enabled: true,
            request_max_concurrency: None,
            request_rpm_limit: default_codex_account_rpm_limit(),
            request_min_start_interval_ms: None,
            codex_image_generation_enabled: false,
            codex_image_generation_max_concurrency: default_codex_image_generation_max_concurrency(
            ),
            proxy_mode: "inherit".to_string(),
            proxy_config_id: None,
            effective_proxy_source: "binding".to_string(),
            effective_proxy_url: Some("http://127.0.0.1:11111".to_string()),
            effective_proxy_config_name: None,
            last_refresh: None,
            access_token_expires_at: None,
            auth_refresh_error_message: None,
            last_usage_checked_at: None,
            last_usage_success_at: None,
            usage_error_message: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/accounts/{}/refresh-usage",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_post(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminLlmGatewayAccountModelsProbeView {
    pub ok: bool,
    pub message: String,
    pub checked_at: i64,
}

pub async fn refresh_admin_llm_gateway_account_auth(
    name: &str,
) -> Result<AccountSummaryView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AccountSummaryView {
            name: name.to_string(),
            status: "active".to_string(),
            account_id: None,
            email: None,
            plan_type: Some("Pro".to_string()),
            route_weight_tier: "auto".to_string(),
            primary_remaining_percent: Some(100.0),
            secondary_remaining_percent: Some(100.0),
            rate_limit_reset_credits_available: Some(1),
            map_gpt53_codex_to_spark: false,
            auto_refresh_enabled: true,
            request_max_concurrency: None,
            request_rpm_limit: default_codex_account_rpm_limit(),
            request_min_start_interval_ms: None,
            codex_image_generation_enabled: false,
            codex_image_generation_max_concurrency: default_codex_image_generation_max_concurrency(
            ),
            proxy_mode: "inherit".to_string(),
            proxy_config_id: None,
            effective_proxy_source: "binding".to_string(),
            effective_proxy_url: Some("http://127.0.0.1:11111".to_string()),
            effective_proxy_config_name: None,
            last_refresh: None,
            access_token_expires_at: None,
            auth_refresh_error_message: None,
            last_usage_checked_at: None,
            last_usage_success_at: None,
            usage_error_message: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/accounts/{}/refresh-auth",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_post(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn refresh_admin_llm_gateway_account_usage(
    name: &str,
) -> Result<AccountSummaryView, String> {
    refresh_admin_llm_gateway_account(name).await
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct ConsumeCodexRateLimitResetCreditResponse {
    pub code: String,
    pub windows_reset: i64,
    pub account: AccountSummaryView,
    pub replayed: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ConsumeCodexRateLimitResetCreditRequest {
    pub idempotency_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct CodexRateLimitResetCreditsDetails {
    pub credits: Vec<CodexRateLimitResetCreditDetails>,
    pub available_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct CodexRateLimitResetCreditDetails {
    pub id: String,
    pub reset_type: String,
    pub status: String,
    pub granted_at: String,
    pub expires_at: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub async fn fetch_admin_llm_gateway_account_rate_limit_reset_credits(
    name: &str,
) -> Result<CodexRateLimitResetCreditsDetails, String> {
    #[cfg(feature = "mock")]
    {
        Ok(CodexRateLimitResetCreditsDetails {
            available_count: 1,
            credits: vec![CodexRateLimitResetCreditDetails {
                id: "mock-reset-credit".to_string(),
                reset_type: "codex_rate_limits".to_string(),
                status: "available".to_string(),
                granted_at: "2026-07-01T00:00:00Z".to_string(),
                expires_at: None,
                title: Some(format!("Reset credit for {name}")),
                description: None,
            }],
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/accounts/{}/rate-limit-reset-credits",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn consume_admin_llm_gateway_account_rate_limit_reset_credit(
    name: &str,
    request: &ConsumeCodexRateLimitResetCreditRequest,
) -> Result<ConsumeCodexRateLimitResetCreditResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(ConsumeCodexRateLimitResetCreditResponse {
            code: "reset".to_string(),
            windows_reset: 2,
            replayed: false,
            account: AccountSummaryView {
                name: name.to_string(),
                status: "active".to_string(),
                account_id: None,
                email: None,
                plan_type: Some("Pro".to_string()),
                route_weight_tier: "auto".to_string(),
                primary_remaining_percent: Some(100.0),
                secondary_remaining_percent: Some(100.0),
                rate_limit_reset_credits_available: Some(0),
                map_gpt53_codex_to_spark: false,
                auto_refresh_enabled: true,
                request_max_concurrency: None,
                request_rpm_limit: default_codex_account_rpm_limit(),
                request_min_start_interval_ms: None,
                codex_image_generation_enabled: false,
                codex_image_generation_max_concurrency:
                    default_codex_image_generation_max_concurrency(),
                proxy_mode: "inherit".to_string(),
                proxy_config_id: None,
                effective_proxy_source: "binding".to_string(),
                effective_proxy_url: Some("http://127.0.0.1:11111".to_string()),
                effective_proxy_config_name: None,
                last_refresh: None,
                access_token_expires_at: None,
                auth_refresh_error_message: None,
                last_usage_checked_at: None,
                last_usage_success_at: Some(Date::now() as i64),
                usage_error_message: None,
            },
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/accounts/{}/rate-limit-reset-credits/consume",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_post(&url)
            .json(request)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn probe_admin_llm_gateway_account_models(
    name: &str,
) -> Result<AdminLlmGatewayAccountModelsProbeView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminLlmGatewayAccountModelsProbeView {
            ok: true,
            message: "Codex models probe succeeded".to_string(),
            checked_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/accounts/{}/probe-models",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_post(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroBalanceView {
    pub current_usage: f64,
    pub usage_limit: f64,
    pub remaining: f64,
    pub upstream_usage_limit: Option<f64>,
    pub manual_usage_limit: Option<f64>,
    pub next_reset_at: Option<i64>,
    pub subscription_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroPublicStatusView {
    pub name: String,
    pub provider: Option<String>,
    pub disabled: bool,
    pub disabled_reason: Option<String>,
    pub subscription_title: Option<String>,
    pub current_usage: Option<f64>,
    pub usage_limit: Option<f64>,
    pub remaining: Option<f64>,
    pub next_reset_at: Option<i64>,
    pub cache: KiroCacheView,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroModelView {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub model_type: String,
    pub max_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroModelsResponse {
    pub object: String,
    pub data: Vec<KiroModelView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroAccessResponse {
    pub base_url: String,
    pub gateway_path: String,
    pub auth_cache_ttl_seconds: u64,
    pub accounts: Vec<KiroPublicStatusView>,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroCacheView {
    pub status: String,
    pub refresh_interval_seconds: u64,
    pub last_checked_at: Option<i64>,
    pub last_success_at: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroPrefixTreeRuntimeStats {
    pub resident_tokens: u64,
    pub max_tokens: u64,
    pub node_count: usize,
    pub leaf_count: usize,
    pub edge_count: usize,
    pub child_capacity: usize,
    pub estimated_memory_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroConversationAnchorRuntimeStats {
    pub entries: usize,
    pub max_entries: usize,
    pub estimated_memory_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminKiroCacheStatsResponse {
    pub mode: String,
    pub page_size_tokens: usize,
    pub prefix_tree: KiroPrefixTreeRuntimeStats,
    pub conversation_anchors: KiroConversationAnchorRuntimeStats,
    pub process_memory: ProcessMemoryRuntimeStats,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminAnthropicUpstreamUsageRollupView {
    pub input_uncached_tokens: u64,
    pub input_cached_tokens: u64,
    pub output_tokens: u64,
    pub billable_tokens: u64,
    pub usage_missing_events: u64,
    pub last_used_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminAnthropicUpstreamChannelView {
    pub name: String,
    pub status: String,
    pub base_url: String,
    pub has_api_key: bool,
    pub weight: u64,
    pub max_concurrency: u64,
    #[serde(default = "default_anthropic_upstream_rpm_limit")]
    pub rpm_limit: u64,
    pub min_start_interval_ms: u64,
    pub cache_hit_rate_limits: Vec<llm_access_core::store::AnthropicCacheHitRateLimit>,
    pub proxy_mode: String,
    pub proxy_config_id: Option<String>,
    pub last_error: Option<String>,
    pub models: Vec<String>,
    pub last_models_status: Option<String>,
    pub last_models_latency_ms: Option<u64>,
    pub last_models_checked_at: Option<i64>,
    pub last_models_error: Option<String>,
    pub last_test_model: Option<String>,
    pub last_test_status: Option<String>,
    pub last_test_latency_ms: Option<u64>,
    pub last_test_at: Option<i64>,
    pub last_test_error: Option<String>,
    pub usage: AdminAnthropicUpstreamUsageRollupView,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminAnthropicUpstreamProbeResponseView {
    pub ok: bool,
    pub status: String,
    pub status_code: Option<u16>,
    pub latency_ms: u64,
    pub error: Option<String>,
    pub channel: AdminAnthropicUpstreamChannelView,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminAnthropicUpstreamChannelsResponse {
    pub channels: Vec<AdminAnthropicUpstreamChannelView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct CreateAdminAnthropicUpstreamChannelInput {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub status: Option<String>,
    pub weight: Option<u64>,
    pub max_concurrency: Option<u64>,
    pub rpm_limit: Option<u64>,
    pub min_start_interval_ms: Option<u64>,
    pub cache_hit_rate_limits: Vec<llm_access_core::store::AnthropicCacheHitRateLimit>,
    pub proxy_mode: Option<String>,
    pub proxy_config_id: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct PatchAdminAnthropicUpstreamChannelInput {
    pub status: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub weight: Option<u64>,
    pub max_concurrency: Option<u64>,
    pub rpm_limit: Option<u64>,
    pub min_start_interval_ms: Option<u64>,
    pub cache_hit_rate_limits: Option<Vec<llm_access_core::store::AnthropicCacheHitRateLimit>>,
    pub proxy_mode: Option<String>,
    pub proxy_config_id: Option<Option<String>>,
    pub clear_last_error: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct TestAdminAnthropicUpstreamModelInput {
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct ModerationKeywordView {
    pub id: i64,
    pub keyword: String,
    pub categories: Vec<String>,
    pub note: Option<String>,
    pub source: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct ModerationCategoryView {
    pub code: String,
    pub label: String,
    pub description: String,
    pub severity: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminModerationCategoriesResponse {
    pub categories: Vec<ModerationCategoryView>,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct AddAdminModerationCategoryInput {
    pub code: String,
    pub label: String,
    pub description: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct AddAdminModerationCategoriesInput {
    pub categories: Vec<AddAdminModerationCategoryInput>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct ModerationGateStatsView {
    pub loaded: bool,
    pub loaded_at_ms: Option<i64>,
    pub keyword_count: usize,
    pub banned_session_count: usize,
    pub suppressed_hit_count: usize,
    pub blocked_requests_total: u64,
    pub persist_failures_total: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminModerationKeywordsResponse {
    pub keywords: Vec<ModerationKeywordView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
    pub stats: ModerationGateStatsView,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct AddAdminModerationKeywordsInput {
    pub content: String,
    pub format: Option<String>,
    pub note: Option<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AddAdminModerationKeywordsResponse {
    pub inserted: usize,
    pub duplicates: usize,
    pub parsed: usize,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct ModerationBannedSessionView {
    pub id: i64,
    pub hit_key: String,
    pub session_key: String,
    pub provider: String,
    pub key_id: String,
    pub key_name: String,
    pub session_id: String,
    pub matched_keyword: String,
    pub matched_categories: Vec<String>,
    pub matched_context: String,
    pub match_start: i64,
    pub match_end: i64,
    pub match_prefix_sha256: String,
    pub keyword_set_hash: String,
    pub endpoint: String,
    pub model: String,
    pub client_ip: String,
    pub status: String,
    pub review_note: Option<String>,
    pub banned_at_ms: i64,
    pub reviewed_at_ms: Option<i64>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct ModerationBannedSessionDetailView {
    pub session: ModerationBannedSessionView,
    pub request_headers_json: String,
    pub request_body_json: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminModerationBannedSessionsResponse {
    pub sessions: Vec<ModerationBannedSessionView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct ReviewModerationBannedSessionInput {
    pub banned: bool,
    pub review_note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct KiroAccountView {
    pub name: String,
    pub auth_method: String,
    pub provider: Option<String>,
    pub email: Option<String>,
    pub expires_at: Option<String>,
    pub profile_arn: Option<String>,
    pub has_refresh_token: bool,
    pub disabled: bool,
    pub disabled_reason: Option<String>,
    pub issue_kind: Option<String>,
    pub issue_summary: Option<String>,
    pub issue_at_ms: Option<i64>,
    pub source: Option<String>,
    pub source_db_path: Option<String>,
    pub last_imported_at: Option<i64>,
    pub subscription_title: Option<String>,
    pub region: Option<String>,
    pub auth_region: Option<String>,
    pub api_region: Option<String>,
    pub machine_id: Option<String>,
    pub kiro_channel_max_concurrency: u64,
    #[serde(default = "default_kiro_channel_rpm_limit")]
    pub kiro_channel_rpm_limit: u64,
    pub kiro_channel_min_start_interval_ms: u64,
    pub minimum_remaining_credits_before_block: f64,
    pub manual_usage_limit: Option<f64>,
    #[serde(default = "default_kiro_pool_strategy")]
    pub pool_strategy: String,
    pub proxy_mode: String,
    pub proxy_config_id: Option<String>,
    pub effective_proxy_source: String,
    pub effective_proxy_url: Option<String>,
    pub effective_proxy_config_name: Option<String>,
    pub proxy_url: Option<String>,
    pub balance: Option<KiroBalanceView>,
    pub cache: KiroCacheView,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct CreateManualKiroAccountInput {
    pub name: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub profile_arn: Option<String>,
    pub expires_at: Option<String>,
    pub auth_method: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub region: Option<String>,
    pub auth_region: Option<String>,
    pub api_region: Option<String>,
    pub machine_id: Option<String>,
    pub provider: Option<String>,
    pub email: Option<String>,
    pub subscription_title: Option<String>,
    pub kiro_channel_max_concurrency: Option<u64>,
    pub kiro_channel_rpm_limit: Option<u64>,
    pub kiro_channel_min_start_interval_ms: Option<u64>,
    pub minimum_remaining_credits_before_block: Option<f64>,
    pub manual_usage_limit: Option<f64>,
    pub pool_strategy: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub struct PatchKiroAccountInput {
    pub status: Option<String>,
    pub kiro_channel_max_concurrency: Option<u64>,
    pub kiro_channel_rpm_limit: Option<u64>,
    pub kiro_channel_min_start_interval_ms: Option<u64>,
    pub minimum_remaining_credits_before_block: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_usage_limit: Option<Option<f64>>,
    pub pool_strategy: Option<String>,
    pub proxy_mode: Option<String>,
    pub proxy_config_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminKiroAccountsResponse {
    pub accounts: Vec<KiroAccountView>,
    pub summary: AdminAccountsSummaryView,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub has_more: bool,
    pub generated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
pub struct AdminKiroAccountStatusesResponse {
    pub accounts: Vec<KiroAccountView>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
    pub generated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdminKiroAccountStatusesQuery {
    pub prefix: Option<String>,
    pub q: Option<String>,
    pub issue: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(any(not(feature = "mock"), test))]
pub(super) fn build_admin_kiro_account_statuses_url(
    query: &AdminKiroAccountStatusesQuery,
) -> String {
    let mut url = format!("{}/admin/kiro-gateway/accounts/statuses", llm_access_admin_base());
    let mut params = Vec::new();
    if let Some(prefix) = query.prefix.as_deref() {
        params.push(format!("prefix={}", urlencoding::encode(prefix)));
    }
    if let Some(q) = query.q.as_deref() {
        params.push(format!("q={}", urlencoding::encode(q)));
    }
    if let Some(issue) = query.issue.as_deref() {
        params.push(format!("issue={}", urlencoding::encode(issue)));
    }
    if let Some(limit) = query.limit {
        params.push(format!("limit={limit}"));
    }
    if let Some(offset) = query.offset {
        params.push(format!("offset={offset}"));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    url
}

#[cfg(any(not(feature = "mock"), test))]
pub(super) fn build_admin_kiro_cache_stats_url_for_ts(ts: u64) -> String {
    format!("{}/admin/kiro-gateway/cache-stats?_ts={ts}", llm_access_admin_base())
}

#[cfg(any(not(feature = "mock"), test))]
pub(super) fn build_admin_kiro_usage_event_detail_url(event_id: &str) -> String {
    format!(
        "{}/admin/kiro-gateway/usage/{}",
        llm_access_admin_base(),
        urlencoding::encode(event_id)
    )
}

pub async fn fetch_kiro_access() -> Result<KiroAccessResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(KiroAccessResponse {
            base_url: "http://localhost:3000/api/kiro-gateway".to_string(),
            gateway_path: "/api/kiro-gateway".to_string(),
            auth_cache_ttl_seconds: 60,
            accounts: vec![],
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/kiro-gateway/access?_ts={}", API_BASE, Date::now() as u64);
        let response = api_get(&url)
            .header("Cache-Control", "no-cache, no-store, max-age=0")
            .header("Pragma", "no-cache")
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_kiro_models() -> Result<KiroModelsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(KiroModelsResponse {
            object: "list".to_string(),
            data: vec![
                KiroModelView {
                    id: "claude-sonnet-4-6".to_string(),
                    object: "model".to_string(),
                    created: 1_770_314_400,
                    owned_by: "anthropic".to_string(),
                    display_name: "Claude Sonnet 4.6".to_string(),
                    model_type: "chat".to_string(),
                    max_tokens: 32_000,
                },
                KiroModelView {
                    id: "claude-haiku-4-5-20251001".to_string(),
                    object: "model".to_string(),
                    created: 1_727_740_800,
                    owned_by: "anthropic".to_string(),
                    display_name: "Claude Haiku 4.5".to_string(),
                    model_type: "chat".to_string(),
                    max_tokens: 32_000,
                },
            ],
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/kiro-gateway/v1/models", API_BASE);
        let response = api_get(&url)
            .header("Cache-Control", "no-cache, no-store, max-age=0")
            .header("Pragma", "no-cache")
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_kiro_keys_page(
    limit: usize,
    offset: usize,
) -> Result<AdminLlmGatewayKeysResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminLlmGatewayKeysResponse {
            keys: vec![],
            summary: AdminLlmGatewayKeysSummaryView::default(),
            auth_cache_ttl_seconds: 60,
            total: 0,
            limit,
            offset,
            has_more: false,
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let cache_buster = Date::now() as u64;
        let url = format!(
            "{}/admin/kiro-gateway/keys?limit={limit}&offset={offset}&_ts={cache_buster}",
            llm_access_admin_base()
        );
        let response = api_get(&url)
            .header("Cache-Control", "no-cache, no-store, max-age=0")
            .header("Pragma", "no-cache")
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_kiro_account_groups_page(
    limit: usize,
    offset: usize,
) -> Result<AdminAccountGroupsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminAccountGroupsResponse {
            groups: vec![],
            total: 0,
            limit,
            offset,
            has_more: false,
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/account-groups?limit={limit}&offset={offset}",
            llm_access_admin_base()
        );
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_kiro_account_group_options(
) -> Result<Vec<AdminAccountGroupOptionView>, String> {
    #[cfg(feature = "mock")]
    {
        Ok(Vec::new())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/kiro-gateway/account-group-options", llm_access_admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json::<AdminAccountGroupOptionsResponse>()
            .await
            .map(|resp| resp.options)
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn create_admin_kiro_account_group(
    input: CreateAdminAccountGroupInput<'_>,
) -> Result<AdminAccountGroupView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminAccountGroupView {
            id: "mock-kiro-group".to_string(),
            provider_type: "kiro".to_string(),
            name: input.name.to_string(),
            account_names: input.account_names.to_vec(),
            created_at: 0,
            updated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/kiro-gateway/account-groups", llm_access_admin_base());
        let response = api_post(&url)
            .json(&serde_json::json!({
                "name": input.name,
                "account_names": input.account_names,
            }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn patch_admin_kiro_account_group(
    group_id: &str,
    input: PatchAdminAccountGroupInput<'_>,
) -> Result<AdminAccountGroupView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminAccountGroupView {
            id: group_id.to_string(),
            provider_type: "kiro".to_string(),
            name: input.name.unwrap_or("mock").to_string(),
            account_names: input
                .account_names
                .map(|value| value.to_vec())
                .unwrap_or_default(),
            created_at: 0,
            updated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/account-groups/{}",
            llm_access_admin_base(),
            urlencoding::encode(group_id)
        );
        let mut body = serde_json::Map::new();
        if let Some(name) = input.name.map(str::trim).filter(|value| !value.is_empty()) {
            body.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        }
        if let Some(account_names) = input.account_names {
            body.insert(
                "account_names".to_string(),
                serde_json::Value::Array(
                    account_names
                        .iter()
                        .map(|value| serde_json::Value::String(value.clone()))
                        .collect(),
                ),
            );
        }
        let response = api_patch(&url)
            .json(&serde_json::Value::Object(body))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn delete_admin_kiro_account_group(group_id: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = group_id;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/account-groups/{}",
            llm_access_admin_base(),
            urlencoding::encode(group_id)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

pub async fn create_admin_kiro_key(
    name: &str,
    quota_billable_limit: u64,
) -> Result<AdminLlmGatewayKeyView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminLlmGatewayKeyView {
            id: "mock-kiro".to_string(),
            name: name.to_string(),
            secret: "sf-kiro-mock".to_string(),
            key_hash: "hash".to_string(),
            status: "active".to_string(),
            provider_type: "kiro".to_string(),
            public_visible: false,
            quota_billable_limit,
            usage_input_uncached_tokens: 0,
            usage_input_cached_tokens: 0,
            usage_output_tokens: 0,
            usage_credit_total: 0.0,
            usage_credit_missing_events: 0,
            codex_image_usage_tokens: 0,
            codex_image_usage_missing_events: 0,
            codex_image_last_used_at: None,
            remaining_billable: quota_billable_limit as i64,
            last_used_at: None,
            created_at: 0,
            updated_at: 0,
            route_strategy: None,
            account_group_id: None,
            fixed_account_name: None,
            auto_account_names: None,
            preferred_pool_strategy: default_kiro_pool_strategy(),
            kiro_anthropic_upstream_pool_mode: default_anthropic_upstream_pool_mode(),
            model_name_map: None,
            kiro_model_group_preferences: BTreeMap::new(),
            kiro_model_channel_preferences: BTreeMap::new(),
            request_max_concurrency: None,
            request_min_start_interval_ms: None,
            moderation_enabled: true,
            kiro_request_validation_enabled: true,
            kiro_cache_estimation_enabled: true,
            kiro_zero_cache_debug_enabled: false,
            kiro_full_request_logging_enabled: false,
            kiro_remote_media_resolution_enabled: false,
            kiro_latency_routing_enabled: true,
            kiro_protected_content_validation_enabled: false,
            kiro_cctest_text_handling_enabled: false,
            kiro_cache_policy_override_json: None,
            kiro_billable_model_multipliers_override_json: None,
            effective_kiro_cache_policy_json: String::new(),
            uses_global_kiro_cache_policy: true,
            effective_kiro_billable_model_multipliers_json:
                default_kiro_billable_model_multipliers_json(),
            uses_global_kiro_billable_model_multipliers: true,
            codex_fast_enabled: true,
            codex_responses_lite_enabled: true,
            codex_strict_session_rejection_enabled: false,
            codex_image_generation_enabled: false,
            codex_image_standalone_generation_enabled: false,
            codex_image_direct_generation_enabled: false,
            kiro_candidate_credit_summary: None,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/kiro-gateway/keys", llm_access_admin_base());
        let response = api_post(&url)
            .json(&serde_json::json!({
                "name": name,
                "quota_billable_limit": quota_billable_limit
            }))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn patch_admin_kiro_key(
    key_id: &str,
    request: PatchAdminLlmGatewayKeyRequest<'_>,
) -> Result<AdminLlmGatewayKeyView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (
            key_id,
            request.name,
            request.status,
            request.public_visible,
            request.quota_billable_limit,
            request.route_strategy,
            request.account_group_id,
            request.fixed_account_name,
            request.auto_account_names,
            request.preferred_pool_strategy,
            request.kiro_anthropic_upstream_pool_mode,
            request.model_name_map,
            request.kiro_model_group_preferences,
            request.kiro_model_channel_preferences,
            request.request_max_concurrency,
            request.request_min_start_interval_ms,
            request.moderation_enabled,
            request.kiro_request_validation_enabled,
            request.kiro_cache_estimation_enabled,
            request.kiro_zero_cache_debug_enabled,
            request.kiro_full_request_logging_enabled,
            request.kiro_remote_media_resolution_enabled,
            request.kiro_latency_routing_enabled,
            request.kiro_protected_content_validation_enabled,
            request.kiro_cctest_text_handling_enabled,
            request.kiro_cache_policy_override_json,
            request.kiro_billable_model_multipliers_override_json,
            request.request_max_concurrency_unlimited,
            request.request_min_start_interval_ms_unlimited,
        );
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/keys/{}",
            llm_access_admin_base(),
            urlencoding::encode(key_id)
        );
        let mut body = serde_json::Map::new();
        if let Some(name) = request
            .name
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            body.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        }
        if let Some(status) = request
            .status
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            body.insert("status".to_string(), serde_json::Value::String(status.to_string()));
        }
        if let Some(public_visible) = request.public_visible {
            body.insert("public_visible".to_string(), serde_json::Value::Bool(public_visible));
        }
        if let Some(quota_billable_limit) = request.quota_billable_limit {
            body.insert(
                "quota_billable_limit".to_string(),
                serde_json::Value::Number(quota_billable_limit.into()),
            );
        }
        if let Some(strategy) = request.route_strategy {
            body.insert(
                "route_strategy".to_string(),
                serde_json::Value::String(strategy.to_string()),
            );
        }
        if let Some(group_id) = request.account_group_id {
            body.insert(
                "account_group_id".to_string(),
                serde_json::Value::String(group_id.to_string()),
            );
        }
        if let Some(account_name) = request.fixed_account_name {
            body.insert(
                "fixed_account_name".to_string(),
                serde_json::Value::String(account_name.to_string()),
            );
        }
        if let Some(account_names) = request.auto_account_names {
            body.insert(
                "auto_account_names".to_string(),
                serde_json::Value::Array(
                    account_names
                        .iter()
                        .map(|value| serde_json::Value::String(value.clone()))
                        .collect(),
                ),
            );
        }
        if let Some(preferred_pool_strategy) = request.preferred_pool_strategy {
            body.insert(
                "preferred_pool_strategy".to_string(),
                serde_json::Value::String(preferred_pool_strategy.to_string()),
            );
        }
        if let Some(mode) = request.kiro_anthropic_upstream_pool_mode {
            body.insert(
                "kiro_anthropic_upstream_pool_mode".to_string(),
                serde_json::Value::String(mode.to_string()),
            );
        }
        if let Some(model_name_map) = request.model_name_map {
            let value = serde_json::to_value(model_name_map)
                .map_err(|e| format!("Serialize error: {:?}", e))?;
            body.insert("model_name_map".to_string(), value);
        }
        if let Some(preferences) = request.kiro_model_group_preferences {
            let value = serde_json::to_value(preferences)
                .map_err(|e| format!("Serialize error: {:?}", e))?;
            body.insert("kiro_model_group_preferences".to_string(), value);
        }
        if let Some(preferences) = request.kiro_model_channel_preferences {
            let value = serde_json::to_value(preferences)
                .map_err(|e| format!("Serialize error: {:?}", e))?;
            body.insert("kiro_model_channel_preferences".to_string(), value);
        }
        if let Some(moderation_enabled) = request.moderation_enabled {
            body.insert(
                "moderation_enabled".to_string(),
                serde_json::Value::Bool(moderation_enabled),
            );
        }
        if let Some(kiro_request_validation_enabled) = request.kiro_request_validation_enabled {
            body.insert(
                "kiro_request_validation_enabled".to_string(),
                serde_json::Value::Bool(kiro_request_validation_enabled),
            );
        }
        if let Some(kiro_cache_estimation_enabled) = request.kiro_cache_estimation_enabled {
            body.insert(
                "kiro_cache_estimation_enabled".to_string(),
                serde_json::Value::Bool(kiro_cache_estimation_enabled),
            );
        }
        if let Some(kiro_zero_cache_debug_enabled) = request.kiro_zero_cache_debug_enabled {
            body.insert(
                "kiro_zero_cache_debug_enabled".to_string(),
                serde_json::Value::Bool(kiro_zero_cache_debug_enabled),
            );
        }
        if let Some(kiro_full_request_logging_enabled) = request.kiro_full_request_logging_enabled {
            body.insert(
                "kiro_full_request_logging_enabled".to_string(),
                serde_json::Value::Bool(kiro_full_request_logging_enabled),
            );
        }
        if let Some(kiro_remote_media_resolution_enabled) =
            request.kiro_remote_media_resolution_enabled
        {
            body.insert(
                "kiro_remote_media_resolution_enabled".to_string(),
                serde_json::Value::Bool(kiro_remote_media_resolution_enabled),
            );
        }
        if let Some(kiro_latency_routing_enabled) = request.kiro_latency_routing_enabled {
            body.insert(
                "kiro_latency_routing_enabled".to_string(),
                serde_json::Value::Bool(kiro_latency_routing_enabled),
            );
        }
        if let Some(kiro_protected_content_validation_enabled) =
            request.kiro_protected_content_validation_enabled
        {
            body.insert(
                "kiro_protected_content_validation_enabled".to_string(),
                serde_json::Value::Bool(kiro_protected_content_validation_enabled),
            );
        }
        if let Some(kiro_cctest_text_handling_enabled) = request.kiro_cctest_text_handling_enabled {
            body.insert(
                "kiro_cctest_text_handling_enabled".to_string(),
                serde_json::Value::Bool(kiro_cctest_text_handling_enabled),
            );
        }
        if let Some(kiro_cache_policy_override_json) = request.kiro_cache_policy_override_json {
            body.insert(
                "kiro_cache_policy_override_json".to_string(),
                kiro_cache_policy_override_json
                    .map(|raw| serde_json::Value::String(raw.to_string()))
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        if let Some(kiro_billable_model_multipliers_override_json) =
            request.kiro_billable_model_multipliers_override_json
        {
            body.insert(
                "kiro_billable_model_multipliers_override_json".to_string(),
                kiro_billable_model_multipliers_override_json
                    .map(|raw| serde_json::Value::String(raw.to_string()))
                    .unwrap_or(serde_json::Value::Null),
            );
        }
        let response = api_patch(&url)
            .json(&serde_json::Value::Object(body))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn delete_admin_kiro_key(key_id: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = key_id;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/keys/{}",
            llm_access_admin_base(),
            urlencoding::encode(key_id)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

pub async fn fetch_admin_kiro_usage_events(
    query: &AdminLlmGatewayUsageEventsQuery,
) -> Result<AdminLlmGatewayUsageEventsResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = query;
        Ok(AdminLlmGatewayUsageEventsResponse {
            total: 0,
            offset: 0,
            limit: 20,
            has_more: false,
            current_rpm: 0,
            current_in_flight: 0,
            retention_days: default_usage_analytics_retention_days(),
            totals: AdminUsageTotalsView::default(),
            events: vec![],
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!("{}/admin/kiro-gateway/usage", llm_access_admin_base());
        let mut params = Vec::new();
        if let Some(key_id) = query
            .key_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("key_id={}", urlencoding::encode(key_id)));
        }
        if let Some(start_ms) = query.start_ms {
            params.push(format!("start_ms={start_ms}"));
        }
        if let Some(end_ms) = query.end_ms {
            params.push(format!("end_ms={end_ms}"));
        }
        if let Some(source) = query
            .source
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("source={}", urlencoding::encode(source)));
        }
        if let Some(model) = query
            .model
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("model={}", urlencoding::encode(model)));
        }
        if let Some(account_name) = query
            .account_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("account_name={}", urlencoding::encode(account_name)));
        }
        if let Some(endpoint) = query
            .endpoint
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("endpoint={}", urlencoding::encode(endpoint)));
        }
        if let Some(status_code) = query.status_code {
            params.push(format!("status_code={status_code}"));
        }
        if let Some(status_kind) = query
            .status_kind
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            params.push(format!("status_kind={}", urlencoding::encode(status_kind)));
        }
        if let Some(limit) = query.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(offset) = query.offset {
            params.push(format!("offset={offset}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_kiro_usage_event_detail(
    event_id: &str,
) -> Result<AdminLlmGatewayUsageEventDetailView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = event_id;
        Ok(AdminLlmGatewayUsageEventDetailView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = build_admin_kiro_usage_event_detail_url(event_id);
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_anthropic_upstream_channels(
) -> Result<AdminAnthropicUpstreamChannelsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminAnthropicUpstreamChannelsResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/kiro-gateway/anthropic-upstreams", llm_access_admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn create_admin_anthropic_upstream_channel(
    input: &CreateAdminAnthropicUpstreamChannelInput,
) -> Result<AdminAnthropicUpstreamChannelView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = input;
        Ok(AdminAnthropicUpstreamChannelView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/kiro-gateway/anthropic-upstreams", llm_access_admin_base());
        let response = api_post(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn patch_admin_anthropic_upstream_channel(
    name: &str,
    input: &PatchAdminAnthropicUpstreamChannelInput,
) -> Result<AdminAnthropicUpstreamChannelView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (name, input);
        Ok(AdminAnthropicUpstreamChannelView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/anthropic-upstreams/{}",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_patch(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn refresh_admin_anthropic_upstream_models(
    name: &str,
) -> Result<AdminAnthropicUpstreamProbeResponseView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = name;
        Ok(AdminAnthropicUpstreamProbeResponseView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/anthropic-upstreams/{}/refresh-models",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_post(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn test_admin_anthropic_upstream_model(
    name: &str,
    input: &TestAdminAnthropicUpstreamModelInput,
) -> Result<AdminAnthropicUpstreamProbeResponseView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (name, input);
        Ok(AdminAnthropicUpstreamProbeResponseView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/anthropic-upstreams/{}/test",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_post(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn delete_admin_anthropic_upstream_channel(name: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = name;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/anthropic-upstreams/{}",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

pub async fn fetch_admin_moderation_categories() -> Result<AdminModerationCategoriesResponse, String>
{
    #[cfg(feature = "mock")]
    {
        Ok(AdminModerationCategoriesResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/moderation/categories", llm_access_admin_base());
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn add_admin_moderation_categories(
    input: &AddAdminModerationCategoriesInput,
) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = input;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/moderation/categories", llm_access_admin_base());
        let response = api_post(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

pub async fn delete_admin_moderation_category(code: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = code;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/moderation/categories/{}",
            llm_access_admin_base(),
            urlencoding::encode(code)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

pub async fn fetch_admin_moderation_keywords(
    search: &str,
    limit: usize,
    offset: usize,
) -> Result<AdminModerationKeywordsResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (search, limit, offset);
        Ok(AdminModerationKeywordsResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut params = vec![format!("limit={limit}"), format!("offset={offset}")];
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            params.push(format!("q={}", urlencoding::encode(trimmed)));
        }
        let url = format!(
            "{}/admin/llm-gateway/moderation/keywords?{}",
            llm_access_admin_base(),
            params.join("&")
        );
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn add_admin_moderation_keywords(
    input: &AddAdminModerationKeywordsInput,
) -> Result<AddAdminModerationKeywordsResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = input;
        Ok(AddAdminModerationKeywordsResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/moderation/keywords", llm_access_admin_base());
        let response = api_post(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn delete_admin_moderation_keyword(id: i64) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = id;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/llm-gateway/moderation/keywords/{id}", llm_access_admin_base());
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}

pub async fn fetch_admin_moderation_banned_sessions(
    status: &str,
    search: &str,
    limit: usize,
    offset: usize,
) -> Result<AdminModerationBannedSessionsResponse, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (status, search, limit, offset);
        Ok(AdminModerationBannedSessionsResponse::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut url = format!(
            "{}/admin/llm-gateway/moderation/banned-sessions?status={}&limit={limit}&\
             offset={offset}",
            llm_access_admin_base(),
            urlencoding::encode(status)
        );
        let search = search.trim();
        if !search.is_empty() {
            url.push_str("&q=");
            url.push_str(&urlencoding::encode(search));
        }
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_moderation_banned_session(
    id: i64,
) -> Result<ModerationBannedSessionDetailView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = id;
        Ok(ModerationBannedSessionDetailView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/moderation/banned-sessions/{id}",
            llm_access_admin_base()
        );
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn review_admin_moderation_banned_session(
    id: i64,
    input: &ReviewModerationBannedSessionInput,
) -> Result<ModerationBannedSessionView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (id, input);
        Ok(ModerationBannedSessionView::default())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/llm-gateway/moderation/banned-sessions/{id}/review",
            llm_access_admin_base()
        );
        let response = api_post(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_kiro_accounts() -> Result<AdminKiroAccountsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminKiroAccountsResponse {
            accounts: vec![],
            summary: AdminAccountsSummaryView::default(),
            total: 0,
            limit: 0,
            offset: 0,
            has_more: false,
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let mut offset = 0;
        let mut result =
            fetch_admin_kiro_accounts_page(ADMIN_GATEWAY_INVENTORY_PAGE_LIMIT, offset).await?;
        while result.has_more {
            offset = result.accounts.len();
            let next =
                fetch_admin_kiro_accounts_page(ADMIN_GATEWAY_INVENTORY_PAGE_LIMIT, offset).await?;
            let returned = next.accounts.len();
            result = merge_admin_kiro_account_pages(result, next);
            if returned == 0 {
                break;
            }
        }
        result.has_more = false;
        result.limit = result.accounts.len();
        Ok(result)
    }
}

pub async fn fetch_admin_kiro_accounts_page(
    limit: usize,
    offset: usize,
) -> Result<AdminKiroAccountsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminKiroAccountsResponse {
            accounts: vec![],
            summary: AdminAccountsSummaryView::default(),
            total: 0,
            limit,
            offset,
            has_more: false,
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/accounts?limit={limit}&offset={offset}",
            llm_access_admin_base()
        );
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_kiro_account_statuses(
    query: &AdminKiroAccountStatusesQuery,
) -> Result<AdminKiroAccountStatusesResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminKiroAccountStatusesResponse {
            accounts: vec![],
            total: 0,
            limit: query.limit.unwrap_or(24),
            offset: query.offset.unwrap_or(0),
            generated_at: 0,
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = build_admin_kiro_account_statuses_url(query);
        let response = api_get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn fetch_admin_kiro_cache_stats() -> Result<AdminKiroCacheStatsResponse, String> {
    #[cfg(feature = "mock")]
    {
        Ok(AdminKiroCacheStatsResponse {
            mode: "prefix_tree".to_string(),
            page_size_tokens: 64,
            ..AdminKiroCacheStatsResponse::default()
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = build_admin_kiro_cache_stats_url_for_ts(Date::now() as u64);
        let response = api_get(&url)
            .header("Cache-Control", "no-cache, no-store, max-age=0")
            .header("Pragma", "no-cache")
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn import_admin_kiro_account(
    name: Option<&str>,
    sqlite_path: Option<&str>,
    kiro_channel_max_concurrency: Option<u64>,
    kiro_channel_rpm_limit: Option<u64>,
    kiro_channel_min_start_interval_ms: Option<u64>,
) -> Result<KiroAccountView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (
            name,
            sqlite_path,
            kiro_channel_max_concurrency,
            kiro_channel_rpm_limit,
            kiro_channel_min_start_interval_ms,
        );
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/kiro-gateway/accounts/import-local", llm_access_admin_base());
        let mut body = serde_json::Map::new();
        if let Some(name) = name.map(str::trim).filter(|value| !value.is_empty()) {
            body.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        }
        if let Some(path) = sqlite_path.map(str::trim).filter(|value| !value.is_empty()) {
            body.insert("sqlite_path".to_string(), serde_json::Value::String(path.to_string()));
        }
        if let Some(value) = kiro_channel_max_concurrency {
            body.insert(
                "kiro_channel_max_concurrency".to_string(),
                serde_json::Value::Number(serde_json::Number::from(value)),
            );
        }
        if let Some(value) = kiro_channel_rpm_limit {
            body.insert(
                "kiro_channel_rpm_limit".to_string(),
                serde_json::Value::Number(serde_json::Number::from(value)),
            );
        }
        if let Some(value) = kiro_channel_min_start_interval_ms {
            body.insert(
                "kiro_channel_min_start_interval_ms".to_string(),
                serde_json::Value::Number(serde_json::Number::from(value)),
            );
        }
        let response = api_post(&url)
            .json(&serde_json::Value::Object(body))
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn create_admin_kiro_manual_account(
    input: &CreateManualKiroAccountInput,
) -> Result<KiroAccountView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = input;
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!("{}/admin/kiro-gateway/accounts", llm_access_admin_base());
        let response = api_post(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn patch_admin_kiro_account(
    name: &str,
    input: &PatchKiroAccountInput,
) -> Result<KiroAccountView, String> {
    #[cfg(feature = "mock")]
    {
        let _ = (name, input);
        Err("mock not supported".to_string())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/accounts/{}",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_patch(&url)
            .json(input)
            .map_err(|e| format!("Serialize error: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn refresh_admin_kiro_account_balance(name: &str) -> Result<KiroBalanceView, String> {
    #[cfg(feature = "mock")]
    {
        Ok(KiroBalanceView {
            current_usage: 0.0,
            usage_limit: 1_000.0,
            remaining: 1_000.0,
            upstream_usage_limit: None,
            manual_usage_limit: None,
            next_reset_at: None,
            subscription_title: Some(format!("mock-{name}")),
        })
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/accounts/{}/balance",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_post(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}

pub async fn delete_admin_kiro_account(name: &str) -> Result<(), String> {
    #[cfg(feature = "mock")]
    {
        let _ = name;
        Ok(())
    }

    #[cfg(not(feature = "mock"))]
    {
        let url = format!(
            "{}/admin/kiro-gateway/accounts/{}",
            llm_access_admin_base(),
            urlencoding::encode(name)
        );
        let response = api_delete(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        if !response.ok() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed: {text}"));
        }
        Ok(())
    }
}
