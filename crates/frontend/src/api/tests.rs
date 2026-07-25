use super::*;

#[test]
fn reset_credit_consume_request_keeps_caller_idempotency_and_optional_credit() {
    let selected = ConsumeCodexRateLimitResetCreditRequest {
        idempotency_key: "attempt-1".to_string(),
        credit_id: Some("credit-1".to_string()),
    };
    let selected_json = serde_json::to_value(selected).expect("selected request");
    assert_eq!(selected_json["idempotency_key"], "attempt-1");
    assert_eq!(selected_json["credit_id"], "credit-1");

    let generic = ConsumeCodexRateLimitResetCreditRequest {
        idempotency_key: "attempt-2".to_string(),
        credit_id: None,
    };
    let generic_json = serde_json::to_value(generic).expect("generic request");
    assert_eq!(generic_json["idempotency_key"], "attempt-2");
    assert!(generic_json.get("credit_id").is_none());
}

#[test]
fn reset_credit_details_preserve_picker_fields() {
    let details: CodexRateLimitResetCreditsDetails = serde_json::from_value(serde_json::json!({
        "available_count": 1,
        "credits": [{
            "id": "credit-1",
            "reset_type": "codex_rate_limits",
            "status": "available",
            "granted_at": "2026-07-01T00:00:00Z",
            "expires_at": "2026-07-31T00:00:00Z",
            "title": "One reset",
            "description": "Resets exhausted windows"
        }]
    }))
    .expect("reset-credit details");

    assert_eq!(details.available_count, 1);
    assert_eq!(details.credits[0].id, "credit-1");
    assert_eq!(details.credits[0].expires_at.as_deref(), Some("2026-07-31T00:00:00Z"));
    assert_eq!(details.credits[0].title.as_deref(), Some("One reset"));
}

#[test]
fn image_generation_request_serializes_b64_response_format() {
    let value = serde_json::to_value(AdminGpt2ApiRsImageGenerationRequest::default())
        .expect("request should serialize");

    assert_eq!(value.get("response_format"), Some(&serde_json::json!("b64_json")));
}

#[test]
fn admin_kiro_account_statuses_response_defaults_are_empty() {
    let response: AdminKiroAccountStatusesResponse =
        serde_json::from_str("{}").expect("response should parse");

    assert!(response.accounts.is_empty());
    assert_eq!(response.total, 0);
    assert_eq!(response.limit, 0);
    assert_eq!(response.offset, 0);
}

#[test]
fn kiro_account_view_parses_issue_fields() {
    let account: KiroAccountView = serde_json::from_value(serde_json::json!({
        "name": "kiro-a",
        "issue_kind": "auth_401",
        "issue_summary": "Kiro status API returned 401 Unauthorized",
        "issue_at_ms": 123456
    }))
    .expect("account should parse");

    assert_eq!(account.issue_kind.as_deref(), Some("auth_401"));
    assert_eq!(account.issue_summary.as_deref(), Some("Kiro status API returned 401 Unauthorized"));
    assert_eq!(account.issue_at_ms, Some(123_456));
}

