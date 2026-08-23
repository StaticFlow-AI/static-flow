---
title: "【源码级拆解】Codex 0.149.0 默认开启特性：CodeMode——解锁工具调用的新篇章"
summary: "Codex 和 Claude Code 各自独立地走到了同一个结论：控制流归模型写的程序，能力归 Harness 的结构化边界。Codex 用 JavaScript 编排工具，Claude Code 用 JavaScript 编排 Agent——层级不同，形状一样。本文从 Codex parallel_tool_calls=false 却仍并行的反常现象出发，逐层拆开 CodeModeOnly、V8 与 Rust Tool Runtime 的分工，再对照 Claude Code Dynamic Workflow 的 agent()/pipeline() 编排体系。"
detailed_summary_zh: |
  两条独立演进的 Agent Harness 路线——OpenAI Codex 和 Anthropic Claude Code——在各自的最新版本中收敛到了同一个架构分工：控制流归模型写的程序，能力归 Harness 的结构化边界。

  第一层铺垫是 Claude Code 的 Bash-first 策略。在低打断路径中，CC 劝模型优先用 Shell 完成读取、搜索和编辑，而不是调用专用的 Read/Edit/Write 工具。本质上是 Harness 用一部分可观测性换取模型的编排自由——Shell 本身就是一门编排语言，能在一次调用内表达多步骤、分支和过滤。

  第二层是 Codex Code Mode 的源码拆解。起点是一个反常现象：Responses Lite 协议明确设定 parallel_tool_calls=false，Codex 却表现得像并行调用了多个工具。答案是 CodeModeOnly 工具模式把兼容的直接工具从顶层隐藏，模型只剩一个 exec 入口；模型返回一段 JavaScript，由独立的 V8 Host 进程执行，通过 Promise.all 产生多个嵌套工具调用，再经 RuntimeEvent::ToolCall 回到 Rust Tool Runtime。并行没有绕过协议，只是发生的层级从服务端响应移到了客户端程序。安全性来自能力收窄而非对生成代码的信任：V8 没有文件系统、网络、Node API，只有注册进 tools 对象的白名单工具可被调用，权限判断始终在 Rust 侧。V8 本身被拆进独立的 codex-code-mode-host 二进制（rusty_v8 预编译静态库压缩后 37 MB），不使用 Code Mode 的用户不必为其付费。

  第三层是演进线。文章用 Codex 自身的 Git 历史印证从通用 js_repl 到受限 Code Mode 的半年收紧过程，再转向 Claude Code 的 Dynamic Workflow：模型写一段 JavaScript，用 agent()/pipeline() 编排子 Agent，中间结果留在脚本变量而不是上下文里，脚本可存盘、可参数化、可分发为 plugin。Codex 编排工具，Claude Code 编排 Agent，抬高了一层但形状相同。两者对"程序不能干什么"的规定几乎一致，对"程序可以怎么组织工作"几乎不设限。
detailed_summary_en: |
  Two independently evolving Agent Harness lines — OpenAI Codex and Anthropic Claude Code — converge in their latest releases on the same architectural split: control flow belongs to a model-authored program, capability belongs to the harness's structured boundary.

  The first layer is Claude Code's Bash-first strategy. In low-interruption permission modes, CC tells the model to prefer Shell for reading, searching, and editing over the dedicated Read/Edit/Write tools. The harness trades observability for model orchestration freedom — Shell is already a composition language that expresses multi-step, branching, and filtering in a single call.

  The second layer is a source-level teardown of Codex Code Mode. It starts from an anomaly: the Responses Lite protocol explicitly sets parallel_tool_calls=false, yet Codex behaves as though it calls multiple tools in parallel. The answer: CodeModeOnly hides compatible direct tools from the top level, leaving the model with only an exec entry point. The model returns JavaScript, executed by a separate V8 host process; Promise.all spawns multiple nested tool calls that travel back to the Rust Tool Runtime via RuntimeEvent::ToolCall. Parallelism didn't bypass the protocol — it moved from the server response to the client-side program. Security comes from capability restriction, not trust in generated code: V8 has no filesystem, no network, no Node API — only whitelisted tools registered in the tools object may be called, and permission checks remain in Rust. V8 itself is packaged in a separate codex-code-mode-host binary (rusty_v8 prebuilt static library: 37 MB compressed); users who don't use Code Mode don't pay for it.

  The third layer is the evolution curve. The article uses Codex's own Git history to trace a six-month narrowing from a general js_repl to the restricted Code Mode, then turns to Claude Code's Dynamic Workflows: the model writes JavaScript that orchestrates subagents through agent()/pipeline(), intermediate results live in script variables rather than the context window, and scripts can be saved, parameterized, and distributed as plugins. Codex orchestrates tools; Claude Code orchestrates agents — one level higher, same shape. Both specify almost identical restrictions on what the program cannot do, and almost no restrictions on how it organizes work.
