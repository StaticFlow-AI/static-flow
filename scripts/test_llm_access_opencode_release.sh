#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
test_dir="$(mktemp -d /tmp/llm-access-opencode-release-test.XXXXXX)"
release_id="opencode-test-$$"
stage="$ROOT_DIR/tmp/llm-access-cloud-release/$release_id-opencode"
trap 'rm -rf "$test_dir" "$stage"' EXIT
mkdir -p "$test_dir/bin" "$test_dir/child" "$test_dir/target/release"
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
if [[ "$1" == build ]]; then
  printf '#!/bin/sh\nexit 0\n' > "$CARGO_TARGET_DIR/release/llm-access-opencode-gateway"
  chmod +x "$CARGO_TARGET_DIR/release/llm-access-opencode-gateway"
fi
SH
cat > "$test_dir/bin/scp" <<'SH'
#!/usr/bin/env bash
for arg in "$@"; do [[ "$arg" != *neon.env* ]] || exit 1; done
SH
cat > "$test_dir/bin/ssh" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "${!#}" > "$FAKE_REMOTE_COMMAND"
SH
cat > "$test_dir/bin/pgrep" <<'SH'
#!/usr/bin/env bash
exit 1
SH
chmod +x "$test_dir/bin/"*
PATH="$test_dir/bin:$PATH" FAKE_REMOTE_COMMAND="$test_dir/remote-command" \
  LLM_ACCESS_DIR="$test_dir/child" LLM_ACCESS_CLOUD_RELEASE_CONFIG="$test_dir/config.env" \
  RELEASE_ID="$release_id" "$ROOT_DIR/scripts/release_llm_access_cloud_opencode_only.sh" >/dev/null
grep -F 'get("revision", "")' "$test_dir/remote-command" >/dev/null
grep -F 'test "$revision" = '\''aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'\''' "$test_dir/remote-command" >/dev/null
grep -F 'sha256sum -c' "$test_dir/remote-command" >/dev/null
printf 'OpenCode release verifies the revision and binary hash without staging credentials\n'
