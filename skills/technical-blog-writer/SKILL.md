---
name: technical-blog-writer
description: >-
  Write, restructure, or review source-verified technical blog posts for StaticFlow
  (single-system source teardowns, multi-system mechanism comparisons, and engineering
  deep-dives) with truth-first premise auditing, reader-first 总-分 narrative flow,
  plain metaphor-free prose, visually inspected 2x diagrams, and bilingual/KaTeX/LanceDB
  publishing hygiene. Use as the default skill whenever the user asks to draft, rewrite,
  polish, or audit an original technical blog post.
---

# Technical Blog Writer

Default house skill for original technical blog posts on StaticFlow (`ackingliu.top`).
Designed to produce clear, source-grounded engineering articles that read naturally from
first paragraph to code index — avoiding rigid specification-table bloat, folksy slang,
robotic AI clichés, and fabricated comparison hooks.

## Skill Routing: Which Writing Skill to Use

| Scenario | Skill to Use | Why |
|---|---|---|
| **Original technical blog post** (~2k–20k chars: source teardown, 2-system mechanism comparison, Agent Harness / Rust / systems / infra post) | **`technical-blog-writer` (this skill)** | Balances source-verified rigor with reader-first narrative flow, plain prose, 1–3 high-signal tables, and `2x` diagrams |
| **Book-length subsystem monograph** (40k+ chars modeled on `ClickHouse 延迟物化深度解析`) | `../deep-dive-blog-style/SKILL.md` | Intentionally enforces 25+ micro-tables, per-stage ASCII progress bars, and reader-routing matrices that bloat normal posts |
| **Internal engineering design doc / ops runbook** | `../tech-impl-deep-dive-writer/SKILL.md` | Optimizes for internal subsystem specs and incident playbooks rather than public blog narrative |
| **Reposting / translating an external article** | `../external-blog-repost-publisher/SKILL.md` | Enforces full-text fidelity to a third-party source rather than original synthesis |

## Load References & Companion Skills
- **Required before outlining & before publishing**: [writing-and-verification-checklist.md](./references/writing-and-verification-checklist.md) — pre-outline fact-audit worksheet, canonical blueprints, prose red-flag rewrite table, automated KaTeX/anchor/parity validator, and scratch-DB safe live-DB write recipe.
- **Required when creating or updating diagrams**: [diagram-rendering-guide.md](./references/diagram-rendering-guide.md) — HTML/CSS/SVG → headless Chrome `2x` PNG pipeline, bilingual typography/anti-crowding rules, and mandatory `read_file` visual inspection gate.
- **Companion skills for summary & DB operations**: `../article-summary-architect/SKILL.md`, `../staticflow-cli-publisher/SKILL.md`.

### Voice Calibration Exemplars (Read-Only in Content LanceDB `articles`)
- `delegating-orchestration-from-bash-first-to-codex-code-mode` — unadorned source-level prose, real observation opening, 3 high-signal tables across the whole post, crisp closing.
- `codex-auto-compact-deep-dive` — execution-loop spine (`run_turn`), provenance-stamped excerpts (`// path:line`), concrete arithmetic examples, honest pros/cons.
- `goodbye-swift-hello-rust` — natural background framing and smooth section transitions without cold, abrupt starts.

---

## Five Non-Negotiable Pillars

### 1. Truth-First Premise & Edge-Case Audit (Before Outlining)
- **Never invent a shared premise or symmetric hook.** Before writing an outline, verify each subject's *actual* optimization goal in primary docs and source code. Never claim two tools share a goal (e.g., "both exist to save tokens") when one deliberately spends *more* tokens to complete large or adversarial tasks reliably while the other minimizes tokens via deterministic execution.
- **Anchor comparisons on real surface similarity.** When two systems optimize for different goals, state the honest reason engineers compare them: their *execution mechanisms look similar on the surface* (e.g., both move multi-step orchestration out of ad-hoc chat turns into executable code/workflows), so tracing how each runs inside or around the execution loop clarifies their boundary.
- **Verify subtle runtime semantics at exact `path:line` before drafting:**
  - Boolean/guard polarity (does `guard: true` mean *run* or *skip*?).
  - Failure semantics (exact retry cap vs total attempts, when a call returns `null` vs throws/aborts, fail-open vs fail-closed fallbacks).
  - Replay, caching, concurrency locks, and deterministic sandbox restrictions.
  - Runtime coupling & availability surfaces (standalone multi-client CLI vs host/SDK-coupled feature).
- **Pin exact versions/commits** in a compact blockquote (`> **代码版本**：...` / `> **Code Version**: ...`) near the top before entering source internals.

