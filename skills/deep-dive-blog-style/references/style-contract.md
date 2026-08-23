# Style Contract

Measured against `ClickHouse 延迟物化深度解析` (2066 lines, 55k chars, 100 headings).
Numbers are targets to scale, not quotas to hit exactly.

## 1. Heading Rules

### Required
- Number the spine: `## N.`, `### N.M`, and `#### N.M.K` where sub-steps are
  sequential. In the exemplar, 9 of 10 `##` and 38 of 42 `###` are numbered.
- Leave `####` free-form where it carries a rhetorical beat rather than a step:
  28 of 47 are unnumbered. Two productive unnumbered forms:
  - `#### 名词：一句话解释` — term plus gloss
  - `#### 示例：SortingStep（展示复杂 Step 的设计）` — a worked example that
    announces its own purpose in parentheses
- Use question headings at `#### ` depth for genuine "why" mechanism questions.
  `#### 为什么外层必须有 ORDER BY？` earns its place; it names a trap readers
  actually hit. Keep these to a handful (4 in the exemplar).
- Let section sizes be uneven. The prerequisites chapter ran 763 lines (37% of
  the article) while reference chapters ran 60–107. Depth follows need.

### Forbidden
- Question headings at `##` or `###` depth — the navigable spine stays
  declarative.
- Unnumbered `##`/`###` outside the TOC and the References buckets.
- Marketing register anywhere: "ultimate", "revolutionary", "shocking",
  "you won't believe", "silver bullet".
- Filler chapter names: "其他", "杂项", "一些想法", "Misc".

## 2. Prose Texture

### Required
- One paragraph, one line. All 125 prose paragraphs in the exemplar are single
  unwrapped lines. Median 50 characters, p90 ~86.
- End roughly half of prose lines with `：`, handing off to the artifact below.
  59 of 125 end in `：` versus 52 in `。`.
- Use inline `→` chains and arithmetic phrasing to quantify inside a sentence:
  `读取全部 200 列 → 排序 → 丢弃 99.99999% 的行 → 返回 10 行`.
- Open body chapters with a one-line contract that states scope and vantage
  point, naming the previous chapter when continuity matters:
  `上一章从全局视角介绍了…，本章按**数据流动顺序**深入每个组件的实现细节。`
- Let reference chapters (TOC, debugging, code index, references) drop straight
  to `###` with no preamble.

### Forbidden
- Paragraphs past ~170 characters. Convert to a table, a list, or split.
- Multi-sentence exposition where a comparison table would carry it.
- Restating in prose what the code excerpt's own comments already say.
- Preamble that announces the section without adding scope information.

## 3. Evidence Rules

### Required
- Pin the code version in a blockquote under the H1 before any prose, plus the
  path root the references assume:
  `> **代码版本**：本文基于 ClickHouse \`v26.1.1.1-new\` tag 分析。`
- Stamp every source excerpt with `// path:startLine-endLine` as the first line
  inside the fence. Full path on first mention per chapter, bare filename after.
- Precede each component subsection with the two-line source pointer:
  `> **位置**：path:line-range` then `> **核心函数**：fn() + fn2()`.
- Trim excerpts to the lines the prose discusses; elide with `...`. Overall
  median 7 lines; `cpp` median ~17; ceiling ~57.
- Annotate excerpts in the article's own language. 27% of excerpt lines are the
  author's comments, not upstream's.
- Give every performance claim a table with dataset, cache state, and units:
  `| 方案 | 耗时 | 内存峰值 |`.
- Aggregate all source locations into a code-index chapter with
  component→path and function→location→role tables. Use `{h,cpp}` brace
  notation for header/impl pairs.
- Annotate every reference with one line on why it is worth reading, and group
  references into unnumbered `###` topic buckets. Cite venue and year for
  papers.

### Forbidden
- Unpinned code references — the reader cannot reproduce what you read.
- Pasting a function whole when a handful of lines carry the argument.
- Performance numbers without dataset or cache state.
- A bare link list as the references section.
- `file:line` lists as the body of an explanation rather than an appendix.

## 4. Comparison Rules

### Required
- Give every design decision a table against what was rejected. 30 tables in
  the exemplar, mostly 2–5 columns and 2–5 rows.
- Prefer the four-column trap-and-resolution shape where two approaches each
  fail differently: `| 问题 | 纯 Pull 的困境 | 纯 Push 的困境 | 解决方案 |`.
- Include the manual workaround a reader might write by hand as a real
  contender, not a straw man.
- Locate the single real difference between close alternatives, then measure
  only that. The exemplar reduces a 15% win to one avoided sort, O(n) versus
  O(n log n).
- Report the cost column too. `#### 为什么 V2 比 AST 重写快一点，但内存消耗也高一点？`
  argues against the article's own subject, sizes the 140 MiB, and says when it
  matters.
- Treat a superseded version as the motivating failure, quantified: numbered
  `#### 问题 N：…` subsections, a benchmark proving each, then a
  `| 旧问题 | 新方案 |` resolution table.

### Forbidden
- Presenting the chosen design with no rejected alternative.
- Ranking approaches without explaining the loss column.
- Dismissing the old version as merely bad instead of measuring it.

## 5. Navigation Rules

### Required
- Open with a TOC of ordered anchor links mirroring the numbered `##` headings.
- Reference sections as `[§N 标题](#n-标题)` with the `§` sigil, both forward
  and backward. 22 anchor links in the exemplar.
- Give the reader a routing table: `| 你想了解... | 去哪里看 |`.
- Offer reading paths with time estimates: quick pass, full source read,
  practical use.
- Declare skippability up front where a chapter is optional, with an anchor to
  where to jump: `> 💡 如果你已熟悉…，可跳过本章直接阅读 [§4](#…)`.
- Mark position inside a multi-stage walkthrough:
  ```
  当前位置：[① 列裁剪] → ② → ③ → ④ → ⑤ → ⑥
                  ↑ 你在这里
  ```
- Use `---` only immediately before a heading, never mid-prose.
- Guard terminology collisions with a `⚠️` blockquote at the point two term
  families first meet.
- Reserve `💡` for insight and aside, `⚠️` for warning and disambiguation,
  `✅`/`❌` for verdicts inside tables.

### Forbidden
- Anchor links whose text does not match the target heading.
- `---` as a decorative break inside prose.
- Emoji as decoration outside these four roles.

## 6. Quality Gates

Run all of these before publishing.

1. **Reproducibility** — every path and line number resolves in the pinned
   tree. Spot-check three.
2. **Quantified hook** — §1 costs the naive path in real units before naming
   the solution.
3. **Measured claims** — no performance statement without a table carrying
   dataset and cache state.
4. **Comparison coverage** — every design decision has its rejected
   alternative in a table.
5. **Honest loss column** — at least one section explains where the chosen
   design is worse, with the cost sized.
6. **Flow ordering** — mechanism chapters follow the data flow, and no chapter
   is organized by source file.
7. **Prose discipline** — no paragraph past ~170 characters; roughly half end
   in `：`.
8. **Excerpt discipline** — no unstamped excerpt; at most a few blocks past 40
   lines.
9. **Verification chapter** — how to confirm the mechanism fired, why it
   silently does not, which settings toggle it, and an A/B recipe.
10. **Tail appendices** — code index tables and annotated, bucketed references.
11. **Navigability** — TOC anchors resolve; optional chapters declare
    skippability; multi-stage walkthroughs carry position markers.
