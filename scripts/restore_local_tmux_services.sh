#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

source "$ROOT_DIR/scripts/lib_pingora_gateway_conf.sh"

CONF_FILE="${CONF_FILE:-$ROOT_DIR/conf/pingora/staticflow-gateway.yaml}"
PINGORA_CONF_TEMPLATE_FILE="${PINGORA_CONF_TEMPLATE_FILE:-$ROOT_DIR/conf/pingora/staticflow-gateway.yaml.template}"
BACKEND_BIN="${BACKEND_BIN:-$ROOT_DIR/target/release-backend/static-flow-backend}"
AI_REVIEW_ENV_FILE="${AI_REVIEW_ENV_FILE:-$ROOT_DIR/.local/llm-access-neon.env}"
AI_REVIEW_BIN="${AI_REVIEW_BIN:-/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow/debug/llm-access-ai-review}"
GPT2API_BIN="${GPT2API_BIN:-/mnt/wsl/data4tb/static-flow-data/cargo-target/gpt2api_rs/release/gpt2api-rs}"
GPT2API_TARGET_DIR="${GPT2API_TARGET_DIR:-/mnt/wsl/data4tb/static-flow-data/cargo-target/gpt2api_rs/release}"
MEDIA_ROOT="${MEDIA_ROOT:-/mnt/e/videos/static}"
HOME_PBMAPPER_SERVER="${HOME_PBMAPPER_SERVER:-lb7666.top:7666}"

DRY_RUN=0
WITH_AI_REVIEW=0
ONLY_STATUS=0

log() { echo "[restore-tmux] $*"; }
warn() { echo "[restore-tmux][WARN] $*" >&2; }
fail() { echo "[restore-tmux][ERROR] $*" >&2; exit 1; }
q() { printf '%q' "$1"; }

usage() {
  cat <<'EOF'
Usage:
  ./scripts/restore_local_tmux_services.sh [--dry-run] [--with-ai-review]
  ./scripts/restore_local_tmux_services.sh status

Restores the local tmux-supervised service set after a reboot:
  - sf-media
  - active sf-backend-<slot> from conf/pingora/staticflow-gateway.yaml
  - sf-gateway
  - pbmapper-sf-backend-aws
  - pbmapper-llm-access-aws
  - pbmapper-home-ubuntu-aws
  - gpt2api-rs

AI review is intentionally opt-in:
  --with-ai-review starts sf-ai-review, sf-ai-review-ui, and pbmapper-ai-review-ui-aws.
  It only registers the local UI through pbmapper; it does not create a public Caddy route.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --with-ai-review)
      WITH_AI_REVIEW=1
      shift
      ;;
    status)
      ONLY_STATUS=1
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

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "missing command: $1"
}

session_exists() {
  tmux has-session -t "$1" 2>/dev/null
}

port_listening() {
  ss -ltn 2>/dev/null | awk '{print $4}' | grep -qE "(^|:)${1}$"
}

start_tmux() {
  local session="$1"
  local command="$2"
  local quoted

  if session_exists "$session"; then
    log "skip $session: tmux session already exists"
    return
  fi

  printf -v quoted '%q' "$command"
  if [[ "$DRY_RUN" == "1" ]]; then
    log "dry-run start $session: $command"
    return
  fi

  log "start $session"
  tmux new-session -d -s "$session" -c "$ROOT_DIR" "exec /usr/bin/bash -lc $quoted"
}

wait_http() {
  local name="$1"
  local url="$2"
  local attempts="${3:-40}"

  if [[ "$DRY_RUN" == "1" ]]; then
    return
  fi

  for _ in $(seq 1 "$attempts"); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      log "ready $name: $url"
      return
    fi
    sleep 0.5
  done
  warn "$name did not become ready yet: $url"
}

wait_tcp() {
  local name="$1"
  local port="$2"
  local attempts="${3:-40}"

  if [[ "$DRY_RUN" == "1" ]]; then
    return
  fi

  for _ in $(seq 1 "$attempts"); do
    if port_listening "$port"; then
      log "ready $name: 127.0.0.1:$port"
      return
    fi
    sleep 0.5
  done
  warn "$name did not open port yet: 127.0.0.1:$port"
}