### 2. Reader-First Narrative Progression
Structure every post in the order a technical reader builds mental models:
1. **Warm Context Opening (2–4 short paragraphs)**: Never open cold with giant YAML/JSON dumps or textbook definitions. Start from a concrete observation, apparent contradiction, or real workflow shift, then state clearly what question the article answers (plus a short numbered roadmap if >4 chapters).
2. **Concrete Examples Before Abstract "Why"**: Show minimal, noise-trimmed runnable snippets (8–25 lines) of what each system or mechanism *actually is* — explaining non-obvious naming quirks or core primitives up front — *before* asking why their designs diverge.
3. **Ask "Why"**: Transition naturally from the concrete examples to the architectural question (*why* do two similar-looking mechanisms make opposite runtime choices, or *why* does an observed anomaly happen?).
4. **总-分 (Overview → Breakdown) Execution-Loop Spine**:
   - **总 (Overview)**: Map the shared execution loop or end-to-end data flow (e.g., Agent Harness Loop, request pipeline, query lifecycle) in **one** unified chapter + overview diagram showing exact control/capability boundaries.
   - **分 (Breakdown)**: Unpack each system or stage in **runtime execution order** (never file-by-file inventory): stage constraint → mechanism prose → trimmed provenance-stamped code (`// path:start-end`) → edge/failure behavior.
5. **Single Tight Comparison & Trade-Off Chapter**: Consolidate side-by-side comparison, honest limitations/costs (where each approach is slower, more coupled, lossier, or higher-cost), and selection/composition guidance into **one** non-repetitive chapter before `## 结语` / `## Closing`, `## 源码索引` / `## Source Index`, and `## 参考资料` / `## References`. Never split comparison across 2–3 overlapping tail chapters.

### 3. Plain, Streamlined Prose (Zero Metaphors, Zero AI Clichés, 1–3 Tables)
- **Plain, direct technical language**: Keep paragraphs compact (1–4 sentences) with explicit causal links and concrete engineering nouns/verbs.
- **Strictly ban folksy metaphors & slang**: Never use colloquialisms, anthropomorphic gimmicks, or hype tropes (`老大哥`, `捂热发车`, `广播体操`, `光杆司令`, `人肉路由器`, `老鼠屎`, `三板斧`, `看家本领`, `银弹`, `黑魔法`, calling software tools `物种`, etc.).
- **Strictly ban robotic AI buzzwords & outline scaffolding**: Never write hollow jargon (`认知扇出`, `建立直观体感`, `控制平面归属`, `心智负担`, `赋能`, `抓手`, `闭环`, `不攻自破`, `降维打击`) and never leak outline labels into headings (`分述一：...`, `分述二：...`, `横向对比与组合落地`). Write natural, specific human headings.
- **Strict table budget (1–3 high-signal tables)**: Use a table *only* when comparing ≥3 items/dimensions or summarizing a version/metric matrix where 2D structure clearly beats prose. Never put a table in every subsection.

