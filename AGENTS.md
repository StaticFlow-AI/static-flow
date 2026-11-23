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

- StaticFlow owns the site/backend, public frontend consumers, the legacy admin
  handoff, mirrored LLM HTTP contract types, Caddy/systemd integration, cloud
  release scripts, and the production operations runbook.
- `deps/llm-access` owns provider integrations (Codex, Kiro, Cursor/Grok,
  Antigravity, OpenCode), request conversion, fallback-graph routing,
  account/key policy, moderation, Neon Postgres persistence + Valkey request
  caching, migrations, usage journal & tiered DuckDB analytics, Codex image
  generation, loopback OAuth management, Linux host/process resource monitoring,
  AI review, and the standalone `apps/llm-access-frontend` operations console.
  Its own `AGENTS.md` is mandatory for work in that repository.
- A change spanning the HTTP boundary must update and verify both sides. Do not
  move private implementation back into StaticFlow or hide a contract mismatch
  behind a speculative compatibility shim.
- Git status, branches, commits, tests, and build artifacts are repository-local.
  Use `git -C deps/llm-access ...` when inspecting the child from this root; do
  not treat a clean parent status as proof that the child is clean.

## LLM / Multi-Provider Operator Console (Hard Rule)

The live operator console for Codex, Kiro, Cursor/Grok, Antigravity, and
OpenCode keys, accounts, groups, fallback routing graphs, usage, host/process
telemetry, moderation, model pricing, runtime config, and proxies is the React
app in `deps/llm-access/apps/llm-access-frontend` (local `127.0.0.1:19191`,
tmux `sf-llm-access-frontend`). Follow `deps/llm-access/AGENTS.md` for that
work.

Default StaticFlow Yew `/admin/llm-gateway*` and `/admin/kiro-gateway*` routes
are handoff pages into that console. The old Yew admin
(`crates/frontend/src/pages/admin_llm_gateway*.rs`,
`admin_kiro_gateway*.rs`) is mounted only when the parent frontend is built
with `STATICFLOW_ENABLE_LEGACY_LLM_ADMIN=1`.

When the task is creating or editing a provider key, account, fallback route,
or any other llm-access control-plane form:

- Edit `deps/llm-access/apps/llm-access-frontend`:
  - Provider inventories (`/console/{codex,kiro,cursor,antigravity,opencode}/*`):
    `src/console/pages/InventoryPage.tsx`
  - Create dialogs: `TypedActionDialog` / `TypedValueFields` in
    `src/console/components.tsx`
  - Existing-key & fallback-graph edits: `src/console/key-editor.tsx`,
    `src/console/fallback-editor.tsx`, `src/console/fallback-canvas.tsx`
  - Account edits: `src/console/account-editor.tsx`
  - Host resources, runtime settings, model pricing, proxies:
    `src/console/pages/{SystemResourcesPage,SettingsPage,PricingPage,ProxiesPage}.tsx`
- Do not treat the Yew pages as the live UI. A `/admin/kiro-gateway/keys` or
  `/admin/llm-gateway/keys` URL in the browser is a handoff, not the form that
  submits the JSON body.

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
- For a coordinated llm-access change, commit the child repository first. When
  publishing through pull requests, the child PR must pass its required review
  and CI and be merged into the child repository's default branch before the
  StaticFlow gitlink is updated or any parent PR (including a draft) is created.
  A child feature branch or open PR being remotely reachable is not sufficient.
  After the child merge, fetch its default branch and verify that the exact
  referenced child commit is an ancestor of that remote branch; only then
  commit/push the parent consumer, release, and gitlink changes. Never publish
  an unreproducible or not-yet-integrated gitlink.
- A source commit, parent gitlink update, remote push, and production release are
  separate actions. Do not infer authorization for a push or deployment from a
  request to implement or commit a change.

### Active Private `llm-access` Direct-Main Mode

The private-repository GitHub Actions allowance is currently exhausted. Until
the user explicitly says that private Actions capacity has been restored,
completed maintainer work in `deps/llm-access` uses a local-review/direct-main
workflow instead of opening a PR or waiting for GitHub-hosted CI:

- Work on the child repository's `main` branch. Before publishing, perform a
  dedicated local code-review pass over the exact diff for correctness,
  security, provider/API contract alignment, failure behavior, and test
  coverage; resolve every credible finding.