#[test]
fn admin_kiro_cache_stats_response_defaults_are_empty() {
    let response: AdminKiroCacheStatsResponse =
        serde_json::from_str(r#"{"mode":"prefix_tree"}"#).expect("response should parse");

    assert_eq!(response.mode, "prefix_tree");
    assert_eq!(response.page_size_tokens, 0);
    assert_eq!(response.prefix_tree.resident_tokens, 0);
    assert_eq!(response.conversation_anchors.entries, 0);
    assert_eq!(response.process_memory.rss_bytes, None);
}

#[test]
fn admin_gateway_key_view_defaults_full_request_logging_off() {
    let key: AdminLlmGatewayKeyView =
        serde_json::from_str(r#"{"id":"k","name":"K","provider_type":"kiro"}"#)
            .expect("key should parse");

    assert!(!key.kiro_full_request_logging_enabled);
    assert!(!key.kiro_remote_media_resolution_enabled);
    assert!(key.codex_image_generation_enabled);
    assert!(key.codex_image_standalone_generation_enabled);
    assert!(!key.codex_image_direct_generation_enabled);
    assert!(key.codex_responses_lite_enabled);
}

#[test]
fn admin_gateway_key_view_defaults_codex_image_usage_to_zero() {
    let key: AdminLlmGatewayKeyView =
        serde_json::from_str(r#"{"id":"k","name":"K","provider_type":"codex"}"#)
            .expect("key should parse");

    assert_eq!(key.codex_image_usage_tokens, 0);
    assert_eq!(key.codex_image_usage_missing_events, 0);
    assert_eq!(key.codex_image_last_used_at, None);
}

#[test]
fn account_summary_view_preserves_email_and_defaults_missing_email() {
    let account: AccountSummaryView =
        serde_json::from_str(r#"{"name":"codex-a","email":"a@example.com"}"#)
            .expect("account should parse");
    assert_eq!(account.email.as_deref(), Some("a@example.com"));

    let legacy: AccountSummaryView =
        serde_json::from_str(r#"{"name":"codex-a"}"#).expect("legacy account should parse");
    assert_eq!(legacy.email, None);
}

#[test]
fn admin_account_page_merges_append_rows_without_losing_totals() {
    let first = AccountListResponse {
        accounts: vec![AccountSummaryView {
            name: "a1".to_string(),
            ..AccountSummaryView::default()
        }],
        total: 2,
        limit: 1,
        offset: 0,
        has_more: true,
        summary: AdminAccountsSummaryView {
            total: 2,
            active_count: 1,
            ..AdminAccountsSummaryView::default()
        },
        generated_at: 10,
    };
    let next = AccountListResponse {
        accounts: vec![AccountSummaryView {
            name: "a2".to_string(),
            ..AccountSummaryView::default()
        }],
        total: 2,
        limit: 1,
        offset: 1,
        has_more: false,
        summary: AdminAccountsSummaryView {
            total: 2,
            active_count: 2,
            ..AdminAccountsSummaryView::default()
        },
        generated_at: 20,
    };

    let merged = merge_admin_codex_account_pages(first, next);

    assert_eq!(
        merged
            .accounts
            .iter()
            .map(|account| account.name.as_str())
            .collect::<Vec<_>>(),
        ["a1", "a2"]
    );
    assert_eq!(merged.limit, 2);
    assert_eq!(merged.offset, 0);
    assert!(!merged.has_more);
    assert_eq!(merged.summary.active_count, 2);
    assert_eq!(merged.generated_at, 20);
}

#[test]
fn account_summary_view_deserializes_rate_limit_reset_credits() {
    let account: AccountSummaryView = serde_json::from_value(serde_json::json!({
        "name": "codex-pro",
        "status": "active",
        "rate_limit_reset_credits_available": 2
    }))
    .expect("account summary should parse");

    assert_eq!(account.rate_limit_reset_credits_available, Some(2));
    assert!(!account.auto_reset_rate_limit_enabled);
    assert_eq!(account.auto_reset_rate_limit_threshold_percent, 3);
    assert!(!account.codex_image_generation_enabled);
    assert_eq!(account.codex_image_generation_max_concurrency, 3);
}

#[test]
fn account_summary_view_deserializes_auto_reset_settings() {
    let account: AccountSummaryView = serde_json::from_value(serde_json::json!({
        "name": "codex-pro",
        "status": "active",
        "auto_reset_rate_limit_enabled": true,
        "auto_reset_rate_limit_threshold_percent": 7
    }))
    .expect("auto reset settings should parse");

    assert!(account.auto_reset_rate_limit_enabled);
    assert_eq!(account.auto_reset_rate_limit_threshold_percent, 7);
}

#[test]
fn build_admin_kiro_account_statuses_url_encodes_prefix_and_window() {
    let url = build_admin_kiro_account_statuses_url(&AdminKiroAccountStatusesQuery {
        prefix: Some("alpha team".to_string()),
        q: Some("ntagueik".to_string()),
        issue: Some("abnormal".to_string()),
        limit: Some(24),
        offset: Some(48),
    });

    assert!(url.contains("/admin/kiro-gateway/accounts/statuses"));
    assert!(url.contains("prefix=alpha%20team"));
    assert!(url.contains("q=ntagueik"));
    assert!(url.contains("issue=abnormal"));
    assert!(url.contains("limit=24"));
    assert!(url.contains("offset=48"));
}

#[test]
fn build_admin_kiro_cache_stats_url_uses_admin_prefix_and_cache_buster() {
    let url = build_admin_kiro_cache_stats_url_for_ts(123);

    assert!(url.contains("/admin/kiro-gateway/cache-stats"));
    assert!(url.contains("_ts=123"));
}

#[test]
fn build_admin_kiro_usage_event_detail_url_encodes_event_id() {
    let url = build_admin_kiro_usage_event_detail_url("llm usage/one");

    assert!(url.contains("/admin/kiro-gateway/usage/llm%20usage%2Fone"));
}

#[test]
fn build_llm_gateway_model_catalog_url_uses_public_api_prefix() {
    let url =
        build_llm_gateway_model_catalog_url_for_ts(Some("/llm-gateway/model-catalog.json"), 123);

    assert!(url.contains("/api/llm-gateway/model-catalog.json"));
    assert!(url.contains("_ts=123"));
}

#[test]
fn derive_local_media_api_base_from_http_api_base_uses_backend_origin() {
    let base = derive_local_media_api_base_from_api_base("http://127.0.0.1:39080/api");
    assert_eq!(base, "http://127.0.0.1:39080/admin/local-media/api");
}

#[test]
fn derive_llm_access_admin_base_from_relative_api_base_uses_same_origin_admin_path() {
    let base = derive_llm_access_admin_base_from_api_base("/api");
    assert_eq!(base, "");
}

#[test]
fn derive_llm_access_admin_base_from_local_http_api_base_uses_backend_origin() {
    let base = derive_llm_access_admin_base_from_api_base("http://127.0.0.1:39080/api");
    assert_eq!(base, "http://127.0.0.1:39080");
}

#[test]
fn resolve_llm_access_admin_base_prefers_explicit_override() {
    let base = resolve_llm_access_admin_base(Some("https://llm-admin.example.com/"), "/api");
    assert_eq!(base, "https://llm-admin.example.com");
}

#[test]
fn derive_local_media_api_base_from_same_origin_falls_back_to_relative_admin_path() {
    let base = derive_local_media_api_base_from_api_base("");
    assert_eq!(base, "/admin/local-media/api");
}

#[test]
fn resolve_local_media_asset_url_for_base_rewrites_relative_admin_asset_to_backend_origin() {
    let url = resolve_local_media_asset_url_for_base(
        "http://127.0.0.1:39080/admin/local-media/api",
        "/admin/local-media/api/poster?file=demo.mp4",
    );
    assert_eq!(url, "http://127.0.0.1:39080/admin/local-media/api/poster?file=demo.mp4");
}

#[test]
fn resolve_local_media_asset_url_for_base_keeps_same_origin_relative_path_when_base_is_relative() {
    let url = resolve_local_media_asset_url_for_base(
        "/admin/local-media/api",
        "/admin/local-media/api/playback/raw?file=demo.mp4",
    );
    assert_eq!(url, "/admin/local-media/api/playback/raw?file=demo.mp4");
}

#[test]
#[cfg(not(feature = "mock"))]
fn build_admin_local_media_raw_playback_uses_raw_mode_and_encoded_url() {
    let response = build_admin_local_media_raw_playback("未归类/demo clip.mp4");
    assert_eq!(response.status, LocalMediaPlaybackStatus::Ready);
    assert_eq!(response.mode, Some(LocalMediaPlaybackMode::Raw));
    assert_eq!(response.title, "demo clip.mp4");
    assert!(response
        .player_url
        .as_deref()
        .unwrap_or_default()
        .contains("playback/raw?file=%E6%9C%AA%E5%BD%92%E7%B1%BB%2Fdemo%20clip.mp4"));
}

#[test]
fn build_admin_local_media_upload_tasks_url_uses_admin_prefix() {
    assert!(build_admin_local_media_upload_tasks_url()
        .ends_with("/admin/local-media/api/uploads/tasks"));
}

#[test]
fn compaction_runtime_config_deserializes_worker_count() {
    let config: CompactionRuntimeConfig = serde_json::from_str(
        r#"{
                "enabled": true,
                "scan_interval_seconds": 900,
                "fragment_threshold": 128,
                "prune_older_than_hours": 1,
                "worker_count": 4
            }"#,
    )
    .expect("compaction config should parse");

    assert_eq!(config.worker_count, 4);
}

#[test]
fn music_runtime_config_deserializes_admin_payload() {
    let config: MusicRuntimeConfig = serde_json::from_str(
        r#"{
                "play_dedupe_window_seconds": 60,
                "comment_rate_limit_seconds": 90,
                "list_default_limit": 20
            }"#,
    )
    .expect("music config should parse");

    assert_eq!(config.play_dedupe_window_seconds, 60);
    assert_eq!(config.comment_rate_limit_seconds, 90);
    assert_eq!(config.list_default_limit, 20);
}

#[test]
fn llm_gateway_runtime_config_ignores_legacy_usage_maintenance_fields() {
    let config: LlmGatewayRuntimeConfig = serde_json::from_str(
            r#"{
                "auth_cache_ttl_seconds": 60,
                "max_request_body_bytes": 8388608,
                "account_failure_retry_limit": 10,
                "kiro_channel_max_concurrency": 1,
                "kiro_channel_min_start_interval_ms": 0,
                "codex_status_refresh_min_interval_seconds": 240,
                "codex_status_refresh_max_interval_seconds": 300,
                "codex_status_account_jitter_max_seconds": 10,
                "kiro_status_refresh_min_interval_seconds": 240,
                "kiro_status_refresh_max_interval_seconds": 300,
                "kiro_status_account_jitter_max_seconds": 10,
                "usage_event_flush_batch_size": 256,
                "usage_event_flush_interval_seconds": 15,
                "usage_event_flush_max_buffer_bytes": 8388608,
                "usage_event_maintenance_enabled": true,
                "usage_event_maintenance_interval_seconds": 3600,
                "usage_event_detail_retention_days": 7,
                "usage_analytics_retention_days": 14,
                "kiro_cache_kmodels_json": "{}",
                "kiro_billable_model_multipliers_json": "{\"haiku\":1.0,\"opus\":1.0,\"sonnet\":1.0}",
                "kiro_cache_policy_json": "{}",
                "kiro_prefix_cache_mode": "prefix_tree",
                "kiro_prefix_cache_max_tokens": 4000000,
                "kiro_prefix_cache_entry_ttl_seconds": 21600,
                "kiro_conversation_anchor_max_entries": 20000,
                "kiro_conversation_anchor_ttl_seconds": 86400
            }"#,
        )
        .expect("llm gateway runtime config should parse");

    assert_eq!(config.usage_event_flush_interval_seconds, 15);
    assert_eq!(config.codex_client_version, DEFAULT_LLM_GATEWAY_CODEX_CLIENT_VERSION);
    assert_eq!(config.duckdb_usage_memory_limit_mib, 1024);
    assert_eq!(config.duckdb_usage_checkpoint_threshold_mib, 16);
    assert!(config.usage_journal_enabled);
    assert_eq!(config.usage_journal_max_file_bytes, 64 * 1024 * 1024);
    assert_eq!(config.usage_journal_max_file_age_ms, 300_000);
    assert_eq!(config.usage_journal_max_files, 128);
    assert_eq!(config.usage_journal_block_target_uncompressed_bytes, 1024 * 1024);
    assert_eq!(config.usage_journal_block_max_events, 1024);
    assert_eq!(config.usage_journal_fsync_interval_ms, 250);
    assert_eq!(config.usage_journal_zstd_level, 3);
    assert_eq!(config.usage_journal_consumer_lease_ms, 300_000);
    assert!(!config.usage_journal_delete_bad_files);
    assert_eq!(config.kiro_context_usage_min_request_tokens, 15_000);
    assert_eq!(config.usage_query_bind_addr, "127.0.0.1:19081");
    assert_eq!(config.usage_query_base_url, "http://127.0.0.1:19081");
    assert_eq!(config.usage_analytics_retention_days, 14);
    assert!(config.codex_session_affinity_enabled);
    assert_eq!(config.codex_session_affinity_max_entries, 20_000);
    assert_eq!(config.codex_session_affinity_ttl_seconds, 21_600);
    assert!(config.codex_fallback_affinity_enabled);
    assert_eq!(config.codex_fallback_affinity_ttl_seconds, 1_800);
    assert_eq!(config.codex_fallback_affinity_prefix_bytes, 4_096);
    assert_eq!(config.codex_fallback_affinity_min_body_bytes, 128);
    // Snapshot fields are omitted from the JSON above; serde defaults apply.
    assert!(!config.kiro_cache_snapshot_enabled);
    assert_eq!(config.kiro_cache_snapshot_interval_seconds, 300);
    assert_eq!(config.kiro_cache_snapshot_ttl_seconds, 86_400);
    assert_eq!(config.kiro_cache_snapshot_max_tokens, 0);
    assert_eq!(config.kiro_cache_snapshot_max_anchor_entries, 0);
}

#[test]
fn admin_usage_events_response_deserializes_retention_days() {
    let response: AdminLlmGatewayUsageEventsResponse = serde_json::from_str(
        r#"{
                "total": 0,
                "offset": 0,
                "limit": 20,
                "has_more": false,
                "current_rpm": 0,
                "current_in_flight": 0,
                "retention_days": 7,
                "totals": {
                    "event_count": 12,
                    "input_uncached_tokens": 1200,
                    "input_cached_tokens": 300,
                    "output_tokens": 450,
                    "billable_tokens": 1650
                },
                "events": [],
                "generated_at": 1700000000000
            }"#,
    )
    .expect("usage response should parse retention days");

    assert_eq!(response.retention_days, 7);
    assert_eq!(response.totals.event_count, 12);
    assert_eq!(response.totals.billable_tokens, 1_650);
}

#[test]
fn usage_journal_status_contract_is_available_to_admin_pages() {
    let status = AdminUsageJournalStatusView::default();
    let _fetch = fetch_admin_usage_journal_status;

    assert_eq!(status.current_rpm, 0);
    assert_eq!(status.current_in_flight, 0);
    assert_eq!(status.worker.processed_events, 0);
    assert_eq!(status.worker.process_memory.rss_bytes, None);
    assert!(status.sealed_files.is_empty());
}
