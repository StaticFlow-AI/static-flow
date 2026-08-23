# Skeleton and Snippet Forms

Copy the skeleton, then fill it. Snippet forms below are the reusable shapes
that recur throughout the exemplar. Adapt section count to the subject; keep the
order and the mechanics.

## Article Skeleton

```markdown
# <主题> 深度解析

<一句话说明本文讲什么，从哪里到哪里。>

> **代码版本**：本文基于 <project> `<tag-or-commit>` 分析。
> <可选：路径根说明，例如"本文所有代码路径以 deps/lance/ 为根"。>

---

## 目录

1. [引言与背景](#1-引言与背景)
2. [前置知识](#2-前置知识)
3. [<旧方案>实现原理](#3-旧方案实现原理)
4. [<新方案>全局执行流程](#4-新方案全局执行流程)
5. [<新方案>核心组件详解](#5-新方案核心组件详解)
6. [<方案对比>](#6-方案对比)
7. [调试技巧与常见问题](#7-调试技巧与常见问题)
8. [代码索引](#8-代码索引)
9. [References](#9-references)

---

## 1. 引言与背景

### 1.1 什么是 <机制>
<最小可复现片段 + 朴素路径的代价，用真实单位。>

### 1.2 <机制>的通用性
<可迁移的核心思想，以及它还能用在哪些场景；然后明确本文只聊哪一个。>

### 1.3 <机制>演进史
<版本表：版本 | 符号名 | PR/Issue | 时间 | 状态。旧版本的问题 + 实测数字。>

### 1.4 后续章节导览
<「你想了解… | 去哪里看」路由表 + 带时间估计的阅读路径。>

---

## 2. 前置知识

> 💡 如果你已熟悉 <框架背景>，可跳过本章直接阅读 [§4 …](#4-…)。

<### 2.x 逐个铺垫执行模型、数据结构、存储布局。用 #### 示例：X（展示…）
把抽象接口落到一个真实类上。>

---

## 3. <旧方案>实现原理

> 本章帮助你理解<旧方案>的设计思路和为什么被废弃，为理解<新方案>做铺垫。

### 3.1 核心思想
### 3.2 关键组件
### 3.3 执行流程图解
### 3.4 性能问题分析
#### 问题 1：<具体缺陷>
#### 问题 2：…
### 3.5 <新方案>如何解决这些问题
<| 旧问题 | 新方案 | 收敛表。>

---

## 4. <新方案>全局执行流程

本章从全局视角介绍<新方案>的执行流程：<子问题一>、<子问题二>、<子问题三>。

### 4.1 端到端流程图
<调用栈 ASCII 图，标出机制发生在哪一步。>
### 4.2 <入口函数>做了什么（总览）
### 4.3 优化前后的 <计划/结构> 对比
<同一格式画两遍，便于 diff。>
### 4.4 执行期：为什么<不该发生的事>不会发生
<各步骤的详细实现见 [§5 …](#5-…)。>

---

## 5. <新方案>核心组件详解

上一章从全局视角介绍了执行流程，本章按**数据流动顺序**深入每个组件的实现细节。

### 5.0 数据流总览
<编号阶段 ①→⑥ + 关键数据结构表（数据结构 | 作用 | 对应阶段）。>

### 5.1 <阶段一>
<进度标记 → 本节范围句 → 位置/核心函数 blockquote → 机制说明 →
裁剪过的代码 → 举例 → 结果。见下方 Stage Template。>

### 5.2 … 5.6 <其余阶段，同一模板>

---

## 6. <方案对比>

本章对比<N>种方案的优劣，重点讲解<关键机制>，以及<新方案>相比<替代方案>的优势。

### 6.1 三种方案概述
### 6.2 <替代方案>的原理
### 6.3 <新方案> vs <替代方案>：详细对比
#### 为什么<新方案>快一点，但<某项成本>也高一点？
<实测表 + 唯一真实差异 + 复杂度对照 + 什么时候这个成本要紧。>
### 6.4 性能对比总结

---

## 7. 调试技巧与常见问题

### 7.1 如何确认<机制>是否生效
### 7.2 常见的优化失败原因
### 7.3 调试设置
### 7.4 性能对比测试方法
### 7.5 常见问题排查

---

## 8. 代码索引

### 8.1 关键代码路径索引表
### 8.2 关键函数索引

---

## 9. References

### <子主题一>
### <子主题二>
```

## Snippet Forms

### Version pin

```markdown
> **代码版本**：本文基于 ClickHouse `v26.1.1.1-new` tag 分析。
```

### Reader routing table

```markdown
| 你想了解... | 去哪里看 |
|------------|---------|
| <框架执行原理> | [§2 前置知识](#2-前置知识) |
| <旧方案为什么被废弃> | [§3 …](#3-…)（可跳过） |
| <如何调试> | [§7 调试技巧](#7-调试技巧与常见问题) |

**推荐阅读路径**：

- **快速了解**（~15 分钟）：直接跳到 §4 看整体流程，再看 §6 理解设计决策
- **深入源码**（~1 小时）：§2 → §4 → §5，按数据流顺序理解每个组件
- **实际使用**：§4 了解原理后，直接看 §7 学习调试技巧
```

