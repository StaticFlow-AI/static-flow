#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PUBLIC_BUILD_SUBMODULES=(
  deps/ffmpeg-sidecar
  deps/jieba-rs
  deps/lance
  deps/lancedb
  deps/pingora
  patches/object_store
)

git -C "$ROOT_DIR" submodule sync -- "${PUBLIC_BUILD_SUBMODULES[@]}"
git -C "$ROOT_DIR" submodule update --init --recursive -- "${PUBLIC_BUILD_SUBMODULES[@]}"
