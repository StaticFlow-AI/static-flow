#!/usr/bin/env bash
# Cursor deliberately connects without migrations. Apply its additive Grok
# schema before activating a binary that reads those columns.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLM_ACCESS_DIR="${LLM_ACCESS_DIR:-$ROOT_DIR/deps/llm-access}"
if [[ -z "${LLM_ACCESS_CONTROL_DATABASE_URL:-}" ]]; then
  # shellcheck source=/dev/null
  source "${LLM_ACCESS_LOCAL_NEON_ENV_FILE:-$ROOT_DIR/.local/llm-access-neon.env}"
fi
: "${LLM_ACCESS_CONTROL_DATABASE_URL:?missing control database URL}"
SQL_FILE="$(mktemp /tmp/grok-reset-migration.XXXXXX)"
trap 'rm -f "$SQL_FILE"' EXIT
cat > "$SQL_FILE" <<'SQL'
BEGIN;
SET LOCAL lock_timeout = '10s';
LOCK TABLE llm_access_schema_migrations IN EXCLUSIVE MODE;
SELECT EXISTS(SELECT 1 FROM llm_access_schema_migrations WHERE version = 86) AS already_applied \gset
\if :already_applied
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM llm_access_schema_migrations WHERE version = 86 AND name = 'grok_reset_credits') THEN
    RAISE EXCEPTION 'Migration 86 has an unexpected name';
  END IF;
END $$;
\else
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM llm_access_schema_migrations WHERE version = 85) THEN
    RAISE EXCEPTION 'Apply preceding llm-access migrations before Grok reset migration 86';
  END IF;
END $$;
SQL
cat "$LLM_ACCESS_DIR/crates/llm-access-migrations/migrations/postgres/0086_grok_reset_credits.sql" >> "$SQL_FILE"
cat >> "$SQL_FILE" <<'SQL'
INSERT INTO llm_access_schema_migrations(version, name, applied_at_ms)
VALUES (86, 'grok_reset_credits', (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint);
\endif
-- Also verify a previously recorded migration has its required objects.
SELECT auto_reset_rate_limit_enabled, auto_reset_rate_limit_threshold_percent FROM llm_cursor_accounts LIMIT 0;
SELECT user_id FROM llm_grok_reset_credit_guards LIMIT 0;
SELECT idempotency_key FROM llm_grok_reset_credit_attempts LIMIT 0;
COMMIT;
SQL
psql --dbname="$LLM_ACCESS_CONTROL_DATABASE_URL" -X -v ON_ERROR_STOP=1 -f "$SQL_FILE"
