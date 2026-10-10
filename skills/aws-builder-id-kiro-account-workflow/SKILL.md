---
name: aws-builder-id-kiro-account-workflow
description: "Coordinate consecutive AWS Builder ID accounts across requested password/MFA maintenance and Kiro login, llm-access import, and request verification. Use for combined account workflows or 'continue' with another Builder ID credential bundle; inherit the last selected operation and support onboarding first. Delegate individual security/login steps to the existing skills. Not AWS root/IAM or Kiro social login."
---

# AWS Builder ID and Kiro Account Workflow

Use this entrypoint to carry one account through the user's selected stages and
continue with the next account without losing the established scope or login
session. The detailed procedures remain in the linked skills; do not copy their
browser automation, import policy, or endpoint schemas into this workflow.

## Select the requested stages from conversation context

| User intent | Work to perform |
| --- | --- |
| Log into Kiro, import, or “先登录 Kiro 并上号” | Establish Kiro credentials, import/verify the account, and test a real request. Security maintenance is not part of this selection. |
| Change password/MFA and then onboard Kiro | Complete the requested security changes, retain the browser, then authorize/import/test Kiro. |
| Security maintenance only | Complete only the specified password/MFA changes; retain the same-account session when this is part of a continuing Kiro workflow. |
| “继续” with another account bundle | Carry forward the last explicitly selected live-account workflow, including its ordering and verification scope. |

A discussion about improving/creating skills does not reset the previously
selected live-account workflow. If the latest instruction says to onboard first,
finish onboarding now; do not silently append an earlier unconfirmed security
change. Ask one concise scope question only when the conversation leaves materially
different choices unresolved, and continue independent preflight while awaiting it.
Do not repeat the scope question for every account once a batch mode is established.

An `email----password----TOTP` bundle provides login material for that account;
it does not by itself select password rotation or a new password. Reuse an explicit
batch password choice when it applies, including exact case and punctuation. A
password chosen only for a previous account is not a permanent skill default.
Keep secrets out of skill files, trackers, arguments, and diagnostic output.

## Establish the account and resume point

Keep a small account-specific checkpoint in task context: verified email, selected
stages, completed evidence, owned browser/profile and debugger metadata, isolated
CLI data root, gateway account name, and pending user action. Exclude passwords,
TOTP secrets/codes, OAuth URLs, cookies, tokens, and test-key secrets from that
checkpoint. Obtain available secrets through the authorized conversation/private
input when actually needed.

1. Inspect only the target account's known CLI/browser state and gateway identity.
   Preserve another account's default Kiro login and browser. Use a separate
   account data root for fresh CLI login, never `kiro-cli logout` to switch users.
2. If `whoami` already confirms the exact Builder ID identity, reuse those CLI
   credentials. Otherwise reuse a verified same-account AWS browser for device
   authorization. Start a fresh visible browser only when no usable matching
   browser remains; reusing a password alone is not browser-session reuse.
3. Resume pending MFA/consent in the existing browser and device-flow process.
   After a user reply, inspect state before replaying a mutation. If a flow
   expired, stop its poller before starting its replacement.

## Run the selected stages

### Password or MFA maintenance, when requested

Read and follow [AWS Builder ID Security](../aws-builder-id-security/SKILL.md).
Use the current accepted password and current authenticator. Send a supplied seed
to 2fa.fun only under the user's applicable authorization; permission for an old
seed does not extend to a newly enrolled phone secret.

For phone enrollment, pause at the loaded QR page for the user, then verify the new
device before removing the specifically identified old one. Preserve completed
steps if another factor blocks the next step. Retain the authenticated browser
for Kiro continuation; clear secret variables and close the OTP tab separately.
Do not delete active AWS sessions, change recovery email, or alter unrelated MFA.

### Kiro login, import, and real request

Read and follow [Kiro AWS Builder ID Onboarder](../kiro-aws-builder-id-onboarder/SKILL.md),
which delegates SQLite parsing and new-account policy to
[Kiro Local Account Importer](../kiro-local-account-importer/SKILL.md).

Complete its chain of evidence: device authorization when needed, exact Builder ID
`whoami`, correct isolated SQLite, verified-email import or existing-account
verification, balance through the selected US proxy, and a real public request
pinned to that exact gateway account with matching usage attribution. Report the
actual subscription and dated balance. Remove only this run's temporary test key.

An authenticated AWS security page is not yet a Kiro login. A balance response is
not yet a successful model request. An already healthy gateway account should be
verified, not passed through new-account overwrite or rollback.

## Finish and continue

Update the requested account tracker using only verified results. In this repo,
use `docs/aws-builder-id-account-security-tracker.md`. Distinguish “not requested”
from “pending” and “completed”; an onboarding-only account must not be recorded as
having its password/MFA changed. Password hints apply only to accounts whose
corresponding password change was confirmed.

Report the account, selected stages completed, gateway name, actual test outcome,
and any remaining manual step. Clear temporary secret state/helpers and retain the
owned AWS browser for continuation unless cleanup is requested. On the next
account, preserve this account's credential directory and use the next account's
own identity/session. Follow local commit rules for tracker/skill edits; publishing
or deploying is a separate request.

Writing or updating this workflow alone does not authorize live account actions.
