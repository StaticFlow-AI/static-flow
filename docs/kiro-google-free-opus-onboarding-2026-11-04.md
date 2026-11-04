# Google Kiro Free / Opus 账号导入记录（2026-11-04）

核验时间：**2026-11-04 16:22:35，UTC+08:00**。用户要求通过 Mac 上的 Google 登录状态完成远程 Kiro 授权，将新账号导入网关，使用新加坡节点、提高并发，并将网关手动额度设为 20,000；用户明确说明 Free 套餐可使用 Opus，上游积分显示不准确。

| 项目 | 已核验结果 |
| --- | --- |
| 网关账号 | `kiro-ssmerommagdalena558-google-social` |
| 登录邮箱 | `ssmerommagdalena558@gmail.com` |
| 认证方式 | `social` / `google`；隔离目录的 `kiro-cli whoami` 已确认邮箱 |
| 实际上游套餐 | `KIRO FREE` |
| 固定代理 | `aws_sg5`（新加坡） |
| 代理配置 ID | `llm-proxy-77b182943a884d589ff5aa3456e3bd31` |
| 实际代理出口 | `<redacted-sg-proxy-ip>`，出站查询确认 `country=SG` |
| 并发上限 | 100 |
| RPM 上限 | 6000 |
| 最小启动间隔 | 0 ms |
| 最低剩余额度阈值 | 0 credits |
| 手动额度上限 | **20000.00 credits** |
| 验证后已用量 | 0.59 credits（上游报告值） |
| 验证后网关显示剩余 | 19999.41 credits |
| 原始上游额度上限 | 550.00 credits（保留原始值，不据此判断 Opus 是否可用） |
| 账号状态 | 已启用、无账号异常，余额缓存 `ready` |

`manual_usage_limit=20000` 是网关侧的校准额度上限，显示剩余按该上限减去上游报告的已用量计算；它不会修改 Kiro 上游套餐或余额，也不代表上游已用量因此变得准确。此次保留 `KIRO FREE` 套餐原值，按用户要求覆盖标准导入流程的学生套餐、美国节点、3 并发及最低余额门槛；并发上限按本轮说明设为 100，同时设置 RPM 6000、启动间隔 0 ms。

Google 设备授权在用户自己的 Mac 浏览器中完成，本次流程未收集 Google 密码。凭据写入独立 Kiro CLI 数据目录 `/home/ts_user/.local/share/kiro-social-onboard/google-free-opus-20261104T081740Z`，目录权限 `0700`、SQLite `0600`，未对默认 CLI 执行 logout 或清理。导入前完整分页检查了 447 个现有账号，确认账号名和已验证邮箱不重复；导入后通过余额中的上游身份进一步排除重复账号。本文不保存令牌、授权代码、密码或验证码。

实际公网验证请求使用仅固定到此账号的临时私有 Key，禁用 Anthropic 上游池和策略模型回退：`/v1/messages` 请求 `claude-opus-5.5` 返回 HTTP **200**、`KIRO_OPUS_OK` 和 `stop_reason=end_turn`。用量记录已匹配到此账号，`usage_missing=false`；记录 ID 为 `llm-usage-e343c738-7d3d-4215-b8b1-523bb04236c0`。因此本轮实际验证了该 Free 账号可以完成原生 Opus 请求，并非仅依据套餐名称或手动积分推断。

并发 100 是已保存的账号上限，本轮未做 100 路并发压测。临时验证 Key 已删除并通过通用 Key 查询确认不存在；请求后重新刷新余额、复核账号和新加坡固定绑定，结果如上表。代理位置证据来自该节点的真实出站查询及生效固定绑定；journal 预览中的账号归属用于核对实际请求。临时本地登录通道在本轮完成后关闭，云端正式代理及账号绑定保留。

## 后续调度调整（2026-11-04）

完成时间：**2026-11-04 18:16:21，UTC+08:00**。按用户要求，将本账号的 `pool_strategy` 从 `balanced` 改为 `credit_first`，加入余额优先账号池；这是本次调整时唯一的 `credit_first` 账号。

根据 Key 原有账号组、固定账号或自动账号范围，选出候选账号包含本账号的 **31 个 Kiro Key**，统一设置 `preferred_pool_strategy=credit_first`、`kiro_low_load_routing_enabled=false`。其中 28 个 Key 原先开启低负载优先；实际调度源码中，低负载排序先于账号池和余额排序，因此将其关闭，使余额优先生效。

涉及 Key 中 22 个启用、9 个停用，状态保持不变。完整比较变更前后的 API 配置，除上述两个调度字段外，Key 凭据、额度、账号范围、模型设置、图路由和回退等配置均未变更；另外 13 个原账号范围不包含本账号的 Key 均未变更。账号的新加坡绑定、并发 100、RPM 6000、启动间隔 0、手动额度上限 20,000 保持原值。

使用新建的临时私有 Key 验证自动选择：`route_strategy=auto`、首选池 `credit_first`、关闭低负载优先、范围为全部 Kiro 账号，未固定到某个账号或限定单账号列表。原生 `claude-opus-5.5` 公网请求返回 HTTP 200、`KIRO_PRIORITY_OK`、`stop_reason=end_turn`，用量记录确认实际选择 `kiro-ssmerommagdalena558-google-social` 且 `usage_missing=false`。临时验证 Key 已删除并确认不存在。优先策略仍受账号健康、容量和既有会话亲和性等正常调度条件约束。

本次调整的 Key：

| Key 名称 | 原有状态 |
| --- | --- |
| `for-jye-tmp` | `active` |
| `for-tuye-tmp` | `active` |
| `for-laohan-tmp` | `active` |
| `for-maye-tmp` | `disabled` |
| `release-canary-kiro-empty-20260920` | `disabled` |
| `for-qiushen-backup` | `active` |
| `for-everyone` | `active` |
| `for-chaowei-claude` | `active` |
| `for-yl-tmp` | `active` |
| `for-tuye-new` | `active` |
| `my-ci-bot` | `active` |
| `for-dog-jiye` | `active` |
| `for-myself` | `active` |
| `new-admin` | `disabled` |
| `for-external-cache-new` | `active` |
| `for-myself-fanyi` | `active` |
| `gaohaocheng-ziyong` | `disabled` |
| `for-panpan-kiro` | `active` |
| `for-dousha-dev` | `active` |
| `for-yl-kiro` | `disabled` |
| `for-tuye-claude` | `active` |
| `for-maye-claude` | `disabled` |
| `for-ziyan` | `disabled` |
| `for-kuankuan-kiro` | `active` |
| `for-gaohancheng3` | `active` |
| `for-gaohancheng2` | `active` |
| `for-gaohancheng1` | `active` |
| `for-gaohansheng` | `disabled` |
| `for-huye` | `active` |
| `for-external-cache` | `disabled` |
| `admin` | `active` |
