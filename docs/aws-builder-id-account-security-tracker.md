# AWS Builder ID 账号安全维护记录

创建日期：2026-10-10

记录本轮用户提供并完成核对的 AWS Builder ID 账号，以及密码和验证器维护结果。
不记录密码原文、密码中的具体数字段、TOTP 密钥、二维码、一次性验证码、Cookie 或会话令牌。

## 账号清单

| AWS Builder ID 账号 | 密码状态 | 2FA 状态 | 更新日期 |
| --- | --- | --- | --- |
| `dyta.sentya@binus.ac.id` | 已按用户最终指定的密码更新，AWS 页面确认成功 | 用户已绑定手机验证器；当前仅保留 `MFA 2`，旧 `MFA 1` 已删除 | 2026-10-10 |
| `faith.seumahu@binus.ac.id` | 已按用户确认沿用的密码更新，AWS 页面确认成功 | 用户已绑定手机验证器；当前仅保留 `MFA 2`，旧 `MFA 1` 已删除 | 2026-10-11 |
| `faiz.abdurahman@binus.ac.id` | 已按用户确认沿用的密码更新，AWS 页面确认成功，新密码已通过再次登录验证 | 用户已绑定手机验证器；当前仅保留 `MFA 2`，旧 `MFA 1` 已删除 | 2026-10-11 |
| `andre.hasibuan@binus.ac.id` | 已按用户确认沿用的密码更新，AWS 页面确认成功；新密码已通过 AWS 再次身份验证 | 用户已绑定手机验证器；当前仅保留 `MFA 2`，旧 `MFA 1` 已删除 | 2026-10-11 |
| `faiz.fauzan@binus.ac.id` | 已按用户确认沿用的密码更新，AWS 页面确认成功 | 用户已绑定手机验证器；当前仅保留 `MFA 2`，旧 `MFA 1` 已删除 | 2026-10-11 |

## 密码助记

适用于本次已确认完成改密的 `dyta.sentya@binus.ac.id`、
`faith.seumahu@binus.ac.id`、`faiz.abdurahman@binus.ac.id`、
`andre.hasibuan@binus.ac.id` 和 `faiz.fauzan@binus.ac.id`：

> 联想本次使用的服务名，首字母大写；接着先感叹、后疑问；末尾沿用本次约定的数字段。

此处仅保留助记线索，不记录完整密码或具体数字。

## 2026-10-10 操作与核验

- 用户在 AWS 新验证器二维码页面完成手机绑定，新设备名称为
  `dyta.sentya@binus.ac.id's MFA 2`，注册日期为 2026-10-10。
- 删除原设备 `dyta.sentya@binus.ac.id's MFA 1`，其注册日期为 2026-10-09。
  AWS 明确显示删除成功，设备列表随后仅剩 `MFA 2`。
- 修改密码前，使用新手机验证器的验证码完成 AWS 再次身份验证。
- 按用户最终指定的密码完成修改。AWS 安全页面明确显示
  “Your password has been successfully updated.”，并确认 `MFA 2` 仍在设备列表中。

## 2026-10-10 Kiro 接入与请求验证

| 项目 | 已验证结果 |
| --- | --- |
| 登录身份 | `dyta.sentya@binus.ac.id`，`kiro-cli whoami` 显示 `Logged in with Builder ID` |
| llm-access 账号名 | `kiro-dyta-sentya-builder-id`，已启用，`auth_method=idc`、`provider=aws` |
| 订阅与额度 | `KIRO STUDENT`，额度上限 1000 credits；导入时剩余 999.96 credits |
| 代理与调度 | 固定美国代理 `do-us-3`；并发 3，最小请求间隔 683 ms，最低剩余额度阈值 10 credits |
| 公网请求 | `https://ackingliu.top/v1/messages`，模型 `claude-sonnet-5.5`，HTTP 200，返回 `KIRO_OK` |
| 用量归属 | usage journal 确认请求属于上述账号，输入 10 tokens、输出 2 tokens，`usage_missing=false` |

