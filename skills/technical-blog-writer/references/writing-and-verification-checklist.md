# Writing and Verification Checklist

Operational companion to `../SKILL.md`. Run §1–§2 before drafting, §3 while writing, §4 before any DB write, and §5 during publication.

---

## 1. Pre-Outline Premise and Fact Audit

Before writing an outline, verify every core claim against primary source code and official docs. Never rely on secondhand summaries or symmetry assumptions.

### Premise Audit Questions
1. **Actual optimization goal**: What does each system's own README/spec say it optimizes for (e.g., completion quality & scale at *higher* token cost vs repeatable determinism at *lower* token cost; latency vs throughput; observability vs orchestration freedom)?
2. **Honest reason to compare (for multi-system posts)**: If their goals differ, what surface similarity makes engineers confuse or compare them (e.g., both express multi-step orchestration as code/workflows around an Agent Harness Loop)?
3. **Version & path pin**: Record exact repo commit SHA / release tag / spec version and path root for the top-of-article blockquote.

### Edge-Case Verification Matrix
Verify every applicable item in source before outlining:
- **Zero pattern-completed keywords (`rg -i` gate)**: Never append plausible-sounding mechanisms (`JSONB`, `Base64` redaction, `stateless` disk behavior) from model intuition — grep the repo for every technical noun you claim exists.
- **Exact API signatures & schema literals**: Verify call forms (e.g., statement `phase(title)` vs callback `phase(title, fn)`), enum strings (`type: behavior` vs `behavior_method`), and exact primitive/global counts.
- **Boolean / guard polarity**: Does `guard: <expr>` run when `true` or skip when `true`? Check both the evaluator and production workflow/test fixtures, and call out counterintuitive polarity in prose when showing an example.
- **Retry, stall, and failure semantics**: Exact constant names and values (e.g., `5` restarts = `6` total attempts?), side effects left on disk across retries, and when an API returns `null` vs throws/aborts (including inside `pipeline()`/`parallel()` vs direct `await`).
- **Determinism, caching & persistence granularity**: Blocked globals (`Date.now()`, `Math.random()`, no-arg `new Date()`, `import()`), exact cache-sharing / stagger release triggers (e.g., when #1's response *begins* vs *finishes*), and local session-replay files vs cross-run artifact datastores.
- **Separate per-feature support matrices**: Never conflate two subcommands' client/mode support (e.g., repo skill installation across 9 clients vs session-mining across 2; interactive pause conditions vs non-interactive `-p` / SDK availability).
- **Exact `path:start-end` line ranges**: Confirm every cited file path and line range resolves to the claimed function/logic at the pinned commit.

---

## 2. Canonical Article Blueprints

Adapt section titles to the concrete topic — **never** copy structural role labels (`分述一`, `总览`, `横向对比与组合落地`) into published headings.

### Blueprint A: Two-System Mechanism Comparison
1. **Opening (unnumbered, 2–4 short paragraphs)**: Real observation or engineering context → why both systems caught attention → the honest comparison question (similar surface mechanics, different goals/boundaries) → compact numbered roadmap + `> **代码版本**` blockquote.
2. **`## 一、<具体跑起来长什么样 / 两个最小实例>`**: Concrete, noise-trimmed runnable example of System A (8–25 lines, clarifying any counterintuitive naming up front) + concrete runnable example of System B → end by asking *why* two similar-looking workflows sit in completely different places at runtime.
3. **`## 二、<它们在 Execution / Harness Loop 里的真实位置>` (总 / Overview)**: One unified execution-loop diagram (`2x` PNG) + crisp explanation of the loop stages and where System A vs System B takes control, spends tokens, and enforces boundaries.
4. **`## 三、<System A 的运行时机制>` (分 / Breakdown 1)**: Walk System A's runtime path in execution order (e.g., DAG/plan expansion → expressions/data flow → secret/sandbox boundary → persistence/resume), anchoring key invariants with short provenance-stamped code excerpts.
5. **`## 四、<System B 的运行时机制>` (分 / Breakdown 2)**: Walk System B's runtime path in execution order (e.g., restricted JS/VM host → dispatch/concurrency primitives → structured output & stall/retry handling → replay/pause behavior).
6. **`## 五、<对比、局限与选型>` (Single Comparison Chapter)**: **One** tight chapter containing: (a) 1 high-signal side-by-side comparison table across core dimensions, (b) honest limitations/costs of each system, and (c) clear decision criteria (including when combining them makes sense and when it is unnecessary overhead).
7. **`## 结语` + `## 源码索引` + `## 参考资料`**: 2–3 short synthesis paragraphs, pinned `path:line` links, and primary references.

### Blueprint B: Single-System Source Teardown
1. **Opening (unnumbered, 2–4 short paragraphs)**: Concrete runtime behavior or counterintuitive observation (e.g., `parallel_tool_calls = false` yet tools run concurrently; UI shows 20% context left yet auto-compact waits) → central thesis → roadmap + `> **代码版本**` blockquote.
2. **`## 一、<前置背景或触发矛盾的最小现场>`**: Minimal reproducible example or baseline mechanism that exposes the cost/contradiction (plus 1 compact trade-off table only if comparing baseline vs new approach).
3. **`## 二、<端到端执行链路总览>` (总 / Overview)**: Full call/data flow in one view (`2x` diagram or clean vertical call chain where simple) separating control plane from capability/storage plane.
4. **`## 三 ~ 四、<按数据流展开核心阶段>` (分 / Breakdown)**: Walk each runtime stage in execution order: problem at this stage → mechanism → trimmed `// path:start-end` code excerpt (typically 6–25 lines) → concrete numeric/state example and failure handling.
5. **`## 五、<设计取舍与边界>` (Single Trade-Off Chapter)**: What this design wins, what it intentionally sacrifices or leaves unsolved (honest loss column / TODOs), and how to verify or debug it in practice.
6. **`## 结语` + `## 源码索引` + `## 参考资料`**.

---

## 3. Prose Calibration: What to Ban vs Write

| Defect Type | Bad (Strictly Banned) | Good (Plain Technical Prose) |
|---|---|---|
| **Folksy slang & anthropomorphic metaphors** | `第一个子 Agent 当“老大哥”先把缓存捂热再发车，避免大家一拥而上做广播体操；失败了也不会因为一颗老鼠屎坏了一锅粥。` | `第一个子 Agent 先单独发送请求预热共享前缀缓存（最长等待 5000 ms），待首个响应开始返回后再并发启动其余子 Agent；单个子 Agent 停滞超限后在 pipeline()/parallel() 内返回 null，不会直接中断整批任务。` |
| **Robotic AI buzzwords & empty meta-talk** | `为了帮读者建立直观体感，我们先对齐控制平面归属的心智模型，从而实现高维度的认知扇出与确定性闭环。` | `先看两段最小可运行示例，弄清它们各自接收什么输入、由谁驱动执行，再拆解背后的运行时差异。` |
| **Fabricated symmetric hook** | `面对高昂的上下文开销，Swamp 和 Dynamic Workflows 都是为了省 Token 而生的编排利器。` | `Swamp 的目标是把重复执行的任务固化成确定性流程以大幅压低 Token 消耗；Dynamic Workflows 则相反，它主动消耗更多 Token 拉起多路子 Agent，换取复杂任务的完成质量与规模上限。之所以把两者放在一起看，是因为它们表面上都在用代码编排多步执行。` |
| **Template-leaking headings** | `## 四、分述二：Dynamic Workflows 深度拆解`<br>`## 五、横向对比与组合落地` | `## 四、Dynamic Workflows：受限 JS 运行时内的子 Agent 编排`<br>`## 五、对比、边界与选型` |

**Quick Prose Rules**:
- Prefer 1–4 sentences per paragraph; split walls of text, but do not chop every single sentence into an isolated one-liner unless rhythm demands it.
- Keep code excerpts short (median 8–20 lines, hard ceiling ~45 lines), elide unrelated lines with `...`, and stamp the first line with `// path/to/file.ext:start-end` (or `# ...` for Python/YAML/TOML).
- Limit tables to **1–3 per article**. If a 2-row table says in 6 cells what one clear sentence can say, write the sentence.

---

## 4. Automated Pre-Publish Validation Script

Run this check in Python against your `/tmp/<work_dir>/` markdown files before touching LanceDB. It verifies StaticFlow's exact `slugifyHeadingText` rules (`crates/frontend/index.html`), catches inline KaTeX `$` traps, and checks ZH/EN structural parity:

```python
import re, sys
from pathlib import Path

PUNCT_RE = re.compile(r"""[`~!@#$%^&*()+=[\]{}|\\:;"'<>,.?/！？。；：、“”‘’【】（）《》]""")

def sf_slugify(text: str) -> str:
    s = text.strip().lower()
    s = PUNCT_RE.sub("", s)
    s = s.replace("_", "-")
    s = re.sub(r"\s+", "-", s)
    s = re.sub(r"-+", "-", s).strip("-")
    return s

def strip_fenced_blocks(md: str) -> list[str]:
    bt, tl = "`" * 3, "~" * 3
    out, fence = [], None
    for line in md.splitlines():
        t = line.strip()
        m = bt if t.startswith(bt) else (tl if t.startswith(tl) else None)
        if m:
            if fence == m:
                fence = None
            elif fence is None:
                fence = m
            out.append("")
            continue
        out.append("" if fence else line)
    return out

def extract_structure(md: str):
    non_fenced_lines = strip_fenced_blocks(md)
    headings, seen = [], {}
    for line in non_fenced_lines:
        m = re.match(r"^(#{1,6})\s+(.+?)\s*$", line)
        if m:
            plain_h = re.sub(r"(`+)([^`]*?)\1", r"\2", m.group(2))
            plain_h = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", plain_h)
            plain_h = plain_h.replace("**", "").replace("__", "").replace("*", "")
            base = sf_slugify(plain_h) or f"heading-{len(headings)+1}"
            cnt = seen.get(base, 0) + 1
            seen[base] = cnt
            hid = base if cnt == 1 else f"{base}-{cnt}"
            headings.append((len(m.group(1)), plain_h, hid))

    cleaned = re.sub(
        r"(`+)([^`]*?)\1",
        lambda m: " CODE" + ("\n" * m.group(2).count("\n")) + " ",
        "\n".join(non_fenced_lines),
    )
    prose_dollars = [
        (i + 1, l) for i, l in enumerate(cleaned.splitlines())
        if re.search(r"(^|[^\\])\$", l)
    ]
    anchors = re.findall(r"\]\(#([^)\s]+)\)", cleaned)
    valid_ids = {h[2] for h in headings}
    bad_anchors = [a for a in anchors if a not in valid_ids]
    fences = len(re.findall(r"^(?:`{3}|~{3})", md, flags=re.M)) // 2
    tables = len(re.findall(r"^\|(?:\s*:?-+:?\s*\|)+\s*$", cleaned, flags=re.M))
    images = re.findall(r"!\[[^\]]*\]\(([^)]+)\)", cleaned)
    return {
        "headings": headings,
        "prose_dollars": prose_dollars,
        "bad_anchors": bad_anchors,
        "fences": fences,
        "tables": tables,
        "images": images,
    }

# Usage: python3 check.py new_content_zh.md new_content_en.md new_summary_zh.md new_summary_en.md
zh_path, en_path, sum_zh_path, sum_en_path = [Path(p) for p in sys.argv[1:5]]
zh, en = extract_structure(zh_path.read_text()), extract_structure(en_path.read_text())
sum_zh = extract_structure(sum_zh_path.read_text())
sum_en = extract_structure(sum_en_path.read_text())

errors = []
for label, info in [("zh", zh), ("en", en), ("summary_zh", sum_zh), ("summary_en", sum_en)]:
    if info["prose_dollars"]:
        errors.append(f"[{label}] Unescaped prose '$' lines: {info['prose_dollars']}")
    if info["bad_anchors"]:
        errors.append(f"[{label}] Broken #anchor links: {info['bad_anchors']} (valid: {[h[2] for h in info['headings']]})")

if [h[0] for h in zh["headings"]] != [h[0] for h in en["headings"]]:
    errors.append(f"Heading level mismatch: zh={[h[0] for h in zh['headings']]} vs en={[h[0] for h in en['headings']]}")
if zh["fences"] != en["fences"]:
    errors.append(f"Code fence count mismatch: zh={zh['fences']} vs en={en['fences']}")
if zh["tables"] != en["tables"]:
    errors.append(f"Table count mismatch: zh={zh['tables']} vs en={en['tables']}")
if len(zh["images"]) != len(en["images"]):
    errors.append(f"Image count mismatch: zh={len(zh['images'])} vs en={len(en['images'])}")

if errors:
    print("VALIDATION FAILED:\n- " + "\n- ".join(errors))
    sys.exit(1)
print("ALL GATES PASSED:", {"headings": len(zh["headings"]), "fences": zh["fences"], "tables": zh["tables"], "images": len(zh["images"])})
```

---

## 5. Live LanceDB Safe Publication and HTTP Verification

### Step 5.1: Resolve Fresh `sf-cli`
Check in order (rebuild with `CARGO_TARGET_DIR=/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow cargo build -p sf-cli --release --jobs 8` only if no other Rust build is running and source is newer):
1. `/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow/release/sf-cli`
2. `./target/release/sf-cli`
3. `./target/debug/sf-cli`

Never use legacy `./bin/sf-cli` snapshots.

### Step 5.2: Import Verified `2x` Diagram PNGs
```bash
SF_CLI=/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow/release/sf-cli
LIVE_DB=/mnt/wsl/data4tb/static-flow-data/lancedb

"$SF_CLI" write-images \
  --db-path "$LIVE_DB" \
  --dir /tmp/<work_dir>/img_upload \
  --generate-thumbnail \
  --no-auto-optimize
```
Note: `ImageRecord.id` equals the lowercase hex SHA-256 of the PNG file bytes, served at `/api/images/<sha256>` (and accepted as `/api/images/<sha256>` or `images/<sha256>` in Markdown).

### Step 5.3: Scratch-DB Vector Computation + Two-Case Live Write Rule
When bypassing `write-article` on live `/mnt/wsl/data4tb/static-flow-data/lancedb` (whenever updating an existing live article, or when live `taxonomies` has unindexed rows that cause `write-article` merge-insert errors), **never** run `write-article`, `backfill-article-vectors`, `delete-rows`, `optimize`, or `ensure-indexes` directly on `$LIVE_DB`:

1. **Inspect & back up any existing live row first**: Run read-only `"$SF_CLI" db --db-path "$LIVE_DB" count-rows articles --where "id = '<article_id>'"` and `"$SF_CLI" db --db-path "$LIVE_DB" list-indexes articles --with-stats`. When the row already exists, back it up (read-only via `"$SF_CLI" api --db-path "$LIVE_DB" get-article <article_id>` or Python `lancedb`) to `/tmp/<work_dir>/backup_article.json`, preserving `created_at` (UTC epoch ms `i64`), `date` (`YYYY-MM-DD`), `featured_image`, `author`, `tags`, `category`, `article_kind`, `source_url`, and `interactive_page_id`.
2. **Generate 512d `vector_zh` and 384d `vector_en` in an isolated scratch DB**:
   ```bash
   SCRATCH=/tmp/<work_dir>/scratch_db
   rm -rf "$SCRATCH"
   "$SF_CLI" init --db-path "$SCRATCH"
   "$SF_CLI" write-article \
     --db-path "$SCRATCH" \
     --file /tmp/<work_dir>/new_content_zh.md \
     --id "<article_id>" \
     --date "<YYYY-MM-DD>" \
     --summary "<concise card summary>" \
     --tags "<tag1,tag2>" \
     --category "<Category>" \
     --category-description "<Category description>" \
     --content-en-file /tmp/<work_dir>/new_content_en.md \
     --summary-zh-file /tmp/<work_dir>/new_summary_zh.md \
     --summary-en-file /tmp/<work_dir>/new_summary_en.md
   "$SF_CLI" db --db-path "$SCRATCH" backfill-article-vectors
   ```
3. **Write to `$LIVE_DB` using the exact two-case rule, then delete `scratch_db`**:
   - **Case A — Inserting a brand-new article row** (only when the target `id` does not yet exist in `articles` and `articles.id_idx` has no conflicting unindexed rows): assemble `/tmp/<work_dir>/new_record.json` matching `ArticleRecord` (`id`, `title`, `content`, `content_en`, `summary`, `detailed_summary` as JSON string `{"zh":...,"en":...}`, `tags`, `category`, `author`, `date`, `featured_image`, `read_time`, `article_kind`, `source_url`, `interactive_page_id`, non-null 384d `vector_en`, non-null 512d `vector_zh`, `created_at` and `updated_at` in UTC ms `i64`), then run:
     ```bash
     "$SF_CLI" db --db-path "$LIVE_DB" upsert-article \
       --json "$(cat /tmp/<work_dir>/new_record.json)"
     rm -rf "$SCRATCH"
     ```
   - **Case B — Updating an existing live article row**: prefer `"$SF_CLI" db --db-path "$LIVE_DB" update-rows articles --where "id = '<article_id>'" --set ...` (`table.update().only_if(...)`) updating **only** `content`, `content_en`, `summary`, `detailed_summary`, `read_time`, `vector_zh`, `vector_en`, and `updated_at` (leaving `date`, `created_at`, `featured_image`, `tags`, `category`, `article_kind`, etc. untouched on the live row), because repeated `upsert-article` (`merge_insert(&["id"])`) fails with `Ambiguous merge inserts` whenever `articles.id_idx` has `unindexed_rows > 0`:
     ```python
     import lancedb, subprocess, time

     scratch_row = lancedb.connect("/tmp/<work_dir>/scratch_db").open_table("articles").to_arrow().to_pylist()[0]
     sql_str = lambda s: "'" + s.replace("'", "''") + "'"
     vec_sql = lambda v, dim: f"arrow_cast([{','.join(f'CAST({x} AS FLOAT)' for x in v)}], 'FixedSizeList({dim}, Float32)')"
     now_ms = int(time.time() * 1000)

     subprocess.run([
         "/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow/release/sf-cli",
         "db", "--db-path", "/mnt/wsl/data4tb/static-flow-data/lancedb",
         "update-rows", "articles",
         "--where", f"id = {sql_str(scratch_row['id'])}",
         "--set", f"content={sql_str(scratch_row['content'])}",
         "--set", f"content_en={sql_str(scratch_row['content_en'])}",
         "--set", f"summary={sql_str(scratch_row['summary'])}",
         "--set", f"detailed_summary={sql_str(scratch_row['detailed_summary'])}",
         "--set", f"read_time={int(scratch_row['read_time'])}",
         "--set", f"vector_zh={vec_sql(scratch_row['vector_zh'], 512)}",
         "--set", f"vector_en={vec_sql(scratch_row['vector_en'], 384)}",
         "--set", f"updated_at=arrow_cast({now_ms}, 'Timestamp(Millisecond, None)')",
     ], check=True)
     ```
     Then clean up `rm -rf "$SCRATCH"`.

### Step 5.4: Proxy-Free Live HTTP `200` Verification
Always unset local proxy env vars so checks hit local Pingora (`127.0.0.1:39180`) and public Caddy directly:
```bash
env -u http_proxy -u https_proxy -u HTTP_PROXY -u HTTPS_PROXY -u all_proxy -u ALL_PROXY bash -c '
  curl -fsSI "http://127.0.0.1:39180/api/articles/<article_id>" | head -n 5
  # When already routed publicly:
  curl -fsSI "https://ackingliu.top/api/articles/<article_id>" | head -n 5
  # Verify every embedded /api/images/<sha256> and internal /posts/<slug>:
  for sha in <sha1> <sha2>; do
    curl -fsSI "http://127.0.0.1:39180/api/images/$sha" | head -n 1
  done
'
```
Confirm via `"$SF_CLI" api --db-path "$LIVE_DB" get-article <article_id>` (or Python `lancedb`) that `vector_zh` (len `512`), `vector_en` (len `384`), `featured_image`, `date`, `content_en`, and `detailed_summary` are all intact, and verify `git status --short` in `/home/ts_user/rust_pro/static_flow` has zero stray temp files.
