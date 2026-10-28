# Kiro / Outlook / AWS 批处理脱敏源码包

这是原批处理的脱敏源码快照，附齐本地辅助脚本，不含真实账号、密码、MFA 密钥、令牌、台账、日志或导入回执。运行环境和服务端接口仍须使用者自行准备；不是免配置的一键工具。打包只做离线检查，未替接收者做真实账号端到端验证。

## 包含的流程

Outlook 改密与新密码验证 → AWS 安全设置 → 改 AWS 密码 → 删除全部活跃会话 → 新密码及 MFA 独立复登 → 删除已确认的旧 MFA → Kiro CLI 设备授权 → Prism 导入与真实模型验证 → 私密台账和凭据备份。

正常成功路径有两次完整 AWS 登录（清理会话前后）；AWS 在敏感操作中要求的再次认证、失败重试仍可能增加认证次数。原来只有一个 MFA 时添加新设备，独立复登验证后才删旧设备；原来两个 MFA 时保留。Outlook 额外验证受阻且仍能读取邮件时，保留 Outlook 原密码，走 AWS 恢复邮箱绑定。无法读取原邮箱时并不保证能继续。

每成功完成两个账号，在下一个待处理账号开始前切节点。自动识别 PROXY 的 aws-sg，或悠兔的专线香港；记录切换前后出口 IP。重启进程计数归零；失败、跳过不计数。没有 Clash Party 时先设置 KIRO_CLASH_ROTATE=0。

## 接收者必须自行准备

- macOS、Node.js、Python 3、curl、Google Chrome、可用的 kiro-cli。后端用 macOS Keychain 和本机 Kiro SQLite，不能直接在 Windows/Linux 原样跑。
- 可访问的 HTTP 代理；自动切换要求 Clash Party/mihomo 的 Unix 控制 socket，且有对应选择器。
- 与本包接口兼容的 Prism 管理端和自己的管理密钥。普通 AWS API 无法替代 Prism 的导入、校验接口。
- 恢复邮箱服务：自己的 HTTPS API 与管理员令牌。仅填一个 SMTP/Outlook 地址不够。
- 自己获授权的账号、独立 Outlook/AWS 密码，以及已有 AWS TOTP seed。验证码需人完成，不会绕过微软额外验证。

## 配置和运行

1. 在源码包目录安装依赖：`npm install`。Chrome 使用现有安装。
2. 在仓库外创建私密 BATCH_RUN_DIR（权限 700）。根据两个 example.json 填写该目录的 accounts.json 与 mfa-secrets.json（权限 600），不要把真实数据写回分享包。密钥字段必须是真实 Base32 seed。保留原密钥的独立备份。
3. 若没有既有台账，把 ledger.example.md 复制到私密目录。新密码和邮箱 API token 分别存进权限 600 的文件。不要覆盖已有台账。
4. 复制并私密编辑 env.example.sh，设置真实绝对路径和自己的服务地址、管理密钥，然后 `source /absolute/private/env.sh`。不要把这份已填配置转发给别人。
5. 关闭 Kiro 桌面守护进程；确认 `kiro-cli whoami` 为 Not logged in。本机同时只运行一个批次，不能和自己的 Kiro 登录共用凭据。禁止用 kiro-cli logout 清理已导入令牌。
6. `python3 preflight.py` 分页查重；有重复则停止人工核对。然后 `node sync-ledger.cjs` 记录原始凭据，最后 `node run.cjs` 开始处理。打开的 Chrome 保持可见，Security check 由人完成。

私密目录会生成 states.json、replacement-mfa.json 及独立备份、import-run/、密码回执、错误回执等；这些都含敏感信息，不要再次打包。ACCOUNT_LEDGER_FILE 也会保存密码和 seed。

## 服务接口约定

Prism：Bearer 管理密钥；GET /admin/v1/accounts?limit=200&cursor=...（items、nextCursor），GET /admin/v1/accounts/:id；POST /admin/v1/accounts/import（method=json、items.document、refreshBeforeImport）；POST /admin/v1/accounts/:id/actions/verify（model）。结构见 scripts/kiro_builder_backend.py。

恢复邮箱服务：Bearer 管理令牌；POST /aliases/batch，入参 account_ids，返回 aliases（account_id、alias_email、status=active）；GET /mailboxes/:email/messages?limit=20，返回 messages，包含 sender、subject、received_at、recipient、body_text/body_html。OTP 必须满足发件人、收件人及新鲜度检查。结构见 run.cjs 和 scripts/kiro_builder_security.cjs。

## 断点和限制

已 verified / failed 的账号默认跳过。失败重试前必须核对对应回执和远端状态，只将已确认可重试的指定账号 failed 标志清除，保留其他阶段回执；不能删除整份状态重来。已提交但不确定的改密禁止重复提交；已导入但模型验证失败不能算成功。可使用 backend.py 的 verify-import / import-saved 子命令单独核对，文件实际名是 scripts/kiro_builder_backend.py。

创建 BATCH_RUN_DIR/stop-after-current 可请求当前账号结束后停下；不需要关闭认证窗口。账号页面和账号间浏览器上下文会被关闭，凭据成功归档后只清本地登录，不撤销远端刷新令牌。

本包仍是持续调试中的流程快照：上游页面变化、风控、超时、恢复邮箱 API 差异、提交结果不明等需要人工核对。不要把“具备所有源文件”理解为接收方环境已验证可用。
