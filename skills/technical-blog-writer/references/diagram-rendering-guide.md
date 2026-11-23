# Diagram Rendering Guide

Companion to `../SKILL.md` Pillar 4. Use this guide whenever a technical blog post needs visual diagrams that clarify execution loops, runtime stages, or isolation boundaries better than prose or ASCII boxes.

---

## 1. When to Use Rendered `2x` PNG Diagrams

Prefer HTML/CSS + inline SVG rendered to `2x` Retina PNG over ASCII art or client-side Mermaid when:
- **Mapping the "总" (Overview) Execution Loop**: Showing where two systems or multiple components sit relative to a shared loop (e.g., User → Agent Harness Loop → Tool/Subagent Runtime → Persistence).
- **Multi-lane or Side-by-Side Architecture Comparisons**: Showing how System A (external CLI / DAG engine) and System B (in-harness restricted VM / subagent dispatcher) differ structurally in one glance.
- **Multi-stage State / Retry / Guard Flows**: Where branch conditions (e.g., `guard` polarity, `null` vs error paths, lock acquisition) become unreadable in multi-paragraph prose or narrow ASCII art.

Keep simple 3–5 line linear call chains in fenced `text` blocks if a full PNG adds no structural insight; typically **1–4 rendered diagrams per article** hits the sweet spot.

For bilingual StaticFlow posts (`content` + `content_en`), **always render localized pairs** (`zh_01_*.png` for `content` and `en_01_*.png` for `content_en`) so English readers never hit Chinese diagram text and vice versa.

---

## 2. Visual, Typography & Anti-Crowding Rules

1. **Clean Editorial Light Canvas (Works in Both Light & Dark Site Themes)**:
   - Use an opaque `#ffffff` or `#f8fafc` outer card with `1px solid #cbd5e1` border and `16px` border-radius so the PNG remains crisp and legible even when the reader switches StaticFlow into dark mode (never use transparent backgrounds with dark text!).
2. **Verified Host Font Stack**:
   - Prose/UI font: `'Inter', 'Noto Sans CJK SC', system-ui, -apple-system, sans-serif`
   - Code/identifier font: `'JetBrains Mono', 'Noto Sans Mono CJK SC', ui-monospace, monospace`
3. **Restrained Semantic Palette (Max 2–3 Functional Accents)**:
   - Neutral structure: `#0f172a` primary text, `#475569` secondary labels, `#cbd5e1` / `#94a3b8` connectors, `#f8fafc` / `#f1f5f9` container fills.
   - Functional accents: e.g., Slate/Blue (`#eff6ff` fill, `#3b82f6` border, `#1d4ed8` title) for deterministic/host runtime; Indigo/Amber (`#eef2ff` or `#fffbeb` fill, `#6366f1` or `#d97706` border) for LLM/Agent turns; Emerald/Rose (`#ecfdf5` / `#fff1f2`) strictly for pass/fail or allow/block paths.
4. **Layout & Anti-Crowding Discipline**:
   - **Use HTML Flexbox / CSS Grid for boxes and text**, reserving `<svg>` strictly for clean arrows or connectors. Hard-coded SVG `<text x="..." y="...">` coordinates frequently clip or collide when switching from compact Chinese to 35%-wider English strings!
   - **Never write literal Markdown backticks (`` `foo` ``) inside HTML or SVG text**. Use `<code>foo</code>` styled with subtle background (`#f1f5f9`), border (`#e2e8f0`), and monospace font.
   - **Generous sizing**: Design at `1080px`–`1240px` logical CSS width (`2160px`–`2480px` physical pixels at `2x` scale), minimum `13px`–`14px` body/badge text, `16px`–`20px` card padding, and explicit `gap` between columns so English labels wrap cleanly without touching borders or arrows.
   - **Fact-match every label**: Every constant, arrow direction, boolean condition, and return type (`null` vs error, `guard` run vs skip) inside a diagram must match the primary-source audit 100%.

---

## 3. Reproducible Headless Chrome `2x` + Pillow Crop Template

Store rendering scripts and outputs under `/tmp/<work_dir>/diagrams/` and `/tmp/<work_dir>/img_upload/` (never inside the git repository).

