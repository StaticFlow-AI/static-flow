# Builder ID browser details

Read this when browser automation tools are unavailable, when using an authorized
2fa.fun flow, or when AWS controls do not react as expected. These are observed
UI shapes; inspect the live DOM before using them. Do not depend on generated IDs,
a fixed AWS directory ID, a previous debugging port, or an old temporary script.

## Visible Chrome through DevTools

With local shell access, Chrome and modern Node.js can provide a fallback without
Playwright. Launch a dedicated profile beneath an OS temporary directory with
user-only permissions. Useful flags are:

```text
--user-data-dir=<new isolated directory>
--remote-debugging-address=127.0.0.1
--remote-debugging-port=0
--no-first-run
--no-default-browser-check
```

Read the allocated port from that profile's `DevToolsActivePort`; discover page
targets through loopback `/json/list` and connect to the selected page's
`webSocketDebuggerUrl`. Confirm the listening debugger is loopback-only. Add
`--proxy-server` only for the selected, preflighted proxy. Do not silently replace
an explicit proxy with direct access. Keep this browser visible for phone MFA.

Use `Runtime.evaluate` for focused DOM observations and `Page.bringToFront` for a
handoff. After navigation, allow the page to finish loading and rediscover its
target/context when necessary. Treat a blank intermediate document as loading,
not as successful completion. Give CDP requests finite timeouts and handle socket
closure; a disconnected controller is not evidence of a failed AWS mutation.

Keep controller code free of credentials. Supply secrets through environment
variables or private stdin (disable terminal echo for credential input to a PTY).
Do not echo commands containing secrets. Redact secrets before serializing results;
avoid input values, cookies, browser storage, raw network bodies, and URL queries
in diagnostic output. Preserve only the port/profile identifier needed to resume.

Separate controller shutdown from browser shutdown: clear process-local secrets
and detach the CDP socket while keeping Chrome and its profile for a retained
handoff. Keep the verified account and browser ownership metadata in task context;
do not put transient ports or profile paths into a permanent account tracker.

## Forms and authenticators

| Observed page | Useful DOM evidence |
| --- | --- |
| Email login | Visible `input[type=email]`, then **Continue** |
| Password reauthentication | Visible password input, then **Continue** |
| MFA challenge | **MFA code**, often `input[placeholder='Enter code']` |
| MFA method selection | Radio input whose value is `TOTP`; choose it before **Next** |
| App enrollment | Image with alt text **QR Code**, then **Authenticator code** and **Assign MFA** |
| New password | Separate **New password** and **Confirm new password** inputs |
| Device inventory | Radio control's enclosing row with device name and registration date |

For QR handoff, check the image is visible, complete, and has nonzero natural
dimensions without reading its payload. Read the inventory again after enrollment;
returning to the security page without the new device is not sufficient.

Prefer browser-tool fill and click actions. With CDP, focus the actual visible
field, select its existing content using keyboard input, and use `Input.insertText`.
For pointer clicks, move to the visible target and send a complete press/release
sequence (`buttons=1` while pressed, `buttons=0` after release). Verify the resulting
field and form state without printing values. A DOM value setter plus synthetic
`input`/`change` events is a fallback to validate against the real flow, not proof
that AWS processed the input normally.

A programmatic `element.click()` returned successfully during the observed password
flow while leaving the form unchanged. A real pointer click subsequently exposed
**Invalid password**. If this happens, inspect visible errors, input validity, and
whether a request was sent. Use `Input.dispatchMouseEvent` on the current visible
button's bounding rectangle only after establishing that a prior mutation was not
submitted. Do not repeatedly click while the outcome remains uncertain. If needed,
inspect a masked screenshot of the password form; avoid capturing QR/secret pages.

When deleting a device, scope the first **Delete** to the visible inventory action
and the second to the visible **Delete MFA device?** dialog. Hidden dialogs can
contain additional Delete buttons. Recheck the selected old row before confirmation.

## Action pacing and bounded retries

Allow 1–2 seconds between individual browser actions, including consecutive fills,
clicks, and navigation. Await each action; do not dispatch a form's entire sequence
at once. Then require the next expected control to be visible, enabled, and stable
before acting. A fixed delay alone does not establish readiness. After redirects,
reacquire the page/context; poll loading state about once a second with a finite
deadline, normally 30 seconds. Generate an OTP only after its input is ready.

For a recoverable browser step, default to three attempts total, counting the
initial submission, across the controller and its caller. Wait about 5 seconds
before the first retry and 10 before the second; MFA instead follows the fresh
time-window procedure below. Honor an explicitly selected stop-on-first-error mode
and server cooldowns. Exhausting a step's budget requires a handoff, not a fresh
budget from restarting the helper or changing accounts, proxies, or login methods.

Before each retry, inspect the current page, verified account, callback/helper
state, and any success evidence. If the step already succeeded, advance from there.
Inspect again after the backoff, immediately before retry/reload; the earlier error
snapshot may be stale because the request completed during the wait. While a new
submission is pending, an unchanged prior error/request ID is not a new failure.
Wait for the current request's result or the next page instead of spending another
attempt on that old message. Retain handled error IDs across a page reload.
If submission is still pending or its outcome is unknown, reconcile it before
resubmitting. Keep passwords, MFA enrollment/deletion, account import, and token
exchange out of a generic retry loop; each needs its own result check. In particular,
never replay a one-use authorization code after an uncertain token exchange.

Account labels and MFA rows can load after their form or success banner. An absent
email during rendering calls for a bounded readiness wait; a different explicit
email is an identity mismatch. Record an explicit password-update confirmation
before waiting for the MFA inventory. A loading inventory must not turn a confirmed
password change into a report that the password is unchanged or trigger resubmission.

