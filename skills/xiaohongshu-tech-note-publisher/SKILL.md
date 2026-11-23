---
name: xiaohongshu-tech-note-publisher
description: >-
  Transform technical blog posts (from StaticFlow local articles or external
  URLs) into high-taste Xiaohongshu (小红书) 3:4 Retina content assets —
  single-focus viral covers, full-bleed Swiss editorial inner slides, fluent
  human-voiced copy, and zero-diversion Creator Center publishing. Use whenever
  converting an article into Xiaohongshu cards/copy or publishing/updating a
  Xiaohongshu note.
---

# Xiaohongshu Technical Note Publisher

End-to-end workflow for turning dense engineering articles into high-CTR,
editorial-grade Xiaohongshu (`小红书`) visual notes (`2160 × 2880` Retina PNGs +
title + body copy + interactive `#话题` tags) and publishing them safely through
Xiaohongshu Creator Center.

## Load References & Helpers
- Required for slide rendering:
  [editorial-design-system.md](./references/editorial-design-system.md) —
  `COVER_BASE_CSS`, `EDITORIAL_CSS`, and HTML blueprints for single-focus covers
  plus the 5 Swiss editorial inner-slide motifs.
- Copy & compliance validator:
  [check_xhs_copy.py](./scripts/check_xhs_copy.py) — checks weighted title
  length (`<= 20`), body + `#话题` budget (`<= 970`), and scans for banned
  domains/diversion phrases.

---

## 1. Non-Negotiable Hard Rules

### 1.1 Anti-Throttling & Link Etiquette (`防限流与评论区附链铁律`)
Xiaohongshu throttles (`限流`) overt off-platform diversion across body text,
image OCR, and promotional comments. Obey all three gates unconditionally:
1. **Zero Link Mentions or Diversion Wording in Title/Body (`正文绝不提链接`)**:
   Never write `个人博客`, `已导入博客`, `全文见博客`, `链接放评论区`, `评论区置顶`,
   `见评论区`, `点击主页`, `私信获取`, or any URL/domain (including author domains
   like `t3.gg` or example domains like `webhook.site`) anywhere in title or
   body. Close the post naturally with an engineering takeaway and discussion
   question.
2. **Zero Domains or Diversion Text on Slide Images (`配图防 OCR 铁律`)**:
   Never render any domain name, URL, QR code, or phrases like `完整精翻入库` /
   `链接见评论区` / `评论区聊聊` anywhere on cover or inner slides.
3. **Comment-Section Article Link Etiquette (`评论区自然嵌链规范`)**:
   - When attaching the full-article link in the note's comment section, **never**
     post a bare standalone URL, **never** include `https://` or `http://`, and
     **never** use salesy/promotional hype (`🔗`, exclamation spam, "重磅福利",
     "速戳跳转", "快去我的博客看").
   - Instead, weave the protocol-free address (`ackingliu.top/posts/<slug>`)
     naturally into the middle of a calm, conversational paragraph explaining
     that because the original article is long, a complete Chinese translation
     with clean reading typography has been prepared there for readers who want
     to read the full text comfortably.

### 1.2 Cover Single-Focus Law (`封面单焦点一句流铁律`)
Slide `01` has **only one job**: stop the thumb in Xiaohongshu's 2-column mobile
feed and drive clicks.
- **Zero PPT Chrome**: Strip away outer `.frame` borders, grid wallpaper, top
  header bars (`AI ENGINEERING NOTE` / `01 / 06`), eyebrow pills, subtitle
  paragraphs, bottom pill strips, and bottom footer bars.
- **Single Visual Anchor**: Render **ONE high-tension, scroll-stopping
  statement** (`79px–168px` ultra-bold display type) on a full-bleed warm ivory
  radial canvas (`#FFFDF8` → `#F5F0E6`), keeping **65%+ pure negative space**,
  surgical terracotta (`#C85A28`) emphasis on 1–2 focal tokens, and a single
  quiet bottom-left byline.
- **Viral Hook Formulas** (always render **3 distinct cover candidates** `01`,
  `v2`, `v3` for user selection):
  - *Official Team / Lab Post*: **`[X 团队最新发文 / 刚发新文 / 最新揭秘] + [干了什么惊天动地的反常识事实]`**
    (e.g. `Claude 团队最新揭秘：/ 90% 代码全靠 AI 自产自审 / 首次亮出「防崩底牌」`).
  - *Practitioner / Review Post*: **Flagship clash** (`内测完 GPT-6.2 / 我切回了 Opus`),
    **raw emotional thesis** (`为什么我讨厌 / GPT-6.2 Astra`), or **absurd
    quantified regression** (`GPT-6.2 跑了 7 天 / 代码反而慢了 56%`).

### 1.3 Inner Slide Swiss Editorial Law (`内页瑞士杂志排版铁律`)
Slides `02`–`06` must maintain visual continuity with the cover:
- Use the same full-bleed warm radial paper (`#FFFDF8` → `#F4EFE5`) with generous
  margins (`86px 96px`) and `40%+` open breathing room.
- **No boxes-inside-boxes**: Replace nested rounded cards and pink pills with
  `1px` hairlines, `2px` book rules, oversized tabular numbers, and `5px`
  terracotta left-bar pull-quotes.
