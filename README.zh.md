# StaticFlow

[English](./README.md) | [CLI 使用手册](./docs/cli-user-guide.zh.md)

本地优先的写作、知识管理与媒体平台，全栈 Rust 实现。Axum 后端 + Yew/WASM 前端 +
LanceDB 存储。支持文章发布、AI 评论审核、音乐管理、图片资产导入、外部文章转载，
以及公开 LLM 接入层 — 全部自托管在本地机器上，可选云端边缘入口。

## 功能特性

- 全文搜索与语义（向量）搜索，支持跨语言回退
- 双语文章发布，Markdown 源文件 + 元数据自动补全
- 图片导入，blob v2 存储、缩略图、向量相似搜索
- 音乐库：网易云/NCM 导入、歌词、许愿功能
- AI 评论审核：后台 Codex agent worker 自动回复
- 外部博客转载：风格感知翻译 + 自适应源格式处理
- 交互式页面镜像：JS 重度外部页面本地化
- 公开 LLM 接入层：OpenAI 兼容（Codex）和 Anthropic 兼容（Kiro）网关，
  配额管理与用量计费

## 项目结构

全栈 Rust monorepo，包含 11 个公开 workspace crate。托管式 LLM 网关位于
独立私有 workspace，仅向有权限的维护者以可选 submodule 形式提供：

```text
static-flow/
├── crates/
│   ├── frontend/                # Yew/WASM 站点与站点 Admin（LLM/Kiro 运维控制台在 deps/llm-access/apps/llm-access-frontend）
│   ├── shared/                  # 共享领域类型与兼容 facade
│   ├── store/                   # 基于 LanceDB 的内容、评论和音乐存储
│   ├── embedding/               # 文本和图片 embedding 服务
│   ├── backend/                 # Axum HTTP 服务 — handler、路由、worker
│   ├── cli/                     # sf-cli — 写入/查询/嵌入/优化流程
│   ├── media-service/           # 图片/音频处理服务
│   ├── media-types/             # 共享媒体类型
│   ├── email-notifier/          # 邮件通知工具
│   ├── gateway/                 # 支持蓝绿切换的 Pingora 入口
│   └── runtime/                 # 日志、追踪与信号处理工具
├── skills/              # Codex/Claude agent skill 定义
├── scripts/             # Shell 脚本 — 启动器、worker runner、e2e 测试
├── docs/                # 技术文档、实现深潜、运维手册
├── conf/                # 配置文件（Pingora gateway YAML、systemd 模板）
├── content/             # 文章 Markdown 源文件与图片
├── tools/               # 第三方工具（ncmdump-rs、pb-mapper）
├── deps/                # 公开依赖 submodule + 私有 llm-access
└── patches/             # 供应商 crate 补丁
```

## 前置依赖

- Rust stable 工具链（edition 2021）
- `wasm32-unknown-unknown` target：`rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/) 前端构建：`cargo install trunk`
- 公开构建子模块：`scripts/init_public_build_submodules.sh`
- 建议：将 `CARGO_TARGET_DIR` 设置到大容量挂载点，避免根文件系统空间不足

## 快速开始

```bash
# 1. 克隆并初始化子模块
git clone https://github.com/StaticFlow-AI/static-flow.git
cd static-flow
scripts/init_public_build_submodules.sh

# 2. 设置构建产物目录（根据实际环境调整路径）
export CARGO_TARGET_DIR=/path/to/large-mount/cargo-target/static_flow

# 3. 编译后端 + CLI
cargo build --release -p static-flow-backend -p sf-cli --jobs 8

# 4. 初始化 LanceDB 表结构
$CARGO_TARGET_DIR/release/sf-cli init --db-path ./data/lancedb

# 5. 构建前端（自托管模式，同源 API）
bash scripts/build_frontend_selfhosted.sh

# 6. 启动后端（同时 serve 前端静态文件 + API）
bash scripts/start_backend_selfhosted.sh --daemon
```

本地开发（热重载）：

```bash
bash scripts/start_backend_selfhosted.sh            # 终端 1：后端
bash scripts/start_frontend_with_api.sh --open       # 终端 2：trunk 开发服务器
```

后端：`http://127.0.0.1:39080` | 前端开发：`http://127.0.0.1:38080`

## 部署模式

- **自托管（生产）**：后端提供 API + 前端静态文件，运行在 Pingora gateway 后面。
  当前生产使用 AWS Lightsail Caddy 做 TLS 和路由拆分，并通过 pb-mapper
  完成云端到本地的中继。
  详见 [docs/ops-runbook.md](./docs/ops-runbook.md)。
- **本地开发**：Trunk 开发服务器热重载，自动代理 `/api` 到后端。
- **GitHub Pages**：纯前端静态部署，API 通过 `STATICFLOW_API_BASE` 配置。
  CI：`.github/workflows/deploy.yml`。

## 当前生产形态

当前生产已经拆成“云端 LLM 层 + 本地内容层”：

- `https://ackingliu.top` 与经 Cloudflare 代理的 `https://staticflow.cc`
  都进入同一个 AWS Lightsail Caddy 源站
- LLM 路径（`/v1/*`、`/cc/v1/*`、`/api/llm-gateway/*`、`/api/kiro-gateway/*`、
  `/api/codex-gateway/*`、`/api/llm-access/*`）直接留在云端，进入独立
  `llm-access`
