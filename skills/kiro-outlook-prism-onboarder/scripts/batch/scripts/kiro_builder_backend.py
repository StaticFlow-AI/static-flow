#!/usr/bin/env python3
"""Kiro macOS credential handoff. Browser steps live in kiro_builder_assisted.cjs."""
import datetime
import json
import os
import re
from pathlib import Path
import sqlite3
import subprocess
import sys
import time
import tempfile
import urllib.parse
import urllib.request

ROOT = Path(os.environ.get('KIRO_RUN_DIR', str(Path.home() / '.local/state/kiro-onboarding/kiro-builder')))
ROOT.mkdir(parents=True, exist_ok=True, mode=0o700)
DB = Path.home() / 'Library/Application Support/kiro-cli/data.sqlite3'
BASE = os.environ['KIRO_IMPORT_BASE'].rstrip('/')
KEYS = ['kirocli:odic:token', 'kirocli:odic:device-registration']
CURRENT_EMAIL = None
os.umask(0o077)

def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    path.chmod(0o600)

def progress(step, **details):
    event = dict(at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                 email=CURRENT_EMAIL, step=step, **details)
    save(ROOT / 'backend-progress.json', event)
    print('BACKEND_STAGE ' + json.dumps(event), flush=True)

def who():
    for attempt in range(3):
        try:
            result = subprocess.check_output(['kiro-cli', 'whoami'], text=True, timeout=20)
        except subprocess.CalledProcessError as error:
            result = error.output or ''
        except subprocess.TimeoutExpired:
            result = ''
        if re.search(r'^Email:[ \t]*\S+@\S+[ \t]*$', result, re.M) or 'Not logged in' in result.splitlines():
            return result
        if attempt < 2:
            time.sleep(3)
    raise RuntimeError('Kiro identity lookup omitted email; local credentials preserved')

def api(path, body=None):
    progress('api_request', method='GET' if body is None else 'POST', path=path.split('?')[0])
    key = os.environ['KIRO_ADMIN_KEY']
    # Pass headers/body through stdin, never argv. Curl is verified against this
    # endpoint; the local Python TLS transport intermittently drops handshakes.
    config = 'url = ' + json.dumps(BASE + path) + '\n'
    config += 'header = ' + json.dumps('Authorization: Bearer ' + key) + '\n'
    config += 'header = "Content-Type: application/json"\n'
    if body is not None:
        config += 'data-binary = ' + json.dumps(json.dumps(body)) + '\n'
    timeout = 35 if body is None else 120
    tunnel_port = os.environ.get('KIRO_API_TUNNEL_PORT')
    proxy = os.environ.get('KIRO_PROXY', 'http://127.0.0.1:7890')
    connect_to = []
    if tunnel_port:
        destination = urllib.parse.urlparse(BASE)
        if not tunnel_port.isdigit() or not 1 <= int(tunnel_port) <= 65535 or destination.scheme != 'https' or destination.port not in (None, 443):
            raise RuntimeError('Invalid local HTTPS tunnel configuration')
        # Connect through a caller-established SSH forward while retaining the
        # original HTTPS hostname, SNI and certificate validation.
        connect_to = ['--connect-to', f'{destination.hostname}:443:127.0.0.1:{tunnel_port}']
        proxy = ''
    command = ['curl', '--silent', '--show-error', '--connect-timeout', '10', '--max-time', str(timeout), '--proto', '=https',
               '--proxy', proxy, '--config', '-', '--write-out', '\n%{http_code} %{time_appconnect} %{size_upload}'] + connect_to
    for attempt in range(3):
        result = subprocess.run(command, input=config, capture_output=True, text=True, timeout=timeout + 5)
        payload, _, metrics = result.stdout.rpartition('\n')
        parts = metrics.split()
        status = parts[0] if parts else ''
        if not result.returncode:
            break
        # Reconnect only when TLS never completed and no request body was sent.
        # Once TLS completed, even an empty/missing response is an uncertain write.
        unsent = (len(parts) == 3 and status == '000' and
                  parts[1].replace('.', '').isdigit() and float(parts[1]) == 0 and
                  parts[2].replace('.', '').isdigit() and float(parts[2]) == 0)
        if attempt < 2 and result.returncode in (5, 6, 7, 28, 35) and unsent:
            progress('api_reconnect_before_request', attempt=attempt + 2, curlExit=result.returncode)
            time.sleep(2)
            continue
        raise RuntimeError('Prism transport failed (curl exit ' + str(result.returncode) + '); inspect before retrying a write')
    if not status.isdigit() or not 200 <= int(status) < 300:
        raise RuntimeError('Prism HTTP ' + status + '; account operation not verified')
    value = json.loads(payload)
    progress('api_response', method='GET' if body is None else 'POST', path=path.split('?')[0], status=int(status))
    return value

