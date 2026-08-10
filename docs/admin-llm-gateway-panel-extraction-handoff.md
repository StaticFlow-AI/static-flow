# LLM Gateway 管理面板拆页交接文档（P4 剩余部分）

> **✅ 本文档描述的全部工作已于 2026-07-10 完成**（commits `e6ec2f6` keys /
> `a9d4ca7` settings / `7ddbb3d` groups / `958b40a` accounts / `f2df173`
> requests / `3af98c3` overview 换装）。mega 面板从 8786 行缩到 ~2600 行，
> 只剩 overview 落地页 + 供子页复用的 `pub(crate)` helper 与三个 editor
> card（KeyEditorCard / AccountGroupEditorCard / ProxyConfigEditorCard）。
> 以下内容保留作历史参考；行号与"待拆"状态均已过时。
>
> 拆页期间的两处有意行为变化：
> 1. Create Key 表单原先渲染在 Settings tab 里，现随 kiro 布局移到
>    `/admin/llm-gateway/keys` 页顶部。
> 2. Groups 页现在主动拉全量账号清单喂 member picker——原 mega 在直接进入
>    groups tab 时传给 editor card 的 accounts 是空的（成员编辑会误清空）。
> 3. overview 的"待审核"计数改为独立轻量扫描（三个队列各取第一页，按
>    requests 页同一套 needs-action 状态过滤），不再依赖 requests tab 状态。

> 交接基线：commit `5906431`（Usage tab 拆页已完成）。
> 目标读者：接手把 `crates/frontend/src/pages/admin_llm_gateway.rs`（mega 面板，
> 现 8786 行）剩余 tab 拆成独立 `.admin-shell` 页的人。
> 所有行号标注均以 `5906431` 为准，**随编辑会漂移**——请以 `} // end TAB_X`
> 注释锚点和函数名为准，用 `rg` 重新定位。

## 1. 背景与目标

这是一次持续的管理面板重构（内部代号 P1→P4），视觉基准是
`deps/llm-access/crates/llm-access-ai-review/ui` 的 React 控制台。目标：

- 视觉像 ai-review 控制台（`.admin-shell` 设计系统，定义在 `crates/frontend/input.css` 末尾）。
- 功能不堆在一个巨型组件里；每个 section 一条真路由、一个独立页。
- 首屏最少大请求，懒加载不损体验；"不重复不打扰"。

kiro 面板（`admin_kiro_gateway*.rs`）已全部拆完，是**最佳参照模板**。
llm 面板（本文件）还剩 5 个 tab + overview 换装。

## 2. 当前状态

| tab | 状态 | 独立页文件 | 真路由 |
|---|---|---|---|
| Overview | ✅ 已换装（`3af98c3`，admin-shell 落地页 + 导航卡） | `admin_llm_gateway.rs`（仅 overview） | `Route::AdminLlmGateway` |
| Keys | ✅ 已拆（`e6ec2f6`，含原 Settings 内的 Create Key 表单） | `admin_llm_gateway_keys.rs` | `Route::AdminLlmGatewayKeys` |
| Groups | ✅ 已拆（`7ddbb3d`，页内自拉全量账号喂 member picker） | `admin_llm_gateway_groups.rs` | `Route::AdminLlmGatewayGroups` |
| Accounts | ✅ 已拆（`958b40a`，账号专属 helper/测试随页迁移） | `admin_llm_gateway_accounts.rs` | `Route::AdminLlmGatewayAccounts` |
| Usage | ✅ 已拆（`5906431`） | `admin_llm_gateway_usage.rs` | `Route::AdminLlmGatewayUsage` |
| Journal | ✅ 已拆（`dd4d3a3`） | `admin_llm_gateway_journal.rs` | `Route::AdminLlmGatewayJournal` |
| Requests | ✅ 已拆（`f2df173`，徽章改为 overview 独立 pending 扫描） | `admin_llm_gateway_requests.rs` | `Route::AdminLlmGatewayRequests` |
| Settings | ✅ 已拆（`a9d4ca7`） | `admin_llm_gateway_settings.rs` | `Route::AdminLlmGatewaySettings` |

