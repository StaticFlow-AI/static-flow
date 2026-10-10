---
name: aws-builder-id-security
description: "Manage AWS Builder ID passwords and MFA in a visible isolated browser, including authorized 2fa.fun login, user-scanned replacement authenticators, verified old-device removal, and requested account records with non-secret password hints. Use for Builder ID security maintenance, including accounts used for Kiro; not AWS root/IAM credentials or Kiro OAuth onboarding."
---

# AWS Builder ID Security

Complete the requested password or authenticator changes for the exact supplied
Builder ID accounts. Preserve completed steps across user replies: after the user
scans a QR code, continue from that browser instead of restarting account login.

## Scope and credential handling

- Act only on accounts supplied or explicitly identified for this task. “All
  accounts” does not authorize modifying unrelated accounts found in an older tracker.
- Existing authorization to change a password or replace MFA covers the relevant
  AWS form submissions and confirmations. Do not repeatedly ask permission for
  those same actions; ask when an input is missing or a different action is needed.
- Never put passwords, TOTP secrets, codes, cookies, or tokens in shell arguments,
  scripts, logs, repository documents, or skill files. Use hidden input, private
  stdin, or process environment variables, and clear them after use. Do not print
  full authentication URLs containing query parameters.
- Keep the exact user-selected password, including punctuation. If AWS rejects
  it, report the visible validation error and request a replacement. Do not
  silently capitalize, append symbols, or choose a different password.
- Changing MFA does not authorize changing the recovery email, password, or
  other settings. Password rotation does not authorize deleting all sessions.
  Kiro login/import, gateway account changes, and Git publication are separate work.

## Browser and login

1. Open an isolated, visible browser at
   [AWS Builder ID profile](https://profile.aws.amazon.com/). Use a fresh profile
   per account or reattach the known isolated profile for that same account.
   Verify the signed-in email before modifying security settings.
2. Honor an explicitly requested proxy and verify it is reachable first. Without
   a proxy requirement, use a working connection; do not assume another skill's
   historical localhost proxy port is available or mandatory for Builder ID.
3. Complete the email/password form through the visible AWS controls. Observe
   the actual destination before filling secrets: current flows use the profile
   host and regional `*.signin.aws` hosts (including SSO/credentials subdomains).
4. Handle the existing MFA challenge using the user's authenticator code, or the
   provided TOTP secret through `https://2fa.fun/` **only when the user authorized
   sending that secret there**. See [browser details](references/browser-details.md)
   for the observed selectors, fresh-code handling, and a DevTools fallback.
5. For CAPTCHA, recovery, passkey/device prompts, or other verification requiring
   the user's device, leave the browser visible and request that specific action.
   An invalid code is not a reason to replay the same code or repeat login blindly.

## Replace the authenticator

1. In **Security**, read the existing MFA inventory before adding anything.
   Retain the exact old device name and registration date in task context;
   ordering and names such as `MFA 1` / `MFA 2` are not universal identifiers.
2. Choose **Register device → Authenticator app → Next**. AWS may require
   reauthentication or a recovery email before enrollment. Complete available
   authentication; if a recovery address must be supplied, ask for it rather
   than choosing one from unrelated accounts.
3. Clarify how the user wants to enroll the new authenticator when it is not
   already specified. For phone enrollment, display the QR code, verify it has
   loaded, bring the browser forward, and pause for the user to scan it and
   complete **Assign MFA**. Do not print, capture into repository artifacts, or
   send the new secret to 2fa.fun as a side effect of using that site for login.
4. After the user confirms enrollment, inspect the AWS inventory again. Require
   a newly registered device consistent with this enrollment before deleting
   anything. If the replacement cannot be identified uniquely, stop and clarify.
5. When old-device removal is within the requested scope, select the exact old
   row and verify its name/date, then confirm **Delete MFA device?**. Do not choose
   a row merely by position. If the old row is already absent, do not delete
   another device to compensate.
6. Require AWS's deletion-success message and a resulting inventory that retains
   the new device and excludes the old one. Preserve any unrelated authenticators.

When the user explicitly wants to stop at the QR page, that is the handoff point:
retain the browser and report what the user must do next. The old device remains
until new enrollment is confirmed. Never report replacement complete at QR display.

## Change the password

1. Choose **Security → Change password**. AWS may request the current password
   and MFA again even when the profile page is already authenticated.
2. If the old authenticator was removed, use the newly enrolled one. When the
   user holds its secret, request manual browser entry or a fresh code; do not
   retry the removed secret.
3. Enter the user-selected password into both new-password fields. Use current
   AWS validation as the authority. Consult
   [AWS password troubleshooting](https://docs.aws.amazon.com/signin/latest/userguide/troubleshooting-builder-id-issues.html)
   if the reason is unclear; do not assume a general password-policy statement
   proves that a particular candidate was accepted.
4. Submit **Change password** once and inspect the result. DOM fields matching,
   a click returning successfully, or HTTP 200 alone are insufficient. If the
   form persists without a result, follow the interaction diagnostics in
   [browser details](references/browser-details.md) before considering a retry.
5. Require AWS's success confirmation, such as **Your password has been
   successfully updated.** Verify any new MFA device still remains. If no
   separate sign-in was performed, report the successful password update without
   claiming a fresh full login was tested.

Only mark a step complete after its corresponding evidence. On ambiguous
submission outcomes, inspect the current state before another mutation; on an
explicit validation failure, fix the stated input with the user first. Keep an
already-completed MFA change complete if password rotation is blocked.

## Account records and handoff

When the user asks for repository records, follow its existing tracker convention.
In StaticFlow, use `docs/aws-builder-id-account-security-tracker.md` unless the
user names a different destination. Keep a separate row for every processed
account, with the date, password result, MFA result, and any pending step. Update
only facts verified during this task; preserve earlier records and unrelated work.

Password hints are optional and require a user request. Write a conceptual cue
the user can recognize, such as a service-name association and a privately known
number segment. Do not record the actual number, a reversible encoding, or enough
fragments to reconstruct the full password. Never turn this run's password,
account, punctuation pattern, or authenticator secret into a skill default.

Before committing a tracker, review the exact diff for credential leakage and
check its formatting. Follow repository-local commit rules; do not infer push
authorization from a request to record accounts.

The final report identifies the processed account(s), confirmed changes, any
remaining manual step, and the tracker path when written. Clear temporary secret
variables and close the 2fa.fun tab separately from the AWS session.

For accounts being maintained for Kiro, retain the same-account AWS window and
profile after security changes so the next authorized step can reuse them. Do not
require the user to announce Kiro onboarding before preserving this continuity.
Keep the verified email, profile path, browser PID, and discovered debugger port
in task context, without credentials or authentication URLs. Releasing a browser
controller must not terminate that browser. Report that the window was retained.

When Kiro onboarding is requested, continue with the
[Builder ID onboarder](../kiro-aws-builder-id-onboarder/SKILL.md); preserving a
window alone does not authorize login/import. If the user requests browser cleanup,
or the task is an unrelated standalone security change with no retained handoff,
close only the task's browser and remove its temporary profile after it exits.
Do not terminate unrelated Chrome sessions or delete AWS's active-session list.

Official reference: [Manage Builder ID MFA](https://docs.aws.amazon.com/signin/latest/userguide/mfa-builder-id.html).
