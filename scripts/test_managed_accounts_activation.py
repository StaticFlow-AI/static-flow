#!/usr/bin/env python3
"""Failure-path tests for the coordinated schema cutover."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("activation", Path(__file__).with_name("activate_llm_access_managed_accounts.py"))
activation = importlib.util.module_from_spec(spec)
spec.loader.exec_module(activation)


class CutoverTests(unittest.TestCase):
    def exercise(self, mismatch):
        events = []
        funcs = "CREATE OR REPLACE FUNCTION old_a();\nCREATE OR REPLACE FUNCTION old_b();"
        def sql(query):
            events.append(("sql", query))
            if "max(version)" in query:
                return "90"
            if "pg_get_functiondef" in query:
                return funcs
            return ""
        def run(*args, **kwargs):
            events.append(("run", args))
            if "sha256sum" in args:
                return activation.hashlib.sha256(b"fixture").hexdigest() + " executable"
            return ""
        with tempfile.TemporaryDirectory() as directory:
            stage = Path(directory)
            for name in activation.SERVICES:
                (stage / name).write_bytes(b"fixture")
            (stage / "manifest.json").write_text(json.dumps({"child_revision":"fixture", "binaries":{name:activation.hashlib.sha256(b"fixture").hexdigest() for name in activation.SERVICES}}))
            (stage / "0091_managed_accounts.sql").write_text("SELECT 'migration';")
            states = {"MainPID":"123", "NRestarts":"0", "ActiveState":"active"}
            with patch.object(activation, "sql", sql), patch.object(activation, "run", run), patch.object(activation, "state", return_value=states), patch.object(activation,"healthy",return_value=True), patch.object(activation,"fingerprint",side_effect=[{"rows":4},{"rows":5 if mismatch else 4}]):
                if mismatch:
                    with self.assertRaisesRegex(RuntimeError,"fingerprint"):
                        activation.activate(stage)
                else:
                    activation.activate(stage)
                    self.assertEqual(json.loads((stage/"activation.json").read_text())["child_revision"],"fixture")
        return events

    def test_success_commits_before_new_binaries_start(self):
        events = self.exercise(False)
        migration = next(i for i,event in enumerate(events) if event[0]=="sql" and "INSERT INTO llm_access_schema_migrations" in event[1])
        start = next(i for i,event in enumerate(events) if event[0]=="run" and "start" in event[1])
        self.assertLess(migration,start)
        self.assertFalse(any(event[0]=="sql" and "RENAME TO llm_cursor_accounts" in event[1] for event in events))

    def test_failed_validation_restores_schema_before_old_services_start(self):
        events = self.exercise(True)
        rollback = next(i for i,event in enumerate(events) if event[0]=="sql" and "RENAME TO llm_cursor_accounts" in event[1])
        starts = [i for i,event in enumerate(events) if event[0]=="run" and "start" in event[1]]
        self.assertEqual(len(starts),1)
        self.assertLess(rollback,starts[0])
        self.assertIn("CREATE OR REPLACE FUNCTION old_a",events[rollback][1])
        self.assertFalse(any(event[0]=="run" and any(str(arg).endswith(".managed-new") for arg in event[1]) for event in events))


if __name__ == "__main__":
    unittest.main()