- Run the child repository's required local formatting, tests, and zero-warning
  Clippy gates. The exhausted hosted minutes do not weaken any local quality
  gate.
- Fetch `origin`, confirm that local `main` has not diverged from the remote,
  then commit and push directly to the child `main` when the task includes
  publication. Verify the remote branch tip after the push.
- For coordinated StaticFlow changes, only after the child commit is remotely
  reachable may the parent consumer/gitlink be committed and pushed to its own
  `main`, again after its applicable local review and quality gates.
- This mode applies to the private `llm-access` repository only. Existing PR/CI
  rules remain available for repositories with working hosted CI. It does not
  turn a request to implement or commit into authorization to deploy production.

## Communication Preference
Spend time thinking through the task before acting. Do not send optional
commentary or routine progress/status updates unless they unblock the work,
report a real blocker, or the user explicitly asks for status; preserve
reasoning continuity over conversational progress reports.

## Xiaohongshu Publishing & Copywriting (Hard Rule)
When generating Xiaohongshu (`小红书`) cards/copy or publishing notes via
Creator Center (`xiaohongshu-tech-note-publisher`), strictly prevent
off-platform traffic-diversion throttling (`站外引流限流`):
- **No link mentions or diversion wording in title or body**: Never write
  `个人博客`, `已导入博客`, `全文见博客`, `链接放评论区`, `评论区置顶`, `见评论区`,
  `点击主页`, `私信获取`, or any external URLs/domains in the title or body copy;
  close the post naturally as a self-contained technical note.
- **No domains or diversion text on slide images (OCR gate)**: Never render
  `ackingliu.top`, `staticflow.cc`, any `.top`/`.com`/`.cc`/`.gg` domain, QR
  codes, or phrases like `完整精翻入库` / `链接见评论区` anywhere on slides.
- **Comment-section link format (natural paragraph, no `https://`, no ad tone)**:
  When posting the full-article address in comments, never post a bare URL,
  never include `https://` or `http://`, and avoid promotional/salesy symbols or
  slogans (`🔗`, "速戳", "必看"); weave the protocol-free path
  (`ackingliu.top/posts/<slug>`) naturally inside a calm, conversational
  paragraph noting full translation and clean reading layout.

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
- Main Codex/Kiro LLM paths (`/v1/*`, `/cc/v1/*`, `/api/llm-gateway/*`,
  `/api/kiro-gateway/*`, `/api/codex-gateway/*`, `/api/llm-access/*`)
  → cloud `llm-access` `127.0.0.1:19080`
- Codex image generation/edit paths (`/v1/images/{generations,edits}`,
  `/api/{codex,llm}-gateway/[v1/]images/{generations,edits}`)
  → cloud `llm-access-codex-image` `127.0.0.1:19082`
- Cursor/Grok gateway path (`/api/cursor-gateway/v1/*`, stripped to `/v1/*`)
  → cloud `llm-access-cursor` `127.0.0.1:19090`
- Antigravity gateway path (`/api/antigravity-gateway/v1/*`, stripped to `/v1/*`)
  → cloud `llm-access-antigravity` `127.0.0.1:19095`
- Cloud loopback-only services (never exposed on public Caddy):
  - `llm-access-usage-worker` `127.0.0.1:19081` (journal consumer & usage queries)
  - `llm-access-opencode` `127.0.0.1:19085` (OpenCode decision gateway for Kiro Thinking Guard)
  - `llm-access-monitor` `127.0.0.1:19092` (Linux `/proc` + cgroup v2 + PSI + JuiceFS sampler)
  - `llm-access-oauth` `127.0.0.1:19194` (remote OAuth account login & console grant handoff)
- Non-LLM paths → cloud pb-mapper client `127.0.0.1:39080` → configured cloud
  pb-mapper relay from private env
  → local Pingora `127.0.0.1:39180` → active backend slot (`:39080` / `:39081`)
  + local media service (`:39085`)