验证使用仅绑定该账号的临时 Key，关闭其他上游池和策略回退；请求完成后已删除该 Key，
并核对其不再出现在 Key 列表中。导入时首选代理 `tmp-us-for-kiro` 的额度刷新失败，
随后通过 `do-us-3` 完成额度与真实请求验证。

该账号使用独立的本地凭据目录，保留原有默认 Kiro CLI 账号。检查此账号身份：

```bash
XDG_DATA_HOME=/home/ts_user/.local/share/kiro-builder-accounts/dyta-sentya kiro-cli whoami
```

本记录不保存登录密码、OAuth 凭据或临时测试 Key。

## 2026-10-11 操作与核验

- 用户为 `faith.seumahu@binus.ac.id` 完成手机验证器绑定，新设备名称为
  `faith.seumahu@binus.ac.id's MFA 2`，注册日期为 2026-10-11。
- 删除原设备 `faith.seumahu@binus.ac.id's MFA 1`，其注册日期为 2026-10-10。
  AWS 明确显示删除成功，设备列表随后仅剩 `MFA 2`。
- 使用新手机验证器的验证码通过修改密码前的再次身份验证。
- 用户确认沿用前一账号最终设置的密码；修改后 AWS 明确显示
  “Your password has been successfully updated.”，并确认 `MFA 2` 仍在设备列表中。
- 密码助记沿用本文上方的提示，不记录完整密码、具体数字或验证码。

## 2026-10-11 Kiro 接入与请求验证

| 项目 | 已验证结果 |
| --- | --- |
| 登录身份 | `faith.seumahu@binus.ac.id`，`kiro-cli whoami` 显示 `Logged in with Builder ID` |
| llm-access 账号名 | `kiro-faith-seumahu-builder-id`，已启用，`auth_method=idc`、`provider=aws`，无账号异常 |
| 订阅与额度 | `KIRO STUDENT`，额度上限 1000 credits；导入时剩余 999.95 credits |
| 代理与调度 | 固定美国代理 `aws_us_east1`；并发 3，最小请求间隔 919 ms，最低剩余额度阈值 10 credits |
| 公网请求 | `https://ackingliu.top/v1/messages`，模型 `claude-sonnet-5.5`，HTTP 200，返回 `KIRO_OK`，以 `end_turn` 正常结束 |
| 用量归属 | usage journal 确认请求属于上述账号，输入 10 tokens、输出 2 tokens，`usage_missing=false` |

本次复用用户确认的新密码，并通过新手机验证器完成 AWS 登录和 Kiro CLI 授权。
导入前完整分页检查 345 个现有账号，导入后完成上游身份去重及账号配置核验。
首选代理 `tmp-us-for-kiro` 额度刷新失败，随后通过 `aws_us_east1` 完成额度与真实请求验证。

验证使用仅绑定该账号的临时 Key，关闭其他上游池和策略回退；请求完成后已删除该 Key，
并按精确 ID 核对其不再出现在完整 Key 列表中。上表额度为导入时数值，不是测试后的刷新值。

该账号使用独立的本地凭据目录，原有默认 Kiro CLI 账号仍保留。检查此账号身份：

```bash
XDG_DATA_HOME=/home/ts_user/.local/share/kiro-builder-accounts/faith-seumahu kiro-cli whoami
```

本记录不保存登录密码、验证码、OAuth 凭据或临时测试 Key。

## 2026-10-11 faiz Kiro 接入与请求验证

| 项目 | 已验证结果 |
| --- | --- |
| 登录身份 | `faiz.abdurahman@binus.ac.id`，`kiro-cli whoami` 显示 `Logged in with Builder ID` |
| llm-access 账号名 | `kiro-faiz-abdurahman-builder-id`，已启用，`auth_method=idc`、`provider=aws`，无账号异常 |
| 订阅与额度 | `KIRO STUDENT`，额度上限 1000 credits；导入时剩余 999.95 credits |
| 代理与调度 | 固定美国代理 `do-us-10`；并发 3，最小请求间隔 801 ms，最低剩余额度阈值 10 credits |
| 公网请求 | `https://ackingliu.top/v1/messages`，模型 `claude-sonnet-5.5`，HTTP 200，返回 `KIRO_OK`，以 `end_turn` 正常结束 |
| 用量归属 | usage journal 确认请求属于上述账号，输入 10 tokens、输出 2 tokens，`usage_missing=false` |

