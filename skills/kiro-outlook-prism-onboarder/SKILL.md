---
name: kiro-outlook-prism-onboarder
description: "Configure, inspect, and resume the supplied macOS Outlook/AWS Builder ID security and Kiro CLI-to-Prism batch workflow. Use when the user selects this bundled full workflow or its standalone tools; route ordinary StaticFlow llm-access or IDE onboarding to aws-builder-id-kiro-account-workflow."
---

# Kiro Outlook / AWS / Prism 批处理

本 skill 保存用户提供的 `kiro-onboarding-share-20261028.zip` 脱敏源码快照。
[scripts/batch/](scripts/batch/) 内 19 个原文件保持原样，包括依赖声明、配置示例和辅助脚本。
先读随包 [README.md](scripts/batch/README.md)，按实际任务检查下面对应的入口；
这里的封装和离线验证不代表接收环境已完成真实账号端到端验证。

## 选择适用流程

- macOS 上需要 Outlook、AWS 安全维护和 Kiro CLI → Prism 的整套批处理时，使用本包。
- 本项目 Linux/WSL 的 llm-access 导入或 IDE 优先认证，使用
  [aws-builder-id-kiro-account-workflow](../aws-builder-id-kiro-account-workflow/SKILL.md)。
  本包的 Prism `/admin/v1/accounts`、Bearer 密钥和验证回执与 llm-access 接口不同。
- 只检查、配置或保存本 skill 时，停在相应工作；不要顺便运行账号流程。

## 明确本轮执行范围

沿用会话里已经明确的账号、密码选择和步骤授权，不重复询问已有授权。
登录或导入本身不包含 Outlook 改密、AWS 会话全删、恢复邮箱变更、新 MFA 注册或旧 MFA 删除。
将密码和 seed 写入明文台账也须属于用户选定的凭据留存方式；已有留存授权可直接沿用。
范围不清时一次问清，继续不依赖答案的配置检查。

完整 [run.cjs](scripts/batch/run.cjs) 固定串联 Outlook 改密、必要时绑定恢复邮箱、
AWS 改密、清理全部会话、独立复登、条件式 MFA 更换、CLI 设备授权和 Prism 导入验证。
它没有任意跳过安全步骤的开关：只有一个 MFA 时会注册替代设备并在验证后删旧设备，
已有两个 MFA 时保留。若用户要求保留原 MFA 或只做部分步骤，选择匹配的独立入口或先调整流程，
不能直接运行完整入口。

完整入口在失败后标记并继续队列，且会关闭该账号的浏览器上下文；结束时关闭自身浏览器。
用户要求遇错暂停、保留窗口时，使用支持交接的单账号入口或先调整这些行为。
验证码和微软设备确认按当前授权处理，需要用户设备或不支持的验证时请求具体操作。
原包没有全局逐动作间隔开关。用户要求每步停顿 1–2 秒时，先在所选入口落实逐步等待和
页面就绪检查，不能用账号之间的停顿代替，也不将原始快照视为已经满足该要求。

## 配置与运行

本项目当前登录连接规则为每三个账号交替：第 1–3 个直连、第 4–6 个使用
`http://127.0.0.1:11115`，第 7–9 个直连，之后循环。按本批输入序号计算，
失败或续跑不会重排账号或重置序号。可用
[login-network-policy.cjs](scripts/login-network-policy.cjs) 生成并核对配置。
这是登录连接切换；导入后的推理请求仍遵循目标网关的独立代理策略。

Linux / llm-access 批次可用 [outlook-cdp.cjs](scripts/outlook-cdp.cjs) 复用随包 Outlook
单项功能，在当前账号的独立 Chrome 上连接 CDP，逐次操作后等待至少 2 秒，
并在新上下文中实际输入新密码验证。凭据仅经关闭回显的私密 stdin 传入；
先完成邮箱改密，再复用同一 Chrome 衔接 Builder ID IDE 授权与 llm-access 验证。
Microsoft 所需的额外验证单独处理，不使用 AWS TOTP 代答。
本批用户已明确：若 Outlook 需要额外设备验证且改密尚未提交，保留原邮箱密码、记录跳过原因，
继续 AWS/Kiro 其他步骤，无需逐个询问。适配器的 `skip-microsoft-verification` 仅在该状态下退出，
已有提交回执时拒绝跳过；台账分别统计实际改密与经授权保留原密码。
用户要求处理微软设备验证时可保留其标签页，在同一账号的新标签页先完成已授权的 AWS/Kiro 步骤；
恢复时选择保留的 Microsoft 页，台账区分邮箱已改密、经授权跳过和待验证。
若“Stay signed in”页三次点击后仍未推进，适配器等待页面稳定后返回失败。
仅在身份匹配、尚无改密提交回执时，在同一窗口重新进入 My Account 续跑；
保存累计次数，最多三次，不重建账号窗口或切换连接。已有 submitted 回执时只验证新密码。

运行环境须有 macOS、Node.js、Python 3、curl、Google Chrome、kiro-cli，
以及兼容的 Prism 管理端和恢复邮箱 API。后端固定使用 macOS Keychain 和
`~/Library/Application Support/kiro-cli/data.sqlite3`；使用已获授权的独占 CLI 环境，
先核对现有登录和桌面守护进程，保护其他账号。不要设置另一个 HOME 或用 `kiro-cli logout` 切号。

