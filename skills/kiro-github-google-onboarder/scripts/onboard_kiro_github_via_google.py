#!/usr/bin/env python3
"""Run the canonical Kiro GitHub flow with GitHub sign-in through Google."""

from __future__ import annotations

import argparse
import getpass
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


SKILL_DIR = Path(__file__).resolve().parents[1]
BASE_SCRIPT = (
    SKILL_DIR.parent
    / "kiro-social-onboarder/scripts/onboard_kiro_social_github.py"
)
BROWSER_DRIVER = SKILL_DIR / "scripts/drive_kiro_github_via_google.mjs"
DEFAULT_PROXY = "http://127.0.0.1:11111"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--google-email", required=True)
    parser.add_argument("--account-name")
    parser.add_argument("--proxy", default=DEFAULT_PROXY)
    parser.add_argument("--admin-base-url", default="http://127.0.0.1:19182")
    parser.add_argument("--google-password-env", default="KIRO_GOOGLE_PASSWORD")
    parser.add_argument(
        "--google-totp-secret-env", default="KIRO_GOOGLE_TOTP_SECRET"
    )
    parser.add_argument(
        "--github-totp-secret-env", default="KIRO_GITHUB_TOTP_SECRET"
    )
    parser.add_argument("--manual-timeout-seconds", type=int, default=900)
    parser.add_argument("--token-poll-timeout-seconds", type=int, default=900)
    parser.add_argument("--expect-usage-limit", type=float, default=1000.0)
    parser.add_argument("--replace-account", action="store_true")
    parser.add_argument("--no-expect-student", action="store_true")
    parser.add_argument("--keep-browser", action="store_true")
    parser.add_argument(
        "--chrome-profile",
        help="Reuse an isolated Chrome profile containing prior provider sessions",
    )
    parser.add_argument(
        "--attach-debug-port",
        type=int,
        help="Reuse an already authenticated isolated Chrome session",
    )
    parser.add_argument("--dry-run", action="store_true")
    return parser.parse_args(argv)


def default_account_name(email: str) -> str:
    localpart = email.partition("@")[0].strip().lower()
    if not localpart:
        raise SystemExit("Google email must contain a non-empty local part")
    safe = "".join(character if character.isalnum() else "-" for character in localpart)
    return f"kiro-{safe.strip('-')}-github-social"


def resolve_secret(env_name: str, prompt: str) -> str:
    value = os.environ.get(env_name)
    if value:
        return value
    if sys.stdin.isatty():
        value = getpass.getpass(prompt)
        if value:
            return value
    raise SystemExit(f"{env_name} is required")


def build_base_command(args: argparse.Namespace, account_name: str) -> list[str]:
    command = [
        sys.executable,
        str(BASE_SCRIPT),
        "--account-name",
        account_name,
        "--proxy",
        args.proxy,
        "--admin-base-url",
        args.admin_base_url,
        "--manual-timeout-seconds",
        str(args.manual_timeout_seconds),
        "--token-poll-timeout-seconds",
        str(args.token_poll_timeout_seconds),
        "--expect-usage-limit",
        str(args.expect_usage_limit),
        "--github-via-google",
        "--browser-driver",
        str(BROWSER_DRIVER),
    ]
    if args.replace_account:
        command.append("--replace-account")
    if args.no_expect_student:
        command.append("--no-expect-student")
    if args.keep_browser:
        command.append("--keep-browser")
    if args.chrome_profile:
        command.extend(["--chrome-profile", args.chrome_profile])
    if args.attach_debug_port:
        command.extend(["--attach-debug-port", str(args.attach_debug_port)])
    return command


def dry_run_summary(args: argparse.Namespace, account_name: str) -> dict[str, Any]:
    return {
        "dry_run": True,
        "google_email": args.google_email,
        "account_name": account_name,
        "proxy": args.proxy,
        "google_password_env": args.google_password_env,
        "google_totp_secret_env": args.google_totp_secret_env,
        "github_totp_secret_env": args.github_totp_secret_env,
        "browser_driver": str(BROWSER_DRIVER),
        "base_script": str(BASE_SCRIPT),
        "replace_account": args.replace_account,
        "keep_browser": args.keep_browser,
        "chrome_profile": args.chrome_profile,
        "attach_debug_port": args.attach_debug_port,
    }


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    account_name = args.account_name or default_account_name(args.google_email)
    if args.dry_run:
        print(json.dumps(dry_run_summary(args, account_name), ensure_ascii=False, indent=2))
        return 0

    google_password = resolve_secret(
        args.google_password_env, f"Google password for {args.google_email}: "
    )
    google_totp_secret = resolve_secret(
        args.google_totp_secret_env,
        f"Google TOTP secret for {args.google_email}: ",
    )
    github_totp_secret = resolve_secret(
        args.github_totp_secret_env,
        f"GitHub TOTP secret for {args.google_email}: ",
    )

    if not BASE_SCRIPT.is_file() or not BROWSER_DRIVER.is_file():
        raise RuntimeError("Kiro onboarding script or browser driver is missing")

    env = os.environ.copy()
    env.update(
        {
            "KIRO_GOOGLE_EMAIL": args.google_email,
            "KIRO_GOOGLE_PASSWORD": google_password,
            "KIRO_GOOGLE_TOTP_SECRET": google_totp_secret,
            "KIRO_GITHUB_TOTP_SECRET": github_totp_secret,
        }
    )
    env.setdefault("KIRO_STEP_DELAY_MIN_MS", "1000")
    env.setdefault("KIRO_STEP_DELAY_MAX_MS", "2000")
    result = subprocess.run(build_base_command(args, account_name), env=env, check=False)
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
