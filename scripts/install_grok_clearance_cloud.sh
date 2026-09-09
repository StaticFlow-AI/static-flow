#!/usr/bin/env bash
# Install the isolated browser-verification service used by Grok billing RPCs.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_FILE="${LLM_ACCESS_CLOUD_RELEASE_CONFIG:-$ROOT_DIR/.local/llm-access-cloud-release-aws.env}"
# shellcheck source=/dev/null
source "$CONFIG_FILE"
GCP_DEST="${GCP_DEST:-$GCP_USER@$GCP_HOST}"
GCP_SSH_KEY="${GCP_SSH_KEY/#\~/$HOME}"
ssh -i "$GCP_SSH_KEY" -o IdentitiesOnly=yes -o BatchMode=yes "$GCP_DEST" 'sudo -n bash -s' <<'REMOTE'
set -euo pipefail
[[ "$(uname -m)" == x86_64 ]] || { echo 'This pinned browser bundle requires x86_64' >&2; exit 1; }
. /etc/os-release
[[ "$ID" == ubuntu && "$VERSION_ID" == 24.04 ]] || { echo 'This installer requires Ubuntu 24.04' >&2; exit 1; }
VERSION=3.5.0
ARCHIVE_SHA=05551d5846cfffd62c3ea24e2d70af5de314470fc7fd5434b3ca130616092b33
INSTALL_DIR="/opt/grok-clearance/$VERSION"
TEMP_DIR="$(mktemp -d /tmp/grok-clearance-install.XXXXXX)"
trap 'rm -rf "$TEMP_DIR"' EXIT
DEBIAN_FRONTEND=noninteractive NEEDRESTART_MODE=l apt-get install -y --no-install-recommends \
  xvfb libnss3 libatk1.0-0 libatk-bridge2.0-0 libcups2t64 libx11-xcb1 libxcomposite1 \
  libxdamage1 libxfixes3 libxrandr2 libgbm1 libcairo2 libpango-1.0-0 libasound2t64 libwayland-server0
if [[ ! -x "$INSTALL_DIR/flaresolverr/flaresolverr" ]]; then
  curl -fsSL --max-time 180 "https://github.com/FlareSolverr/FlareSolverr/releases/download/v$VERSION/flaresolverr_linux_x64.tar.gz" -o "$TEMP_DIR/bundle.tar.gz"
  printf '%s  %s\n' "$ARCHIVE_SHA" "$TEMP_DIR/bundle.tar.gz" | sha256sum --check --status
  mkdir -p "$INSTALL_DIR"
  tar --no-same-owner -xzf "$TEMP_DIR/bundle.tar.gz" -C "$INSTALL_DIR"
fi
id grok-clearance >/dev/null 2>&1 || useradd --system --home-dir /var/lib/grok-clearance --shell /usr/sbin/nologin grok-clearance
install -d -m 0700 -o grok-clearance -g grok-clearance /var/lib/grok-clearance
ln -sfn "$INSTALL_DIR" /opt/grok-clearance/current
cat > /etc/systemd/system/grok-clearance.service <<'UNIT'
[Unit]
Description=Grok billing browser verification
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=grok-clearance
Group=grok-clearance
StateDirectory=grok-clearance
StateDirectoryMode=0700
WorkingDirectory=/opt/grok-clearance/current/flaresolverr
Environment=HOST=127.0.0.1
Environment=PORT=8191
Environment=LOG_LEVEL=warn
Environment=HEADLESS=true
ExecStart=/opt/grok-clearance/current/flaresolverr/flaresolverr
Restart=on-failure
RestartSec=10
TimeoutStopSec=15
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/grok-clearance
MemoryAccounting=yes
MemoryMax=1G
MemorySwapMax=256M
CPUQuota=100%
TasksMax=256
UMask=0077

[Install]
WantedBy=multi-user.target
UNIT
systemctl daemon-reload
systemctl enable --now grok-clearance.service
systemctl show grok-clearance.service -p ActiveState -p SubState -p MainPID -p NRestarts
REMOTE