def lookup_account(email):
    host = os.environ.get('KIRO_LOOKUP_SSH')
    if host:
        if host != urllib.parse.urlparse(BASE).hostname:
            raise RuntimeError('SSH lookup host differs from import target')
        progress('readonly_ssh_lookup')
        script = """import sqlite3,json
email = %r
c=sqlite3.connect('file:/root/logs/request_logs.db?mode=ro',uri=True)
row=c.execute('SELECT value FROM prism_state WHERE key=?',('credentials',)).fetchone()
if not row:raise RuntimeError('Credentials state missing')
items=json.loads(row[0])
if not isinstance(items,list):raise RuntimeError('Unexpected credentials state schema')
print(json.dumps([{'id':str(x['id']),'values':{'email':x.get('email')}} for x in items if (x.get('email') or '').lower()==email.lower()]))
""" % email
        command = ['ssh', '-o', 'ConnectTimeout=12', '-o', 'StrictHostKeyChecking=yes',
                   '-o', 'ServerAliveInterval=5', '-o', 'ServerAliveCountMax=2',
                   '-o', 'ControlMaster=auto', '-o', 'ControlPersist=600',
                   '-o', 'ControlPath=' + str(Path.home() / '.ssh/qtok-kiro-%C')]
        proxy = os.environ.get('KIRO_SSH_PROXY_COMMAND')
        if proxy:
            command += ['-o', 'ProxyCommand=' + proxy]
        password_file = os.environ.get('KIRO_LOOKUP_PASSWORD_FILE')
        if password_file:
            if Path(password_file).stat().st_mode & 0o077:
                raise RuntimeError('SSH password file permissions must be private')
            command = ['sshpass', '-f', password_file] + command + ['-o', 'PubkeyAuthentication=no']
        else:
            command += ['-o', 'BatchMode=yes']
        command += ['root@' + host, 'python3 -']
        # This SELECT is safe to retry; never apply these retries to import POSTs.
        for attempt in range(3):
            progress('readonly_ssh_attempt', attempt=attempt + 1)
            try:
                result = subprocess.run(command, input=script, capture_output=True, text=True, timeout=20)
                if result.returncode == 0:
                    break
            except subprocess.TimeoutExpired:
                pass
            if attempt == 2:
                raise RuntimeError('Read-only SSH lookup failed after three bounded attempts; no import submitted')
            time.sleep(2)
        matches = json.loads(result.stdout)
        if not isinstance(matches,list) or any(x.get('values',{}).get('email','').lower()!=email.lower() for x in matches):
            raise RuntimeError('Unexpected SSH lookup identity')
        return matches
    matches, cursor, seen = [], None, set()
    while True:
        page = api('/admin/v1/accounts?limit=200' + ('&cursor=' + urllib.parse.quote(cursor, safe='') if cursor else ''))
        matches.extend(a for a in page.get('items', []) if a.get('values', {}).get('email') == email)
        cursor = page.get('nextCursor')
        if not cursor or cursor in seen or not page.get('items'):
            return matches
        seen.add(cursor)

def archived_auth(email, directory):
    directory = Path(directory)
    manifest = json.loads((directory / 'manifest.json').read_text())
    assert manifest.get('email') == email and manifest.get('status') == 'detached', 'Archive identity/status mismatch'
    auth = json.loads((directory / 'auth.json').read_text())
    for key in KEYS:
        assert key in auth, 'Archive credentials incomplete'
        file = directory / (key.replace(':', '_') + '.txt')
        if key in manifest.get('keychainServices', []):
            assert file.stat().st_mode & 0o077 == 0, 'Archive permissions are not private'
            assert json.loads(file.read_text()) == auth[key], 'Archive and keychain backup disagree'
    return auth

