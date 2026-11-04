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

## 真实流量未命中新账号的排查与修正（2026-11-04）

用户反馈仍没有请求进入本账号，并确认检查当前请求最多的 Key。近期用量确认该 Key 为 **`for-external-cache-new`**（`kiro-key-aafeab48def04996a55c166046e7dc65`）。变更前分页采集的 291 条实际请求中，本账号命中数为 0；路由诊断分别为 215 次 `warm_prefix_selected`、73 次 `affinity_reused`、3 次 `new_binding`。

此前余额优先配置已正确保存，但前缀缓存和会话亲和性选择先于冷请求的账号池、余额排序，关闭低负载优先并不会关闭这些缓存规则。此外，冷选择中的会话绑定数量比较也先于账号池排序。此前新建临时 Key 的成功测试只证明冷请求可以选中新账号，没有覆盖这个真实 Key 的缓存流量，因此不足以证明实际流量会转移。

使用现有配置完成修正，未修改调度源码或重启服务。新建账号组 `priority-ssmerommagdalena558`，ID 为 `kiro-group-33c4c9cddedf4355a70391fb49faae66`，成员仅为 `kiro-ssmerommagdalena558-google-social`。仅在 `for-external-cache-new` 的 `kiro_model_group_preferences` 中，将当前实际使用的下列 5 个模型映射到该组：

| 模型 | 首选账号组 |
| --- | --- |
| `claude-opus-4-7` | `priority-ssmerommagdalena558` |
| `claude-opus-4-8` | `priority-ssmerommagdalena558` |
| `claude-opus-5.5` | `priority-ssmerommagdalena558` |
| `claude-sonnet-5` | `priority-ssmerommagdalena558` |
| `claude-sonnet-5.5` | `priority-ssmerommagdalena558` |

模型账号组偏好会先约束缓存和会话亲和性的首选账号，再参与冷请求排序，使其他账号的旧缓存不再抢先选中。该 Key 仍为全账号范围的 `auto` 路由；首选账号不可用时仍可选择其他账号，原有策略回退和外部上游回退配置保留。完整比较 API 配置，除该 Key 的模型账号组偏好外，其余配置及另外 43 个 Kiro Key 均未变更。模型映射按精确名称生效，未来新增模型需另行配置，不能视为所有模型的通配偏好。

服务端 Key 更新时间为 **2026-11-04 18:43:03.347，UTC+08:00**。验证时发现云端时钟比本机快约 55 秒，因此以服务端 `updated_at` 为变更边界，并用事件完成时间减去请求耗时筛除变更前已启动的请求。合并实时 journal 与已持久化 Usage、按事件 ID 去重后，已核实变更后 **29 条真实请求**进入本账号：19 条 `claude-sonnet-5.5`、7 条 `claude-sonnet-5`、3 条 `claude-opus-5.5`，全部 HTTP 200，`usage_missing=false`。这是可核实样本数，journal 会轮转、持久化存在延迟，不作为整个时间段的完整请求总数。

| 模型 | 真实 Key 的成功请求示例 |
| --- | --- |
| `claude-opus-5.5` | `llm-usage-ae8d9d67-e1b7-4a34-82f6-fc66a96cd994` |
| `claude-sonnet-5.5` | `llm-usage-0d000c4b-6b35-460f-ae7b-3ef341993355` |
| `claude-sonnet-5` | `llm-usage-7b7551e4-029d-4f3c-ac59-ddc4e844a31d` |

其中 5 条已持久化事件的路由诊断包含 2 次 `warm_prefix_selected`、1 次 `affinity_reused`、2 次 `new_binding`，所选账号均为本账号。其余样本来自实时 journal，未使用新的临时 Key 代替真实流量验证。复核账号仍固定使用新加坡 `aws_sg5`，并发 100、RPM 6000、最小启动间隔 0、最低余额门槛 0、手动额度上限 20,000 均保留；余额缓存当时报告已用 50、剩余 19,950，此数仍受上游报告准确性限制。

## 积分核对与回退排查（2026-11-04）

