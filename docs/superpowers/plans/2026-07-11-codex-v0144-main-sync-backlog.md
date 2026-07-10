# Codex v0.144 / Main API Sync Backlog

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Each behavior change must start with a focused failing test and finish with the affected-crate clippy gate.

**Goal:** Fix the production Codex 429/reset-credit correctness gaps, then synchronize StaticFlow's Codex API behavior with Codex `rust-v0.144.1` and current `main` without prematurely exposing models whose transport contract the gateway cannot satisfy.

**Audit baseline:** `/home/ts_user/rust_pro/codex` was clean and exactly matched upstream `main` at `d2d00b6632dc991aa4471db0529773029cae5d68` on 2026-07-11. The latest stable tag observed was `rust-v0.144.1` (`44918ea10c0f99151c6710411b4322c2f5c96bea`). StaticFlow still defaults to Codex client `0.142.0` and carries the v0.142 bundled catalog.

**Architecture:** Treat a pinned, audited upstream model catalog as the source of model capabilities, not just a display list. Load its transport profiles before request normalization; account-specific live catalogs may narrow visibility but must not grant capabilities absent from the pinned snapshot. Routing and cooldown decisions must consume explicit catalog/upstream metadata. Keep legacy public aliases as an explicit compatibility overlay. Separate account-wide failures from model-limit failures, and separate read-only reset-credit discovery from explicit, idempotent consumption.

**Tech Stack:** Rust, Axum, Reqwest, Yew/WASM, Postgres/Neon, existing `llm-access-*` crates.

---

## Release Invariants

- [ ] Do not raise the effective `codex_client_version` above `0.142.0` until the Tasks 4, 6, 7, and 8 protocol gate is verified end to end.
- [ ] Do not advertise `gpt-5.6-*` merely by copying the new catalog; model visibility and request capability must switch atomically.
- [ ] Preserve current `gpt-5.3-codex` / Spark userspace through an explicit legacy alias policy. Do not claim that legacy entry is part of the upstream catalog.
- [ ] Never infer a quota bucket from a model-name substring. Use `x-codex-active-limit`, catalog metadata, or explicit runtime configuration.
- [ ] Reset credits remain manual-only. Background usage refresh must never call a consume endpoint.
- [ ] A Responses stream is successful only after `response.completed`; `response.incomplete`, timeout, or EOF before completion is an error.
- [ ] Release only the AWS cloud `llm-access` plane for these changes. Do not rebuild or restart local Pingora/StaticFlow unless a separate non-LLM change requires it.

## Priority and Dependency Order

| Priority | Work item | Activation dependency |
|---|---|---|
| P0 | Model-limit-aware 429 routing and exact cooldowns | None; fixes the observed outage |
| P0 | Idempotent, auditable reset-credit consumption | None; protects a scarce destructive action |
| P0 | Strict Responses stream terminal-state handling | None; fixes false-success accounting |
| P0 | Responses Lite request/header support | Required before 5.6 visibility/version bump |
| P1 | Catalog/client `0.144.1` activation and 5.6 models | Tasks 4, 6, 7, and 8 pass as one protocol gate |
| P1 | `max`/`ultra`, reasoning defaults, `stream_options`, `bio_policy` | Can ship with the activation release |
| P2 | Model cache, adapter completeness, heuristic removal | Usage evidence and compatibility review |

Recommended rollout:

1. Ship P0 correctness/safety fixes while retaining client `0.142.0` and the current public catalog.
2. Ship Responses Lite, the unified reasoning normalizer, optional protocol fields/errors, and stable version identity behind catalog-derived capability handling; keep 5.6 hidden.
3. Raise the effective backend/frontend/Neon runtime client version to `0.144.1`, then expose only models whose capability profile is supported.
4. Observe per-model success/429/incomplete rates before removing any legacy mapping.

## Audited Source Delta

