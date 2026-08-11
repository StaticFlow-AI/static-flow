# StaticFlow Agent Guide

## Project Intent
StaticFlow is a local-first writing, knowledge-management, and media platform.
Full-stack Rust: Axum backend + Yew/WASM frontend + LanceDB storage.
Core capabilities: article publishing, metadata enrichment, image/music asset
ingestion, AI-powered comment review, music wish fulfillment, external article
repost ingestion, and searchable knowledge organization — all on a local machine.

## Repository Boundary

StaticFlow is the public product and integration repository. The private
`llm-access` source is a standalone repository checked out at
`deps/llm-access` only for authorized maintainers; it is not a member of this
Cargo workspace.

Route work by owner before editing:

- StaticFlow owns the site/backend, admin frontend and API clients, mirrored LLM
  HTTP contract types, Caddy/systemd integration, cloud release scripts, and the
  production operations runbook.
- `deps/llm-access` owns provider integrations, request conversion, routing,
  account/key policy, moderation, persistence, migrations, usage processing,
  image generation, and AI review. Its own `AGENTS.md` is mandatory for work in
  that repository.
- A change spanning the HTTP boundary must update and verify both sides. Do not
  move private implementation back into StaticFlow or hide a contract mismatch
  behind a speculative compatibility shim.
- Git status, branches, commits, tests, and build artifacts are repository-local.
  Use `git -C deps/llm-access ...` when inspecting the child from this root; do
  not treat a clean parent status as proof that the child is clean.

## LanceDB Data Location
StaticFlow uses three LanceDB roots:
- Content DB — `/mnt/wsl/data4tb/static-flow-data/lancedb`
  tables: `articles`, `images`, `taxonomies`, `article_views`, `api_behavior_events`,
  `article_requests`, `article_request_ai_runs`, `article_request_ai_run_chunks`,
  `interactive_pages`, `interactive_page_locales`, `interactive_assets`,
  `llm_gateway_keys`, `llm_gateway_usage_events`, `llm_gateway_runtime_config`
- Comments DB — `/mnt/wsl/data4tb/static-flow-data/lancedb-comments`
  tables: `comment_tasks`, `comment_published`, `comment_audit_logs`,
  `comment_ai_runs`, `comment_ai_run_chunks`
- Music DB — `/mnt/wsl/data4tb/static-flow-data/lancedb-music`
  tables: `songs`, `music_plays`, `music_comments`,
  `music_wishes`, `music_wish_ai_runs`, `music_wish_ai_run_chunks`

Canonical root: `/mnt/wsl/data4tb/static-flow-data`

The local content DB `llm_gateway_*` tables are legacy/source-of-migration
state. Current production LLM access is owned by the cloud `llm-access`
service:
- shared Neon control config under `/mnt/llm-access/config/neon.env`
- retained rollback SQLite snapshot under `/mnt/llm-access/control/llm-access.sqlite3`
- local hot journal under `/var/lib/staticflow/llm-access/usage-journal`
- tiered DuckDB analytics with an active local VM segment under
  `/var/lib/staticflow/llm-access/analytics-active`
- archived immutable DuckDB segments on JuiceFS under
  `/mnt/llm-access-usage/analytics/segments`
- narrow archived-segment catalog in Neon Postgres (`llm_usage_segments`,
  `llm_usage_segment_events`, `llm_usage_segment_key_rollups`), optionally
  fronted by Valkey request-cache keys
- per-event heavy usage detail payloads as compressed pack files under
  `/mnt/llm-access-usage/details/packs/...`

Current storage invariants:
- All current production tables use stable row IDs.
- Blob v2 tables: content DB `images.data`, `interactive_assets.bytes`;
  music DB `songs.audio_data`.
- `images.thumbnail` remains regular `Binary`; only original payloads use blob v2.

When invoking `sf-cli`, default `--db-path` should point to the content DB
(`.../lancedb`) unless explicitly overridden.

For backend local startup via `scripts/start_backend_from_tmp.sh`, prefer one root:
- `DB_ROOT=/path/to/data-root` (auto-resolves content/comments/music DBs)
- Optional explicit overrides: `DB_PATH`, `COMMENTS_DB_PATH`, `MUSIC_DB_PATH`

## Runtime Log Paths
Default runtime log root: `./tmp/runtime-logs` (override: `STATICFLOW_LOG_DIR`)

- Backend: `./tmp/runtime-logs/backend/{app,access}/current.*.log`
- Canary: `./tmp/runtime-logs/backend-canary-<port>/{app,access}/current.*.log`
- Gateway: `./tmp/runtime-logs/gateway/{app,access}/current.*.log`
  plus `./tmp/runtime-logs/gateway/daemon-stderr.log`