- Local operator console (`127.0.0.1:19191`) reaches cloud control planes via
  `pbmapper-llm-access-aws` (`:19182`), `pbmapper-llm-access-cursor-aws`
  (`:19183`), and SSH loopback tunnels for monitor (`:19092`), OAuth (`:19194`),
  Antigravity (`:19195`), and OpenCode (`:19196`)

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
  - Cloud `llm-access*` on AWS for LLM/API paths (`/v1/*`, `/cc/v1/*`,
    `/api/llm-gateway/*`, `/api/kiro-gateway/*`, `/api/codex-gateway/*`,
    `/api/llm-access/*`, `/api/cursor-gateway/v1/*`,
    `/api/antigravity-gateway/v1/*`, plus loopback OpenCode/image/worker/OAuth/monitor)
  - Local self-hosted StaticFlow behind Pingora for non-LLM site/backend/media paths
- If the change only touches `llm-access*`, provider gateways, or operator
  console backend contracts, default the production release target to the
  affected AWS `llm-access*` service, not the local `39180` Pingora stack.
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
- Live AWS `llm-access*.service` units run as `ts_user`, not `llm-access`; do
  not change the systemd templates back to a non-existent service user unless
  you also provision that user on the host.
- Cloud `llm-access` services must stay independently deployable. Use the
  targeted release script for the changed binary/unit and never restart an
  unaffected service:
  - `scripts/release_llm_access_cloud_api_only.sh` (`llm-access.service`, `:19080`)
  - `scripts/release_llm_access_cloud_worker_only.sh` (`llm-access-usage-worker.service`, `:19081`)
  - `scripts/release_llm_access_cloud_codex_image_only.sh` (`llm-access-codex-image.service`, `:19082`)
  - `scripts/release_llm_access_cloud_opencode_only.sh` (`llm-access-opencode.service`, `:19085`)
  - `scripts/release_llm_access_cloud_cursor_only.sh` (`llm-access-cursor.service`, `:19090`)
  - `scripts/release_llm_access_cloud_antigravity_only.sh` (`llm-access-antigravity.service` `:19095` & `llm-access-oauth.service` `:19194`)
- The usage worker now depends on the shared control JuiceFS mount, the
  dedicated usage JuiceFS mount, and the shared Neon config file
  `/mnt/llm-access/config/neon.env`. Do not reintroduce
  `LLM_ACCESS_USAGE_DETAILS_OBJECT_STORE_URL` or direct R2 detail uploads;
  packed usage details now live under `/mnt/llm-access-usage/details/...`.
- For cloud `llm-access` memory changes, remember that
  `/etc/systemd/system/llm-access.service.d/resource-guard.conf` can override
  the base unit. Raising the limit in the template alone is not sufficient if a
  later drop-in still pins the old ceiling.
- Keep the llm-access console Overview and Kiro runtime landing pages
  lightweight (`/console`, `/console/kiro/runtime` in
  `apps/llm-access-frontend`). Do not eagerly fetch full
  `accounts`/`keys`/`groups` inventory on first paint when those pages only
  need summary/config/cache preview data. Key inventory belongs on
  `/console/kiro/keys` and `/console/codex/keys`.

For full cloud/AWS/Valkey/JuiceFS/systemd details and emergency recovery, see
`docs/ops-runbook.md` and `docs/llm-access-monitor.md`.

Local tmux-supervised runtime (restored via `scripts/restore_local_tmux_services.sh`;
slot color is runtime state, not a constant — verify `conf/pingora/staticflow-gateway.yaml`,
`39180/api/healthz`, and active tmux sessions before assuming blue vs green):

Legacy GCP rollback sessions may still exist locally. Do not assume they are
the active production path.