截至 **23:20:23，UTC+08:00**，逐笔核对从 **18:40:00** 起记录到网关的 3,914 条该账号请求，并补入此前一次自动路由验证的计量记录，确认本账号已消耗 **至少 1,678.8963 credits**。其中 211 条记录的积分未能完整计量或归属，包含 19 条跨账号聚合请求；没有把这些聚合请求的全部积分算到本账号。原始导入验证请求对应的归档文件缺失，其积分也未计入。该数字因此是已知下限，并不包含无法核对的更早历史或其他客户端的使用。

按网关手动额度 20,000 计算，已知消耗对应的剩余额度至多为 **18,321.1037 credits**。上游余额接口刷新后仍报告已用 50，原实现只是用手动上限减去该上游值，因此页面仍显示剩余 19,950，无法反映已确认的请求消耗。全时段 Usage 汇总值与逐笔记录存在差异，本次使用账号、时间范围和事件 ID 核对逐笔计量；缺失计量没有按零消耗处理。

已在本地实现按账号持久化累计积分，并让手动额度随已知消耗扣减；缺少计量和未回填历史时，界面将已用标为下限、剩余标为上限。上游原始用量仍单独保留，普通账号继续使用上游余额。日志与回放格式、幂等更新和历史积分回填的操作说明见 `deps/llm-access/docs/kiro-manual-credit-budgets.md`。**本次代码尚未部署，生产余额仍是旧算法；历史回填也尚未执行。**

本地前端构建一度被运行中的 Vite 预览服务直接提供，导致新文案提前可见；**23:33，UTC+08:00** 已恢复并验证此前 `d640cf9` 的前端构建，未重启服务。后端和历史积分均未在此期间更新。新前端构建保留在私有临时备份，等待协调升级。

关于用户反馈的回退，云端原始日志在 **22:43—22:56，UTC+08:00** 多次记录该账号触发 `USER_REQUEST_RATE_EXCEEDED`，单次账号冷却 30 秒。此前生产诊断将部分带重试时间的上游限流错误放进 `aws_bedrock_auth_refresh_failure` 字段，不能据该字段名称认定认证刷新失败；本地代码已补充来源区分，待部署后生效。

该 Key 的一组 69 条已持久化样本均成功完成，20 条使用本账号，19 条在首选模型上因冷却或失败切换到其他 Kiro 账号，30 条使用当时尚未配置新账号偏好的模型（`claude-opus-4-6`、`claude-opus-5` 或 `claude-opus-5.5-thinking-high`）。这组样本不代表完整实时流量。

后续真实记录确认本账号已成功处理 `claude-opus-5.5-thinking-high`（`llm-usage-89cc9535-59ba-41bb-90be-c0cf09f28ade`）与 `claude-haiku-4-5-20251001`（`llm-usage-1f40c64a-4aa4-4d94-8cf2-bb42513554fd`）。已准备将这两个精确模型名补入该 Key 的现有首选组；计划尚未应用，应在实际调整前重新核对账号限流和 Key 配置。`claude-opus-4-6`、`claude-opus-5` 尚无本账号成功记录，本轮未为它们强制增加偏好或主动施加探测流量。

积分核对窗口内另有 190 条 HTTP 403。抽查其中 3 条（时间横跨该窗口），记录显示上游拒绝了请求并给出审核标识；这一抽样不能证明其余 403 的具体原因，也不能单凭此认定账号被禁用。本账号在相关时期及之后仍有成功请求，现有审核策略保持生效。

为降低上游限流，已于 **23:05:24，UTC+08:00** 将该账号设置为 **30 RPM、最小启动间隔 2,000 ms**；并发上限 100、新加坡固定节点 `aws_sg5` 和手动上限 20,000 保留。**23:06—23:12** 的有限实时 journal 样本中，14 条该 Key 请求均为 HTTP 200，其中 11 条使用本账号；该时段未观察到本账号的上游频率冷却日志。调整前临近时段也没有新的同类日志，因此这些样本尚不足以证明调速本身解决了限流。
