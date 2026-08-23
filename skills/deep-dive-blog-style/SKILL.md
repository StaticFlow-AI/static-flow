---
name: deep-dive-blog-style
description: Write long-form source-verified technical deep-dive blog posts in the author's established voice — numbered sections with a navigable TOC, quantified problem framing, data-flow-ordered mechanism walkthroughs with "you are here" markers, comparison tables for every design alternative, trimmed code excerpts anchored to pinned source locations, and an annotated references section. Use when writing or restructuring a deep-dive article about a real codebase, or when reviewing a draft for style conformance.
---

# Deep-Dive Blog Style

The house style for long-form technical articles on ackingliu.top. Derived from
`ClickHouse 延迟物化深度解析` (~55k chars, 2066 lines), which is the reference
exemplar for this skill.

The governing idea: **the reader is an engineer who will verify your claims
against the source tree.** Every mechanism is explained before its code, every
number is measured, every design choice is compared against the alternative
that was rejected, and every claim is anchored to a pinned commit or tag.

## Load References
- Required: `references/style-contract.md` — measured texture targets and hard rules
- Required: `references/skeleton-template.md` — section skeleton and reusable snippet forms

## Relationship to `tech-impl-deep-dive-writer`

Both cover mechanism-first technical writing. Use this skill for **published
blog articles**, where reader navigation and quantified evidence matter. Use
`tech-impl-deep-dive-writer` for **internal implementation docs**.

They conflict on one rule: that skill forbids question headings; this style uses
them deliberately at `####` depth for "why" mechanism questions (4 occurrences
in the exemplar). This skill's rule wins for blog articles. See
`references/style-contract.md` §1.

## Non-Negotiables

1. **Pin the code version** in a blockquote directly under the H1, before any
   prose. Tag, branch, or commit — plus what path root the references assume.
2. **Number every section**, `## 1.`, `### 1.1`, `#### 1.1.1`, and open with a
   TOC of anchor links to the `##` level.
3. **Quantify the problem before naming the solution.** Open with a concrete
   snippet and the cost of the naive path in real units, not a definition.
4. **Never claim a performance win without measured numbers** in a table, with
   the dataset and cache state stated.
5. **Every design decision gets a comparison table** against what was rejected.
6. **Prose paragraphs stay short and single-line.** Median ~50 characters; every
   paragraph in the exemplar is one unwrapped line. Past ~170 characters it
   should be a table, a list, or two paragraphs.
7. **Prose hands off to artifacts.** Nearly half of all prose lines (47%) end in
   `：` and introduce the code block, table, or diagram directly below. Prose is
   connective tissue; the artifact is the payload.
8. **Code excerpts are trimmed and provenance-stamped.** Overall median 7 lines
   (`cpp` median ~17, hard ceiling ~57). Every source excerpt opens with a
   `// path:startLine-endLine` comment. Elide boilerplate with `...` and keep
   only the lines the prose discusses. Annotate in the article's language —
   27% of excerpt lines are the author's own comments, not upstream's.
9. **State limitations honestly.** Where a chosen design is worse, say so and
   size the cost. The exemplar devotes a whole `####` to why the winner is also
   the heavier option.

## Workflow

1. **Establish the source baseline.** Check out the exact tag/commit. Record it
   for the version-pin blockquote. Every path and line number in the article
   must resolve in that tree.
2. **Find the quantified hook.** The smallest snippet that exposes the problem,
   plus the cost of the naive path. This becomes §1.1.
3. **Generalize, then scope.** State the transferable idea and where else it
   applies (§1.2), then narrow explicitly to what the article covers.
4. **Build the evolution table** if the mechanism has version history: version,
   symbol name, PR/Issue link, dates, status. Then show why the old version
   failed, with measurements.
5. **Write the navigation aids** (§1.4): a "what you want → where to look"
   table, and reading paths with time estimates.
6. **Decide the prerequisites chapter.** If the mechanism needs framework
   context, write it as §2 with explicit permission to skip.
7. **Establish the data flow spine.** Number the stages. This ordering governs
   the mechanism chapters — organize by flow, never by file.
8. **Walk each stage** with the fixed stage template: progress marker → scope
   sentence → source pointer → mechanism prose → trimmed code → worked example
   → result.
9. **Compare against alternatives**, including the manual workaround a user
   might write by hand. Locate the single real difference, then measure it.
10. **Add the debugging chapter**: how to confirm the mechanism fired, why it
    silently does not, the settings to toggle, and an A/B measurement recipe.
11. **Close with the appendices**: code index (paths and functions), then
    references grouped by subtopic with a one-line annotation on each.
12. **Run the quality gates** in `references/style-contract.md` §6.

## Structural Signature

Target proportions, measured from the exemplar. Scale with length; hold ratios.

| Element | Exemplar count | Rule |
|---|---|---|
| `##` chapters | 10 | Numbered, plus TOC / Code Index / References |
| `###` / `####` | 42 / 47 | `####` carries the mechanism detail |
| Tables | 30 | Roughly one per `###`; comparison is the default shape |
| Code blocks | 71 | Only three fence tags exist: 32 `cpp`, 15 `sql`, 24 untagged |
| `sql` block size | median 2 lines | Just the query under discussion |
| Long code (>40 lines) | 3 | Rare and deliberate |
| Diagram blocks | 23 | Untagged fences; 14 micro-markers, 9 real diagrams |
| `---` rules | 26 | Only ever immediately before a heading |
| Bold spans | 155 | Terms on first use, and conclusions |
| Inline code spans | 210 | Every identifier, path, setting, and type |
| Blockquote lines | 37 | Version pin, source pointers, 💡 insights, skip notes |
| `file:line` refs | 33 | In source-pointer blockquotes and code comments |
| Anchor links | 22 | TOC, navigation table, forward and back references |

## Anti-Patterns

- Opening with a textbook definition instead of a costed example.
- Organizing mechanism chapters by source file rather than by data flow.
- Pasting a function whole when six lines carry the argument.
- Performance claims with no table, dataset, or cache state.
- A design section that presents the chosen approach with no rejected alternative.
- Long unbroken prose paragraphs; a paragraph doing a table's job.
- Bare `file:line` lists as the body of an explanation rather than an appendix.
- An unannotated link dump as the references section.
- Marketing register: "ultimate", "revolutionary", "shocking", "silver bullet".
- Unpinned code references, so the reader cannot reproduce what you read.