| tmux session | Role | Address |
|---|---|---|
| `sf-gateway` | Pingora ingress (do not stop) | `127.0.0.1:39180` |
| `sf-backend-blue` or `sf-backend-green` | Current active/inactive backend slots | `127.0.0.1:39080` / `127.0.0.1:39081` |
| `sf-media` | Local media streaming/transcode service | `127.0.0.1:39085` |
| `gpt2api-rs` | GPT2API image gateway | `127.0.0.1:18787` |
| `pbmapper-sf-backend-aws` | Registers gateway with active AWS cloud relay | configured in private env |
| `pbmapper-llm-access-aws` | Subscribes active AWS cloud `llm-access` locally | `127.0.0.1:19182` |
| `pbmapper-llm-access-cursor-aws` | Subscribes cloud `llm-access-cursor` admin locally (`--full`) | `127.0.0.1:19183` |
| `llm-access-monitor-aws` | SSH tunnel to cloud `llm-access-monitor` (`--full`) | `127.0.0.1:19092` |
| `sf-llm-access-oauth-aws` | SSH tunnel to cloud `llm-access-oauth` (`--full`) | `127.0.0.1:19194` |
| `sf-llm-access-antigravity-aws` | SSH tunnel to cloud `llm-access-antigravity` (`--full`) | `127.0.0.1:19195` |
| `sf-llm-access-opencode-aws` | SSH tunnel to cloud `llm-access-opencode` (`--full`) | `127.0.0.1:19196` |
| `sf-ai-review` | Local AI review backend for operator console (`--full`) | `127.0.0.1:19190` |
| `sf-llm-access-frontend` | llm-access operations console Vite preview (`--full`) | `127.0.0.1:19191` |
| `pbmapper-llm-access-frontend-aws` | Publishes the local console through the AWS relay (`--full`) | `127.0.0.1:19191` |
| `pbmapper-home-ubuntu-aws` | Registers local SSH with active AWS cloud relay | configured in private env |
| `pbmapper-codex-remote-aws` | Registers local Codex remote endpoint with active AWS cloud relay | configured in private env |
| `antigravity-manager` | Optional local Antigravity Manager (`--with-antigravity`) | `127.0.0.1:8045` |

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

The multi-provider LLM operator console is not this Yew app. Build, test, and
preview it from `deps/llm-access/apps/llm-access-frontend`
(`pnpm typecheck && pnpm test:runtime && pnpm build`, then `pnpm preview` on
`127.0.0.1:19191`).

## Skill Routing (Soft Rule)
Use the following skill (`skills/<name>/SKILL.md`, symlinked under
`.agents/skills` and `.claude/skills`) by default according to task type:

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
- Writing, restructuring, or auditing source-verified original technical blog posts (default blog writing skill):
  `technical-blog-writer`
- Writing mechanism-first technical implementation documentation/specs:
  `tech-impl-deep-dive-writer`
- Writing or reviewing book-length source-verified deep-dive monographs in the author's voice:
  `deep-dive-blog-style`
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
- Multi-provider LLM admin console (create/edit keys, accounts, groups,
  fallback graphs, usage, host resources, moderation, runtime config):
  `deps/llm-access/apps/llm-access-frontend` and `deps/llm-access/AGENTS.md`.
  Not the Yew `/admin/kiro-gateway*` or `/admin/llm-gateway*` pages.
- Checking daily Kiro usage credits and account breakdowns:
  `kiro-usage-day-report`
- Recalibrating Kiro cache-estimation coefficients from usage samples:
  `kiro-kmodel-calibrator`
- Importing local Kiro CLI SQLite credentials or onboarding Kiro accounts:
  `kiro-local-account-importer`, `kiro-social-onboarder`,
  `kiro-github-google-onboarder`, `kiro-aws-builder-id-onboarder`,
  `aws-builder-id-kiro-account-workflow`, `aws-builder-id-security`,
  `kiro-outlook-prism-onboarder`
- Automating pickup-code ZIP retrieval from plus.keria.cc.cd:
  `keria-plus-pickup`
- Submitting or approving pending LLM Gateway account contributions in bulk:
  `submitting-llm-gateway-account-batches`,
  `approving-llm-gateway-account-batches`
- Rotating GitHub/Google account passwords or drafting GitHub suspension appeals:
  `github-password-rotator`, `google-password-rotator`,
  `github-account-suspension-appeal`
- Searching local Codex session history:
  `codex-session-history`
- Drafting GitHub PR titles, bodies, and maintainer comments:
  `github-pr-message-writer`
- Generating Xiaohongshu (小红书) 3:4 Swiss editorial cards/copy or publishing notes in Creator Center:
  `xiaohongshu-tech-note-publisher`

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
StaticFlow tracks 12 git submodules across `deps/`, `tools/`, and `patches/`.
Six public build submodules are required to compile the workspace
(`scripts/init_public_build_submodules.sh`), while the remaining six are
optional runtime services, CLI utilities, or the private `llm-access` workspace
(which is a release input and integration peer, not a StaticFlow Cargo
dependency).