Monitor 页 (`admin_llm_gateway_monitor.rs`) 本就是独立页，不在本次范围。

7 条真路由都已在 `router.rs` 的 `switch()` 里就位——目前 keys/groups/accounts/
requests/settings 五条都仍指向 `AdminLlmGatewayPage tab="<x>"`（mega 内按 prop 渲染）。
拆哪个 tab，就把对应那行 switch 分支改成指向新页组件（usage 的改法见 `5906431` diff）。

## 3. 核心机制（拆页前必须理解）

`AdminLlmGatewayPage`（组件定义在 `#[function_component(AdminLlmGatewayPage)]`）
是一个巨型函数组件。它的运作方式：

### 3.1 active_tab 与路由
- `active_tab: String` 从 `props.tab` 派生，缺省 `TAB_OVERVIEW`（见组件顶部
  `let active_tab = props.tab...`）。
- mount effect（`use_effect_with(props.tab.clone(), ...)`）：把 legacy `?tab=`
  深链一次性 `navigator.replace` 到对应真路由。**每个拆出去的独立页不需要这段**
  ——它只服务 mega 的 overview 落地页。
- `on_tab_click`：把 tab 名 push 成 `llm_tab_route(&tab)`。独立页由 `render_tab_bar`
  的 `Link` 导航过去，不需要 `on_tab_click`。
- `fn llm_tab_route(tab)` 把 tab 字符串映射到 `Route`（TAB_KEYS→Keys 等）。

### 3.2 巨型 `reload` callback —— 拆页的核心难点
`let reload = { ... }`（`rg -n 'let reload = \{'`）是一个 `Callback<bool>`，
`bool` = force_base 语义（true=连 config 一起强刷）。它在一个 `spawn_local` 里
**按 active_tab 门控**并发拉取多路数据：

```
fetch_admin_llm_gateway_config()                 // 总是
fetch_admin_llm_gateway_proxy_configs()          // 总是（settings 用）
fetch_admin_llm_gateway_proxy_bindings()         // 总是（settings 用）
keys 分页          if should_load_llm_gateway_keys_inventory(tab)  → tab==KEYS
group_options      if should_load_llm_gateway_group_options(tab)   → tab==KEYS
account_groups页   if tab == TAB_GROUPS
accounts页         if tab == TAB_ACCOUNTS
codex_status       if tab == TAB_ACCOUNTS
import_jobs        if should_load_llm_gateway_import_jobs(tab)      → tab==ACCOUNTS
```

这三个门控 helper 在文件顶部：`should_load_llm_gateway_keys_inventory` /
`should_load_llm_gateway_group_options` / `should_load_llm_gateway_import_jobs`。
测试 `llm_inventory_load_helpers_follow_active_tab` 覆盖它们。

**拆 keys/groups/accounts 的本质**：把该 tab 那一路 fetch 从 `reload` 里剥出来，
搬到新页自己的 mount effect（`use_effect_with((), ...)` + `spawn_local`）。
新页不再依赖 mega 的 `reload`。参照 usage 页 `reload_usage` 和 journal 页的
mount effect。当 keys/groups/accounts **全部**拆走后，`reload` 会瘦身到只剩
config + proxy 两路（settings 用），届时若 settings 也拆走，`reload`、`active_tab`
门控、`on_tab_click`、`render_tab_bar` 及绝大多数 `*_input` 状态全部变死代码，
overview 换装时一并清除。

### 3.3 requests 是独立的三条 reload（不走大 reload）
requests tab 有自己的三个 callback，**不挂在** `reload` 上：
- `let reload_token_requests = { ... }`
- `let reload_account_contribution_requests = { ... }`
- `let reload_sponsor_requests = { ... }`