def import_account(email, archive=None):
    global CURRENT_EMAIL
    CURRENT_EMAIL = email
    if archive:
        auth = archived_auth(email, archive)
        progress('archived_identity_verified')
    else:
        progress('local_identity')
        assert 'Email: ' + email in who().splitlines(), 'Local identity does not match target'
        db = sqlite3.connect('file:' + str(DB) + '?mode=ro', uri=True)
        try:
            auth = {k: json.loads(v) for k, v in db.execute('SELECT key,value FROM auth_kv')}
        finally:
            db.close()
    matches = lookup_account(email)
    assert len(matches) <= 1, 'Duplicate identities require inspection'
    save(ROOT / (email + '-auth.json'), auth)
    if matches:
        cid = matches[0]['id']
        print('Existing account:', cid, flush=True)
    else:
        reg, tok = auth[KEYS[1]], auth[KEYS[0]]
        document = dict(email=email, authMethod='idc', clientId=reg['client_id'],
            clientSecret=reg['client_secret'], refreshToken=tok['refresh_token'],
            region=reg['region'], priority=int(os.environ.get('KIRO_PRIORITY', '10')))
        result = api('/admin/v1/accounts/import', {'method': 'json', 'items': [
            {'document': document, 'refreshBeforeImport': True}]})
        save(ROOT / (email + '-import.json'), result)
        assert result['items'][0]['status'] == 'succeeded', result
        cid = result['items'][0]['resourceId']
        print('Imported:', email, cid, flush=True)
    verify_import(email, str(cid))

def verify_import(email, cid):
    global CURRENT_EMAIL
    CURRENT_EMAIL = email
    verify = api('/admin/v1/accounts/' + cid + '/actions/verify', {'model': 'claude-sonnet-4-6'})
    receipt = dict(email=email, id=cid, status='unknown', health='unknown', meters=[], verification=verify)
    save(ROOT / (email + '-verified.json'), receipt)
    if verify.get('success') is True:
        try:
            account = api('/admin/v1/accounts/' + cid)
            receipt.update(status=account['status'], health=account['health'], meters=account['meters'])
        except (RuntimeError, subprocess.TimeoutExpired):
            receipt['metadataStatus'] = 'unavailable; model verification succeeded'
            progress('metadata_unavailable_after_successful_verification')
    save(ROOT / (email + '-verified.json'), receipt)
    path = ROOT / 'verified-imports.json'
    records = json.loads(path.read_text()) if path.exists() else {'node': BASE, 'accounts': []}
    records['accounts'] = [a for a in records['accounts'] if a['email'] != email] + [receipt]
    records.pop('nextEmail', None)
    save(path, records)
    print(json.dumps(receipt, ensure_ascii=False), flush=True)
    assert verify.get('success') is True, 'Real request verification failed'

def detach(email):
    assert 'Email: ' + email in who().splitlines(), 'Local identity does not match account to detach'
    records = json.loads((ROOT / 'verified-imports.json').read_text())
    receipt = next((a for a in records['accounts'] if a['email'] == email), None)
    assert receipt, 'No saved import receipt'
    if not receipt['verification'].get('success'):
        imported = json.loads((ROOT / (email + '-import.json')).read_text())
        assert any(i.get('status') == 'succeeded' and str(i.get('resourceId')) == str(receipt['id'])
                   for i in imported['items']), 'Import not confirmed'
        remote = api('/admin/v1/accounts/' + str(receipt['id']))
        assert remote.get('values', {}).get('email') == email, 'Imported identity mismatch'
    daemon = subprocess.run(['pgrep', '-f', 'kiro_cli_desktop|Kiro CLI.app'], capture_output=True)
    assert daemon.returncode == 1, 'Close the Kiro desktop daemon first'
    db = sqlite3.connect(DB)
    backup = ROOT / (email + '-before-detach.sqlite3')
    assert not backup.exists(), 'Existing backup: inspect before repeating detach'
    db.backup(sqlite3.connect(backup))
    for key in KEYS:
        found = subprocess.run(['security', 'find-generic-password', '-s', key, '-w'], capture_output=True)
        if found.returncode == 0:
            (ROOT / (email + '-' + key.replace(':', '_') + '.txt')).write_bytes(found.stdout)
            result = subprocess.run(['security', 'delete-generic-password', '-s', key], capture_output=True)
            assert result.returncode == 0, 'Could not detach keychain entry'
        elif found.returncode != 44:
            raise RuntimeError('Could not inspect keychain entry')
        db.execute('DELETE FROM auth_kv WHERE key=?', (key,))
    db.commit()
    result = subprocess.run(['kiro-cli', 'whoami'], capture_output=True, text=True)
    assert 'Not logged in' in result.stdout + result.stderr
    print('Backed up and detached locally:', email, flush=True)

