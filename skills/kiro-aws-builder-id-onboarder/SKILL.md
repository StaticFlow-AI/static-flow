---
name: kiro-aws-builder-id-onboarder
description: "Onboard Kiro through AWS Builder ID, preferring IDE browser authorization for fresh logins, then import into llm-access and verify an account-pinned request. Also supports existing IDE or CLI credentials. Not social login, organization IAM Identity Center setup, or password/MFA changes."
---

# Kiro AWS Builder ID Onboarder

Complete Builder ID authentication, Kiro authorization, llm-access import, and
the requested connectivity test for the exact user-supplied account. Preserve completed
steps across turns. A logged-in AWS profile page is not yet a Kiro credential.

## Choose the login mode

Inspect available state before asking for credentials. The user may already have
authorized reuse of an earlier login or supplied the current password in this
conversation. Prefer **Kiro IDE Builder ID authorization with PKCE** for new or
explicitly renewed logins. Reuse a healthy existing account when no mode change
is requested. Keep CLI device login available for an explicit CLI choice; do not
silently switch to it when IDE login requires user intervention.

| Mode | Observed state | Action |
| --- | --- | --- |
| Reuse credentials | A verified same-account IDE token and its matching client registration, or an already healthy gateway account, are available | Validate identity and credential provenance; skip another login unless renewal or IDE conversion was requested. |
| Reuse CLI credentials | Kiro CLI confirms the exact Builder ID email and no IDE conversion is requested | Use that isolated SQLite; skip browser login. |
| Reuse the AWS browser | The retained AWS session belongs to that email, but IDE credentials are needed | Run IDE portal and Builder ID PKCE authorization in that same browser. |
| Sign in again | No usable matching session, or a fresh login was requested | Use a visible isolated browser and account-specific IDE credential directory; complete password/MFA and IDE authorization. |

An unrelated default CLI account, a browser title, a matching quota, or a credential
that cannot refresh does not qualify for reuse. Ordinary access-token expiry alone
does not require a fresh login. Keep another account's login intact. If a reused
session reaches a login challenge, continue that pending flow with the missing
authentication step instead of starting a second one. After Builder ID security
maintenance, inspect the retained same-account browser before launching Chrome.
Reusing a password and reusing an authenticated browser session are different:
if that browser was closed or cleared, say so briefly before fresh login.

Read [IDE login](references/ide-login.md) for the default fresh-login path and
[CLI login](references/login.md) only for CLI reuse or an explicit CLI flow. Read
[import and request verification](references/import-and-verify.md) once the exact
Kiro identity is established.

## Boundaries

- User authorization to log in, import, and test covers ordinary device consent,
  the specified account write, and a small account-pinned test. Do not ask again
  for those same actions. Ask only for missing input, ambiguous identity, or a
  genuinely different action. A login-only request stops before gateway import.
- Use credentials from the current authorized conversation or a verified session;
  do not search unrelated account records for passwords. After a password change,
  use the final accepted password. After MFA replacement, use the current device.
- Keep passwords, TOTP seeds/codes, OAuth URLs containing codes, tokens, cookies,
  client secrets, proxy passwords, and test-key secrets out of tool output,
  command-line arguments, scripts, trackers, and this skill. Use private stdin or
  process-local variables. Normal IDE cache, CLI SQLite, and browser storage are
  expected; restrict the new account directory and files to the current user.
- Never run `kiro-cli logout` as account-switch preparation. Prefer an isolated
  `XDG_DATA_HOME`; do not replace `HOME` or clear another account's metadata.
- Honor a user-selected login proxy. Otherwise use a working connection for AWS
  login; do not inherit an unavailable historical localhost proxy from a social
  skill. Gateway accounts still follow the importer's active-US-proxy policy.
- Do not change passwords, MFA devices, recovery email, existing keys/groups,
  other accounts, or network topology as a side effect of onboarding.
- Only delete a failed import that this run demonstrably created. An existing
  account must never enter the new-account importer's delete-on-failure path.

## Shared completion path

1. Confirm the requested email, actual browser/CLI identity, selected data root,
   and intended gateway account. In StaticFlow, the mapped admin API normally is
   `http://127.0.0.1:19182`; the operator console is the React app on `:19191`.
   Verify reachability before gateway writes.
2. Follow the selected login mode. For IDE, verify the same-account browser,
   Builder ID portal result, callback state, successful token exchange, and matching
   client registration. For CLI, require `kiro-cli whoami` in that exact data
   environment to show **Builder ID** and the requested email. Inspect metadata
   without printing credentials; a prior CLI login does not prove IDE authorization.
3. Read the sibling [Kiro local account importer](../kiro-local-account-importer/SKILL.md).
   Builder ID uses an IDC/OIDC token plus client registration. IDE cache records
   use `authMethod=IdC`, `provider=BuilderId`; import as `auth_method=idc` while
   preserving `BuilderId`. CLI records may use the equivalent provider `aws`.
   Preserve the verified email on import. The SQLite importer is for CLI files;
   parse IDE JSON locally as described in the IDE reference.
   Do not switch to Google/GitHub or require `auth_method=social`.
4. Resolve an existing account by exact identity before writing. For a new account,
   a reasonable name is `kiro-<email-localpart>-builder-id`, normalized for the API;
   use the user's explicit name when provided. Paginate account inventory for
   duplicate detection and proxy balancing; a large `limit` does not bypass caps.
5. Review a credential-redacted import plan, apply it, and refresh balance through
   the selected proxy. Require the configured minimum remaining credits and
   unique upstream identity. Report the actual subscription; do not assume every
   Builder ID account is `KIRO STUDENT` or has 1000 credits. Enforce a requested
   subscription requirement before declaring that part complete.
6. When asked to make requests work, test the real gateway path with a key pinned
   to this account. Require generated content, successful completion, and account
   attribution. A health check, balance response, or CLI login alone is insufficient.
   Clean up only the temporary test key created for this run.

## Resume and finish

- If the user completes MFA or consent, inspect the same browser and pending
  IDE callback listener or CLI process first. Do not restart a successful login
  or reimport an already-created account.
- If device authorization expires, stop its pending poller and start one new flow
  in the same isolated environment. If the outcome of a gateway write is unclear,
  read the exact account before any retry.
- Stop on an unresolved identity mismatch, missing manual MFA, unsupported region,
  explicit account restriction, or exhausted eligible proxies. State the specific
  remaining step and retain completed results; do not keep trying login methods
  or proxies against an explicit restriction.
- If the user requests intervention on errors, pause after the first failed login
  or validation, retain completed steps, and wait before another proxy, login
  method, or account. Honor an authorized request to provide that account's current
  password and authenticator information through the private conversation; keep
  those secrets out of trackers and diagnostic output.
- For a requested account tracker, record email, gateway name, auth type,
  subscription/dated balance, proxy, and actual test result. StaticFlow's existing
  Builder ID tracker is `docs/aws-builder-id-account-security-tracker.md`. Credentials
  stay out of the tracker. Follow local commit rules; do not infer push authority.
- Report the account, import status, actual model/result, and any unresolved step.
  Clear in-memory secrets and temporary helpers; preserve another account's CLI
  state. For this continuing Builder ID/Kiro workflow, retain the authenticated
  browser/profile unless the user requests cleanup. Detach the controller and
  retain only non-secret reconnection metadata in task context.

Creating or validating this skill is not an instruction to repeat live login,
account import, password changes, or billable model requests.
