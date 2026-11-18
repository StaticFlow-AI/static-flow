#!/usr/bin/env bash
set -euo pipefail

# Build frontend for self-hosted mode (backend serves SPA + API on same origin).
#
# Key difference from GitHub Pages build:
#   STATICFLOW_API_BASE=/api  (relative, same-origin)
#
# Output: crates/frontend/dist/  (ready to be served by backend via FRONTEND_DIST_DIR)
#
# Usage:
#   ./scripts/build_frontend_selfhosted.sh
#   ./scripts/build_frontend_selfhosted.sh --out /path/to/output
#   ./scripts/build_frontend_selfhosted.sh --skip-npm  # skip npm install

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRONTEND_DIR="$ROOT_DIR/crates/frontend"
STANDALONE_SRC_DIR="$FRONTEND_DIR/standalone"
STANDALONE_DIST_DIR="$FRONTEND_DIR/dist/standalone"
OUTPUT_DIR=""
SKIP_NPM="false"
NPM_CACHE_DIR="${NPM_CACHE_DIR:-$ROOT_DIR/tmp/npm-cache}"
FRONTEND_DEFAULT_FEATURES="${FRONTEND_DEFAULT_FEATURES:-1}"
FRONTEND_FEATURES="${FRONTEND_FEATURES:-}"

log() { echo "[build-selfhosted] $*"; }
fail() { echo "[build-selfhosted][ERROR] $*" >&2; exit 1; }

# Newer trunk versions parse NO_COLOR as a boolean option and reject legacy
# numeric values like "1". Normalize common truthy values before invoking trunk.
if [[ "${NO_COLOR:-}" == "1" ]]; then
  export NO_COLOR="true"
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      [[ $# -ge 2 ]] || fail "--out requires a path"
      OUTPUT_DIR="$2"; shift 2 ;;
    --skip-npm)
      SKIP_NPM="true"; shift ;;
    -h|--help)
      echo "Usage: $0 [--out <dir>] [--skip-npm]"
      echo "  --out <dir>   Copy dist to a custom directory after build"
      echo "  --skip-npm    Skip npm install (use if deps are already installed)"
      echo ""
      echo "Feature env:"
      echo "  FRONTEND_DEFAULT_FEATURES=0   Build with --no-default-features"
      echo "  FRONTEND_FEATURES=<list>      Extra trunk --features list"
      exit 0 ;;
    *) fail "Unknown option: $1" ;;
  esac
done

command -v trunk >/dev/null 2>&1 || fail "trunk not found. Install with: cargo install trunk"
[[ -d "$FRONTEND_DIR" ]] || fail "frontend directory not found: $FRONTEND_DIR"

# Ensure npm deps
if [[ "$SKIP_NPM" != "true" ]]; then
  if [[ ! -d "$FRONTEND_DIR/node_modules/@tailwindcss/cli" || ! -d "$FRONTEND_DIR/node_modules/three" || ! -d "$FRONTEND_DIR/node_modules/esbuild" ]]; then
    log "Installing frontend npm dependencies..."
    mkdir -p "$NPM_CACHE_DIR"
    (cd "$FRONTEND_DIR" && NPM_CONFIG_CACHE="$NPM_CACHE_DIR" npm install)
  fi
fi

log "Building frontend for self-hosted mode (API_BASE=/api)..."
log "Building standalone GPT2API frontend..."
"$ROOT_DIR/scripts/build_gpt2api_frontend.sh"

cd "$FRONTEND_DIR"
TRUNK_ARGS=(build --release)
if [[ "$FRONTEND_DEFAULT_FEATURES" == "0" ]]; then
  TRUNK_ARGS+=(--no-default-features)
fi
EFFECTIVE_FEATURES="$FRONTEND_FEATURES"
if [[ "${STATICFLOW_ENABLE_LEGACY_LLM_ADMIN:-}" == "1" ]]; then
  if [[ -n "$EFFECTIVE_FEATURES" ]]; then
    EFFECTIVE_FEATURES="$EFFECTIVE_FEATURES,legacy-llm-admin"
  else
    EFFECTIVE_FEATURES="legacy-llm-admin"
  fi
fi
if [[ -n "$EFFECTIVE_FEATURES" ]]; then
  TRUNK_ARGS+=(--features "$EFFECTIVE_FEATURES")
fi
NPM_CONFIG_CACHE="$NPM_CACHE_DIR" \
STATICFLOW_API_BASE="/api" \
trunk "${TRUNK_ARGS[@]}"

# Optimize WASM binary size via wasm-opt (-Oz --all-features --strip-debug --strip-dwarf --strip-producers)
WASM_OPT_BIN=""
if command -v wasm-opt >/dev/null 2>&1; then
  WASM_OPT_BIN="$(command -v wasm-opt)"