tags: ["agent harness", "claude code", "codex", "code mode", "programmatic tool calling", "bash", "dynamic workflow", "ultracode"]
category: "AI Engineering"
category_description: "AI Agent、模型工具调用、Harness 架构与工程实现"
author: "ackingliu"
date: "2026-08-23"
featured_image: "images/delegating-orchestration-from-bash-first-to-codex-code-mode-cover.png"
---

# 【源码级拆解】Codex 0.149.0 默认开启特性：CodeMode——解锁工具调用的新篇章

这篇文章源于一次个人探索。用最新 Codex 写代码时我注意到它经常一口气回来好几组命令的结果，看起来像是并行工具调用。但去源码里一翻，Responses Lite 分支明确把顶层 `parallel_tool_calls` 设成了 `false`——那并行到底是哪来的？顺着这个疑问去看了 Codex 最新源码，就发现了 Code Mode 这套机制。

> 其实在此之前先是发现了最新 Claude Code 的 Bash-first 的 prompt 以及对应的源码判断（这也是为什么第一章先写这个的原因）。

读完源码后我最大的感受是：Codex 和 Claude Code 都在把 Harness 的控制粒度往“代码”这一层推进。Codex 目前让模型写 JavaScript 动态编排工具，Claude Code 则进一步用 JavaScript 动态编排 subagent；层级不同，但形状已经很接近了（虽然严格说 subagent 也是工具）。Harness 负责能力和边界这件事没变，真正变化的是模型开始从“选择下一次 tool call”，走向“直接写程序组织一整段执行流程”。

我不太相信 Codex 只会停在工具层。Code Mode 现在的独立 V8、能力白名单和进程隔离，本身就很像是在为更高层的动态 Agent 编排准备运行时；而 Claude Code 的 Dynamic Workflow 其实很早已经展示了这种模式会是什么样子。

本文的内容如下：

1. **Bash-first 策略** — Harness 为什么要把编排权交给模型？
2. **Codex Code Mode 实现** — 逐层打开：
   - CodeModeOnly 怎么改工具拓扑
   - `exec` 怎么经由 V8 → Rust Tool Runtime 完成嵌套调用
   - 并行为什么没绕过协议
   - 安全性靠什么保证
3. **演进线** — 从 Git 历史看通用 REPL → 受限 Code Mode 的路径
4. **Claude Code Dynamic Workflow** — Codex 下一步大概率要做的东西

