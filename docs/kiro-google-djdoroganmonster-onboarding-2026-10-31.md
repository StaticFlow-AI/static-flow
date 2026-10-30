# Kiro Google POWER 账号与余额优先路由记录（2026-10-31）

最终核验时间：2026-10-31 03:15:34，UTC+08:00。

| 项目 | 已核验结果 |
| --- | --- |
| 邮箱 | `djdoroganmonster@gmail.com` |
| 网关账号 | `kiro-djdoroganmonster-google-social` |
| 登录与身份 | 独立 Kiro CLI 设备授权；`whoami` 为对应邮箱、Google 登录 |
| 认证方式 | `social / google` |
| 套餐与额度 | `KIRO POWER`，上限 10,000 credits；导入校验时剩余 9,999.87 |
| 状态 | 已启用 |
| 并发上限 | 10 |
| RPM 上限 | 5 |
| 最小启动间隔 | 425 ms |
| 最低剩余额度 | 10 credits |
| 账号调度池 | `credit_first`（余额优先） |
| 固定代理 | `aws_sg5`，`llm-proxy-77b182943a884d589ff5aa3456e3bd31` |
| 实际新加坡出口 | `<redacted-sg-proxy-ip>`，SG / Singapore |

Google 图片验证及用户手机确认后，保留已登录的同账号浏览器，为明确过期的 Kiro 设备码重新签发一次授权，再取得社交登录凭据。CLI 凭据保存在独立目录 `~/.local/share/kiro-social-onboard/djdoroganmonster-google-20261031`，SQLite 权限为 `0600`；原默认 Kiro CLI 认证记录的哈希前后相同。未修改 Google 密码。本文不保存密码、令牌、设备码或验证码。

新加坡验证在生产主机上通过实际运行的 `http-proxy-cli :11117` 出站，核对其远端地址和强制代理配置，并通过代理查询真实出口及该出口的地理位置，确认 SG。账号固定代理配置已回读，余额刷新正常。独占该账号的临时 Key 经公网 `/v1/messages` 请求 `claude-sonnet-5.5`，返回 HTTP 200、`KIRO_OK`、`end_turn`；Usage 为 `llm-usage-32fcb705-62ab-47c0-9c06-7008a1b53f5e`，账号归属匹配。临时测试 Key 已删除并确认不存在。

## 活跃 Key 的最终配置

完整读取 43 个 Kiro Key，其中 31 个启用；按控制面 `last_used_at` 选择过去 24 小时内使用过的 3 个启用 Key。最终意图是余额较多的账号更容易被选中，同时允许旧账号参与，采用现有余额优先规则，不绑定新账号。

| Key | 首选调度池 | 低负载优先覆盖 | 账号范围与模型组 |
| --- | --- | --- | --- |
| `for-external-cache-new` | `credit_first` | 关闭 | 自动全号池，原模型组配置保留 |
| `for-myself` | `credit_first` | 关闭 | 自动全号池，原模型组配置保留 |
| `for-everyone` | `credit_first` | 关闭 | 自动全号池，原模型组配置保留 |

低负载排序原本排在池策略前，会使池内余额排序难以发挥作用；关闭该覆盖后使用池策略，同时保留会话亲和、可用性和容量约束。账号专属的临时优先组已撤回并删除，未保留按模型强制偏向此号的规则。原模型映射、专线、配额、保护检查和后备链均保留。当前规则是优先排序，未新增按余额比例随机抽样功能。

最终使用各生产 Key 的自动路由发送普通算术请求，均返回 HTTP 200、`42`、`end_turn`，对应 Usage 的 Key 和实际账号均已核对；未将旧账号被选中视为失败：

| Key | 实际账号 | Usage ID |
| --- | --- | --- |
| `for-external-cache-new` | `kiro-varel-robert-github-social` | `llm-usage-6c23987a-675a-452c-b68b-865fe3b8cfe7` |
| `for-myself` | `kiro-inayah-safitri-builder-id` | `llm-usage-41e0cd8e-a611-4136-8088-a3d3e404cc21` |
| `for-everyone` | `kiro-immanuel-siboro-builder-id` | `llm-usage-d5da9fb5-14af-40b4-bd5e-082875112742` |

验证中的另一项观察：`for-external-cache-new` 的原有 Thinking Guard 已启用，`KIRO_OK` 标识命中运行时关键词 `kiro`；两次此类请求返回 HTTP 504，Usage 未记录被选账号。此次未停用或修改保护配置；最终以普通算术请求核对正常推理与路由，不据此宣称前置思考检查健康。相关失败与成功证据保留在私有任务目录。

本次拥有的浏览器、登录代理客户端及临时图片已清理，CLI 正常凭据与脱敏核验记录保留。生产新加坡代理、其他浏览器、默认 CLI 身份及未选中的 Key 保持原状态。