| Surface | StaticFlow now | Codex `main` at `d2d00b6632` | Required decision |
|---|---|---|---|
| Model catalog | 6 entries; includes local `gpt-5.3-codex` | 8 entries; adds `gpt-5.6-sol`, `gpt-5.6-terra`, `gpt-5.6-luna` and no longer lists 5.3 | Import by pinned commit, retain 5.3 only as an explicit compatibility overlay |
| 5.6 transport | Unsupported | All three use Responses Lite and `code_mode_only`, minimum client `0.144.0` | Implement Task 4 before visibility |
| 5.6 reasoning | Unsupported | Sol/Terra allow `max` and `ultra`; Luna allows `max` | Drive validation from each catalog entry |
| Reasoning wire value | `max -> xhigh`; unknown/`ultra` can fall back to `medium` | `max -> max`; user `ultra` is sent as wire `max` | Use one shared open normalizer |
| Reset credits | Count only; one-click generic consume | Detail list, optional `credit_id`, caller-owned idempotency key, Cancel-first picker | Implement Task 2 |
| Responses stream | EOF/incomplete can reach local success handling | Only `response.completed` is success | Implement Task 3 |
| Optional protocol | Drops `stream_options`; misses `bio_policy` terminal classification | Supports sequential-cutoff summaries; treats `bio_policy` as InvalidRequest | Implement Task 7 |

Pin the imported model catalog to the audited upstream commit in the update commit message or generated-file provenance. Do not silently track an unreviewed moving `main` file.

---

## Task 1: Make 429 Routing Limit-Aware

**Observed failure:** An account may have general quota while the Spark-specific `codex_bengalfox` bucket is exhausted. Current routing ranks only the general account summary, classifies quota failures without retaining the active limit/reset time, and applies a fixed account-wide cooldown. One model-specific 429 can therefore freeze healthy models on that account.

**Files:**

- Modify `crates/llm-access/src/provider/codex_upstream_error.rs`
- Modify `crates/llm-access/src/provider/limiter.rs`
- Modify `crates/llm-access/src/provider/codex_dispatch.rs`
- Modify `crates/llm-access/src/provider/route_selection.rs`
- Modify `crates/llm-access/src/provider/errors.rs`
- Modify `crates/llm-access-store/src/postgres/codex_routing.rs`
- Modify related types under `crates/llm-access-core/src/store/`

- [ ] Extend classified quota errors with `limit_id`, `limit_name`, selected primary/secondary reset timestamp, and upstream `Retry-After`.
- [ ] Parse the full `x-codex-*rate-limit*` header family and `x-codex-active-limit`, matching upstream `codex-api/src/rate_limits.rs` semantics.
- [ ] Parse both delta-seconds and HTTP-date `Retry-After`; never cap it to 30 seconds during classification.
- [ ] Never retry before `Retry-After`. If the server-side retry budget is shorter, return the original rate-limit response instead of sleeping for a random shorter interval.
- [ ] Key quota cooldowns by `(account_id, limit_id)`; keep transport, auth, and account-disabled cooldowns account-wide.
- [ ] Carry the request's resolved limit identity into route selection using explicit catalog/header/config metadata.
- [ ] Rank candidate accounts by the matching model-limit bucket before the generic account summary.
- [ ] Return a structured OpenAI-compatible 429 with `Retry-After` and a stable local reason code when every compatible route is cooling down.
- [ ] Record local route rejections in usage/audit diagnostics, distinguishing upstream quota, model-bucket cooldown, account cooldown, key concurrency, and account concurrency.
- [ ] Add tests proving an exhausted Spark bucket does not block `gpt-5.4`/`gpt-5.5`, while another Spark-capable account is still selected.
- [ ] Add tests for HTTP-date `Retry-After`, exact reset selection, and no early retry.

**Evidence anchors:**

- Local routing: `crates/llm-access-store/src/postgres/codex_routing.rs:20`
- Local classifier/retry: `crates/llm-access/src/provider/codex_upstream_error.rs:52`, `crates/llm-access/src/provider/errors.rs:85`
- Local cooldown/dispatch: `crates/llm-access/src/provider/limiter.rs:69`, `crates/llm-access/src/provider/codex_dispatch.rs:937`
- Upstream headers: `../codex/codex-rs/codex-api/src/api_bridge.rs:85`, `../codex/codex-rs/codex-api/src/rate_limits.rs:27`

## Task 2: Make Reset-Credit Consumption Explicit, Idempotent, and Auditable

