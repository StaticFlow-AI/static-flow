#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ANTIGRAVITY_DIR="${ANTIGRAVITY_DIR:-$ROOT_DIR/deps/AntigravityManager}"
OAUTH_SOURCE_COMMIT="${ANTIGRAVITY_OAUTH_SOURCE_COMMIT:-51edd21}"
OAUTH_SOURCE_PATH="src/modules/cloud-account/services/OAuthClientRegistryService.ts"
CHECK_ONLY=0

fail() {
  echo "[antigravity-manager][ERROR] $*" >&2
  exit 1
}

if [[ "${1:-}" == "--check" ]]; then
  CHECK_ONLY=1
  shift
fi
[[ $# -eq 0 ]] || fail "unknown argument: $1"

[[ -d "$ANTIGRAVITY_DIR" ]] || fail "missing checkout: $ANTIGRAVITY_DIR"
[[ -d "$ANTIGRAVITY_DIR/node_modules" ]] || fail "missing node_modules: $ANTIGRAVITY_DIR/node_modules"
command -v git >/dev/null 2>&1 || fail "missing command: git"
command -v sed >/dev/null 2>&1 || fail "missing command: sed"

node_toolchain_supported() {
  local npm_major

  command -v node >/dev/null 2>&1 || return 1
  command -v npm >/dev/null 2>&1 || return 1
  node -e '
    const [major, minor] = process.versions.node.split(".").map(Number);
    process.exit(major > 22 || (major === 22 && minor >= 14) ? 0 : 1);
  ' || return 1
  npm_major="$(npm --version | cut -d. -f1)"
  [[ "$npm_major" =~ ^[0-9]+$ && "$npm_major" -ge 10 ]]
}

if ! node_toolchain_supported; then
  export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
  if [[ -s "$NVM_DIR/nvm.sh" ]]; then
    # shellcheck source=/dev/null
    source "$NVM_DIR/nvm.sh"
    nvm use --silent default >/dev/null
  fi
fi
node_toolchain_supported || fail "Antigravity Manager requires Node >=22.14 and npm >=10"

oauth_source="$(git -C "$ANTIGRAVITY_DIR" show "$OAUTH_SOURCE_COMMIT:$OAUTH_SOURCE_PATH")" \
  || fail "cannot read OAuth defaults from commit $OAUTH_SOURCE_COMMIT"
client_id="$(printf '%s\n' "$oauth_source" | sed -nE "s/^const CLIENT_ID = '([^']+)';$/\1/p")"
client_secret="$(printf '%s\n' "$oauth_source" | sed -nE "s/^const CLIENT_SECRET = '([^']+)';$/\1/p")"
unset oauth_source

[[ -n "$client_id" ]] || fail "OAuth client ID was not found"
[[ -n "$client_secret" ]] || fail "OAuth client secret was not found"
export ANTIGRAVITY_DEFAULT_OAUTH_CLIENT_ID="$client_id"
export ANTIGRAVITY_DEFAULT_OAUTH_CLIENT_SECRET="$client_secret"
unset client_id client_secret

export DISPLAY="${DISPLAY:-:0}"
export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

if [[ ! -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" && ! -S /tmp/.X11-unix/X0 ]]; then
  fail "WSLg display sockets are not ready"
fi

if [[ "$CHECK_ONLY" == "1" ]]; then
  exit 0
fi

cd "$ANTIGRAVITY_DIR"
exec npm start
