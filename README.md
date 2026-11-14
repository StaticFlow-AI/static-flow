# StaticFlow

[中文文档](./README.zh.md) | [CLI Guide (ZH)](./docs/cli-user-guide.zh.md)

A local-first writing, knowledge-management, and media platform built entirely
in Rust. Axum backend + Yew/WASM frontend + LanceDB storage. Includes article
publishing, AI-powered comment review, music management, image asset ingestion,
external article repost, and a public LLM access layer — all self-hosted on a
local machine with optional cloud edge ingress.

## Features

- Full-text and semantic (vector) search with cross-language fallback
- Bilingual article publishing with Markdown source and metadata enrichment
- Image ingestion with blob v2 storage, thumbnails, and vector similarity search
- Music library with Netease/NCM ingestion, lyrics, and wish fulfillment
- AI-powered comment moderation via background Codex agent workers
- External blog repost ingestion with style-aware translation
- Interactive page mirroring for JS-heavy external content
- Public multi-provider LLM access layer: OpenAI-compatible (Codex),
  Anthropic-compatible (Kiro), Cursor/Grok, Antigravity, and OpenCode decision
  gateways with fallback-graph routing, quota management, and tiered usage
  accounting

## Architecture

Full-stack Rust monorepo with 11 public workspace crates. The managed LLM
gateway is developed in a separate private workspace mounted as an optional
submodule for authorized maintainers:

```text
static-flow/
├── crates/
│   ├── frontend/                # Yew/WASM public site + site admin (LLM operator console is deps/llm-access/apps/llm-access-frontend)
│   ├── shared/                  # Shared domain types and compatibility facade
│   ├── store/                   # LanceDB-backed content, comments, and music stores
│   ├── embedding/               # Text and image embedding services
│   ├── backend/                 # Axum HTTP server — handlers, routes, workers
│   ├── cli/                     # sf-cli — write/query/embed/optimize workflows
│   ├── media-service/           # Image/audio/video local media service
│   ├── media-types/             # Shared media type definitions
│   ├── email-notifier/          # Email notification utilities
│   ├── gateway/                 # Pingora ingress with blue/green switching
│   └── runtime/                 # Logging, tracing, and signal utilities
├── skills/                      # Codex/Claude agent skill definitions (31 skills)
├── scripts/                     # Shell scripts — launchers, worker runners, cloud releases, e2e tests
├── docs/                        # Technical docs, deep-dives, ops runbook
├── conf/                        # Configuration (Pingora gateway YAML)
├── deployment-examples/         # Self-hosted & cloud systemd templates (+ legacy Nginx examples)
├── content/                     # Article Markdown source files and images
├── tools/                       # Third-party utilities (ncmdump-rs, pb-mapper)
├── deps/                        # Public dependency submodules + private llm-access
└── patches/                     # Vendored crate patches (object_store)
```

## Prerequisites

- Rust toolchain pinned by `rust-toolchain.toml` (currently `nightly-2026-08-21`)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/) for frontend builds: `cargo install trunk`
- Public build submodules: `scripts/init_public_build_submodules.sh`
- Recommended: set `CARGO_TARGET_DIR` to a large-capacity mount to avoid
  filling the root filesystem

## Quick Start

```bash
# 1. Clone and initialize submodules
git clone https://github.com/StaticFlow-AI/static-flow.git
cd static-flow
scripts/init_public_build_submodules.sh

# 2. Set build artifact directory (adjust path to your setup)
export CARGO_TARGET_DIR=/path/to/large-mount/cargo-target/static_flow

# 3. Build backend + CLI
cargo build --release -p static-flow-backend -p sf-cli --jobs 8

# 4. Initialize LanceDB tables
$CARGO_TARGET_DIR/release/sf-cli init --db-path ./data/lancedb

# 5. Build frontend (self-hosted, same-origin API)
bash scripts/build_frontend_selfhosted.sh

# 6. Start backend (serves frontend static files + API)
bash scripts/start_backend_selfhosted.sh --daemon
```