**Observed behavior:** Current background refresh only reads usage; no automatic reset-credit consumer was found. The risky path is the admin UI: it offers a one-click consume action, generates the idempotency UUID inside each backend request, does not select a specific credit, and has no durable attempt ledger. A successful upstream consume followed by a lost browser response can make the next click consume another credit.

**Files:**

- Modify `crates/llm-access-core/src/store/codex_status.rs`
- Add `crates/llm-access-core/src/store/codex_reset_credit.rs`
- Modify `crates/llm-access-core/src/store/traits.rs`
- Modify `crates/llm-access-store/src/postgres.rs` and its Codex store modules
- Add `crates/llm-access-migrations/migrations/postgres/0046_codex_reset_credit_attempts.sql`
- Modify `crates/llm-access/src/codex_status.rs`
- Modify `crates/llm-access/src/admin.rs`
- Modify `crates/frontend/src/api.rs`
- Modify `crates/frontend/src/pages/admin_llm_gateway_accounts.rs`

- [ ] Add an on-demand admin endpoint that lists reset-credit details only when the picker opens. Preserve upstream fields: id, reset type, status, granted/expiry timestamps, title, and description.
- [ ] Change consume input to `{ idempotency_key, credit_id? }`; validate both as bounded non-empty identifiers.
- [ ] Generate the idempotency key in the UI once per logical attempt and reuse it across browser retry, backend 401 token refresh, timeout retry, and page-level retry recovery.
- [ ] Send the selected `credit_id` when present; retain the legacy no-id form only when upstream returns no selectable detail.
- [ ] Add a confirmation picker whose default selection is Cancel and which displays expiry/details before consumption.
- [ ] Persist one attempt row per idempotency key with account id, optional credit id, actor/admin fingerprint, client IP, request/trace/upstream request ids, start/result timestamps, result code, returned windows, and sanitized error.
- [ ] Enforce a unique database constraint on `idempotency_key`; a duplicate local request must return/reconcile the original attempt instead of issuing a second upstream POST.
- [ ] Never store bearer/admin tokens or raw secrets in the audit row.
- [ ] Add tests for lost response followed by retry, 401 refresh reusing the same key, concurrent duplicate submissions, selected-credit forwarding, and success/failure audit persistence.
- [ ] Add an admin-readable attempt-history view only after the write-path invariants are covered; keep it paginated.

**Evidence anchors:**

- Local UUID/consume: `crates/llm-access/src/codex_status.rs:1166`
- Local admin/UI trigger: `crates/llm-access/src/admin.rs:2405`, `crates/frontend/src/pages/admin_llm_gateway_accounts.rs:2410`
- Upstream detail/consume client: `../codex/codex-rs/backend-client/src/client/rate_limit_resets.rs:37`
- Upstream contracts/UI safety: `../codex/codex-rs/app-server-protocol/src/protocol/v2/account.rs:308`, `../codex/codex-rs/tui/src/chatwidget/usage.rs:159`

## Task 3: Require a Real Responses Completion Event

**Files:**

- Modify `crates/llm-access/src/provider/codex_sse.rs`
- Modify `crates/llm-access/src/provider/codex_dispatch.rs`
- Modify relevant response adapter tests in `crates/llm-access-codex/src/response.rs`

- [ ] Introduce an explicit stream terminal state: pending, completed, incomplete, failed.
- [ ] Treat `response.incomplete` as an upstream error and retain its incomplete reason/code.
- [ ] Treat EOF or timeout before `response.completed` as an error for both streamed and force-streamed/non-streaming requests.
- [ ] Do not emit Chat `[DONE]`, mark usage successful, or save a recovery anchor on an incomplete stream.
- [ ] Keep native Responses events byte-compatible on the success path.
- [ ] Test `response.created + delta + EOF`, explicit `response.incomplete`, timeout, and completed happy paths for Responses, Chat, and Anthropic adapters.

**Evidence anchors:**

- Local false-success paths: `crates/llm-access/src/provider/codex_sse.rs:66`, `crates/llm-access/src/provider/codex_dispatch.rs:2243`
- Upstream terminal rules: `../codex/codex-rs/codex-api/src/sse/responses.rs:422`, `../codex/codex-rs/codex-api/src/sse/responses.rs:515`

## Task 4: Stage the Pinned Catalog and Implement Responses Lite