它们由 requests 段落的 effect（`use_effect_with((active_tab.clone(),) ...` 里
`if active_tab == TAB_KEYS/ACCOUNTS` 那批同级的 requests effect）在进入 tab 时触发。
`total_pending`（`let total_pending = ...`）聚合三者 pending 数，喂给 `render_tab_bar`
的徽章参数 `Some((TAB_REQUESTS, total_pending))`。**这就是 requests 与 overview 绑定
的原因**：徽章在 tab bar 上，tab bar 在 mega 里。拆 requests 时 total_pending 的
计算要么随页走（新页自己算徽章无意义，因为徽章在导航条），要么保留在 mega 供
overview 导航卡用。建议：requests 与 overview 一起在最后处理。

### 3.4 已在 mega 内、供新页复用的 `pub(crate)` 共享符号
usage/journal 拆页时已把一批 helper 提为 `pub(crate)`，都留在 mega：

- 常量：`USAGE_PAGE_SIZE` / `USAGE_SOURCE_*` / `USAGE_STATUS_KIND_*` /
  `USAGE_KEY_OPTION_LIMIT`
- usage/latency/格式化：`format_optional_latency_ms[_or_na]` / `usage_account_label`
  / `effective_routing_wait_ms` / `format_optional_bytes` / `format_optional_duration_ms`
  / `usage_retry_title` / `usage_stream_state_label` / `usage_stream_state_badge_classes`
  / `format_stream_summary` / `compute_other_latency_ms` / `LatencyBreakdown` /
  `format_latency_breakdown` / `routing_diagnostics_summary` / `format_credit4` /
  `usage_source_label` / `usage_status_kind_label` / `parse_datetime_local_input_to_ms`
  / `format_datetime_local_input` / `usage_time_description` / `UsageReloadArgs` /
  `normalized_usage_filter_text` / `normalized_usage_status_kind` / `preview_text` /
  `copy_icon_button` / `pretty_headers_json` / `pretty_json_text` /
  `usage_journal_preview_message` / `usage_journal_preview_has_full_message`

拆新 tab 时，凡是多个 tab 共用的 helper 就地提 `pub(crate)`、新页 `use
super::admin_llm_gateway::{...}` 导入；tab 专属的 helper 随页搬走。

### 3.5 Editor 卡片子组件（keys/groups/settings 拆页会用到）
mega 内已有三个 `#[function_component]` 卡片 + 各自 Props：
- `KeyEditorCard` / `KeyEditorCardProps`（keys tab）
- `AccountGroupEditorCard` / `AccountGroupEditorCardProps`（groups tab）
- `ProxyConfigEditorCard` / `ProxyConfigEditorCardProps`（settings tab）

kiro 侧拆 keys 的经验（见 `admin-panel-redesign-progress` 记忆 P3b-5）：**不物理
搬动**上千行的卡片，而是把卡片 + 其 Props（字段全 `pub(crate)`）提为 `pub(crate)`，
新页直接 import 复用，卡片内部私有 helper 全部不动。llm 侧同理照做。

## 4. 拆页配方（切片搬移法，已验证 5 次）

参照 `admin-panel-redesign-progress` 记忆里的 P4-2/P4-3 配方，标准流程：

1. **共享 helper 提 `pub(crate)`**：先判定新页要用哪些 mega helper，就地把它们
   （及 tab 专属但被共享 helper 依赖的私有函数如 `preview_text`）改 `pub(crate)`。
2. **建新页 scaffold**：`pages/admin_llm_gateway_<tab>.rs`，`#[function_component]`
   自己的状态 + mount effect 自取数（不依赖 mega reload）+ `.admin-shell` markup。
   markup 可先"切片搬移"（把 mega 那段 `if active_tab == TAB_X { <section>...` 里的
   markup 整体搬来，按钮 `btn-terminal`→`.admin-shell` 的 `primary/ghost` reskin），
   内部密集 markup 不必逐行换装（可接受，后续可选精修）。
