# Kiro IDE Builder ID authorization

Prefer this path for fresh Builder ID logins. It obtains an IDE client grant;
renaming a CLI credential or changing its provider field does not convert the
grant. Preserve usable CLI credentials separately.

The protocol below was verified in the official Kiro IDE 1.2.4 package,
`kiro.kiro-agent/packages/kiro-shared/dist/portal-auth-provider-*.js` and
`external-idp-auth-provider-*.js`. Check the available official package before
assuming version-specific parameters remain current. The
[official authentication guide](https://kiro.dev/docs/getting-started/authentication/)
documents browser-based IDE sign-in.

## Browser and isolation

Reuse the retained browser only after verifying the exact supplied account's AWS
email. Preserve its profile, debugger metadata, and any pending callback listener
across user replies. A matching browser title or a local cache label is insufficient.

Honor the selected login proxy for both Chrome and all outbound authorization/token
requests. Verify Chrome's actual proxy flag and real egress. Local callback traffic
must stay on loopback. Use a separate account credential directory (`0700`) and
token/client files (`0600`); do not overwrite another account's global IDE cache.
Do not replace `HOME` to change accounts. Normal IDE credential files are expected;
checkpoints, trackers, and diagnostic output must exclude their secret values.

Use the visible official IDE or implement its observed browser/PKCE protocol.
When using a protocol helper, report that mode accurately instead of claiming
an IDE application was opened. Keep authorization URLs, codes, state, PKCE
verifiers, and registration/token responses in process memory.

## Portal and Builder ID flow

1. Start one loopback callback server before opening the portal. In the audited
   package, the portal uses `http://localhost:<port>` and accepts
   `/oauth/callback` or `/signin/callback`. Candidate ports are
   `3128, 4649, 6588, 8008, 9091, 49153, 50153, 51153, 52153, 53153`.
2. Open `https://app.kiro.dev/signin` in the verified browser with fresh `state`,
   a SHA-256 PKCE `code_challenge`, `code_challenge_method=S256`, the callback
   `redirect_uri`, and `redirect_from=KiroIDE`. Choose **Builder ID**.
3. Validate callback state and require `login_option=builderid`, its `issuer_url`,
   and `idc_region`. This selects the provider; it has not obtained an IDE token.
   For the audited personal Builder ID flow, the issuer/start URL is
   `https://view.awsapps.com/start` and the region is `us-east-1`.
4. Register the IDE client through
   `POST https://oidc.<region>.amazonaws.com/client/register` with:

   ```json
   {
     "clientName": "Kiro IDE",
     "clientType": "public",
     "scopes": [
       "codewhisperer:completions",
       "codewhisperer:analysis",
       "codewhisperer:conversations",
       "codewhisperer:transformations",
       "codewhisperer:taskassist"
     ],
     "grantTypes": ["authorization_code", "refresh_token"],
     "redirectUris": ["http://127.0.0.1/oauth/callback"],
     "issuerUrl": "https://view.awsapps.com/start"
   }
   ```

5. Start the IDC callback listener on a dynamic loopback port. Use a new state and
   PKCE verifier for this second flow. Open
   `https://oidc.<region>.amazonaws.com/authorize` with `response_type=code`,
   `client_id`, `redirect_uri=http://127.0.0.1:<port>/oauth/callback`, comma-joined
   `scopes`, `state`, `code_challenge`, and `code_challenge_method=S256`.
6. Complete any AWS password/MFA challenge in the same browser using the current
   authorized credentials. Verify the account and application before granting
   consent. Pause for the user's required verification; keep the listener alive.
7. Validate callback state and exchange its code once through
   `POST https://oidc.<region>.amazonaws.com/token` with `clientId`, `clientSecret`,
   `grantType=authorization_code`, the exact `redirectUri`, `code`, and
   `codeVerifier`. Successful callback receipt alone does not prove token exchange.
   On ambiguous exchange failure, reconcile state before replaying a one-use code.

Close only owned callback servers after completion. Drain/destroy their owned
connections during shutdown: a browser preconnect can otherwise leave server
closure waiting indefinitely. Retain the authenticated Chrome window separately.

## Import the IDE credential

The IDE token file is `kiro-auth-token.json`: `accessToken`, `refreshToken`,
`expiresAt`, `clientIdHash`, `authMethod=IdC`, `provider=BuilderId`, and `region`.
Its matching client registration file contains `clientId`, `clientSecret`, and
`expiresAt`. The audited hash is SHA-1 of `JSON.stringify({startUrl})`; the default
IDE stores both files under `~/.aws/sso/cache`. Use the task's isolated cache when
one was selected, and require that token and registration come from this flow.

Parse both JSON files locally. Build the existing `/accounts/import-auth` payload
with `access_token`, `refresh_token`, `expires_at`, `client_id`, `client_secret`,
`auth_method=idc`, `provider=BuilderId`, the actual region fields, and verified
email. Do not infer email from a filename or quota. Retain the verified browser
identity and same-flow provenance; reconcile any conflicting upstream identity.
Do not pass IDE JSON to the CLI importer's `--sqlite-file` argument.

Follow [import and verification](import-and-verify.md) for full inventory duplicate
checks, proxy ranking, scheduling, balance, exact-account public request attribution,
and temporary-key cleanup. Reuse its policy functions where useful. Existing
accounts require an authorized update preserving their settings and must never
enter new-account rollback. Record the IDE grant's provenance without recording
tokens. A public request must use the imported IDE credential and be attributed
to that exact account before the full onboarding is reported as successful.