- 非 LLM StaticFlow 路径继续经过云端 pb-mapper，回到本地 Pingora 和当前激活
  backend slot

云端 `llm-access` 也已经拆成两个进程：

- `llm-access.service`：provider/admin API、Neon 控制面、账号刷新、usage journal
  生产
- `llm-access-usage-worker.service`：journal 消费、tiered DuckDB usage
  analytics、usage 查询接口

当前 usage analytics 存储布局：

- 共享 Neon 控制面配置：`/mnt/llm-access/config/neon.env`
- 保留的回退 SQLite 快照：`/mnt/llm-access/control/llm-access.sqlite3`
- 热 journal：`/var/lib/staticflow/llm-access/usage-journal`
- 当前可写 DuckDB：`/var/lib/staticflow/llm-access/analytics-active`
- 归档 immutable DuckDB segment + catalog：
  `/mnt/llm-access-usage/analytics`
- 单条事件的重明细 payload：pack 形式写入
  `/mnt/llm-access-usage/details/packs/...`

也就是说，生产 usage 明细的大字段已经不再放在 hot DuckDB 里，而是由 worker
把 summary 写入 DuckDB、把重明细通过独立的 JuiceFS usage mount 落成 pack
文件。

## CLI 概览

`sf-cli` 提供 LanceDB 操作：写入文章/图片、同步笔记、查询/搜索、管理索引、
优化表、调试 API 响应。

```bash
# 同步本地笔记目录（markdown + 图片 → LanceDB）
sf-cli sync-notes --db-path ./data/lancedb --dir ./content --recursive --generate-thumbnail

# 查询文章
sf-cli query --db-path ./data/lancedb --table articles --limit 10

# 数据库管理
sf-cli db --db-path ./data/lancedb list-tables
sf-cli db --db-path ./data/lancedb optimize articles

# API 兼容调试命令
sf-cli api --db-path ./data/lancedb search --q "staticflow"
sf-cli api --db-path ./data/lancedb semantic-search --q "前端 架构"
```

完整 CLI 用法：[docs/cli-user-guide.zh.md](./docs/cli-user-guide.zh.md)

## API 概览

后端默认监听 `127.0.0.1:39080`（生产环境在 Pingora `39180` 后面）。

| 端点 | 说明 |
|------|------|
| `GET /api/articles` | 文章列表（支持 tag/category 过滤） |
| `GET /api/articles/:id` | 文章详情 |
| `GET /api/articles/:id/raw/:lang` | 原始 Markdown（`lang=zh\|en`） |
| `POST /api/articles/:id/view` | 记录浏览（60 秒去重） |
| `GET /api/articles/:id/view-trend` | 浏览趋势（按天/小时，Asia/Shanghai） |
| `GET /api/articles/:id/related` | 相关文章（向量相似） |
| `POST /api/comments/submit` | 提交评论（限流） |
| `GET /api/comments/list` | 文章公开评论列表 |
| `GET /api/search?q=` | 全文搜索 |
| `GET /api/semantic-search?q=` | 语义搜索（向量，跨语言） |
| `GET /api/images` | 图片列表 |
| `GET /api/images/:id` | 图片二进制（支持 `?thumb=true`） |
| `GET /api/image-search?id=` | 以图搜图 |
| `GET /api/tags` | 标签列表 |
| `GET /api/categories` | 分类列表 |

每个响应包含 `x-request-id` 和 `x-trace-id` 用于关联追踪。

## 开发

```bash
export CARGO_TARGET_DIR=/path/to/large-mount/cargo-target/static_flow

# 编译
cargo build -p static-flow-backend -p sf-cli --jobs 8

# 测试
cargo test -p static-flow-shared -p static-flow-backend --jobs 8

# Lint（提交前修复所有警告）
cargo clippy -p static-flow-shared -p static-flow-backend --jobs 8 -- -D warnings

# 格式化（仅改动文件 — 不要在 workspace 根目录运行 cargo fmt --all）
rustfmt path/to/changed_file.rs

# 前端自托管构建
bash scripts/build_frontend_selfhosted.sh

# 前端热重载开发
bash scripts/start_frontend_with_api.sh --open

# CLI E2E 测试
./scripts/test_cli_e2e.sh
```

关键环境变量：
- `DB_ROOT`：LanceDB 数据根目录（默认 `/mnt/wsl/data4tb/static-flow-data`）
- `PORT`：后端端口（默认 `39080`）
- `STATICFLOW_API_BASE`：前端构建时 API 基址（自托管用 `/api`）
- `STATICFLOW_LLM_ACCESS_MODE=external`：将 LLM 路由代理到独立服务

## 数据仓库（Hugging Face）

运行时数据存储在两个 Hugging Face 数据集仓库和一个本地音乐库中：
- Content DB：[LB7666/my_lancedb_data](https://huggingface.co/datasets/LB7666/my_lancedb_data)
- Comments DB：[LB7666/static-flow-comments](https://huggingface.co/datasets/LB7666/static-flow-comments)
- Music DB：仅本地，`/mnt/wsl/data4tb/static-flow-data/lancedb-music`

## License

MIT
