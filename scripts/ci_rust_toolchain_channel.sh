#!/usr/bin/env bash
# Print the dated toolchain channel from rust-toolchain.toml.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
channel="$(
  python3 -c 'import pathlib, tomllib, sys; print(tomllib.loads(pathlib.Path(sys.argv[1]).read_text())["toolchain"]["channel"])' \
    "$ROOT_DIR/rust-toolchain.toml"
)"
if [[ -z "$channel" ]]; then
  echo "rust-toolchain.toml is missing toolchain.channel" >&2
  exit 1
fi
printf '%s\n' "$channel"
