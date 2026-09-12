#!/usr/bin/env bash
set -euo pipefail

# Release only the standalone Antigravity data plane and the private OAuth
# manager. The main API, Cursor data plane, usage worker, and image gateway are
# deliberately outside this script's activation set.
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLM_ACCESS_DIR="${LLM_ACCESS_DIR:-$ROOT_DIR/deps/llm-access}"
CONFIG_FILE="${LLM_ACCESS_CLOUD_RELEASE_CONFIG:-$ROOT_DIR/.local/llm-access-cloud-release-aws.env}"
RENDER_DIR="$(mktemp -d "$ROOT_DIR/tmp/llm-access-antigravity-release.XXXXXX")"
BUILD_JOBS="${BUILD_JOBS:-4}"

cleanup() { rm -rf "$RENDER_DIR"; }
trap cleanup EXIT
fail() { printf '[llm-access-release-antigravity][ERROR] %s\n' "$*" >&2; exit 1; }
q() { printf '%q' "$1"; }
require() { command -v "$1" >/dev/null 2>&1 || fail "missing command: $1"; }

for command in cargo git scp sha256sum ssh; do require "$command"; done
[[ -r "$CONFIG_FILE" ]] || fail "missing release config: $CONFIG_FILE"
# shellcheck source=/dev/null
source "$CONFIG_FILE"
for variable in CARGO_TARGET_DIR GCP_SSH_KEY REMOTE_RELEASE_DIR; do
  [[ -n "${!variable:-}" ]] || fail "missing $variable in $CONFIG_FILE"
done
if [[ -z "${GCP_DEST:-}" ]]; then
  [[ -n "${GCP_USER:-}" && -n "${GCP_HOST:-}" ]] || fail "missing GCP_DEST or GCP_USER/GCP_HOST"
  GCP_DEST="$GCP_USER@$GCP_HOST"
fi
[[ -r "$GCP_SSH_KEY" ]] || fail "SSH key is not readable: $GCP_SSH_KEY"
[[ -r "$LLM_ACCESS_DIR/Cargo.toml" ]] || fail "llm-access checkout is missing: $LLM_ACCESS_DIR"
[[ -z "$(git -C "$LLM_ACCESS_DIR" status --porcelain)" ]] || fail "llm-access checkout is dirty"
[[ -z "$(git -C "$ROOT_DIR" status --porcelain --ignore-submodules=dirty)" ]] || fail "StaticFlow checkout is dirty"

export CARGO_TARGET_DIR
cd "$LLM_ACCESS_DIR"
cargo test -p llm-access-cursor-protocol -p llm-access-cursor -p llm-access-antigravity -p llm-access-oauth --locked --jobs "$BUILD_JOBS"
cargo clippy -p llm-access-cursor-protocol -p llm-access-cursor -p llm-access-antigravity -p llm-access-oauth --all-targets --locked --jobs "$BUILD_JOBS" -- -D warnings
cargo build -p llm-access-antigravity -p llm-access-oauth --release --locked --jobs "$BUILD_JOBS"

ANTI_BIN="$CARGO_TARGET_DIR/release/llm-access-antigravity"
OAUTH_BIN="$CARGO_TARGET_DIR/release/llm-access-oauth"
[[ -x "$ANTI_BIN" && -x "$OAUTH_BIN" ]] || fail "release binaries were not built"
ANTI_SHA="$(sha256sum "$ANTI_BIN" | awk '{print $1}')"
OAUTH_SHA="$(sha256sum "$OAUTH_BIN" | awk '{print $1}')"
RELEASE_ID="${RELEASE_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short=12 HEAD)}"
STAGE="$ROOT_DIR/tmp/llm-access-cloud-release/$RELEASE_ID-antigravity"
mkdir -p "$STAGE"
cp "$ANTI_BIN" "$STAGE/llm-access-antigravity.$RELEASE_ID"
cp "$OAUTH_BIN" "$STAGE/llm-access-oauth.$RELEASE_ID"
cp "$ROOT_DIR/deployment-examples/systemd/llm-access-antigravity.service.template" "$STAGE/llm-access-antigravity.service"
cp "$ROOT_DIR/deployment-examples/systemd/llm-access-oauth.service.template" "$STAGE/llm-access-oauth.service"
printf '%s  %s\n%s  %s\n' "$ANTI_SHA" "llm-access-antigravity.$RELEASE_ID" "$OAUTH_SHA" "llm-access-oauth.$RELEASE_ID" > "$STAGE/SHA256SUMS"

SSH_OPTS=(-i "$GCP_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes)
REMOTE_DIR_Q="$(q "$REMOTE_RELEASE_DIR")"
scp "${SSH_OPTS[@]}" "$STAGE/llm-access-antigravity.$RELEASE_ID" "$STAGE/llm-access-oauth.$RELEASE_ID" "$STAGE/llm-access-antigravity.service" "$STAGE/llm-access-oauth.service" "$STAGE/SHA256SUMS" "$GCP_DEST:$REMOTE_RELEASE_DIR/"
ssh "${SSH_OPTS[@]}" "$GCP_DEST" "set -e
  cd $REMOTE_DIR_Q
  sha256sum -c SHA256SUMS
  timestamp=\$(date -u +%Y%m%dT%H%M%SZ)
  sudo cp -a /usr/local/bin/llm-access-antigravity /usr/local/bin/llm-access-antigravity.backup.\$timestamp 2>/dev/null || true
  sudo cp -a /usr/local/bin/llm-access-oauth /usr/local/bin/llm-access-oauth.backup.\$timestamp 2>/dev/null || true
  sudo install -o root -g root -m 0755 llm-access-antigravity.$RELEASE_ID /usr/local/bin/llm-access-antigravity
  sudo install -o root -g root -m 0755 llm-access-oauth.$RELEASE_ID /usr/local/bin/llm-access-oauth
  sudo install -o root -g root -m 0644 llm-access-antigravity.service /etc/systemd/system/llm-access-antigravity.service
  sudo install -o root -g root -m 0644 llm-access-oauth.service /etc/systemd/system/llm-access-oauth.service
  before_api=\$(sudo systemctl show -p NRestarts --value llm-access.service)
  before_cursor=\$(sudo systemctl show -p NRestarts --value llm-access-cursor.service)
  before_worker=\$(sudo systemctl show -p NRestarts --value llm-access-usage-worker.service)
  sudo systemctl daemon-reload
  sudo systemctl enable llm-access-antigravity.service >/dev/null
  sudo systemctl restart llm-access-antigravity.service
  sudo systemctl restart llm-access-oauth.service
  for attempt in \$(seq 1 30); do curl -fsS http://127.0.0.1:19095/healthz >/dev/null && curl -fsS http://127.0.0.1:19194/ >/dev/null && break; sleep 1; done
  curl -fsS http://127.0.0.1:19095/healthz >/dev/null
  curl -fsS http://127.0.0.1:19194/ >/dev/null
  test \"\$(sudo systemctl show -p NRestarts --value llm-access.service)\" = \"\$before_api\"
  test \"\$(sudo systemctl show -p NRestarts --value llm-access-cursor.service)\" = \"\$before_cursor\"
  test \"\$(sudo systemctl show -p NRestarts --value llm-access-usage-worker.service)\" = \"\$before_worker\"
  sudo systemctl is-active --quiet llm-access-antigravity.service
  sudo systemctl is-active --quiet llm-access-oauth.service
  sha256sum /usr/local/bin/llm-access-antigravity /usr/local/bin/llm-access-oauth
"
printf 'Antigravity release %s deployed (child %s)\n' "$RELEASE_ID" "$(git rev-parse HEAD)"
