# GitHub / Kiro Account Onboarding Tracker

Created: 2026-07-08

Purpose: track GitHub security-login, password-rotation, and Kiro refresh work
done during the July 2026 onboarding run.

Security rule: do not record GitHub passwords, new passwords, TOTP secrets,
2FA codes, cookies, browser profiles, or session tokens in this document.
Only record operational status and follow-up work.

## 2026-07-08 GitHub Security Login

These accounts were opened with the isolated browser helper in `login-only`
mode, using proxy `http://127.0.0.1:11112`. The helper reached GitHub security
settings and left the browser available for manual security changes.

| GitHub username | Operation | Result | Notes |
|---|---|---|---|
| `linkueisz3` | Security-settings login | `login_completed` | Re-run after switching proxy from `11111` to `11112`. |
| `linkueisz4` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |
| `linkueisz5` | Security-settings login | `login_completed` | No further automated action taken. |
| `linkueisz8` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |
| `linkueisz9` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |
| `linkueisz10` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |
| `linkueisz11` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |
| `linkueisz12` | Security-settings login | `login_completed` | Required manual verification/restriction handling before reaching settings. |
| `linkueisz13` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |
| `linkueisz14` | Security-settings login | `login_completed` | Required manual verification/restriction handling before reaching settings. |
| `linkueisz15` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |
| `linkueisz16` | Security-settings login | `login_completed` | GitHub reported verification/restriction before reaching settings. |
| `linkueisz17` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |
| `linkueisz18` | Security-settings login | `login_completed` | GitHub briefly reported verification/restriction before reaching settings. |

## 2026-07-08 GitHub Password Rotation

These accounts were processed with the password-rotation helper, using proxy
`http://127.0.0.1:11112`, automatic app-code 2FA via the provided TOTP secret,
and `--keep-browser` so the browser remained available for manual 2FA changes.
The shared new password is intentionally not recorded here.

| GitHub username | Operation | Result | Follow-up |
|---|---|---|---|
| `Hartmannxz` | Password rotation | `password_change_completed` | User to manually update 2FA in the retained browser session. |
| `Carmellaz` | Password rotation | `password_change_completed` | User to manually update 2FA in the retained browser session. |
| `Jacklynzz` | Password rotation | `password_change_completed` | User to manually update 2FA in the retained browser session. |
| `Bradtkez` | Password rotation | `password_change_completed` | User to manually update 2FA in the retained browser session. |
| `Naderxxz` | Password rotation | `password_change_completed` | User to manually update 2FA in the retained browser session. |

## Earlier GitHub Password Rotation Note

| GitHub username | Operation | Result | Notes |
|---|---|---|---|
| `linkeuisz3` | Password rotation | `password_change_completed` | This was the initially supplied spelling. User later corrected the separate account name to `linkueisz3`, which was handled with login-only. Verify spelling before using this account operationally. |

## Kiro GitHub Social Refreshes

These accounts were refreshed through the Kiro GitHub social onboarding flow.
Passwords and auth payloads are intentionally not recorded here.

| Account name | Email recorded by Kiro/llm-access | Result | Remaining credits / proxy notes |
|---|---|---|---|
| `onovansdf` | `onovansdf@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 742.86. |
| `endrickgd` | `endrick@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 770.4; proxy `aws_us_east1`. |
| `adenrtheds` | `adenrtheds@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 515.18; proxy `do-us-2`. |
| `ulligandw` | `ulligandw@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 649.25; proxy `tmp-us-for-kiro`. |
| `terhilder` | `terhilder@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 777.43; proxy `aws_us_east1`. |
| `bbonsrfds` | `bbonsrfds@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 807.08; proxy `do-us-2`. |
| `ennoxfs` | `ennoxfs@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 761.41; proxy `tmp-us-for-kiro`. |
| `allahaneas` | `allahaneas@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 816.8; proxy `aws_us_east1`. GitHub password was later rotated and login-only security access succeeded. |
| `afanchard` | `afanchard@utexas.edu` | Ready, KIRO STUDENT 1000 | Remaining 831.02; proxy `do-us-2`. |

## Suspended / Disabled Kiro Accounts

These accounts were not usable because GitHub or Kiro account state blocked the
refresh. Keep detailed appeal work in `docs/github-account-suspension-appeal-tracker.md`.

| Account name | Observed state | Notes |
|---|---|---|
| `gsetynm` | GitHub suspended | llm-access remained `auth_401`; remaining 796.56. |
| `jurdgh` | GitHub suspended | Old state remained `auth_401`; remaining 771.66. |
| `ghdgb` | GitHub suspended | Old state remained `auth_401`; remaining 566.28. |
| `fgredb` | GitHub suspended | Old state remained `auth_401`; remaining 764.61. |
| `gyhjf` | GitHub suspended | Old state remained `auth_401`; remaining 774.09. |
| `drfgyyhj` | GitHub suspended | Old state remained `auth_401`; remaining 823.77. |
| `ghrdfh` | GitHub suspended | Old state remained `auth_401`; remaining 410.98. |
| `hydgjj` | GitHub suspended; llm-access disabled | `disabled=true`, `issue_kind=disabled`, `issue_summary=account status is disabled`. |

## Operational Notes

- For GitHub security-login and password-rotation tasks, prefer proxy
  `http://127.0.0.1:11112` unless the user explicitly switches proxy again.
- For login-only tasks, successful completion means the helper reached GitHub
  security settings; it does not imply password or 2FA was changed.
- For password-rotation tasks, successful completion means the helper submitted
  the password-change form and reported completion. Manual 2FA changes remain
  a user action.
- Do not commit or persist secrets from the account batch. If another tracker
  needs a credential reference, use an external secret store name rather than
  the credential value.
