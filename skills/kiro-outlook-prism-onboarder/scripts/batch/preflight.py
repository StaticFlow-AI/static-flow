import json, os, sys, urllib.parse
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent / 'scripts'))
import kiro_builder_backend as backend
root = Path(os.environ['BATCH_RUN_DIR'])
accounts = json.loads((root / 'accounts.json').read_text())
assert isinstance(accounts, list) and accounts
matches = {a['email']: [] for a in accounts}
assert len(matches) == len(accounts), 'Duplicate input emails'
cursor = None
seen = set()
while True:
    page = backend.api('/admin/v1/accounts?limit=200' + ('&cursor=' + urllib.parse.quote(cursor, safe='') if cursor else ''))
    for item in page.get('items', []):
        email = item.get('values', {}).get('email')
        if email in matches:
            matches[email].append({'id': item['id'], 'email': email})
    cursor = page.get('nextCursor')
    if not cursor:
        break
    assert cursor not in seen, 'Repeated cursor'
    seen.add(cursor)
backend.save(root / 'preflight.json', {'node': backend.BASE, 'accounts': matches})
assert not any(matches.values()), 'Remote duplicates found; inspect before proceeding'
print('Preflight passed:', len(accounts), 'accounts')