def preserve_failed(email):
    """Archive an unimported identity before switching accounts; never revoke it."""
    global CURRENT_EMAIL
    CURRENT_EMAIL = email
    assert 'Email: ' + email in who().splitlines(), 'Local identity does not match failed account'
    daemon = subprocess.run(['pgrep', '-f', 'kiro_cli_desktop|Kiro CLI.app'], capture_output=True)
    assert daemon.returncode == 1, 'Close the Kiro desktop daemon first'
    pending = ROOT / 'pending-credentials'
    pending.mkdir(mode=0o700, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix='account-', dir=pending))
    manifest = dict(email=email, at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                    directory=str(directory), status='backing_up', importStatus='unconfirmed')
    db = sqlite3.connect(DB)
    try:
        original = dict(db.execute('SELECT key,value FROM auth_kv'))
        assert all(k in original for k in KEYS), 'Incomplete local credentials; preserved in place'
        backup = directory / 'data.sqlite3'
        copy = sqlite3.connect(backup)
        try:
            db.backup(copy)
            assert dict(copy.execute('SELECT key,value FROM auth_kv')) == original, 'SQLite backup mismatch'
        finally:
            copy.close()
        auth = {k: json.loads(v) for k, v in original.items()}
        save(directory / 'auth.json', auth)
        assert json.loads((directory / 'auth.json').read_text()) == auth, 'Auth backup mismatch'
        present = []
        # Finish and verify EVERY backup before deleting any credential.
        for key in KEYS:
            found = subprocess.run(['security', 'find-generic-password', '-s', key, '-w'], capture_output=True)
            if found.returncode == 0:
                file = directory / (key.replace(':', '_') + '.txt')
                file.write_bytes(found.stdout)
                assert file.read_bytes() == found.stdout, 'Keychain backup mismatch'
                present.append(key)
            elif found.returncode != 44:
                raise RuntimeError('Cannot back up keychain; credentials preserved in place')
        manifest.update(status='backed_up', keychainServices=present)
        save(directory / 'manifest.json', manifest)
        for file in directory.iterdir():
            file.chmod(0o600)
            with file.open('rb') as f:
                os.fsync(f.fileno())
        fd = os.open(directory, os.O_RDONLY)
        try:
            os.fsync(fd)
        finally:
            os.close(fd)
        progress('failed_credentials_backed_up', directory=str(directory))
        for key in present:
            result = subprocess.run(['security', 'delete-generic-password', '-s', key], capture_output=True)
            assert result.returncode == 0, 'Could not detach keychain entry; backup retained'
        for key in KEYS:
            db.execute('DELETE FROM auth_kv WHERE key=?', (key,))
        db.commit()
        result = subprocess.run(['kiro-cli', 'whoami'], capture_output=True, text=True, timeout=20)
        assert 'Not logged in' in (result.stdout + result.stderr).splitlines(), 'Local detach not confirmed; backup retained'
        manifest['status'] = 'detached'
        save(directory / 'manifest.json', manifest)
        save(ROOT / (email + '-pending.json'), manifest)
        progress('failed_credentials_preserved_and_detached', directory=str(directory))
    finally:
        db.close()

def login():
    process = subprocess.Popen(['kiro-cli', 'login', '--license', 'free', '--use-device-flow'],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    for line in process.stdout:
        if 'Logging in...' not in line and line.strip():
            print(line.strip(), flush=True)
    raise SystemExit(process.wait())

if __name__ == '__main__':
    command = sys.argv[1]
    if command == 'status':
        result = subprocess.run(['kiro-cli', 'whoami'], capture_output=True, text=True)
        print(result.stdout.strip(), flush=True)
        raise SystemExit(result.returncode)
    elif command == 'import':
        import_account(sys.argv[2])
    elif command == 'import-saved':
        import_account(sys.argv[2], sys.argv[3])
    elif command == 'detach':
        detach(sys.argv[2])
    elif command == 'preserve-failed':
        preserve_failed(sys.argv[2])
    elif command == 'verify-import':
        email, cid = sys.argv[2:4]
        matches = lookup_account(email)
        assert len(matches) == 1 and str(matches[0]['id']) == cid, 'Remote identity mismatch'
        verify_import(email, cid)
    elif command == 'login':
        login()
    else:
        raise SystemExit('Usage: backend.py import EMAIL | import-saved EMAIL ARCHIVE | verify-import EMAIL ID | detach EMAIL | preserve-failed EMAIL | login')
