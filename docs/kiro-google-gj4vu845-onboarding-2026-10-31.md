# Kiro Google POWER 账号接入记录（2026-10-31）

最终核验时间：2026-10-31 03:28:04，UTC+08:00。

| 项目 | 已核验结果 |
| --- | --- |
| 邮箱 | `gj4vu845@gmail.com` |
| 网关账号 | `kiro-gj4vu845-google-social` |
| 登录与身份 | 独立 Kiro CLI 设备授权，`whoami` 确认为对应邮箱、Google 登录 |
| 认证方式 | `social / google` |
| 套餐与额度 | `KIRO POWER`，上限 10,000 credits；请求验证后剩余 9,999.82 |
| 状态 | 已启用 |
| 并发上限 | 10 |
| RPM 上限 | 5 |
| 最小启动间隔 | 425 ms |
| 最低剩余额度 | 10 credits |
| 账号调度池 | `credit_first`（余额优先） |
| 固定代理 | `aws_sg5`，`llm-proxy-77b182943a884d589ff5aa3456e3bd31` |
| 实际新加坡出口 | `<redacted-sg-proxy-ip>`，SG / Singapore |

Google 两次明确拒绝所提供的密码后，切换到官方“尝试其他方式”。用户完成手机通知确认及 Google Authenticator 验证，页面显示 Welcome back；选择 Continue，未修改 Google 密码。保留该账号已验证的独立浏览器会话，重新申请 Kiro 设备授权并取得社交登录凭据，避免重复 Google 验证。

先以禁用状态导入，再在同一次账号更新中绑定固定新加坡代理并启用。完整账号清单及上游身份去重通过，套餐和余额刷新正常。生产主机实际运行的 `http-proxy-cli :11117` 使用强制代理；经该客户端查询实际出口及地理位置，确认为新加坡。

通过只允许该账号的临时 Key，经公网 `/v1/messages` 请求 `claude-sonnet-5.5`，返回 HTTP 200、`KIRO_OK`、`end_turn`。Usage `llm-usage-c22934c8-a889-41bf-ab91-10b4d7c35152` 的实际账号匹配，`usage_missing=false`；临时 Key 已删除并确认不存在。

现有 `for-external-cache-new`、`for-myself`、`for-everyone` 保持自动全号池、首选 `credit_first`、低负载优先覆盖关闭；新账号可以参与余额优先路由。本次没有修改这些 Key，也没有创建专属优先组或强制绑定新账号。

正常 CLI 凭据和脱敏核验记录保留在独立目录 `~/.local/share/kiro-social-onboard/gj4vu845-google-20261031`，SQLite 权限为 `0600`。原默认 Kiro CLI 认证记录的哈希前后相同。任务拥有的浏览器、临时浏览器资料、登录代理客户端及截图已清理，生产新加坡代理保持运行。本文不记录密码、验证码、设备码或令牌。
