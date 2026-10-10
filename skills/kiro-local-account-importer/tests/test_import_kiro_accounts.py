import importlib.util
import io
import json
import os
import sqlite3
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "import_kiro_accounts.py"
SPEC = importlib.util.spec_from_file_location("import_kiro_accounts", SCRIPT)
module = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = module
SPEC.loader.exec_module(module)


class ImportKiroAccountsTest(unittest.TestCase):
    def test_us_proxy_filter_keeps_only_us_nodes(self):
        proxies = [
            {"id": "sg", "name": "aws_sg1"},
            {"id": "aws-us", "name": "aws_us3_east1"},
            {"id": "do-us", "name": "do-us-1"},
            {"id": "home-us", "name": "us-home1"},
            {"id": "homeus", "name": "my-homeus2"},
            {"id": "dmit", "name": "dmit-us"},
            {"id": "ae", "name": "azure-ae"},
            {"id": "sg-proxy", "name": "proxy_to_sg"},
        ]

        filtered = module.filter_required_region_proxies(proxies, "us")

        self.assertEqual(
            [proxy["name"] for proxy in filtered],
            ["aws_us3_east1", "do-us-1", "us-home1", "my-homeus2", "dmit-us"],
        )

    def test_default_minimum_remaining_credits_is_ten(self):
        args = module.parse_args([])

        self.assertEqual(args.minimum_remaining_credits, 10.0)

    def test_balance_must_refresh_with_minimum_remaining(self):
        self.assertTrue(
            module.balance_has_minimum_remaining({"remaining": 10.0}, minimum_remaining=10.0)
        )
        self.assertFalse(
            module.balance_has_minimum_remaining({"remaining": 9.99}, minimum_remaining=10.0)
        )
        self.assertFalse(module.balance_has_minimum_remaining({}, minimum_remaining=10.0))

    def test_failed_validated_import_is_deleted(self):
        calls = []

        def fake_request_json(method, base_url, path, token, body=None, timeout=30.0):
            calls.append((method, path, body))
            if method == "GET" and path.startswith("/admin/kiro-gateway/accounts?"):
                return {"accounts": [], "has_more": False}
            if method == "POST" and path == "/admin/kiro-gateway/accounts/import-auth":
                return {"name": "kiro-bad"}
            if method == "PATCH":
                return {"name": "kiro-bad"}
            if method == "POST" and path == "/admin/kiro-gateway/accounts/kiro-bad/balance":
                return {"remaining": 5.0, "usage_limit": 100.0, "current_usage": 95.0}
            if method == "DELETE":
                return {"status": "ok"}
            raise AssertionError((method, path, body))

        original = module.request_json
        module.request_json = fake_request_json
        try:
            args = module.parse_args(["--apply"])
            auth = module.ImportedAuth(
                name="kiro-bad",
                sqlite_path=Path("/tmp/kiro.sqlite3"),
                body={"name": "kiro-bad", "minimum_remaining_credits_before_block": 10.0},
            )
            result = module.import_account(
                auth,
                args,
                proxies=[{"id": "us-proxy-1", "name": "do-us-1"}],
                min_interval_ms=337,
            )
        finally:
            module.request_json = original

        self.assertFalse(result["validated"])
        self.assertTrue(result["deleted"])
        self.assertIn(("DELETE", "/admin/kiro-gateway/accounts/kiro-bad", None), calls)

    def test_duplicate_user_id_import_is_deleted(self):
        calls = []

        def fake_request_json(method, base_url, path, token, body=None, timeout=30.0):
            calls.append((method, path, body))
            if method == "GET" and path.startswith("/admin/kiro-gateway/accounts?"):
                return {"accounts": [], "has_more": False}
            if method == "POST" and path == "/admin/kiro-gateway/accounts/import-auth":
                return {"name": "kiro-duplicate"}
            if method == "PATCH":
                return {"name": "kiro-duplicate"}
            if method == "POST" and path == "/admin/kiro-gateway/accounts/kiro-duplicate/balance":
                return {
                    "remaining": 1000.0,
                    "usage_limit": 1000.0,
                    "current_usage": 0.0,
                    "user_id": "upstream-1",
                }
            if method == "DELETE":
                return {"status": "ok"}
            raise AssertionError((method, path, body))

        original = module.request_json
        module.request_json = fake_request_json
        try:
            args = module.parse_args(["--apply"])
            auth = module.ImportedAuth(
                name="kiro-duplicate",
                sqlite_path=Path("/tmp/kiro.sqlite3"),
                body={"name": "kiro-duplicate", "minimum_remaining_credits_before_block": 10.0},
            )
            result = module.import_account(
                auth,
                args,
                proxies=[{"id": "us-proxy-1", "name": "do-us-1"}],
                min_interval_ms=337,
                existing_user_ids={"upstream-1": "kiro-existing"},
            )
        finally:
            module.request_json = original

        self.assertFalse(result["validated"])
        self.assertTrue(result["deleted"])
        self.assertEqual(result["duplicate_of"], "kiro-existing")
        self.assertEqual(result["duplicate_user_id"], "upstream-1")
        self.assertIn(("DELETE", "/admin/kiro-gateway/accounts/kiro-duplicate", None), calls)

    def test_inventory_uses_returned_page_size_and_includes_later_duplicate_ids(self):
        pages = [
            {"accounts": [{"name": "first", "email": None}], "has_more": True},
            {"accounts": [{"name": "second", "upstream_user_id": "user-2"}], "has_more": False},
        ]
        with patch.object(module, "request_json", side_effect=pages) as request:
            accounts = module.fetch_kiro_accounts("http://admin", None)
        self.assertEqual([a["name"] for a in accounts], ["first", "second"])
        self.assertEqual(module.existing_user_id_map(accounts), {"user-2": "second"})
        self.assertIn("offset=1", request.call_args_list[1].args[2])

    def test_incomplete_or_repeated_inventory_stops(self):
        invalid_pages = [
            [{"accounts": []}],
            [{"accounts": [], "has_more": True}],
            [{"accounts": [{"name": "repeated"}], "has_more": True}] * 2,
        ]
        for pages in invalid_pages:
            with self.subTest(pages=pages):
                with patch.object(module, "request_json", side_effect=pages):
                    with self.assertRaises(ValueError):
                        module.fetch_kiro_accounts("http://admin", None)

    def test_existing_name_or_email_never_enters_create_or_delete_path(self):
        auth = module.ImportedAuth(
            "new-name", Path("/tmp/test.sqlite3"),
            {"name": "new-name", "email": "verified@example.test"},
        )
        for existing in [
            {"name": "new-name", "email": None},
            {"name": "other-name", "email": "VERIFIED@example.test"},
        ]:
            with self.subTest(existing=existing):
                with patch.object(module, "request_json") as request:
                    with self.assertRaisesRegex(ValueError, "existing name/email"):
                        module.import_account(
                            auth, module.parse_args(["--apply"]),
                            [{"id": "us", "name": "do-us-1"}], 300,
                            existing_accounts=[existing],
                        )
                    request.assert_not_called()

    def test_direct_import_call_checks_later_inventory_pages_before_write(self):
        pages = [
            {"accounts": [{"name": "unrelated", "email": None}], "has_more": True},
            {"accounts": [{"name": "existing"}], "has_more": False},
        ]
        auth = module.ImportedAuth("existing", Path("/tmp/test.sqlite3"), {"name": "existing"})
        with patch.object(module, "request_json", side_effect=pages) as request:
            with self.assertRaises(ValueError):
                module.import_account(auth, module.parse_args(["--apply"]), [], 300)
        self.assertTrue(all(call.args[0] == "GET" for call in request.call_args_list))

    def test_missing_emails_do_not_match_each_other(self):
        auth = module.ImportedAuth("new", Path("/tmp/test.sqlite3"), {"name": "new"})
        with patch.object(module, "request_json") as request:
            result = module.import_account(
                auth, module.parse_args([]), [], 300,
                existing_accounts=[{"name": "one", "email": None}, {"name": "two"}],
            )
        self.assertFalse(result["applied"])
        request.assert_not_called()

    def test_delete_success_requires_exact_account_absence(self):
        with patch.object(module, "request_json", return_value={}), patch.object(
            module, "fetch_kiro_accounts", return_value=[{"name": "still-present"}],
        ):
            self.assertFalse(module.delete_account(module.parse_args([]), "still-present"))

    def test_verified_email_is_rejected_for_multiple_databases(self):
        with patch.object(module, "discover_sqlite_files", return_value=[Path("a"), Path("b")]):
            with patch.object(module, "request_json") as request:
                with self.assertRaisesRegex(SystemExit, "exactly one SQLite"):
                    module.main(["--email", "verified@example.test"])
                request.assert_not_called()

    def test_admin_token_can_be_supplied_without_command_line_secret(self):
        with patch.dict(os.environ, {"STATICFLOW_ADMIN_TOKEN": "test-admin-token"}):
            self.assertEqual(module.parse_args([]).admin_token, "test-admin-token")

    def test_builder_id_sqlite_to_import_preserves_verified_email_and_balances_fixed_proxies(self):
        calls = []
        nullable_account = {"name": "unrelated", "email": None, "proxy_mode": "direct", "proxy_config_id": "p1"}
        fixed_account = {"name": "fixed", "email": "other@example.test", "proxy_mode": "fixed", "proxy_config_id": "p2"}

        def request(method, base, path, token, body=None, timeout=30.0):
            calls.append((method, path, body))
            if path == "/admin/llm-gateway/proxy-configs":
                return {"proxy_configs": [
                    {"id": "p1", "name": "do-us-1", "status": "active"},
                    {"id": "p2", "name": "do-us-2", "status": "active"},
                ]}
            if path.startswith("/admin/kiro-gateway/accounts?"):
                if "offset=0" in path:
                    return {"accounts": [nullable_account], "has_more": True}
                return {"accounts": [fixed_account], "has_more": False}
            if path.startswith("/internal/kiro-gateway/latency-ranking"):
                return {"proxies": [{"proxy_config_id": "p2", "avg_first_token_ms": 1, "first_token_samples": 10}]}
            if method == "POST" and path.endswith("/import-auth"):
                return {"name": body["name"]}
            if method == "PATCH":
                return {"name": "kiro-test"}
            if method == "POST" and path.endswith("/balance"):
                return {"remaining": 100, "user_id": "new-user", "subscription_title": "TEST PLAN"}
            raise AssertionError((method, path))

        with tempfile.TemporaryDirectory() as directory:
            database = Path(directory) / "data.sqlite3"
            with sqlite3.connect(database) as conn:
                conn.execute("CREATE TABLE auth_kv (key TEXT, value TEXT)")
                conn.executemany("INSERT INTO auth_kv VALUES (?, ?)", [
                    ("kirocli:odic:token", json.dumps({"access_token": "test-access", "refresh_token": "test-refresh", "region": "us-east-1"})),
                    ("kirocli:odic:device-registration", json.dumps({"client_id": "test-client", "client_secret": "test-client-secret", "region": "us-east-1"})),
                ])
            output = io.StringIO()
            with patch.object(module, "request_json", side_effect=request), redirect_stdout(output):
                result = module.main([
                    "--sqlite-file", str(database), "--account-name", "kiro-test",
                    "--email", "verified@example.test", "--apply",
                ])
        self.assertEqual(result, 0)
        imported = next(body for method, path, body in calls if path.endswith("/import-auth"))
        self.assertEqual((imported["auth_method"], imported["provider"], imported["email"]), ("idc", "aws", "verified@example.test"))
        self.assertEqual(imported["client_secret"], "test-client-secret")
        selected_proxy = next(body for method, path, body in calls if method == "PATCH")
        self.assertEqual(selected_proxy, {"proxy_mode": "fixed", "proxy_config_id": "p1"})
        for secret in ["test-access", "test-refresh", "test-client-secret"]:
            self.assertNotIn(secret, output.getvalue())


if __name__ == "__main__":
    unittest.main()
