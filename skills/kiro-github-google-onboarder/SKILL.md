---
name: kiro-github-google-onboarder
description: Onboard or refresh a Kiro social account when Kiro uses GitHub as its provider but GitHub itself must be entered through Continue with Google. Automates Google credentials, Google authenticator verification through 2fa.fun, GitHub authenticator verification through 2fa.fun, Kiro device approval, llm-access import, proxy assignment, and KIRO STUDENT balance verification.
---

# Kiro GitHub via Google Onboarder

## Boundaries

- Never print, store, or pass passwords, TOTP secrets, generated codes, OAuth tokens, or cookies on the command line.
- Use hidden prompts or environment variables only. Clear caller-side secret variables after completion.
- Use one HTTP proxy for Chrome, Kiro OAuth, `kiro-cli whoami`, and balance verification.
- Stop automatic retries on CAPTCHA, recovery, device-trust, or unusual-verification pages and leave the visible browser for manual handling.
- Never call `kiro-cli logout`; reuse the canonical cleanup/import flow from `kiro-social-onboarder`.
- Do not report success until `kiro-cli whoami`, llm-access import, and the expected Kiro balance all succeed.

## Standard Flow

Run the bundled wrapper. Supply all three secrets through hidden prompts:

```bash
python3 skills/kiro-github-google-onboarder/scripts/onboard_kiro_github_via_google.py \
  --google-email user@example.com \
  --account-name kiro-user-github-social \
  --proxy http://127.0.0.1:11111
```

The wrapper:

1. Resolves the Google password, Google TOTP secret, and GitHub TOTP secret without command-line exposure.
2. Starts the canonical Kiro GitHub device flow with `loginProvider: "Github"`.
3. Selects GitHub's Google sign-in path, submits Google credentials, and handles Google authenticator verification through 2fa.fun.
4. Handles GitHub authenticator verification through 2fa.fun, then approves GitHub and Kiro authorization.
5. Reuses `kiro-social-onboarder` for SQLite backup/cleanup, token persistence, `kiro-cli whoami`, llm-access import, proxy assignment, balance refresh, and KIRO STUDENT/1000-credit verification.

## Options

- `--google-email EMAIL`: required Google email.
- `--account-name NAME`: optional; defaults to `kiro-<email-localpart>-github-social`.
- `--proxy URL`: defaults to `http://127.0.0.1:11111`.
- `--google-password-env NAME`: defaults to `KIRO_GOOGLE_PASSWORD`.
- `--google-totp-secret-env NAME`: defaults to `KIRO_GOOGLE_TOTP_SECRET`.
- `--github-totp-secret-env NAME`: defaults to `KIRO_GITHUB_TOTP_SECRET`.
- `--manual-timeout-seconds 900`: browser challenge timeout.
- `--replace-account`: replace only the exact requested llm-access account name.
- `--keep-browser`: keep the isolated browser open after the flow.
- `--chrome-profile PATH`: reopen a retained isolated profile when a device code expires after provider login.
- `--attach-debug-port PORT`: reuse an already authenticated isolated Chrome session with a fresh Kiro device code.
- `--dry-run`: print a redacted plan without reading secrets or changing state.

The wrapper defaults browser action delays to a random 1–2 seconds. Override with `KIRO_STEP_DELAY_MIN_MS` and `KIRO_STEP_DELAY_MAX_MS`.

## Failure Handling

- If Google or GitHub rejects a code, do not resubmit the same code repeatedly. Wait for a fresh code or complete the step manually.
- If Google requests a familiar device, recovery method, CAPTCHA, passkey, or phone prompt, stop browser automation on that page.
- If GitHub reports suspension or account recovery, stop before Kiro approval.
- If OAuth succeeds but import fails, query the exact target account before rerunning; do not create a second account name.
- Preserve any manually disabled llm-access account state unless the failure is a refreshable 401.

## Verification

```bash
python3 -m py_compile \
  skills/kiro-github-google-onboarder/scripts/onboard_kiro_github_via_google.py

node --check \
  skills/kiro-github-google-onboarder/scripts/drive_kiro_github_via_google.mjs

python3 -m unittest \
  skills/kiro-github-google-onboarder/tests/test_onboard_kiro_github_via_google.py

node --test \
  skills/kiro-github-google-onboarder/tests/drive_kiro_github_via_google.test.mjs
```
