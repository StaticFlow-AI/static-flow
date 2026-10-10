---
name: kiro-local-account-importer
description: Use when importing local Kiro CLI accounts into StaticFlow/llm-access from local SQLite files, including discovering auth SQLite files under specified directories or the default Kiro path, reading auth_kv locally, calling the locally mapped llm-access admin API, and applying Kiro account concurrency plus proxy assignment policy.
---

# Kiro Local Account Importer

Use this skill to import Kiro accounts without involving the StaticFlow backend.
The workflow runs locally, reads SQLite files directly, and calls the mapped
`llm-access` admin API such as `http://127.0.0.1:19182`.

## Boundaries

- Do not change backend code for this workflow.
- Do not ask remote/cloud `llm-access` to read local SQLite paths.
- Parse local SQLite locally, then call `llm-access` admin APIs.
- Do not print raw `access_token`, `refresh_token`, or `client_secret`.
- Dry-run first unless the user explicitly wants a real import.
- This script creates new accounts. It rejects an existing exact name or verified
  email before writing; use the appropriate onboarding skill's existing-account
  verification/refresh flow to preserve that account's settings. Never route an
  existing account through new-account rollback.
- Verify CLI identity in the chosen data root before supplying `--email`. For
  Builder ID, use the exact isolated SQLite path and require IDC credentials;
  an unrelated default social login must remain untouched.

## API Flow

Use `scripts/import_kiro_accounts.py`.

The script:

1. Discovers SQLite files from `--sqlite-file`, `--search-dir`, or the default
   `~/.local/share/kiro-cli/data.sqlite3`.
2. Reads supported Kiro auth records from `auth_kv`:
   - social: `kirocli:social:token`
   - IDC/OIDC: `kirocli:odic:token`, `kirocli:oidc:token`
   - device registration: matching `device-registration` keys
3. Reads all account pages using `has_more` and offsets based on returned row
   counts. Rejects incomplete/repeated pagination and existing names/emails, then
   creates the Kiro account through:
   `POST /admin/kiro-gateway/accounts/import-auth`
4. Sets standard scheduling:
   - `kiro_channel_max_concurrency = 3`
   - `kiro_channel_min_start_interval_ms = random[200, 1000]`
   - `minimum_remaining_credits_before_block >= 10`
5. Assigns proxy by patching:
   `PATCH /admin/kiro-gateway/accounts/{name}`
6. Validates the account by refreshing balance through the selected proxy:
   `POST /admin/kiro-gateway/accounts/{name}/balance`
7. Accepts the import only when balance refresh succeeds, remaining credits are
   at least `10`, and the refreshed upstream `user_id` is not already present
   on an existing Kiro account. If refresh fails, try the next United States
   proxy. If all proxies fail, if the account's remaining credits are below
   `10`, or if the refreshed `user_id` is a duplicate, delete the newly imported
   account and report it as invalid. `deleted: true` also requires rereading the
   inventory and confirming that exact name is absent. An uncertain create result
   must be reconciled by exact name before another attempt.

## Proxy Policy

Default proxy behavior:

- Fetch active proxies from `GET /admin/llm-gateway/proxy-configs`.
- Keep only active United States proxy nodes. With the current proxy schema,
  this is inferred from established proxy names such as `do-us-*`, `aws_us*`,
  `us-home*`, `my-homeus*`, and `dmit-us`.
- Count active Kiro accounts currently fixed to each proxy via paginated
  `GET /admin/kiro-gateway/accounts?limit=200&offset=...`. Requesting a large limit
  does not bypass the server's page cap. Ignore direct/auto bindings and disabled
  accounts when counting fixed proxy load.
- Try to read hot Kiro latency ranking from
  `/internal/kiro-gateway/latency-ranking?source=hot&window=1h`.
- Prefer the least-used active United States proxy first.
- Use hot first-token latency only as a tie-breaker within the same account
  count bucket, then break remaining ties by proxy name.
- If latency data is unavailable, choose the least-used active proxy and break
  ties by proxy name.
- Direct/no-proxy imports are not allowed for Kiro accounts. Do not hardcode the
  previous run's successful proxy or permanently exclude a node after one failure.

## Commands

Default dry run:

```bash
python3 skills/kiro-local-account-importer/scripts/import_kiro_accounts.py \
  --admin-base-url http://127.0.0.1:19182
```

Dry run for one explicit SQLite and an email already verified with `kiro-cli whoami`:

```bash
python3 skills/kiro-local-account-importer/scripts/import_kiro_accounts.py \
  --admin-base-url http://127.0.0.1:19182 \
  --sqlite-file /path/to/data.sqlite3 \
  --account-name kiro-main \
  --email user@example.com
```

`--email` applies to exactly one SQLite file; omit it for a directory batch.
The script handles nullable email fields when checking existing accounts. Email
is supplied from verified login identity, not inferred from a matching balance.

Dry run for a directory:

```bash
python3 skills/kiro-local-account-importer/scripts/import_kiro_accounts.py \
  --admin-base-url http://127.0.0.1:19182 \
  --search-dir /path/to/kiro-auth-dbs \
  --name-prefix kiro-
```

Apply after reviewing dry-run output:

```bash
python3 skills/kiro-local-account-importer/scripts/import_kiro_accounts.py \
  --admin-base-url http://127.0.0.1:19182 \
  --search-dir /path/to/kiro-auth-dbs \
  --name-prefix kiro- \
  --apply
```

If the mapped admin API requires a token, supply `STATICFLOW_ADMIN_TOKEN` through
the process environment or a hidden prompt. The script reads it automatically;
do not expand it into command-line arguments or print it.

## Verification

After real import, query the intended name and compare the returned name/email
exactly. Inspect the full paginated inventory when checking upstream duplicates.

```bash
curl -fsS 'http://127.0.0.1:19182/admin/kiro-gateway/accounts?q=kiro-main&limit=20&offset=0' \
  | jq '.accounts[] | select(.name == "kiro-main") | {name, email, disabled, auth_method, provider, kiro_channel_max_concurrency, kiro_channel_min_start_interval_ms, minimum_remaining_credits_before_block, proxy_mode, proxy_config_id}'
```

Expected:

- imported accounts exist;
- max concurrency is `3`;
- min start interval is between `200` and `1000`;
- minimum remaining credits is at least `10`;
- proxy mode is `fixed`;
- proxy name is a United States node.
- the real import output has `validated: true` and includes the refreshed
  balance response.
- no imported account duplicates an existing refreshed Kiro `user_id`.

Balance validation proves import/auth/proxy access. For a request to verify actual
model use, follow the [Builder ID request verification](../kiro-aws-builder-id-onboarder/references/import-and-verify.md#real-gateway-request)
after establishing the account identity. Report the actual subscription and dated
balance, not an assumed student tier or an import-time balance as post-test state.
