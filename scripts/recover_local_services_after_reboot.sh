#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_MOUNT="${DATA_MOUNT:-/mnt/wsl/data4tb}"
DATA_ROOT="${DB_ROOT:-$DATA_MOUNT/static-flow-data}"
READ_PROBE="${RECOVERY_READ_PROBE:-$DATA_ROOT/cargo-target/static_flow/release-backend/static-flow-backend}"

DRY_RUN=0
STATUS_ONLY=0

log() { echo "[recover-local] $*"; }
fail() { echo "[recover-local][ERROR] $*" >&2; exit 1; }

usage() {
  cat <<'EOF'
Usage:
  ./scripts/recover_local_services_after_reboot.sh
  ./scripts/recover_local_services_after_reboot.sh --dry-run
  ./scripts/recover_local_services_after_reboot.sh status

Validates the external ext4 data disk, then strictly restores the complete local stack,
including Antigravity Manager and AI Reviewer.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    status)
      STATUS_ONLY=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ "$STATUS_ONLY" == "1" ]]; then
  exec "$ROOT_DIR/scripts/restore_local_tmux_services.sh" status
fi

if [[ "$DRY_RUN" == "1" ]]; then
  exec "$ROOT_DIR/scripts/restore_local_tmux_services.sh" --dry-run --strict --full
fi

command -v findmnt >/dev/null 2>&1 || fail "missing command: findmnt"
command -v head >/dev/null 2>&1 || fail "missing command: head"
command -v mountpoint >/dev/null 2>&1 || fail "missing command: mountpoint"
command -v timeout >/dev/null 2>&1 || fail "missing command: timeout"

mountpoint -q "$DATA_MOUNT" || fail "$DATA_MOUNT is not mounted; run the Windows recovery launcher as administrator"

fs_type="$(findmnt -n -o FSTYPE --target "$DATA_MOUNT")"
[[ "$fs_type" == "ext4" ]] || fail "$DATA_MOUNT uses $fs_type instead of ext4"

mount_options="$(findmnt -n -o OPTIONS --target "$DATA_MOUNT")"
case ",$mount_options," in
  *,rw,*) ;;
  *) fail "$DATA_MOUNT is not mounted read-write" ;;
esac

[[ -d "$DATA_ROOT/lancedb" ]] || fail "missing content database: $DATA_ROOT/lancedb"
[[ -d "$DATA_ROOT/lancedb-comments" ]] || fail "missing comments database: $DATA_ROOT/lancedb-comments"
[[ -d "$DATA_ROOT/lancedb-music" ]] || fail "missing music database: $DATA_ROOT/lancedb-music"
[[ -r "$READ_PROBE" ]] || fail "missing recovery read probe: $READ_PROBE"

log "probing the data disk before starting services"
timeout --kill-after=2s 8s head -c 4096 "$READ_PROBE" >/dev/null \
  || fail "data disk read probe failed: $READ_PROBE"

log "data disk is mounted and readable; restoring the complete service set"
exec "$ROOT_DIR/scripts/restore_local_tmux_services.sh" --strict --full
