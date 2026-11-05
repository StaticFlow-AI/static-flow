import os
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parent.parent

with tempfile.TemporaryDirectory(prefix='kiro-review-release-mock-') as tmp:
    tmp = pathlib.Path(tmp)
    bin_dir = tmp / 'bin'
    bin_dir.mkdir()
    log = tmp / 'commands.log'
    child = tmp / 'llm-access'
    migrations = child / 'crates/llm-access-migrations/migrations/postgres'
    migrations.mkdir(parents=True)
    (child / 'Cargo.toml').touch()
    for name in ('0099_cursor_key_cache_rate.sql', '0100_key_reasoning_effort.sql',
                 '0102_managed_cache_default.sql'):
        (migrations / name).touch()
    (bin_dir / 'cargo').write_text('''#!/usr/bin/env bash
set -eu
printf 'cargo %s\\n' "$*" >> "$MOCK_LOG"
if [[ "$1" == build ]]; then
  mkdir -p "$CARGO_TARGET_DIR/release"
  for bin in llm-access llm-access-usage-worker llm-access-codex-image; do
    printf '#!/bin/sh\\nexit 0\\n' > "$CARGO_TARGET_DIR/release/$bin"
    chmod 755 "$CARGO_TARGET_DIR/release/$bin"
  done
fi
''')
    (bin_dir / 'ssh').write_text('''#!/usr/bin/env bash
set -eu
printf 'ssh %s\\n' "$*" >> "$MOCK_LOG"
''')
    (bin_dir / 'scp').write_text('''#!/usr/bin/env bash
set -eu
printf 'scp %s\\n' "$*" >> "$MOCK_LOG"
for arg in "$@"; do
  if [[ "$arg" = /* && "$arg" != "$MOCK_KEY" ]]; then
    test -f "$arg"
  fi
done
''')
    (bin_dir / 'psql').write_text('''#!/usr/bin/env bash
set -eu
printf 'psql invoked\\n' >> "$MOCK_LOG"
''')
    (bin_dir / 'git').write_text('''#!/usr/bin/env bash
set -eu
if [[ "$*" == *"rev-parse --short=12 HEAD"* ]]; then
  printf '123456789abc\\n'
elif [[ "$*" == *"rev-parse HEAD"* ]]; then
  printf '123456789abcdef0123456789abcdef012345678\\n'
fi
''')
    for executable in bin_dir.iterdir():
        executable.chmod(0o755)
    key = tmp / 'not-a-key'
    key.touch()
    neon = tmp / 'placeholder.env'
    neon.write_text('LLM_ACCESS_CONTROL_DATABASE_URL=placeholder\n')
    config = tmp / 'release.env'
    config.write_text(
        f'CARGO_TARGET_DIR={tmp}/target\n'
        f'GCP_SSH_KEY={key}\n'
        'GCP_DEST=mock@host\n'
        f'REMOTE_RELEASE_DIR={tmp}/remote\n'
        'BUILD_JOBS=4\n'
    )
    (root / 'tmp').mkdir(exist_ok=True)
    for preserve in ('1', '0'):
        output = tmp / f'output-{preserve}'
        log.write_text('')
        env = os.environ.copy()
        env.update({
            'PATH': str(bin_dir) + os.pathsep + env['PATH'],
            'MOCK_LOG': str(log), 'MOCK_KEY': str(key),
            'LLM_ACCESS_CLOUD_RELEASE_CONFIG': str(config),
            'LLM_ACCESS_LOCAL_NEON_ENV_FILE': str(neon),
            'LLM_ACCESS_DIR': str(child),
            'LLM_ACCESS_RELEASE_OUT': str(output),
            'LLM_ACCESS_PRESERVE_RUNTIME_ENV': preserve,
            'RELEASE_ID': f'mock-{preserve}',
            'ALLOW_DIRTY': '1',
        })
        for script in ('prepare_llm_access_cloud_release.sh', 'release_llm_access_cloud_api_only.sh'):
            subprocess.run(['bash', '-n', str(root / 'scripts' / script)], check=True)
        result = subprocess.run(['bash', str(root / 'scripts/release_llm_access_cloud_api_only.sh')],
                                env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if result.returncode:
            raise RuntimeError(f'mocked release {preserve} failed:\n{result.stdout}')
        entries = list(output.iterdir())
        staged = {path.name for path in entries}
        lines = log.read_text().splitlines()
        uploads = [line for line in lines if line.startswith('scp ')]
        activation = [line for line in lines if line.startswith('ssh ')][-1]
        assert len(uploads) == 2
        if preserve == '1':
            assert any(name.startswith('llm-access.mock-') for name in staged)
            assert not any(name.startswith(('llm-access-neon.env.', 'llm-access-usage-worker.',
                                            'llm-access-codex-image.')) for name in staged), staged
            assert not any('neon.env' in line or 'usage-worker.' in line or
                           'codex-image.' in line for line in uploads)
            assert 'LLM_ACCESS_STAGED_NEON_ENV=/dev/null/no-staged-env' in activation
            assert '--bin llm-access' in next(line for line in lines if line.startswith('cargo build'))
        else:
            assert any(name.startswith('llm-access-neon.env.') for name in staged)
            assert any(name.startswith('llm-access-usage-worker.') for name in staged)
            assert any(name.startswith('llm-access-codex-image.') for name in staged)
            assert 'LLM_ACCESS_STAGED_NEON_ENV=' not in activation
        print(f'mocked API release preserve_runtime_env={preserve}: passed')
