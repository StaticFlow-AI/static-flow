---
name: kiro-aws-builder-id-onboarder
description: "Onboard Kiro through AWS Builder ID by reusing an existing AWS browser session or Kiro CLI login, or signing in again, then import into llm-access and verify a real account-pinned request. Use for Builder ID Kiro login, account import, or connectivity verification; not GitHub/Google social login, organization IAM Identity Center setup, or password/MFA changes."
---

# Kiro AWS Builder ID Onboarder

Complete Builder ID authentication, Kiro authorization, llm-access import, and
the requested connectivity test for the exact user-supplied account. Preserve completed
steps across turns. A logged-in AWS profile page is not yet a Kiro credential.

## Choose between the two login modes

Inspect available state before asking for credentials. The user may already have
authorized reuse of an earlier login or supplied the current password in this
conversation.

| Mode | Observed state | Action |
| --- | --- | --- |
| Reuse an existing login | Kiro CLI is logged in with Builder ID as the exact requested email, in a known data directory | Validate that identity and use its SQLite credentials; skip browser login and device authorization. |
| Reuse an existing login | An authorized AWS browser session belongs to that email, but Kiro credentials are absent | Start a Kiro device flow and open its authorization URL in that same browser. Complete Kiro consent; AWS may still require password/MFA reauthentication. |
| Sign in again | No usable matching AWS session or Kiro credentials, or the user explicitly requests a fresh login | Use an isolated Kiro data directory and visible browser, enter the current credentials, complete any MFA, and authorize Kiro. |

An unrelated default CLI account, a browser title, a matching quota, or a credential
that cannot refresh does not qualify for reuse. Ordinary access-token expiry alone
does not require a fresh login. Keep another account's login intact. If a reused
session reaches a login challenge, continue that pending flow with the missing
authentication step instead of starting a second one. After Builder ID security
maintenance, inspect the retained same-account browser before launching Chrome.
Reusing a password and reusing an authenticated browser session are different:
if that browser was closed or cleared, say so briefly before fresh login.

Read [login and browser details](references/login.md) for either mode. Read
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
  process-local variables. Normal CLI SQLite/browser credential storage is expected;
  restrict the new account directory and files to the current user.
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
2. Follow the selected login mode. Require `kiro-cli whoami` in that exact data
   environment to show **Builder ID** and the requested email. Inspect credential
   key names and metadata without printing values.
3. Read the sibling [Kiro local account importer](../kiro-local-account-importer/SKILL.md).
   Builder ID uses the IDC/OIDC token plus device-registration records and maps
   to `auth_method=idc`, `provider=aws`. Preserve the verified email on import.
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

- If the user completes MFA or consent, inspect the same browser and CLI process
  first. Do not restart a successful login or reimport an already-created account.
- If device authorization expires, stop its pending poller and start one new flow
  in the same isolated environment. If the outcome of a gateway write is unclear,
  read the exact account before any retry.
- Stop on an unresolved identity mismatch, missing manual MFA, unsupported region,
  explicit account restriction, or exhausted eligible proxies. State the specific
  remaining step and retain completed results; do not keep trying login methods
  or proxies against an explicit restriction.
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
