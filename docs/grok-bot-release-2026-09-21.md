# Grok Bot production release — 2026-09-21

Source: llm-access `1c965385e5097f74522f3ded14da44daf84861eb`.
The release includes OAuth polling/catalog alignment from `60b51a5`, model,
fallback and request adaptation from `335c7af`, and live acceptance notes.
StaticFlow recorded the published child at `6656d4f9` before deployment.

## Public contract

Use the existing Cursor gateway base URL:
`https://ackingliu.top/api/cursor-gateway/v1`, with a Cursor-channel API key.
`GET /models` exposes exactly these ten Bot items, all prefixed `grok-bot:`:

- `default`, `grok-4.6`, `grok-4.5`, `composer-2.5`
- `gpt-5.6-luna`, `gpt-5.4-mini`, `claude-haiku-4-5`
- `gemini-3.1-pro`, `gemini-3-flash`, `gemini-2.5-flash`

The previously shared 38-model catalog is filtered before public discovery and
account admission. The existing live Cursor account group now contains Cursor,
Grok Build and the already-authorized Grok Bot account. Other key/group policy
was preserved.

Authenticated per-key/moderation fallback exhausts the eligible Cursor/Grok pool,
then tries Bot `default` before returning to the configured outer chain. The
existing outer order remains Antigravity Gemini 3.8 Flash Tiered, then Sonnet 4.6.
Explicit transport/model calls retain their selection; account-group, review,
concurrency and RPM restrictions remain enforced.

Messages and Responses accept streaming/buffered text, images, inline UTF-8 text
documents, function/custom/namespaced tools and complete call/result history.
Hosted search/fetch use the selected account, enforce `max_uses`, continue
inference and emit tool lifecycle events. Unsupported binary/remote documents,
forced tool choices, domain/location restrictions and constrained output formats
return explicit request errors.

## Verification

- Dedicated local review and scoped formatting; the clean release snapshot
  passed 3,223 workspace tests and all-target workspace Clippy with `-D warnings`.
  Isolated PostgreSQL tests ran. Two optional live-Valkey tests remained ignored;
  Redis has an existing dependency future-incompatibility notice.
- Live local acceptance covered default text, streaming Responses, images,
  text documents, Messages/Responses function call/result continuation, hosted
  search/fetch, and an authenticated unavailable-pool-to-Bot-default request.
  Deterministic tests also cover all three provider hops, leases and tracing.
- Public model discovery returned ten Bot entries. Default text returned the
  expected marker; Grok 4.6 search and streaming fetch both completed and returned
  real tool results. The temporary validation key was deleted.
- Three successful public inference events were read back from durable Usage.
  The two hosted-tool requests each recorded two inference rounds. Records retain
  account, requested/mapped model, conversion diagnostics, token provenance,
  inference invocation IDs, tool status, result count and duration.
- OAuth console grant/cookie/inventory checks passed after restart, and the
  existing Bot OAuth session remained active.
- Installed and running binary hashes matched the artifacts below. All three
  targeted services were active with `NRestarts=0`. Usage-worker and Antigravity
  PID/start time/restart counters were unchanged. The inactive image gateway
  remained inactive; no local StaticFlow/Pingora release was performed.

| Binary | SHA-256 |
|---|---|
| `llm-access` | `20601ee294d610ff5996fb40508e8adee168a0613e6a66b9ffd99b7e3e215d34` |
| `llm-access-cursor` | `a655e35b47656167f0fbf0cf1143d9ca9b47a73c06edfff688acece06ba7bb51` |
| `llm-access-oauth` | `a4ab6c4fc10d99fb281824471243505060040ff90c62f6f7d6149e7a760aaca2` |

## Observed upstream limitation

The verified catalog establishes inference access, not identical tool support
across every model. Tool-bearing default and Haiku requests returned upstream
`ERROR_PROVIDER_ERROR` / 429 during acceptance; Grok 4.6 tool requests succeeded.
Fallback keeps the requested Bot `default` and propagates failure to the outer
configured chain. It does not silently substitute a named Bot model.

Default resolved to `cursor-grok-4.5-high` during live acceptance. Grok 4.6 with
`effort=low` resolved to `cursor-grok-4.6-low-fast`; both actual model identities
were verified in Usage. Upstream may change these resolutions.

The release used a clean child worktree so unrelated local Antigravity edits and
`docs/codex-turn-state.md` were neither committed nor deployed. Credentials and
raw acceptance payloads remain outside the repositories.