3. **`mod.rs` 注册**：`pub mod admin_llm_gateway_<tab>;`（保持字母序）。
4. **接路由（Phase A，两文件并存先编译）**：`router.rs` 的 `switch()` 把
   `Route::AdminLlmGateway<Tab>` 分支从 `AdminLlmGatewayPage tab="<x>"` 改成
   新页组件。此时先 `cargo check` 确认新页能编译。
5. **从 mega 剪除（Phase B）**：删掉 mega 里该 tab 的 render 段（`} // end TAB_X`
   锚点之间）、tab 专属状态、tab 专属回调/effect、以及从 `reload` 里剥掉该 tab 的
   fetch 分支。
6. **清理死代码**：删 mega 变为 import-only 的 import、死状态；迁移 tab 专属测试到
   新页（`include_str!` 类改指向、`#[cfg(test)]` 参照函数随页走）。
7. **验证 + 格式化 + 独立 commit**（见 §6）。

## 5. 逐 tab 拆分要点

### 5.1 Settings（建议先做，最独立）
- **render 段**：`// ── Settings Tab ──` 到 `} // end TAB_SETTINGS`
  （`5906431` 约 5720–6698，~978 行）。
- **自取数**：只需 `fetch_admin_llm_gateway_config` + `fetch_admin_llm_gateway_proxy_configs`
  + `fetch_admin_llm_gateway_proxy_bindings`（这三路目前在 `reload` 里"总是拉"）。
  新页 mount effect 自己拉这三路即可，不碰 keys/groups/accounts 门控。
- **专属状态**（约 40 个 config `*_input` + proxy 簇）：`ttl_input`、
  `max_request_body_input`、`account_failure_retry_limit_input`、所有 `codex_*_input`、
  `kiro_*_input`、`usage_flush_*_input`、`duckdb_usage_*_input`、
  `usage_analytics_retention_days_input`、`kiro_cctest_proxy_*_input`、
  `saving_runtime_config`、`proxy_configs`、`proxy_config_scope`、`proxy_bindings`、
  `create_proxy_*`、`creating_proxy`、`codex_proxy_binding_input`、
  `kiro_proxy_binding_input`、`saving_proxy_binding_provider`、
  `migrating_legacy_kiro_proxy`、`proxy_config_search`、`proxy_config_active_query`、
  `proxy_config_show_active_only`。
- **专属回调**：`on_save_runtime_config`、`on_create_proxy_config`、proxy binding 保存、
  `import_admin_legacy_kiro_proxy_configs`、proxy 搜索 effect（`proxy_config_active_query`
  的 debounce effect）。
- **专属子组件**：`ProxyConfigEditorCard` + Props 提 `pub(crate)` 供新页复用。
- **API**：`update_admin_llm_gateway_config`、`create_admin_llm_gateway_proxy_config`、
  `update_admin_llm_gateway_proxy_binding`、`patch_admin_llm_gateway_proxy_config`、
  `reset_admin_llm_gateway_proxy_config_override`、`refresh_admin_llm_gateway_proxy_traffic`、
  `import_admin_legacy_kiro_proxy_configs`。
- **注意**：settings 拆走后，`reload` 里 config/proxy 三路是否还需保留取决于 overview
  是否还要展示 config 派生值（overview 目前用 `key_summary` 等，不直接用 config
  input）。谨慎确认后再从 `reload` 剥离。

### 5.2 Keys
- **render 段**：`// ── Keys Tab ──` 到 `} // end TAB_KEYS`（约 6700–6842，~142 行
  markup；真正大头是 `KeyEditorCard` ~700 行）。
- **自取数**：`reload` 里 `should_load_llm_gateway_keys_inventory` + `..._group_options`
  两路（keys 分页 + group options）。参照 kiro keys 页（`admin_kiro_gateway_keys.rs`）：
  新页自拉 keys 分页 + group-options + config（派生 cache 默认）。
