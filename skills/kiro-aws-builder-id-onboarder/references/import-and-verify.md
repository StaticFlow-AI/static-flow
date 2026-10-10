# Import Builder ID credentials and prove request routing

Use the sibling [importer skill](../../kiro-local-account-importer/SKILL.md) and
[Python implementation](../../kiro-local-account-importer/scripts/import_kiro_accounts.py)
as the credential parser and import-policy implementation. The bundled script
paginates inventory, accepts a verified `--email` for one SQLite, and rejects
existing name/email matches. Prefer this path over rebuilding a temporary importer.

## Read state and prepare the import

1. Confirm the mapped admin API is reachable. The current StaticFlow default is
   `http://127.0.0.1:19182`. Use the requested environment if different. Read any
   required admin token from a process environment variable and build the header
   in memory; do not expose it in curl or Python command-line arguments.
2. Search accounts by the intended name/email, then compare returned fields exactly.
   Enumerate with `limit`/`offset` and follow `has_more` when checking duplicate
   upstream IDs and proxy load. Advance by the number of returned rows; an empty
   page with `has_more=true` is an error. A single `limit=10000` request is not a
   complete inventory. Do not assume every resource supports `q` filtering.
3. Parse the chosen local SQLite on this machine. Require IDC/OIDC credentials and
   the matching device registration. Map them to `auth_method=idc`, `provider=aws`,
   and retain the actual region/profile metadata. Set `email` from the verified
   Builder ID identity; token JSON may not contain the email. Existing inventory
   email fields may be null; compare with `(account.get("email") or "").casefold()`.
4. Use the user's exact target name or a normalized `kiro-<localpart>-builder-id`.
   If the name already exists with a conflicting/unknown identity, stop before
   overwrite. If the same account is already healthy, verify it rather than import
   again. If an existing account needs an authorized credential refresh, preserve
   its proxy/scheduling settings and use an update flow with no delete-on-failure
   cleanup. Check the current import API's replacement behavior before that write.

For a new account, run the importer with `--sqlite-file <exact isolated database>`,
`--account-name <intended name>`, and `--email <verified Builder ID email>`. Review
its dry run, then use the same arguments with `--apply` when import is authorized.
Do not rerun a completed import just to produce another dry-run plan.

If composing its Python functions, reuse `fetch_kiro_accounts` for complete
inventory and pass that list as `existing_accounts=` to `import_account`, along
with `existing_user_id_map(accounts)`. Set `auth.body['email']` from the verified
CLI identity. Count only enabled accounts with `proxy_mode=fixed` for proxy load.
The same inventory must support both duplicate checks and proxy ranking.

Print only account name, auth type, chosen proxy name, scheduling, validation
status, and selected balance fields. Keep token/client/proxy secrets in memory.

## Import policy and evidence

The established new-account policy is:

- max concurrency `3`;
- minimum start interval chosen in `[200, 1000]` ms;
- minimum remaining credits at least `10`;
- fixed active US proxy, balanced by active account count, latency as a tie-breaker;
- actual auth/API region preserved; stop if its proxy-region mapping is unsupported.

Use `POST /admin/kiro-gateway/accounts/import-auth`, set the selected proxy via
`PATCH /admin/kiro-gateway/accounts/{name}`, and refresh with
`POST /admin/kiro-gateway/accounts/{name}/balance`. URL-encode account names.
Require successful refresh, sufficient remaining credits, and a non-duplicate
upstream `user_id`. Report the returned subscription and limit. Student status and
1000 credits are observations for some accounts, not defaults to manufacture.

On a proxy transport failure, try the next eligible active proxy within a bounded
candidate list. Stop on an explicit account/auth restriction instead of rotating
proxies indefinitely. On an uncertain create response, query the exact name before
retrying. Roll back only this run's confirmed new invalid record; never delete an
older account found by duplicate detection. Verify cleanup rather than assuming it.

Reread the exact imported account and verify email, `disabled=false`,
`auth_method=idc`, `provider=aws`, proxy selection, and scheduling. Existing-account
refresh preserves established policy rather than applying new-account defaults.

## Real gateway request

For “正常请求通”, exercise the intended public gateway, normally
`https://ackingliu.top/v1/messages`, using the imported account. Do not use an
unrestricted shared key: a successful response from another account proves nothing
about this one.

1. Read `GET /admin/kiro-gateway/models` (`models[]`) and choose an entry with
   `enabled=true` and `upstream=true`, using its `model_id`, or the user's requested
   model. Do not assume fields named `id` or `name`, or freeze a past model name
   into the workflow. If it is unavailable, report that specifically.
2. Reuse an authorized key only if already pinned to this exact account and suitable
   for the test. Otherwise create a uniquely named temporary Kiro key with
   `POST /admin/kiro-gateway/keys`; a small test budget such as
   `quota_billable_limit=200000` and `public_visible=false` is sufficient. Keep its
   returned `id` and `secret` in process memory. Do not print the raw response.
3. Patch only that temporary key through `/admin/kiro-gateway/keys/{id}`:
   `route_strategy=fixed`, `fixed_account_name=<exact account>`,
   `kiro_anthropic_upstream_pool_mode=disabled`,
   `kiro_policy_fallback_enabled=false`, `request_max_concurrency=1`.
   Verify the returned effective binding. Keep moderation/security defaults intact.
   If a reused key permits fallback, use a temporary key instead of modifying it.
4. Send one short nonstreaming Anthropic-compatible request with `x-api-key`,
   `anthropic-version: 2023-06-01`, the selected `model`, `max_tokens: 64`, and a
   user message such as `Reply with exactly KIRO_OK.` Use a bounded timeout.
   Check HTTP status, generated text, and successful completion; HTTP 200 by itself
   is insufficient. If a streaming test is requested, require terminal completion
   and inspect error events as well.
5. Verify usage attribution to the exact account and test key. Query
   `/admin/kiro-gateway/usage?key_id=...&start_ms=...&limit=...`.
   `since` is not the supported time-filter field. Analytics can lag; check
   `/admin/llm-access/usage-journal/preview?limit=...&offset=...`; events are under
   `preview.events`, while `has_more` is top-level. Match the `key_id`/account/event,
   following pagination if necessary. The journal can prove
   attribution before analytics catches up. Report any still-missing evidence
   separately; do not repeat a billable request merely to fill the usage list.
6. Delete the exact temporary key in a `finally` cleanup path, including when the
   model request fails. Verify absence by exact ID in a paginated key inventory;
   a `q` query returning rows is not proof that this key remains. If creation had
   an ambiguous result, reconcile its unique name before another create or cleanup.
   Surface a cleanup failure and the non-secret key ID. Never delete a reused key.

A direct `POST /admin/kiro-gateway/accounts/{name}/probe-model` request with
`{"model":"<selected upstream model>"}` is useful for diagnosing an upstream
failure. Its `ok` result is narrower evidence than a full public gateway request;
do not present it as proof of the public key/routing path.

Record only the verified email, gateway name, subscription/balance with its time,
proxy name, tested model, response outcome, and attribution. Mark an import-time
balance as such; do not present it as a refreshed balance after the test.
