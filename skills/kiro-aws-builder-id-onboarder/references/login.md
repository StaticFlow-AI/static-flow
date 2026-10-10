# Builder ID login: reuse or sign in again

Use this reference for the browser/CLI portion. Confirm commands with the installed
`kiro-cli login --help`; the observed CLI supports
`login --license free --use-device-flow`. Builder ID is the personal `free` login
method even when the upstream account has a paid or student subscription.

## Identify the existing state

Inspect only the default CLI data location and account-specific locations identified
by the current task. For a candidate data root, run:

```bash
XDG_DATA_HOME="$kiro_data_root" kiro-cli whoami
```

The expected result is `Logged in with Builder ID` and the exact requested email.
For an existing default login, the normal SQLite path is
`~/.local/share/kiro-cli/data.sqlite3`; do not assume an isolated login changed it.
Every subsequent CLI command and SQLite import must use the same chosen root.

Supported Builder ID keys in `auth_kv` are:

- `kirocli:odic:token` or `kirocli:oidc:token`;
- the corresponding `kirocli:odic:device-registration` or
  `kirocli:oidc:device-registration`.

The token supplies access/refresh tokens, expiration, and region; device
registration supplies `client_id` and `client_secret`. The optional profile is
under `state['api.codewhisperer.profile']`. Print key names and field presence,
not raw values. A social token is not a Builder ID credential; the sibling
importer currently prefers social records when both are present. Resolve that
conflict in an isolated account directory rather than importing the wrong record
or deleting unrelated default auth state.

If the matching Kiro login is usable, continue directly to import/verification.
Let normal token refresh handle access-token expiry; do not restart browser login
merely because an access token expired. Use the fresh-login path if refresh or
identity verification actually requires it.

Use the browser metadata retained by the preceding security task before choosing
a fresh profile. Probe that specific debugger and verify the same account. A
missing `/tmp` controller script does not mean Chrome or its session is gone;
reattach to the live browser with available tooling. If the old browser/profile
was actually removed, explain that credentials can be reused but the browser
session cannot. Ask only for a factor that the resulting live page requires.

## Mode 1: reuse the matching AWS browser

Reuse an available visible browser session identified by the user or this task
after establishing that it belongs to the requested email. Prefer the retained
isolated browser from earlier work; do not assume an unrelated default profile
is authorized. With CDP, discover the current DevTools port and page target from
the task's browser state; no port, PID, profile path, target ID, or `/tmp` helper
from a previous run is a permanent dependency. If that browser cannot be automated,
hand off device authorization there to the user, or use the authorized credentials
in a new isolated flow when the task permits fresh login.

If only AWS is logged in, obtain Kiro credentials as follows:

1. Select or create the account's isolated CLI data root. Start the device flow
   below and retain that one process while browser authorization is pending.
2. Open the CLI-generated authorization URL in the verified AWS browser. Transfer
   the URL in memory; redact its query/fragment and device code from progress logs.
   For a fresh login, start here as well: logging into the profile page separately
   can add another reauthentication before device authorization.
3. Reinspect the page after redirects. A profile session may remember the email
   but still ask for the password or current MFA. Supply what is already authorized
   and available; request only the missing factor.
4. Match the displayed device code against the code from this exact CLI process,
   then choose **Confirm and continue**. Verify the application is **Kiro CLI**
   before choosing **Allow access** for its chat/code-analysis/suggestion scopes.
5. Require CLI authorization success, then rerun `whoami` in the same data root.

Do not treat the AWS security page, password success notification, device-code
display, or an **Allow access** click alone as completed Kiro login.

When driving `kiro-cli` from a controller, retain its one pending device-flow
process across user replies. Capture PTY output privately, store the authorization
URL and code separately in memory, and emit only state transitions; spinner output
must not bury the URL in a truncated buffer. Disable echo on the secret-input PTY.
Use a bounded manual wait consistent with the device flow's expiry. An expired
flow requires stopping that poller before starting a replacement in the same root.

AWS may show a blank intermediate page after MFA or a button click. Wait for the
expected visible page, within a finite timeout, before comparing the device code
or deciding that a click failed. Reacquire the DOM/context after redirects. Check
both CLI authorization success and `whoami`, then retain the browser separately
from clearing the controller's secrets.

## Mode 2: sign in again

Use an account-specific root, for example
`~/.local/share/kiro-builder-accounts/<account-slug>`. Create a new root with mode
`0700` and keep its credential database owner-readable/writable only. Do not chmod
a shared parent directory or point a new account at another account's existing root.

```bash
XDG_DATA_HOME="$kiro_data_root" kiro-cli login --license free --use-device-flow
```

First verify `whoami` in a newly selected root says not logged in; this proves that
the installed CLI respects isolation. If it unexpectedly shows another account,
resolve the data-location issue before login. Do not use logout to work around it.

Open the generated URL in a fresh visible browser profile, or reuse an already
verified same-account profile that has simply expired. With Chrome/CDP, bind the
debugger to loopback, use a dynamic port, and discover `DevToolsActivePort`. Use
finite command timeouts and reacquire the page after navigation.

Fill the user's current email/password only on observed AWS sign-in destinations
such as regional `*.signin.aws` hosts. After navigation through `view.awsapps.com`,
check the actual destination before secret entry. Do not pass secrets via shell
arguments or print input values/full authentication URLs.

- If a password was rotated earlier, use the final accepted password rather than
  an earlier rejected candidate or the original password.
- If the user now owns the authenticator, bring the MFA page forward and request
  manual entry or one fresh code. Keep the current flow pending for the response.
- If an active TOTP seed is supplied and the user explicitly authorizes 2fa.fun,
  use that seed there, transfer a fresh code privately, and close the OTP tab.
  A removed seed and a previous one-time code cannot be reused. Permission to
  process an old seed does not cover a newly enrolled phone authenticator.
- Pause for CAPTCHA, device approval, recovery, or another factor requiring the
  user's device. Do not change security settings to get past the challenge.

After sign-in, finish the same device-code confirmation and Kiro consent steps as
Mode 1. Capture only success/identity metadata. If a synthetic button click does
nothing, inspect validation and page state before another submission; a native
pointer click may be needed for AWS's controlled forms. Never blindly repeat an
ambiguous authorization or gateway mutation.

Official reference: [Kiro authentication](https://kiro.dev/docs/cli/authentication/).
