# Kiro 认证异常与重登清单（更新至 2026-11-02）

## 2026-11-02 最新复核与待处理清单

复核时间：**2026-11-02T12:49:09+08:00**；首次读取为 2026-11-02 12:22:57（UTC+08:00）。两次均完整读取 **200 + 200 + 47 = 447** 个唯一账号；期间账号身份、认证异常、启停及调度设置一致。

**相比 2026-10-31，新增 401 为 0，解除 401 为 0。** 当前仍有 **168 个 401 + 2 个旧 IDC `invalid_grant` = 170 个认证异常账号**。上次提到的 **161 个**仅是 DO US 代理上的启用账号；全号池实际启用 **217 个**。

下载：[最新 170 个认证异常与待处理清单 CSV](kiro-relogin-current-2026-11-02.csv)。清单增加处理分组、登录方式、当前/历史邮箱、历史上号结论、旧尝试注意事项、建议下一步和本次快照时间；既有暂缓、忽略标记保持原样。历史 CSV 保留为比较基线，不覆盖当时的事实。

| 当前分类 | 数量 | 与 2026-10-31 比较 |
| --- | ---: | --- |
| 全部账号 | 447 | 无新增或删除 |
| 启用且无认证异常 | 217 | 原 219 个减少 2 个，均为本轮手动停用 |
| 401 Bad credentials | 168 | 名单完全一致，没有新增 |
| IDC refresh token 无效 | 2 | 仍为 `KitWilliam`、`laohan_MarleyMacdo` |
| 仅停用，无认证失效记录 | 60 | 原 58 个增加 2 个 |
| 停用合计 | 230 | 168 + 2 + 60，不能全部算需要重登 |

这两个新增停用账号是 `kiro-djdoroganmonster-google-social` 和 `kiro-gj4vu845-google-social`：前一轮排障已按用户要求因目标模型不可用而停用，当前 `issue_kind=disabled`，未记录新的 401，**不加入重登队列**。`for-kabi` 的停用和原有账号限定不因本次清单更新而改变。

217 个启用账号中：**DO US 161 个，AWS 美国代理 55 个，新加坡代理 1 个**。DO US 11 个代理各有 14–15 个启用账号。读取时 217 个账号的缓存状态全部为 `ready`，均有最近检查时间，最旧距复核约 **247 秒**；这是已有认证/余额检查的状态，不等于每个账号都具备所有模型和充足额度。

| 清单处理分组 | 数量 | 处理依据 |
| --- | ---: | --- |
| 优先重登，历史成功身份已核对 | 134 | 10 月 31 日那批 GitHub 账号，仍持续 401；不是今天新增 |
| 身份已知，先核对历史未完成尝试 | 5 | `laohan88`、`laohan91`、`laohan143`、`laohan144`、`laohan146` |
| 先确认登录身份 | 24 | 不依据账号名或相邻成功记录推断真实登录名 |
| IDC 重新授权 | 2 | 与 GitHub 401 分开处理 |
| 按用户要求暂缓 | 3 | GitHub 经 Google 登录的既有暂缓项 |
| 按用户要求忽略 | 2 | 原 Google 忽略项与失联账号；不进入执行队列 |

上次已恢复的 **99 个账号仍全部启用且无 401**。`laohan192` 的历史旁注只说明它未完成；相邻行的成功身份属于 `laohan193`，没有移用到 `laohan192`。`laohan91`、`laohan143`、`laohan144` 的历史改密状态未确认，已在清单内单独提示先核对现用凭据。

本次仅 GET 读取管理接口并整理项目清单，没有强制刷新、重登、改密、启停、代理或 Key 写入，也没有发起模型调用。401 表示凭据被拒绝，不能据此判定账号封禁；最近检查时间推进不作为新增 401 的依据。脱敏快照及核验结果保存在 `/home/ts_user/.local/share/kiro-account-audits/20261102-122256`。

## 2026-10-31 原始审计（以下保留历史口径）

