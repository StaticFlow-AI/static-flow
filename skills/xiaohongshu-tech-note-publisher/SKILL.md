---
name: xiaohongshu-tech-note-publisher
description: >-
  Generate 3:4 editorial cards and copy for Xiaohongshu (小红书) technical notes
  and publish via Xiaohongshu Creator Center with strict anti-throttling / zero
  traffic-diversion compliance (never post external-link comments, never write
  blog/comment-link diversion phrases in body copy, and never render domain
  names or diversion prompts on slide images).
---

# Xiaohongshu Technical Note Publisher

Use this skill whenever generating Xiaohongshu (`小红书`) slide cards, drafting
titles/body copy/hashtags, or publishing notes through
`https://creator.xiaohongshu.com/publish/publish?source=official`.

## 1. Anti-Throttling & Zero Traffic-Diversion Rules (HARD RULE)

Xiaohongshu aggressively throttles (`限流`) notes detected as off-platform
traffic diversion (`站外引流`) across comments, post text, **and image OCR**.
Follow every rule below unconditionally unless the user explicitly overrides it
in that exact turn:

1. **NO Diversion Comments (`严禁引流评论`)**
   - Never post comments containing URLs (`http://`, `https://`), bare domains
     (e.g. `ackingliu.top`, `staticflow.cc`, `.com`, `.top`, `.cc`, `.io`,
     `github.com`), or off-platform pointers ("原文链接", "个人博客", "姊妹篇链接").
   - Never use homophone/obfuscated diversion tricks ("点主页", "搜同名",
     "私信领", "置顶看地址", "公号/知乎").
   - Publishing ends once the note itself is published in Creator Center — do
     **not** navigate to `www.xiaohongshu.com` afterward to add or pin a link
     comment.

2. **NO Diversion Wording in Title or Body Copy (`正文严禁引流话语`)**
   - Never include phrases such as `个人博客`, `已导入博客`, `全文见博客`,
     `链接放评论区`, `评论区置顶`, `见评论区`, `看置顶`, `点击主页`, `私信获取`, or
     any external URL / personal domain name anywhere in the title or body.
   - Note: technical config filenames inside inline prose (such as
     `requirements.toml` or `config.toml`) are fine when discussing technical
     content,, provided you avoid off-platform URLs, domains ( avoid even example
     suspicious domains like `webhook.site` in plain body text when avoidable —
     prefer phrasing like `外带测试域名`), and any call-to-action pointing outside
     the post.
   - End the body copy naturally with a concise engineering takeaway or an open
     technical discussion question — never a traffic-diversion footer.

3. **NO Domains or Diversion Text on Slide Images (`配图严禁域名与导流字样 — 防 OCR`)**
   - Xiaohongshu runs OCR on every uploaded slide. Never render `ackingliu.top`,
     `staticflow.cc`, any `.top`/`.com`/`.cc` domain, QR codes, or phrases like
     `完整精翻入库`, `全文见个人博客`, `链接见评论区置顶` in slide headers, footers,
     or badges.
   - Keep slide footers purely editorial/structural (e.g. `AI ENGINEERING NOTE`,
     `左滑看核心拆解 →`, `一图收藏 · 核心架构清单`).

---

## 2. Visual & Copy Style (Author Baseline)

- **Canvas**: `1080 × 1440` CSS pixels rendered with `deviceScaleFactor: 2`
  (`2160 × 2880` PNG, `3:4` vertical Retina) via headless Chromium/Playwright.
- **Palette (Warm Claude-Editorial Minimalist)**:
  - Background: `#FAF8F3` warm cream with subtle `48px` grid (`rgba(20,20,19,0.032)`)
  - Primary ink: `#141413`
  - Secondary muted ink: `#5E5A53` / `#7C776E`
  - Accent terracotta: `#C85A28` (soft tint `#FCF2EC`, border `#E5B8A1`)
  - Card surfaces: `#FFFFFF` and `#F1EEE6`, border `#DEDAD0`
- **Cover Minimalism Law (`封面极简主义铁律`)**:
  - Covers (`01`) must preserve **large warm-cream whitespace (`60%+ 留白`)** —
    never cram architecture cards, multi-column grids, or multi-paragraph boxes
    onto Slide 01.
  - Stick strictly to: 1 small top outline pill → **giant `~100px` 3-line hero
    headline** (1 terracotta accent phrase + 2 crisp dark-ink lines) → 1 short
    `78px` terracotta divider bar → 2 short lead lines (`~36px`) → 1 single row
    of at most 3 minimal oat pills above the footer.
- **Slide Density**: Keep content slides clean, scannable, and generous with whitespace
  (5–7 cards typical). Put detailed mechanism explanations in the body copy
  rather than cramming walls of text onto slides. Verify zero overflow
  (`scrollHeight <= clientHeight`) and no orphan single-character wraps.
- **Title**: `<= 20` Xiaohongshu weighted characters (CJK = `1`, ASCII = `0.5`).
  Direct, high-signal engineering framing (e.g. `OpenAI官宣：全天候Agent安全指南`).
- **Body Copy**: Direct, zero-fluff engineering breakdown (`<= 920` UTF-16 code
  units including interactive `#话题` tags, safely under Xiaohongshu's `1000`
  limit). Remember each interactive `#Tag` adds internal `[话题]# ` overhead.

---

## 3. Creator Center Browser Publishing Workflow

1. **Stage Uploads Before Opening / Without Cross-Domain Navigation**:
   - Stage bare filenames (`01` → `0N`) into `<Artifact Dir>/browser_sandbox/uploads/`.
   - In WSL + Windows Antigravity Webview setups, the browser runs on the
     Windows host (`C:\Users\<WinUser>\.gemini\antigravity\brain\<conv-id>\browser_sandbox\uploads`),
     so copy the PNGs to **both** the WSL artifact `browser_sandbox/uploads/`
     and `/mnt/c/Users/<WinUser>/.gemini/antigravity/brain/<conv-id>/browser_sandbox/uploads/`.
   - `uploads/` is wiped on cross-domain navigation and subagent turn exit; if a
     login redirect or initial navigation clears `uploads/`, keep the tab open
     on `creator.xiaohongshu.com/publish/publish` (`上传图文`), re-copy the PNGs,
     and upload immediately via `send_message` without navigating away.
2. **Publish Steps**:
   - Open `https://creator.xiaohongshu.com/publish/publish?source=official` and
     switch from `上传视频` to `上传图文`. (If redirected to `/login`, stop and
     let the user scan the QR code; never attempt to bypass login or CAPTCHA.)
   - Upload slides in exact `01` → `0N` order and wait until all thumbnails
     finish uploading (`N/18`, cover #1 verified).
   - Fill Title (`<= 20`) ensuring the live counter updates, insert Body via
     `keyboard.insertText`, then type `#Tag` at the end one by one and select
     each popup suggestion so they become real blue interactive topic tags.
   - Confirm body counter `<= 1000`, click `发布`, and verify `发布成功` /
     `笔记管理` listing. Stop there — **do not post any link or diversion comment**.
