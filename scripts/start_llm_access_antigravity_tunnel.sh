#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_FILE="${LLM_ACCESS_CLOUD_RELEASE_CONFIG:-$ROOT_DIR/.local/llm-access-cloud-release-aws.env}"
[[ -r "$CONFIG_FILE" ]] || { echo "missing config file: $CONFIG_FILE" >&2; exit 1; }
# shellcheck source=/dev/null
source "$CONFIG_FILE"
: "${GCP_SSH_KEY:?missing GCP_SSH_KEY}"
DEST="${GCP_DEST:-${GCP_USER:?missing GCP_USER}@${GCP_HOST:?missing GCP_HOST}}"

while true; do
  ssh -i "$GCP_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes \
    -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 -o ServerAliveCountMax=3 \
    -N -L 127.0.0.1:19195:127.0.0.1:19095 "$DEST" || true
  sleep 3
done