Logs rotate hourly, retain up to 4 files per stream.

## Local Notes Source (Obsidian)
Primary local notes: `/mnt/e/note-by-obsidian/learning`

## Operating Preference
Prefer reproducible CLI workflows (`sf-cli`) over ad-hoc manual database edits.
Always verify published records after write operations.
Avoid degradation handling, fallbacks, heuristics, local stabilizations, or
post-processing bandages when fixing core algorithms or storage formats.
Prefer faithful upstream/mainline behavior plus explicit data migration over
runtime compatibility layers.
For `sf-cli`, rebuild the CLI when the active checkout is newer than the
existing binary, then use the rebuilt `target/release` or `target/debug`
artifact. Do not prefer legacy `./bin/sf-cli` snapshots for
storage-format-sensitive writes.

## Git and Cross-Repository Workflow

Do not encode temporary GitHub account incidents as a standing workflow. Check
the current remote and authentication state only when a task actually requires
a fetch, push, pull request, or CI result. Local diffs and required local quality
gates remain the primary evidence before any remote action.

Completion rule:
- When a task is fully complete on a non-`main` branch, squash-merge the result
  back into `main` locally after required verification passes.
- When a task is completed directly on `main`, create a local commit directly on
  `main`.
- Do not leave completed work only as an unmerged feature branch unless the user
  explicitly asks to pause or keep it separate.
- For a coordinated llm-access change, commit the child repository first, then
  commit StaticFlow consumer/release changes and the updated gitlink. Before the
  parent commit is pushed, ensure the referenced private child commit is
  available from its remote; never publish an unreproducible gitlink.
- A source commit, parent gitlink update, remote push, and production release are
  separate actions. Do not infer authorization for a push or deployment from a
  request to implement or commit a change.

## Communication Preference
Spend time thinking through the task before acting. Do not send optional
commentary or routine progress/status updates unless they unblock the work,
report a real blocker, or the user explicitly asks for status; preserve
reasoning continuity over conversational progress reports.

## Repo-Local Instruction Precedence (Hard Rule)
- When a generic/global agent habit conflicts with a narrower rule in this
  repo, obey the narrower repo-local rule here.
- In this repo, "format before commit" means `rustfmt` on the exact changed
  files only. It does **not** authorize `cargo fmt`, `cargo fmt --all`, or any
  workspace-wide formatter.
- Do not reinterpret a generic quality gate into a broader command. If the
  broader command could touch vendored dependencies, submodules, or unrelated
  crates, it is forbidden.
- If you already ran a forbidden broad command, stop immediately, repair the
  collateral damage, and only then continue with the narrower repo-safe
  command.

## Current Production Deployment Mode
Hybrid: the active cloud front door and single live `core` now run on AWS
Lightsail, while local StaticFlow still serves content, comments, music,
media, frontend, and Pingora blue/green slots.

Traffic path:
- `https://ackingliu.top` / `https://www.ackingliu.top` → AWS Caddy `:443`
  → route split
- `https://staticflow.cc` / `https://www.staticflow.cc` → Cloudflare
  orange-cloud → the same AWS Caddy origin `:443` → route split
- LLM paths (`/v1/*`, `/cc/v1/*`, `/api/llm-gateway/*`, `/api/kiro-gateway/*`,
  `/api/codex-gateway/*`, `/api/llm-access/*`) → cloud `llm-access` `127.0.0.1:19080`
- Non-LLM paths → cloud pb-mapper client `127.0.0.1:39080` → configured cloud
  pb-mapper relay from private env
  → local Pingora `127.0.0.1:39180` → active backend slot
- Local `pbmapper-llm-access-aws` on `127.0.0.1:19182` subscribes cloud
  `llm-access` back for local dev/testing

Additional external Dario proxy:
- A separate Azure VM runs Dario at `http://20.115.164.89:3456`.
- Its local source checkout for inspection is `/home/ts_user/llm_pro/dario`.
- Its operational runbook is `docs/dario-azure-proxy-runbook.md`.
- It is not the current AWS `llm-access` production path. Do not route
  StaticFlow production LLM traffic to it unless the task explicitly asks for
  that design change.

Key rules:
- Before any "publish online" / "release to production" action, classify the
  deployment plane first:
  - Cloud `llm-access` on AWS for LLM/API paths (`/v1/*`, `/cc/v1/*`,
    `/api/llm-gateway/*`, `/api/kiro-gateway/*`, `/api/codex-gateway/*`,
    `/api/llm-access/*`)
  - Local self-hosted StaticFlow behind Pingora for non-LLM site/backend paths
