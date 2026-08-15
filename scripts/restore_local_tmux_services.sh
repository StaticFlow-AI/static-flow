#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

source "$ROOT_DIR/scripts/lib_pingora_gateway_conf.sh"

CONF_FILE="${CONF_FILE:-$ROOT_DIR/conf/pingora/staticflow-gateway.yaml}"
PINGORA_CONF_TEMPLATE_FILE="${PINGORA_CONF_TEMPLATE_FILE:-$ROOT_DIR/conf/pingora/staticflow-gateway.yaml.template}"
BACKEND_BIN="${BACKEND_BIN:-$ROOT_DIR/target/release-backend/static-flow-backend}"
AI_REVIEW_ENV_FILE="${AI_REVIEW_ENV_FILE:-$ROOT_DIR/.local/llm-access-neon.env}"
AI_REVIEW_BIN="${AI_REVIEW_BIN:-/mnt/wsl/data4tb/static-flow-data/cargo-target/static_flow/release/llm-access-ai-review}"
AI_REVIEW_START_SCRIPT="${AI_REVIEW_START_SCRIPT:-$ROOT_DIR/scripts/start_ai_review_local.sh}"
LLM_ACCESS_FRONTEND_DIR="${LLM_ACCESS_FRONTEND_DIR:-$ROOT_DIR/deps/llm-access/apps/llm-access-frontend}"
LLM_ACCESS_FRONTEND_SERVICE_KEY="${LLM_ACCESS_FRONTEND_SERVICE_KEY:-ai-review-ui}"
GPT2API_BIN="${GPT2API_BIN:-/mnt/wsl/data4tb/static-flow-data/cargo-target/gpt2api_rs/release/gpt2api-rs}"
GPT2API_TARGET_DIR="${GPT2API_TARGET_DIR:-/mnt/wsl/data4tb/static-flow-data/cargo-target/gpt2api_rs/release}"
ANTIGRAVITY_DIR="${ANTIGRAVITY_DIR:-$ROOT_DIR/deps/AntigravityManager}"
ANTIGRAVITY_CONFIG_FILE="${ANTIGRAVITY_CONFIG_FILE:-$HOME/.antigravity-agent/gui_config.json}"
MEDIA_ROOT="${MEDIA_ROOT:-/mnt/e/videos/static}"
HOME_PBMAPPER_SERVER="${HOME_PBMAPPER_SERVER:-lb7666.top:7666}"

DRY_RUN=0
STRICT_READINESS=0
FULL_RECOVERY=0
WITH_ANTIGRAVITY=0
WITH_LLM_ACCESS_FRONTEND=0
ONLY_STATUS=0

log() { echo "[restore-tmux] $*"; }
warn() { echo "[restore-tmux][WARN] $*" >&2; }
fail() { echo "[restore-tmux][ERROR] $*" >&2; exit 1; }
q() { printf '%q' "$1"; }

usage() {
  cat <<'EOF'
Usage:
  ./scripts/restore_local_tmux_services.sh [--dry-run] [--strict] [--full]
  ./scripts/restore_local_tmux_services.sh [--with-antigravity] [--with-llm-access-frontend]
  ./scripts/restore_local_tmux_services.sh status

Restores the local tmux-supervised service set after a reboot:
  - sf-media
  - active sf-backend-<slot> from conf/pingora/staticflow-gateway.yaml
  - sf-gateway
  - pbmapper-sf-backend-aws
  - pbmapper-llm-access-aws
  - pbmapper-home-ubuntu-aws
  - gpt2api-rs

Optional services:
  --with-antigravity starts Antigravity Manager and verifies its authenticated API.
  --with-llm-access-frontend starts sf-ai-review, sf-llm-access-frontend, and pbmapper-llm-access-frontend-aws.
  --with-ai-review is retained as an alias for the same combined frontend stack.
  --full enables both optional service groups and verifies the local /llm-access route.
  --strict exits immediately when a service does not become ready.

AI review and the llm-access frontend are intentionally opt-in unless --full is used:
  The frontend keeps the existing ai-review-ui mapper key by default for remote
  compatibility; it does not create a public Caddy route.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --strict)
      STRICT_READINESS=1
      shift
      ;;
    --full)
      FULL_RECOVERY=1
      WITH_ANTIGRAVITY=1
      WITH_LLM_ACCESS_FRONTEND=1
      shift
      ;;
    --with-antigravity)
      WITH_ANTIGRAVITY=1
      shift
      ;;
    --with-ai-review|--with-llm-access-frontend)
      WITH_LLM_ACCESS_FRONTEND=1
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

ensure_node_toolchain() {
  if node_toolchain_supported; then
    return
  fi

  export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
  if [[ -s "$NVM_DIR/nvm.sh" ]]; then
    # shellcheck source=/dev/null
    source "$NVM_DIR/nvm.sh"
    nvm use --silent default >/dev/null
  fi
  node_toolchain_supported || fail "Antigravity Manager requires Node >=22.14 and npm >=10"
}

