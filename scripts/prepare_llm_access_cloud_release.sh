#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLM_ACCESS_DIR="${LLM_ACCESS_DIR:-$ROOT_DIR/deps/llm-access}"
REMOTE_SCRIPT="$ROOT_DIR/scripts/activate_llm_access_cloud_release.sh"
RENDER_SCRIPT="$ROOT_DIR/scripts/render_llm_access_cloud_bundle.sh"
CONFIG_FILE=""
LOCAL_NEON_ENV_FILE="${LLM_ACCESS_LOCAL_NEON_ENV_FILE:-$ROOT_DIR/.local/llm-access-neon.env}"

log() {
  printf '[llm-access-release] %s\n' "$*"
}

fail() {
  printf '[llm-access-release][ERROR] %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

shell_quote() {
  printf '%q' "$1"
}

expand_path() {
  case "$1" in
    "~")
      printf '%s\n' "$HOME"
      ;;
    "~/"*)
      printf '%s/%s\n' "$HOME" "${1#"~/"}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

default_config_file() {
  local preferred="$ROOT_DIR/.local/llm-access-cloud-release-aws.env"
  local legacy="$ROOT_DIR/.local/llm-access-cloud-release.env"
  if [[ -n "${LLM_ACCESS_CLOUD_RELEASE_CONFIG:-}" ]]; then
    printf '%s\n' "$LLM_ACCESS_CLOUD_RELEASE_CONFIG"
  elif [[ -r "$preferred" ]]; then
    printf '%s\n' "$preferred"
  else
    printf '%s\n' "$legacy"
  fi
}

load_config() {
  CONFIG_FILE="$(default_config_file)"
  [[ -r "$CONFIG_FILE" ]] || fail "missing config file: $CONFIG_FILE; copy conf/llm-access-cloud-release.env.example and edit it"
  # shellcheck source=/dev/null
  source "$CONFIG_FILE"
}

require_var() {
  local name="$1"
  [[ -n "${!name:-}" ]] || fail "missing required config value: $name in $CONFIG_FILE"
}

env_file_has_nonempty_var() {
  local file="$1"
  local name="$2"
  (
    set -a
    # shellcheck source=/dev/null
    source "$file"
    set +a
    local value="${!name:-}"
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    [[ -n "$value" ]]
  )
}

require_env_file_var() {
  local file="$1"
  local name="$2"
  local label="$3"
  env_file_has_nonempty_var "$file" "$name" || fail "$label does not define $name: $file"
}

for cmd in cargo curl df git scp sha256sum ssh; do
  require_cmd "$cmd"
done

load_config

BUILD_JOBS="${BUILD_JOBS:-4}"
ALLOW_DIRTY="${ALLOW_DIRTY:-0}"
PREPARE_TARGET="${LLM_ACCESS_ACTIVATE_TARGET:-both}"
case "$PREPARE_TARGET" in
  api | worker | image | cursor | both) ;;
  *)
    fail "unsupported prepare target: $PREPARE_TARGET (expected api, worker, image, cursor, or both)"
    ;;
esac
require_var CARGO_TARGET_DIR
require_var GCP_SSH_KEY
require_var REMOTE_RELEASE_DIR

if [[ -z "${GCP_DEST:-}" ]]; then
  require_var GCP_USER
  require_var GCP_HOST
  GCP_DEST="$GCP_USER@$GCP_HOST"
fi

CARGO_TARGET_DIR="$(expand_path "$CARGO_TARGET_DIR")"
GCP_SSH_KEY="$(expand_path "$GCP_SSH_KEY")"
LOCAL_NEON_ENV_FILE="$(expand_path "$LOCAL_NEON_ENV_FILE")"

