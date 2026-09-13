#!/usr/bin/env python3
"""Verify this article bundle against pinned local Codex Git objects, offline.

This checks citations and literal excerpts, not the truth of prose, live service
behavior, external URL availability, or browser rendering.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path
from urllib.parse import unquote, urlsplit


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True, help="Local Codex Git repository")
    args = parser.parse_args()
    bundle = Path(__file__).resolve().parent
    manifest = json.loads((bundle / "source-map.json").read_text(encoding="utf-8"))
    revision = manifest["revision"]
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("Manifest revision must be a full Git SHA")

    def git(*arguments: str) -> str:
        result = subprocess.run(
            ["git", "-C", str(args.repo.expanduser().resolve()), *arguments],
            check=True,
            capture_output=True,
        )
        return result.stdout.decode("utf-8")

    resolved = git("rev-parse", "--verify", f"{revision}^{{commit}}").strip()
    if resolved != revision:
        raise ValueError("Pinned commit did not resolve exactly")

    files: dict[str, list[str]] = {}
    citations: set[tuple[str, int, int]] = set()

    def excerpt(path: str, start: int, end: int) -> str:
        if path not in files:
            files[path] = git("show", f"{revision}:{path}").splitlines(keepends=True)
        lines = files[path]
        if not 1 <= start <= end <= len(lines):
            raise ValueError(f"Invalid source range: {path}:{start}-{end} ({len(lines)} lines)")
        return "".join(lines[start - 1 : end])

    for key, source in manifest["sources"].items():
        identity = (source["path"], source["start"], source["end"])
        actual_hash = hashlib.sha256(excerpt(*identity).encode("utf-8")).hexdigest()
        if actual_hash != source["sha256"]:
            raise ValueError(f"Source digest mismatch: {key}")
        citations.add(identity)

    documents = sorted(bundle.glob("*.md"))
    if {p.name for p in documents} != {"index.md", "source-analysis.md"}:
        raise ValueError("Expected the two Markdown documents in this bundle")
    contents = {p: p.read_text(encoding="utf-8") for p in documents}
    anchors: dict[Path, set[str]] = {}
    for path, text in contents.items():
        ids = re.findall(r'<a id="([^"]+)"></a>', text)
        if len(ids) != len(set(ids)):
            raise ValueError(f"Duplicate anchors in {path.name}")
        anchors[path] = set(ids)

    counts = {"excerpts": 0, "source_links": 0, "local_links": 0, "external_links": 0}
    source_url = re.compile(
        r"https://github\.com/openai/codex/blob/([^/]+)/([^#]+)#L(\d+)-L(\d+)"
    )
    source_block = re.compile(
        r"```rust\n// (codex-rs/[^:\n]+):(\d+)-(\d+)\n(.*?)```", re.DOTALL
    )
    for path, text in contents.items():
        if re.search(r"\{\{(?:link|code):", text):
            raise ValueError(f"Unresolved source placeholder in {path.name}")
        if len(re.findall(r"^```", text, re.MULTILINE)) % 2:
            raise ValueError(f"Unbalanced code fences in {path.name}")

        blocks = list(source_block.finditer(text))
        if len(blocks) != len(re.findall(r"^```rust$", text, re.MULTILINE)):
            raise ValueError(f"Unmarked Rust excerpt in {path.name}")
        for match in blocks:
            source_path, first, last, body = match.groups()
            identity = (source_path, int(first), int(last))
            if identity not in citations:
                raise ValueError(f"Excerpt missing from manifest: {identity}")
            if body != excerpt(*identity):
                raise ValueError(f"Literal excerpt mismatch in {path.name}: {identity}")
            counts["excerpts"] += 1

        # Links in literal examples are not Markdown links in the rendered document.
        prose = re.sub(r"^```[^\n]*\n.*?^```\s*$", "", text, flags=re.MULTILINE | re.DOTALL)
        for target in re.findall(r"\[[^\]\n]+\]\(([^)\s]+)\)", prose):
            if target.startswith("https://github.com/openai/codex/blob/"):
                match = source_url.fullmatch(target)
                if match is None:
                    raise ValueError(f"Source link lacks an explicit line range: {target}")
                sha, source_path, first, last = match.groups()
                identity = (source_path, int(first), int(last))
                if sha != revision or identity not in citations:
                    raise ValueError(f"Unpinned or unregistered source link: {target}")
                excerpt(*identity)
                counts["source_links"] += 1
                continue
            parsed = urlsplit(target)
            if parsed.scheme:
                if parsed.scheme != "https":
                    raise ValueError(f"Unexpected external URL scheme: {target}")
                counts["external_links"] += 1
                continue
            destination = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
            if not destination.is_relative_to(bundle) or not destination.is_file():
                raise ValueError(f"Missing or out-of-bundle local link in {path.name}: {target}")
            if parsed.fragment and parsed.fragment not in anchors.get(destination, set()):
                raise ValueError(f"Missing local anchor in {path.name}: {target}")
            counts["local_links"] += 1

    print(f"PASS: pinned commit {revision}")
    print(f"PASS: {len(manifest['sources'])} source records across {len(files)} Git files")
    print(
        f"PASS: {counts['excerpts']} literal Rust excerpts, "
        f"{counts['source_links']} pinned source links, "
        f"{counts['local_links']} local links and anchors"
    )
    print(f"Not network-checked: {counts['external_links']} external documentation/tree/commit links")
    print("Not tested: Codex runtime, live backends, performance, or diagram rendering")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"FAIL: {error}") from error
