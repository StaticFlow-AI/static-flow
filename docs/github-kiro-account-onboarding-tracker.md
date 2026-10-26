# GitHub / Kiro Account Onboarding Tracker

Created: 2026-07-08

Purpose: track GitHub security-login, password-rotation, and Kiro onboarding or
refresh work across account batches.

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

<!-- github-ide-batch-20261026:start -->

## 2026-10-26 GitHub / Kiro IDE 批次（20 个账号）

用户确认统一改密并保留现有 2FA，按 GitHub 登录、邮箱归属核验、改密、Kiro IDE GitHub PKCE 授权、网关导入及公网请求归属验证的顺序逐个处理；成功与阻塞分别记录如下。
第 1–5、11–15 个登录直连；第 6–10、16–20 个经 `127.0.0.1:11115`。浏览器与授权请求遵循同一登录连接，导入后的推理请求固定到活跃美国代理。
使用官方 IDE 1.2.4 包中 GitHub PKCE 的实际协议；未启动 IDE 应用，也未使用 CLI device login。隔离 CLI 仅读取同一份 IDE 凭据来核验 Kiro 身份，默认 CLI 登录保持原样。
早期动作间隔约 1.5 秒；后续按用户要求，每次填入、点击和跳转后至少等待 2 秒，再检查页面就绪。临近过期的 2FA 等待新时间窗口。用户另行明确授权本批图片验证码的代答与提交。
原始 2FA 与邮箱的对应关系单独保存到私有持久文件 `~/.local/share/kiro-github/private-records/2026-10-26-original-2fa.json`，权限 `0600`；仓库不保存密码或验证器密钥。

预检完整分页读取 405 个线上账号，目标账号名及邮箱均未重复。

| 序号 | 邮箱 | GitHub 用户名 | 登录连接 | 改密 | IDE 授权 | 导入 / 实际请求 |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | `raditya.wibawa@binus.ac.id` | `radityawbawa` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 2 | `varel.robert@binus.ac.id` | `varelrobert` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 3 | `haegel.manurung@binus.ac.id` | `michellelewism356` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 4 | `samuel.prasetyo@binus.ac.id` | `samuelprasetyo2251` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 5 | `reno.natanagara@binus.ac.id` | `Reno-Rahman` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 6 | `rafa.fahimsya@binus.ac.id` | `rafafahimsya` | 本地端口代理 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 7 | `muhammad.pratama079@binus.ac.id` | 待核验 | 本地端口代理 | 未修改 | 待完成 | 待导入 |
| 8 | `phoebe.yala@binus.ac.id` | `pboebeyala` | 本地端口代理 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 9 | `sergio.putra@binus.ac.id` | `sergioputra` | 本地端口代理 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 10 | `celine.anggita@binus.ac.id` | `celineanggita` | 本地端口代理 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 11 | `stanly.valerian@binus.ac.id` | `stanlyvalerian` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 12 | `rifqi.farikh@binus.ac.id` | `rapaaaaaaaaaa` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 13 | `chesya.sandi@binus.ac.id` | `holmaielbrireana` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 14 | `inayah.abdullah@binus.ac.id` | `inayahabdullah` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 15 | `fahrezi.pratama@binus.ac.id` | `fahrezipratama` | 直连 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 16 | `ejaz.halid@binus.ac.id` | `ejazhalid` | 本地端口代理 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 17 | `darren.natael@binus.ac.id` | `taylordunlow` | 本地端口代理 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 18 | `najla.dinata@binus.ac.id` | `najladinata` | 本地端口代理 | GitHub 已确认更新 | 已核验 | HTTP 200 / KIRO_OK / Usage 归属已核验 |
| 19 | `nathania.putri001@binus.ac.id` | 待核验 | 本地端口代理 | 未修改 | 待完成 | 待导入 |
| 20 | `richardo.surya@binus.ac.id` | 待核验 | 本地端口代理 | 未修改 | 待完成 | 待导入 |

已改密 17/20，IDE 授权 17/20，实际请求归属核验 17/20。原有 2FA 保留；未另行执行新密码的完整登录测试。