1. 在仓库外创建绝对路径的 `BATCH_RUN_DIR`（0700）。从
   [accounts.example.json](scripts/batch/accounts.example.json)、
   [mfa-secrets.example.json](scripts/batch/mfa-secrets.example.json) 准备私密输入（0600），
   将 [ledger.example.md](scripts/batch/ledger.example.md) 用于新台账，保留已有台账和原密钥备份。
2. 在仓库外编辑 [env.example.sh](scripts/batch/env.example.sh) 的副本。
   `MAILBOX_PASSWORD_FILE` 提供完整入口同时用于 Outlook 和 AWS 的目标密码；不同目标需要调整流程。
   按 README 配置自己的 Prism、恢复邮箱 API 和凭据，检查示例占位值已替换，避免打印配置。
   台账会保存明文密码和 seed，台账、日志、回执、备份都只能留在私密目录，不能写入本项目文档。
3. 原始整包源码仍内置每成功两个账号切一次 Clash 节点，计数在进程重启时归零。
   用户未选择该策略、没有匹配的 Clash Party socket，或已有固定/每五个切换规则时，
   在私密配置设置 `KIRO_CLASH_ROTATE=0`，按用户实际规则管理已授权的代理。
   本项目当前三账号交替规则也应关闭该内置轮换，使用上述独立的登录连接策略。
4. 在本 skill 的 `scripts/batch/` 目录安装声明的依赖：`npm install --ignore-scripts`。
   原包固定 Playwright 1.60.0，使用系统 Chrome；本次保存不代表依赖或平台已安装。
5. 仅在完整流程已获授权、配置与独占环境检查完成后执行：

   ```bash
   # 工作目录：本 skill 的 scripts/batch/
   source /absolute/private/env.sh
   export KIRO_RUN_DIR="$BATCH_RUN_DIR/import-run"
   python3 preflight.py
   node sync-ledger.cjs
   node run.cjs
   ```

`preflight.py` 会联网完整分页检查 Prism 重复邮箱；`sync-ledger.cjs` 会写入私密凭据台账；
`run.cjs` 会真实修改账号。执行前检查包里的固定验证模型 `claude-sonnet-4-6` 与目标服务兼容。
显式 `KIRO_LOOKUP_SSH` 会经 SSH 读取 Prism 状态库，`KIRO_API_TUNNEL_PORT` 使用已建立的 HTTPS 转发；
这些配置只用于用户指定的服务，不自动创建隧道或扩大主机操作范围。

## 单项入口与续跑

- [kiro_builder_security.cjs](scripts/batch/scripts/kiro_builder_security.cjs)：AWS 安全维护。
  `AWS_SECURITY_SCOPE=factors` 仍可能改恢复邮箱和 MFA，不等于保留现状；`--otp` 会打印动态码，
  不用于常规诊断或共享日志。
- [outlook_change_password.cjs](scripts/batch/scripts/outlook_change_password.cjs)：Outlook 改密与新会话验证；
  [kiro_change_builder_email.cjs](scripts/batch/scripts/kiro_change_builder_email.cjs)：恢复邮箱流程。
- [kiro_builder_assisted.cjs](scripts/batch/scripts/kiro_builder_assisted.cjs)：可见 Chrome 中的 CLI 设备授权；
  [kiro_builder_backend.py](scripts/batch/scripts/kiro_builder_backend.py)：身份读取、导入、模型验证与本地凭据归档。
  使用单项工具前读该入口实际配置；部分 `--help` 中提及的历史 `.md` 未包含在原包内，
  配置和接口以随包 README 与源码为准。

已 verified / failed 的账号默认跳过，`RETRY_EMAILS` 只筛选邮箱，不会清除 failed 标志。
重试前核对该邮箱的 states、改密回执、导入回执和远端实际状态，只解除已确认可重试的失败状态，
保留成功阶段和次数。已提交但结果不明的改密不能重新提交；已导入、模型验证失败不能重建成另一个账号。
用 `verify-import EMAIL ID` 验证已有导入；`import-saved EMAIL ARCHIVE` 须先检查归档身份。
这些后端子命令仍可能联网写入或发起计费模型请求。

会话删除与旧 MFA 删除分别以结果和独立新密码/新 MFA 复登证明为准。
后端 `detach` / `preserve-failed` 会备份后清理匹配账号的本机 SQLite 与 Keychain，
并非只断开浏览器；确认账号匹配和归档完整后才能执行，不能撤销远端刷新令牌。
`stop-after-current` 文件用于请求当前账号结束后停止，不代替不确定提交后的状态核对。

完成报告区分 Outlook、AWS 密码/MFA/恢复邮箱、CLI 身份、Prism 导入 ID 和
`verification.success === true` 的实际模型验证结果。Prism 单账号验证不能代替
llm-access 公网 Key 路由与 Usage 归属证明。汇报时引用脱敏状态，不复制私密台账内容。

## 离线验证

保存或检查本包时，仅做 metadata、JSON、JavaScript 语法和 Python AST 检查，
以及明确隔离的本地台账样例验证。不要用 `run.cjs`、`preflight.py` 或后端命令作为语法检查。
原始整包的 Keychain / Prism 后端仍只适用于其 macOS 环境；Linux CDP 单项适配与
llm-access / IDE 衔接分别验证，不将其中一个入口的结果当作整包已通过验证。
