#!/usr/bin/env bash
set -euo pipefail

log() { echo "[ai-review-start] $*" >&2; }

env_value() {
  local wanted="$1"
  local file="$2"
  local key value

  while IFS='=' read -r key value; do
    key="${key#"${key%%[![:space:]]*}"}"
    key="${key%"${key##*[![:space:]]}"}"
    [[ "$key" == "$wanted" ]] || continue
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    value="${value%\"}"
    value="${value#\"}"
    value="${value%\'}"
    value="${value#\'}"
    printf '%s' "$value"
    return 0
  done <"$file"
  return 1
}

tcp_reachable() {
  local host="$1"
  local port="$2"
  timeout 3 bash -c "</dev/tcp/$host/$port" >/dev/null 2>&1
}

maybe_override_synthetic_database_dns() {
  local env_file="$1"
  local database_url authority host_port host port resolved_ip public_ip endpoint_id
  local scheme rest suffix userinfo rewritten_host_port

  database_url="${LLM_ACCESS_CONTROL_DATABASE_URL:-${DATABASE_URL:-}}"
  if [[ -z "$database_url" ]]; then
    database_url="$(env_value LLM_ACCESS_CONTROL_DATABASE_URL "$env_file" || true)"
  fi
  if [[ -z "$database_url" ]]; then
    database_url="$(env_value DATABASE_URL "$env_file" || true)"
  fi
  [[ -n "$database_url" ]] || return 0

  authority="${database_url#*://}"
  authority="${authority%%/*}"
  host_port="${authority##*@}"
  host="${host_port%%:*}"
  port="${host_port##*:}"
  [[ "$port" != "$host_port" ]] || port=5432
  [[ -n "$host" ]] || return 0

  resolved_ip="$(getent ahostsv4 "$host" 2>/dev/null | awk 'NR == 1 { print $1 }' || true)"
  case "$resolved_ip" in
    198.18.*|198.19.*) ;;
    *) return 0 ;;
  esac
  if tcp_reachable "$host" "$port"; then
    return 0
  fi
  if [[ "$database_url" == *"sslmode=verify-full"* ]]; then
    log "database DNS returned unreachable synthetic address $resolved_ip; cannot use an IP override with sslmode=verify-full"
    return 0
  fi
  command -v dig >/dev/null 2>&1 || {
    log "database DNS returned unreachable synthetic address $resolved_ip, but dig is unavailable"
    return 0
  }

  while read -r public_ip; do
    [[ -n "$public_ip" ]] || continue
    if tcp_reachable "$public_ip" "$port"; then
      scheme="${database_url%%://*}"
      rest="${database_url#*://}"
      suffix="${rest#"$authority"}"
      userinfo="${authority%"$host_port"}"
      rewritten_host_port="${host_port/"$host"/"$public_ip"}"
      database_url="${scheme}://${userinfo}${rewritten_host_port}${suffix}"
      if [[ "$host" == *.neon.tech && "$database_url" != *"options=endpoint%3D"* ]]; then
        endpoint_id="${host%%.*}"
        if [[ "$database_url" == *"?"* ]]; then
          database_url+="&options=endpoint%3D${endpoint_id}"
        else
          database_url+="?options=endpoint%3D${endpoint_id}"
        fi
      fi
      export LLM_ACCESS_CONTROL_DATABASE_URL="$database_url"
      log "database DNS returned unreachable synthetic address $resolved_ip; using reachable public address $public_ip"
      return 0
    fi
  done < <(dig +short @1.1.1.1 "$host" A | awk '/^[0-9]+(\.[0-9]+){3}$/')

  log "database DNS returned unreachable synthetic address $resolved_ip; no reachable public address found"
}

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <env-file> <ai-review-binary> [arguments...]" >&2
  exit 2
fi

env_file="$1"
binary="$2"
shift 2

maybe_override_synthetic_database_dns "$env_file"
exec "$binary" --env-file "$env_file" "$@"
