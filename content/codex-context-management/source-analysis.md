---
title: "Codex 上下文管理源码解析：从 Agent Loop 到历史恢复"
summary: "主文的源码伴读材料：固定提交、分支条件、状态更新、Responses 协议、history/notes 后端边界、测试入口与离线引用校验。"
tags: ["Codex", "源码解析", "Compaction", "TokenBudget"]
category: "AI Engineering"
author: "ackingliu"
date: "2026-09-14"
---

# Codex 上下文管理源码解析：从 Agent Loop 到历史恢复

> 基线：[OpenAI Codex `516f2780fd227a80cd9fe89488f5039245090b71`](https://github.com/openai/codex/tree/516f2780fd227a80cd9fe89488f5039245090b71)，提交时间 2026-09-13 16:14:41 UTC，核对日期 2026-09-14。本文配套[技术博客](index.md)，以执行链解释当前实现，不包含客户端修改，也不把设计建议写成上游代码。

建议先沿一条具体请求读代码：在启用 TokenBudget 的任务中，模型输出 `new_context({})`；客户端消费这个请求，创建新窗口；扩展尝试提供恢复提示；模型随后按需要调用历史或笔记工具。然后关闭这一假设，回到同一个分派点，沿 remote V2 分支读一遍。两次最后都到达窗口与历史替换基础设施，差别会比按文件名浏览更清楚。

目录：

- [1. 基线、证据边界与阅读顺序](#analysis-1)
- [2. 配置怎样变成有效能力](#analysis-2)
- [3. new_context 的完整调用链](#analysis-3)
- [4. 远程 V2 的完整调用链](#analysis-4)
- [5. history / notes 的工具与后端协议](#analysis-5)
- [6. 预算提醒和窗口计量](#analysis-6)
- [7. 公共检查点与尚未完成的融合](#analysis-7)
- [8. 测试源码能证明哪些约束](#analysis-8)
- [9. 近期开源变化应怎样解读](#analysis-9)
- [10. 离线复核与源码索引](#analysis-10)

<a id="analysis-1"></a>
## 1. 基线、证据边界与阅读顺序

本次分析从本地 Codex 仓库刷新 `origin/main`，再创建固定提交的独立 worktree 读取代码。原工作 checkout 和已安装 Codex 程序没有因此升级。“最新”在这里的含义是核对时取得的开源 `main` 提交，不是稳定版发行号，也不代表正在运行的二进制。

本文使用三类证据，结论强度分别处理：

| 证据 | 可以证明 | 不能证明 |
|---|---|---|
| 实际入口、分派条件和调用实现 | 该快照在相应有效条件下如何执行 | 某个真实账户是否启用了该条件 |
| 工具 schema、提示和后端契约 | 客户端向模型暴露什么以及预期怎样使用 | 模型每次都遵循指导，或后端内部实现细节 |
| 现有测试源码 | 测试作者覆盖了哪些行为断言 | 本次已经运行通过、或真实服务端有同样时延 |

阅读顺序如下：

```text
session/token_budget.rs         有效配置与实验启用条件
          ↓
tools/spec_plan.rs              new_context 的注册
          ↓
handlers/new_context_window.rs  请求标记
          ↓
session/turn.rs                 循环消费请求、策略分派
          ├─ compact_token_budget.rs → fresh
          └─ compact_remote_v2*.rs  → 远程压缩
                         ↓
session/mod.rs                  窗口与历史替换、检查点
                         ↓
ext/history-notes               新窗口恢复入口与后续工具调用
```

这里有些箭头表达的是运行阶段关系，例如 history 工具在新窗口之后按需被模型调用，不是前一个函数直接调用后一个文件中的所有工具。

<a id="analysis-2"></a>
## 2. 配置怎样变成有效能力

**当前位置：线程启动及模型配置解析，尚未进入一次换窗。**

`apply_experimental_context()` 不只看一个布尔开关。它同时检查实验功能、起始模型能力、provider 的 Codex 后端路由支持、认证方式、显式凭据设置和账户条件，还要经过功能管理规则。成功之后启用 TokenBudget，并设置原生 history / notes 标志。[实验模式的生效条件](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/token_budget.rs#L13-L58)。

| 条件组 | 源码中的关键字段或判断 | 阅读注意 |
|---|---|---|
| 功能请求 | `Feature::ContextManagement` | 仅有 feature 注册不代表已启用 |
| 模型能力 | `supports_experimental_context` | 起始模型也受约束，不是随时换一个名字即可 |
| provider | `supports_codex_backend_routes()`、`requires_openai_auth` | 普通兼容 Responses provider 不自动拥有 history 后端 |
| 认证覆盖 | `env_key`、bearer、`auth`、AWS 等排除条件 | 不要只检查当前是否有 token |
| 账户资格 | ChatGPT auth + Plus / Pro / ProLite 枚举 | 客户端枚举不等于线上开放公告 |
| 管理要求 | `enable(TokenBudget)` 后再检查 enabled | enable 调用成功也不一定覆盖管理层强制关闭 |

模型默认配置是另一条生效来源。`apply_model_defaults()` 先判断模型的 token-budget 默认 `enabled`，再避开显式用户配置和管理层限制。`resolve_token_budget()` 则负责将提醒、指导语和缓冲区等模型默认值解析进当前配置。[模型默认值与显式配置的关系](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/token_budget.rs#L81-L159)。

因此，排查时应查看本次任务的有效配置和实际模型元数据。内置 `models.json` 只是一个固定快照；它不能代替运行时目录，也不能把 `max_context_window` 当成当前窗口。

原生扩展还有自己的安装判断：

```rust
// codex-rs/ext/history-notes/src/extension.rs:45-64
impl HistoryNotesExtension {
    fn update_config(&self, thread_store: &ExtensionData, config: &Config) {
        if config
            .token_budget
            .as_ref()
            .is_some_and(|token_budget| token_budget.use_history_notes_extension)
            && config.model_provider.is_openai()
            && self.auth_manager.current_auth_uses_codex_backend()
        {
            thread_store.insert(HistoryNotesExtensionConfig {
                backend: HistoryNotesBackend::new(create_model_provider(
                    config.model_provider.clone(),
                    Some(self.auth_manager.clone()),
                )),
            });
        } else {
            thread_store.remove::<HistoryNotesExtensionConfig>();
        }
    }
}
```

[原生 history 和 notes 的安装条件](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/extension.rs#L45-L64)。

这解释了为什么 `TokenBudget=true` 与“history / notes 工具一定可用”不是同一个事实。实验路径会协调这些条件，但单独配置底层功能时仍应核对整条依赖。

最后，`spec_plan.rs` 在 TokenBudget 条件下注册 `new_context` 和 `get_context_remaining`。前者明确为 `DirectModelOnly`。历史与笔记扩展也使用 `DirectModelOnly`，因此不要假设它们必然作为 JavaScript Code Mode 的 `tools.*` 嵌套方法出现。[上下文工具的注册条件](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/tools/spec_plan.rs#L1203-L1215)；[扩展工具仅对模型直接暴露](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/tools.rs#L309-L315)。

<a id="analysis-3"></a>
## 3. new_context 的完整调用链

**当前位置：模型已经获得 `new_context` 的 schema 并发起调用。**

### 3.1 工具只发出请求，不在执行器里换窗

定义中的参数对象为空，禁止额外属性，没有策略字段。handler 接收 function payload 后调用 session 的请求方法，再返回一个表示请求已受理的工具结果。返回文案明确不进行会话总结。[new_context 工具定义](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/tools/handlers/new_context_window_spec.rs#L6-L17)；[new_context 工具执行器](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/tools/handlers/new_context_window.rs#L13-L43)。

底层状态操作如下：

```rust
// codex-rs/core/src/state/auto_compact_window.rs:95-103
    pub(super) fn request_new_context_window(&mut self) {
        self.new_context_window_requested = true;
    }

    pub(super) fn take_new_context_window_request(&mut self) -> bool {
        let requested = self.new_context_window_requested;
        self.new_context_window_requested = false;
        requested
    }
```

[窗口请求标记的写入与消费](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/state/auto_compact_window.rs#L95-L103)。

这个 bool 保存“需要新窗口”这一事实，不保存模型想保留哪些信息，也没有 write-notes-success 或 recovery-ready 字段。多个请求也不会自动形成一列不同策略任务。

### 3.2 循环消费请求，再由有效模式决定策略

`run_turn` 在一次采样和工具处理后取得 `needs_follow_up` 与 token status，然后决定是否切换。模型请求和容量阈值在这里汇合。[循环边界上的切换判断](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/turn.rs#L600-L609)。

进入 `run_auto_compact()` 后，TokenBudget 分支最先执行并返回；后面的 remote V2 分支没有机会成为这个模式下的自动应急策略。[自动切换的策略分支](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/turn.rs#L1408-L1452)。

| 入口 | TokenBudget 已启用 | TokenBudget 未启用且支持远程 V2 | 未启用且不支持远程 |
|---|---|---|---|
| 自动容量处理 | fresh | 远程压缩 | 客户端组织摘要 |
| 手动 compact task | fresh | 远程压缩 | 客户端组织摘要 |
| 模型 `new_context` | 注册后设置 fresh 请求 | 该条件下不由此注册 | 该条件下不由此注册 |

手动入口见 [手动压缩的策略分支](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/tasks/compact.rs#L35-L66)。表中的“fresh 请求”描述正常注册与有效配置配合的路径，不表示执行器自身另有一个策略枚举。

### 3.3 生命周期和业务行为需要分开读

```rust
// codex-rs/core/src/compact_token_budget.rs:63-81
    let turn_context = &step_context.turn;
    let pre_compact_outcome = run_pre_compact_hooks(sess, turn_context, trigger).await;
    match pre_compact_outcome {
        PreCompactHookOutcome::Continue => {}
        PreCompactHookOutcome::Stopped => return Err(CodexErr::TurnAborted),
    }

    let compaction_item = TurnItem::ContextCompaction(ContextCompactionItem::new());
    sess.emit_turn_item_started(turn_context, &compaction_item)
        .await;
    sess.start_new_context_window(step_context, world_state)
        .await;
    sess.emit_turn_item_completed(turn_context, compaction_item)
        .await;

    let post_compact_outcome = run_post_compact_hooks(sess, turn_context, trigger).await;
    if let PostCompactHookOutcome::Stopped = post_compact_outcome {
        return Err(CodexErr::TurnAborted);
    }
```

[fresh 路径的生命周期](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_token_budget.rs#L63-L81)。

同一个 `ContextCompaction` item 可以包围 fresh 操作。pre-hook 可以停止；post-hook 也可以在安装之后停止后续流程。所以看到“turn 被停止”还要辨别发生在哪一侧，不能直接推断新窗口尚未安装。

这段代码没有压缩模型请求，也没有调用 notes 写工具。若用户自定义 hook 做了额外工作，那是 hook 的行为，不能归因于 fresh 核心算法。

### 3.4 新上下文如何组成

```rust
// codex-rs/core/src/session/mod.rs:4385-4435
    pub(crate) async fn start_new_context_window(
        &self,
        step_context: &StepContext,
        world_state: Arc<WorldState>,
    ) -> u64 {
        let turn_context = step_context.turn.as_ref();
        let retained_client_developer_messages =
            if self.enabled(Feature::RetainClientDeveloperMessages) {
                let history = self.clone_history().await;
                crate::compact_remote_v2::truncate_retained_messages_for_remote_compaction(
                    history
                        .annotated_items()
                        .iter()
                        .filter(|item| {
                            crate::compact_remote_v2::is_client_authored_developer_message(item)
                        })
                        .cloned()
                        .collect(),
                    crate::compact_remote_v2::RETAINED_MESSAGE_TOKEN_BUDGET,
                )
            } else {
                Vec::new()
            };
        let window = {
            let mut state = self.state.lock().await;
            state.start_new_context_window()
        };
        let (window_number, window_ids) = window;
        let context_items = self
            .build_initial_context_with_world_state(step_context, world_state.as_ref())
            .await
            .into_iter()
            .map(ResponseItemEnvelope::new)
            .chain(retained_client_developer_messages)
            .collect();
        let turn_context_item = step_context.to_turn_context_item();
        self.replace_compacted_history(
            context_items,
            Some(turn_context_item),
            Some(world_state),
            CompactedHistoryMetadata {
                message: String::new(),
                window_number,
                window_ids,
                compaction_response_id: None,
                compaction_model_hash: None,
            },
        )
        .await;
        self.recompute_token_usage(turn_context).await;
        window_number
```

[新窗口的构建与安装](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/mod.rs#L4385-L4435)。

这段代码值得逐部分看：

| 操作 | 实际作用 | 不应误读为 |
|---|---|---|
| 收集 client-authored developer 消息 | 在对应 feature 开启时保留这类消息，并应用预算 | 自动保留所有 system / developer 历史消息 |
| `state.start_new_context_window()` | 推进窗口状态 | 清空工作目录、重启工具进程 |
| `build_initial_context_with_world_state()` | 依据当前状态重建初始信息、收集扩展提示 | 把旧会话自动变成摘要 |
| `replace_compacted_history()` | 替换活动历史并记录恢复状态 | 删除全部持久化历史 |
| `compaction_response_id: None` | fresh 没有对应压缩响应 ID | 证明整个换窗过程没有任何网络活动 |
| `recompute_token_usage()` | 重新计算新活动上下文用量 | 把累计费用或历史总 usage 清零 |

fresh 过程中构建初始上下文可能请求 `thread_hint`，所以“跳过总结请求”和“完全不访问网络”应严格区分。

窗口推进保持 first ID，更新 previous 与 current，并重置提醒状态：

```rust
// codex-rs/core/src/state/auto_compact_window.rs:77-94
    pub(super) fn advance(&mut self) -> (u64, AutoCompactWindowIds) {
        self.window_number = self.window_number.saturating_add(1);
        self.ids.previous_window_id = Some(self.ids.window_id);
        self.ids.window_id = Uuid::now_v7();
        self.new_context_window_requested = false;
        self.token_budget_reminder_delivered = false;
        self.auto_compact_fallback_delivered = false;
        (self.window_number, self.ids)
    }

    pub(super) fn claim_token_budget_reminder(&mut self) -> bool {
        !std::mem::replace(&mut self.token_budget_reminder_delivered, true)
    }

    pub(super) fn claim_auto_compact_fallback(&mut self) -> bool {
        !std::mem::replace(&mut self.auto_compact_fallback_delivered, true)
    }

```

[窗口 ID 和提醒状态推进](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/state/auto_compact_window.rs#L77-L94)。

它为历史定位提供了窗口身份，但仅有 ID 还不能证明后台已完成历史摄取。

<a id="analysis-4"></a>
## 4. 远程 V2 的完整调用链

**当前位置：自动或手动入口选择了 `RemoteCompactionSupport::V2`。**

### 4.1 请求准备

`compact_remote_v2_attempt.rs` 从活动历史建立带注解的 prompt 输入，保留需要单独处理的 metadata，处理被禁用的 direct-tool metadata，再追加 `ResponseItem::CompactionTrigger {}`。它使用当前工具路由器提供的模型可见工具清单，并构造专门的 compaction metadata。[CompactionTrigger 请求构建](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2_attempt.rs#L69-L91)。

归一化协议关系如下：

| 层 | 关键内容 | 实现位置 |
|---|---|---|
| 模型输入 | 历史项 + `compaction_trigger` | `compact_remote_v2_attempt.rs` |
| 请求环境 | base instructions、当前可见工具、相应设置 | Prompt 和 compaction metadata |
| 传输 | `ModelClientSession::stream()` | `compact_remote_v2.rs` |
| HTTP 表面 | provider 相对 `/responses` | `codex-api` endpoint |
| 返回验证 | compaction 项计数 + Completed | `collect_compaction_output()` |

依据：[压缩复用 Responses 流式客户端](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L364-L417)；[Responses 的 provider 相对路径](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/codex-api/src/endpoint/responses.rs#L38-L47)；[响应收集和完整性校验](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L419-L481)。

这里没有把公共 API 自动模式的 `context_management` 字段加入调用链。那是另一份 API 契约；不要用公共文档中的请求例子替换客户端当前源码。

### 4.2 响应流、重试与 usage

`run_remote_compaction_request_v2()` 使用 provider 的 stream retry 配置，并受远程压缩自己的重试上限约束。可重试错误交给共同的响应流重试处理；不可重试错误直接返回。函数并不在失败后调用 `start_new_context_window()`。[压缩复用 Responses 流式客户端](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L364-L417)。

响应收集器只在 `OutputItemDone` 里计数 `Compaction` 项，收到 `Completed` 时记录观测到的响应与 usage，随后检查流是否完整及压缩项数量。因此可以出现“已经记录服务端报告的 usage，但压缩结果结构不合格”的情况。相应测试专门覆盖了这个顺序；它说明的是 usage 记录，不是对实际计费政策的推断。[响应收集和完整性校验](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L419-L481)；[输出校验前记录 usage 的测试](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/tests/suite/compact_remote.rs#L435-L492)。

最低成功条件包括：

```text
收到 response.completed
AND 完成的输出中恰好有一个 Compaction 项
```

它不要求总输出项数量为一，也不保证服务端返回的压缩表示在语义上完全无损。后一项不是这种结构校验能证明的。

### 4.3 安装结果时保留哪些原始内容

```rust
// codex-rs/core/src/compact_remote_v2.rs:483-509
fn build_v2_compacted_history(
    prompt_input: Vec<ResponseItem>,
    prompt_input_metadata: Vec<Option<CodexHarnessMetadata>>,
    compaction_output: ResponseItem,
    retain_client_developer_messages: bool,
    image_budget: RetainedImageBudget,
) -> (Vec<ResponseItemEnvelope>, usize) {
    debug_assert_eq!(prompt_input.len(), prompt_input_metadata.len());
    let prompt_input = prompt_input
        .into_iter()
        .zip(prompt_input_metadata)
        .map(|(item, metadata)| ResponseItemEnvelope { item, metadata })
        .collect::<Vec<_>>();
    let retained = v2_history_item_groups(prompt_input)
        .filter(|group| {
            is_retained_for_remote_compaction_v2(&group.source, retain_client_developer_messages)
        })
        .flat_map(HistoryItemGroup::into_items)
        .collect::<Vec<_>>();
    let mut retained =
        truncate_retained_messages(retained, RETAINED_MESSAGE_TOKEN_BUDGET, image_budget);
    let retained_image_count = retained
        .iter()
        .map(|envelope| retained_input_image_count(&envelope.item))
        .sum::<usize>();
    retained.push(ResponseItemEnvelope::new(compaction_output));
    (retained, retained_image_count)
```

[保留消息和压缩项的组合](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L483-L509)。

`v2_history_item_groups()` 与保留判断先确定哪些消息可以进入新窗口；截断函数再应用保留消息预算和图片预算；最后追加服务端压缩项。`RETAINED_MESSAGE_TOKEN_BUDGET` 是 64,000 token，不是整份新上下文的大小上限。

更细的保留规则在 [哪些原始消息可以留下](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L534-L578)：真实 user / hook prompt 与一般合成上下文不同；client developer 需要 feature 和来源标记；agent message 也有内容类别和大小限制。不能简化成“保留所有用户与所有 agent 消息”。

随后调用链处理窗口推进、上下文注入和公共历史替换。远程路径会提供 compaction response ID 以及相应模型 hash；fresh 路径这两项为空。记录这些元数据有助于诊断，仍需结合实际分支和请求判断。[压缩结果安装顺序](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L297-L355)。

<a id="analysis-5"></a>
## 5. history / notes 的工具与后端协议

**当前位置：客户端注册恢复工具，并把一次工具请求转给后端。**

### 5.1 九个工具，对应九个后端动作

```rust
// codex-rs/ext/history-notes/src/tools.rs:70-101
    fn name(self) -> &'static str {
        match self {
            Self::HistoryListWindows => "list_windows",
            Self::HistoryListItems => "list_items",
            Self::HistoryReadItem => "read_item",
            Self::HistorySearchContents => "search_contents",
            Self::NotesListFilesByPrefix => "list_files_by_prefix",
            Self::NotesReadFile => "read_file",
            Self::NotesSearchContents => "search_contents",
            Self::NotesAppendToFile => "append_to_file",
            Self::NotesWriteFile => "write_file",
        }
    }

    fn endpoint(self) -> &'static str {
        match self {
            Self::HistoryListWindows => "alpha/history/v2/list_windows",
            Self::HistoryListItems => "alpha/history/v2/list_items",
            Self::HistoryReadItem => "alpha/history/v2/read_item",
            Self::HistorySearchContents => "alpha/history/v2/search_contents",
            Self::NotesListFilesByPrefix => "alpha/notes/v2/list_files_by_prefix",
            Self::NotesReadFile => "alpha/notes/v2/read_file",
            Self::NotesSearchContents => "alpha/notes/v2/search_contents",
            Self::NotesAppendToFile => "alpha/notes/v2/append_to_file",
            Self::NotesWriteFile => "alpha/notes/v2/write_file",
        }
    }

    fn supports_parallel_tool_calls(self) -> bool {
        !matches!(self, Self::NotesAppendToFile | Self::NotesWriteFile)
    }

```

[history 和 notes 的工具到路由映射](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/tools.rs#L70-L101)。

这些路径是 provider-relative 后端路径。它们不是稳定公开的 `/v1` 通用 API 承诺，也不意味着一个只兼容 Responses 的网关可以凭空实现 history 后端。

| 工具 | 关键参数或契约 | 与上下文恢复的关系 |
|---|---|---|
| `history.list_windows` | agent 选择、返回窗口目录 | 寻找窗口范围 |
| `history.list_items` | window、role、tool 过滤，限制预览长度 | 把回查范围缩小到少量 item |
| `history.read_item` | 必需 `window_id` + `item_id`；可限制字符范围 | 已知引用时直接恢复细节 |
| `history.search_contents` | 区分大小写的 literal query 和过滤条件 | 没有引用时搜索定位 |
| `notes.list_files_by_prefix` | 虚拟路径前缀 | 找到笔记文件 |
| `notes.read_file` | 路径及可选行范围 | 读取交接材料 |
| `notes.search_contents` | 字面查询 | 定位笔记条目 |
| `notes.append_to_file` | 路径 + 文本 | 增量写入 |
| `notes.write_file` | 路径 + 文本 | 创建或替换笔记 |

详细 schema 见 [历史和笔记工具的参数契约](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/tools.rs#L141-L241)。

代码明确把两个写工具标记为不支持并行调用。这是工具层的调用能力声明，不代表已经实现了任意多 agent 之间的数据库事务或锁。当前公开 namespace 描述与模型指导语对跨 agent 写入范围还存在表述差异，因此本文不据其推断最终权限策略；准确权限需要服务端契约或针对性验证。[历史和笔记的命名空间契约](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/tools.rs#L20-L27)；[内置 Astra 模型与 TokenBudget 提示配置](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/models-manager/models.json#L1-L112)。

### 5.2 Backend 不是本地文件适配器

```rust
// codex-rs/ext/history-notes/src/backend.rs:38-55
            return Err("History tool arguments must be a JSON object".to_string());
        };
        arguments_object.insert(
            "context".to_string(),
            json!({
                "session_id": session_id,
                "current_agent_name": current_agent_name,
            }),
        );

        let provider = self.provider.api_provider().await.map_err(|_| {
            format!("{OPERATION_ERROR_PREFIX} Could not resolve the backend provider.")
        })?;
        let auth = self.provider.api_auth().await.map_err(|_| {
            format!("{OPERATION_ERROR_PREFIX} Could not resolve backend authentication.")
        })?;

        let mut request = provider.build_request(Method::POST, path);
```

[后端请求注入会话与 agent 身份](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/backend.rs#L38-L55)。

请求注入会话和 agent 身份之后，backend 通过 provider 解析 API 配置与认证，设置截断策略 header，给部分操作设置加密参数标识，最后发送 POST。该模块的请求超时为 35 秒。[后端请求、截断策略和超时](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/backend.rs#L14-L95)。

这里看不到本地 JSONL 读取、文件路径打开或离线 notes 回退实现。`notes` 名称中的 file 描述的是虚拟文件抽象，不能据此把后端数据等同为本地文件。

如果工具返回 `encrypted_output`，客户端把它放入专用的内容项：

```rust
// codex-rs/ext/history-notes/src/tools.rs:332-350
    fn new(mut result: Value) -> Result<Self, FunctionCallError> {
        // Separate attachments before serializing any text or retaining log output.
        let images = result.as_object_mut().and_then(|map| map.remove("images"));
        // The server applies the requested output budget before encryption.
        let mut output = match result.get("encrypted_output").and_then(Value::as_str) {
            Some(encrypted_content) => FunctionCallOutputPayload::from_content_items(vec![
                FunctionCallOutputContentItem::EncryptedContent {
                    encrypted_content: encrypted_content.to_string(),
                },
            ]),
            None => FunctionCallOutputPayload::from_text(result.to_string()),
        };
        if let Some(images) = images {
            let invalid_image = || {
                FunctionCallError::RespondToModel(
                    "History backend returned invalid image content.".to_string(),
                )
            };
            let images = images.as_array().ok_or_else(invalid_image)?;
```

[加密工具结果的客户端封装](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/tools.rs#L332-L350)。

同一实现还处理返回的图片附件。客户端注释表明截断应在后端加密之前发生；这与加密后客户端再次按明文长度裁剪是不同的处理顺序。

公开代码足以确认这些 wire-level 细节，但看不到私有后端的数据库组织、密钥管理和完整的恢复逻辑。引用清单刻意只声明可见范围。

### 5.3 thread hint 只是窗口初始信息的一部分

贡献器请求 `alpha/notes/v2/thread_hint`，要求 `text` 字段，检查最多 4,000 字节，再构造 `PromptSlot::ContextWindow` 的片段。请求失败、字段缺失、超长或空文本分别导致没有可注入内容。[窗口提示的后端读取与注入](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/extension.rs#L97-L151)。

“初始上下文中没有 notes 提示”因而有多种可能：扩展未启用、身份或配置不存在、后端请求失败，或者后端返回空文本。不能只凭一个空片段断言没有历史，也不能断言客户端已完成其他恢复。

### 5.4 摄取请求和可读取状态是两件事

```rust
// codex-rs/core/src/session/session.rs:701-723
    fn with_window_and_fork_metadata(
        &self,
        turn_context: &TurnContext,
        responses_metadata: CodexResponsesMetadata,
        window_number: u64,
        context_window_id: uuid::Uuid,
    ) -> CodexResponsesMetadata {
        CodexResponsesMetadata {
            window_number: Some(window_number),
            context_window_id: Some(context_window_id),
            analytics_enabled: Some(self.services.analytics_events_client.is_enabled()),
            history_ingest_requested: turn_context
                .config
                .token_budget
                .as_ref()
                .is_some_and(|config| config.use_history_notes_extension)
                .then_some(true),
            forked_from_ordinal_exclusive: self
                .forked_from_ordinal_exclusive
                .filter(|_| responses_metadata.forked_from_thread_id.is_some()),
            ..responses_metadata
        }
    }
```

[history_ingest_requested 与窗口元数据](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/session.rs#L701-L723)。

`history_ingest_requested` 根据原生扩展标志加入请求 metadata。它表达客户端请求后端摄取历史的意图。当前这一段代码没有等待所有 item 可检索的确认。

工具契约声明 history 为最终一致性；notes 的成功写后直接读取和列表 / 搜索的一致性语义也不同。因此，依赖“刚生成就立刻搜索命中”的恢复流程需要额外处理，不能把一次空搜索当成已证明信息永久丢失。[历史和笔记的命名空间契约](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/tools.rs#L20-L27)。

<a id="analysis-6"></a>
## 6. 预算提醒和窗口计量

**当前位置：客户端计算当前窗口还能安全推进多少。**

`context_window_token_status()` 先取得全部活动上下文用量，再按 scope 计算自动阈值范围。`Total` 计整个活动窗口；`BodyAfterPrefix` 从使用量中扣除窗口 prefill 基线。完整可用模型上下文上限独立计算。[两种计量范围与容量边界](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/context_window.rs#L60-L109)。

核心计算如下：

```rust
// codex-rs/core/src/session/context_window.rs:82-109
    // The model's full context window is a hard cap, independent of the auto-compaction scope.
    let full_context_window_limit = model_info.resolved_context_window().map(|context_window| {
        context_window.saturating_mul(model_info.effective_context_window_percent) / 100
    });

    // Report remaining tokens against the base (unbuffered) window, capped by the full context.
    let base_window_tokens_remaining = [
        tokens_remaining(auto_compact_scope_limit, auto_compact_scope_tokens),
        tokens_remaining(full_context_window_limit, active_context_tokens),
    ]
    .into_iter()
    .flatten()
    .min();

    // Only reserve the fallback buffer when there is a fallback prompt to use it.
    let auto_compact_fallback_buffer_tokens = config
        .token_budget
        .as_ref()
        .map_or(0, crate::config::TokenBudgetConfig::fallback_buffer_tokens);
    let buffered_auto_compact_limit = auto_compact_scope_limit
        .map(|limit| limit.saturating_add(auto_compact_fallback_buffer_tokens));

    // Force compaction once the buffered window or the model's full context window is reached.
    let full_context_window_limit_reached =
        full_context_window_limit.is_some_and(|limit| active_context_tokens >= limit);
    let token_limit_reached = buffered_auto_compact_limit
        .is_some_and(|limit| auto_compact_scope_tokens >= limit)
        || full_context_window_limit_reached;
```

[剩余预算和缓冲区计算](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/context_window.rs#L82-L109)。

需要分别观测的量至少有四个：全部活动用量、scope 内用量、未加缓冲的剩余量、完整窗口剩余量。只有一个 UI 百分比时，容易把不同口径混成同一个容量。

`maybe_record()` 只处理提醒注入。它先判断 TokenBudget 和配置，达到提醒阈值时 claim 一次提醒；基础剩余量为零且允许 fallback 时，再 claim 一次 fallback 指导语。两个状态都在窗口推进时重置。[预算提醒与 fallback 提示的注入](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/token_budget.rs#L161-L224)；[窗口 ID 和提醒状态推进](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/state/auto_compact_window.rs#L77-L94)。

注意这里的 `record_conversation_items()` 将提示加入会话，不能把它理解为工具调用，也不能据函数名声称模型已经写了 notes。

主文用内置模型配置计算出的 13,600 token 空间，是有明确假设的算术示例。若实际模型窗口、effective percent、scope 或覆盖配置不同，就应该重新计算，而不是把该数字写进产品逻辑。

<a id="analysis-7"></a>
## 7. 公共检查点与尚未完成的融合

**当前位置：某条策略已经准备好新的活动历史，准备把它交给公共状态管理。**

`replace_compacted_history()` 构造 `CompactedItem`，其中包含 replacement history、窗口编号、窗口 ID 和压缩响应标识等信息。随后协调 settings 持久化锁、替换内存中的 annotated history、记录 retained context 和 Guardian 相关恢复信息，按需设置完整 world-state baseline，最后写入 rollout items。[替换历史与持久化检查点](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/mod.rs#L3929-L4005)。

阅读这段时应区分四个动作：

| 动作 | 当前代码的作用 | 额外的融合协议还需要解决什么 |
|---|---|---|
| 构造检查点 | 表达新的活动历史与窗口状态 | 如何标记仅准备成功但尚未提交的候选状态 |
| 更新内存状态 | 让后续推理使用新的上下文 | 与并发到达的输入如何协调版本 |
| 本地持久化 | 保存 rollout 的恢复材料 | 与远端 notes / history 如何形成可验证关系 |
| 后续生命周期 | 排队相应 session-start hook 来源 | 重启后如何确认哪些动作已执行 |

公共函数说明两条路径共享一部分落地机制；它不说明分布式原子提交已经实现。当前 `new_context` 请求标记只存 bool，也没有携带必须保留的状态清单。

若希望验证主文提出的融合方向，可以把新增工作拆成可审查的四个接口边界：策略请求、交接材料验证、候选上下文构造、可恢复提交。不要先把当前 `new_context` schema 改成自动压缩，然后才补失败语义。

这一节是实现差距分析。主文里的 `WindowTransitionRequest` 和 prepare / commit 状态机均是建议模型，不存在于本次提交的这些源码类型中。

<a id="analysis-8"></a>
## 8. 测试源码能证明哪些约束

以下测试已阅读源码定位，**本次没有执行**。其中不少使用本地模拟 SSE / WebSocket 服务，并不连接真实模型后端；测试通过也不等于真实模型必定写好笔记。

| 关注点 | 测试函数 | 直接验证的方向 |
|---|---|---|
| 实验能力门槛 | `experimental_context_requires_capable_model_and_codex_backend` | 模型与 provider 条件组合 |
| 起始模型 | `token_budget_history_notes_requires_capable_starting_model` | 原生历史笔记功能的起始能力要求 |
| hooks | `token_budget_compaction_runs_compact_hooks` | fresh 模式仍进入 compact 生命周期 |
| 缓冲区 | `token_budget_auto_compact_fallback_uses_buffer_until_new_context` | 基础阈值后还能按缓冲设计继续 |
| 强制换窗 | `token_budget_auto_compact_fallback_rolls_over_after_buffer` | 缓冲结束后的推进行为 |
| 主动换窗 | `new_context_tool_skips_auto_compact_fallback` | 工具暴露、窗口 ID 关系、旧内容移出以及后续请求 |
| 远程后续输入 | `remote_compact_v2_reuses_compaction_trigger_for_followups` | 远程压缩请求与后续历史形状 |
| 重试预算 | `remote_compact_v2_retries_failures_with_stream_retry_budget` | 使用受限流重试 |
| usage 时序 | `remote_compact_v2_records_usage_before_output_validation` | 结构校验之前记录完成响应 usage |

定位：[实验能力与起始模型测试](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/tests/suite/token_budget.rs#L315-L441)；[fresh 模式的 hooks 测试](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/tests/suite/token_budget.rs#L1233-L1282)；[缓冲区与强制换窗测试](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/tests/suite/token_budget.rs#L1411-L1570)；[new_context 跳过旧 fallback 的测试](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/tests/suite/token_budget.rs#L1574-L1677)；[远程压缩与后续请求测试](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/tests/suite/compact_remote.rs#L750-L830)；[远程流式重试预算测试](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/tests/suite/compact_remote.rs#L1100-L1180)；[输出校验前记录 usage 的测试](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/tests/suite/compact_remote.rs#L435-L492)。

主动换窗测试特别值得完整读：模拟第一次响应调用 `new_context`，随后继续执行任务，断言 first ID 不变、previous 指向原窗口、current ID 已变化，并验证后续请求不再包含原先那条用户文本。它检验了换窗行为，不等于检验真实模型已经通过 notes 恢复了所有任务约束。

需要运行时，应先阅读目标 checkout 的 `AGENTS.md`、准备 nextest 和该仓库要求的环境，在 Codex 仓库根目录串行执行。下面只是复现建议，本文没有运行：

```bash
just test -p codex-core -E 'test(suite::token_budget::)'
just test -p codex-core -E 'test(suite::compact_remote::)'
```

这些命令遵循该快照 `justfile` 对 `cargo nextest` 的封装。它们会产生 Rust 构建产物，应按所在环境的构建预算执行，不能把“可复现命令”误当成本次测试记录。

目前源码测试与本文设计之间仍有明显空隙：没有据此证明“notes 失败后一定阻止 fresh”“远程失败后自动安全改用 fresh”“所有历史条目都已摄取”等性质。要实现主文的融合协议，就需要为这些新增契约补测试，并做真实恢复实验。

<a id="analysis-9"></a>
## 9. 近期开源变化应怎样解读

删除 legacy 文件很容易制造“整个 compact 都消失了”的印象。近期提交的真实含义，需要结合替代入口一起读。

| UTC 日期 | 固定提交 | 与本主题相关的变化 | 不应推导出的结论 |
|---|---|---|---|
| 2026-09-09 | [`3dc1e2a584`](https://github.com/openai/codex/commit/3dc1e2a58406dc69db5812539adfee7d89fa9ef7) | 对支持的 provider 始终使用流式远程压缩 | 不是删除远程压缩能力 |
| 2026-09-09 | [`1ac689cc7d`](https://github.com/openai/codex/commit/1ac689cc7de0e637d35271edbbc132d1553d1767) | 删除不再使用的 legacy remote 实现 | 不是取消所有 compact 入口 |
| 2026-09-11 | [`4dcce4f0c4`](https://github.com/openai/codex/commit/4dcce4f0c47e0183e4b05ffcbb38b4fffb8b8042) | 限制不支持的起始模型使用 token-budget history / notes | 不是任意模型都能靠配置获得同样能力 |
| 2026-09-13 | [`1715e55076`](https://github.com/openai/codex/commit/1715e55076737158ba61d43158ede504de6d4ce1) | 调整 direct tool-call metadata 与输出绑定；压缩准备相应处理 | 不是新增自适应换窗策略 |
| 2026-09-13 | [`16537b20a5`](https://github.com/openai/codex/commit/16537b20a5ec0ea9aa079f4ad4b0e30e8a9efacf) | 普通请求 metadata 和工具 hooks 使用捕获的 step 设置 | 不能宣称压缩请求所有配置都已使用同一完整快照 |

最后一点在代码里有明确边界：普通 `responses_metadata()` 从 `StepContext` 取执行设置和工具信息；`compaction_responses_metadata()` 仍有 TODO，指出请求其他部分尚基于 turn，远程压缩目前补充的是已确定的工具清单。[普通采样与压缩请求的元数据快照](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/session.rs#L647-L699)。

这处 TODO 对未来融合很有启发。若策略层要在准备阶段生成候选新窗口，就必须准确知道候选结果对应哪一份设置和历史。不能因为普通采样已经使用 step snapshot，就假设压缩路径也全部完成同样的收敛。

<a id="analysis-10"></a>
## 10. 离线复核与源码索引

交付目录包含四个文件：

| 文件 | 作用 |
|---|---|
| [index.md](index.md) | 完整技术博客，包含当前机制与明确标注的未来设计 |
| [source-analysis.md](source-analysis.md) | 本篇逐函数分析与测试阅读指南 |
| [source-map.json](source-map.json) | 固定提交、路径、引用范围与源码内容 SHA-256 |
| [verify_sources.py](verify_sources.py) | 从本地 Git 对象离线复核引用、代码摘录、目录与相对链接 |

在已包含该提交对象的本地 Codex 仓库上执行：

```bash
python3 verify_sources.py --repo /path/to/codex
```

校验器从自身所在目录读取文档和来源清单，用 `git show <固定提交>:<路径>` 读取源码，不切分支、不 fetch、不读取账户凭据、不运行 Codex，也不请求后端 history / notes。工作区即使位于其他提交，它核对的仍是来源清单里固定的 Git 对象。

它检查源码范围及哈希、带源码标记的 Rust 摘录、两篇文档的固定 SHA 链接、Markdown 本地链接与显式目录锚点。它不检查外部网站实时可用性、Mermaid 的浏览器布局、模型行为、服务端实现或性能。

源码索引：

| 模块 | 关键职责 | 入口 |
|---|---|---|
| `core/src/session/token_budget.rs` | 激活、默认配置、提醒注入 | [实验模式的生效条件](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/token_budget.rs#L13-L58)；[预算提醒与 fallback 提示的注入](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/token_budget.rs#L161-L224) |
| `core/src/session/context_window.rs` | scope、buffer 与完整上限 | [两种计量范围与容量边界](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/context_window.rs#L60-L109) |
| `core/src/tools/handlers/new_context_window*` | 工具契约与请求标记 | [new_context 工具定义](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/tools/handlers/new_context_window_spec.rs#L6-L17)；[new_context 工具执行器](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/tools/handlers/new_context_window.rs#L13-L43) |
| `core/src/session/turn.rs` | 循环触发与自动策略分派 | [采样之后的完整控制流](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/turn.rs#L550-L640)；[自动切换的策略分支](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/turn.rs#L1408-L1452) |
| `core/src/tasks/compact.rs` | 手动入口 | [手动压缩的策略分支](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/tasks/compact.rs#L35-L66) |
| `core/src/compact_token_budget.rs` | fresh 生命周期 | [fresh 路径的生命周期](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_token_budget.rs#L63-L81) |
| `core/src/compact_remote_v2_attempt.rs` | 压缩请求构建 | [CompactionTrigger 请求构建](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2_attempt.rs#L69-L91) |
| `core/src/compact_remote_v2.rs` | stream、验证、结果保留与安装 | [响应收集和完整性校验](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L419-L481)；[保留消息和压缩项的组合](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/compact_remote_v2.rs#L483-L509) |
| `core/src/session/mod.rs` | 新窗口、替换历史与检查点 | [新窗口的构建与安装](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/mod.rs#L4385-L4435)；[替换历史与持久化检查点](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/mod.rs#L3929-L4005) |
| `ext/history-notes/src/extension.rs` | 扩展安装与 thread hint | [原生 history 和 notes 的安装条件](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/extension.rs#L45-L64)；[窗口提示的后端读取与注入](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/extension.rs#L97-L151) |
| `ext/history-notes/src/tools.rs` | 工具 schema、路由和输出 | [history 和 notes 的工具到路由映射](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/tools.rs#L70-L101)；[加密工具结果的客户端封装](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/tools.rs#L332-L350) |
| `ext/history-notes/src/backend.rs` | 认证后端请求和预算传递 | [后端请求、截断策略和超时](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/history-notes/src/backend.rs#L14-L95) |
| `core/src/session/session.rs` | step、window 和 history 摄取元数据 | [普通采样与压缩请求的元数据快照](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/session.rs#L647-L699)；[history_ingest_requested 与窗口元数据](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/core/src/session/session.rs#L701-L723) |
| `rollout/src/model_context.rs` | 从记录重建活动模型上下文 | [从 rollout 重建活动模型上下文](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/rollout/src/model_context.rs#L1-L130) |
| `ext/memories`、`memories/write` | 独立本地长期记忆路径 | [独立的本地 memories 扩展](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/ext/memories/src/extension.rs#L136-L172)；[本地记忆写入的启动任务](https://github.com/openai/codex/blob/516f2780fd227a80cd9fe89488f5039245090b71/codex-rs/memories/write/src/start.rs#L24-L94) |

这组入口足以复核主文的核心判断：当前同时存在远程压缩与 fresh 两种延续任务的方式；history / notes 为 fresh 提供按需恢复能力；公共窗口设施已经复用，而自适应策略与可验证交接协议仍属于后续设计空间。