核查时间：**2026-10-31T03:55:53+08:00**。完整分页读取线上账号：200 + 200 + 47 = **447** 个唯一账号；审计开始、结束各读一遍，账号、认证异常分类和主要调度配置一致。

### 核查结论

**本次新增 134 个认证失效账号，全部为此前成功上过号的 GitHub 账号。** 历史表中的真实 GitHub 用户名、恢复成功记录及邮箱均已逐项核对，134/134 与线上邮箱一致。

| 分类 | 数量 | 口径 |
| --- | ---: | --- |
| 本次新增 401 | 134 | 对比 2026-10-15 恢复完成后的状态；全部有明确历史成功证据 |
| 上轮遗留 401 | 34 | 含此前暂缓或未完成处理的账号，不重复算新增 |
| 当前 401 合计 | 168 | 均记录 Bad credentials；不能据此判定 GitHub 封禁 |
| 旧 IDC refresh token 无效 | 2 | KitWilliam、laohan_MarleyMacdo，仍为 invalid_grant |
| 当前认证异常合计 | 170 | 168 个 401 + 2 个旧 IDC 错误 |
| 仅被停用 | 58 | 无凭据失效证据，不计入需重登统计 |
| 已启用且无认证异常 | 219 | 仅指当前认证状态；不保证额度充足 |

上次 2026-10-12 至 10-15 恢复的 **99 个账号目前全部启用且无 401**，没有被误算为这次新增异常。上轮遗留的 34 个 401 也未被误算为本次新增。

### 新增异常的检查日期

以下为状态缓存中的最近失败检查日期（北京时间），不是可证明的首次失效时间。

| 最近失败检查日 | 数量 | 网关账号 |
| --- | ---: | --- |
| 2026-10-23 | 35 | `laohan39`、`laohan41`–`laohan74` |
| 2026-10-25 | 27 | `laohan77`–`laohan87`、`laohan89`–`laohan90`、`laohan92`–`laohan105` |
| 2026-10-26 | 38 | `laohan106`–`laohan128`、`laohan130`–`laohan142`、`laohan145`、`laohan147` |
| 2026-10-27 | 11 | `laohan148`–`laohan156`、`laohan159`–`laohan160` |
| 2026-10-29 | 12 | `laohan162`–`laohan173` |
| 2026-10-30 | 11 | `laohan174`–`laohan184` |

### 时间特征与判断

这 134 个账号的凭据导入时间为 2026-07-25T02:11:51+08:00 至 2026-08-01T19:19:52+08:00。最后成功检查距离上次导入为 **89.99598–89.99987 天**，失败检查为 **89.99898–90.00314 天**，均集中在约 90 天处。

因此，本批很像授权有效期到期后需要重新授权；这是基于导入与检查时间的推断，没有从官方规则或逐号重登确认 90 天是固定有效期。401 本身不能证明 GitHub 封禁，也不是额度耗尽的错误。

### 新增 134 个账号完整映射

下载：[新增重登清单 CSV](kiro-relogin-new-2026-10-31.csv)；[全部 170 个认证异常 CSV](kiro-auth-abnormal-2026-10-31.csv)。CSV 另含导入时间、最后成功时间、最近失败检查时间、生效代理及历史文档证据。