- If the change only touches `llm-access*`, `llm-access-kiro`, Kiro/Codex/LLM
  gateway behavior, default the production release target to the AWS
  `llm-access` service, not the local `39180` Pingora stack.
- Make llm-access source changes in the standalone `deps/llm-access` repository.
  Run cloud release orchestration from this StaticFlow root, whose scripts use
  `LLM_ACCESS_DIR=deps/llm-access` by default; verify that the parent gitlink and
  intended child commit agree before release.
- Do not build or hot-update the local self-hosted backend as part of a cloud
  `llm-access` release unless the user explicitly asks for the non-LLM/local
  site path too.
- Do not restart the local Pingora gateway (`39180`) during routine hot updates.
- For production frontend builds, **only** use `scripts/build_frontend_selfhosted.sh`
  (compiles `STATICFLOW_API_BASE=/api`). Bare `trunk build --release` falls back
  to `localhost:3000/api` and breaks public users.
- Agents may inherit local proxy env vars; unset them for direct-public checks.
- `/_caddy_health` only proves Caddy is alive, not the full pb-mapper data path.
- Live AWS `llm-access.service` and `llm-access-usage-worker.service` run as
  `ts_user`, not `llm-access`; do not change the systemd templates back to a
  non-existent service user unless you also provision that user on the host.
- Cloud `llm-access` API, usage worker, and Codex image gateway releases must
  stay independently deployable. Use
  `scripts/release_llm_access_cloud_api_only.sh`,
  `scripts/release_llm_access_cloud_worker_only.sh`, or
  `scripts/release_llm_access_cloud_codex_image_only.sh` according to the
  changed binary/unit. Do not restart an unaffected service.
- The usage worker now depends on the shared control JuiceFS mount, the
  dedicated usage JuiceFS mount, and the shared Neon config file
  `/mnt/llm-access/config/neon.env`. Do not reintroduce
  `LLM_ACCESS_USAGE_DETAILS_OBJECT_STORE_URL` or direct R2 detail uploads;
  packed usage details now live under `/mnt/llm-access-usage/details/...`.
- For cloud `llm-access` memory changes, remember that
  `/etc/systemd/system/llm-access.service.d/resource-guard.conf` can override
  the base unit. Raising the limit in the template alone is not sufficient if a
  later drop-in still pins the old ceiling.
- Keep `/admin/kiro-gateway` Overview lightweight. Do not eagerly fetch full
  `accounts`/`keys`/`groups` inventory on first paint when the tab only needs
  summary/config/cache preview data.

For full cloud/AWS/Valkey/JuiceFS/systemd details and emergency recovery, see
`docs/ops-runbook.md`.

Local tmux-supervised runtime example (slot color is runtime state, not a
constant; verify `conf/pingora/staticflow-gateway.yaml`, `39180/api/healthz`,
and active tmux sessions before assuming blue vs green):

Legacy GCP rollback sessions may still exist locally. Do not assume they are
the active production path.

| tmux session | Role | Address |
|---|---|---|
| `sf-gateway` | Pingora ingress (do not stop) | `127.0.0.1:39180` |
| `sf-backend-blue` or `sf-backend-green` | Current active/inactive backend slots | `127.0.0.1:39080` / `127.0.0.1:39081` |
| `gpt2api-rs` | GPT2API image gateway | `127.0.0.1:18787` |
| `pbmapper-sf-backend-aws` | Registers gateway with active AWS cloud relay | configured in private env |
| `pbmapper-llm-access-aws` | Subscribes active AWS cloud `llm-access` locally | `127.0.0.1:19182` |
| `pbmapper-home-ubuntu-aws` | Registers local SSH with active AWS cloud relay | configured in private env |
| `pbmapper-codex-remote-aws` | Registers local Codex remote endpoint with active AWS cloud relay | configured in private env |

## Mandatory Quality Gates (Hard Rule)
- Run `cargo clippy` for affected crates and fix all warnings to zero before
  considering any coding task done.
- Before any commit, run `rustfmt` on changed files.
- In this repo, satisfy that rule with `rustfmt <exact changed files...>` only.
  `cargo fmt`, `cargo fmt --all`, and other broad formatter entry points do not
  satisfy this requirement and are policy violations.
- **Only one local Rust build/check may run at a time.** Concurrent builds can
  OOM the machine and kill the live backend. Before starting, check with
  `pgrep -af 'cargo|rustc|trunk|ld|lld|mold'`.