else
  for candidate in "$HOME"/.cache/trunk/wasm-opt-*/bin/wasm-opt; do
    if [[ -x "$candidate" ]]; then
      WASM_OPT_BIN="$candidate"
    fi
  done
fi
if [[ -n "$WASM_OPT_BIN" ]]; then
  for wasm_file in "$FRONTEND_DIR"/dist/*_bg.wasm; do
    [[ -f "$wasm_file" ]] || continue
    before_bytes="$(wc -c < "$wasm_file")"
    "$WASM_OPT_BIN" -Oz --all-features --strip-debug --strip-dwarf --strip-producers "$wasm_file" -o "${wasm_file}.opt"
    mv "${wasm_file}.opt" "$wasm_file"
    after_bytes="$(wc -c < "$wasm_file")"
    if command -v node >/dev/null 2>&1 && [[ -f "$FRONTEND_DIR/dist/index.html" ]]; then
      WASM_FILE="$wasm_file" INDEX_HTML="$FRONTEND_DIR/dist/index.html" node - <<'NODE'
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

const wasmFile = process.env.WASM_FILE;
const indexPath = process.env.INDEX_HTML;
const wasmName = path.basename(wasmFile).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const digest = 'sha384-' + crypto.createHash('sha384').update(fs.readFileSync(wasmFile)).digest('base64');
const html = fs.readFileSync(indexPath, 'utf8');
const tagRe = new RegExp(`(<link\\b[^>]*\\bhref="/?${wasmName}"[^>]*\\bintegrity=")sha384-[^"]+(")`, 'g');
const updated = html.replace(tagRe, `$1${digest}$2`);
if (updated !== html) {
  fs.writeFileSync(indexPath, updated);
}
NODE
    fi
    log "wasm-opt -Oz: $(basename "$wasm_file") ${before_bytes} -> ${after_bytes} bytes (SRI synced)"
  done
else
  log "WARN: wasm-opt not found; skipping post-link WASM size optimization"
fi

# Copy standalone pages that are linked from the SPA homepage.
rm -rf "$STANDALONE_DIST_DIR"
mkdir -p "$STANDALONE_DIST_DIR"
if [[ -d "$STANDALONE_SRC_DIR" ]]; then
  cp -r "$STANDALONE_SRC_DIR/." "$STANDALONE_DIST_DIR/"
  log "Copied standalone pages: $STANDALONE_SRC_DIR -> $STANDALONE_DIST_DIR"
else
  log "No standalone source directory found at $STANDALONE_SRC_DIR; leaving dist/standalone empty"
fi

# Precompress static assets (.br quality 11 + .gz level 9) for zero-CPU ServeDir delivery.
# Top-level dist/index.html is excluded because backend seo.rs injects route metadata dynamically.
if command -v node >/dev/null 2>&1; then
  DIST_DIR="$FRONTEND_DIR/dist" node - <<'NODE'
const fs = require('fs');
const path = require('path');
const zlib = require('zlib');

const distDir = process.env.DIST_DIR;
const exts = new Set(['.wasm', '.js', '.mjs', '.css', '.svg', '.json', '.webmanifest', '.html']);
let count = 0;

function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(full);
      continue;
    }
    if (!entry.isFile()) continue;
    if (full === path.join(distDir, 'index.html')) continue;
    const ext = path.extname(entry.name).toLowerCase();
    if (!exts.has(ext)) continue;
    const raw = fs.readFileSync(full);
    if (raw.length < 256) continue;
    const br = zlib.brotliCompressSync(raw, {
      params: {
        [zlib.constants.BROTLI_PARAM_QUALITY]: 11,
        [zlib.constants.BROTLI_PARAM_SIZE_HINT]: raw.length,
      },
    });
    if (br.length < raw.length) {
      fs.writeFileSync(full + '.br', br);
    }
    const gz = zlib.gzipSync(raw, { level: 9 });
    if (gz.length < raw.length) {
      fs.writeFileSync(full + '.gz', gz);
    }
    count += 1;
  }
}

walk(distDir);
console.log(`[build-selfhosted] Precompressed ${count} static assets (.br + .gz)`);
NODE
fi

log "Build complete: $FRONTEND_DIR/dist/"

# Optional: copy to custom output dir
if [[ -n "$OUTPUT_DIR" ]]; then
  mkdir -p "$OUTPUT_DIR"
  cp -r "$FRONTEND_DIR/dist/"* "$OUTPUT_DIR/"
  log "Copied to: $OUTPUT_DIR/"
fi

log "Done. Use FRONTEND_DIST_DIR=$FRONTEND_DIR/dist when starting backend."
