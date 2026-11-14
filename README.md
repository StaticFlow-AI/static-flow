# StaticFlow

[中文文档](./README.zh.md) | [CLI Guide (ZH)](./docs/cli-user-guide.zh.md)

A local-first writing, knowledge-management, and media platform built in Rust
(Axum + Yew/WASM + LanceDB). Self-hosted on a single machine with full-text and
vector search, bilingual publishing, media management, and background AI
workflows.

## Features

- Full-text and cross-language semantic (vector) search
- Bilingual Markdown publishing with automated metadata and summaries
- Image library with blob v2 storage, thumbnails, and image/text vector search
- Music library with audio/lyrics ingestion and wish fulfillment
- AI-assisted comment moderation via background agent workers
- External blog repost with style-aware translation
- Standalone mirroring for interactive external pages

## Architecture

```text
static-flow/
├── crates/
│   ├── frontend/        # Yew/WASM SPA frontend
│   ├── shared/          # Shared domain types and API contracts
│   ├── store/           # LanceDB content, comments, and music stores
│   ├── embedding/       # Text and image embedding services
│   ├── backend/         # Axum HTTP server and background workers
│   ├── cli/             # sf-cli — write, query, embed, and optimize workflows
│   ├── media-service/   # Local audio/video/image streaming service
│   ├── media-types/     # Shared media types
│   ├── email-notifier/  # Email notification utilities
│   ├── gateway/         # Pingora local ingress with blue/green switching
│   └── runtime/         # Logging, tracing, and signal utilities
├── scripts/             # Build, launcher, worker, and E2E scripts
├── docs/                # Guides and technical deep-dives
└── conf/                # Gateway configuration templates
```

## Quick Start

Prerequisites: Rust toolchain from `rust-toolchain.toml`,
`rustup target add wasm32-unknown-unknown`, and `cargo install trunk`.

```bash
# 1. Clone and initialize build submodules
git clone https://github.com/StaticFlow-AI/static-flow.git
cd static-flow
scripts/init_public_build_submodules.sh

# 2. Build backend + CLI
export CARGO_TARGET_DIR=/path/to/cargo-target/static_flow
cargo build --release -p static-flow-backend -p sf-cli --jobs 8

# 3. Initialize LanceDB tables
$CARGO_TARGET_DIR/release/sf-cli init --db-path ./data/lancedb

# 4. Build self-hosted frontend and start backend
bash scripts/build_frontend_selfhosted.sh
DB_ROOT=./data bash scripts/start_backend_selfhosted.sh --daemon
```

Local development with hot-reload:

```bash
bash scripts/start_backend_selfhosted.sh          # terminal 1 (:39080)
bash scripts/start_frontend_with_api.sh --open     # terminal 2 (:38080)
```

## CLI

```bash
sf-cli sync-notes --db-path ./data/lancedb --dir ./content --recursive --generate-thumbnail
sf-cli query --db-path ./data/lancedb --table articles --limit 10
sf-cli db --db-path ./data/lancedb optimize articles
sf-cli api --db-path ./data/lancedb semantic-search --q "architecture"
```

Full reference: [docs/cli-user-guide.zh.md](./docs/cli-user-guide.zh.md)

## API Overview

| Endpoint | Description |
|----------|-------------|
| `GET /api/healthz` | Health check |
| `GET /api/articles` | Article list (tag/category filter) |
| `GET /api/articles/:id` | Article detail (`.../raw/:lang` for Markdown) |
| `GET /api/articles/:id/related` | Vector-similar articles |
| `GET /interactive-pages/:page_id` | Standalone mirrored interactive page |
| `GET /api/search?q=` | Full-text search |
| `GET /api/semantic-search?q=` | Cross-language vector search |
| `GET /api/images` | Image catalog (`/:id` for binary/thumbnail) |
| `GET /api/image-search?id=` | Image-to-image similarity search |
| `GET /api/image-search-text?q=` | Text-to-image vector search |
| `GET /api/music` | Music catalog, audio stream, and lyrics |
| `POST /api/comments/submit` | Submit article comment |
| `GET /api/comments/list` | Published comments for an article |
| `GET /api/tags` / `GET /api/categories` | Taxonomies |

## Development

```bash
export CARGO_TARGET_DIR=/path/to/cargo-target/static_flow

cargo test -p static-flow-shared -p static-flow-backend --jobs 8
cargo clippy -p static-flow-shared -p static-flow-backend --jobs 8 -- -D warnings
rustfmt path/to/changed_file.rs   # format changed files only; never run cargo fmt --all
./scripts/test_cli_e2e.sh
```

## License

MIT
