# Xiaohongshu Swiss Editorial Design System & Slide Blueprints

Canonical CSS tokens and HTML skeletons for `2160 × 2880` (`1080 × 1440` @ `2x`)
Xiaohongshu technical decks.

## 1. Shared CSS Tokens

### 1.1 `COVER_BASE_CSS` (Slide `01` — Single-Focus Poster)

```css
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  width: 1080px;
  height: 1440px;
  overflow: hidden;
  font-family: "Noto Sans CJK SC", "Source Han Sans SC", "PingFang SC", "Microsoft YaHei", -apple-system, sans-serif;
  background: radial-gradient(circle at 50% 42%, #FFFDF8 0%, #F5F0E6 100%);
  color: #161513;
  -webkit-font-smoothing: antialiased;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 120px 104px;
  position: relative;
}
.bg-mark {
  position: absolute;
  top: 90px;
  left: 72px;
  font-family: Georgia, "Times New Roman", serif;
  font-size: 380px;
  line-height: 1;
  color: rgba(200, 90, 40, 0.075);
  user-select: none;
  pointer-events: none;
}
.statement {
  position: relative;
  z-index: 1;
  font-weight: 900;
  letter-spacing: -0.035em;
  line-height: 1.26;
}
.clay { color: #C85A28; }
.hl {
  position: relative;
  display: inline-block;
  color: #C85A28;
  z-index: 1;
}
.hl::after {
  content: "";
  position: absolute;
  left: -8px;
  right: -8px;
  bottom: 10px;
  height: 30px;
  background: rgba(200, 90, 40, 0.14);
  border-radius: 6px;
  z-index: -1;
}
.underline-ink {
  position: relative;
  display: inline-block;
  z-index: 1;
}
.underline-ink::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  bottom: 6px;
  height: 14px;
  background: rgba(200, 90, 40, 0.28);
  border-radius: 4px;
  z-index: -1;
}
.byline {
  position: absolute;
  left: 104px;
  bottom: 92px;
  font-size: 28px;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: #8F897E;
  display: inline-flex;
  align-items: center;
  gap: 14px;
}
.byline-bar {
  width: 36px;
  height: 4px;
  border-radius: 2px;
  background: #C85A28;
  opacity: 0.75;
}
```

### 1.2 `EDITORIAL_CSS` (Slides `02`–`06` — Full-Bleed Swiss Editorial)

```css
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  width: 1080px;
  height: 1440px;
  overflow: hidden;
  font-family: "Noto Sans CJK SC", "Source Han Sans SC", "PingFang SC", "Microsoft YaHei", -apple-system, sans-serif;
  background: radial-gradient(circle at 50% 35%, #FFFDF8 0%, #F4EFE5 100%);
  color: #161513;
  -webkit-font-smoothing: antialiased;
  padding: 86px 96px 84px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}
.mono {
  font-family: "Noto Sans Mono CJK SC", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
}
.kicker-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 22px;
  border-bottom: 1px solid rgba(22, 21, 19, 0.13);
}
.kicker-left {
  display: inline-flex;
  align-items: center;
  gap: 14px;
  font-size: 22px;
  font-weight: 700;
  color: #6B655C;
  letter-spacing: 0.03em;
}
.kicker-num { color: #C85A28; font-weight: 900; font-size: 23px; }
.kicker-page { font-size: 20px; font-weight: 600; color: #9A948A; letter-spacing: 0.06em; }
.headline {
  font-size: 70px;
  font-weight: 900;
  line-height: 1.22;
  letter-spacing: -0.034em;
  color: #161513;
  margin-top: 34px;
}
.clay { color: #C85A28; }
.sage { color: #1F6A37; }
.hl { position: relative; display: inline-block; color: #C85A28; z-index: 1; }
.hl::after {
  content: "";
  position: absolute;
  left: -5px;
  right: -5px;
  bottom: 6px;
  height: 20px;
  background: rgba(200, 90, 40, 0.14);
  border-radius: 5px;
  z-index: -1;
}
.pull-quote {
  border-left: 5px solid #C85A28;
  padding: 4px 0 4px 28px;
  font-size: 27px;
  font-weight: 700;
  line-height: 1.56;
  color: #2A2825;
}
.pull-quote .muted {
  display: block;
  margin-top: 10px;
  font-size: 22px;
  font-weight: 500;
  color: #706A60;
}
.foot-whisper {
  padding-top: 22px;
  border-top: 1px solid rgba(22, 21, 19, 0.11);
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 21px;
  font-weight: 600;
  color: #8F897E;
}
.foot-next { color: #C85A28; font-weight: 700; }
```

