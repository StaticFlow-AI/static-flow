# StaticFlow

[English](./README.md) | [CLI 使用手册](./docs/cli-user-guide.zh.md)

基于全栈 Rust（Axum + Yew/WASM + LanceDB）构建的本地优先写作、知识管理与媒体平台。支持单机自托管，内置全文与向量语义检索、双语文章发布、图片/音乐资产管理及后台 AI 工作流。

## 功能特性

- 全文检索与跨语言语义（向量）搜索
- 双语 Markdown 发布与元数据、摘要自动生成
- 图片管理：blob v2 存储、缩略图、以图搜图与跨模态以文搜图
- 音乐库：音频/歌词导入、流式播放与点歌许愿
- AI 辅助评论审核：后台 Agent Worker 自动处理
- 外部博客转载与自适应风格翻译
- JS 重度外部页面的本地独立交互镜像

## 项目结构

```text
static-flow/
├── crates/
│   ├── frontend/        # Yew/WASM 前端应用
│   ├── shared/          # 共享领域类型与 API 契约
│   ├── store/           # LanceDB 内容、评论与音乐存储层
│   ├── embedding/       # 文本与图片向量化服务
│   ├── backend/         # Axum HTTP 服务与后台 Worker
│   ├── cli/             # sf-cli — 写入、查询、向量化与表优化工具
│   ├── media-service/   # 本地音视频/图片媒体流服务
│   ├── media-types/     # 共享媒体类型定义
│   ├── email-notifier/  # 邮件通知组件
│   ├── gateway/         # 支持蓝绿无损切换的 Pingora 本地网关
│   └── runtime/         # 日志、追踪与信号处理工具
├── scripts/             # 构建、启动、Worker 与 E2E 测试脚本
├── docs/                # 使用指南与技术实现文档
└── conf/                # 网关配置模板
```

## 快速开始

前置依赖：安装 `rust-toolchain.toml` 指定工具链，并执行 `rustup target add wasm32-unknown-unknown` 与 `cargo install trunk`。

```bash
# 1. 克隆并初始化构建子模块
git clone https://github.com/StaticFlow-AI/static-flow.git
cd static-flow
scripts/init_public_build_submodules.sh

# 2. 编译后端与 CLI
export CARGO_TARGET_DIR=/path/to/cargo-target/static_flow
cargo build --release -p static-flow-backend -p sf-cli --jobs 8

# 3. 初始化 LanceDB 基础表
$CARGO_TARGET_DIR/release/sf-cli init --db-path ./data/lancedb

# 4. 构建自托管前端并启动后端
bash scripts/build_frontend_selfhosted.sh
DB_ROOT=./data bash scripts/start_backend_selfhosted.sh --daemon
```

本地热重载开发：

```bash
bash scripts/start_backend_selfhosted.sh          # 终端 1 (:39080)
bash scripts/start_frontend_with_api.sh --open     # 终端 2 (:38080)
```

## CLI 概览

```bash
sf-cli sync-notes --db-path ./data/lancedb --dir ./content --recursive --generate-thumbnail
sf-cli query --db-path ./data/lancedb --table articles --limit 10
sf-cli db --db-path ./data/lancedb optimize articles
sf-cli api --db-path ./data/lancedb semantic-search --q "前端 架构"
```

完整文档见 [docs/cli-user-guide.zh.md](./docs/cli-user-guide.zh.md)。

## API 概览

| 端点 | 说明 |
|------|------|
| `GET /api/healthz` | 健康检查 |
| `GET /api/articles` | 文章列表（支持 tag/category 过滤） |
| `GET /api/articles/:id` | 文章详情（`.../raw/:lang` 获取原始 Markdown） |
| `GET /api/articles/:id/related` | 向量相似相关文章 |
| `GET /interactive-pages/:page_id` | 独立交互镜像页入口 |
| `GET /api/search?q=` | 全文检索 |
| `GET /api/semantic-search?q=` | 跨语言语义检索 |
| `GET /api/images` | 图片目录（`/:id` 获取原图或缩略图） |
| `GET /api/image-search?id=` | 以图搜图 |
| `GET /api/image-search-text?q=` | 跨模态以文搜图 |
| `GET /api/music` | 音乐列表、流式播放与歌词接口 |
| `POST /api/comments/submit` | 提交文章评论 |
| `GET /api/comments/list` | 获取已发布评论 |
| `GET /api/tags` / `GET /api/categories` | 标签与分类列表 |

## 开发

```bash
export CARGO_TARGET_DIR=/path/to/cargo-target/static_flow

cargo test -p static-flow-shared -p static-flow-backend --jobs 8
cargo clippy -p static-flow-shared -p static-flow-backend --jobs 8 -- -D warnings
rustfmt path/to/changed_file.rs   # 仅格式化改动文件，禁止运行 cargo fmt --all
./scripts/test_cli_e2e.sh
```

## License

MIT