```python
#!/usr/bin/env python3
import hashlib, subprocess, tempfile
from pathlib import Path
from PIL import Image, ImageChops

CHROME_BIN = "/usr/bin/google-chrome"
OUT_DIR = Path("/tmp/article_work/img_upload")
OUT_DIR.mkdir(parents=True, exist_ok=True)

BASE_CSS = """
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  background: #ffffff;
  padding: 40px;
  font-family: 'Inter', 'Noto Sans CJK SC', system-ui, -apple-system, sans-serif;
  color: #0f172a;
  -webkit-font-smoothing: antialiased;
}
#diagram-card {
  width: 1140px;
  background: #ffffff;
  border: 1.5px solid #cbd5e1;
  border-radius: 16px;
  padding: 28px 32px;
}
code {
  font-family: 'JetBrains Mono', 'Noto Sans Mono CJK SC', monospace;
  font-size: 0.92em;
  background: #f1f5f9;
  border: 1px solid #e2e8f0;
  border-radius: 5px;
  padding: 1px 6px;
  color: #0f172a;
}
"""

def render_diagram(name: str, body_html: str, width: int = 1260, height: int = 2000) -> tuple[Path, str]:
    html = f"<!DOCTYPE html><html><head><meta charset='utf-8'><style>{BASE_CSS}</style></head><body><div id='diagram-card'>{body_html}</div></body></html>"
    with tempfile.TemporaryDirectory() as tmp:
        html_path = Path(tmp) / f"{name}.html"
        raw_png = Path(tmp) / f"{name}_raw.png"
        html_path.write_text(html, encoding="utf-8")
        subprocess.run(
            [
                CHROME_BIN,
                "--headless=new",
                "--disable-gpu",
                "--no-sandbox",
                "--hide-scrollbars",
                "--force-device-scale-factor=2",
                f"--window-size={width},{height}",
                f"--screenshot={raw_png}",
                f"file://{html_path}",
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        img = Image.open(raw_png).convert("RGB")
        # Auto-crop around #diagram-card's #cbd5e1 border against the #ffffff canvas
        bg = Image.new("RGB", img.size, img.getpixel((0, 0)))
        bbox = ImageChops.difference(img, bg).getbbox()
        if not bbox:
            raise RuntimeError(f"Empty render for {name}")
        pad = 20  # 10px clean #ffffff margin at 2x scale
        cropped = img.crop((
            max(0, bbox[0] - pad),
            max(0, bbox[1] - pad),
            min(img.width, bbox[2] + pad),
            min(img.height, bbox[3] + pad),
        ))
        out_path = OUT_DIR / f"{name}.png"
        cropped.save(out_path, format="PNG", optimize=True)
        sha256 = hashlib.sha256(out_path.read_bytes()).hexdigest()
        return out_path, sha256
```

> **Tip**: Make sure `--window-size` height (`height=1800` or `2400`) exceeds the tall card height so Chrome never clips the bottom of `#diagram-card` before Pillow crops it.

---

## 4. Mandatory `read_file` Visual Inspection Gate

Never trust code-only HTML/SVG coordinates blindly. Before running `sf-cli write-images`, call `read_file` on **every** generated PNG (`zh_*.png` and `en_*.png`) and verify all 6 checks:

1. **Zero clipping**: All four outer borders of `#diagram-card` are completely visible (not cut off at the bottom or right edge by `--window-size`).
2. **Zero text collision or awkward wrapping**: No heading, badge, or connector label overlaps an arrow, border, or sibling box; no single CJK character or punctuation mark dangles awkwardly when a minor width/padding tweak fixes it; English labels fit cleanly inside their pills/boxes.
3. **Zero raw Markdown syntax**: Confirm zero literal `` `backticks` ``, `**asterisks**`, or unescaped HTML entities (`&amp;gt;`) appear in rendered text.
4. **Legible contrast & hierarchy**: Title, stage headers, code identifiers, and footnotes are crisp at normal article column width (~760px CSS width).
5. **100% factual & polarity accuracy**: Every arrow direction, retry number, boolean branch (`true`/`false`), and architectural boundary matches the audited source code and prose.
6. **1:1 `zh` / `en` visual parity**: Each `zh` diagram and its `en` counterpart convey the exact same topology, stages, and technical facts.

Only after every PNG passes `read_file` inspection, ingest the directory:

```bash
/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow/release/sf-cli write-images \
  --db-path /mnt/wsl/data4tb/static-flow-data/lancedb \
  --dir /tmp/<work_dir>/img_upload \
  --generate-thumbnail \
  --no-auto-optimize
```

Embed the resulting SHA-256 IDs in Markdown as `![Descriptive caption](/api/images/<sha256>)` (`zh` SHA-256 in `content`, `en` SHA-256 in `content_en`).
