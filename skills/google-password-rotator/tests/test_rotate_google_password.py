import importlib.util
import os
import subprocess
import sys
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "rotate_google_password.py"
SPEC = importlib.util.spec_from_file_location("rotate_google_password", SCRIPT)
module = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = module
SPEC.loader.exec_module(module)


class GooglePasswordRotatorTest(unittest.TestCase):
    def test_parse_args_uses_environment_secret_sources(self):
        args = module.parse_args(
            [
                "--google-email",
                "user@example.com",
                "--current-password-env",
                "OLD_ENV",
                "--new-password-env",
                "NEW_ENV",
                "--totp-secret-env",
                "TOTP_ENV",
            ]
        )

        self.assertEqual(args.google_email, "user@example.com")
        self.assertEqual(args.current_password_env, "OLD_ENV")
        self.assertEqual(args.new_password_env, "NEW_ENV")
        self.assertEqual(args.totp_secret_env, "TOTP_ENV")
        self.assertFalse(hasattr(args, "current_password"))
        self.assertFalse(hasattr(args, "new_password"))

    def test_resolve_secret_reads_environment(self):
        old_value = os.environ.get("GOOGLE_PASSWORD_TEST")
        os.environ["GOOGLE_PASSWORD_TEST"] = "secret-value"
        try:
            value = module.resolve_secret("GOOGLE_PASSWORD_TEST", "ignored")
        finally:
            if old_value is None:
                os.environ.pop("GOOGLE_PASSWORD_TEST", None)
            else:
                os.environ["GOOGLE_PASSWORD_TEST"] = old_value

        self.assertEqual(value, "secret-value")

    def test_parse_args_supports_verification_only_mode(self):
        args = module.parse_args(
            ["--google-email", "user@example.com", "--verification-only"]
        )

        self.assertTrue(args.verification_only)
        self.assertFalse(args.skip_english_language)

    def test_start_browser_helper_passes_secrets_only_via_environment(self):
        calls = []

        class FakeProcess:
            pass

        def fake_which(name):
            return "/usr/bin/node" if name == "node" else None

        def fake_popen(cmd, env, **kwargs):
            calls.append((cmd, env, kwargs))
            return FakeProcess()

        args = module.parse_args(
            ["--google-email", "user@example.com", "--auto-2fa-fun"]
        )
        original_which = module.shutil.which
        original_popen = module.subprocess.Popen
        module.shutil.which = fake_which
        module.subprocess.Popen = fake_popen
        try:
            process = module.start_browser_helper(
                args,
                9222,
                current_password="old-secret",
                new_password="new-secret",
                totp_secret="totp-secret",
            )
        finally:
            module.shutil.which = original_which
            module.subprocess.Popen = original_popen

        self.assertIsInstance(process, FakeProcess)
        self.assertEqual(len(calls), 1)
        cmd, env, _kwargs = calls[0]
        self.assertEqual(cmd, ["node", str(module.NODE_DRIVER)])
        self.assertNotIn("old-secret", cmd)
        self.assertNotIn("new-secret", cmd)
        self.assertNotIn("totp-secret", cmd)
        self.assertEqual(env["GOOGLE_EMAIL"], "user@example.com")
        self.assertEqual(env["GOOGLE_CURRENT_PASSWORD"], "old-secret")
        self.assertEqual(env["GOOGLE_NEW_PASSWORD"], "new-secret")
        self.assertEqual(env["GOOGLE_TOTP_SECRET"], "totp-secret")
        self.assertEqual(env["GOOGLE_AUTO_2FA_FUN"], "1")
        self.assertEqual(env["GOOGLE_VERIFICATION_ONLY"], "0")
        self.assertEqual(env["GOOGLE_SET_ENGLISH_LANGUAGE"], "1")

    def test_verification_only_preserves_preferred_language(self):
        calls = []

        class FakeProcess:
            pass

        def fake_popen(cmd, env, **kwargs):
            calls.append((cmd, env, kwargs))
            return FakeProcess()

        args = module.parse_args(
            ["--google-email", "user@example.com", "--verification-only"]
        )
        original_which = module.shutil.which
        original_popen = module.subprocess.Popen
        module.shutil.which = lambda name: "/usr/bin/node" if name == "node" else None
        module.subprocess.Popen = fake_popen
        try:
            module.start_browser_helper(
                args,
                9222,
                current_password="current-secret",
                new_password="",
            )
        finally:
            module.shutil.which = original_which
            module.subprocess.Popen = original_popen

        self.assertEqual(calls[0][1]["GOOGLE_SET_ENGLISH_LANGUAGE"], "0")

    def test_dry_run_does_not_require_secrets_or_launch_browser(self):
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
        self.assertIn("user@example.com", result.stdout)
        self.assertIn('"set_english_language": true', result.stdout)


if __name__ == "__main__":
    unittest.main()
