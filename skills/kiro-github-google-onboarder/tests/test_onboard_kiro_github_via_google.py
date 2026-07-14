import importlib.util
import os
import subprocess
import sys
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/onboard_kiro_github_via_google.py"
SPEC = importlib.util.spec_from_file_location("onboard_kiro_github_via_google", SCRIPT)
module = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = module
SPEC.loader.exec_module(module)


class KiroGithubGoogleOnboarderTest(unittest.TestCase):
    def test_default_account_name_uses_email_localpart(self):
        self.assertEqual(
            module.default_account_name("User.Name@example.com"),
            "kiro-user-name-github-social",
        )

    def test_base_command_contains_no_secrets(self):
        args = module.parse_args(
            [
                "--google-email",
                "user@example.com",
                "--proxy",
                "http://proxy",
                "--chrome-profile",
                "/tmp/retained-profile",
                "--attach-debug-port",
                "47569",
            ]
        )
        command = module.build_base_command(args, "kiro-user-github-social")

        self.assertIn("--github-via-google", command)
        self.assertIn(str(module.BROWSER_DRIVER), command)
        self.assertNotIn("password-secret", command)
        self.assertNotIn("totp-secret", command)
        self.assertIn("/tmp/retained-profile", command)
        self.assertIn("47569", command)

    def test_resolve_secret_reads_environment(self):
        old_value = os.environ.get("KIRO_CHAIN_TEST_SECRET")
        os.environ["KIRO_CHAIN_TEST_SECRET"] = "secret-value"
        try:
            self.assertEqual(
                module.resolve_secret("KIRO_CHAIN_TEST_SECRET", "ignored"),
                "secret-value",
            )
        finally:
            if old_value is None:
                os.environ.pop("KIRO_CHAIN_TEST_SECRET", None)
            else:
                os.environ["KIRO_CHAIN_TEST_SECRET"] = old_value

    def test_dry_run_needs_no_secrets(self):
        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "--google-email",
                "user@example.com",
                "--dry-run",
            ],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn('"dry_run": true', result.stdout)
        self.assertIn("kiro-user-github-social", result.stdout)


if __name__ == "__main__":
    unittest.main()
