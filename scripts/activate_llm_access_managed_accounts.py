#!/usr/bin/env python3
"""One-time coordinated migration 91; run on AWS against a verified release directory.

The directory contains five binaries, manifest.json, and 0091_managed_accounts.sql.
All other services and runtime configuration are preserved. Rollback restores the
old schema names/functions before any old binary can start again.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
import urllib.request
from urllib.parse import urlsplit, unquote, parse_qsl

SERVICES = ("llm-access", "llm-access-usage-worker", "llm-access-cursor",
            "llm-access-antigravity", "llm-access-oauth")
HEALTH = {"llm-access": "http://127.0.0.1:19080/healthz",
          "llm-access-usage-worker": "http://127.0.0.1:19081/admin/llm-access/usage-worker/status",
          "llm-access-cursor": "http://127.0.0.1:19090/healthz",
          "llm-access-antigravity": "http://127.0.0.1:19095/healthz",
          "llm-access-oauth": "http://127.0.0.1:19194/"}


def run(*args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs).strip()


def sql(statement):
    # Credentials stay in the child environment, never argv or output.
    connection = run("sudo", "bash", "-c", 'set -a; source /mnt/llm-access/config/neon.env; printf "%s" "$LLM_ACCESS_CONTROL_DATABASE_URL"')
    url = urlsplit(connection)
    if url.scheme not in ("postgres", "postgresql") or not url.hostname or not url.path.lstrip("/"):
        raise RuntimeError("invalid control database URL")
    env = os.environ.copy()
    env.update(PGHOST=url.hostname or "", PGPORT=str(url.port or 5432),
               PGUSER=unquote(url.username or ""), PGPASSWORD=unquote(url.password or ""),
               PGDATABASE=unquote(url.path.lstrip("/")))
    for key, value in parse_qsl(url.query):
        variable = {"sslmode":"PGSSLMODE", "channel_binding":"PGCHANNELBINDING", "options":"PGOPTIONS", "connect_timeout":"PGCONNECT_TIMEOUT"}.get(key)
        if not variable:
            raise RuntimeError("unsupported database URL option: " + key)
        env[variable] = value
    result = subprocess.run(["psql", "-X", "-Atq", "-v", "ON_ERROR_STOP=1"],
                            input=statement, text=True, capture_output=True, env=env)
    if result.returncode:
        raise RuntimeError("Postgres operation failed: " + result.stderr[:2000])
    return result.stdout.strip()


def state(service):
    keys = ("MainPID", "NRestarts", "ActiveState", "ExecMainStartTimestamp")
    return dict(line.split("=", 1) for line in run(
        "systemctl", "show", *("--property=" + key for key in keys), service).splitlines())


def fingerprint(table, binding):
    return json.loads(sql(f"""SELECT json_build_object(
        'rows',(SELECT count(*) FROM {table}),
        'providers',(SELECT json_object_agg(upstream_provider,n) FROM
           (SELECT upstream_provider,count(*) n FROM {table} GROUP BY 1) p),
        'credentials',(SELECT md5(string_agg(account_id || ':' || auth_json::text,',' ORDER BY account_id)) FROM {table}),
        'bindings',(SELECT md5(string_agg(id::text || ':' || {binding},',' ORDER BY id))
           FROM llm_oauth_sessions WHERE {binding} IS NOT NULL),
        'outgoing_foreign_keys',(SELECT count(*) FROM pg_constraint WHERE contype='f' AND conrelid='{table}'::regclass),
        'foreign_keys',(SELECT count(*) FROM pg_constraint WHERE contype='f' AND confrelid='{table}'::regclass)
        )::text;"""))


def healthy():
    for service, url in HEALTH.items():
        if state(service)["ActiveState"] != "active":
            return False
        try:
            with urllib.request.urlopen(url, timeout=3) as response:
                if response.status != 200:
                    return False
        except (OSError, ValueError):
            return False
    return True


def activate(stage):
    manifest = json.loads((stage / "manifest.json").read_text())
    for name in SERVICES:
        if hashlib.sha256((stage / name).read_bytes()).hexdigest() != manifest["binaries"][name]:
            raise RuntimeError("staged binary digest mismatch: " + name)
    if sql("SELECT max(version) FROM llm_access_schema_migrations;") != "90":
        raise RuntimeError("one-time cutover requires schema version 90")
    # Save the exact old functions, including changes since their original migration.
    functions = sql("SELECT pg_get_functiondef(oid) || ';' FROM pg_proc WHERE proname IN "
                    "('llm_register_account_oauth','llm_capture_account_oauth') AND pronamespace='public'::regnamespace;")
    if functions.count("CREATE OR REPLACE FUNCTION") != 2:
        raise RuntimeError("expected two OAuth functions")
    backup = stage / "rollback"
    backup.mkdir(mode=0o700)
    (backup / "oauth-functions.sql").write_text(functions)
    before_services = {name: state(name) for name in SERVICES}
    for name in SERVICES:
        run("sudo", "cp", "-a", "/usr/local/bin/" + name, str(backup / name))
    migrated = False
    try:
        run("sudo", "systemctl", "stop", *SERVICES)
        before = fingerprint("llm_cursor_accounts", "cursor_account_name")
        (backup / "before.json").write_text(json.dumps(before))
        migration = (stage / "0091_managed_accounts.sql").read_text()
        sql("BEGIN; SET LOCAL lock_timeout='15s'; "
            "LOCK TABLE llm_access_schema_migrations IN EXCLUSIVE MODE; " + migration +
            "\nINSERT INTO llm_access_schema_migrations(version,name,applied_at_ms) "
            "VALUES(91,'managed_accounts',(extract(epoch from clock_timestamp())*1000)::bigint); COMMIT;")
        migrated = True
        after = fingerprint("llm_managed_accounts", "managed_account_name")
        if before != after:
            raise RuntimeError("post-migration fingerprint mismatch")
        for name in SERVICES:
            # Never overwrite an executable inode used by another process.
            target = "/usr/local/bin/" + name
            run("sudo", "install", "-m", "0755", str(stage / name), target + ".managed-new")
            run("sudo", "mv", "-f", target + ".managed-new", target)
        run("sudo", "systemctl", "start", *SERVICES)
        deadline = time.monotonic() + 120
        while not healthy():
            if time.monotonic() >= deadline:
                raise RuntimeError("new services did not become healthy")
            time.sleep(2)
        after_services = {name: state(name) for name in SERVICES}
        for name, status in after_services.items():
            if status["NRestarts"] != "0":
                raise RuntimeError("automatic restart during cutover: " + name)
            digest = run("sudo", "sha256sum", "/proc/" + status["MainPID"] + "/exe").split()[0]
            if digest != manifest["binaries"][name]:
                raise RuntimeError("running executable differs from release: " + name)
        result = {"child_revision": manifest["child_revision"], "fingerprint": after,
                  "services_before": before_services, "services_after": after_services,
                  "binaries": manifest["binaries"]}
        (stage / "activation.json").write_text(json.dumps(result, indent=2))
        print(json.dumps(result))
    except Exception:
        run("sudo", "systemctl", "stop", *SERVICES)
        # Check the database even if the network dropped after COMMIT.
        migrated = migrated or sql("SELECT to_regclass('public.llm_managed_accounts') IS NOT NULL;") == "t"
        if migrated:
            sql("BEGIN; DROP INDEX idx_llm_managed_accounts_provider; ALTER TABLE llm_managed_accounts RENAME TO llm_cursor_accounts; "
                "ALTER TABLE llm_oauth_sessions RENAME COLUMN managed_account_name TO cursor_account_name; "
                + functions + "\nDELETE FROM llm_access_schema_migrations WHERE version=91; COMMIT;")
        for name in SERVICES:
            target = "/usr/local/bin/" + name
            run("sudo", "cp", "-a", str(backup / name), target + ".rollback")
            run("sudo", "mv", "-f", target + ".rollback", target)
        run("sudo", "systemctl", "start", *SERVICES)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    activate(args.stage.resolve())
