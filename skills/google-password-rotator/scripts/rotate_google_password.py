#!/usr/bin/env python3
"""Rotate a Google account password in an isolated visible browser.

Secrets are accepted only through environment variables or hidden TTY prompts.
They are passed to the DevTools helper through environment variables, never
command-line arguments, and are not printed.
"""

from __future__ import annotations

import argparse
import getpass
import json
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path
from typing import Any


DEFAULT_PROXY = "http://127.0.0.1:11111"
DEFAULT_SETTINGS_URL = "https://myaccount.google.com/signinoptions/password"
NODE_DRIVER = Path(__file__).with_name("drive_google_password_change.mjs")


def log(message: str) -> None:
    print(message, flush=True)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--google-email", required=True, help="Google account email")
    parser.add_argument("--proxy", default=DEFAULT_PROXY)
    parser.add_argument("--settings-url", default=DEFAULT_SETTINGS_URL)
    parser.add_argument("--current-password-env", default="GOOGLE_CURRENT_PASSWORD")
    parser.add_argument("--new-password-env", default="GOOGLE_NEW_PASSWORD")
    parser.add_argument("--totp-secret-env", default="GOOGLE_TOTP_SECRET")
    parser.add_argument("--manual-timeout-seconds", type=int, default=900)
    parser.add_argument("--chrome-bin")
    parser.add_argument("--chrome-profile")
    parser.add_argument("--debug-port", type=int)
    parser.add_argument(
        "--attach-debug-port",
        type=int,
        help="Attach to an existing isolated Chrome DevTools port instead of launching Chrome",
    )
    parser.add_argument("--keep-browser", action="store_true")
    parser.add_argument(
        "--verification-only",
        action="store_true",
        help="Complete Google reauthentication and stop without changing the password",
    )
    parser.add_argument(
        "--skip-english-language",
        action="store_true",
        help="Do not make English (United States) the preferred Google account language after changing the password",
    )
    parser.add_argument(
        "--auto-2fa-fun",
        action="store_true",
        help="Use a TOTP secret with 2fa.fun for Google authenticator prompts",
    )
    parser.add_argument("--dry-run", action="store_true")
    return parser.parse_args(argv)


def resolve_secret(env_name: str, prompt: str) -> str:
    value = os.environ.get(env_name)
    if value:
        return value
    if sys.stdin.isatty():
        value = getpass.getpass(prompt)
        if value:
            return value
    raise SystemExit(f"{env_name} is required")


def chrome_binary(explicit: str | None) -> str:
    if explicit:
        return explicit
    for name in ("google-chrome", "chromium", "chromium-browser"):
        found = shutil.which(name)
        if found:
            return found
    raise RuntimeError("Chrome/Chromium binary not found")


def find_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def wait_http_json(url: str, timeout: float = 20.0) -> Any:
    deadline = time.monotonic() + timeout
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    while time.monotonic() < deadline:
        try:
            with opener.open(url, timeout=2) as response:
                return json.loads(response.read().decode("utf-8"))
        except Exception:
            time.sleep(0.25)
    raise RuntimeError(f"timed out waiting for {url}")


def wait_for_page_target(port: int) -> None:
    pages = wait_http_json(f"http://127.0.0.1:{port}/json/list", timeout=25)
    page = next((item for item in pages if item.get("type") == "page"), None)
    if not page:
        raise RuntimeError("Chrome DevTools page target not found")


def launch_chrome(args: argparse.Namespace) -> tuple[subprocess.Popen[Any], int, str]:
    port = args.debug_port or find_free_port()
    profile_dir = args.chrome_profile or tempfile.mkdtemp(prefix="google-password-rotator-")
    cmd = [
        chrome_binary(args.chrome_bin),
        f"--user-data-dir={profile_dir}",
        f"--proxy-server={args.proxy}",
        "--no-first-run",
        "--no-default-browser-check",
        "--disable-background-networking",
        "--disable-gpu",
        "--disable-software-rasterizer",
        "--remote-debugging-address=127.0.0.1",
        f"--remote-debugging-port={port}",
        args.settings_url,
    ]
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=args.keep_browser,
    )
    return process, port, profile_dir