| Submodule | Path | Repository | Role |
|---|---|---|---|
| lance | `deps/lance` | `acking-you/lance` | Public build dep (`feat/static-flow`) |
| lancedb | `deps/lancedb` | `acking-you/lancedb` | Public build dep (`feat/static-flow`) |
| pingora | `deps/pingora` | `acking-you/pingora` | Public build dep (`feat/static-flow`) |
| ffmpeg-sidecar | `deps/ffmpeg-sidecar` | `acking-you/ffmpeg-sidecar` | Public build dep (media transcoding) |
| jieba-rs | `deps/jieba-rs` | `StaticFlow-AI/jieba-rs` | Public build dep (Chinese tokenization) |
| object_store | `patches/object_store` | `acking-you/arrow-rs-object-store` | Public build patch (`dev` branch) |
| llm-access | `deps/llm-access` | private `acking-you/llm-access` | Managed multi-provider LLM services + React console |
| gpt2api_rs | `deps/gpt2api_rs` | `acking-you/gpt2api_rs` | Optional standalone GPT2API image gateway (`:18787`) |
| juicefs | `deps/juicefs` | `acking-you/juicefs` | Optional cloud JuiceFS client build source |
| AntigravityManager | `deps/AntigravityManager` | `acking-you/AntigravityManager` | Optional local Antigravity OAuth/proxy manager (`:8045`) |
| ncmdump-rs | `tools/ncmdump-rs` | `acking-you/ncmdump-rs` | Optional NCM/music ingestion CLI tool |
| pb-mapper | `tools/pb-mapper` | `acking-you/pb-mapper` | Cloud-to-local TCP tunnel & relay utility |

Key points:
- Rust toolchain is pinned in `rust-toolchain.toml` (`nightly-2026-08-21`).
- Root `Cargo.toml` uses path deps, not crates.io. Root workspace has
  `exclude = ["deps"]` so vendored projects never become workspace members.
- After cloning for public StaticFlow work:
  `scripts/init_public_build_submodules.sh`
- Authorized llm-access maintainers may additionally run:
  `git submodule update --init deps/llm-access`
- Do not run `cargo fmt` in `deps/lance` or `deps/lancedb`.
- When modifying submodule source, obey the submodule's own instructions and
  commit inside it first. For PR-based work, do not update or publish the parent
  gitlink until the child PR has passed review/CI and merged into the child
  default branch; remote reachability from only the child PR branch does not
  satisfy this rule. Verify the referenced child commit against the fetched
  child default branch before creating the parent PR.

## Codebase Structure
```
# Public workspace crates (11) — all under crates/
crates/frontend/                  Yew/WASM public site + site admin. LLM/Kiro/Codex
                                  `/admin/{llm,kiro}-gateway*` routes are handoff
                                  pages unless STATICFLOW_ENABLE_LEGACY_LLM_ADMIN=1
crates/shared/                    Shared domain types and compatibility facade
crates/store/                     LanceDB-backed content, comments, and music stores
crates/embedding/                 Text and image embedding services
crates/backend/                   Axum HTTP server — handlers, routes, state, workers, email
crates/cli/                       sf-cli binary — LanceDB write/query/embed/optimize workflows
crates/media-service/             Local media streaming/processing service (sf-media :39085)
crates/media-types/               Shared media type definitions
crates/email-notifier/            Email notification utilities (package static-flow-email)
crates/gateway/                   Pingora local ingress gateway (blue/green upstream switching)
crates/runtime/                   Shared runtime utilities (logging, tracing, signal handling)

# Non-crate directories
skills/              31 agent skill definitions (symlinked by .agents/skills & .claude/skills)
scripts/             Shell/Python scripts — launchers, workers, cloud releases, CI affected-set, e2e tests
docs/                Technical documentation, deep-dives, ops & monitor runbooks
content/             Article Markdown source files and images
conf/                Configuration files (Pingora gateway YAML templates)
deployment-examples/ Active self-hosted & cloud systemd templates (systemd/) + legacy Nginx configs
tools/               Third-party utility submodules (ncmdump-rs, pb-mapper)
bin/                 Exported local binaries (ignored in git)
patches/             Vendored crate patches (object_store)
deps/                Public dependency submodules + private llm-access workspace
                     (live operator console: deps/llm-access/apps/llm-access-frontend)
```