For local development with hot-reload:

```bash
bash scripts/start_backend_selfhosted.sh            # terminal 1: backend
bash scripts/start_frontend_with_api.sh --open       # terminal 2: trunk dev server
```

Backend: `http://127.0.0.1:39080` | Frontend dev: `http://127.0.0.1:38080`

## Deployment

StaticFlow supports three deployment modes:

- **Self-hosted (production)**: Backend serves API + frontend static files behind
  a Pingora gateway. Current production uses AWS Lightsail Caddy for TLS and
  route splitting, plus pb-mapper for the
  cloud-to-local relay. See [docs/ops-runbook.md](./docs/ops-runbook.md).
- **Local development**: Trunk dev server with hot-reload, proxying `/api` to backend.
- **GitHub Pages**: Frontend-only static deploy; API calls go to configured
  `STATICFLOW_API_BASE`. CI: `.github/workflows/deploy.yml`.

## Current Production Shape

Current production is split between a cloud LLM tier and a local content tier:

- `https://ackingliu.top` and the Cloudflare-fronted `https://staticflow.cc`
  terminate at the same AWS Lightsail Caddy origin
- LLM paths (`/v1/*`, `/cc/v1/*`, `/api/llm-gateway/*`, `/api/kiro-gateway/*`,
  `/api/codex-gateway/*`, `/api/llm-access/*`, `/api/cursor-gateway/v1/*`,
  `/api/antigravity-gateway/v1/*`) stay on the cloud VM and go to standalone
  `llm-access` services
- non-LLM StaticFlow paths continue through cloud pb-mapper and land on the
  local Pingora gateway (`:39180`) + active backend slot (`:39080` / `:39081`)
  and local media service (`:39085`)

The cloud `llm-access` tier is decomposed into independently deployable
processes:

- `llm-access.service` (`:19080`): main Codex/Kiro gateway, shared admin plane,
  Neon Postgres control plane + Valkey request cache, account refreshers, and
  hot usage journal production
- `llm-access-usage-worker.service` (`:19081`): journal consumption, tiered
  DuckDB usage analytics, packed detail blobs, and usage query routes
- `llm-access-codex-image.service` (`:19082`): standalone Codex image gateway
- `llm-access-opencode.service` (`:19085`): typed OpenCode decision gateway
  (primary Kiro Thinking Guard review tier)
- `llm-access-cursor.service` (`:19090`) and `llm-access-antigravity.service`
  (`:19095`): thin Cursor/Grok and Antigravity data planes sharing Neon + hot
  usage journal
- `llm-access-oauth.service` (`:19194`) and `llm-access-monitor.service`
  (`:19092`): loopback OAuth manager and Linux `/proc` + cgroup v2 + PSI +
  JuiceFS host/process sampler

Current usage analytics storage shape:

- shared Neon control config: `/mnt/llm-access/config/neon.env`
- retained rollback SQLite snapshot: `/mnt/llm-access/control/llm-access.sqlite3`
- hot journal: `/var/lib/staticflow/llm-access/usage-journal`
- active mutable DuckDB: `/var/lib/staticflow/llm-access/analytics-active`
- archived immutable DuckDB segments:
  `/mnt/llm-access-usage/analytics/segments`
- narrow archived-segment catalog: Neon Postgres tables (`llm_usage_segments`,
  `llm_usage_segment_events`, `llm_usage_segment_key_rollups`)
- heavy per-event detail payloads: packed files under
  `/mnt/llm-access-usage/details/packs/...`

This means production usage detail payloads are no longer stored in the hot
DuckDB files. The worker writes summary facts to DuckDB and stores heavy detail
packs through the dedicated JuiceFS usage mount instead of direct R2 uploads.

## CLI Reference

`sf-cli` provides LanceDB operations: write articles/images, sync notes,
query/search, manage indexes, optimize tables, and debug API responses.

