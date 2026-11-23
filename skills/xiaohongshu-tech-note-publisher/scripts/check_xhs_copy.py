#!/usr/bin/env python3
"""Validate Xiaohongshu title weight, body length, and zero-diversion compliance."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

BANNED_PATTERNS = [
    r"https?://",
    r"\backingliu\.top\b",
    r"\bstaticflow\.cc\b",
    r"\b[a-zA-Z0-9-]+\.(?:top|com|cc|gg|io|site|org|net|dev)\b",
    r"个人博客",
    r"已导入博客",
    r"全文见博客",
    r"链接放评论区",
    r"评论区置顶",
    r"见评论区",
    r"点击主页",
    r"点主页",
    r"私信获取",
    r"私信领",
    r"完整精翻入库",
]

AWKWARD_SHORTHAND = [
    "只砸",
    "冒烟自修",
    "多大盘子",
    "大扫除修剪",
]


def xhs_weighted_title_len(title: str) -> float:
    return sum(0.5 if ord(ch) < 128 else 1.0 for ch in title)


def check_payload(titles: list[str], body: str, tags: list[str]) -> int:
    errors: list[str] = []
    warnings: list[str] = []

    for t in titles:
        w = xhs_weighted_title_len(t)
        print(f"[TITLE] {w:5.1f} / 20.0  |  {t}")
        if w > 20.0:
            errors.append(f"Title exceeds 20 weighted chars ({w}): {t}")

    tag_line = " ".join(f"#{tag.lstrip('#')}" for tag in tags)
    visible_full = f"{body.rstrip()}\n\n{tag_line}".strip()
    visible_chars = len(visible_full)
    utf16_units = len(visible_full.encode("utf-16-le")) // 2
    print(f"[BODY]  visible_chars={visible_chars} / 1000  (utf16_units={utf16_units})")
    if visible_chars > 970:
        errors.append(
            f"Visible body + tags length ({visible_chars}) exceeds safe 970/1000 budget."
        )

    combined = "\n".join([*titles, visible_full])
    for pat in BANNED_PATTERNS:
        matches = re.findall(pat, combined, flags=re.IGNORECASE)
        if matches:
            errors.append(f"Banned diversion/domain pattern '{pat}' matched: {matches}")

    for phrase in AWKWARD_SHORTHAND:
        if phrase in combined:
            warnings.append(f"Awkward shorthand detected: '{phrase}'")

    for w in warnings:
        print(f"[WARN]  {w}")
    for e in errors:
        print(f"[ERROR] {e}")

    if errors:
        return 1
    print("[OK]    All Xiaohongshu length & anti-throttling checks passed.")
    return 0


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: check_xhs_copy.py <payload.json>", file=sys.stderr)
        print(
            'JSON format: {"titles": ["..."], "body": "...", "tags": ["Claude", ...]}',
            file=sys.stderr,
        )
        return 2
    data = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    return check_payload(
        titles=data.get("titles", []),
        body=data.get("body", ""),
        tags=data.get("tags", []),
    )


if __name__ == "__main__":
    raise SystemExit(main())
