# Kiro Google Pro+ 账号导入记录（2026-10-19）

核验时间：**2026-10-19 20:37:36，UTC+08:00**。用户要求将新注册并购买 Pro+ 的 Google 账号导入 llm-access，并固定使用新加坡代理。

| 项目 | 已核验结果 |
| --- | --- |
| 网关账号 | `kiro-ackingliu-google-social` |
| Google / Kiro 邮箱 | `ackingliu@gmail.com` |
| 认证方式 | `social` / `google`；独立目录中的 `kiro-cli whoami` 身份一致 |
| 套餐 | `KIRO PRO+` |
| 总额度 | 2000.00 credits |
| 实际请求验证后的剩余额度 | **1999.92 credits**，已用 0.08 credits |
| 代理模式 | `fixed` |
| 生效代理 | `aws_sg5`（新加坡） |
| 代理配置 ID | `llm-proxy-77b182943a884d589ff5aa3456e3bd31` |
| 已验证代理出口 | `<redacted-sg-proxy-ip>`；出口查询返回 `country=SG`、`city=Singapore` |
| 并发上限 | 3 |
| RPM 上限 | 5 |
| 最小启动间隔 | 446 ms |
| 最低剩余额度阈值 | 10 credits |
| 账号状态 | 已启用，`issue_kind` 为空，余额缓存 `ready` |

通过用户在 Google 已登录浏览器中完成 Kiro 设备授权取得凭据；本次流程未收集 Google 密码。认证写入独立 Kiro CLI 数据目录 `/home/ts_user/.local/share/kiro-social-onboard/ackingliu-google-pro-plus`，SQLite 权限为 `0600`。本机原有默认 Kiro CLI 身份已另行核对，保持原账号。本文不保存令牌、授权代码或验证码。

导入前完整分页检查了 354 个现有账号，确认目标账号名与已验证邮箱未重复；导入后的余额核验进一步确认上游身份未重复。使用 [Kiro 社交登录技能](../skills/kiro-social-onboarder/SKILL.md) 和 [本地账号导入技能](../skills/kiro-local-account-importer/SKILL.md) 的认证、解析、导入及余额核验流程；按用户明确要求，代理候选固定为已验证的新加坡节点 `aws_sg5`。创建后先设置固定代理，再刷新余额，未套用学生套餐校验。

实际请求使用仅固定到此账号的临时私有 Key，关闭 Anthropic 上游池及策略回退；公网 `/v1/messages` 请求 `claude-sonnet-5.5` 返回 HTTP **200**、`KIRO_OK` 和 `stop_reason=end_turn`。当前生产 journal 中唯一对应请求为 `llm-usage-33ce39fb-ef83-400b-a32a-ff7313ae18a8`，账号为 `kiro-ackingliu-google-social`，输入 10 / 输出 2 tokens，`usage_missing=false`。

账号固定代理配置与实际新加坡节点出口均已验证。journal 预览接口提供账号归属，不包含代理字段；代理证据来自复核后的生效固定绑定和该节点的真实出站查询。历史用量入库是异步流程，本次直接核对当前生产 journal，未重复发送验证请求。临时 Key 已删除，并通过通用 Key 查询接口确认不存在。验证后再次刷新余额，结果如上表。