| 邮箱 | 固定推理代理 | 导入时订阅 / 剩余 credits | 最小间隔（ms） | Usage ID |
| --- | --- | --- | ---: | --- |
| `raditya.wibawa@binus.ac.id` | `do-us-4` | KIRO STUDENT / 999.99 | 643 | `llm-usage-d29614a8-c864-4d21-a254-24b3c7dce41f` |
| `varel.robert@binus.ac.id` | `do-us-9` | KIRO STUDENT / 999.99 | 499 | `llm-usage-67387a96-5fe8-47cc-a1a6-2ce6490d8351` |
| `haegel.manurung@binus.ac.id` | `do-us-4` | KIRO STUDENT / 999.99 | 251 | `llm-usage-3a6abe49-ddf0-478e-9376-759bf73617da` |
| `samuel.prasetyo@binus.ac.id` | `do-us-9` | KIRO STUDENT / 999.99 | 239 | `llm-usage-16f935f0-45a7-49f6-8946-c643ab1ee8a4` |
| `reno.natanagara@binus.ac.id` | `do-us-4` | KIRO STUDENT / 999.99 | 997 | `llm-usage-f0c529f8-c1ef-41f1-8c5c-50e1f4a3dc58` |
| `rafa.fahimsya@binus.ac.id` | `do-us-9` | KIRO STUDENT / 999.99 | 447 | `llm-usage-a7a5bdd8-36ac-4a90-9f73-cb4e109bffcb` |
| `phoebe.yala@binus.ac.id` | `do-us-9` | KIRO STUDENT / 999.99 | 668 | `llm-usage-b127cfb2-a4a3-4965-b3a1-bc0e1c7598d8` |
| `sergio.putra@binus.ac.id` | `do-us-4` | KIRO STUDENT / 999.99 | 663 | `llm-usage-0245a4b0-0335-4563-96b6-5127c2f8c454` |
| `celine.anggita@binus.ac.id` | `do-us-9` | KIRO STUDENT / 999.99 | 737 | `llm-usage-0cbdad6d-7c30-4aee-acdb-b15d54914836` |
| `stanly.valerian@binus.ac.id` | `do-us-4` | KIRO STUDENT / 999.99 | 749 | `llm-usage-c3d57a5c-e73f-4025-b646-3b55ad2267c7` |
| `rifqi.farikh@binus.ac.id` | `do-us-9` | KIRO STUDENT / 999.94 | 1000 | `llm-usage-014a101d-eb7c-424c-9f68-0a27c2586ed6` |
| `chesya.sandi@binus.ac.id` | `do-us-11` | KIRO STUDENT / 999.98 | 427 | `llm-usage-f3c2a944-3b25-45ae-94f8-f4bb44096fbe` |
| `inayah.abdullah@binus.ac.id` | `do-us-4` | KIRO STUDENT / 999.99 | 261 | `llm-usage-1228e1b0-e772-4a0a-9891-3dd09d07bae4` |
| `fahrezi.pratama@binus.ac.id` | `do-us-4` | KIRO STUDENT / 999.99 | 767 | `llm-usage-33bd7853-50a7-4e01-9065-65fa3f39c689` |
| `ejaz.halid@binus.ac.id` | `do-us-9` | KIRO STUDENT / 999.99 | 845 | `llm-usage-10921bc4-dde4-4b5b-af58-c1f6bedaf6bd` |
| `darren.natael@binus.ac.id` | `do-us-1` | KIRO STUDENT / 999.98 | 925 | `llm-usage-64882c7e-aae7-4b18-8641-e995ae9914a4` |
| `najla.dinata@binus.ac.id` | `do-us-11` | KIRO STUDENT / 999.99 | 726 | `llm-usage-a5eb54a5-618a-4daa-b7c8-e1af2ae502a0` |

已完成账号使用 IDE 来源的 `auth_method=social` / GitHub provider，并发 3、最小间隔 200–1000 ms、最低剩余额度 10 credits。每个测试 Key 仅固定该账号，关闭直接 Anthropic 池及策略回退；临时 Key 已删除并核验不存在。

raditya.wibawa@binus.ac.id：初次推理代理 tmp-us-for-kiro 的额度校验返回 502；新建记录已撤回并核验不存在。保留改密与认证，改用合格美国代理后验证成功；该失败代理只在本批排除，未更改全局代理配置。

varel.robert、reno.natanagara、sergio.putra：IDE 授权曾返回 invalid_request / Github Error - 500，未返回授权码。保留同一浏览器和已完成的改密，经有限重试均已恢复并通过实际请求归属核验。

rafa.fahimsya@binus.ac.id：前三个 IDE 授权流程返回 500；按用户要求继续后续账号后回头重试，第 4 个流程已恢复。Kiro 身份、学生额度、公网请求及 Usage 归属均已通过，历史失败记录保留。

inayah.abdullah@binus.ac.id：首个 IDE 流程累计提交 6 次图片验证仍未完成，末次提交前已目视核对选中项。保留改密和原验证器，后续同一浏览器的新授权流程第 1 轮图片验证通过（累计第 7 次），已完成导入和实际请求核验；历史计数保留。

图片选择由同一可见浏览器输入助手连续接收、逐项执行，每项仍等待至少 2 秒，减少控制器往返额外耗时。临近到期或已使用的 2FA 时间窗口不重复提交，等待下一窗口；没有更改系统时间。

nathania.putri001@binus.ac.id 与 richardo.surya@binus.ac.id：各以用户提供密码及已授权统一密码登录一次，原生输入后的字段值均核验一致，GitHub 均返回 Incorrect username or password。已请求用户核对邮箱和当前密码；未改密、未修改 MFA、未授权或导入 Kiro。

muhammad.pratama079@binus.ac.id：GitHub 明确显示 Account suspended，停止自动重试并记录到封禁跟踪文档；未改密、未修改 MFA、未授权或导入 Kiro，也未提交申诉。

`muhammad.pratama079@binus.ac.id` 待处理：GitHub explicitly shows Account suspended。

`nathania.putri001@binus.ac.id` 待处理：GitHub rejected provided password and authorized uniform password; correct credentials required。

`richardo.surya@binus.ac.id` 待处理：GitHub rejected provided password and authorized uniform password; correct credentials required。

最终完整分页读取 422 个线上账号，本批 17 个通过配置及请求证据核验，另有 3 个保留上述阻塞；临时测试 Key 剩余 0。

<!-- github-ide-batch-20261026:end -->
