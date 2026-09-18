#!/usr/bin/env bash
# Managed gateways start without migrations; install key usage/effort policies first.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLM_ACCESS_DIR="${LLM_ACCESS_DIR:-$ROOT_DIR/deps/llm-access}"
if [[ -z "${LLM_ACCESS_CONTROL_DATABASE_URL:-}" ]]; then
  source "${LLM_ACCESS_LOCAL_NEON_ENV_FILE:-$ROOT_DIR/.local/llm-access-neon.env}"
fi
: "${LLM_ACCESS_CONTROL_DATABASE_URL:?missing control database URL}"
SQL_FILE="$(mktemp /tmp/cursor-key-usage-migration.XXXXXX)"
trap 'rm -f "$SQL_FILE"' EXIT
cat > "$SQL_FILE" <<'SQL'
BEGIN;
SET LOCAL lock_timeout = '10s';
LOCK TABLE llm_access_schema_migrations IN EXCLUSIVE MODE;
SELECT EXISTS(SELECT 1 FROM llm_access_schema_migrations WHERE version = 99) AS already_applied \gset
\if :already_applied
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM llm_access_schema_migrations WHERE version = 99 AND name = 'cursor_key_cache_rate') THEN
    RAISE EXCEPTION 'Migration 99 has an unexpected name';
  END IF;
END $$;
\else
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM llm_access_schema_migrations WHERE version = 98 AND name = 'codex_key_fallback') THEN
    RAISE EXCEPTION 'Apply preceding llm-access migrations before Cursor key usage migration 99';
  END IF;
END $$;
SQL
cat "$LLM_ACCESS_DIR/crates/llm-access-migrations/migrations/postgres/0099_cursor_key_cache_rate.sql" >> "$SQL_FILE"
cat >> "$SQL_FILE" <<'SQL'
INSERT INTO llm_access_schema_migrations(version, name, applied_at_ms)
VALUES (99, 'cursor_key_cache_rate', (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint);
\endif
SELECT EXISTS(SELECT 1 FROM llm_access_schema_migrations WHERE version = 100) AS effort_applied \gset
\if :effort_applied
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM llm_access_schema_migrations WHERE version = 100 AND name = 'key_reasoning_effort') THEN
    RAISE EXCEPTION 'Migration 100 has an unexpected name';
  END IF;
END $$;
\else
SQL
cat "$LLM_ACCESS_DIR/crates/llm-access-migrations/migrations/postgres/0100_key_reasoning_effort.sql" >> "$SQL_FILE"
cat >> "$SQL_FILE" <<'SQL'
INSERT INTO llm_access_schema_migrations(version, name, applied_at_ms)
VALUES (100, 'key_reasoning_effort', (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint);
\endif
SELECT EXISTS(SELECT 1 FROM llm_access_schema_migrations WHERE version = 101) AS fast_mode_applied \gset
\if :fast_mode_applied
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM llm_access_schema_migrations WHERE version = 101 AND name = 'codex_fast_mode') THEN
    RAISE EXCEPTION 'Migration 101 has an unexpected name';
  END IF;
END $$;
\else
DO $$ BEGIN
  RAISE EXCEPTION 'Apply preceding llm-access migration 101 before managed cache default migration 102';
END $$;
\endif
SELECT EXISTS(SELECT 1 FROM llm_access_schema_migrations WHERE version = 102) AS cache_default_applied \gset
\if :cache_default_applied
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM llm_access_schema_migrations WHERE version = 102 AND name = 'managed_cache_default') THEN
    RAISE EXCEPTION 'Migration 102 has an unexpected name';
  END IF;
END $$;
\else
SQL
cat "$LLM_ACCESS_DIR/crates/llm-access-migrations/migrations/postgres/0102_managed_cache_default.sql" >> "$SQL_FILE"
cat >> "$SQL_FILE" <<'SQL'
INSERT INTO llm_access_schema_migrations(version, name, applied_at_ms)
VALUES (102, 'managed_cache_default', (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint);
\endif
SELECT cursor_cache_hit_rate_bps, reasoning_effort FROM llm_key_route_config LIMIT 0;
COMMIT;
SQL
psql --dbname="$LLM_ACCESS_CONTROL_DATABASE_URL" -X -v ON_ERROR_STOP=1 -f "$SQL_FILE"