### Evolution table

```markdown
| 版本 | 函数名 | PR/Issue | 时间 | 状态 |
|------|--------|----------|------|------|
| V1 | `optimizeLazyMaterialization` | [PR #55518](url) | 2023-10 创建，2025-03 合入 | 已废弃 |
| V2 | `optimizeLazyMaterialization2` | [PR #90309](url) | 2025-11 创建，2025-12 合入 | 当前使用 |
```

### Stage template (§5.x, repeat verbatim per stage)

````markdown
### 5.1 列裁剪

```
当前位置：[① 列裁剪] → ② → ③ → ④ → ⑤ → ⑥
                ↑ 你在这里
```

本节讲解数据流的第一步：如何将原始列拆分为主列和延迟列。

> **位置**：`src/Processors/QueryPlan/Optimizations/optimizeLazyMaterialization.cpp:573-689`
> **核心函数**：`optimizeLazyMaterialization2()` + `keepOnlyRequiredColumnsAndCreateLazyReadStep()`

#### 5.1.1 前置条件检查

优化器的"门卫"，只有满足所有条件才会启用：

```cpp
// optimizeLazyMaterialization.cpp:575-607
bool optimizeLazyMaterialization2(QueryPlan::Node & root, ...)
{
    // 必须是 LimitStep → SortingStep → ... → ReadFromMergeTree 结构
    auto * limit_step = typeid_cast<LimitStep *>(root.step.get());
    if (!limit_step || limit_step->withTies()) return false;
    ...
}
```

**示例**：`SELECT a, b, upper(c) FROM t ORDER BY a LIMIT 10`
- `a` 是排序列 → 主列
- `b`, `c` 不参与排序 → 延迟列

**结果**：
- 原 `ReadFromMergeTree` → 只读主列
- 新 `LazilyReadFromMergeTree` → 只读延迟列
````

### Trap-and-resolution table

```markdown
| 问题 | 纯 Pull 的困境 | 纯 Push 的困境 | ClickHouse 的解决方案 |
|------|---------------|---------------|----------------------|
| **LIMIT** | ✅ 停止请求即可 | ❌ 生产者不知道何时停止 | ✅ `setNotNeeded()` 通知上游 |
| **实现复杂度** | ✅ 简单 | ❌ 复杂 | ⚠️ 中等（状态机需仔细设计） |
```

### Insight callout

```markdown
> 💡 **关键洞察**：`__global_row_index` 是两个分支之间的"桥梁"。
> 1. <第一层含义>
> 2. <第二层含义>
>
> 这种设计使得 <系统> 既能 <收益一>，又能 <收益二>。
```

### Terminology guard

```markdown
> ⚠️ **术语区分**：
> - **V1/V2**：延迟物化的两个实现版本
> - **first pass/second pass**：QueryPlan 优化的两遍遍历
>
> 这是两个不同的概念。
```

### Benchmark pair with loss column

```markdown
根据实测数据（hits 数据集，LIMIT 100000）：

| 方案 | 耗时 | 内存峰值 |
|------|------|----------|
| V2 延迟物化 | 0.623 sec | 887.49 MiB |
| AST 重写 | 0.737 sec | 747.00 MiB |

**唯一的差异在第 5 步——顺序恢复**：

| 方案 | 顺序恢复方式 | 时间复杂度 |
|------|-------------|-----------|
| V2 | `inverted_permutation` 单次遍历 | O(n) |
| AST 重写 | 外层 `ORDER BY` 二次排序 | O(n log n) |

这就是 V2 快 15% 左右的原因：省去了一次完整的排序操作。
```

### Debugging chapter core

```markdown
### 7.1 如何确认<机制>是否生效

**方法 1：EXPLAIN PLAN**
<命令 + 生效时会看到什么。>

**方法 2：查看日志**
<日志开关 + 搜什么关键字。>

### 7.2 常见的优化失败原因

| 原因 | 说明 |
|------|------|
| LIMIT 过大 | 超过 `max_limit_for_lazy_materialization` |
| 非 MergeTree 表 | 只支持 MergeTree 系列 |

### 7.5 常见问题排查

**Q: 为什么<机制>没有生效？**

检查：
1. <条件一>
2. <条件二>
```

### Code index

```markdown
| 组件 | 文件路径 |
|------|----------|
| 优化器主文件 | `src/Processors/QueryPlan/Optimizations/optimizeLazyMaterialization.cpp` |
| ColumnLazy (V1) | `src/Columns/ColumnLazy.{h,cpp}` |

| 函数 | 位置 | 作用 |
|------|------|------|
| `calculateGlobalOffset` | optimizeLazyMaterialization.cpp:241 | 计算全局行号 |
```

### Annotated reference entry

```markdown
### 查询执行模型

- [Morsel-Driven Parallelism: A NUMA-Aware Query Evaluation Framework](url) (SIGMOD 2014) - HyPer 团队提出的并行框架，解释了为什么 Push 模型更适合现代多核 NUMA 架构
- [Query Engines: Push vs. Pull](url) - 一篇通俗易懂的博客，用代码示例解释两种模型的区别
```