> **代码版本**：Claude Code `2.1.236` 本地发行包；OpenAI Codex commit [`343074d4207d`](https://github.com/openai/codex/tree/343074d4207d572809bd8cea15f4be1d09d98e0b)，源码日期 2026-08-22。

## 一、Bash-first——当模型需要编排语言时最自然的解法

最新的 Claude Code 在低打断路径中有一段措辞直接的提示词：

> Do your work through the Bash tool wherever it can accomplish the job: read files with cat, head, or sed -n, search with grep and find, and make file changes with sed, heredocs, or short scripts, rather than using the dedicated Read, Edit, Write tools. Fall back to a dedicated tool only when Bash genuinely cannot do the job.

只要 Bash 干得了，就别去分别调用 `Read`、`Edit`、`Write`。这有点反直觉——专用工具是 Harness 花了力气做出来的，参数结构化、结果可渲染、权限可按字段判断，现在却主动劝模型绕过它们。

原因在于编排成本。假设要找出三个 Rust 模块里的超时配置并比较默认值，专用工具路径是：

```text
模型 -> Grep -> 模型 -> Read -> 模型 -> Read -> 模型 -> 汇总
```

每个箭头都是一次模型往返。而 Bash 能把整件事压成一次调用：

```bash
rg -n "timeout|deadline" crates/{core,server,client}/src \
  | sed -n '1,120p'
```

省下往返只是表面。这一次调用里，模型还多决定了搜索范围、过滤规则、输出上限。这些判断原本摊在多轮 Agent Loop 里，模型一步步给、Harness 一步步做，现在一并压进了一段可执行的程序里。

翻阅 Claude Code `2.1.236` 本地发行包，三种低打断模式都开启了 Bash-first：

```text
bypassPermissions -> Bash-first
auto (steerOnly)  -> Bash-first
bashFirst 开关     -> Bash-first
```

共同点在于都在压缩逐次确认和工具往返，让模型连续做完更长的一段工作。

代价同样明确。一条复合命令可以同时读文件、改文件、联网、拉起子进程，而 Harness 拿到的只是一个字符串。取舍如下：

| 维度           | 专用工具                          | Bash-first                 |
| -------------- | --------------------------------- | -------------------------- |
| 模型编排能力   | 每次表达一个结构化动作            | 一次表达多步骤、分支和过滤 |
| 模型往返       | 通常较多                          | 通常较少                   |
| Harness 可视化 | 参数和结果天然结构化              | 需要理解命令及其组合关系   |
| 权限边界       | 可以按工具和字段判断              | 往往需要分析整条命令       |
| 错误处理       | 返回模型后重新决策                | 可在脚本内处理已预见错误   |
| 回滚能力       | 取决于 Harness 是否实现事务或快照 | 同样不天然具备回滚         |

Bash-first 的本质：Harness 用一部分可观测性，换模型的编排自由。这个交换值不值得，取决于模型有多强。带着这个视角我们再切到 Codex。

## 二、Codex Code Mode——解锁工具调用的新玩法

### 2.1 矛盾：parallel_tool_calls 关了，但行为像并行

起初我还以为并行工具调用与默认开启了 `parallel_tool_calls` 有关，结果深入代码发现并非如此。

Codex 构造 Responses 请求的地方明确写着，只要走 Responses Lite，顶层并行工具调用就被关掉：

```rust
parallel_tool_calls: prompt.parallel_tool_calls && !model_info.use_responses_lite,
```

顶层 flag 是 `false`，服务端返回的响应里不该出现多个并列的 `tool_call`。那些"同时执行的命令"只能来自别的地方。

### 2.2 答案：CodeModeOnly 隐藏直接工具，模型只看到 `exec`

Codex 在模型元数据上定义了三种工具模式：

```rust
pub enum ToolMode {
    Direct,
    CodeMode,
    CodeModeOnly,
}
```

`CodeModeOnly` 把兼容 Code Mode 的直接工具从顶层**隐藏**，转而注册为 `exec` 内部可调用的嵌套工具：

```text
Direct
模型 -> exec_command / view_image / MCP tool / ...

CodeModeOnly
模型 -> exec(JavaScript)
             -> tools.exec_command(...)
             -> tools.view_image(...)
             -> tools.mcp__server__tool(...)
```

顶层根本没有别的选项，不需要“请优先使用 `exec`”之类的 prompt。模型会走 `exec`，因为它没别的可走。

### 2.3 执行链：模型 → JavaScript → V8 callback → Rust Tool Runtime

模型返回的是一段原始 JavaScript——不是 JSON，也不是带引号的字符串：

```js
const [client, protocol] = await Promise.all([
  tools.exec_command({
    cmd: "rg -n 'parallel_tool_calls' codex-rs/core/src/client.rs"
  }),
  tools.exec_command({
    cmd: "rg -n 'CodeModeOnly' codex-rs/protocol/src/openai_models.rs"
  })
]);

text({ client, protocol });
```

这段代码的路径：

```text
模型生成 custom_tool_call: exec
       |
       v
Codex Core 把源码交给独立 Code Mode Host
       |
       v
V8 创建新的 isolate，作为 async module 求值
       |
       v
JavaScript 调用全局 tools.* 方法
       |
       v
V8 callback 生成 RuntimeEvent::ToolCall，返回一个待解析的 Promise
       |
       v
Codex Core 把嵌套调用交回普通 Rust Tool Runtime
       |
       v
权限、Sandbox、并发准入与具体工具执行
       |
       v
结果解析 Promise，JavaScript 继续归并和筛选
       |
       v
text()/image()/audio() 汇成一个 custom_tool_call_output
```

V8 承担的是控制流：变量、循环、条件、`try/catch`、Promise、结果归并。它阉割了大部分系统能力——没有 Node，没有文件系统，没有网络，没有 `console`，只有 Harness 允许的全局帮助函数和一个 `tools` 对象。

> JavaScript 是控制面，Rust 工具是能力面。

### 2.4 并行的实现：Promise.all + Rust 读写锁

回到那个矛盾。Responses Lite 关掉的是一个响应里的多个**顶层**工具调用。而 Code Mode 的模型响应，顶层永远只有一个：

```text
模型响应
- custom_tool_call: exec
```

`exec` 内部调了几个工具，是本地 JavaScript 和 Rust Runtime 共同完成的。模型用 `Promise.all` 同时发出多个嵌套调用，Rust 侧按工具自身是否支持并行来决定准入——支持并行的调用共享读锁，不支持的取写锁串行执行。

与 `parallel_tool_calls` 相比，并行发生的位置变了：

```text
parallel_tool_calls
服务端：多个顶层工具调用（Bash、Read、Search）
客户端：执行多个工具调用

Code Mode
服务端：一个顶层 exec
客户端：exec 内部产生多个嵌套工具调用
```

同一段程序还能表达依赖关系、条件分支、重试、过滤、聚合和提前退出——这些是 `parallel_tool_calls` 无论怎么调都表达不出来的。

### 2.5 安全性来自能力收窄

Code Mode 的安全模型由几个点共同构成：

1. **运行进程分离**：Code Mode session 跑在独立 Host 里，按需启动。
2. **语言环境收窄**：没有 Node、文件系统、网络和 `console`。
3. **能力白名单**：只有注册进 `tools` 对象的工具可被调用，`exec` 也不能递归调用自己。
4. **复用原有 Tool Runtime**：嵌套调用重新走 Rust 侧的正常工具路径。
5. **权限与 Sandbox 不下放**：具体工具仍负责执行前的权限判断和参数校验。
6. **并发准入由 Harness 决定**：不是 `Promise.all` 说了算。
7. **输出显式化**：只有经 `text()`/`image()`/`audio()` 提交的内容才进入最终输出，并受 token 上限约束。

这是**能力安全**，不是对模型生成代码的信任。模型拿到了自由的控制流，却拿不到任何未注册的副作用能力。

三种方案对比：

| 方案                   | 编排语言          | 产生副作用的执行器   | Harness 保留的结构           |
| ---------------------- | ----------------- | -------------------- | ---------------------------- |
| 专用工具               | 模型与 Agent Loop | 每个独立工具         | 最完整，但编排往返多         |
| Claude Code Bash-first | Shell             | Shell 命令及其子进程 | 主要保留命令级边界           |
| Codex Code Mode        | JavaScript        | Rust 侧受控工具      | 保留嵌套工具、参数和结果结构 |

### 2.6 V8 的封装：独立二进制、体积隔离、fail-closed

V8 没有进 Codex 主 CLI。全 workspace 只有两个 crate 依赖 `v8`，引入它的 `code-mode-runtime` 唯一的下游是 `code-mode-host`：

```text
codex（主 CLI bin）              ── 不依赖 v8
codex-code-mode-host（另一个 bin） ── code-mode-runtime ── v8 150.4.0
```

rusty_v8 的预编译静态库压缩后就是 37 MB。拆进程同时办了两件事：进程级隔离和**体积隔离**——不用 Code Mode 的用户不必为此付费。

主 CLI 靠约定找 host 可执行文件：

```rust
// codex-rs/install-context/src/lib.rs:172-179
pub fn code_mode_host_program(&self) -> PathBuf {
    self.bundled_resource(CODE_MODE_HOST_EXECUTABLE_NAME)
        .map_or_else(
            || self.code_mode_host_program_from_exe(std::env::current_exe().ok().as_deref()),
            AbsolutePathBuf::into_path_buf,
        )
}
```

找不到时的降级分两档：

```rust
// codex-rs/core/src/tools/code_mode/mod.rs:101-113
let behavior = match tool_mode {
    ToolMode::Direct => "Falling back to direct tools",
    ToolMode::CodeMode | ToolMode::CodeModeOnly => "Code mode will fail closed",
};
```

`Direct` 能回退到直接工具，`CodeModeOnly` 只能 fail closed——兼容工具已从顶层隐藏，host 缺失意味着模型手里既没有 `exec` 也没有那些直接工具。

rusty_v8 由 Deno 维护，默认不从源码编译 V8 而是直接下载预编译静态库。`code-mode-runtime` 还开了 `v8_enable_sandbox`，这个 feature 在 rusty_v8 里强制拉上指针压缩——V8 的沙箱需要把堆指针约束在预留虚拟地址空间内：

```toml
# v8-150.4.0/Cargo.toml:115
v8_enable_sandbox = ["v8_enable_pointer_compression"]
```

“收窄”不只是不注入 Node API，主要是加强 V8 自己的内存层防护。

## 三、演进线——从专用工具到程序化编排

### 3.1 四个阶段

**第一阶段，一个 Schema 对应一个动作。** Function Calling 解决"可靠调用外部能力"：模型选工具、生成参数，Harness 校验后执行，结果交还模型。结构清晰，代价是每个中间结果都可能触发一次模型往返。

**第二阶段，工具数量膨胀。** Agent 能力扩展后工具列表越来越长，Harness 引入命名空间、MCP、延迟加载和 Tool Search。优化的是"怎么让模型找到正确工具"，逐步调用的基本形式没变。

**第三阶段，Bash-first。** Shell 是现成的本地组合语言，让模型优先用它，能立刻砍掉文件探索和批量编辑里的大量往返。代价：编排和副作用混在同一个字符串里。

**第四阶段，程序化工具调用。** Code Mode 把"程序"本身变成一次工具调用：模型用代码描述控制流，代码只能通过显式能力接口影响外部世界。第三阶段的缺口——编排和副作用不分离——正好在这里补上了。

程序化编排划的是一条分工线：

```text
需要新语义判断       -> 回到模型
可由确定性代码完成   -> 留在程序化执行器
可能产生外部副作用   -> 经过 Harness 能力边界
```

### 3.2 Codex Git 历史的印证

上面四个阶段不是事后归纳。Codex 的 Git 历史几乎照着走的，从通用 JavaScript REPL 一步步收紧成能力受限的 Code Mode：

| 日期       | 变化                      | 意义                                 |
| ---------- | ------------------------- | ------------------------------------ |
| 2026-02-11 | 引入实验性 `js_repl`      | 模型开始拥有持久 JavaScript 执行环境 |
| 2026-02-12 | 加入 `js_repl_tools_only` | 尝试把工具调用收敛到 JavaScript      |
| 2026-03-09 | 引入 `code_mode`          | 把程序化工具编排独立成正式概念       |
| 2026-03-13 | 加入 `code_mode_only`     | 从提示偏好升级为工具暴露策略         |
| 2026-03-20 | Code Mode 迁移到 V8       | 摆脱对通用 Node 环境的依赖           |
| 2026-04-24 | 移除旧 `js_repl`          | 两条实验路径完成收敛                 |
| 2026-07-30 | 只通过独立 Host 运行      | 明确进程级运行边界                   |
| 2026-07-31 | 启用受限 V8               | 进一步收窄执行环境                   |

收紧方向一致：起点是通用执行能力（`js_repl`），终点是受限环境加白名单能力。从 `code_mode` 到 `code_mode_only` 是改工具拓扑，最后两次把运行边界从语言层推到了进程层。Code Mode 是一条走了半年的工程路径。

## 四、Claude Code Dynamic Workflow——编排对象从工具抬到 Agent

提到通过 JS 代码控制工具调用，我能想到的是 Claude Code 的 Dynamic Workflow，故再次进行畅想和对比。

Claude Code 的 Dynamic Workflow 用一段 JavaScript 脚本编排子 Agent——和 Codex Code Mode 编排工具是同一个形状，抬高了一层：

```text
Codex Code Mode：     模型 -> JavaScript -> tools.*  -> Rust 受控工具
Claude Code Workflow：模型 -> JavaScript -> agent()  -> 子 Agent（各自再调工具）
```

一个具体的 Workflow 脚本：

```javascript
export const meta = {
  name: 'audit-routes',
  description: 'Audit every route handler for missing auth checks',
}

const found = await agent('List every .ts file under src/routes/.', {
  schema: {
    type: 'object',
    required: ['files'],
    properties: { files: { type: 'array', items: { type: 'string' } } }
  },
})

const audits = await pipeline(found.files, file =>
  agent(`Audit ${file} for missing authentication checks.`, { label: file }),
)

return audits.filter(Boolean)
```

`agent()` 起一个子 Agent，`pipeline()` 按列表逐项分发，body 是带 top-level await 的普通 JavaScript。fan-out、聚合、过滤都写在代码里，中间结果留在**脚本变量**而不是模型上下文里。

脚本可以存进 `.claude/workflows/` 变成斜杠命令、通过 `args` 传参、装进 plugin 带命名空间分发。运行中可以用 `/workflows` 看每个 phase 的 agent 数、token 量和耗时，钻进单个 agent 读它的 prompt 和结果。

它给了一组硬边界，和 Codex Code Mode 的七条限制类似：

- 脚本自己**不能碰文件系统和 shell**——干活的是 agent，脚本只做协调
- 不能 `import()` 加载模块
- 最多 16 个 agent 并发
- 单次运行上限 1000 个 agent

`import()` 限制的理由最能说明设计哲学的一致性：脚本 body 是纯 JavaScript，需要库的活儿放进 agent 的任务里。这跟 Codex 给 V8 的限定几乎一致——控制流归程序，能力归受控执行器。

## 结语

Harness 并不会因为模型拿到编排自由而变得不重要，反而更吃重。模型的编排自由越多，Harness 越需要把能力这一侧守稳：决定哪些工具可见、哪些副作用可执行、哪些调用可以并发，以及整个过程如何被审批、取消、记录和恢复。

两家对“程序不能做什么”的限制几乎一致，但对“程序怎么组织工作”都给了模型很大的自由。Harness 负责定义能力和边界，控制流则越来越多地交给模型生成的程序。

这不像某一家的实现偏好，更像是两条独立演进的路线最后自然收敛到了同一个方向。Agent Harness 的下一阶段，或许不会再有人类来替模型安排每一步，而是把边界做得足够可靠，然后让模型自己决定里面的路怎么走。

## 源码索引

- [`ToolMode::{Direct, CodeMode, CodeModeOnly}`](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/protocol/src/openai_models.rs#L329-L335)
- [Responses Lite 对顶层 `parallel_tool_calls` 的处理](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/core/src/client.rs#L944-L958)
- [`CodeModeOnly` 隐藏兼容的直接工具](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/core/src/tools/spec_plan.rs#L682-L693)
- [收集并注册 Code Mode 嵌套工具](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/core/src/tools/spec_plan.rs#L706-L805)
- [`exec` 的 V8 环境与全局能力说明](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/code-mode-protocol/src/description.rs#L15-L39)
- [创建 V8 isolate 并执行 async module](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/code-mode-runtime/src/runtime/mod.rs#L168-L225)
- [V8 Promise 与 `RuntimeEvent::ToolCall` 的桥接](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/code-mode-runtime/src/runtime/callbacks.rs#L28-L72)
- [嵌套调用重新进入普通 Tool Runtime](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/core/src/tools/code_mode/mod.rs#L327-L376)
- [并行工具使用读锁，串行工具使用写锁](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/core/src/tools/parallel.rs#L140-L160)
- [独立 Code Mode Host](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/code-mode/src/remote_session.rs#L37-L88)
- [`code-mode-host` 的独立 `[[bin]]` 目标](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/code-mode-host/Cargo.toml#L7-L9)
- [`v8` 依赖与 `v8_enable_sandbox` feature](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/code-mode-runtime/Cargo.toml#L18-L24)
- [workspace 精确锁定 `v8 = "=150.4.0"`](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/Cargo.toml#L489)
- [主 CLI 定位 host 可执行文件](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/install-context/src/lib.rs#L172-L196)
- [host 缺失时的降级分档](https://github.com/openai/codex/blob/343074d4207d572809bd8cea15f4be1d09d98e0b/codex-rs/core/src/tools/code_mode/mod.rs#L101-L116)

## 参考资料

- [Claude Code：Choose a permission mode](https://code.claude.com/docs/en/permission-modes)
- [Claude Code：Configure permissions](https://code.claude.com/docs/en/permissions)
- [Claude Code：Orchestrate subagents at scale with dynamic workflows](https://code.claude.com/docs/en/workflows)
- [Claude Code：Subagents](https://code.claude.com/docs/en/sub-agents)
- [Anthropic：Programmatic tool calling](https://platform.claude.com/docs/en/agents-and-tools/tool-use/programmatic-tool-calling)
- [Anthropic：Advanced tool use](https://www.anthropic.com/engineering/advanced-tool-use)
- [OpenAI Codex source](https://github.com/openai/codex/tree/343074d4207d572809bd8cea15f4be1d09d98e0b)