print_status() {
  log "tmux sessions"
  tmux list-sessions 2>/dev/null | grep -E '^(sf-|pbmapper-|gpt2api-rs)' || true
  log "listening ports"
  ss -ltnp 2>/dev/null | grep -E ':(18787|19182|19190|19191|39080|39081|39085|39180)\b' || true
}

active_slot() {
  pingora_ensure_conf_file "$CONF_FILE" "$PINGORA_CONF_TEMPLATE_FILE"
  pingora_staticflow_conf_value "$CONF_FILE" "active_upstream"
}

slot_port() {
  local slot="$1"
  local addr
  addr="$(pingora_staticflow_upstream_addr "$CONF_FILE" "$slot")"
  [[ -n "$addr" ]] || fail "missing address for slot=$slot in $CONF_FILE"
  echo "${addr##*:}"
}

start_media() {
  local cmd
  cmd="cd $(q "$ROOT_DIR") && MEDIA_BIN=$(q "$ROOT_DIR/bin/static-flow-media-canary") STATICFLOW_LOCAL_MEDIA_ROOT=$(q "$MEDIA_ROOT") exec ./scripts/start_media_service_canary.sh --port 39085"
  start_tmux "sf-media" "$cmd"
  wait_http "sf-media" "http://127.0.0.1:39085/internal/local-media/list?limit=1" 30
}

start_backend() {
  local slot
  local port
  local cmd

  [[ -x "$BACKEND_BIN" ]] || fail "missing executable: $BACKEND_BIN"
  slot="$(active_slot)"
  [[ "$slot" == "blue" || "$slot" == "green" ]] || fail "unsupported active_upstream=$slot"
  port="$(slot_port "$slot")"

  if port_listening "$port" && ! session_exists "sf-backend-$slot"; then
    warn "127.0.0.1:$port is already listening but sf-backend-$slot tmux session is absent; leaving existing process untouched"
    return
  fi

  cmd="cd $(q "$ROOT_DIR") && BACKEND_BIN=$(q "$BACKEND_BIN") exec ./scripts/start_backend_selfhosted_slot.sh $(q "$slot")"
  start_tmux "sf-backend-$slot" "$cmd"
  wait_http "sf-backend-$slot" "http://127.0.0.1:${port}/api/articles?page=1&per_page=1" 80
}

start_gateway() {
  local cmd
  cmd="cd $(q "$ROOT_DIR") && STATICFLOW_LOG_SERVICE=gateway exec ./scripts/pingora_gateway.sh run"
  start_tmux "sf-gateway" "$cmd"
  wait_http "sf-gateway" "http://127.0.0.1:39180/api/articles?page=1&per_page=1" 60
}

start_pbmapper_sf_backend() {
  local cmd
  [[ -f "$ROOT_DIR/.local/pbmapper/sf-backend.env" ]] || fail "missing .local/pbmapper/sf-backend.env"
  cmd="cd $(q "$ROOT_DIR") && set -a && . .local/pbmapper/sf-backend.env && set +a && exec pb-mapper-server-cli tcp-server --key \"\$SERVICE_KEY\" --addr \"\$LOCAL_ADDR\""
  start_tmux "pbmapper-sf-backend-aws" "$cmd"
}

start_pbmapper_llm_access() {
  local cmd
  [[ -f "$ROOT_DIR/.local/pbmapper/llm-access.env" ]] || fail "missing .local/pbmapper/llm-access.env"
  cmd="cd $(q "$ROOT_DIR") && set -a && . .local/pbmapper/llm-access.env && set +a && exec pb-mapper-client-cli tcp-server --key \"\$SERVICE_KEY\" --addr \"\$LOCAL_ADDR\""
  start_tmux "pbmapper-llm-access-aws" "$cmd"
  wait_tcp "pbmapper-llm-access-aws" "19182" 40
}