**Why this gates 5.6:** Current 5.6 catalog entries use `use_responses_lite=true`. Lite is not the normal Responses body with one extra header: it changes instructions, tools, input, reasoning context, and parallel-tool behavior. StaticFlow currently overwrites native instructions with its legacy global prompt and drops the Lite header.

**Files:**

- Replace `crates/llm-access-codex/codex_models.json` from upstream commit `d2d00b6632`
- Modify `crates/llm-access-codex/src/models.rs`
- Modify `crates/llm-access-codex/src/request/prepare.rs`
- Modify `crates/llm-access-codex/src/request/native_responses.rs`
- Modify `crates/llm-access-codex/src/request/chat_completions.rs`
- Modify `crates/llm-access-codex/src/anthropic_messages.rs`
- Modify `crates/llm-access/src/provider/codex_auth.rs`
- Modify `crates/llm-access/src/provider/codex_models.rs`

- [ ] Import the audited catalog fields by pinned commit, then retain `gpt-5.3-codex`/Spark only through an explicit, tested compatibility overlay.
- [ ] Filter every public model/catalog response by effective client version and implemented transport capability. With the runtime still at `0.142.0`, staging this file must not expose any 5.6 entry.
- [ ] Parse a compact `CodexModelTransportProfile` from the pinned bundled catalog at service start. Include `use_responses_lite`, `tool_mode`, supported reasoning efforts, default reasoning effort, base instructions, and context limits.
- [ ] Resolve the pinned transport profile before route/account selection and request normalization; do not branch on hard-coded `gpt-5.6-*` strings. Account-specific live catalogs may only narrow model visibility.
- [ ] For Lite native Responses, do not inject the legacy global instructions or top-level tools.
- [ ] On native official-client requests, validate and preserve existing Lite `additional_tools`/developer input items without adding a second copy. Synthesize each prefix exactly once only for Chat/Anthropic adapter requests, matching upstream order.
- [ ] Set `reasoning.context = "all_turns"` and `parallel_tool_calls = false` for Lite without overwriting explicit compatible client state.
- [ ] After resolving the trusted profile, have the gateway set/overwrite `x-openai-internal-codex-responses-lite` on Responses and compact calls. Delete or ignore any inbound value; this header is client-generated, not an upstream response header.
- [ ] Apply the same model profile to Chat and Anthropic conversions so all public surfaces produce the same upstream Lite contract.
- [ ] Decide whether `/v1/alpha/search` is required by the supported Lite/code-mode flow. If required, add it as an explicit path with tests; otherwise document and reject it, rather than silently proxying an unknown path.
- [ ] Test exact standard 5.5 body/header stability and exact Lite native/Chat/Anthropic/compact body and header shapes.
- [ ] Keep WebSocket disabled; Lite does not authorize a partial WebSocket implementation.

**Evidence anchors:**

- Local instruction overwrite/header allowlist: `crates/llm-access-codex/src/request/native_responses.rs:19`, `crates/llm-access/src/provider/codex_auth.rs:329`
- Upstream Lite body/header: `../codex/codex-rs/core/src/client.rs:836`, `../codex/codex-rs/core/src/client.rs:1897`
- Upstream search endpoint: `../codex/codex-rs/codex-api/src/endpoint/search.rs:31`

## Task 5: Activate the v0.144 Catalog and Client Version

**Files:**

- Modify `crates/llm-access-core/src/store/mod.rs`
- Add the next Postgres migration after Task 2 to update only the known `0.142.0` runtime value to `0.144.1`
- Modify `crates/frontend/src/api.rs`
- Modify `crates/frontend/src/pages/llm_access_guide.rs`

- [ ] Verify the staged catalog exposes `gpt-5.6-sol`, `gpt-5.6-terra`, and `gpt-5.6-luna`, with their distinct reasoning-effort/capability profiles, only after the effective version changes.
- [ ] Keep unsupported or minimum-version-incompatible models hidden in both `/v1/models` and the downloadable catalog.
- [ ] Keep `gpt-5.3-codex`/Spark as a clearly tested compatibility overlay until measured usage permits removal.
- [ ] Change backend/frontend defaults and the live Neon runtime config together; verify the migration only updates the expected old value.
- [ ] Smoke-test `/v1/models`, native Responses, Chat, Anthropic, compact, and per-model routing for every newly visible model before production rollout.