上号阶段按用户要求先完成 Kiro 登录和接入，当时未修改密码或 MFA。复用已确认身份的
AWS 浏览器会话后，直接进入 Kiro 设备码确认与授权页面，没有再次要求密码或验证码。
导入使用已更新的技能脚本，完成分页查重、已核验邮箱传递和账号设置复核。
首选代理 `tmp-us-for-kiro` 额度刷新返回 HTTP 502，随后通过 `do-us-10` 验证成功。

验证使用仅绑定该账号的临时 Key，关闭其他上游池和策略回退；请求完成后已删除并
按精确 ID 核对其不再出现在完整 Key 列表中。上表额度为导入时数值。
登录窗口保留以便后续继续操作，原有默认 Kiro CLI 账号仍保留。检查此账号身份：

```bash
XDG_DATA_HOME=/home/ts_user/.local/share/kiro-builder-accounts/faiz-abdurahman kiro-cli whoami
```

## 2026-10-11 faiz 安全维护

- 复用该账号上号后保留的 AWS 浏览器，核对原验证器为
  `faiz.abdurahman@binus.ac.id's MFA 1`，注册日期为 2026-10-10。
- 按用户要求沿用此前最终确认的密码；AWS 明确显示
  “Your password has been successfully updated.”。
- 登记新验证器前，使用更新后的密码和原验证器通过 AWS 再次登录验证。
- 用户完成手机验证器绑定后，AWS 列表显示新设备
  `faiz.abdurahman@binus.ac.id's MFA 2`，注册日期为 2026-10-11。
- 按原设备名称和注册日期选中旧 `MFA 1`，确认删除；AWS 明确显示
  “MFA 1 has been successfully deleted.”，随后列表仅剩新 `MFA 2`。
- 密码助记沿用本文上方提示，不记录密码、验证码或验证器密钥。
  已登录的 AWS 窗口继续保留，方便后续复用。

## 2026-10-11 andre Kiro 接入与请求验证

| 项目 | 已验证结果 |
| --- | --- |
| 登录身份 | `andre.hasibuan@binus.ac.id`，`kiro-cli whoami` 显示 `Logged in with Builder ID` |
| llm-access 账号名 | `kiro-andre-hasibuan-builder-id`，已启用，`auth_method=idc`、`provider=aws` |
| 订阅与额度 | `KIRO STUDENT`，额度上限 1000 credits；导入时剩余 999.96 credits |
| 代理与调度 | 固定美国代理 `do-us-3`；并发 3，最小请求间隔 969 ms，最低剩余额度阈值 10 credits |
| 公网请求 | `https://ackingliu.top/v1/messages`，模型 `claude-sonnet-5.5`，HTTP 200，返回 `KIRO_OK`，以 `end_turn` 正常结束 |
| 用量归属 | usage journal 确认请求属于上述账号，输入 10 tokens、输出 2 tokens，`usage_missing=false` |

导入前使用独立 CLI 数据目录完成 Builder ID 登录、设备码核对和 Kiro CLI 授权，
保留原有默认 Kiro 账号。导入脚本完成完整分页查重、邮箱传递和账号配置核验。
首选代理 `tmp-us-for-kiro` 的额度刷新返回 HTTP 502，随后通过 `do-us-3` 验证成功。

公网测试使用仅绑定此账号的临时 Key，关闭其他上游池和策略回退；请求耗时 7.54 秒。
测试后已删除临时 Key，并按精确 ID 核对其不再出现在完整 Key 列表中。
上表额度为导入时数值，不是测试后的刷新值。检查此账号身份：

```bash
XDG_DATA_HOME=/home/ts_user/.local/share/kiro-builder-accounts/andre-hasibuan kiro-cli whoami
```

