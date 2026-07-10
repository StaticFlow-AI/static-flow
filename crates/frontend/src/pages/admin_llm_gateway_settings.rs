//! LLM gateway settings page (`/admin/llm-gateway/settings`).
//!
//! Owns the gateway-wide runtime config form, the provider proxy bindings,
//! the legacy Kiro proxy migration action, and the shared proxy-config
//! inventory. The heavyweight `ProxyConfigEditorCard` stays defined in
//! `admin_llm_gateway` and is reused here so the editor logic is not
//! duplicated.

use gloo_timers::callback::Timeout;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_llm_gateway::{
    normalize_optional_form_string, proxy_url_after_socks5h_confirmation, ProxyConfigEditorCard,
};
use crate::{
    api::{
        create_admin_llm_gateway_proxy_config, fetch_admin_llm_gateway_config,
        fetch_admin_llm_gateway_proxy_bindings, fetch_admin_llm_gateway_proxy_configs,
        import_admin_legacy_kiro_proxy_configs, update_admin_llm_gateway_config,
        update_admin_llm_gateway_proxy_binding, AdminUpstreamProxyBindingView,
        AdminUpstreamProxyConfigScopeView, AdminUpstreamProxyConfigView,
        CreateAdminUpstreamProxyConfigInput, LlmGatewayRuntimeConfig,
        DEFAULT_LLM_GATEWAY_CODEX_CLIENT_VERSION,
    },
    components::copy_button::copy_to_clipboard,
    pages::llm_access_shared::format_number_u64,
    router::Route,
};

