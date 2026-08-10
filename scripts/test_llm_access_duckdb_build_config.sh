#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLM_ACCESS_DIR="$ROOT_DIR/deps/llm-access"

grep -F 'DUCKDB_DOWNLOAD_LIB = { value = "1", force = false }' \
  "$LLM_ACCESS_DIR/.cargo/config.toml"

grep -F 'default = ["duckdb-prebuilt"]' \
  "$LLM_ACCESS_DIR/crates/llm-access-store/Cargo.toml"
grep -F 'duckdb-prebuilt = ["duckdb-runtime"]' \
  "$LLM_ACCESS_DIR/crates/llm-access-store/Cargo.toml"
grep -F 'default = ["duckdb-prebuilt"]' \
  "$LLM_ACCESS_DIR/crates/llm-access/Cargo.toml"
grep -F 'duckdb-prebuilt = ["duckdb-runtime"]' \
  "$LLM_ACCESS_DIR/crates/llm-access/Cargo.toml"