| Observation | Recovery within the step budget |
| --- | --- |
| Loading page, temporarily missing control, stale DOM/CDP context | Wait and rediscover the target; do not treat this as an authentication failure or click again while a request is pending. |
| Generic temporary error or failed navigation before any confirmed mutation | Use the page's retry/reload path after the delay, recheck identity, and resume the failed step in the same browser and auth flow. |
| `ERR-837 - Account issue` without an explicit lock, suspension, or cooldown | Preserve the error/request ID; it does not establish the cause or prove suspension. Permit the bounded delayed retry of the failed sign-in stage when its retry/reload path is available. Preserve the pending callback listener. |
| Rejected TOTP code | Recheck the current authenticator and time, then wait for a fresh window as below. |
| Explicit wrong password, invalid new-password policy, locked/disabled account, rate limit, or required user-held factor | Stop automated submissions and report the required correction or cooldown; unchanged input is not a transient retry. |

Store only the stage, attempt count, timing, error category/request ID, and verified
outcome in the checkpoint. Resume a user-completed MFA/consent from the actual page,
even when the next password form omits the email; retain the prior verified identity
chain instead of unnecessarily navigating back and challenging the user again.

## TOTP timing and recovery

Use an authorized current authenticator; a supplied seed may be processed locally
without disclosing it to an external OTP service. Keep the seed and codes in memory.
Use its configured algorithm, digit count, and period; do not infer custom parameters
from a seed alone. TOTP's default period is 30 seconds, not a millisecond-precision
race. Clock offset, arrival near a boundary, and reuse of an already accepted code
are distinct possibilities to check. See [RFC 6238 sections 4–6](https://www.rfc-editor.org/rfc/rfc6238).

1. Compare the generator's clock with a reliable current time source. When using
   AWS HTTPS `Date` headers diagnostically, use the selected proxy, measure request
   start/end, and estimate offset from the local midpoint. Sample again if latency
   or caching makes the value uncertain; headers have finite precision and are not
   an exact view of the MFA verifier's clock. Apply a fresh offset only inside the
   helper; do not change the machine clock or hardcode a previous run's offset.
2. Fill/submit a fresh code with enough remaining lifetime for UI waits and network
   transit, normally at least 15 seconds. Otherwise wait for the next boundary plus
   1–2 seconds and generate then. If a code was already accepted for login and the
   next security step asks again in that window, wait for the next window as well.
   Track the account and attempted/accepted time-step counter, not the secret code.
3. After the first rejection, verify the account/current device, refresh the time
   estimate, and wait for the next untried window. Retry only a newly generated code,
   with at most two fresh-window retries under the same three-attempt step budget.
   Do not sweep codes from adjacent past/future windows or replay the rejected code.
   A new step cannot reuse an already attempted window for that account.
4. On success, continue from the resulting page. On budget exhaustion or an explicit
   lock/cooldown, retain the challenge and hand off. Waiting can resolve a near-expiry
   or reused-code problem; it does not repair a wrong seed or a persistent clock offset.

[AWS's Builder ID troubleshooting guidance](https://docs.aws.amazon.com/signin/latest/userguide/troubleshooting-builder-id-issues.html)
also identifies clock synchronization as relevant to authenticator failures. During
the October 2026 batch a measured clock offset was corrected and one challenge then
passed; another required user intervention. Those observations do not establish
that every MFA failure or ERR-837 was caused by timing or automation speed.

## Authorized text CAPTCHA

When the user asks the assistant to complete AWS Security check by reading the
displayed characters, follow that authorization rather than automatically handing
off every CAPTCHA. Open **Verify** once, capture the current owned challenge, read
the visible characters, and fill its **Verification answer** field. Submit once,
wait for the result, and require explicit success such as **You have successfully
solved the challenge** before continuing the pending password/MFA flow.

The observed text challenge uses a cross-origin iframe from
`cdn.us-east-1.threat-mitigation.aws.amazon.com`; top-page selectors do not reach
its input. Use frame-aware browser controls or native pointer/keyboard input after
checking that the target coordinates belong to that iframe. Do not type into the
underlying password field. Reacquire the current screenshot/geometry if layout
changes. Capture only the challenge where possible, excluding unmasked passwords,
MFA enrollment QR codes, and unrelated account details.

Keep the answer transient, without recording it or the challenge image in Git or
durable account records. Remove owned temporary screenshots after completion.
On an explicit incorrect/expired-answer response, read the new current challenge
and use the same bounded attempt budget; do not replay the old answer or submit
repeatedly while a request is pending. Hand off an unreadable/unsupported challenge,
an exhausted budget, or a verification that actually requires the user's device.

## Authorized 2fa.fun use

Send a TOTP secret to this site only when that disclosure is authorized. In the
isolated browser, open `https://2fa.fun/` without putting the secret in its URL.
The observed form uses `#SECRET2FA` (a textarea) and **Get Codes**.

- Enter only the required secret, not a full account/password bundle.
- Read the resulting six-digit code specifically from `input.faotp.value`.
  Do not search arbitrary page text or copy the adjacent secret field.
- `.time2fa` contains the remaining validity window. Allow enough time to submit
  (around 15 seconds); when nearly expired, wait for a fresh interval and reread
  or regenerate. Keep individual waits short and bounded.
- Transfer the code in memory to the verified AWS MFA input and submit promptly.
  Never print the code or save it to a tracker.
- If AWS rejects it, follow the same bounded fresh-window recovery above; never
  replay that code. Recheck time and the current authenticator before asking for
  user intervention when the retry budget is exhausted or a user-held factor is needed.
- Close the 2fa.fun tab after use. Do not send a newly enrolled phone secret there
  just because the user authorized the old secret for initial login.