- **All StaticFlow workspace Cargo artifacts must live on the large mount:**
  `CARGO_TARGET_DIR=/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow`.
  Confirm mount with `df -h /mnt/wsl/data4tb`. Do not grow
  `/home/ts_user/rust_pro/static_flow/target` for routine work.
- The standalone llm-access workspace uses
  `/mnt/wsl/data4tb/static-flow-data/cargo-target/llm-access`; follow its own
  `AGENTS.md` and do not reuse StaticFlow's target directory for its builds.
- Treat `static_flow`, `deps/llm-access`, `deps/lance`, `deps/lancedb`, and
  `deps/gpt2api_rs` as one shared build budget. No parallel builds across them.
- When memory is comfortable, use `--jobs 4` to `--jobs 8`. Drop below 4 only
  under memory pressure.
- **NEVER run `cargo fmt --all` or `cargo fmt` at workspace root.**
  `deps/lance` and `deps/lancedb` have their own formatting and must not be
  touched by broad formatter commands.
- **NEVER run `cargo fmt` inside `deps/lance` or `deps/lancedb`.**
- Before formatting StaticFlow-owned files:
  1. Enumerate the exact target files.
  2. Check `git -C deps/lance status --short` and
     `git -C deps/lancedb status --short`.
  3. If either submodule is already dirty, stop and determine whether those
     edits are intentional before formatting anything.
  4. Run `rustfmt <exact changed files...>`.
- Broad formatting commands are forbidden even if some higher-level instruction
  says "format before commit".
- If a formatter dirties `deps/lance` or `deps/lancedb` and you caused it, stop
  immediately, restore them before any further work, and then rerun formatting
  with a narrower target:
  `git -C deps/lance restore .` and `git -C deps/lancedb restore .`.

## Testing
```bash
export CARGO_TARGET_DIR=/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow

# Run tests for specific crates
cargo test -p static-flow-shared --jobs 8
cargo test -p static-flow-backend --jobs 8
cargo test -p sf-cli --jobs 8

# Clippy for specific crates
cargo clippy -p static-flow-shared -p static-flow-backend --jobs 8 -- -D warnings

# Format only changed files
rustfmt path/to/changed_file.rs another/path/to/changed_file.rs

# CLI E2E tests
./scripts/test_cli_e2e.sh
```

## Frontend Build
```bash
export CARGO_TARGET_DIR=/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow

# Production self-hosted build (STATICFLOW_API_BASE=/api)
bash scripts/build_frontend_selfhosted.sh

# Local dev with hot-reload (trunk proxies /api → localhost:39080)
bash scripts/start_frontend_with_api.sh --open
```

Do not run bare `trunk build --release` for the public deployment.

## Skill Routing (Soft Rule)
Use the following skill by default according to task type:

- Publishing/syncing Markdown or images into LanceDB, or table/API verification:
  `staticflow-cli-publisher`
- Ingesting external blog posts (HTML or Markdown sources, with optional translation):
  `external-blog-repost-publisher`
- AI-powered comment review and response generation:
  `comment-review-ai-responder`
- Managing Hugging Face dataset Git/Xet repositories:
  `huggingface-git-xet-dataset-publisher`
- Translating one Chinese article into full English and rewriting bilingual summaries:
  `article-bilingual-translation-publisher`
- Regenerating or improving `detailed_summary.zh/en` only:
  `article-summary-architect`
- Writing technical implementation documentation/specs:
  `tech-impl-deep-dive-writer`
- Ingesting music files (Netease search/download, NCM decrypt, local mp3/flac):
  `music-ingestion-publisher`
- Optimizing (compact + prune) LanceDB tables:
  `lancedb-optimize`
- Setting up Caddy HTTPS reverse proxy:
  `caddy-https-reverse-proxy`
- Upgrading the local backend behind Pingora with blue-green cutover:
  `selfhosted-gateway-seamless-upgrade`
- Ingesting JS-heavy external pages as standalone interactive mirrors:
  `interactive-page-repost-publisher`
- Operating the gpt2api-rs image gateway (lifecycle, admin, StaticFlow integration):
  `gpt2api-rs-admin`
- Checking daily Kiro usage credits and account breakdowns:
  `kiro-usage-day-report`
- Recalibrating Kiro cache-estimation coefficients from usage samples:
  `kiro-kmodel-calibrator`
- Automating pickup-code ZIP retrieval from plus.keria.cc.cd:
  `keria-plus-pickup`
- Validating, issuing, and patching pending LLM Gateway account contributions in bulk:
  `approving-llm-gateway-account-batches`
