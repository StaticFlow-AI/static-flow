#!/usr/bin/env python3
"""One-time schema 91 -> 93 cutover, with Antigravity started last.

Run only from a verified release containing five binaries and both migrations.
Before Antigravity is started, rollback can restore the prior provider identity.
After it starts, keep new usage readers running: old workers cannot decode new
Antigravity events. A failure at that point requires a forward repair.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import time
import urllib.request

from activate_llm_access_managed_accounts import HEALTH, SERVICES, run, sql, state

TABLES = ("llm_keys", "llm_account_groups", "llm_proxy_bindings",
          "llm_proxy_config_endpoint_checks", "llm_account_model_usage_rollups",
          "llm_model_token_prices")
MIGRATIONS = ((92, "antigravity_control_plane"), (93, "antigravity_model_prices"))


def literal(value):
    return "'" + json.dumps(value).replace("'", "''") + "'::jsonb"


def snapshot():
    return json.loads(sql("""SELECT json_build_object(
      'keys', (SELECT COALESCE(json_agg(json_build_object('id',key_id,'provider',provider_type)), '[]') FROM llm_keys),
      'groups', (SELECT COALESCE(json_agg(json_build_object('id',group_id,'provider',provider_type)), '[]') FROM llm_account_groups),
      'prices', (SELECT COALESCE(json_agg(json_build_object('provider',provider_type,'model',model)), '[]') FROM llm_model_token_prices),
      'ag_proxy', (SELECT count(*) FROM llm_proxy_bindings WHERE provider_type='antigravity'),
      'key_invariant', (SELECT md5(string_agg(key_id || ':' || secret || ':' || key_hash || ':' || quota_billable_limit::text, ',' ORDER BY key_id)) FROM llm_keys),
      'usage_invariant', (SELECT md5(string_agg(key_id || ':' || billable_tokens::text, ',' ORDER BY key_id)) FROM llm_key_usage_rollups),
      'account_invariant', (SELECT md5(string_agg(account_id || ':' || auth_json::text, ',' ORDER BY account_id)) FROM llm_managed_accounts)
    )::text;"""))


def rollback_sql(before):
    parts = ["BEGIN; SET LOCAL lock_timeout='15s';"]
    for table, field, key in [("llm_keys", "key_id", "keys"), ("llm_account_groups", "group_id", "groups")]:
        parts.append(f"UPDATE {table} t SET provider_type=b.value->>'provider' FROM jsonb_array_elements({literal(before[key])}) b(value) WHERE t.{field}=b.value->>'id';")
    parts.append("UPDATE llm_account_model_usage_rollups SET provider_type='cursor' WHERE provider_type='antigravity';")
    parts.append(f"DELETE FROM llm_model_token_prices p WHERE NOT EXISTS (SELECT 1 FROM jsonb_array_elements({literal(before['prices'])}) b(value) WHERE p.provider_type=b.value->>'provider' AND p.model=b.value->>'model');")
    if before["ag_proxy"] == 0:
        parts.append("DELETE FROM llm_proxy_bindings WHERE provider_type='antigravity';")
    for table in TABLES:
        parts.append(f"ALTER TABLE {table} DROP CONSTRAINT {table}_provider_type_check; ALTER TABLE {table} ADD CONSTRAINT {table}_provider_type_check CHECK (provider_type IN ('codex','kiro','cursor'));")
    parts.append("DELETE FROM llm_access_schema_migrations WHERE version IN (92,93); COMMIT;")
    return "\n".join(parts)


def wait_healthy(names):
    deadline = time.monotonic() + 60
    while True:
        healthy = True
        for name in names:
            if state(name)["ActiveState"] != "active":
                healthy = False
                break
            try:
                with urllib.request.urlopen(HEALTH[name], timeout=3) as response:
                    healthy = response.status == 200
            except (OSError, ValueError):
                healthy = False
            if not healthy:
                break
        if healthy:
            return
        if time.monotonic() >= deadline:
            raise RuntimeError("services did not become healthy: " + ", ".join(names))
        time.sleep(1)


def activate(stage):
    manifest = json.loads((stage / "manifest.json").read_text())
    for name in SERVICES:
        if hashlib.sha256((stage / name).read_bytes()).hexdigest() != manifest["binaries"][name]:
            raise RuntimeError("staged digest mismatch: " + name)
    for filename, digest in manifest["migrations"].items():
        if hashlib.sha256((stage / filename).read_bytes()).hexdigest() != digest:
            raise RuntimeError("staged migration digest mismatch: " + filename)
    if sql("SELECT max(version) FROM llm_access_schema_migrations;") != "91":
        raise RuntimeError("cutover requires schema version 91")
    # A rollback to three providers is only valid before the namespace exists.
    if sql("SELECT count(*) FROM llm_keys WHERE provider_type='antigravity';") != "0":
        raise RuntimeError("Antigravity keys already exist")
    backup = stage / "rollback"
    backup.mkdir(mode=0o700)
    before_services = {name: state(name) for name in SERVICES}
    caddy_before = state("caddy")
    for name in SERVICES:
        run("sudo", "cp", "-a", "/usr/local/bin/" + name, str(backup / name))
    ag_started = False
    before = None
    try:
        run("sudo", "systemctl", "stop", *SERVICES)
        # Old failed account-rollup batches still carry the Cursor namespace.
        # Drain them with the old service before changing historical ownership.
        backlog = Path("/var/lib/staticflow/llm-access/usage-journal/llm-access-antigravity/control-rollups")
        for directory in (backlog / "sealed", backlog / "consuming"):
            if directory.exists() and any(directory.iterdir()):
                raise RuntimeError("Antigravity control rollups must drain before migration")
        before = snapshot()
        (backup / "control-before.json").write_text(json.dumps(before))
        transaction = """BEGIN; SET LOCAL lock_timeout='15s';
        LOCK TABLE llm_access_schema_migrations IN EXCLUSIVE MODE;
        DO $$ BEGIN
          IF (SELECT max(version) FROM llm_access_schema_migrations) <> 91 THEN
            RAISE EXCEPTION 'cutover requires schema version 91';
          END IF;
        END $$;
        """
        for version, name in MIGRATIONS:
            transaction += (stage / f"{version:04}_{name}.sql").read_text()
            transaction += f"\nINSERT INTO llm_access_schema_migrations(version,name,applied_at_ms) VALUES({version},'{name}',(extract(epoch from clock_timestamp())*1000)::bigint);\n"
        sql(transaction + "COMMIT;")
        after = snapshot()
        for key in ("key_invariant", "usage_invariant", "account_invariant"):
            if before[key] != after[key]:
                raise RuntimeError("migration changed " + key)
        for name in SERVICES:
            target = "/usr/local/bin/" + name
            run("sudo", "install", "-m", "0755", str(stage / name), target + ".ag-admin-new")
            run("sudo", "mv", "-f", target + ".ag-admin-new", target)
        readers = tuple(name for name in SERVICES if name != "llm-access-antigravity")
        run("sudo", "systemctl", "start", *readers)
        wait_healthy(readers)
        # From here onward new usage may have provider_type=antigravity.
        ag_started = True
        run("sudo", "systemctl", "start", "llm-access-antigravity")
        wait_healthy(("llm-access-antigravity",))
        after_services = {name: state(name) for name in SERVICES}
        for name, status in after_services.items():
            if status["NRestarts"] != "0":
                raise RuntimeError("automatic restart: " + name)
            digest = run("sudo", "sha256sum", "/proc/" + status["MainPID"] + "/exe").split()[0]
            if digest != manifest["binaries"][name]:
                raise RuntimeError("running digest differs: " + name)
        if state("caddy") != caddy_before:
            raise RuntimeError("Caddy state changed")
        result = {"child_revision": manifest["child_revision"], "schema_version": 93,
                  "services_before": before_services, "services_after": after_services,
                  "binaries": manifest["binaries"], "preserved_key_credentials_and_usage": True,
                  "caddy_unchanged": True}
        (stage / "activation.json").write_text(json.dumps(result, indent=2))
        print(json.dumps(result))
    except Exception:
        if ag_started:
            (stage / "forward-repair-required").write_text("Keep updated readers: Antigravity usage may exist.\n")
            raise
        run("sudo", "systemctl", "stop", *SERVICES)
        if before is not None and sql("SELECT max(version) FROM llm_access_schema_migrations;") == "93":
            sql(rollback_sql(before))
        for name in SERVICES:
            target = "/usr/local/bin/" + name
            run("sudo", "cp", "-a", str(backup / name), target + ".rollback")
            run("sudo", "mv", "-f", target + ".rollback", target)
        run("sudo", "systemctl", "start", *SERVICES)
        wait_healthy(SERVICES)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    activate(args.stage.resolve())