def start_browser_helper(
    args: argparse.Namespace,
    port: int,
    *,
    current_password: str,
    new_password: str,
    totp_secret: str = "",
) -> subprocess.Popen[Any]:
    if not NODE_DRIVER.is_file():
        raise FileNotFoundError(f"Node DevTools driver not found: {NODE_DRIVER}")
    if not shutil.which("node"):
        raise RuntimeError("node is required for browser automation")
    env = os.environ.copy()
    env.update(
        {
            "GOOGLE_DEVTOOLS_PORT": str(port),
            "GOOGLE_EMAIL": args.google_email,
            "GOOGLE_CURRENT_PASSWORD": current_password,
            "GOOGLE_NEW_PASSWORD": new_password,
            "GOOGLE_MANUAL_TIMEOUT_SECONDS": str(args.manual_timeout_seconds),
            "GOOGLE_SETTINGS_URL": args.settings_url,
            "GOOGLE_AUTO_2FA_FUN": "1" if args.auto_2fa_fun else "0",
            "GOOGLE_TOTP_SECRET": totp_secret,
            "GOOGLE_VERIFICATION_ONLY": "1" if args.verification_only else "0",
            "GOOGLE_SET_ENGLISH_LANGUAGE": (
                "0" if args.skip_english_language or args.verification_only else "1"
            ),
        }
    )
    return subprocess.Popen(["node", str(NODE_DRIVER)], env=env)


def dry_run_summary(args: argparse.Namespace) -> dict[str, Any]:
    return {
        "dry_run": True,
        "google_email": args.google_email,
        "proxy": args.proxy,
        "settings_url": args.settings_url,
        "current_password_env": args.current_password_env,
        "new_password_env": args.new_password_env,
        "totp_secret_env": args.totp_secret_env,
        "manual_timeout_seconds": args.manual_timeout_seconds,
        "auto_2fa_fun": args.auto_2fa_fun,
        "keep_browser": args.keep_browser,
        "verification_only": args.verification_only,
        "set_english_language": (
            not args.skip_english_language and not args.verification_only
        ),
        "attach_debug_port": args.attach_debug_port,
        "driver": str(NODE_DRIVER),
    }


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    if args.dry_run:
        log(json.dumps(dry_run_summary(args), ensure_ascii=False, indent=2))
        return 0

    current_password = resolve_secret(
        args.current_password_env,
        f"Current Google password for {args.google_email}: ",
    )
    new_password = ""
    if not args.verification_only:
        new_password = resolve_secret(
            args.new_password_env,
            f"New Google password for {args.google_email}: ",
        )
        if current_password == new_password:
            raise SystemExit("new password must differ from current password")
    totp_secret = ""
    if args.auto_2fa_fun:
        totp_secret = resolve_secret(
            args.totp_secret_env,
            f"TOTP secret for {args.google_email}: ",
        )

    process: subprocess.Popen[Any] | None = None
    helper: subprocess.Popen[Any] | None = None
    profile_dir: str | None = None
    try:
        if args.attach_debug_port:
            port = args.attach_debug_port
        else:
            process, port, profile_dir = launch_chrome(args)
        wait_for_page_target(port)
        helper = start_browser_helper(
            args,
            port,
            current_password=current_password,
            new_password=new_password,
            totp_secret=totp_secret,
        )
        code = helper.wait()
        if code != 0:
            raise RuntimeError(f"browser helper failed with code {code}")
        status = (
            "verification_completed"
            if args.verification_only
            else "password_change_completed"
        )
        log(json.dumps({"status": status, "google_email": args.google_email}))
        return 0
    finally:
        if helper and helper.poll() is None:
            helper.terminate()
            try:
                helper.wait(timeout=5)
            except subprocess.TimeoutExpired:
                helper.kill()
        if process and not args.keep_browser:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
        if profile_dir and not args.keep_browser and not args.chrome_profile:
            shutil.rmtree(profile_dir, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
