#!/usr/bin/env bash
# One-time schema-91 cutover; use the independent release scripts afterwards.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLM_ACCESS_DIR="${LLM_ACCESS_DIR:-$ROOT_DIR/deps/llm-access}"
CONFIG_FILE="${LLM_ACCESS_CLOUD_RELEASE_CONFIG:-$ROOT_DIR/.local/llm-access-cloud-release-aws.env}"
# shellcheck source=/dev/null
source "$CONFIG_FILE"
: "${CARGO_TARGET_DIR:?}" "${GCP_SSH_KEY:?}" "${REMOTE_RELEASE_DIR:?}"
GCP_DEST="${GCP_DEST:-${GCP_USER:?}@${GCP_HOST:?}}"
export CARGO_TARGET_DIR
[[ -z "$(git -C "$LLM_ACCESS_DIR" status --porcelain)" ]]
[[ -z "$(git -C "$ROOT_DIR" status --porcelain --ignore-submodules=dirty)" ]]
CHILD_REVISION="$(git -C "$LLM_ACCESS_DIR" rev-parse HEAD)"
[[ "$(git -C "$ROOT_DIR" rev-parse HEAD:deps/llm-access)" = "$CHILD_REVISION" ]]
BUILD_JOBS="${BUILD_JOBS:-4}"
cd "$LLM_ACCESS_DIR"
# Run the workspace tests with an isolated TEST_POSTGRES_URL and Clippy before
# invoking this release. Separate builds prevent Cursor features entering AG.
cargo build -p llm-access --bin llm-access --bin llm-access-usage-worker --release --locked --jobs "$BUILD_JOBS"
cargo build -p llm-access-cursor --release --locked --jobs "$BUILD_JOBS"
cargo build -p llm-access-oauth --release --locked --jobs "$BUILD_JOBS"
cargo build -p llm-access-antigravity --release --locked --jobs "$BUILD_JOBS"
RELEASE_ID="$(date -u +%Y%m%dT%H%M%SZ)-${CHILD_REVISION:0:12}-managed"
STAGE="$ROOT_DIR/tmp/llm-access-cloud-release/$RELEASE_ID"
mkdir -p "$STAGE"
python3 - "$CARGO_TARGET_DIR/release" "$STAGE" "$CHILD_REVISION" <<'PY'
import hashlib,json,shutil,sys
from pathlib import Path
source,stage=map(Path,sys.argv[1:3])
manifest={'child_revision':sys.argv[3],'binaries':{}}
for name in ('llm-access','llm-access-usage-worker','llm-access-cursor','llm-access-antigravity','llm-access-oauth'):
    shutil.copy2(source/name,stage/name)
    manifest['binaries'][name]=hashlib.sha256((stage/name).read_bytes()).hexdigest()
(stage/'manifest.json').write_text(json.dumps(manifest,indent=2))
PY
cp "$LLM_ACCESS_DIR/crates/llm-access-migrations/migrations/postgres/0091_managed_accounts.sql" "$STAGE/"
cp "$ROOT_DIR/scripts/activate_llm_access_managed_accounts.py" "$STAGE/"
SSH_OPTS=(-i "$GCP_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes -o ControlMaster=no -o ControlPath=none)
REMOTE_STAGE="$REMOTE_RELEASE_DIR/$RELEASE_ID"
printf -v REMOTE_STAGE_Q '%q' "$REMOTE_STAGE"
ssh "${SSH_OPTS[@]}" "$GCP_DEST" "mkdir -m 700 -p $REMOTE_STAGE_Q"
scp "${SSH_OPTS[@]}" "$STAGE/"* "$GCP_DEST:$REMOTE_STAGE/"
ssh "${SSH_OPTS[@]}" "$GCP_DEST" "python3 $REMOTE_STAGE_Q/activate_llm_access_managed_accounts.py $REMOTE_STAGE_Q" | tee "$STAGE/activation.json"
printf 'Managed-account cutover complete: %s\n' "$RELEASE_ID"
