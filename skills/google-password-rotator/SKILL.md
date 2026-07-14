---
name: google-password-rotator
description: Use when a Google account password needs to be changed or rotated through a visible isolated browser, including Google sign-in, current-password reauthentication, preferred-language setup, and optional app-code verification through 2fa.fun. Applies when Codex should prefill credentials while the user handles CAPTCHA, passkeys, device prompts, recovery checks, or other unusual verification.
---

# Google Password Rotator

## Boundaries

- Never pass passwords or TOTP secrets on the command line.
- Never print, store, commit, or summarize passwords, TOTP secrets, generated codes, cookies, or session tokens.
- Use an isolated Chrome profile and the requested HTTP proxy for the whole flow. Default proxy: `http://127.0.0.1:11111`.
- Treat CAPTCHA, passkeys, device prompts, recovery checks, and unusual verification as manual user steps.
- Use `--auto-2fa-fun` only when the user explicitly supplies an authenticator secret.
- Do not claim success unless the helper reports `password_change_completed` or the user confirms success in the visible browser.

## Standard Flow

Use hidden TTY prompts for all secrets:

```bash
python3 skills/google-password-rotator/scripts/rotate_google_password.py \
  --google-email user@example.com \
  --proxy http://127.0.0.1:11111 \
  --auto-2fa-fun \
  --manual-timeout-seconds 900
```

The helper:

1. Opens Google password settings in an isolated Chrome profile through the proxy.
2. Fills the Google email and current password.
3. When Google requests an authenticator code and `--auto-2fa-fun` is enabled, opens or reuses `https://2fa.fun/`, submits the secret, reads only `input.faotp.value`, and submits the generated code without printing it.
4. Waits for manual completion of any unsupported verification.
5. Fills the new password and confirmation fields.
6. After Google confirms the password change, makes English (United States) the preferred account language and verifies the saved state.
7. Reports completion only after both the password change and preferred-language update succeed.
8. Removes the isolated Chrome profile unless `--keep-browser` is set.

## Options

- `--google-email EMAIL`: required Google account email.
- `--current-password-env NAME`: defaults to `GOOGLE_CURRENT_PASSWORD`.
- `--new-password-env NAME`: defaults to `GOOGLE_NEW_PASSWORD`.
- `--totp-secret-env NAME`: defaults to `GOOGLE_TOTP_SECRET`.
- `--proxy URL`: browser proxy; defaults to `http://127.0.0.1:11111`.
- `--settings-url URL`: override the Google password settings URL.
- `--manual-timeout-seconds 900`: manual verification timeout.
- `--auto-2fa-fun`: automatically handle Google authenticator-code prompts through 2fa.fun.
- `--keep-browser`: keep the isolated browser open after the helper exits.
- `--attach-debug-port PORT`: resume an existing isolated Chrome session without replaying completed login steps.
- `--verification-only`: complete reauthentication and stop on the requested Google settings page without changing the password.
- `--skip-english-language`: preserve the current preferred language instead of making English (United States) preferred after a password change. Verification-only mode always preserves the language.
- `--dry-run`: print a redacted execution plan without reading secrets or opening Chrome.

## Failure Handling

- If Google reports a wrong password, invalid code, disabled account, or recovery requirement, stop automatic retries and leave the visible page for user inspection.
- If 2fa.fun is used, read generated codes only from `input.faotp.value`; never scrape arbitrary text or the secret field.
- If Google changes the DOM, rerun with `--keep-browser`, inspect visible labels/selectors, and patch `drive_google_password_change.mjs`.
- If the new-password form remains visible after submission, do not assume success.

## Verification

Run safe checks before live use:

```bash
python3 skills/google-password-rotator/scripts/rotate_google_password.py \
  --google-email user@example.com \
  --dry-run

python3 -m py_compile \
  skills/google-password-rotator/scripts/rotate_google_password.py

node --check \
  skills/google-password-rotator/scripts/drive_google_password_change.mjs
```