[[ -x "$REMOTE_SCRIPT" ]] || fail "remote activation script is not executable: $REMOTE_SCRIPT"
[[ -x "$RENDER_SCRIPT" ]] || fail "render script is not executable: $RENDER_SCRIPT"
[[ -r "$LLM_ACCESS_DIR/Cargo.toml" ]] || fail "llm-access submodule is not initialized: $LLM_ACCESS_DIR"
[[ -r "$GCP_SSH_KEY" ]] || fail "SSH key is not readable: $GCP_SSH_KEY"
if [[ "$PREPARE_TARGET" != "cursor" ]]; then
  [[ -r "$LOCAL_NEON_ENV_FILE" ]] || fail "local llm-access runtime env is not readable: $LOCAL_NEON_ENV_FILE"
  require_env_file_var "$LOCAL_NEON_ENV_FILE" LLM_ACCESS_CONTROL_DATABASE_URL "local llm-access runtime env"
  if [[ "$PREPARE_TARGET" == "image" || "$PREPARE_TARGET" == "both" ]]; then
    require_env_file_var "$LOCAL_NEON_ENV_FILE" LLM_ACCESS_CODEX_IMAGE_CONTROL_DATABASE_URL "local llm-access runtime env"
  fi
fi
if [[ "$ALLOW_DIRTY" != "1" ]] && [[ -n "$(git -C "$ROOT_DIR" status --porcelain --ignore-submodules=dirty)" ]]; then
  git -C "$ROOT_DIR" status --short --ignore-submodules=dirty >&2
  fail "StaticFlow working tree or llm-access submodule revision is dirty; commit first or run with ALLOW_DIRTY=1"
fi
if [[ "$ALLOW_DIRTY" != "1" ]] && [[ -n "$(git -C "$LLM_ACCESS_DIR" status --porcelain)" ]]; then
  git -C "$LLM_ACCESS_DIR" status --short >&2
  fail "llm-access working tree is dirty; commit first or run with ALLOW_DIRTY=1"
fi

mkdir -p "$CARGO_TARGET_DIR"
df -h "$CARGO_TARGET_DIR" >/dev/null

export CARGO_TARGET_DIR
export LD_LIBRARY_PATH="$CARGO_TARGET_DIR/debug/deps${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LLM_ACCESS_BUILD_REVISION="$(git -C "$LLM_ACCESS_DIR" rev-parse HEAD)"

cd "$LLM_ACCESS_DIR"

if [[ "$PREPARE_TARGET" == "cursor" ]]; then
  log "running llm-access Cursor test suite"
  cargo test -p llm-access-cursor-protocol -p llm-access-cursor --locked --jobs "$BUILD_JOBS"

  log "running llm-access Cursor clippy"
  cargo clippy -p llm-access-cursor-protocol -p llm-access-cursor --all-targets --locked --jobs "$BUILD_JOBS" -- -D warnings

  log "building Cursor release binary"
  cargo build -p llm-access-cursor --bin llm-access-cursor --release --locked --jobs "$BUILD_JOBS"

  CURSOR_BIN="$CARGO_TARGET_DIR/release/llm-access-cursor"
  [[ -x "$CURSOR_BIN" ]] || fail "built Cursor binary not found or not executable: $CURSOR_BIN"

  GIT_COMMIT="$(git rev-parse HEAD)"
  GIT_SHORT="$(git rev-parse --short=12 HEAD)"
  STATICFLOW_GIT_COMMIT="$(git -C "$ROOT_DIR" rev-parse HEAD)"
  RELEASE_ID="${RELEASE_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$GIT_SHORT}"
  OUT_DIR="${LLM_ACCESS_RELEASE_OUT:-$ROOT_DIR/tmp/llm-access-cloud-release/$RELEASE_ID}"
  STAGED_CURSOR_BIN="$OUT_DIR/llm-access-cursor.$RELEASE_ID"
  MANIFEST="$OUT_DIR/cursor-release.$RELEASE_ID.env"
  SHA_FILE="$OUT_DIR/CURSOR_SHA256SUMS.$RELEASE_ID"
  RENDER_DIR="$OUT_DIR/rendered"
  STAGED_CURSOR_SERVICE_UNIT="$OUT_DIR/llm-access-cursor.service.release"

  mkdir -p "$OUT_DIR"
  "$RENDER_SCRIPT" "$RENDER_DIR"
  cp "$CURSOR_BIN" "$STAGED_CURSOR_BIN"
  cp "$RENDER_DIR/llm-access-cursor.service" "$STAGED_CURSOR_SERVICE_UNIT"
  chmod 0755 "$STAGED_CURSOR_BIN"
  CURSOR_BIN_SHA="$(sha256sum "$STAGED_CURSOR_BIN" | awk '{print $1}')"
  printf '%s  %s\n' "$CURSOR_BIN_SHA" "llm-access-cursor.$RELEASE_ID" >"$SHA_FILE"

  cat >"$MANIFEST" <<EOF