---

## 2. Five Canonical Inner-Slide Motifs

Rotate among these 5 motifs so consecutive slides never repeat the same layout:

1. **Motif A — Oversized Metric Split**:
   - Top `2px solid #161513` rule + 2-column giant `96px` numbers separated by a
     `1px` vertical hairline, followed by a 2-column `3px` left-border trait
     grid and a bottom `.pull-quote`.
2. **Motif B — Swiss Numbered Hairline Ledger (`01 / 02 / 03`)**:
   - `border-top: 2px solid #161513; border-bottom: 2px solid #161513;` with
     `88px`-wide oversized terracotta indices (`46px`) paired with crisp `31px`
     item titles + 2-line explanations.
3. **Motif C — Floating Obsidian Terminal (`#171614`)**:
   - Single sleek dark macOS window (`border-radius: 24px`, `box-shadow: 0 24px 56px rgba(22,21,19,0.14)`)
     showcasing a CLI path, real prompt, or failure trace, paired with one
     hairline callout or `.pull-quote` below.
4. **Motif D — Typographic VERSUS Contrast**:
   - Upper muted block showing verbose/naive approach + centered `VERSUS`
     hairline divider + thunderous `72px` single-line quote with flush hanging
     punctuation (`margin-left: -0.42em; white-space: nowrap;`).
5. **Motif E — Vertical Timeline Rail + Bold Closing Rule**:
   - Left `2px` vertical rail with `14px` hollow/terracotta-filled node circles,
     followed by a `border-top: 2px solid #161513` closing thesis statement.

---

## 3. Playwright Render & Telemetry Skeleton

```js
import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';

const require = createRequire('/home/ts_user/rust_pro/static_flow/package.json');
const { chromium } = require('playwright');

async function renderDeck(pages, artifactDir, winDir) {
  fs.mkdirSync(artifactDir, { recursive: true });
  fs.mkdirSync(winDir, { recursive: true });

  const browser = await chromium.launch({
    headless: true,
    executablePath: '/usr/bin/google-chrome',
    args: ['--no-sandbox', '--disable-dev-shm-usage'],
  });
  const context = await browser.newContext({
    viewport: { width: 1080, height: 1440 },
    deviceScaleFactor: 2,
  });
  const page = await context.newPage();

  for (const item of pages) {
    await page.setContent(item.html, { waitUntil: 'load' });
    await page.evaluate(() => document.fonts.ready);
    const metrics = await page.evaluate(() => ({
      clientH: document.body.clientHeight,
      scrollH: document.body.scrollHeight,
      clientW: document.body.clientWidth,
      scrollW: document.body.scrollWidth,
    }));
    if (metrics.scrollH > metrics.clientH || metrics.scrollW > metrics.clientW) {
      throw new Error(`Overflow on ${item.name}: ${JSON.stringify(metrics)}`);
    }
    const outArtifact = path.join(artifactDir, item.name);
    await page.screenshot({ path: outArtifact, type: 'png' });
    fs.copyFileSync(outArtifact, path.join(winDir, item.name));
    console.log(item.name, JSON.stringify(metrics));
  }
  await browser.close();
}
```
