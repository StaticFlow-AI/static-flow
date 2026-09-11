#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_FILE="${LLM_ACCESS_CLOUD_RELEASE_CONFIG:-$ROOT_DIR/.local/llm-access-cloud-release-aws.env}"
[[ -r "$CONFIG_FILE" ]] || CONFIG_FILE="$ROOT_DIR/.local/llm-access-cloud-release.env"
# shellcheck source=/dev/null
source "$CONFIG_FILE"
: "${GCP_SSH_KEY:?missing GCP_SSH_KEY in $CONFIG_FILE}"
: "${GCP_DEST:?missing GCP_DEST in $CONFIG_FILE}"
while true; do
  ssh -i "$GCP_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes \
    -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 -o ServerAliveCountMax=3 \
    -N -L 127.0.0.1:19092:127.0.0.1:19092 "$GCP_DEST" || true
  sleep 3
done