release_id=$RELEASE_ID
git_commit=$GIT_COMMIT
git_short=$GIT_SHORT
staticflow_git_commit=$STATICFLOW_GIT_COMMIT
built_at_utc=$(date -u +%FT%TZ)
cursor_sha256=$CURSOR_BIN_SHA
cursor_binary=llm-access-cursor.$RELEASE_ID
EOF

  SSH_OPTS=(-i "$GCP_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes)
  REMOTE_RELEASE_DIR_Q="$(shell_quote "$REMOTE_RELEASE_DIR")"
  REMOTE_SCRIPT_NAME="$(basename "$REMOTE_SCRIPT")"

  log "checking Cursor release target: $GCP_DEST"
  ssh "${SSH_OPTS[@]}" "$GCP_DEST" "
    set -e
    mkdir -p $REMOTE_RELEASE_DIR_Q
    systemctl is-active juicefs-llm-access.service >/dev/null
    findmnt -T /mnt/llm-access >/dev/null
  "

  log "uploading Cursor release $RELEASE_ID to $GCP_DEST:$REMOTE_RELEASE_DIR"
  scp "${SSH_OPTS[@]}" \
    "$STAGED_CURSOR_BIN" \
    "$MANIFEST" \
    "$SHA_FILE" \
    "$STAGED_CURSOR_SERVICE_UNIT" \
    "$REMOTE_SCRIPT" \
    "$GCP_DEST:$REMOTE_RELEASE_DIR/"

  log "updating remote Cursor-only latest pointers"
  ssh "${SSH_OPTS[@]}" "$GCP_DEST" "
    set -e
    cd $REMOTE_RELEASE_DIR_Q
    chmod 0755 $REMOTE_SCRIPT_NAME llm-access-cursor.$RELEASE_ID
    sha256sum -c CURSOR_SHA256SUMS.$RELEASE_ID
    ln -sfn llm-access-cursor.$RELEASE_ID llm-access-cursor.latest
    ln -sfn cursor-release.$RELEASE_ID.env cursor-release.latest.env
  "

  cat <<EOF

Prepared llm-access Cursor-only cloud release:
  release_id: $RELEASE_ID
  git_commit: $GIT_COMMIT
  cursor_sha256: $CURSOR_BIN_SHA
  remote_dir: $REMOTE_RELEASE_DIR

Run this on the cloud target to activate it:
  LLM_ACCESS_ACTIVATE_TARGET=cursor \\
  LLM_ACCESS_RELEASE_MANIFEST=$REMOTE_RELEASE_DIR/cursor-release.latest.env \\
  LLM_ACCESS_STAGED_CURSOR_SERVICE_UNIT=$REMOTE_RELEASE_DIR/llm-access-cursor.service.release \\
  $REMOTE_RELEASE_DIR/$REMOTE_SCRIPT_NAME
EOF
  exit 0
fi

log "running llm-access test suite"
cargo test -p llm-usage-journal -p llm-access-core -p llm-access-store -p llm-access -p llm-access-codex-image --jobs "$BUILD_JOBS"

log "running llm-access clippy"
cargo clippy -p llm-usage-journal -p llm-access-core -p llm-access-store -p llm-access -p llm-access-codex-image --jobs "$BUILD_JOBS" -- -D warnings

log "building release binaries"
cargo build -p llm-access -p llm-access-codex-image --release --jobs "$BUILD_JOBS"

API_BIN="$CARGO_TARGET_DIR/release/llm-access"
WORKER_BIN="$CARGO_TARGET_DIR/release/llm-access-usage-worker"
IMAGE_BIN="$CARGO_TARGET_DIR/release/llm-access-codex-image"
[[ -x "$API_BIN" ]] || fail "built API binary not found or not executable: $API_BIN"
[[ -x "$WORKER_BIN" ]] || fail "built usage worker binary not found or not executable: $WORKER_BIN"
[[ -x "$IMAGE_BIN" ]] || fail "built codex image binary not found or not executable: $IMAGE_BIN"

GIT_COMMIT="$(git rev-parse HEAD)"
GIT_SHORT="$(git rev-parse --short=12 HEAD)"
STATICFLOW_GIT_COMMIT="$(git -C "$ROOT_DIR" rev-parse HEAD)"
RELEASE_ID="${RELEASE_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$GIT_SHORT}"
OUT_DIR="${LLM_ACCESS_RELEASE_OUT:-$ROOT_DIR/tmp/llm-access-cloud-release/$RELEASE_ID}"
STAGED_BIN="$OUT_DIR/llm-access.$RELEASE_ID"
STAGED_WORKER_BIN="$OUT_DIR/llm-access-usage-worker.$RELEASE_ID"
STAGED_IMAGE_BIN="$OUT_DIR/llm-access-codex-image.$RELEASE_ID"
STAGED_NEON_ENV="$OUT_DIR/llm-access-neon.env.$RELEASE_ID"
MANIFEST="$OUT_DIR/release.$RELEASE_ID.env"
SHA_FILE="$OUT_DIR/SHA256SUMS.$RELEASE_ID"
RENDER_DIR="$OUT_DIR/rendered"
STAGED_SERVICE_UNIT="$OUT_DIR/llm-access.service.release"
STAGED_WORKER_SERVICE_UNIT="$OUT_DIR/llm-access-usage-worker.service.release"
STAGED_IMAGE_SERVICE_UNIT="$OUT_DIR/llm-access-codex-image.service.release"
STAGED_USAGE_MOUNT_SERVICE_UNIT="$OUT_DIR/juicefs-llm-access-usage.service.release"

mkdir -p "$OUT_DIR"
"$RENDER_SCRIPT" "$RENDER_DIR"
cp "$API_BIN" "$STAGED_BIN"
cp "$WORKER_BIN" "$STAGED_WORKER_BIN"
cp "$IMAGE_BIN" "$STAGED_IMAGE_BIN"
cp "$LOCAL_NEON_ENV_FILE" "$STAGED_NEON_ENV"
cp "$RENDER_DIR/llm-access.service" "$STAGED_SERVICE_UNIT"
cp "$RENDER_DIR/llm-access-usage-worker.service" "$STAGED_WORKER_SERVICE_UNIT"
cp "$RENDER_DIR/llm-access-codex-image.service" "$STAGED_IMAGE_SERVICE_UNIT"
cp "$RENDER_DIR/juicefs-llm-access-usage.service" "$STAGED_USAGE_MOUNT_SERVICE_UNIT"
chmod 0755 "$STAGED_BIN" "$STAGED_WORKER_BIN" "$STAGED_IMAGE_BIN"
chmod 0600 "$STAGED_NEON_ENV"
API_BIN_SHA="$(sha256sum "$STAGED_BIN" | awk '{print $1}')"
WORKER_BIN_SHA="$(sha256sum "$STAGED_WORKER_BIN" | awk '{print $1}')"
IMAGE_BIN_SHA="$(sha256sum "$STAGED_IMAGE_BIN" | awk '{print $1}')"
{
  printf '%s  %s\n' "$API_BIN_SHA" "llm-access.$RELEASE_ID"
  printf '%s  %s\n' "$WORKER_BIN_SHA" "llm-access-usage-worker.$RELEASE_ID"
  printf '%s  %s\n' "$IMAGE_BIN_SHA" "llm-access-codex-image.$RELEASE_ID"
} >"$SHA_FILE"

cat >"$MANIFEST" <<EOF
release_id=$RELEASE_ID
git_commit=$GIT_COMMIT
git_short=$GIT_SHORT
staticflow_git_commit=$STATICFLOW_GIT_COMMIT
built_at_utc=$(date -u +%FT%TZ)
sha256=$API_BIN_SHA
binary=llm-access.$RELEASE_ID
api_sha256=$API_BIN_SHA
api_binary=llm-access.$RELEASE_ID
usage_worker_sha256=$WORKER_BIN_SHA
usage_worker_binary=llm-access-usage-worker.$RELEASE_ID
codex_image_sha256=$IMAGE_BIN_SHA
codex_image_binary=llm-access-codex-image.$RELEASE_ID
control_neon_env=llm-access-neon.env.$RELEASE_ID
runtime_env=llm-access-neon.env.$RELEASE_ID
EOF

SSH_OPTS=(-i "$GCP_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes)
REMOTE_RELEASE_DIR_Q="$(shell_quote "$REMOTE_RELEASE_DIR")"
REMOTE_SCRIPT_NAME="$(basename "$REMOTE_SCRIPT")"

log "checking GCP target: $GCP_DEST"
ssh "${SSH_OPTS[@]}" "$GCP_DEST" "
  set -e
  mkdir -p $REMOTE_RELEASE_DIR_Q
  systemctl is-active juicefs-llm-access.service >/dev/null
  findmnt -T /mnt/llm-access >/dev/null
  curl -fsS http://127.0.0.1:19080/healthz >/dev/null || echo '[llm-access-release][WARN] current llm-access health check failed; staging continues'
"

log "uploading release $RELEASE_ID to $GCP_DEST:$REMOTE_RELEASE_DIR"
scp "${SSH_OPTS[@]}" \
  "$STAGED_BIN" \
  "$STAGED_WORKER_BIN" \
  "$STAGED_IMAGE_BIN" \
  "$STAGED_NEON_ENV" \
  "$MANIFEST" \
  "$SHA_FILE" \
  "$STAGED_SERVICE_UNIT" \
  "$STAGED_WORKER_SERVICE_UNIT" \
  "$STAGED_IMAGE_SERVICE_UNIT" \
  "$STAGED_USAGE_MOUNT_SERVICE_UNIT" \
  "$REMOTE_SCRIPT" \
  "$GCP_DEST:$REMOTE_RELEASE_DIR/"

log "updating remote latest pointers"
ssh "${SSH_OPTS[@]}" "$GCP_DEST" "
  set -e
  cd $REMOTE_RELEASE_DIR_Q
  chmod 0755 $REMOTE_SCRIPT_NAME llm-access.$RELEASE_ID llm-access-usage-worker.$RELEASE_ID llm-access-codex-image.$RELEASE_ID
  sha256sum -c SHA256SUMS.$RELEASE_ID
  ln -sfn llm-access.$RELEASE_ID llm-access.latest
  ln -sfn llm-access-usage-worker.$RELEASE_ID llm-access-usage-worker.latest
  ln -sfn llm-access-codex-image.$RELEASE_ID llm-access-codex-image.latest
  ln -sfn llm-access-neon.env.$RELEASE_ID llm-access-neon.env.latest
  ln -sfn release.$RELEASE_ID.env release.latest.env
"

cat <<EOF

Prepared llm-access cloud release:
  release_id: $RELEASE_ID
  git_commit: $GIT_COMMIT
  api_sha256: $API_BIN_SHA
  usage_worker_sha256: $WORKER_BIN_SHA
  codex_image_sha256: $IMAGE_BIN_SHA
  local_runtime_env: $LOCAL_NEON_ENV_FILE
  remote_dir: $REMOTE_RELEASE_DIR

Run this on GCP to activate it:
  ssh -i "$GCP_SSH_KEY" -o IdentitiesOnly=yes "$GCP_DEST"
  $REMOTE_RELEASE_DIR/$REMOTE_SCRIPT_NAME

Or activate directly from local:
  ssh -i "$GCP_SSH_KEY" -o IdentitiesOnly=yes "$GCP_DEST" '$REMOTE_RELEASE_DIR/$REMOTE_SCRIPT_NAME'
EOF
