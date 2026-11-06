#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
test_dir="$(mktemp -d /tmp/llm-access-worker-preserve-test.XXXXXX)"
trap 'rm -rf "$test_dir"' EXIT
mkdir -p "$test_dir/bin" "$test_dir/child" "$test_dir/target/release"
printf '[package]\nname="fixture"\nversion="0.0.0"\n' > "$test_dir/child/Cargo.toml"
printf 'CARGO_TARGET_DIR=%q\nGCP_SSH_KEY=%q\nGCP_DEST=fixture-host\nREMOTE_RELEASE_DIR=/tmp/fixture-release\n' \
  "$test_dir/target" "$test_dir/fixture-key" > "$test_dir/config.env"
touch "$test_dir/fixture-key"
cat > "$test_dir/bin/git" <<'SH'
#!/usr/bin/env bash
case " $* " in
  *' status '*) ;;
  *' rev-parse --short=12 HEAD '*) printf 'aaaaaaaaaaaa\n' ;;
  *' rev-parse '*) printf 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n' ;;
  *' merge-base '*) ;;
  *) exit 1 ;;
esac
SH
cat > "$test_dir/bin/cargo" <<'SH'
#!/usr/bin/env bash
[[ "$1" == build ]] || exit 1
printf '#!/bin/sh\nexit 0\n' > "$CARGO_TARGET_DIR/release/llm-access-usage-worker"
chmod +x "$CARGO_TARGET_DIR/release/llm-access-usage-worker"
SH
cat > "$test_dir/bin/scp" <<'SH'
#!/usr/bin/env bash
for arg in "$@"; do
  [[ "$arg" != *neon.env* ]] || exit 1
done
printf 'upload\n' >> "$FAKE_RELEASE_LOG"
SH
cat > "$test_dir/bin/ssh" <<'SH'
#!/usr/bin/env bash
printf 'ssh\n' >> "$FAKE_RELEASE_LOG"
SH
cat > "$test_dir/bin/pgrep" <<'SH'
#!/usr/bin/env bash
exit 1
SH
chmod +x "$test_dir/bin/"*
PATH="$test_dir/bin:$PATH" FAKE_RELEASE_LOG="$test_dir/calls.log" \
  LLM_ACCESS_DIR="$test_dir/child" LLM_ACCESS_CLOUD_RELEASE_CONFIG="$test_dir/config.env" \
  LLM_ACCESS_LOCAL_NEON_ENV_FILE="$test_dir/missing-neon.env" \
  LLM_ACCESS_PRESERVE_RUNTIME_ENV=1 LLM_ACCESS_ACTIVATE_TARGET=worker \
  LLM_ACCESS_RELEASE_OUT="$test_dir/staged" RELEASE_ID=fixture \
  "$ROOT_DIR/scripts/prepare_llm_access_cloud_release.sh" >/dev/null
test -x "$test_dir/staged/llm-access-usage-worker.fixture"
grep -q '^usage_worker_sha256=' "$test_dir/staged/worker-release.fixture.env"
test "$(grep -c '^upload$' "$test_dir/calls.log")" = 1
printf 'worker release preserves the remote runtime env\n'