- **专属状态**：`keys`、`keys_summary`、`keys_search`、`keys_sort_mode`、
  `keys_show_active_only`、`keys_page`、`keys_total`、`keys_page_limit`、
  `create_key`、`creating`、`refreshing_key_id`。
- **专属子组件**：`KeyEditorCard` + Props 提 `pub(crate)`。
- **kiro 教训**：keys 一走，mega 的 keys inventory effect/门控/keys 状态全死；连带
  清理。

### 5.3 Groups
- **render 段**：`} // end TAB_KEYS` 后到 `} // end TAB_GROUPS`（约 6844–7018）。
- **自取数**：`reload` 里 `if tab == TAB_GROUPS` 的 account_groups 分页 + group_options。
- **专属状态**：`account_group_options`（可能 keys 也用，判定后决定留/搬）、
  `account_groups_page_items`、`account_groups_total`、`account_groups_page`、
  `account_groups_page_limit`、`account_groups_search`、`account_group_candidate_accounts`、
  `account_group_candidate_loading`、`create_account_group_*`、`creating_account_group`、
  `account_group_form_expanded`。
- **专属子组件**：`AccountGroupEditorCard` + Props 提 `pub(crate)`；member_picker 若
  kiro 已抽公共组件优先复用。

### 5.4 Accounts（最大，~1058 行）
- **render 段**：`// ── Accounts Tab ──` 到 `} // end TAB_ACCOUNTS`（约 7020–8078）。
- **自取数**：`reload` 里 `if tab == TAB_ACCOUNTS` 三路：accounts 分页 + codex_status
  + import_jobs（`should_load_llm_gateway_import_jobs`）。
- **专属状态**：accounts 分页/搜索、import job 相关（`active_import_job` 及其
  `use_effect_with((*active_import_job).clone(), ...)` 轮询 effect）、codex batch import
  表单。
- **注意**：有 import job 轮询 effect 和 codex batch import JSON 解析
  （测试 `parse_admin_codex_batch_import_json_*` 在 mega test mod，随页迁走）。

### 5.5 Requests（与 overview 一起最后做）
- **render 段**：`// ── Requests Tab ──` 到 `} // end TAB_REQUESTS`（约 8081–8545）。
- **自取数**：三条独立 reload（§3.3），不走大 reload。整段可较干净搬走。
- **专属状态**：`token_request_*`、`account_contribution_request_*`、`sponsor_request_*`
  各六个状态 + 三个 `*_action_inflight: HashSet<String>`。
- **API 动作**：`admin_approve_and_issue_llm_gateway_token_request`、
  `admin_reject_llm_gateway_token_request`、
  `admin_approve_and_issue_llm_gateway_account_contribution_request`、
  `admin_validate_llm_gateway_account_contribution_request`、
  `admin_reject_llm_gateway_account_contribution_request`、
  `admin_approve_llm_gateway_sponsor_request`、
  `delete_admin_llm_gateway_sponsor_request`。
- **total_pending 徽章**：`let total_pending = ...` 聚合三者。拆走 requests 后，
  overview 的 tab bar 若保留 requests 徽章，需要一个轻量 pending 计数来源
  （可考虑 overview 单独拉一个 summary，或保留三条 reload 的 pending 数在 mega）。
  推荐把 total_pending 逻辑留 mega，供 overview 导航卡展示"待审核 N"。

### 5.6 Overview 换装（最后一步）
所有 tab 拆走后，`AdminLlmGatewayPage` 只剩 overview。参照 kiro overview 换装
（`61809f6`，记忆 P3b-6）：去掉 `render_tab_bar` 导航条，改成 eyebrow 头 + stat 条
+ 导航卡（链到各子页）+ effective-proxy panel。届时删除 `reload`（若已无 tab 用）、
`active_tab`、`on_tab_click`、`llm_tab_route`（若 legacy 重定向也去掉）、三个门控
helper、几乎所有 `*_input` 状态。保留 overview 需要的 `key_summary`/额度聚合/
`total_pending`。