**Activation gate:** This task must remain unchecked until Tasks 4, 6, 7, and 8 pass as one protocol gate across native/Chat/Anthropic/compact paths. Copying the catalog or changing the database version alone is not a valid implementation.

## Task 6: Synchronize Reasoning Semantics

**Files:**

- Modify `crates/llm-access-codex/src/request/normalization.rs`
- Modify `crates/llm-access-codex/src/anthropic_messages.rs`
- Modify `crates/llm-access-codex/src/request/chat_completions.rs`
- Review `crates/llm-access/src/provider/codex_auth.rs`

- [ ] Replace duplicated effort mappings with one open normalizer used by native, Chat, and Anthropic paths.
- [ ] Preserve `max` as `max`; map user-facing `ultra` to wire `max`; preserve future non-empty efforts unless the selected model profile explicitly rejects them.
- [ ] Keep only documented legacy aliases and test them explicitly.
- [ ] For arbitrary native Responses clients missing reasoning state, add `reasoning: {}` plus encrypted-content include only when required by the selected model/client contract; never hard-code `medium`.
- [ ] Never overwrite an explicit compatible reasoning effort/include supplied by the official client.
- [ ] Audit the `invalid_encrypted_content` retry that strips encrypted state. Prefer session/account affinity; do not silently retry with degraded conversation semantics.
- [ ] Add a table-driven cross-endpoint test matrix for none/minimal/low/medium/high/xhigh/max/ultra/future effort values. Let the selected model profile accept or structurally reject each value.

**Evidence anchors:**

- Local wrong mappings: `crates/llm-access-codex/src/request/normalization.rs:98`, `crates/llm-access-codex/src/anthropic_messages.rs:100`
- Upstream effort/wire behavior: `../codex/codex-rs/protocol/src/openai_models.rs:40`, `../codex/codex-rs/core/src/client.rs:174`
- Upstream always-reasoning request: `../codex/codex-rs/core/src/client.rs:803`

## Task 7: Synchronize Optional Request and Error Semantics

**Files:**

- Modify `crates/llm-access-codex/src/request/mod.rs`
- Modify `crates/llm-access-codex/src/request/normalization.rs`
- Modify `crates/llm-access/src/provider/codex_upstream_error.rs`
- Modify `crates/llm-access/src/provider/codex_stream_error.rs`
- Modify the corresponding provider stream-error tests

- [ ] Allow the known `stream_options.reasoning_summary_delivery = "sequential_cutoff"` value instead of deleting all `stream_options`.
- [ ] Preserve it for native requests and emit it from adapters only when selected by an explicit feature/capability.
- [ ] Treat `bio_policy` as a terminal InvalidRequest code in both preflight and mid-stream error paths.
- [ ] Carry the original upstream safety machine code through classified errors and synthesized `response.failed` events; do not collapse `invalid_prompt`/`bio_policy` to null or an unrelated generic code.
- [ ] Preserve the upstream machine code/body for clients; do not fail over or cool down a healthy account for this request error.
- [ ] Test request preservation plus byte-compatible `response.reasoning_summary_text.done` forwarding.
- [ ] Test that `bio_policy` performs no retry, account switch, or cooldown.

## Task 8: Send Stable Codex Client Identity

**Files:**

- Modify `crates/llm-access-codex/src/request/normalization.rs`
- Modify `crates/llm-access/src/provider/codex_auth.rs`
- Modify `crates/llm-access/src/provider/codex_models.rs`
- Modify `crates/llm-access/src/codex_status.rs`

- [ ] Generate a server-owned `version: <effective_codex_client_version>` header for Responses, compact, models, usage, reset-credit detail, and consume calls.
- [ ] Do not forward an inbound client-controlled `version` as the gateway's identity.
- [ ] Keep the existing originator, session/thread, account, and FedRAMP headers.
- [ ] Keep the current simple User-Agent unless a real operational need requires OS metadata; do not fabricate a Codex host OS.
- [ ] Add request-capture tests proving all Codex upstream endpoints use the same effective version.

