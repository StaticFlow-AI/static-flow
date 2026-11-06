#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLM_ACCESS_DIR="${LLM_ACCESS_DIR:-$ROOT_DIR/deps/llm-access}"
CONFIG_FILE="${LLM_ACCESS_CLOUD_RELEASE_CONFIG:-$ROOT_DIR/.local/llm-access-cloud-release-aws.env}"
BUILD_JOBS="${BUILD_JOBS:-4}"

fail() { printf '[llm-access-release-opencode][ERROR] %s\n' "$*" >&2; exit 1; }
q() { printf '%q' "$1"; }
check_build_budget() {
  df -h /mnt/wsl/data4tb >/dev/null
  if pgrep -af 'cargo|rustc|trunk|ld|lld|mold' | python3 -c 'import os,sys; rows=(line.strip().split(None,1) for line in sys.stdin); sys.exit(0 if any(len(row)>1 and os.path.basename(row[1].split()[0]) in ("cargo","rustc","trunk","ld","lld","mold") for row in rows) else 1)'; then
    fail 'another Rust build or linker is running'
  fi
}

for command in cargo git pgrep python3 scp sha256sum ssh; do command -v "$command" >/dev/null || fail "missing command: $command"; done
[[ -r "$CONFIG_FILE" ]] || fail "missing release config: $CONFIG_FILE"
# shellcheck source=/dev/null
source "$CONFIG_FILE"
for variable in CARGO_TARGET_DIR GCP_SSH_KEY REMOTE_RELEASE_DIR; do
  [[ -n "${!variable:-}" ]] || fail "missing $variable in $CONFIG_FILE"
done
if [[ -z "${GCP_DEST:-}" ]]; then
  [[ -n "${GCP_USER:-}" && -n "${GCP_HOST:-}" ]] || fail 'missing SSH destination'
  GCP_DEST="$GCP_USER@$GCP_HOST"
fi
[[ -r "$GCP_SSH_KEY" ]] || fail 'SSH key is not readable'
[[ -z "$(git -C "$LLM_ACCESS_DIR" status --porcelain)" ]] || fail 'llm-access checkout is dirty'
[[ -z "$(git -C "$ROOT_DIR" status --porcelain --ignore-submodules=dirty)" ]] || fail 'StaticFlow checkout is dirty'
CHILD_REV="$(git -C "$LLM_ACCESS_DIR" rev-parse HEAD)"
[[ "$(git -C "$ROOT_DIR" rev-parse HEAD:deps/llm-access)" == "$CHILD_REV" ]] || fail 'parent gitlink does not match llm-access'
git -C "$LLM_ACCESS_DIR" merge-base --is-ancestor "$CHILD_REV" origin/main || fail 'child revision is not on fetched origin/main'
export CARGO_TARGET_DIR
export LD_LIBRARY_PATH="$CARGO_TARGET_DIR/debug/deps${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LLM_ACCESS_BUILD_REVISION="$CHILD_REV"
cd "$LLM_ACCESS_DIR"
check_build_budget
cargo test -p llm-access-opencode -p llm-access-opencode-gateway --locked --jobs "$BUILD_JOBS"
check_build_budget
cargo clippy -p llm-access-opencode -p llm-access-opencode-gateway --all-targets --locked --jobs "$BUILD_JOBS" -- -D warnings
check_build_budget
cargo build -p llm-access-opencode-gateway --release --locked --jobs "$BUILD_JOBS"

BIN="$CARGO_TARGET_DIR/release/llm-access-opencode-gateway"
[[ -x "$BIN" ]] || fail 'release binary was not built'
BIN_SHA="$(sha256sum "$BIN" | awk '{print $1}')"
RELEASE_ID="${RELEASE_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(git rev-parse --short=12 HEAD)}"
[[ "$RELEASE_ID" =~ ^[a-zA-Z0-9._-]+$ ]] || fail 'invalid release id'
STAGE="$ROOT_DIR/tmp/llm-access-cloud-release/$RELEASE_ID-opencode"
mkdir -p "$STAGE"
cp "$BIN" "$STAGE/llm-access-opencode-gateway.$RELEASE_ID"
cp "$ROOT_DIR/deployment-examples/systemd/llm-access-opencode.service.template" "$STAGE/llm-access-opencode.service"
printf '%s  %s\n' "$BIN_SHA" "llm-access-opencode-gateway.$RELEASE_ID" > "$STAGE/SHA256SUMS-opencode.$RELEASE_ID"
SSH_OPTS=(-i "$GCP_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes -o ControlMaster=no -o ControlPath=none -o ControlPersist=no)
REMOTE_DIR_Q="$(q "$REMOTE_RELEASE_DIR")"
scp "${SSH_OPTS[@]}" "$STAGE/llm-access-opencode-gateway.$RELEASE_ID" "$STAGE/llm-access-opencode.service" "$STAGE/SHA256SUMS-opencode.$RELEASE_ID" "$GCP_DEST:$REMOTE_RELEASE_DIR/"
ssh "${SSH_OPTS[@]}" "$GCP_DEST" "set -e
  cd $REMOTE_DIR_Q
  sha256sum -c SHA256SUMS-opencode.$RELEASE_ID
  systemctl is-active juicefs-llm-access.service >/dev/null
  findmnt -T /mnt/llm-access >/dev/null
  for service in llm-access llm-access-usage-worker llm-access-cursor llm-access-antigravity llm-access-oauth llm-access-codex-image; do
    systemctl show -p MainPID -p NRestarts \"\$service.service\" | sort > \"/tmp/\$service.$RELEASE_ID.before\"
  done
  timestamp=\$(date -u +%Y%m%dT%H%M%SZ)
  sudo cp -a /usr/local/bin/llm-access-opencode-gateway /usr/local/bin/llm-access-opencode-gateway.backup.\$timestamp 2>/dev/null || true
  sudo install -o root -g root -m 0755 llm-access-opencode-gateway.$RELEASE_ID /usr/local/bin/llm-access-opencode-gateway
  sudo install -o root -g root -m 0644 llm-access-opencode.service /etc/systemd/system/llm-access-opencode.service
  sudo systemctl daemon-reload
  sudo systemctl enable llm-access-opencode.service >/dev/null
  sudo systemctl restart llm-access-opencode.service
  for attempt in \$(seq 1 30); do curl -fsS http://127.0.0.1:19085/healthz >/dev/null && break; sleep 1; done
  curl -fsS http://127.0.0.1:19085/healthz >/dev/null
  sudo systemctl is-active --quiet llm-access-opencode.service
  revision=\$(curl -fsS http://127.0.0.1:19085/version | python3 -c 'import json,sys; print(json.load(sys.stdin).get(\"revision\", \"\"))')
  test \"\$revision\" = '$CHILD_REV'
  for service in llm-access llm-access-usage-worker llm-access-cursor llm-access-antigravity llm-access-oauth llm-access-codex-image; do
    systemctl show -p MainPID -p NRestarts \"\$service.service\" | sort | cmp -s - \"/tmp/\$service.$RELEASE_ID.before\"
    rm -f \"/tmp/\$service.$RELEASE_ID.before\"
  done
  echo '$BIN_SHA  /usr/local/bin/llm-access-opencode-gateway' | sha256sum -c
"
printf 'OpenCode release %s deployed (child %s)\n' "$RELEASE_ID" "$CHILD_REV"