## 6. 验证（本地，逐条必过）

> GitHub CI 因账号封禁不可用（见 CLAUDE.md「Temporary Local-Only Development Mode」）。
> 以本地检查为准，独立 commit 直接落 `main`（当前工作分支）。

```bash
export CARGO_TARGET_DIR=/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow
# 0) 确认无并发构建（会 OOM 打挂 live backend）
pgrep -af 'cargo|rustc|trunk|ld|lld|mold'
# 1) wasm 编译（前端唯一构建 target）
cargo check -p static-flow-frontend --target wasm32-unknown-unknown --jobs 6
# 2) clippy（CI 用法：wasm target，不带 --tests）
cargo clippy -p static-flow-frontend --target wasm32-unknown-unknown --jobs 6 -- -D warnings
# 3) 测试（host target；CI 用 cargo test -p static-flow-frontend）
cargo test -p static-flow-frontend --jobs 6
# 4) rustfmt 仅改动文件（禁止 cargo fmt / --all）
rustfmt crates/frontend/src/pages/admin_llm_gateway.rs \
        crates/frontend/src/pages/admin_llm_gateway_<tab>.rs \
        crates/frontend/src/pages/mod.rs crates/frontend/src/router.rs
# 5) 确认 submodule 未被污染
git -C deps/lance status --short && git -C deps/lancedb status --short
```

## 7. 踩过的坑（务必知晓）

1. **缩进锚点陷阱**（kiro P3b-4 教训）：`if active_tab == TAB_X` 在 render 区（12 空格
   缩进）和 effect 门控区（16 空格缩进）各有一处，纯字符串匹配会命中错的那个。剪除
   render 段时用带缩进的唯一锚点（`\n                if active_tab == TAB_X`）或
   `} // end TAB_X` 注释边界。
2. **`rg -c` 会把 `#[cfg(test)]` 体算进去**（usage P4-3 教训）：某类型 `rg -c` 显示
   11 处引用，但非-test 的 wasm 构建仍报它 unused——因为引用全在 test-gated 代码里。
   **信编译器，不信 rg 计数**；test-only 参照 helper 随页迁走而非留 mega。
3. **回调闭合尾巴**：剥离回调时末尾的 `})` / `};` 要连同 end marker 一起删干净，否则
   括号不匹配。
4. **`reload.emit(true/false)` 语义**：true=force_base（连 config 强刷），false=切
   tab/翻页轻刷。剥离 tab fetch 后，别忘了检查其他 tab 的 `reload.emit` 调用点是否
   仍语义正确。
5. **禁止广义 formatter**：`cargo fmt` / `cargo fmt --all` 是策略违规（会污染
   `deps/lance`、`deps/lancedb`）。只 `rustfmt <改动文件>`。
6. **CSS 改动**（若换装动到 `.admin-shell` 令牌）：`cd crates/frontend && npm run tailwind`
   重新编 CSS。纯复用现有 `.admin-shell` 类则不需要。

## 8. 参照文件

- 已完成模板：`admin_llm_gateway_usage.rs`、`admin_llm_gateway_journal.rs`、
  `admin_kiro_gateway_{keys,groups,accounts,usage}.rs`
- 设计系统：`crates/frontend/input.css` 末尾 `.admin-shell` 作用域；样板页
  `admin_kiro_anthropic_upstreams.rs`
- 路由：`crates/frontend/src/router.rs` 的 `switch()`
- 进度记忆：`admin-panel-redesign-progress`（本仓外，agent 记忆）
- 关键 commit：`5906431`(usage) / `dd4d3a3`(journal) / `61809f6`(kiro overview) /
  `ba419af`(kiro keys) / `1bbd219`(kiro 路由驱动)