- **Vary the visual motif across slides** (see
  `references/editorial-design-system.md` for ready-to-adapt templates):
  1. *Oversized Metric Split* (`~90%` | `+10%`)
  2. *Swiss Numbered Hairline List* (`01 / 02 / 03` bounded by `2px` rules)
  3. *Floating Obsidian Terminal* (`#171614` macOS window for CLI/agent scenes)
  4. *Typographic VERSUS Contrast* (faded technical fog vs. giant flush-aligned
     hanging quote `margin-left: -0.42em`)
  5. *Swiss Benchmark Ledger* or *Vertical Timeline Rail* (`Day 01 → 14`)

### 1.4 Fluent Idiomatic Prose Law (`语言自然流畅铁律`)
- Write natural, unhurried, peer-to-peer technical Chinese (`一、二、三` section
  headers, complete grammatical clauses, zero AI-summary `1️⃣ 2️⃣ 3️⃣` emoji
  bullets).
- **Never coin choppy shorthand or forced slang to save character count**
  (strictly avoid awkward compressions like `只砸 3 类测试`, `冒烟自修`, `硬拉`,
  `直出`, `设死`, `喊人`, `多大盘子`, `大扫除修剪`). When trimming length, drop
  secondary sub-points cleanly rather than amputating words inside a sentence.
- Keep Title `<= 20.0` weighted units (CJK = `1.0`, ASCII = `0.5`) and visible
  Body + `#话题` tags `<= 970 / 1000` characters.

---

## 2. End-to-End Asset Generation Workflow

1. **Ingest Source Article**:
   - If given a `ackingliu.top/posts/<slug>` URL or StaticFlow slug, read it
     directly from local StaticFlow (`curl -sS http://127.0.0.1:39180/api/articles/<slug>`
     or `sf-cli api get-article`).
   - Extract: (a) who wrote it & why it matters *now*, (b) the 1–2 most
     counter-intuitive quantified facts, (c) the 3–5 core mechanisms/lessons,
     and (d) concrete contrasts/quotes that dramatize the point.
2. **Design Deck Spine (`01` Cover × 3 Variants + `02`–`06` Inner Slides)**:
   - Map each inner slide to a distinct Swiss motif from
     `references/editorial-design-system.md` so no two adjacent slides share the
     same layout skeleton.
3. **Render Retina PNGs (`2160 × 2880`) & Assert Zero Overflow**:
   - Render at `1080 × 1440` CSS viewport with `deviceScaleFactor: 2` using the
     Playwright Node 24 binary (`/home/ts_user/.nvm/versions/node/v24.18.0/bin/node`)
     and `/usr/bin/google-chrome`.
   - Log `{ clientH, scrollH, clientW, scrollW }` per slide and require
     `scrollH === 1440` and `scrollW === 1080`.
   - Export PNGs to both `<Artifact Dir>/` and Windows folder
     `/mnt/c/Users/23946/Downloads/xhs-<slug>/`.
4. **Mandatory Visual Self-Inspection Gate**:
   - Call `read_file` on **every rendered PNG** (`01`, `v2`, `v3`, `02`..`06`).
   - Fix any single-character orphan wrap (`孤字换行`), cramped right margin on
     `nowrap` lines, or unaligned opening quote before presenting to the user.
5. **Validate Copy & Write Kit Artifact**:
   - Run `python3 skills/xiaohongshu-tech-note-publisher/scripts/check_xhs_copy.py`
     to verify title weights (`<= 20`), body + tags length (`<= 970`), and zero
     forbidden domains/diversion terms.
   - Present the kit in a user-facing Markdown artifact with ````carousel blocks.

---

## 3. Creator Center Browser Publishing Workflow

1. **Dual Staging for WSL + Windows Webview**:
   - The built-in Webview browser runs on the Windows host (`C:\Users\23946\.gemini\antigravity\brain\<conv-id>\browser_sandbox\uploads`).
   - Always copy the selected 6 PNGs (`01` → `06`) to **both**
     `<WSL Artifact Dir>/browser_sandbox/uploads/` and
     `/mnt/c/Users/23946/.gemini/antigravity/brain/<conv-id>/browser_sandbox/uploads/`.
   - Remember `uploads/` is wiped on cross-domain navigation and subagent turn
     exit: if initial navigation from `about:blank` to `creator.xiaohongshu.com`
     clears `uploads/`, keep the subagent paused on `上传图文`, re-copy the PNGs,
     and resume via `send_message` without navigating away.
2. **Safe Image Replacement When Editing Drafts / Existing Notes**:
   - Never click `.title-clear-all` first (dropping to `0/18` can reset the
     editor). Upload the 6 new images **first** (`N + 6 / 18`), wait for all
     spinners to clear, and **then** delete the first `N` old thumbnails (`×` on
     index `0` repeated `N` times) so `6/18` remain in exact `01` → `06` order.
3. **Title, Body, Interactive `#话题` Pills & Completion**:
   - Fill Title (`<= 20`), clear & insert Body via `keyboard.insertText`, then
     append two newlines and type `#Tag` one by one, waiting ~1.2s to click each
     popup suggestion so every tag becomes a blue clickable pill.
   - Click `发布` and confirm listing in `笔记管理`
     (`https://creator.xiaohongshu.com/new/note-manager`).
   - If the user wants the full-article reading reference attached in comments
     (once the note finishes review and is visible on `www.xiaohongshu.com`),
     post a single calm, non-promotional paragraph embedding the protocol-free
     path (`ackingliu.top/posts/<slug>`) mid-sentence without `https://`.
