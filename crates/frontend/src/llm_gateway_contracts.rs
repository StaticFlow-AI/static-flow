//! Public client-side constants and wire types for the LLM gateway admin API.
//!
//! The gateway implementation lives in a private submodule. Keeping the small
//! HTTP contract used by this public frontend here avoids linking server
//! internals into the open-source application.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_CODEX_ACCOUNT_RPM_LIMIT: u64 = 20;
pub(crate) const DEFAULT_CODEX_AUTO_RESET_RATE_LIMIT_THRESHOLD_PERCENT: u64 = 3;
pub(crate) const DEFAULT_KIRO_CHANNEL_RPM_LIMIT: u64 = 5;

pub(crate) const KIRO_POOL_STRATEGY_BALANCED: &str = "balanced";
pub(crate) const KIRO_POOL_STRATEGY_CREDIT_FIRST: &str = "credit_first";
pub(crate) const KIRO_POOL_STRATEGIES: [&str; 2] =
    [KIRO_POOL_STRATEGY_BALANCED, KIRO_POOL_STRATEGY_CREDIT_FIRST];

pub(crate) const KIRO_BILLABLE_MODEL_FAMILIES: [&str; 4] = ["fable", "haiku", "opus", "sonnet"];

pub(crate) const ANTHROPIC_UPSTREAM_POOL_MODE_DISABLED: &str = "disabled";
pub(crate) const ANTHROPIC_UPSTREAM_POOL_MODE_PREFERRED_BEFORE_KIRO: &str = "preferred_before_kiro";
pub(crate) const ANTHROPIC_UPSTREAM_POOL_MODE_KIRO_BEFORE_ANTHROPIC: &str = "kiro_before_anthropic";
pub(crate) const ANTHROPIC_UPSTREAM_POOL_MODE_ONLY: &str = "only";

pub(crate) const DEFAULT_ANTHROPIC_UPSTREAM_BASE_URL: &str = "https://api.anthropic.com";
pub(crate) const DEFAULT_ANTHROPIC_UPSTREAM_WEIGHT: u64 = 100;
pub(crate) const DEFAULT_ANTHROPIC_UPSTREAM_MAX_CONCURRENCY: u64 = 3;
pub(crate) const DEFAULT_ANTHROPIC_UPSTREAM_RPM_LIMIT: u64 = 5;
pub(crate) const DEFAULT_ANTHROPIC_UPSTREAM_MIN_START_INTERVAL_MS: u64 = 0;

const DEFAULT_KIRO_POLICY_FALLBACK_MODEL: &str = "claude-opus-4-6";
const ANTHROPIC_CACHE_HIT_RATE_BASIS_POINTS: u32 = 10_000;
const MAX_ANTHROPIC_CACHE_HIT_RATE_LIMITS: usize = 32;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AnthropicCacheHitRateLimit {
    pub(crate) min_context_tokens: u64,
    pub(crate) max_cache_hit_rate_basis_points: u32,
}

pub(crate) fn default_kiro_pool_strategy() -> String {
    KIRO_POOL_STRATEGY_BALANCED.to_string()
}

pub(crate) fn normalize_kiro_pool_strategy(raw: &str) -> Option<&'static str> {
    match raw.trim() {
        KIRO_POOL_STRATEGY_BALANCED => Some(KIRO_POOL_STRATEGY_BALANCED),
        KIRO_POOL_STRATEGY_CREDIT_FIRST => Some(KIRO_POOL_STRATEGY_CREDIT_FIRST),
        _ => None,
    }
}

pub(crate) fn default_kiro_policy_fallback_model() -> String {
    DEFAULT_KIRO_POLICY_FALLBACK_MODEL.to_string()
}

pub(crate) fn is_kiro_billable_model_family(family: &str) -> bool {
    KIRO_BILLABLE_MODEL_FAMILIES.contains(&family)
}

pub(crate) fn default_kiro_billable_model_multipliers() -> BTreeMap<String, f64> {
    BTreeMap::from([
        ("fable".to_string(), 2.0),
        ("haiku".to_string(), 1.0),
        ("opus".to_string(), 1.0),
        ("sonnet".to_string(), 1.0),
    ])
}