| 序号 | 网关账号 | 历史 GitHub 用户名 | 邮箱 |
| ---: | --- | --- | --- |
| 1 | `laohan39` | `Prosacco1` | `Prosacco@asu.edu` |
| 2 | `laohan41` | `Imeldaa1` | `Imeldaa@asu.edu` |
| 3 | `laohan42` | `Hagenes12` | `Hagenes@asu.edu` |
| 4 | `laohan43` | `Janessaa1` | `Janessaa@asu.edu` |
| 5 | `laohan44` | `Kuhlmannn1` | `Kuhlmannn@asu.edu` |
| 6 | `laohan45` | `Laurinez` | `Laurine@asu.edu` |
| 7 | `laohan46` | `Darlenexc` | `Darlenex@asu.edu` |
| 8 | `laohan47` | `Lehnerxz` | `Lehnerxz@asu.edu` |
| 9 | `laohan48` | `Lefflerxs` | `Lefflerxs@asu.edu` |
| 10 | `laohan49` | `Emoryyy1` | `Emoryyy@asu.edu` |
| 11 | `laohan50` | `Deangeloo1` | `Deangeloo@asu.edu` |
| 12 | `laohan51` | `Smithamm1` | `Smithamm@asu.edu` |
| 13 | `laohan52` | `Nienoww` | `Nienoww@asu.edu` |
| 14 | `laohan53` | `Keelinggg` | `Keelinggg@asu.edu` |
| 15 | `laohan54` | `Reingerrr` | `Reingerrr@asu.edu` |
| 16 | `laohan55` | `Reynoldss12` | `Reynoldss@asu.edu` |
| 17 | `laohan56` | `Kayleyu1` | `Kayleyu@asu.edu` |
| 18 | `laohan57` | `Sengerr2` | `Sengerr@asu.edu` |
| 19 | `laohan58` | `Schulist1` | `Schulist@asu.edu` |
| 20 | `laohan59` | `Arnoldo123z` | `Arnoldo@asu.edu` |
| 21 | `laohan60` | `Kunzeez` | `Kunzeez@asu.edu` |
| 22 | `laohan61` | `Autumnn12` | `Autumnn@asu.edu` |
| 23 | `laohan62` | `Fritschh` | `Fritschh@asu.edu` |
| 24 | `laohan63` | `Carminee123` | `Carmine@asu.edu` |
| 25 | `laohan64` | `Hodkiewicz1` | `Hodkiewicz@asu.edu` |
| 26 | `laohan65` | `Clementinee1` | `Clementinee@asu.edu` |
| 27 | `laohan66` | `Kilback12` | `Kilback@asu.edu` |
| 28 | `laohan67` | `Maxweelll` | `Maxweelll@asu.edu` |
| 29 | `laohan68` | `Bernierzz` | `Bernierzz@asu.edu` |
| 30 | `laohan69` | `Marcellee1` | `Marcellee@asu.edu` |
| 31 | `laohan70` | `Gulgowskiii` | `Gulgowski@asu.edu` |
| 32 | `laohan71` | `Lindgrenx` | `Lindgrenx@asu.edu` |
| 33 | `laohan72` | `Kristoffercc` | `Kristofferc@asu.edu` |
| 34 | `laohan73` | `Newellxxx` | `Newellxxx@asu.edu` |
| 35 | `laohan74` | `Gutkowskizz` | `Gutkowskizz@asu.edu` |
| 36 | `laohan77` | `Ahmedddz2` | `Ahmedd@asu.edu` |
| 37 | `laohan78` | `Phoebee1` | `Phoebee@asu.edu` |
| 38 | `laohan79` | `Virginiaaz` | `Virginiaa@asu.edu` |
| 39 | `laohan80` | `Francescazzz` | `Francesca@asu.edu` |
| 40 | `laohan81` | `Okuneva1` | `Okuneva@asu.edu` |
| 41 | `laohan82` | `Stammzz` | `Stammz@asu.edu` |
| 42 | `laohan83` | `Mayertzz` | `Mayertzz@asu.edu` |
| 43 | `laohan84` | `Goodwinzz` | `Goodwinz@asu.edu` |
| 44 | `laohan85` | `Koeppzz` | `Koeppzz@asu.edu` |
| 45 | `laohan86` | `Kuhlman1` | `Kuhlman@asu.edu` |
| 46 | `laohan87` | `Violetxxz` | `Violetxx@asu.edu` |
| 47 | `laohan89` | `Dawnzzz1` | `Dawnzzz@asu.edu` |
| 48 | `laohan90` | `Francessx` | `Francess@asu.edu` |
| 49 | `laohan92` | `Alfredozzzx` | `Alfredoz@asu.edu` |
| 50 | `laohan93` | `Rolfsonx` | `Rolfson@asu.edu` |
| 51 | `laohan94` | `Torphyx` | `Torphy@asu.edu` |
| 52 | `laohan95` | `Strosinx` | `Strosin@asu.edu` |
| 53 | `laohan96` | `Genovevax` | `Genoveva@asu.edu` |
| 54 | `laohan97` | `Leuschkez` | `Leuschkez@asu.edu` |
| 55 | `laohan98` | `Ottiliezz` | `Ottiliezz@asu.edu` |
| 56 | `laohan99` | `Pourosx` | `Pouros@asu.edu` |
| 57 | `laohan100` | `Kautzerx` | `Kautzer@asu.edu` |
| 58 | `laohan101` | `Denesikx` | `Denesik@asu.edu` |
| 59 | `laohan102` | `Ankundingx` | `Ankunding@asu.edu` |
| 60 | `laohan103` | `Madisenxx` | `Madisenx@asu.edu` |
| 61 | `laohan104` | `Peytonzx` | `Peytonzx@asu.edu` |
| 62 | `laohan105` | `Bogisichx` | `Bogisich@asu.edu` |
| 63 | `laohan106` | `Susannaan` | `Susannaan@asu.edu` |
| 64 | `laohan107` | `Kemmerx` | `Kemmer@asu.edu` |
| 65 | `laohan108` | `Koeppx` | `Koepp@asu.edu` |
| 66 | `laohan109` | `Carrollzx` | `Carrollz@asu.edu` |
| 67 | `laohan110` | `Quintonxx` | `Quintonx@asu.edu` |
| 68 | `laohan111` | `Paucekzz` | `Paucekzz@asu.edu` |
| 69 | `laohan112` | `Hermistonz` | `Hermiston@asu.edu` |
| 70 | `laohan113` | `Sipescccsz` | `Sipescccsz@asu.edu` |
| 71 | `laohan114` | `Prosaccoxxxs` | `Prosaccoxxxs@asu.edu` |
| 72 | `laohan115` | `Jacobsonxxgt` | `Jacobsonxxgt@asu.edu` |
| 73 | `laohan116` | `Ritchiezzwwf` | `Ritchiezzwwf@asu.edu` |
| 74 | `laohan117` | `Carolynebbd` | `Carolynebbd@asu.edu` |
| 75 | `laohan118` | `Barrowssszx` | `Barrowssszx@asu.edu` |
| 76 | `laohan119` | `Martajous` | `Martajous@asu.edu` |
| 77 | `laohan120` | `Greenholtx` | `Greenholtx@asu.edu` |
| 78 | `laohan121` | `Melodyzbbd` | `Melodyzbbd@asu.edu` |
| 79 | `laohan122` | `Greenholtccsv` | `Greenholtccsv@asu.edu` |
| 80 | `laohan123` | `Goyettejjuj` | `Goyettejjuj@asu.edu` |
| 81 | `laohan124` | `Margiexxss` | `Margiexxss@asu.edu` |
| 82 | `laohan125` | `Rolfsonbbfx` | `Rolfsonbbfx@asu.edu` |
| 83 | `laohan126` | `Antoninagggf` | `Antoninagggf@asu.edu` |
| 84 | `laohan127` | `Augustinehhj` | `Augustinehhj@asu.edu` |
| 85 | `laohan128` | `Bradtkeggg` | `Bradtkeggg@asu.edu` |
| 86 | `laohan130` | `Howellhhdx` | `Howellhhdx@asu.edu` |
| 87 | `laohan131` | `Shanahansssx` | `Shanahansssx@asu.edu` |
| 88 | `laohan132` | `Schusterzssc` | `Schusterzssc@asu.edu` |
| 89 | `laohan133` | `Robertxxggsd` | `Robertxxggsd@asu.edu` |
| 90 | `laohan134` | `Barnettggssx` | `Barnettggssx@asu.edu` |
| 91 | `laohan135` | `Stephaniegggs` | `Stephaniegggs@asu.edu` |
| 92 | `laohan136` | `Kautzerggdg` | `Kautzerggdg@asu.edu` |
| 93 | `laohan137` | `Grantxxxs` | `Grantxxxs@asu.edu` |
| 94 | `laohan138` | `Rohanxxxsv` | `Rohanxxxsv@asu.edu` |
| 95 | `laohan139` | `Madelynnzsss` | `Madelynnzsss@asu.edu` |
| 96 | `laohan140` | `Nolanhbhhhh` | `Nolanhbhhhh@asu.edu` |
| 97 | `laohan141` | `Prestonhjhhh` | `Prestonhjhhh@asu.edu` |
| 98 | `laohan142` | `Octaviashhdf` | `Octaviashhdf@asu.edu` |
| 99 | `laohan145` | `Miraclessssx` | `Miraclessss@asu.edu` |
| 100 | `laohan147` | `Katelinkklk` | `Katelinkklk@asu.edu` |
| 101 | `laohan148` | `Dickiddd` | `Dickiddd@asu.edu` |
| 102 | `laohan149` | `Thompsonhh` | `Thompsonhh@asu.edu` |
| 103 | `laohan150` | `Hamillggss` | `Hamillggss@asu.edu` |
| 104 | `laohan151` | `Jaskolskixx` | `Jaskolskixx@asu.edu` |
| 105 | `laohan152` | `Deckowgggg` | `Deckowgggg@asu.edu` |
| 106 | `laohan153` | `Spencerggs` | `Spencerggs@asu.edu` |
| 107 | `laohan154` | `Sincereggg` | `Sincereggg@asu.edu` |
| 108 | `laohan155` | `Nanniessgg` | `Nanniessgg@asu.edu` |
| 109 | `laohan156` | `Rogersjjuh` | `Rogersjjuh@asu.edu` |
| 110 | `laohan159` | `Corkeryssgg` | `Corkeryssgg@asu.edu` |
| 111 | `laohan160` | `Brownghhf` | `Brownghhf@asu.edu` |
| 112 | `laohan162` | `Martinssggs` | `Martinssggs@asu.edu` |
| 113 | `laohan163` | `Mayertssghh` | `Mayertssghh@asu.edu` |
| 114 | `laohan164` | `Volkmanhhh` | `Volkmanhhh@asu.edu` |
| 115 | `laohan165` | `Traceklokk` | `Traceklokk@asu.edu` |
| 116 | `laohan166` | `Jakubowskipp` | `Jakubowskipp@asu.edu` |
| 117 | `laohan167` | `Josephinekjk` | `Josephinekjk@asu.edu` |
| 118 | `laohan168` | `Madelynnsab` | `Madelynnsab@asu.edu` |
| 119 | `laohan169` | `Ameliellko` | `Ameliellko@asu.edu` |
| 120 | `laohan170` | `Barrowsvfvb` | `Barrowsvfvb@asu.edu` |
| 121 | `laohan171` | `Morissettehdh` | `Morissettehdh@asu.edu` |
| 122 | `laohan172` | `Purdyhdgh` | `Purdyhdgh@asu.edu` |
| 123 | `laohan173` | `Leuschkehgh` | `Leuschkehgh@asu.edu` |
| 124 | `laohan174` | `Geovannihhd` | `Geovannihhd@asu.edu` |
| 125 | `laohan175` | `Shieldsshhh` | `Shieldsshhh@asu.edu` |
| 126 | `laohan176` | `Beahanhhhf` | `Beahanhhhf@asu.edu` |
| 127 | `laohan177` | `Daughertyhs` | `Daughertyh@asu.edu` |
| 128 | `laohan178` | `Kiannahhj` | `Kiannahhj@asu.edu` |
| 129 | `laohan179` | `Schoenhh` | `Schoenhh@asu.edu` |
| 130 | `laohan180` | `Bernhardggs` | `Bernhardggs@asu.edu` |
| 131 | `laohan181` | `Maybellokj` | `Maybellokj@asu.edu` |
| 132 | `laohan182` | `Shanelggg` | `Shanelggg@asu.edu` |
| 133 | `laohan183` | `Gerlachjkk` | `Gerlachjkk@asu.edu` |
| 134 | `laohan184` | `Casperjjkkj` | `Casperjjkkj@asu.edu` |

