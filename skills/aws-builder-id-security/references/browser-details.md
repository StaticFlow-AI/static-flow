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

Prefer browser-tool fill and click actions. With CDP, controlled inputs may require
the native input value setter followed by bubbling `input` and `change` events.
Check the resulting form state without printing field values.

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
- If AWS rejects it, do not keep replaying that code. Determine whether it expired
  or whether the selected authenticator is wrong; request user help if unclear.
- Close the 2fa.fun tab after use. Do not send a newly enrolled phone secret there
  just because the user authorized the old secret for initial login.