## 2026-10-11 andre 安全维护

- 核对原验证器为 `andre.hasibuan@binus.ac.id's MFA 1`，注册日期为 2026-10-05。
- 登录时发现本机时间比 AWS 和验证码网站的服务器时间慢约 49 秒；使用服务器时间
  生成验证码后通过验证，未修改系统时间。
- 按用户要求沿用此前最终确认的密码；AWS 明确显示
  “Your password has been successfully updated.”。
- 用户完成手机扫码和 **Assign MFA** 后，AWS 要求重新输入密码；更新后的密码通过验证，
  返回安全页面。设备列表确认新 `andre.hasibuan@binus.ac.id's MFA 2`，注册日期为 2026-10-11。
- 按原设备名称和注册日期选中旧 `MFA 1`，核对删除确认框后提交；AWS 明确显示
  “MFA 1 has been successfully deleted.”，随后列表仅剩新 `MFA 2`。
- 密码助记沿用本文上方提示；本记录不保存密码、验证码、二维码或验证器密钥。
  AWS 登录窗口继续保留，方便后续复用。

## 2026-10-11 faiz.fauzan Kiro 接入与请求验证

| 项目 | 已验证结果 |
| --- | --- |
| 登录身份 | `faiz.fauzan@binus.ac.id`，`kiro-cli whoami` 显示 `Logged in with Builder ID` |
| llm-access 账号名 | `kiro-faiz-fauzan-builder-id`，已启用，`auth_method=idc`、`provider=aws` |
| 订阅与额度 | `KIRO STUDENT`，额度上限 1000 credits；导入时剩余 999.96 credits |
| 代理与调度 | 固定美国代理 `aws_us_east1`；并发 3，最小请求间隔 512 ms，最低剩余额度阈值 10 credits |
| 公网请求 | `https://ackingliu.top/v1/messages`，模型 `claude-sonnet-5.5`，HTTP 200，返回 `KIRO_OK`，以 `end_turn` 正常结束 |
| 用量归属 | usage journal 确认请求属于上述账号，输入 10 tokens、输出 2 tokens，`usage_missing=false` |

使用独立 CLI 数据目录完成 Builder ID 登录、设备码核对和 Kiro CLI 授权，保留原有
默认 Kiro 账号。导入脚本完成完整分页查重、邮箱传递和账号配置核验。
首选代理 `tmp-us-for-kiro` 的额度刷新返回 HTTP 502，随后通过 `aws_us_east1` 验证成功。

公网测试使用仅绑定此账号的临时 Key，关闭其他上游池和策略回退；请求耗时 6.01 秒。
测试后已删除临时 Key，并按精确 ID 核对其不再出现在完整 Key 列表中。
上表额度为导入时数值，不是测试后的刷新值。检查此账号身份：

```bash
XDG_DATA_HOME=/home/ts_user/.local/share/kiro-builder-accounts/faiz-fauzan kiro-cli whoami
```

## 2026-10-11 faiz.fauzan 安全维护

- 核对原验证器为 `faiz.fauzan@binus.ac.id's MFA 1`，注册日期为 2026-10-10。
- 本机时间仍比 AWS 服务器时间慢约 49 秒；按服务器时间生成验证码后通过验证，
  未修改系统时间。
- 按用户要求沿用此前最终确认的密码；AWS 明确显示
  “Your password has been successfully updated.”。
- 用户完成手机扫码和 **Assign MFA** 后，AWS 已返回安全页面。设备列表确认新
  `faiz.fauzan@binus.ac.id's MFA 2`，注册日期为 2026-10-11。
- 按原设备名称和注册日期选中旧 `MFA 1`，核对删除确认框后提交；AWS 明确显示
  “MFA 1 has been successfully deleted.”，列表刷新后仅剩新 `MFA 2`。
- 密码助记沿用本文上方提示；本记录不保存密码、验证码、二维码或验证器密钥。
  AWS 登录窗口继续保留，方便后续复用。