pub(crate) fn default_kiro_billable_model_multipliers_json() -> String {
    serde_json::to_string(&default_kiro_billable_model_multipliers())
        .expect("default billable multipliers should serialize")
}

pub(crate) fn normalize_anthropic_upstream_pool_mode(raw: &str) -> Option<&'static str> {
    match raw.trim() {
        ANTHROPIC_UPSTREAM_POOL_MODE_DISABLED => Some(ANTHROPIC_UPSTREAM_POOL_MODE_DISABLED),
        ANTHROPIC_UPSTREAM_POOL_MODE_PREFERRED_BEFORE_KIRO => {
            Some(ANTHROPIC_UPSTREAM_POOL_MODE_PREFERRED_BEFORE_KIRO)
        },
        ANTHROPIC_UPSTREAM_POOL_MODE_KIRO_BEFORE_ANTHROPIC => {
            Some(ANTHROPIC_UPSTREAM_POOL_MODE_KIRO_BEFORE_ANTHROPIC)
        },
        ANTHROPIC_UPSTREAM_POOL_MODE_ONLY => Some(ANTHROPIC_UPSTREAM_POOL_MODE_ONLY),
        _ => None,
    }
}

pub(crate) fn default_anthropic_upstream_pool_mode() -> String {
    ANTHROPIC_UPSTREAM_POOL_MODE_DISABLED.to_string()
}

pub(crate) fn validate_anthropic_cache_hit_rate_limits(
    limits: &[AnthropicCacheHitRateLimit],
) -> Result<(), String> {
    if limits.len() > MAX_ANTHROPIC_CACHE_HIT_RATE_LIMITS {
        return Err(format!(
            "cache_hit_rate_limits must contain at most {MAX_ANTHROPIC_CACHE_HIT_RATE_LIMITS} \
             rules"
        ));
    }

    let mut previous_threshold = None;
    let mut previous_rate = None;
    for (index, limit) in limits.iter().enumerate() {
        if limit.max_cache_hit_rate_basis_points > ANTHROPIC_CACHE_HIT_RATE_BASIS_POINTS {
            return Err(format!(
                "cache_hit_rate_limits[{index}].max_cache_hit_rate_basis_points must be between 0 \
                 and {ANTHROPIC_CACHE_HIT_RATE_BASIS_POINTS}"
            ));
        }
        if previous_threshold.is_some_and(|value| limit.min_context_tokens <= value) {
            return Err(format!(
                "cache_hit_rate_limits[{index}].min_context_tokens must be strictly increasing"
            ));
        }
        if previous_rate.is_some_and(|value| limit.max_cache_hit_rate_basis_points > value) {
            return Err(format!(
                "cache_hit_rate_limits[{index}].max_cache_hit_rate_basis_points must not increase \
                 as context grows"
            ));
        }
        previous_threshold = Some(limit.min_context_tokens);
        previous_rate = Some(limit.max_cache_hit_rate_basis_points);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_hit_rate_limits_require_ordered_decreasing_rules() {
        let limits = [
            AnthropicCacheHitRateLimit {
                min_context_tokens: 100_000,
                max_cache_hit_rate_basis_points: 8_000,
            },
            AnthropicCacheHitRateLimit {
                min_context_tokens: 500_000,
                max_cache_hit_rate_basis_points: 6_000,
            },
        ];
        assert!(validate_anthropic_cache_hit_rate_limits(&limits).is_ok());

        let invalid = [limits[1], limits[0]];
        assert!(validate_anthropic_cache_hit_rate_limits(&invalid).is_err());
    }

    #[test]
    fn public_defaults_match_the_gateway_contract() {
        assert_eq!(default_kiro_pool_strategy(), "balanced");
        assert_eq!(default_anthropic_upstream_pool_mode(), "disabled");
        assert_eq!(default_kiro_policy_fallback_model(), "claude-opus-4-6");
        assert_eq!(default_kiro_billable_model_multipliers()["fable"], 2.0);
    }
}