## Task 9: Close Adapter and Model-Policy Gaps Without Breaking Userspace

**Files:**

- Modify `crates/llm-access-codex/src/response.rs`
- Modify `crates/llm-access-codex/src/request/normalization.rs`
- Modify `crates/llm-access-codex/src/models.rs`

- [ ] Preserve `custom_tool_call.namespace` through the Chat compatibility adapter, or reject the unsupported conversion explicitly; never silently discard it.
- [ ] Decide and document how Chat surfaces expose `tool_search_call`/`tool_search_output`; native Responses already preserves them.
- [ ] Instrument current uses of the fallback that maps every non-`gpt-*` model to `gpt-5.5`.
- [ ] Replace that heuristic with exact catalog validation plus explicit compatibility aliases only after confirming which existing callers depend on it.
- [ ] Test unknown/future/custom model behavior and return a structured error instead of silently billing a different model.

## Task 10: Cache Model Catalogs Safely

**Files:**

- Modify `crates/llm-access/src/provider/codex_models.rs`
- Reuse the runtime cache pattern already present in `llm-access`; do not introduce a second cache service.

- [ ] Keep request transport profiles sourced from the pinned bundled catalog established in Task 4; this cache is only for account-specific visibility/model responses.
- [ ] Add a bounded ETag/TTL cache (upstream default is 300 seconds) keyed by account/plan scope, effective client version, and alias policy.
- [ ] Revalidate with ETag and retain the last successful entry only for the defined TTL.
- [ ] Do not share a catalog across accounts if upstream plan/model visibility differs.
- [ ] Preserve response ETag semantics and add hit/miss/revalidation metrics.
- [ ] Test account isolation, version-key isolation, ETag 304, expiry, and failed refresh behavior.

---

## Explicitly Out of Scope / No Sync Needed

- Responses WebSocket: the provider is intentionally configured with `supports_websockets = false`. Do not add an incomplete Upgrade proxy.
- Client-only proxy factories, external-auth orchestration, agent assertion/attestation, TUI state, and managed-layer behavior: these are not part of the stored-account gateway data plane.
- Image generation routes: StaticFlow already handles image generations/edits through the dedicated Codex image path.
- Generic native Responses item rewriting: web/image/unknown items are already opaque on the native success path; add adapters only where a compatibility surface would otherwise drop data.
- Interleaved response support: upstream reverted it; do not resurrect it locally.
- Automatic reset-credit use: explicitly forbidden by this plan.

## Verification and Release Checklist

- [ ] Before each Rust build, confirm no other `cargo`, `rustc`, `trunk`, linker, or formatter job is active.
- [ ] Set `CARGO_TARGET_DIR=/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow`.
- [ ] Run focused red/green tests for each task before broad affected-crate tests.
- [ ] Run `rustfmt` on the exact changed Rust files only, after checking both Lance submodules are clean.
- [ ] Run affected-crate tests for `llm-access-codex`, `llm-access-core`, `llm-access-store`, `llm-access`, and frontend logic as applicable.
- [ ] Run affected-crate `cargo clippy -- -D warnings` with zero warnings.
- [ ] Verify migration application and rollback assumptions against a disposable Postgres database before Neon.
- [ ] Canary against upstream with client `0.142.0`, then Lite-hidden `0.144.1`, then enabled 5.6 catalog.
- [ ] Compare success, upstream/local 429, retry, incomplete-stream, account-switch, and reset-credit audit metrics.
- [ ] Release the API with `scripts/release_llm_access_cloud_api_only.sh`; restart the usage worker only if its own code/schema contract changed.

## Audit Trail

The source comparison covered model/catalog/version negotiation, Responses/compact bodies and headers, Chat/Anthropic adapters, SSE terminal/error events, rate-limit headers, reset-credit detail/consume APIs, reasoning efforts, tool item variants, session identity, and existing image routes. Relevant upstream changes include Responses Lite (`33cc928d33`), reset-credit details (`58ec5283156c`), Ultra/Max reasoning (`df1199fddb`, `80f54d1266`), sequential reasoning-summary delivery (`775ef7dcc7`), `bio_policy` (`78df1237d1`), namespaced custom tools (`328e95110c`), and always-present Responses reasoning parameters (`d2d00b6632`).