#[function_component(AdminLlmGatewaySettingsPage)]
pub fn admin_llm_gateway_settings_page() -> Html {
    let config = use_state(|| None::<LlmGatewayRuntimeConfig>);
    let ttl_input = use_state(|| "60".to_string());
    let max_request_body_input = use_state(|| (8 * 1024 * 1024_u64).to_string());
    let account_failure_retry_limit_input = use_state(|| "10".to_string());
    let codex_client_version_input =
        use_state(|| DEFAULT_LLM_GATEWAY_CODEX_CLIENT_VERSION.to_string());
    let codex_refresh_min_input = use_state(|| "240".to_string());
    let codex_refresh_max_input = use_state(|| "300".to_string());
    let codex_account_jitter_max_input = use_state(|| "10".to_string());
    let codex_weight_free_input = use_state(|| "1".to_string());
    let codex_weight_plus_input = use_state(|| "10".to_string());
    let codex_weight_pro5x_input = use_state(|| "50".to_string());
    let codex_weight_pro20x_input = use_state(|| "200".to_string());
    let codex_session_affinity_enabled_input = use_state(|| true);
    let codex_session_affinity_max_entries_input = use_state(|| "20000".to_string());
    let codex_session_affinity_ttl_seconds_input = use_state(|| "21600".to_string());
    let codex_fallback_affinity_enabled_input = use_state(|| true);
    let codex_fallback_affinity_ttl_seconds_input = use_state(|| "1800".to_string());
    let codex_fallback_affinity_prefix_bytes_input = use_state(|| "4096".to_string());
    let codex_fallback_affinity_min_body_bytes_input = use_state(|| "128".to_string());
    let kiro_refresh_min_input = use_state(|| "240".to_string());
    let kiro_refresh_max_input = use_state(|| "300".to_string());
    let kiro_account_jitter_max_input = use_state(|| "10".to_string());
    let usage_flush_batch_size_input = use_state(|| "256".to_string());
    let usage_flush_interval_input = use_state(|| "15".to_string());
    let usage_flush_max_buffer_bytes_input = use_state(|| (8 * 1024 * 1024_u64).to_string());
    let duckdb_usage_memory_limit_mib_input = use_state(|| "1024".to_string());
    let duckdb_usage_checkpoint_threshold_mib_input = use_state(|| "16".to_string());
    let usage_analytics_retention_days_input = use_state(|| "7".to_string());
    let kiro_cctest_proxy_base_url_input = use_state(String::new);
    let kiro_cctest_proxy_api_key_input = use_state(String::new);
    let proxy_configs = use_state(Vec::<AdminUpstreamProxyConfigView>::new);
    let proxy_config_scope = use_state(AdminUpstreamProxyConfigScopeView::default);
    let proxy_bindings = use_state(Vec::<AdminUpstreamProxyBindingView>::new);
    let create_proxy_name = use_state(|| "shared-upstream".to_string());
    let create_proxy_url = use_state(|| "http://127.0.0.1:11111".to_string());
    let create_proxy_username = use_state(String::new);
    let create_proxy_password = use_state(String::new);
    let creating_proxy = use_state(|| false);
    let codex_proxy_binding_input = use_state(String::new);
    let kiro_proxy_binding_input = use_state(String::new);
    let saving_proxy_binding_provider = use_state(|| None::<String>);
    let migrating_legacy_kiro_proxy = use_state(|| false);
    let proxy_config_search = use_state(String::new);
    let proxy_config_active_query = use_state(String::new);
    let proxy_config_show_active_only = use_state(|| false);
    let saving_runtime_config = use_state(|| false);
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

    // Config + proxy inventory + provider bindings load together; the form
    // inputs re-sync from the fetched config on every refresh.
    {
        let config = config.clone();
        let ttl_input = ttl_input.clone();
        let max_request_body_input = max_request_body_input.clone();
        let account_failure_retry_limit_input = account_failure_retry_limit_input.clone();
        let codex_client_version_input = codex_client_version_input.clone();
        let codex_refresh_min_input = codex_refresh_min_input.clone();
        let codex_refresh_max_input = codex_refresh_max_input.clone();
        let codex_account_jitter_max_input = codex_account_jitter_max_input.clone();
        let codex_weight_free_input = codex_weight_free_input.clone();
        let codex_weight_plus_input = codex_weight_plus_input.clone();
        let codex_weight_pro5x_input = codex_weight_pro5x_input.clone();
        let codex_weight_pro20x_input = codex_weight_pro20x_input.clone();
        let codex_session_affinity_enabled_input = codex_session_affinity_enabled_input.clone();
        let codex_session_affinity_max_entries_input =
            codex_session_affinity_max_entries_input.clone();
        let codex_session_affinity_ttl_seconds_input =
            codex_session_affinity_ttl_seconds_input.clone();
        let codex_fallback_affinity_enabled_input = codex_fallback_affinity_enabled_input.clone();
        let codex_fallback_affinity_ttl_seconds_input =
            codex_fallback_affinity_ttl_seconds_input.clone();
        let codex_fallback_affinity_prefix_bytes_input =
            codex_fallback_affinity_prefix_bytes_input.clone();
        let codex_fallback_affinity_min_body_bytes_input =
            codex_fallback_affinity_min_body_bytes_input.clone();
        let kiro_refresh_min_input = kiro_refresh_min_input.clone();
        let kiro_refresh_max_input = kiro_refresh_max_input.clone();
        let kiro_account_jitter_max_input = kiro_account_jitter_max_input.clone();
        let usage_flush_batch_size_input = usage_flush_batch_size_input.clone();
        let usage_flush_interval_input = usage_flush_interval_input.clone();
        let usage_flush_max_buffer_bytes_input = usage_flush_max_buffer_bytes_input.clone();
        let duckdb_usage_memory_limit_mib_input = duckdb_usage_memory_limit_mib_input.clone();
        let duckdb_usage_checkpoint_threshold_mib_input =
            duckdb_usage_checkpoint_threshold_mib_input.clone();
        let usage_analytics_retention_days_input = usage_analytics_retention_days_input.clone();
        let kiro_cctest_proxy_base_url_input = kiro_cctest_proxy_base_url_input.clone();
        let kiro_cctest_proxy_api_key_input = kiro_cctest_proxy_api_key_input.clone();
        let proxy_configs = proxy_configs.clone();
        let proxy_config_scope = proxy_config_scope.clone();
        let proxy_bindings = proxy_bindings.clone();
        let codex_proxy_binding_input = codex_proxy_binding_input.clone();
        let kiro_proxy_binding_input = kiro_proxy_binding_input.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let config = config.clone();
            let ttl_input = ttl_input.clone();
            let max_request_body_input = max_request_body_input.clone();
            let account_failure_retry_limit_input = account_failure_retry_limit_input.clone();
            let codex_client_version_input = codex_client_version_input.clone();
            let codex_refresh_min_input = codex_refresh_min_input.clone();
            let codex_refresh_max_input = codex_refresh_max_input.clone();
            let codex_account_jitter_max_input = codex_account_jitter_max_input.clone();
            let codex_weight_free_input = codex_weight_free_input.clone();
            let codex_weight_plus_input = codex_weight_plus_input.clone();
            let codex_weight_pro5x_input = codex_weight_pro5x_input.clone();
            let codex_weight_pro20x_input = codex_weight_pro20x_input.clone();
            let codex_session_affinity_enabled_input = codex_session_affinity_enabled_input.clone();
            let codex_session_affinity_max_entries_input =
                codex_session_affinity_max_entries_input.clone();
            let codex_session_affinity_ttl_seconds_input =
                codex_session_affinity_ttl_seconds_input.clone();
            let codex_fallback_affinity_enabled_input =
                codex_fallback_affinity_enabled_input.clone();
            let codex_fallback_affinity_ttl_seconds_input =
                codex_fallback_affinity_ttl_seconds_input.clone();
            let codex_fallback_affinity_prefix_bytes_input =
                codex_fallback_affinity_prefix_bytes_input.clone();
            let codex_fallback_affinity_min_body_bytes_input =
                codex_fallback_affinity_min_body_bytes_input.clone();
            let kiro_refresh_min_input = kiro_refresh_min_input.clone();
            let kiro_refresh_max_input = kiro_refresh_max_input.clone();
            let kiro_account_jitter_max_input = kiro_account_jitter_max_input.clone();
            let usage_flush_batch_size_input = usage_flush_batch_size_input.clone();
            let usage_flush_interval_input = usage_flush_interval_input.clone();
            let usage_flush_max_buffer_bytes_input = usage_flush_max_buffer_bytes_input.clone();
            let duckdb_usage_memory_limit_mib_input = duckdb_usage_memory_limit_mib_input.clone();
            let duckdb_usage_checkpoint_threshold_mib_input =
                duckdb_usage_checkpoint_threshold_mib_input.clone();
            let usage_analytics_retention_days_input = usage_analytics_retention_days_input.clone();
            let kiro_cctest_proxy_base_url_input = kiro_cctest_proxy_base_url_input.clone();
            let kiro_cctest_proxy_api_key_input = kiro_cctest_proxy_api_key_input.clone();
            let proxy_configs = proxy_configs.clone();
            let proxy_config_scope = proxy_config_scope.clone();
            let proxy_bindings = proxy_bindings.clone();
            let codex_proxy_binding_input = codex_proxy_binding_input.clone();
            let kiro_proxy_binding_input = kiro_proxy_binding_input.clone();
            let loading = loading.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let result = async {
                    let (cfg_result, proxy_configs_result, proxy_bindings_result) = futures::join!(
                        fetch_admin_llm_gateway_config(),
                        fetch_admin_llm_gateway_proxy_configs(),
                        fetch_admin_llm_gateway_proxy_bindings(),
                    );
                    let proxy_configs_resp = proxy_configs_result?;
                    Ok::<_, String>((
                        cfg_result?,
                        proxy_configs_resp.proxy_config_scope,
                        proxy_configs_resp.proxy_configs,
                        proxy_bindings_result?.bindings,
                    ))
                }
                .await;
                match result {
                    Ok((cfg, scope, proxy_config_items, proxy_binding_items)) => {
                        ttl_input.set(cfg.auth_cache_ttl_seconds.to_string());
                        max_request_body_input.set(cfg.max_request_body_bytes.to_string());
                        account_failure_retry_limit_input
                            .set(cfg.account_failure_retry_limit.to_string());
                        codex_client_version_input.set(cfg.codex_client_version.clone());
                        codex_refresh_min_input
                            .set(cfg.codex_status_refresh_min_interval_seconds.to_string());
                        codex_refresh_max_input
                            .set(cfg.codex_status_refresh_max_interval_seconds.to_string());
                        codex_account_jitter_max_input
                            .set(cfg.codex_status_account_jitter_max_seconds.to_string());
                        codex_weight_free_input.set(cfg.codex_weight_free.to_string());
                        codex_weight_plus_input.set(cfg.codex_weight_plus.to_string());
                        codex_weight_pro5x_input.set(cfg.codex_weight_pro5x.to_string());
                        codex_weight_pro20x_input.set(cfg.codex_weight_pro20x.to_string());
                        codex_session_affinity_enabled_input
                            .set(cfg.codex_session_affinity_enabled);
                        codex_session_affinity_max_entries_input
                            .set(cfg.codex_session_affinity_max_entries.to_string());
                        codex_session_affinity_ttl_seconds_input
                            .set(cfg.codex_session_affinity_ttl_seconds.to_string());
                        codex_fallback_affinity_enabled_input
                            .set(cfg.codex_fallback_affinity_enabled);
                        codex_fallback_affinity_ttl_seconds_input
                            .set(cfg.codex_fallback_affinity_ttl_seconds.to_string());
                        codex_fallback_affinity_prefix_bytes_input
                            .set(cfg.codex_fallback_affinity_prefix_bytes.to_string());
                        codex_fallback_affinity_min_body_bytes_input
                            .set(cfg.codex_fallback_affinity_min_body_bytes.to_string());
                        kiro_refresh_min_input
                            .set(cfg.kiro_status_refresh_min_interval_seconds.to_string());
                        kiro_refresh_max_input
                            .set(cfg.kiro_status_refresh_max_interval_seconds.to_string());
                        kiro_account_jitter_max_input
                            .set(cfg.kiro_status_account_jitter_max_seconds.to_string());
                        usage_flush_batch_size_input
                            .set(cfg.usage_event_flush_batch_size.to_string());
                        usage_flush_interval_input
                            .set(cfg.usage_event_flush_interval_seconds.to_string());
                        usage_flush_max_buffer_bytes_input
                            .set(cfg.usage_event_flush_max_buffer_bytes.to_string());
                        duckdb_usage_memory_limit_mib_input
                            .set(cfg.duckdb_usage_memory_limit_mib.to_string());
                        duckdb_usage_checkpoint_threshold_mib_input
                            .set(cfg.duckdb_usage_checkpoint_threshold_mib.to_string());
                        usage_analytics_retention_days_input
                            .set(cfg.usage_analytics_retention_days.to_string());
                        kiro_cctest_proxy_base_url_input
                            .set(cfg.kiro_cctest_proxy_base_url.clone().unwrap_or_default());
                        kiro_cctest_proxy_api_key_input
                            .set(cfg.kiro_cctest_proxy_api_key.clone().unwrap_or_default());
                        config.set(Some(cfg));
                        proxy_config_scope.set(scope);
                        let codex_bound = proxy_binding_items
                            .iter()
                            .find(|item| item.provider_type == "codex")
                            .and_then(|item| item.bound_proxy_config_id.clone())
                            .unwrap_or_default();
                        let kiro_bound = proxy_binding_items
                            .iter()
                            .find(|item| item.provider_type == "kiro")
                            .and_then(|item| item.bound_proxy_config_id.clone())
                            .unwrap_or_default();
                        proxy_configs.set(proxy_config_items);
                        proxy_bindings.set(proxy_binding_items);
                        codex_proxy_binding_input.set(codex_bound);
                        kiro_proxy_binding_input.set(kiro_bound);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_save_runtime_config = {
        let config = config.clone();
        let ttl_input = ttl_input.clone();
        let max_request_body_input = max_request_body_input.clone();
        let account_failure_retry_limit_input = account_failure_retry_limit_input.clone();
        let codex_client_version_input = codex_client_version_input.clone();
        let codex_refresh_min_input = codex_refresh_min_input.clone();
        let codex_refresh_max_input = codex_refresh_max_input.clone();
        let codex_account_jitter_max_input = codex_account_jitter_max_input.clone();
        let codex_weight_free_input = codex_weight_free_input.clone();
        let codex_weight_plus_input = codex_weight_plus_input.clone();
        let codex_weight_pro5x_input = codex_weight_pro5x_input.clone();
        let codex_weight_pro20x_input = codex_weight_pro20x_input.clone();
        let codex_session_affinity_enabled_input = codex_session_affinity_enabled_input.clone();
        let codex_session_affinity_max_entries_input =
            codex_session_affinity_max_entries_input.clone();
        let codex_session_affinity_ttl_seconds_input =
            codex_session_affinity_ttl_seconds_input.clone();
        let codex_fallback_affinity_enabled_input = codex_fallback_affinity_enabled_input.clone();
        let codex_fallback_affinity_ttl_seconds_input =
            codex_fallback_affinity_ttl_seconds_input.clone();
        let codex_fallback_affinity_prefix_bytes_input =
            codex_fallback_affinity_prefix_bytes_input.clone();
        let codex_fallback_affinity_min_body_bytes_input =
            codex_fallback_affinity_min_body_bytes_input.clone();
        let kiro_refresh_min_input = kiro_refresh_min_input.clone();
        let kiro_refresh_max_input = kiro_refresh_max_input.clone();
        let kiro_account_jitter_max_input = kiro_account_jitter_max_input.clone();
        let usage_flush_batch_size_input = usage_flush_batch_size_input.clone();
        let usage_flush_interval_input = usage_flush_interval_input.clone();
        let usage_flush_max_buffer_bytes_input = usage_flush_max_buffer_bytes_input.clone();
        let duckdb_usage_memory_limit_mib_input = duckdb_usage_memory_limit_mib_input.clone();
        let duckdb_usage_checkpoint_threshold_mib_input =
            duckdb_usage_checkpoint_threshold_mib_input.clone();
        let usage_analytics_retention_days_input = usage_analytics_retention_days_input.clone();
        let kiro_cctest_proxy_base_url_input = kiro_cctest_proxy_base_url_input.clone();
        let kiro_cctest_proxy_api_key_input = kiro_cctest_proxy_api_key_input.clone();
        let saving_runtime_config = saving_runtime_config.clone();
        let load_error = load_error.clone();
        let on_reload = on_reload.clone();
        Callback::from(move |_| {
            let config = config.clone();
            let ttl = (*ttl_input).trim().parse::<u64>();
            let max_request_body_bytes = (*max_request_body_input).trim().parse::<u64>();
            let account_failure_retry_limit =
                (*account_failure_retry_limit_input).trim().parse::<u64>();
            let codex_client_version = (*codex_client_version_input).trim().to_string();
            let codex_status_refresh_min_interval_seconds =
                (*codex_refresh_min_input).trim().parse::<u64>();
            let codex_status_refresh_max_interval_seconds =
                (*codex_refresh_max_input).trim().parse::<u64>();
            let codex_status_account_jitter_max_seconds =
                (*codex_account_jitter_max_input).trim().parse::<u64>();
            let codex_weight_free = (*codex_weight_free_input).trim().parse::<u64>();
            let codex_weight_plus = (*codex_weight_plus_input).trim().parse::<u64>();
            let codex_weight_pro5x = (*codex_weight_pro5x_input).trim().parse::<u64>();
            let codex_weight_pro20x = (*codex_weight_pro20x_input).trim().parse::<u64>();
            let codex_session_affinity_enabled = *codex_session_affinity_enabled_input;
            let codex_session_affinity_max_entries = (*codex_session_affinity_max_entries_input)
                .trim()
                .parse::<u64>();
            let codex_session_affinity_ttl_seconds = (*codex_session_affinity_ttl_seconds_input)
                .trim()
                .parse::<u64>();
            let codex_fallback_affinity_enabled = *codex_fallback_affinity_enabled_input;
            let codex_fallback_affinity_ttl_seconds = (*codex_fallback_affinity_ttl_seconds_input)
                .trim()
                .parse::<u64>();
            let codex_fallback_affinity_prefix_bytes =
                (*codex_fallback_affinity_prefix_bytes_input)
                    .trim()
                    .parse::<u64>();
            let codex_fallback_affinity_min_body_bytes =
                (*codex_fallback_affinity_min_body_bytes_input)
                    .trim()
                    .parse::<u64>();
            let kiro_status_refresh_min_interval_seconds =
                (*kiro_refresh_min_input).trim().parse::<u64>();
            let kiro_status_refresh_max_interval_seconds =
                (*kiro_refresh_max_input).trim().parse::<u64>();
            let kiro_status_account_jitter_max_seconds =
                (*kiro_account_jitter_max_input).trim().parse::<u64>();
            let usage_event_flush_batch_size =
                (*usage_flush_batch_size_input).trim().parse::<u64>();
            let usage_event_flush_interval_seconds =
                (*usage_flush_interval_input).trim().parse::<u64>();
            let usage_event_flush_max_buffer_bytes =
                (*usage_flush_max_buffer_bytes_input).trim().parse::<u64>();
            let duckdb_usage_memory_limit_mib =
                (*duckdb_usage_memory_limit_mib_input).trim().parse::<u64>();
            let duckdb_usage_checkpoint_threshold_mib =
                (*duckdb_usage_checkpoint_threshold_mib_input)
                    .trim()
                    .parse::<u64>();
            let usage_analytics_retention_days = (*usage_analytics_retention_days_input)
                .trim()
                .parse::<u64>();
            let kiro_cctest_proxy_base_url =
                normalize_optional_form_string(kiro_cctest_proxy_base_url_input.as_str());
            let kiro_cctest_proxy_api_key =
                normalize_optional_form_string(kiro_cctest_proxy_api_key_input.as_str());
            let saving_runtime_config = saving_runtime_config.clone();
            let load_error = load_error.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(ttl) = ttl else {
                    load_error.set(Some("TTL 必须是正整数".to_string()));
                    return;
                };
                let Ok(max_request_body_bytes) = max_request_body_bytes else {
                    load_error.set(Some("请求体上限必须是正整数".to_string()));
                    return;
                };
                let Ok(account_failure_retry_limit) = account_failure_retry_limit else {
                    load_error.set(Some("账号失败重试次数必须是非负整数".to_string()));
                    return;
                };
                if codex_client_version.is_empty() {
                    load_error.set(Some("Codex client version 不能为空".to_string()));
                    return;
                }
                let Ok(codex_status_refresh_min_interval_seconds) =
                    codex_status_refresh_min_interval_seconds
                else {
                    load_error.set(Some("Codex 最小轮询间隔必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_status_refresh_max_interval_seconds) =
                    codex_status_refresh_max_interval_seconds
                else {
                    load_error.set(Some("Codex 最大轮询间隔必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_status_account_jitter_max_seconds) =
                    codex_status_account_jitter_max_seconds
                else {
                    load_error.set(Some("Codex 单账号抖动上限必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_weight_free) = codex_weight_free else {
                    load_error.set(Some("Codex free 权重必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_weight_plus) = codex_weight_plus else {
                    load_error.set(Some("Codex plus 权重必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_weight_pro5x) = codex_weight_pro5x else {
                    load_error.set(Some("Codex pro5x 权重必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_weight_pro20x) = codex_weight_pro20x else {
                    load_error.set(Some("Codex pro20x 权重必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_session_affinity_max_entries) = codex_session_affinity_max_entries
                else {
                    load_error.set(Some("Codex 亲和 LRU 容量必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_session_affinity_ttl_seconds) = codex_session_affinity_ttl_seconds
                else {
                    load_error.set(Some("Codex 显式 session TTL 必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_fallback_affinity_ttl_seconds) = codex_fallback_affinity_ttl_seconds
                else {
                    load_error.set(Some("Codex 前缀兜底 TTL 必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_fallback_affinity_prefix_bytes) = codex_fallback_affinity_prefix_bytes
                else {
                    load_error.set(Some("Codex 前缀取样字节必须是非负整数".to_string()));
                    return;
                };
                let Ok(codex_fallback_affinity_min_body_bytes) =
                    codex_fallback_affinity_min_body_bytes
                else {
                    load_error.set(Some("Codex 兜底最小请求体必须是非负整数".to_string()));
                    return;
                };
                let Ok(kiro_status_refresh_min_interval_seconds) =
                    kiro_status_refresh_min_interval_seconds
                else {
                    load_error.set(Some("Kiro 最小轮询间隔必须是非负整数".to_string()));
                    return;
                };
                let Ok(kiro_status_refresh_max_interval_seconds) =
                    kiro_status_refresh_max_interval_seconds
                else {
                    load_error.set(Some("Kiro 最大轮询间隔必须是非负整数".to_string()));
                    return;
                };
                let Ok(kiro_status_account_jitter_max_seconds) =
                    kiro_status_account_jitter_max_seconds
                else {
                    load_error.set(Some("Kiro 单账号抖动上限必须是非负整数".to_string()));
                    return;
                };
                let Ok(usage_event_flush_batch_size) = usage_event_flush_batch_size else {
                    load_error.set(Some("usage flush 批大小必须是非负整数".to_string()));
                    return;
                };
                let Ok(usage_event_flush_interval_seconds) = usage_event_flush_interval_seconds
                else {
                    load_error.set(Some("usage flush 间隔必须是非负整数".to_string()));
                    return;
                };
                let Ok(usage_event_flush_max_buffer_bytes) = usage_event_flush_max_buffer_bytes
                else {
                    load_error.set(Some("usage flush 缓冲上限必须是非负整数".to_string()));
                    return;
                };
                let Ok(duckdb_usage_memory_limit_mib) = duckdb_usage_memory_limit_mib else {
                    load_error.set(Some("DuckDB memory_limit 必须是正整数 MiB".to_string()));
                    return;
                };
                let Ok(duckdb_usage_checkpoint_threshold_mib) =
                    duckdb_usage_checkpoint_threshold_mib
                else {
                    load_error
                        .set(Some("DuckDB checkpoint threshold 必须是正整数 MiB".to_string()));
                    return;
                };
                let Ok(usage_analytics_retention_days) = usage_analytics_retention_days else {
                    load_error.set(Some("Usage analytics retention 必须是正整数天数".to_string()));
                    return;
                };
                let runtime_config = LlmGatewayRuntimeConfig {
                    auth_cache_ttl_seconds: ttl,
                    max_request_body_bytes,
                    account_failure_retry_limit,
                    codex_client_version,
                    codex_status_refresh_min_interval_seconds,
                    codex_status_refresh_max_interval_seconds,
                    codex_status_account_jitter_max_seconds,
                    codex_weight_free,
                    codex_weight_plus,
                    codex_weight_pro5x,
                    codex_weight_pro20x,
                    codex_session_affinity_enabled,
                    codex_session_affinity_max_entries,
                    codex_session_affinity_ttl_seconds,
                    codex_fallback_affinity_enabled,
                    codex_fallback_affinity_ttl_seconds,
                    codex_fallback_affinity_prefix_bytes,
                    codex_fallback_affinity_min_body_bytes,
                    kiro_status_refresh_min_interval_seconds,
                    kiro_status_refresh_max_interval_seconds,
                    kiro_status_account_jitter_max_seconds,
                    usage_event_flush_batch_size,
                    usage_event_flush_interval_seconds,
                    usage_event_flush_max_buffer_bytes,
                    duckdb_usage_memory_limit_mib,
                    duckdb_usage_checkpoint_threshold_mib,
                    usage_analytics_retention_days,
                    usage_journal_enabled: config
                        .as_ref()
                        .map(|current| current.usage_journal_enabled)
                        .unwrap_or(true),
                    usage_journal_max_file_bytes: config
                        .as_ref()
                        .map(|current| current.usage_journal_max_file_bytes)
                        .unwrap_or(64 * 1024 * 1024),
                    usage_journal_max_file_age_ms: config
                        .as_ref()
                        .map(|current| current.usage_journal_max_file_age_ms)
                        .unwrap_or(300_000),
                    usage_journal_max_files: config
                        .as_ref()
                        .map(|current| current.usage_journal_max_files)
                        .unwrap_or(128),
                    usage_journal_block_target_uncompressed_bytes: config
                        .as_ref()
                        .map(|current| current.usage_journal_block_target_uncompressed_bytes)
                        .unwrap_or(1024 * 1024),
                    usage_journal_block_max_events: config
                        .as_ref()
                        .map(|current| current.usage_journal_block_max_events)
                        .unwrap_or(1024),
                    usage_journal_fsync_interval_ms: config
                        .as_ref()
                        .map(|current| current.usage_journal_fsync_interval_ms)
                        .unwrap_or(250),
                    usage_journal_zstd_level: config
                        .as_ref()
                        .map(|current| current.usage_journal_zstd_level)
                        .unwrap_or(3),
                    usage_journal_consumer_lease_ms: config
                        .as_ref()
                        .map(|current| current.usage_journal_consumer_lease_ms)
                        .unwrap_or(300_000),
                    usage_journal_delete_bad_files: config
                        .as_ref()
                        .map(|current| current.usage_journal_delete_bad_files)
                        .unwrap_or(false),
                    usage_query_bind_addr: config
                        .as_ref()
                        .map(|current| current.usage_query_bind_addr.clone())
                        .unwrap_or_else(|| "127.0.0.1:19081".to_string()),
                    usage_query_base_url: config
                        .as_ref()
                        .map(|current| current.usage_query_base_url.clone())
                        .unwrap_or_else(|| "http://127.0.0.1:19081".to_string()),
                    kiro_cache_kmodels_json: config
                        .as_ref()
                        .map(|current| current.kiro_cache_kmodels_json.clone())
                        .unwrap_or_default(),
                    kiro_billable_model_multipliers_json: config
                        .as_ref()
                        .map(|current| current.kiro_billable_model_multipliers_json.clone())
                        .unwrap_or_else(|| "{}".to_string()),
                    kiro_cache_policy_json: config
                        .as_ref()
                        .map(|current| current.kiro_cache_policy_json.clone())
                        .unwrap_or_default(),
                    kiro_context_usage_min_request_tokens: config
                        .as_ref()
                        .map(|current| current.kiro_context_usage_min_request_tokens)
                        .unwrap_or(15_000),
                    kiro_compact_trigger_tokens: config
                        .as_ref()
                        .map(|current| current.kiro_compact_trigger_tokens)
                        .unwrap_or(780_000),
                    kiro_prefix_cache_mode: config
                        .as_ref()
                        .map(|current| current.kiro_prefix_cache_mode.clone())
                        .unwrap_or_else(|| "prefix_tree".to_string()),
                    kiro_prefix_cache_max_tokens: config
                        .as_ref()
                        .map(|current| current.kiro_prefix_cache_max_tokens)
                        .unwrap_or(4_000_000),
                    kiro_prefix_cache_entry_ttl_seconds: config
                        .as_ref()
                        .map(|current| current.kiro_prefix_cache_entry_ttl_seconds)
                        .unwrap_or(21_600),
                    kiro_conversation_anchor_max_entries: config
                        .as_ref()
                        .map(|current| current.kiro_conversation_anchor_max_entries)
                        .unwrap_or(20_000),
                    kiro_conversation_anchor_ttl_seconds: config
                        .as_ref()
                        .map(|current| current.kiro_conversation_anchor_ttl_seconds)
                        .unwrap_or(86_400),
                    kiro_cache_snapshot_enabled: config
                        .as_ref()
                        .map(|current| current.kiro_cache_snapshot_enabled)
                        .unwrap_or(false),
                    kiro_cache_snapshot_interval_seconds: config
                        .as_ref()
                        .map(|current| current.kiro_cache_snapshot_interval_seconds)
                        .unwrap_or(300),
                    kiro_cache_snapshot_ttl_seconds: config
                        .as_ref()
                        .map(|current| current.kiro_cache_snapshot_ttl_seconds)
                        .unwrap_or(86_400),
                    kiro_cache_snapshot_max_tokens: config
                        .as_ref()
                        .map(|current| current.kiro_cache_snapshot_max_tokens)
                        .unwrap_or(0),
                    kiro_cache_snapshot_max_anchor_entries: config
                        .as_ref()
                        .map(|current| current.kiro_cache_snapshot_max_anchor_entries)
                        .unwrap_or(0),
                    kiro_cctest_proxy_base_url,
                    kiro_cctest_proxy_api_key,
                };
                saving_runtime_config.set(true);
                match update_admin_llm_gateway_config(&runtime_config).await {
                    Ok(_) => {
                        load_error.set(None);
                        on_reload.emit(());
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                saving_runtime_config.set(false);
            });
        })
    };

    let on_create_proxy_config = {
        let create_proxy_name = create_proxy_name.clone();
        let create_proxy_url = create_proxy_url.clone();
        let create_proxy_username = create_proxy_username.clone();
        let create_proxy_password = create_proxy_password.clone();
        let creating_proxy = creating_proxy.clone();
        let proxy_config_scope = proxy_config_scope.clone();
        let load_error = load_error.clone();
        let flash = flash.clone();
        let on_reload = on_reload.clone();
        Callback::from(move |_| {
            if !proxy_config_scope.can_edit_slot_metadata {
                flash.emit(("只有 core 节点可以创建代理槽位".to_string(), true));
                return;
            }
            let proxy_url = proxy_url_after_socks5h_confirmation((*create_proxy_url).as_str());
            if proxy_url != (*create_proxy_url).trim() {
                create_proxy_url.set(proxy_url.clone());
            }
            let input = CreateAdminUpstreamProxyConfigInput {
                name: (*create_proxy_name).trim().to_string(),
                proxy_url,
                proxy_username: {
                    let value = (*create_proxy_username).trim().to_string();
                    if value.is_empty() {
                        None
                    } else {
                        Some(value)
                    }
                },
                proxy_password: {
                    let value = (*create_proxy_password).trim().to_string();
                    if value.is_empty() {
                        None
                    } else {
                        Some(value)
                    }
                },
            };
            let create_proxy_name = create_proxy_name.clone();
            let create_proxy_username = create_proxy_username.clone();
            let create_proxy_password = create_proxy_password.clone();
            let creating_proxy = creating_proxy.clone();
            let load_error = load_error.clone();
            let flash = flash.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                creating_proxy.set(true);
                match create_admin_llm_gateway_proxy_config(&input).await {
                    Ok(_) => {
                        create_proxy_name.set(String::new());
                        create_proxy_username.set(String::new());
                        create_proxy_password.set(String::new());
                        load_error.set(None);
                        flash.emit(("已创建代理配置".to_string(), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((format!("创建代理配置失败\n{err}"), true));
                    },
                }
                creating_proxy.set(false);
            });
        })
    };

    let on_save_proxy_binding = {
        let proxy_bindings = proxy_bindings.clone();
        let codex_proxy_binding_input = codex_proxy_binding_input.clone();
        let kiro_proxy_binding_input = kiro_proxy_binding_input.clone();
        let saving_proxy_binding_provider = saving_proxy_binding_provider.clone();
        let load_error = load_error.clone();
        let flash = flash.clone();
        Callback::from(move |provider_type: String| {
            let proxy_config_id = match provider_type.as_str() {
                "codex" => (*codex_proxy_binding_input).clone(),
                "kiro" => (*kiro_proxy_binding_input).clone(),
                _ => String::new(),
            };
            let proxy_bindings = proxy_bindings.clone();
            let codex_proxy_binding_input = codex_proxy_binding_input.clone();
            let kiro_proxy_binding_input = kiro_proxy_binding_input.clone();
            let saving_proxy_binding_provider = saving_proxy_binding_provider.clone();
            let load_error = load_error.clone();
            let flash = flash.clone();
            wasm_bindgen_futures::spawn_local(async move {
                saving_proxy_binding_provider.set(Some(provider_type.clone()));
                match update_admin_llm_gateway_proxy_binding(
                    &provider_type,
                    if proxy_config_id.trim().is_empty() {
                        None
                    } else {
                        Some(proxy_config_id.trim())
                    },
                )
                .await
                {
                    Ok(updated) => {
                        let mut items = (*proxy_bindings).clone();
                        if let Some(existing) = items
                            .iter_mut()
                            .find(|item| item.provider_type == updated.provider_type)
                        {
                            *existing = updated.clone();
                        } else {
                            items.push(updated.clone());
                            items.sort_by(|left, right| {
                                left.provider_type.cmp(&right.provider_type)
                            });
                        }
                        proxy_bindings.set(items);
                        let bound_value = updated.bound_proxy_config_id.clone().unwrap_or_default();
                        match provider_type.as_str() {
                            "codex" => codex_proxy_binding_input.set(bound_value),
                            "kiro" => kiro_proxy_binding_input.set(bound_value),
                            _ => {},
                        }
                        load_error.set(None);
                        flash.emit((
                            format!("已更新 {} 代理绑定", provider_type.to_uppercase()),
                            false,
                        ));
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((
                            format!("保存 {} 代理绑定失败\n{err}", provider_type.to_uppercase()),
                            true,
                        ));
                    },
                }
                saving_proxy_binding_provider.set(None);
            });
        })
    };

    let on_import_legacy_kiro_proxy = {
        let migrating_legacy_kiro_proxy = migrating_legacy_kiro_proxy.clone();
        let load_error = load_error.clone();
        let flash = flash.clone();
        let on_reload = on_reload.clone();
        Callback::from(move |_| {
            let migrating_legacy_kiro_proxy = migrating_legacy_kiro_proxy.clone();
            let load_error = load_error.clone();
            let flash = flash.clone();
            let on_reload = on_reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                migrating_legacy_kiro_proxy.set(true);
                match import_admin_legacy_kiro_proxy_configs().await {
                    Ok(_) => {
                        load_error.set(None);
                        flash.emit(("已导入 legacy Kiro 代理配置".to_string(), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        load_error.set(Some(err.clone()));
                        flash.emit((format!("导入 legacy Kiro 代理配置失败\n{err}"), true));
                    },
                }
                migrating_legacy_kiro_proxy.set(false);
            });
        })
    };

    // ── Proxy config: filter ──
    let proxy_query_lower = (*proxy_config_active_query).trim().to_lowercase();
    let proxy_configs_filtered: Vec<&AdminUpstreamProxyConfigView> = proxy_configs
        .iter()
        .filter(|pc| {
            proxy_query_lower.is_empty()
                || pc.name.to_lowercase().contains(&proxy_query_lower)
                || pc.proxy_url.to_lowercase().contains(&proxy_query_lower)
                || pc
                    .proxy_username
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&proxy_query_lower)
                || pc.id.to_lowercase().contains(&proxy_query_lower)
        })
        .filter(|pc| !*proxy_config_show_active_only || pc.status.as_str() != "disabled")
        .collect();
    let on_proxy_search_submit = {
        let proxy_config_search = proxy_config_search.clone();
        let proxy_config_active_query = proxy_config_active_query.clone();
        Callback::from(move |_: ()| {
            proxy_config_active_query.set((*proxy_config_search).clone());
        })
    };
    let on_proxy_search_input = {
        let proxy_config_search = proxy_config_search.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<HtmlInputElement>() {
                proxy_config_search.set(target.value());
            }
        })
    };
    let on_proxy_search_keydown = {
        let on_proxy_search_submit = on_proxy_search_submit.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_proxy_search_submit.emit(());
            }
        })
    };
    let on_proxy_search_clear = {
        let proxy_config_search = proxy_config_search.clone();
        let proxy_config_active_query = proxy_config_active_query.clone();
        Callback::from(move |_: MouseEvent| {
            proxy_config_search.set(String::new());
            proxy_config_active_query.set(String::new());
        })
    };
    let proxy_scope_view = (*proxy_config_scope).clone();
    let can_create_proxy_config = proxy_scope_view.can_edit_slot_metadata;
    let proxy_scope_summary = if proxy_scope_view.is_core {
        format!(
            "当前节点 {} 使用 core 代理槽位，可创建、删除和重命名槽位。",
            proxy_scope_view.node_id
        )
    } else {
        format!(
            "当前节点 {} 继承 core 代理槽位，只能修改本机代理地址、凭据和状态。",
            proxy_scope_view.node_id
        )
    };

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "LLM Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Settings" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminLlmGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
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
                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                            <div>
                                <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Runtime Config" }</h2>
                                <p class={classes!("mt-2", "mb-0", "text-sm", "text-[var(--muted)]")}>
                                    { "This page owns gateway-wide runtime defaults and llm usage maintenance cadence. Kiro cache simulation, prefix-tree capacity, anchor settings, and per-account scheduler overrides are managed from the Kiro Gateway page." }
                                </p>
                            </div>
                            <Link<Route> to={Route::AdminKiroGateway} classes={classes!("linkbtn")}>
                                { "Open Kiro Gateway" }
                            </Link<Route>>
                        </div>
                        <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-2", "xl:grid-cols-3")}>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "auth_cache_ttl_seconds" }</span>
                                <input
                                    type="number"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*ttl_input).clone()}
                                    oninput={{
                                        let ttl_input = ttl_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                ttl_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "max_request_body_bytes" }</span>
                                <input
                                    type="number"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*max_request_body_input).clone()}
                                    oninput={{
                                        let max_request_body_input = max_request_body_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                max_request_body_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "account_failure_retry_limit" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*account_failure_retry_limit_input).clone()}
                                    oninput={{
                                        let account_failure_retry_limit_input = account_failure_retry_limit_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                account_failure_retry_limit_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <h3 class={classes!("md:col-span-2", "xl:col-span-3", "m-0", "mt-2", "text-xs", "font-semibold", "uppercase", "tracking-wider", "text-[var(--muted)]")}>{ "Codex" }</h3>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_client_version" }</span>
                                <input
                                    type="text"
                                    spellcheck="false"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono")}
                                    value={(*codex_client_version_input).clone()}
                                    oninput={{
                                        let codex_client_version_input = codex_client_version_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_client_version_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_status_refresh_min_interval_seconds" }</span>
                                <input
                                    type="number"
                                    min="240"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_refresh_min_input).clone()}
                                    oninput={{
                                        let codex_refresh_min_input = codex_refresh_min_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_refresh_min_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_status_refresh_max_interval_seconds" }</span>
                                <input
                                    type="number"
                                    min="240"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_refresh_max_input).clone()}
                                    oninput={{
                                        let codex_refresh_max_input = codex_refresh_max_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_refresh_max_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_status_account_jitter_max_seconds" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_account_jitter_max_input).clone()}
                                    oninput={{
                                        let codex_account_jitter_max_input = codex_account_jitter_max_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_account_jitter_max_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_weight_free" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_weight_free_input).clone()}
                                    oninput={{
                                        let codex_weight_free_input = codex_weight_free_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_weight_free_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_weight_plus" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_weight_plus_input).clone()}
                                    oninput={{
                                        let codex_weight_plus_input = codex_weight_plus_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_weight_plus_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_weight_pro5x" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_weight_pro5x_input).clone()}
                                    oninput={{
                                        let codex_weight_pro5x_input = codex_weight_pro5x_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_weight_pro5x_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_weight_pro20x" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_weight_pro20x_input).clone()}
                                    oninput={{
                                        let codex_weight_pro20x_input = codex_weight_pro20x_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_weight_pro20x_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <h3 class={classes!("md:col-span-2", "xl:col-span-3", "m-0", "mt-2", "text-xs", "font-semibold", "uppercase", "tracking-wider", "text-[var(--muted)]")}>{ "Codex Affinity" }</h3>
                            <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                                <input
                                    type="checkbox" class={classes!("min-h-0", "w-auto")}
                                    checked={*codex_session_affinity_enabled_input}
                                    onchange={{
                                        let codex_session_affinity_enabled_input = codex_session_affinity_enabled_input.clone();
                                        Callback::from(move |event: Event| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_session_affinity_enabled_input.set(target.checked());
                                            }
                                        })
                                    }}
                                />
                                <span>{ "启用 Codex 账号亲和" }</span>
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_session_affinity_max_entries" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_session_affinity_max_entries_input).clone()}
                                    oninput={{
                                        let codex_session_affinity_max_entries_input = codex_session_affinity_max_entries_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_session_affinity_max_entries_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_session_affinity_ttl_seconds" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_session_affinity_ttl_seconds_input).clone()}
                                    oninput={{
                                        let codex_session_affinity_ttl_seconds_input = codex_session_affinity_ttl_seconds_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_session_affinity_ttl_seconds_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("flex", "items-center", "gap-2", "text-sm")}>
                                <input
                                    type="checkbox" class={classes!("min-h-0", "w-auto")}
                                    checked={*codex_fallback_affinity_enabled_input}
                                    onchange={{
                                        let codex_fallback_affinity_enabled_input = codex_fallback_affinity_enabled_input.clone();
                                        Callback::from(move |event: Event| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_fallback_affinity_enabled_input.set(target.checked());
                                            }
                                        })
                                    }}
                                />
                                <span>{ "无 session 时启用请求体前缀兜底" }</span>
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_fallback_affinity_ttl_seconds" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_fallback_affinity_ttl_seconds_input).clone()}
                                    oninput={{
                                        let codex_fallback_affinity_ttl_seconds_input = codex_fallback_affinity_ttl_seconds_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_fallback_affinity_ttl_seconds_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_fallback_affinity_prefix_bytes" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_fallback_affinity_prefix_bytes_input).clone()}
                                    oninput={{
                                        let codex_fallback_affinity_prefix_bytes_input = codex_fallback_affinity_prefix_bytes_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_fallback_affinity_prefix_bytes_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "codex_fallback_affinity_min_body_bytes" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*codex_fallback_affinity_min_body_bytes_input).clone()}
                                    oninput={{
                                        let codex_fallback_affinity_min_body_bytes_input = codex_fallback_affinity_min_body_bytes_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                codex_fallback_affinity_min_body_bytes_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <h3 class={classes!("md:col-span-2", "xl:col-span-3", "m-0", "mt-2", "text-xs", "font-semibold", "uppercase", "tracking-wider", "text-[var(--muted)]")}>{ "Kiro" }</h3>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "kiro_status_refresh_min_interval_seconds" }</span>
                                <input
                                    type="number"
                                    min="240"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*kiro_refresh_min_input).clone()}
                                    oninput={{
                                        let kiro_refresh_min_input = kiro_refresh_min_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                kiro_refresh_min_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "kiro_status_refresh_max_interval_seconds" }</span>
                                <input
                                    type="number"
                                    min="240"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*kiro_refresh_max_input).clone()}
                                    oninput={{
                                        let kiro_refresh_max_input = kiro_refresh_max_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                kiro_refresh_max_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "kiro_status_account_jitter_max_seconds" }</span>
                                <input
                                    type="number"
                                    min="0"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*kiro_account_jitter_max_input).clone()}
                                    oninput={{
                                        let kiro_account_jitter_max_input = kiro_account_jitter_max_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                kiro_account_jitter_max_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm", "md:col-span-2")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "kiro_cctest_proxy_base_url" }</span>
                                <input
                                    type="text"
                                    placeholder="https://example.com"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*kiro_cctest_proxy_base_url_input).clone()}
                                    oninput={{
                                        let kiro_cctest_proxy_base_url_input =
                                            kiro_cctest_proxy_base_url_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                kiro_cctest_proxy_base_url_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "kiro_cctest_proxy_api_key" }</span>
                                <input
                                    type="password"
                                    autocomplete="off"
                                    placeholder="留空表示未配置"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*kiro_cctest_proxy_api_key_input).clone()}
                                    oninput={{
                                        let kiro_cctest_proxy_api_key_input =
                                            kiro_cctest_proxy_api_key_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                kiro_cctest_proxy_api_key_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <h3 class={classes!("md:col-span-2", "xl:col-span-3", "m-0", "mt-2", "text-xs", "font-semibold", "uppercase", "tracking-wider", "text-[var(--muted)]")}>{ "Usage / DuckDB" }</h3>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "usage_event_flush_batch_size" }</span>
                                <input
                                    type="number"
                                    min="1"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*usage_flush_batch_size_input).clone()}
                                    oninput={{
                                        let usage_flush_batch_size_input = usage_flush_batch_size_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                usage_flush_batch_size_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "usage_event_flush_interval_seconds" }</span>
                                <input
                                    type="number"
                                    min="1"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*usage_flush_interval_input).clone()}
                                    oninput={{
                                        let usage_flush_interval_input = usage_flush_interval_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                usage_flush_interval_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "usage_event_flush_max_buffer_bytes" }</span>
                                <input
                                    type="number"
                                    min="1"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*usage_flush_max_buffer_bytes_input).clone()}
                                    oninput={{
                                        let usage_flush_max_buffer_bytes_input = usage_flush_max_buffer_bytes_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                usage_flush_max_buffer_bytes_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "duckdb_usage_memory_limit_mib" }</span>
                                <input
                                    type="number"
                                    min="512"
                                    max="2048"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*duckdb_usage_memory_limit_mib_input).clone()}
                                    oninput={{
                                        let duckdb_usage_memory_limit_mib_input = duckdb_usage_memory_limit_mib_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                duckdb_usage_memory_limit_mib_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "duckdb_usage_checkpoint_threshold_mib" }</span>
                                <input
                                    type="number"
                                    min="16"
                                    max="256"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*duckdb_usage_checkpoint_threshold_mib_input).clone()}
                                    oninput={{
                                        let duckdb_usage_checkpoint_threshold_mib_input = duckdb_usage_checkpoint_threshold_mib_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                duckdb_usage_checkpoint_threshold_mib_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "usage_analytics_retention_days" }</span>
                                <input
                                    type="number"
                                    min="1"
                                    max="365"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*usage_analytics_retention_days_input).clone()}
                                    oninput={{
                                        let usage_analytics_retention_days_input = usage_analytics_retention_days_input.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                usage_analytics_retention_days_input.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <details class={classes!("rounded-lg", "border", "border-dashed", "border-[var(--border)]", "bg-[var(--bg)]", "px-3", "py-2", "text-xs", "text-[var(--muted)]", "md:col-span-2", "xl:col-span-3")}>
                                <summary class={classes!("cursor-pointer", "font-semibold", "select-none")}>{ "配置说明" }</summary>
                                <div class={classes!("mt-2")}>
                                <p class={classes!("m-0")}>
                                    { format!("默认 Codex models catalog 版本：{}。不带 client_version 的 `/v1/models` 请求会回落到这里。", DEFAULT_LLM_GATEWAY_CODEX_CLIENT_VERSION) }
                                </p>
                                <p class={classes!("m-0", "mt-1")}>
                                    { "默认轮询窗口：Codex / Kiro 都是 240-300 秒；每个账号请求之间插入 0-10 秒随机抖动。" }
                                </p>
                                <p class={classes!("m-0", "mt-1")}>
                                    { "Codex 自动选号会按 bottleneck remaining * weight 比较；默认倍率是 free=1, plus=10, pro5x=50, pro20x=200。" }
                                </p>
                                <p class={classes!("m-0", "mt-1")}>
                                    { "默认 usage flush：256 条、15 秒、8 MiB；DuckDB writer 默认 memory_limit=1024 MiB、checkpoint_threshold=16 MiB。" }
                                </p>
                                <p class={classes!("m-0", "mt-1")}>
                                    { "llm usage 表现在和其他表共用 /admin 里的 Storage Maintenance 配置：scan interval、fragment threshold、prune 窗口和 worker 数都只有一套。" }
                                </p>
                                </div>
                            </details>
                            <div class={classes!("flex", "items-end", "md:col-span-2", "xl:col-span-3")}>
                                <button class={classes!("primary", "w-full", "md:w-auto")} onclick={on_save_runtime_config} disabled={*saving_runtime_config}>
                                    { if *saving_runtime_config { "保存中..." } else { "保存" } }
                                </button>
                            </div>
                        </div>
                        if let Some(cfg) = (*config).clone() {
                            <div class={classes!("mt-3", "space-y-1", "text-xs", "text-[var(--muted)]")}>
                                <p class={classes!("m-0")}>
                                    { format!("当前 TTL：{} 秒", cfg.auth_cache_ttl_seconds) }
                                </p>
                                <p class={classes!("m-0")}>
                                    { format!("当前请求体上限：{} bytes", format_number_u64(cfg.max_request_body_bytes)) }
                                </p>
                                <p class={classes!("m-0")}>
                                    { format!("当前账号失败重试次数：{}", cfg.account_failure_retry_limit) }
                                </p>
                                <p class={classes!("m-0")}>
                                    { format!("当前 Codex client version：{}", cfg.codex_client_version) }
                                </p>
                                <p class={classes!("m-0")}>
                                    { format!(
                                        "当前 Codex 轮询窗口：{}-{} 秒，单账号抖动上限：{} 秒",
                                        cfg.codex_status_refresh_min_interval_seconds,
                                        cfg.codex_status_refresh_max_interval_seconds,
                                        cfg.codex_status_account_jitter_max_seconds
                                    ) }
                                </p>
                                <p class={classes!("m-0")}>
                                    { format!(
                                        "当前 Kiro 轮询窗口：{}-{} 秒，单账号抖动上限：{} 秒",
                                        cfg.kiro_status_refresh_min_interval_seconds,
                                        cfg.kiro_status_refresh_max_interval_seconds,
                                        cfg.kiro_status_account_jitter_max_seconds
                                    ) }
                                </p>
                                <p class={classes!("m-0")}>
                                    { format!(
                                        "当前 cctest signature proxy：{}，API key：{}",
                                        cfg.kiro_cctest_proxy_base_url.clone().unwrap_or_else(|| "-".to_string()),
                                        if cfg.kiro_cctest_proxy_api_key.as_ref().is_some_and(|value| !value.is_empty()) { "已配置" } else { "未配置" }
                                    ) }
                                </p>
                                <p class={classes!("m-0")}>
                                    { format!(
                                        "当前 usage flush：{} 条 / {} 秒 / {} bytes；DuckDB：{} MiB / {} MiB；保留最近 {} 天",
                                        cfg.usage_event_flush_batch_size,
                                        cfg.usage_event_flush_interval_seconds,
                                        format_number_u64(cfg.usage_event_flush_max_buffer_bytes),
                                        cfg.duckdb_usage_memory_limit_mib,
                                        cfg.duckdb_usage_checkpoint_threshold_mib,
                                        cfg.usage_analytics_retention_days
                                    ) }
                                </p>
                            </div>
                        }
                    </section>

                    <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                        <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Provider Proxy Bindings" }</h2>
                            <button class={classes!("ghost")} onclick={{
                                let on_reload = on_reload.clone();
                                Callback::from(move |_| on_reload.emit(()))
                            }}>
                                { if *loading { "刷新中..." } else { "刷新" } }
                            </button>
                        </div>
                        <div class={classes!("mt-4", "grid", "gap-4")}>
                            {
                                for ["codex", "kiro"].iter().map(|provider| {
                                    let binding = proxy_bindings.iter().find(|item| item.provider_type == *provider).cloned();
                                    let selected_value = if *provider == "codex" {
                                        (*codex_proxy_binding_input).clone()
                                    } else {
                                        (*kiro_proxy_binding_input).clone()
                                    };
                                    let on_change = if *provider == "codex" {
                                        let codex_proxy_binding_input = codex_proxy_binding_input.clone();
                                        Callback::from(move |event: Event| {
                                            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                                codex_proxy_binding_input.set(target.value());
                                            }
                                        })
                                    } else {
                                        let kiro_proxy_binding_input = kiro_proxy_binding_input.clone();
                                        Callback::from(move |event: Event| {
                                            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                                kiro_proxy_binding_input.set(target.value());
                                            }
                                        })
                                    };
                                    let provider_name = (*provider).to_string();
                                    let select_key = format!(
                                        "provider-proxy-binding-{}-{}",
                                        provider_name,
                                        selected_value.clone()
                                    );
                                    html! {
                                        <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                                            <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                                <div>
                                                    <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ provider_name.to_uppercase() }</div>
                                                    <div class={classes!("mt-1", "text-sm", "text-[var(--muted)]")}>
                                                        {
                                                            binding.as_ref()
                                                                .map(|item| format!("{} · {}", item.effective_source, item.effective_proxy_url.clone().unwrap_or_else(|| "-".to_string())))
                                                                .unwrap_or_else(|| "loading".to_string())
                                                        }
                                                    </div>
                                                </div>
                                                <button
                                                    class={classes!("primary")}
                                                    onclick={{
                                                        let on_save_proxy_binding = on_save_proxy_binding.clone();
                                                        let provider_name = provider_name.clone();
                                                        Callback::from(move |_| on_save_proxy_binding.emit(provider_name.clone()))
                                                    }}
                                                    disabled={(*saving_proxy_binding_provider).as_deref() == Some(provider_name.as_str())}
                                                >
                                                    {
                                                        if (*saving_proxy_binding_provider).as_deref() == Some(provider_name.as_str()) {
                                                            "保存中..."
                                                        } else {
                                                            "保存绑定"
                                                        }
                                                    }
                                                </button>
                                            </div>
                                            <label class={classes!("mt-4", "block", "text-sm")}>
                                                <span class={classes!("text-[var(--muted)]")}>{ "绑定到代理配置" }</span>
                                                <select
                                                    key={select_key}
                                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                                    value={selected_value.clone()}
                                                    onchange={on_change}
                                                >
                                                    <option value="" selected={selected_value.is_empty()}>{ "Env fallback" }</option>
                                                    { for proxy_configs.iter().map(|proxy_config| html! {
                                                        <option value={proxy_config.id.clone()} selected={selected_value == proxy_config.id}>
                                                            { format!("{} · {}", proxy_config.name, proxy_config.proxy_url) }
                                                        </option>
                                                    }) }
                                                </select>
                                            </label>
                                            if let Some(binding) = binding {
                                                <div class={classes!("mt-3", "space-y-1", "text-xs", "text-[var(--muted)]")}>
                                                    <p class={classes!("m-0")}>
                                                        { format!("effective_source: {}", binding.effective_source) }
                                                    </p>
                                                    <p class={classes!("m-0", "font-mono", "break-all")}>
                                                        { format!("effective_proxy_url: {}", binding.effective_proxy_url.unwrap_or_else(|| "-".to_string())) }
                                                    </p>
                                                    if let Some(error_message) = binding.error_message {
                                                        <p class={classes!("m-0", "text-red-600", "dark:text-red-300")}>
                                                            { format!("error: {}", error_message) }
                                                        </p>
                                                    }
                                                </div>
                                            }
                                        </article>
                                    }
                                })
                            }
                        </div>
                        <div class={classes!("mt-4", "rounded-xl", "border", "border-dashed", "border-[var(--border)]", "bg-[var(--surface-alt)]", "p-4")}>
                            <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                <div>
                                    <h3 class={classes!("m-0", "text-sm", "font-semibold")}>{ "Legacy Kiro Proxy Migration" }</h3>
                                    <p class={classes!("mt-2", "mb-0", "text-xs", "text-[var(--muted)]")}>
                                        { "扫描 ~/.static-flow/auths/kiro/*.json 中遗留的账号级代理字段，导入为共享代理配置，把对应账号切到 fixed 选择，并清掉旧字段。" }
                                    </p>
                                </div>
                                <button class={classes!("ghost")} onclick={on_import_legacy_kiro_proxy} disabled={*migrating_legacy_kiro_proxy}>
                                    { if *migrating_legacy_kiro_proxy { "导入中..." } else { "导入 Legacy Kiro Proxy" } }
                                </button>
                            </div>
                        </div>
                    </section>

                    <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                        <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Proxy Config Inventory" }</h2>
                        <p class={classes!("mt-2", "mb-0", "text-xs", "text-[var(--muted)]")}>
                            { proxy_scope_summary }
                        </p>
                        <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-2")}>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "Name" }</span>
                                <input
                                    type="text"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*create_proxy_name).clone()}
                                    disabled={!can_create_proxy_config}
                                    oninput={{
                                        let create_proxy_name = create_proxy_name.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                create_proxy_name.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm", "md:col-span-2")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "Proxy URL" }</span>
                                <input
                                    type="text"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "font-mono")}
                                    value={(*create_proxy_url).clone()}
                                    disabled={!can_create_proxy_config}
                                    oninput={{
                                        let create_proxy_url = create_proxy_url.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                create_proxy_url.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "Proxy Username" }</span>
                                <input
                                    type="text"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*create_proxy_username).clone()}
                                    disabled={!can_create_proxy_config}
                                    oninput={{
                                        let create_proxy_username = create_proxy_username.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                create_proxy_username.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <label class={classes!("text-sm")}>
                                <span class={classes!("text-[var(--muted)]")}>{ "Proxy Password" }</span>
                                <input
                                    type="text"
                                    class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                    value={(*create_proxy_password).clone()}
                                    disabled={!can_create_proxy_config}
                                    oninput={{
                                        let create_proxy_password = create_proxy_password.clone();
                                        Callback::from(move |event: InputEvent| {
                                            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                create_proxy_password.set(target.value());
                                            }
                                        })
                                    }}
                                />
                            </label>
                            <div class={classes!("md:col-span-2")}>
                                <button class={classes!("primary")} onclick={on_create_proxy_config} disabled={*creating_proxy || !can_create_proxy_config}>
                                    { if *creating_proxy { "创建中..." } else if can_create_proxy_config { "创建代理配置" } else { "edge 节点不可创建槽位" } }
                                </button>
                            </div>
                        </div>
                        // Search & filter for proxy configs
                        <div class={classes!("mt-4", "border-t", "border-[var(--border)]", "pt-4")}>
                        <div class={classes!("flex", "items-center", "gap-2")}>
                            <div class={classes!("relative", "flex-1")}>
                                <input
                                    type="text"
                                    class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2", "pr-16", "text-sm", "placeholder:text-[var(--muted)]")}
                                    placeholder="搜索代理配置..."
                                    value={(*proxy_config_search).clone()}
                                    oninput={on_proxy_search_input.clone()}
                                    onkeydown={on_proxy_search_keydown.clone()}
                                />
                                if !(*proxy_config_search).is_empty() {
                                    <button
                                        type="button"
                                        class={classes!("absolute", "right-10", "top-1/2", "-translate-y-1/2", "text-[var(--muted)]", "hover:text-[var(--text)]", "text-sm", "px-1")}
                                        onclick={on_proxy_search_clear.clone()}
                                    >
                                        { "✕" }
                                    </button>
                                }
                                <button
                                    type="button"
                                    class={classes!("absolute", "right-2", "top-1/2", "-translate-y-1/2", "rounded", "bg-[var(--primary)]", "px-2", "py-0.5", "text-xs", "text-white")}
                                    onclick={{
                                        let on_proxy_search_submit = on_proxy_search_submit.clone();
                                        Callback::from(move |_: MouseEvent| on_proxy_search_submit.emit(()))
                                    }}
                                >
                                    { "搜索" }
                                </button>
                            </div>
                            <button
                                type="button"
                                class={classes!(
                                    "rounded-full", "px-3", "py-1.5", "text-xs", "font-semibold", "border", "transition-colors",
                                    if *proxy_config_show_active_only {
                                        "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-400/50"
                                    } else {
                                        "bg-[var(--surface)] text-[var(--muted)] border-[var(--border)] hover:text-[var(--text)]"
                                    }
                                )}
                                onclick={{
                                    let proxy_config_show_active_only = proxy_config_show_active_only.clone();
                                    Callback::from(move |_| {
                                        proxy_config_show_active_only.set(!*proxy_config_show_active_only);
                                    })
                                }}
                            >
                                { "Active" }
                            </button>
                        </div>
                        <div class={classes!("mt-2", "text-xs", "text-[var(--muted)]")}>
                            { format!("共 {} 个配置 (匹配 {})", proxy_configs.len(), proxy_configs_filtered.len()) }
                        </div>
                        <div class={classes!("mt-3", "grid", "gap-4")}>
                            if proxy_configs_filtered.is_empty() {
                                <div class={classes!("rounded-xl", "border", "border-dashed", "border-[var(--border)]", "px-4", "py-10", "text-center", "text-[var(--muted)]")}>
                                    { if (*proxy_configs).is_empty() {
                                        "当前还没有可复用的代理配置。"
                                    } else {
                                        "没有匹配的代理配置。尝试调整搜索条件或清除筛选。"
                                    }}
                                </div>
                            } else {
                                { for proxy_configs_filtered.iter().map(|proxy_config| html! {
                                    <ProxyConfigEditorCard
                                        key={proxy_config.id.clone()}
                                        proxy_config={(*proxy_config).clone()}
                                        on_changed={on_reload.clone()}
                                        on_copy={on_copy.clone()}
                                        on_flash={flash.clone()}
                                    />
                                }) }
                            }
                        </div>
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