### 4. High-DPI Visual Diagrams + Mandatory `read_file` Inspection
- Replace dense multi-paragraph flow descriptions or cramped ASCII boxes with clean `2x` Retina PNG diagrams (typically 1–4 per article, rendered in paired `zh_*` and `en_*` variants for bilingual posts).
- Render self-contained HTML/CSS/SVG via headless Google Chrome (`--force-device-scale-factor=2`) + Pillow crop per [diagram-rendering-guide.md](./references/diagram-rendering-guide.md).
- **Mandatory Visual Gate**: Before importing images, **always** open every rendered PNG (`zh` and `en`) with `read_file` to inspect the actual pixels. Fix and re-render immediately if any diagram has text overlap, awkward wrapping/clipping, literal Markdown backticks (`` ` ``) printed raw, low contrast, or any label/arrow that contradicts the source audit.

### 5. StaticFlow Bilingual, KaTeX & Live-DB Publishing Hygiene
- **Zero Prose `$` Rule (KaTeX Trap)**: StaticFlow's frontend (`crates/frontend/index.html`) auto-runs inline KaTeX `{ left: '$', right: '$' }` whenever `/(\$\$|\\\[|\\\(|(^|[^\\])\$[^\s$])/` matches article text. Any unescaped `$` in plain prose (`$20`, `$HOME`, `$1`, `${{...}}`) in `content`, `content_en`, or `detailed_summary` can corrupt surrounding prose into broken math italics. Keep all code `$` tokens inside inline backticks or code fences, and write prose currency as `USD 20` / `20 美元`.
- **1:1 Bilingual Structural Parity**: Generate `content_en` and `detailed_summary` (`zh` + `en`) directly in-session (LLM-native article-level writing; never mechanical line-by-line scripts or external model APIs). `content` and `content_en` must match 1:1 in heading hierarchy, code fence count, table count/columns, link targets, and localized `zh`/`en` diagram `/api/images/<sha256>` URLs. Build `detailed_summary` per `../article-summary-architect/SKILL.md` (natural opening sentence + 3–5 `###` sections with concise grounded bullets, zero raw table markdown, zero prose `$`).
- **Exact StaticFlow `#anchor` Matching**: StaticFlow computes heading IDs via `slugifyHeadingText` (lowercase, strip ``[`~!@#$%^&*()+=[\]{}|\\:;"'<>,.?/！？。；：、“”‘’【】（）《》]``, convert `_` to `-`, collapse whitespace/hyphens; note `_` becomes `-` and `—` is *not* stripped). Validate every inline `[...](#anchor)` link with the exact check in [writing-and-verification-checklist.md](./references/writing-and-verification-checklist.md).
- **Live LanceDB Safe Write Protocol**:
  - Keep all drafts, backups, diagrams, and scratch DBs inside `/tmp/<article_work_dir>/`; leave the repository working tree clean.
  - Resolve `sf-cli` in order: `/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow/release/sf-cli` → `./target/release/sf-cli` → `./target/debug/sf-cli` (rebuild if stale; never use legacy `./bin/sf-cli`).
  - Import diagrams via `<cli> write-images --db-path /mnt/wsl/data4tb/static-flow-data/lancedb --dir <img_dir> --generate-thumbnail --no-auto-optimize`.
  - **Live-DB Taxonomy Guard**: When bypassing `write-article` on live `/mnt/wsl/data4tb/static-flow-data/lancedb` (whenever updating an existing live article or when live `taxonomies` has unindexed rows that fail merge-insert), **never** run `write-article`, `backfill-article-vectors`, `delete-rows`, `optimize`, or `ensure-indexes` directly against live DB. Instead, compute 512d `vector_zh` & 384d `vector_en` in `/tmp/<article_work_dir>/scratch_db` (`sf-cli init` + `write-article` + `db backfill-article-vectors`), follow the exact two-case write rule, and delete `scratch_db`:
    - **Inserting a brand-new article row**: assemble the full `ArticleRecord` JSON with the fresh 512d `vector_zh` & 384d `vector_en`, then run `<cli> db --db-path <live_db> upsert-article --json ...` (only when the target `id` does not yet exist and `articles.id_idx` has no conflicting unindexed rows).
    - **Updating an existing live article row**: prefer `<cli> db --db-path <live_db> update-rows articles --where "id = '<id>'" --set ...` (`table.update().only_if(...)`) updating only `content`, `content_en`, `summary`, `detailed_summary`, `read_time`, `vector_zh`, `vector_en`, and `updated_at`, because repeated `upsert-article` (`merge_insert(&["id"])`) fails with `Ambiguous merge inserts` whenever `articles.id_idx` has `unindexed_rows > 0`.
  - **Proxy-Free HTTP `200` Verification**: Unset `http_proxy`, `https_proxy`, `HTTP_PROXY`, `HTTPS_PROXY`, `all_proxy`, `ALL_PROXY` and verify HTTP `200` on `http://127.0.0.1:39180/api/articles/<id>`, `https://ackingliu.top/api/articles/<id>` (when live), every `/api/images/<sha256>`, and every internal `/posts/<slug>` link.

---

## Standard 6-Step Execution Workflow

1. **Audit Primary Sources**: Read pinned repos/specs, verify the true design premise and all edge-case semantics, and record exact `path:line` anchors ([checklist §1](./references/writing-and-verification-checklist.md#1-pre-outline-premise-and-fact-audit)).
2. **Lock the Narrative Spine**: Map the outline onto *Context → Concrete Examples → Ask "Why" → 总-分 Execution Loop → One Comparison Chapter → Closing + Source Index* ([checklist §2](./references/writing-and-verification-checklist.md#2-canonical-article-blueprints)).
3. **Render & Visually Inspect Diagrams**: Build paired `zh`/`en` HTML/CSS/SVG diagrams, render `2x` PNGs via headless Chrome, inspect **every** PNG with `read_file`, and ingest via `sf-cli write-images` ([diagram guide](./references/diagram-rendering-guide.md)).
4. **Draft ZH, EN & Bilingual Summaries**: Write `content` (`zh`), `content_en` (`en`), and `detailed_summary` (`zh`/`en`) in plain, metaphor-free technical prose with 1:1 structural parity ([checklist §3](./references/writing-and-verification-checklist.md#3-prose-calibration-what-to-ban-vs-write)).
5. **Run Pre-Publish Automated Gates**: Run the validation script checking zero prose `$`, exact StaticFlow `#anchor` slugs, 1:1 bilingual parity, and zero raw backticks in diagrams ([checklist §4](./references/writing-and-verification-checklist.md#4-automated-pre-publish-validation-script)).
6. **Publish Safely & Verify Live**: Back up any existing live row, compute scratch-DB vectors, write via `upsert-article` (brand-new row) or `update-rows articles` (existing live row), and verify proxy-free HTTP `200` across article and image endpoints ([checklist §5](./references/writing-and-verification-checklist.md#5-live-lancedb-safe-publication-and-http-verification)).