```bash
# Sync local notes folder (markdown + images → LanceDB)
sf-cli sync-notes --db-path ./data/lancedb --dir ./content --recursive --generate-thumbnail

# Query articles
sf-cli query --db-path ./data/lancedb --table articles --limit 10

# Database management
sf-cli db --db-path ./data/lancedb list-tables
sf-cli db --db-path ./data/lancedb describe-table articles
sf-cli db --db-path ./data/lancedb optimize articles

# API-compatible debug commands
sf-cli api --db-path ./data/lancedb search --q "staticflow"
sf-cli api --db-path ./data/lancedb semantic-search --q "前端 架构"
```

Full CLI usage: [docs/cli-user-guide.zh.md](./docs/cli-user-guide.zh.md)

## API Overview

Backend listens on `127.0.0.1:39080` by default (behind Pingora on `39180`
in production).

| Endpoint | Description |
|----------|-------------|
| `GET /api/healthz` | Liveness/readiness check (includes active slot metadata) |
| `GET /api/articles` | Article list (tag/category filter) |
| `GET /api/articles/:id` | Article detail |
| `GET /api/articles/:id/raw/:lang` | Raw markdown (`lang=zh\|en`) |
| `POST /api/articles/:id/view` | Track view (60s dedupe) |
| `GET /api/articles/:id/view-trend` | View trend (day/hour, Asia/Shanghai) |
| `GET /api/articles/:id/related` | Related articles (vector similarity) |
| `GET /interactive-pages/:page_id` | Mirrored standalone interactive page entry |
| `POST /api/article-requests` | Submit external article repost/translation request |
| `POST /api/comments/submit` | Submit comment (rate-limited) |
| `GET /api/comments/list` | Public comments for an article |
| `GET /api/search?q=` | Full-text search |
| `GET /api/semantic-search?q=` | Semantic search (vector, cross-language) |
| `GET /api/images` | Image catalog |
| `GET /api/images/:id` | Image binary (`?thumb=true` supported) |
| `GET /api/image-search?id=` | Similar images (by image vector) |
| `GET /api/image-search-text?q=` | Cross-modal text-to-image vector search |
| `GET /api/music` | Music catalog & streaming audio/lyrics endpoints |
| `POST /api/music/wishes` | Submit song ingestion wish for background worker |
| `GET /api/tags` | Tag list |
| `GET /api/categories` | Category list |

Every response includes `x-request-id` and `x-trace-id` for correlation.

## Development

```bash
export CARGO_TARGET_DIR=/path/to/large-mount/cargo-target/static_flow

# Build
cargo build -p static-flow-backend -p sf-cli --jobs 8

# Test
cargo test -p static-flow-shared -p static-flow-backend --jobs 8

# Lint (fix all warnings before commit)
cargo clippy -p static-flow-shared -p static-flow-backend --jobs 8 -- -D warnings

# Format (only changed files — never cargo fmt --all at workspace root)
rustfmt path/to/changed_file.rs

# Frontend self-hosted build
bash scripts/build_frontend_selfhosted.sh

# Frontend dev with hot-reload
bash scripts/start_frontend_with_api.sh --open

# CLI E2E tests
./scripts/test_cli_e2e.sh
```

Key env vars:
- `DB_ROOT`: LanceDB data root (default `/mnt/wsl/data4tb/static-flow-data`)
- `PORT`: Backend port (default `39080`)
- `STATICFLOW_API_BASE`: Frontend build-time API base (`/api` for self-hosted)
- `STATICFLOW_LLM_ACCESS_MODE=external`: Proxy LLM routes to standalone service

## Data Repository (Hugging Face)

Runtime data is stored in two Hugging Face dataset repos plus one local music DB:
- Content DB: [LB7666/my_lancedb_data](https://huggingface.co/datasets/LB7666/my_lancedb_data)
- Comments DB: [LB7666/static-flow-comments](https://huggingface.co/datasets/LB7666/static-flow-comments)
- Music DB: local-only at `/mnt/wsl/data4tb/static-flow-data/lancedb-music`

## License

MIT