session_exists() {
  tmux has-session -t "=$1" 2>/dev/null
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

readiness_failure() {
  if [[ "$STRICT_READINESS" == "1" ]]; then
    fail "$1"
  fi
  warn "$1"
}

wait_http() {
  local name="$1"
  local url="$2"
  local attempts="${3:-40}"

  if [[ "$DRY_RUN" == "1" ]]; then
    return
  fi

  for _ in $(seq 1 "$attempts"); do
    if curl -fsS --connect-timeout 1 --max-time 3 "$url" >/dev/null 2>&1; then
      log "ready $name: $url"
      return
    fi
    sleep 0.5
  done
  readiness_failure "$name did not become ready: $url"
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
  readiness_failure "$name did not open port: 127.0.0.1:$port"
}

print_status() {
  log "tmux sessions"
  tmux list-sessions 2>/dev/null | grep -E '^(sf-|pbmapper-|gpt2api-rs|antigravity-manager)' || true
  log "listening ports"
  ss -ltnp 2>/dev/null | grep -E ':(8045|18787|19182|19190|19191|39080|39081|39085|39180)\b' || true
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

  if [[ "$DRY_RUN" != "1" ]]; then
    [[ -x "$BACKEND_BIN" ]] || fail "missing executable: $BACKEND_BIN"
  fi
  slot="$(active_slot)"
  [[ "$slot" == "blue" || "$slot" == "green" ]] || fail "unsupported active_upstream=$slot"
  port="$(slot_port "$slot")"

  if [[ "$DRY_RUN" != "1" ]] && port_listening "$port" && ! session_exists "sf-backend-$slot"; then
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
  [[ -f "$ROOT_DIR/.local/pbmapper/cloud-server.env" ]] || fail "missing .local/pbmapper/cloud-server.env"
  cmd="cd $(q "$ROOT_DIR") && set -a && . .local/pbmapper/cloud-server.env && set +a && PB_MAPPER_SERVER=$(q "$HOME_PBMAPPER_SERVER") exec pb-mapper-server-cli -p $(q "$HOME_PBMAPPER_SERVER") tcp-server --key home-ubuntu --addr 127.0.0.1:22"
  start_tmux "pbmapper-home-ubuntu-aws" "$cmd"
}

start_gpt2api() {
  local cmd
  if [[ "$DRY_RUN" != "1" ]]; then
    [[ -x "$GPT2API_BIN" ]] || fail "missing executable: $GPT2API_BIN"
  fi
  [[ -f "$ROOT_DIR/conf/gpt2api-rs.json" ]] || fail "missing conf/gpt2api-rs.json"
  cmd="cd $(q "$ROOT_DIR") && ADMIN_TOKEN=\$(jq -r .admin_token conf/gpt2api-rs.json) && exec env LD_LIBRARY_PATH=$(q "$GPT2API_TARGET_DIR/deps") SITE_BASE_URL=https://ackingliu.top GPT2API_PUBLIC_BASE_URL=https://ackingliu.top GPT2API_EMAIL_ACCOUNTS_FILE=$(q "$ROOT_DIR/crates/backend/.local/email_accounts.json") $(q "$GPT2API_BIN") serve --listen 127.0.0.1:18787 --storage-dir /mnt/wsl/data4tb/static-flow-data/gpt2api-rs --admin-token \"\$ADMIN_TOKEN\""
  start_tmux "gpt2api-rs" "$cmd"
  wait_http "gpt2api-rs" "http://127.0.0.1:18787/healthz" 40
}

antigravity_port() {
  jq -er '
    (.proxy.port // 8045)
    | if type == "number" and . >= 1 and . <= 65535 then . else error("invalid proxy port") end
  ' "$ANTIGRAVITY_CONFIG_FILE"
}

antigravity_ready() {
  local api_key
  local port

  [[ -f "$ANTIGRAVITY_CONFIG_FILE" ]] || return 1
  port="$(antigravity_port)" || return 1
  api_key="$(jq -er '.proxy.api_key | select(type == "string" and length > 0)' "$ANTIGRAVITY_CONFIG_FILE")" || return 1

  curl -fsS --connect-timeout 1 --max-time 3 \
    --header @<(printf 'Authorization: Bearer %s\n' "$api_key") \
    "http://127.0.0.1:${port}/v1/models" >/dev/null 2>&1
}

wait_antigravity() {
  local attempts="${1:-120}"
  local port

  if [[ "$DRY_RUN" == "1" ]]; then
    return
  fi

  port="$(antigravity_port)"
  for _ in $(seq 1 "$attempts"); do
    if antigravity_ready; then
      log "ready antigravity-manager: 127.0.0.1:$port"
      return
    fi
    sleep 0.5
  done
  readiness_failure "antigravity-manager did not become ready: 127.0.0.1:$port"
}

start_antigravity() {
  local cmd
  local port

  [[ -d "$ANTIGRAVITY_DIR" ]] || fail "missing Antigravity Manager checkout: $ANTIGRAVITY_DIR"
  [[ -f "$ANTIGRAVITY_CONFIG_FILE" ]] || fail "missing Antigravity Manager config: $ANTIGRAVITY_CONFIG_FILE"
  [[ -d "$ANTIGRAVITY_DIR/node_modules" ]] || fail "missing Antigravity Manager node_modules: $ANTIGRAVITY_DIR/node_modules"
  jq -e '.proxy.auto_start == true' "$ANTIGRAVITY_CONFIG_FILE" >/dev/null \
    || fail "Antigravity Manager proxy.auto_start must be true"
  ANTIGRAVITY_DIR="$ANTIGRAVITY_DIR" "$ROOT_DIR/scripts/start_antigravity_manager.sh" --check
  port="$(antigravity_port)"

  if [[ "$DRY_RUN" != "1" ]] && antigravity_ready; then
    if ! session_exists "antigravity-manager"; then
      warn "Antigravity Manager is ready on port $port but its tmux session is absent; leaving it untouched"
    else
      log "skip antigravity-manager: service is already ready"
    fi
    return
  fi
  if [[ "$DRY_RUN" != "1" ]] && port_listening "$port" && ! session_exists "antigravity-manager"; then
    fail "127.0.0.1:$port is occupied but Antigravity Manager is not healthy"
  fi

  cmd="cd $(q "$ROOT_DIR") && export PATH=$(q "$PATH") && ANTIGRAVITY_DIR=$(q "$ANTIGRAVITY_DIR") exec ./scripts/start_antigravity_manager.sh"
  start_tmux "antigravity-manager" "$cmd"
  wait_antigravity 120
}

start_llm_access_frontend_stack() {
  local api_cmd
  local ui_cmd
  local pbmapper_cmd

  if [[ "$DRY_RUN" != "1" ]]; then
    [[ -x "$AI_REVIEW_BIN" ]] || fail "missing executable: $AI_REVIEW_BIN"
    [[ -x "$AI_REVIEW_START_SCRIPT" ]] || fail "missing executable: $AI_REVIEW_START_SCRIPT"
  fi
  [[ -f "$AI_REVIEW_ENV_FILE" ]] || fail "missing ai review env file: $AI_REVIEW_ENV_FILE"
  [[ -f "$LLM_ACCESS_FRONTEND_DIR/package.json" ]] || fail "missing llm-access frontend: $LLM_ACCESS_FRONTEND_DIR"
  api_cmd="cd $(q "$ROOT_DIR") && if [[ -f $(q "$ANTIGRAVITY_CONFIG_FILE") ]]; then export ANTIGRAVITY_MANAGER_API_KEY=\$(jq -r '.proxy.api_key // .api_key // empty' $(q "$ANTIGRAVITY_CONFIG_FILE")); fi && exec $(q "$AI_REVIEW_START_SCRIPT") $(q "$AI_REVIEW_ENV_FILE") $(q "$AI_REVIEW_BIN") serve --bind 127.0.0.1:19190"
  ui_cmd="cd $(q "$LLM_ACCESS_FRONTEND_DIR") && export PATH=$(q "$PATH") && npm run build && exec npm run preview"
  pbmapper_cmd="cd $(q "$ROOT_DIR") && set -a && . .local/pbmapper/sf-backend.env && set +a && SERVICE_KEY=$(q "$LLM_ACCESS_FRONTEND_SERVICE_KEY") && LOCAL_ADDR=127.0.0.1:19191 && exec pb-mapper-server-cli tcp-server --key \"\$SERVICE_KEY\" --addr \"\$LOCAL_ADDR\""

  start_tmux "sf-ai-review" "$api_cmd"
  wait_http "sf-ai-review" "http://127.0.0.1:19190/api/ai-review/health" 60
  start_tmux "sf-llm-access-frontend" "$ui_cmd"
  wait_http "sf-llm-access-frontend" "http://127.0.0.1:19191/healthz" 120
  wait_http "sf-llm-access-frontend-ai-review" "http://127.0.0.1:19191/api/ai-review/health" 20
  start_tmux "pbmapper-llm-access-frontend-aws" "$pbmapper_cmd"
}

verify_full_recovery() {
  wait_http "gateway-health" "http://127.0.0.1:39180/api/healthz" 20
  wait_http "llm-access-page" "http://127.0.0.1:39180/llm-access" 20
  wait_antigravity 20
  wait_http "sf-ai-review" "http://127.0.0.1:19190/api/ai-review/health" 20
  wait_http "sf-llm-access-frontend" "http://127.0.0.1:19191/console" 20
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

  if [[ "$WITH_ANTIGRAVITY" == "1" ]]; then
    require_command git
    ensure_node_toolchain
    start_antigravity
  else
    log "skip Antigravity Manager: use --with-antigravity or --full to restore it"
  fi

  if [[ "$WITH_LLM_ACCESS_FRONTEND" == "1" ]]; then
    ensure_node_toolchain
    start_llm_access_frontend_stack
  else
    log "skip llm-access frontend: use --with-llm-access-frontend or --full to restore it"
  fi

  if [[ "$FULL_RECOVERY" == "1" ]]; then
    verify_full_recovery
  fi

  print_status
}

main "$@"