start_pbmapper_home_ubuntu() {
  local cmd
  cmd="cd $(q "$ROOT_DIR") && PB_MAPPER_SERVER=$(q "$HOME_PBMAPPER_SERVER") exec pb-mapper-server-cli -p $(q "$HOME_PBMAPPER_SERVER") tcp-server --key home-ubuntu --addr 127.0.0.1:22"
  start_tmux "pbmapper-home-ubuntu-aws" "$cmd"
}

start_gpt2api() {
  local cmd
  [[ -x "$GPT2API_BIN" ]] || fail "missing executable: $GPT2API_BIN"
  [[ -f "$ROOT_DIR/conf/gpt2api-rs.json" ]] || fail "missing conf/gpt2api-rs.json"
  cmd="cd $(q "$ROOT_DIR") && ADMIN_TOKEN=\$(jq -r .admin_token conf/gpt2api-rs.json) && exec env LD_LIBRARY_PATH=$(q "$GPT2API_TARGET_DIR/deps") SITE_BASE_URL=https://ackingliu.top GPT2API_PUBLIC_BASE_URL=https://ackingliu.top GPT2API_EMAIL_ACCOUNTS_FILE=$(q "$ROOT_DIR/crates/backend/.local/email_accounts.json") $(q "$GPT2API_BIN") serve --listen 127.0.0.1:18787 --storage-dir /mnt/wsl/data4tb/static-flow-data/gpt2api-rs --admin-token \"\$ADMIN_TOKEN\""
  start_tmux "gpt2api-rs" "$cmd"
  wait_http "gpt2api-rs" "http://127.0.0.1:18787/healthz" 40
}

start_ai_review() {
  local api_cmd
  local ui_cmd
  local pbmapper_cmd

  [[ -x "$AI_REVIEW_BIN" ]] || fail "missing executable: $AI_REVIEW_BIN"
  [[ -f "$AI_REVIEW_ENV_FILE" ]] || fail "missing ai review env file: $AI_REVIEW_ENV_FILE"
  api_cmd="cd $(q "$ROOT_DIR") && if [[ -f /home/ts_user/.antigravity-agent/gui_config.json ]]; then export ANTIGRAVITY_MANAGER_API_KEY=\$(jq -r '.proxy.api_key // .api_key // empty' /home/ts_user/.antigravity-agent/gui_config.json); fi && exec $(q "$AI_REVIEW_BIN") --env-file $(q "$AI_REVIEW_ENV_FILE") serve --bind 127.0.0.1:19190"
  ui_cmd="cd $(q "$ROOT_DIR/crates/llm-access-ai-review/ui") && exec npm run dev -- --host 127.0.0.1"
  pbmapper_cmd="cd $(q "$ROOT_DIR") && set -a && . .local/pbmapper/sf-backend.env && set +a && SERVICE_KEY=ai-review-ui LOCAL_ADDR=127.0.0.1:19191 exec pb-mapper-server-cli tcp-server --key \"\$SERVICE_KEY\" --addr \"\$LOCAL_ADDR\""

  start_tmux "sf-ai-review" "$api_cmd"
  wait_http "sf-ai-review" "http://127.0.0.1:19190/api/ai-review/health" 60
  start_tmux "sf-ai-review-ui" "$ui_cmd"
  wait_http "sf-ai-review-ui" "http://127.0.0.1:19191/api/ai-review/health" 60
  start_tmux "pbmapper-ai-review-ui-aws" "$pbmapper_cmd"
}

main() {
  require_command tmux
  require_command ss
  require_command curl
  require_command jq
  require_command pb-mapper-server-cli
  require_command pb-mapper-client-cli

  if [[ "$ONLY_STATUS" == "1" ]]; then
    print_status
    exit 0
  fi

  start_media
  start_backend
  start_gateway
  start_pbmapper_sf_backend
  start_pbmapper_llm_access
  start_pbmapper_home_ubuntu
  start_gpt2api

  if [[ "$WITH_AI_REVIEW" == "1" ]]; then
    require_command npm
    start_ai_review
  else
    log "skip ai review: use --with-ai-review when you explicitly want it restored"
  fi

  print_status
}

main "$@"