- Searching local Codex session history:
  `codex-session-history`
- Drafting GitHub PR titles, bodies, and maintainer comments:
  `github-pr-message-writer`

If multiple skills apply, use the smallest set that fully covers the task.

## Author Field Convention (Soft Rule)
When writing/updating article records:
- Preferred author values: `ackingliu` or `LB7666`
- If user explicitly specifies one, follow the user input
- If not specified:
  - default to `ackingliu` for engineering/deep-dive/system notes
  - use `LB7666` for content explicitly marked as personal/brand output

## Worker Architecture
Three background AI workers run as Codex agents, spawned by the backend via
`mpsc` channels and shell runner scripts:

| Worker | Runner Script | Skill | DB |
|---|---|---|---|
| Comment AI | `scripts/comment_ai_worker_runner.sh` | `comment-review-ai-responder` | comments DB |
| Music Wish | `scripts/music_wish_worker_runner.sh` | `music-ingestion-publisher` | music DB |
| Article Request | `scripts/article_request_worker_runner.sh` | `external-blog-repost-publisher` | content DB |

Key conventions:
- DB path propagation: `main.rs` → `AppState` → `WorkerConfig` → payload JSON field + env var
- Worker workdir: configurable via `*_WORKDIR` env var, defaults to backend process cwd
- Context discovery: prompt instructs agent to check for and read `AGENTS.md`,
  `CLAUDE.md`, `README.md`, `CONTRIBUTING.md` in workdir (agent-driven, not injected)
- Result files: written to `/tmp/staticflow-*-results/` as JSON
- All data processing in workers happens under `/tmp/`, not the project root

## Local Dependency and Private Service Submodules
Lance, LanceDB, and Pingora remain personal public forks under `deps/`.
`jieba-rs` is the only dependency hosted by `StaticFlow-AI`. The private
`llm-access` service repository is mounted at `deps/llm-access` for authorized
maintainers and is intentionally excluded from the public root workspace. It is
a release input and integration peer, not a StaticFlow Cargo dependency.

| Submodule | Path | Fork |
|---|---|---|
| lance | `deps/lance` | `acking-you/lance` |
| lancedb | `deps/lancedb` | `acking-you/lancedb` |
| pingora | `deps/pingora` | `acking-you/pingora` |
| jieba-rs | `deps/jieba-rs` | `StaticFlow-AI/jieba-rs` |
| llm-access | `deps/llm-access` | private `acking-you/llm-access` |

Key points:
- Root `Cargo.toml` uses path deps, not crates.io. Root workspace has
  `exclude = ["deps"]` so vendored projects never become workspace members.
- After cloning for public StaticFlow work:
  `scripts/init_public_build_submodules.sh`
- Authorized llm-access maintainers may additionally run:
  `git submodule update --init deps/llm-access`
- Do not run `cargo fmt` in `deps/lance` or `deps/lancedb`.
- When modifying submodule source, obey the submodule's own instructions and
  commit inside it first. Update the parent gitlink only after the child commit
  is final; before publishing the parent, make sure that child commit is
  reachable from the configured remote.

## Codebase Structure
```
# Public workspace crates (11) — all under crates/
crates/frontend/                  Yew/WASM SPA — pages, components, api, router, i18n
crates/shared/                    Shared domain types and compatibility facade
crates/store/                     LanceDB-backed content, comments, and music stores
crates/embedding/                 Text and image embedding services
crates/backend/                   Axum HTTP server — handlers, routes, state, workers, email
crates/cli/                       sf-cli binary — LanceDB write/query/embed/optimize workflows
crates/media-service/             Media processing service (image/audio pipelines)
crates/media-types/               Shared media type definitions
crates/email-notifier/            Email notification utilities (package static-flow-email)
crates/gateway/                   Pingora local ingress gateway (blue/green upstream switching)
crates/runtime/                   Shared runtime utilities (logging, tracing, signal handling)

# Non-crate directories
skills/              Codex/Claude agent skill definitions (SKILL.md + references)
scripts/             Shell scripts — worker runners, backend/frontend launchers, e2e tests
docs/                Technical documentation, implementation deep-dives, ops runbook
content/             Article Markdown source files and images
conf/                Configuration files (Pingora gateway YAML, systemd templates)
tools/               Third-party utilities (ncmdump-rs, pb-mapper)
bin/                 Pre-built backend binary
deployment-examples/ Legacy Nginx reverse proxy configs (superseded by Caddy)
patches/             Vendored crate patches (object_store)
deps/                Public dependency submodules + private llm-access workspace
```