### 上轮遗留的 34 个 401

这里保留之前的处理状态；本次没有重登、改密、启停账号或改变代理、Key、组与调度配置。

| 网关账号 | 邮箱 | 既有恢复进度 |
| --- | --- | --- |
| `laohan88` | 未记录 | 待核对 |
| `laohan91` | 未记录 | 待核对 |
| `laohan143` | 未记录 | 待核对 |
| `laohan144` | 未记录 | 待核对 |
| `laohan146` | 未记录 | 待核对 |
| `laohan157` | 未记录 | 待核对 |
| `laohan158` | 未记录 | 待核对 |
| `laohan192` | 未记录 | 待核对 |
| `129` | 未记录 | 待核对 |
| `kiro-and163-google-social` | 未记录 | 按用户要求忽略（2026-10-12）：Google账号 |
| `kiro-francescagiuseppecominetti-github-social` | francescagiuseppecominetti@gmail.com | 按用户要求暂缓（2026-10-15）：GitHub经Google登录，先不处理 |
| `kiro-leisingriester-github-social` | leisingriester@gmail.com | 按用户要求暂缓（2026-10-15）：GitHub经Google登录，先不处理 |
| `kiro-smithdanielpavmb8172-github-social` | smithdanielpavmb8172@gmail.com | 按用户要求暂缓（2026-10-15）：GitHub经Google登录，先不处理 |
| `kiro-web-github` | 未记录 | 按用户要求忽略（2026-10-15）：标记失联，GitHub身份未确认，停止追查及重登；余额未核验 |
| `laohan-new-1` | 未记录 | 待核对 |
| `laohan-new-10` | 未记录 | 待核对 |
| `laohan-new-11` | 未记录 | 待核对 |
| `laohan-new-12` | 未记录 | 待核对 |
| `laohan-new-13` | 未记录 | 待核对 |
| `laohan-new-14` | 未记录 | 待核对 |
| `laohan-new-15` | 未记录 | 待核对 |
| `laohan-new-16` | 未记录 | 待核对 |
| `laohan-new-17` | 未记录 | 待核对 |
| `laohan-new-18` | 未记录 | 待核对 |
| `laohan-new-19` | 未记录 | 待核对 |
| `laohan-new-2` | 未记录 | 待核对 |
| `laohan-new-20` | 未记录 | 待核对 |
| `laohan-new-3` | 未记录 | 待核对 |
| `laohan-new-4` | 未记录 | 待核对 |
| `laohan-new-5` | 未记录 | 待核对 |
| `laohan-new-6` | 未记录 | 待核对 |
| `laohan-new-7` | 未记录 | 待核对 |
| `laohan-new-8` | 未记录 | 待核对 |
| `laohan-new-9` | 未记录 | 待核对 |

两个旧 IDC 错误账号为 `KitWilliam`、`laohan_MarleyMacdo`；错误均为 `invalid_grant / Invalid refresh token provided`。

### 证据与边界

- 基线：[前次审计](kiro-account-recovery-audit-2026-10-11.md) 中 2026-10-15 19:02:44 的恢复后状态：354 个账号，34 个 401、58 个仅停用、2 个其他错误，99 个已恢复。
- 历史对应关系：[上号恢复快照](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md)。该文件初建于 7 月，随后追加各批次恢复记录；匹配的是明确的 token/email 恢复成功行，不依据邮箱前缀猜 GitHub 用户名。
- 本次为控制面只读审计，全部管理请求为 GET；未逐个强制刷新上游，未进行模型调用。在线状态来自已有缓存及错误记录。
- `last_checked_at` 表示最近检查，不是首次异常；新增依据与前次清单、恢复进度的差集计算。
- 账号认证异常与余额、手动停用分开统计；旧的暂缓、忽略、受限状态保留，不自动修改。
- 私有审计目录：`~/.local/share/kiro-account-audits/20261031-035338`；仅保存脱敏快照和核验结果，不包含密码、验证码、access/refresh token 或客户端密钥。
