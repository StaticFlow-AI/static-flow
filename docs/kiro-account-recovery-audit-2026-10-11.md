# Kiro 异常账号与历史上号记录核对（2026-10-11）

账号状态快照：**2026-10-11 02:16:07，UTC+08:00**。历史上号核对日期：2026-10-11。
本文依据本项目已有文档确认“以前上过号”，不以聊天记录或账号存在于数据库作为成功上号的证据。

## 核对结果

| 范围 | 数量 | 说明 |
| --- | ---: | --- |
| Kiro 账号总数 | 354 | 上一轮已完整分页读取的管理接口快照 |
| 当前停用账号 | 163 | 103 个 `401 Bad credentials`、58 个仅有停用标记、2 个 IDC refresh `invalid_grant` |
| 近期优先重登清单 | 73 | 2026-09-20 起仍有成功检查记录，当前均为 `401 Bad credentials`；72 个 GitHub、1 个 Google |
| 其中：项目文档明确记录曾成功上号或恢复 | **37** | 恢复快照 27 个、上号追踪表 9 个、申诉追踪表已恢复项 1 个 |
| 其中：项目文档未找到明确成功记录 | **36** | 不能据此判断“从未上过号” |
| 较早异常或时间待确认清单 | 90 | 其中 `laohan185` 也有成功恢复记录；连同近期清单共确认 38 个曾上号成功 |

“9 月底开始异常”目前只能用 **9 月 20 日起仍成功检查过、现在异常** 作为筛选近似；
最后成功检查时间是状态/余额检查成功时间，不是历史上号时间，也不等于最后一次模型请求成功。
最近检查时间会被后续检查更新，**不是首次异常时间**，因此不能断言这 73 个都首次在九月底失效。
`401 Bad credentials` 只说明当前凭据被拒绝，不能单凭这一项认定 GitHub 被封禁。

同一快照中，最近维护的 10 个 AWS Builder ID 账号均已启用且没有账号异常，不在这 163 个停用账号中。
这批账号的上号和请求验证记录见 [AWS Builder ID 账号安全维护记录](aws-builder-id-account-security-tracker.md)。

## 项目文档来源与判定方式

| 标记 | 本项目文档 | 本次采用的证据 |
| --- | --- | --- |
| R | [Kiro account recovery quota snapshot](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md) | `Recovery observations` 中的登录名、邮箱、网关账号映射及 token 恢复结果 |
| G | [GitHub / Kiro Account Onboarding Tracker](github-kiro-account-onboarding-tracker.md) | `Kiro GitHub Social Refreshes` 中的 Ready / KIRO STUDENT 结果 |
| S | [GitHub Account Suspension Appeal Tracker](github-account-suspension-appeal-tracker.md) | `Already Recovered` 中的已恢复并重新导入结果，以及历史受限记录 |

R 虽放在 `tmp/` 下，但已纳入本项目 Git 版本管理，是已有项目文档。
其页首日期属于初始额度快照，恢复表后续持续追加，没有逐行操作日期；本文不把所有恢复操作都记成 7 月 11 日。
本次按完整网关账号名与已记录的邮箱、映射关系核对，没有用名字前缀、额度相似或登录名猜测身份。
G 的成功表只写 `Account name`，所以不能直接把那 9 个网关账号名当作已经确认的 GitHub 登录名。

## 近期清单：37 个有成功上号记录

均待本轮重新登录；当前问题均为 `401 Bad credentials`，且 `disabled=true`。
序号沿用完整 CSV，时间均为 2026 年 UTC+08:00。GitHub 登录名“未记录”表示原项目文档没有单独确认该字段。

| CSV 序号 | 网关账号 | 邮箱 | 历史 GitHub 登录名 | 最后成功检查 | 历史证据 | 本次恢复进度 |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | `elestineaw` | `elestineaw@utexas.edu` | `elestineaw` | 10-11 01:05:10 | [R:50](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L50) | 待重新登录 |
| 2 | `unfieldgt` | `unfield@utexas.edu` | `unfieldgt` | 10-11 01:03:25 | [R:49](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L49) | 待重新登录 |
| 3 | `embrookew` | `embrooke@utexas.edu` | `embrookew` | 10-11 00:51:22 | [R:48](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L48) | 待重新登录 |
| 4 | `verharta` | `verharta@utexas.edu` | `verharta` | 10-11 00:42:36 | [R:47](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L47) | 待重新登录 |
| 5 | `bbottadq` | `bbottadq@utexas.edu` | `bbottadq` | 10-11 00:39:33 | [R:46](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L46) | 待重新登录 |
| 6 | `haedradw` | `haedra@utexas.edu` | `haedradw` | 10-11 00:37:42 | [R:45](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L45) | 待重新登录 |
| 7 | `agnoliaqwd` | `agnolia@utexas.edu` | `agnoliaqwd` | 10-11 00:27:55 | [R:44](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L44) | 待重新登录 |
| 8 | `ddggte` | `reuiiisa@uchicago.edu` | `ddggte` | 10-10 23:55:51 | [R:43](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L43) | 待重新登录 |
| 9 | `shfordsdf` | `shfordsdf@utexas.edu` | `shfordsdf` | 10-09 02:20:21 | [R:42](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L42) | 待重新登录 |
| 10 | `oorcrofta` | `oorcrofta@utexas.edu` | `oorcrofta` | 10-09 02:17:45 | [R:41](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L41) | 待重新登录 |
| 11 | `wqalerian` | `wqalerian@utexas.edu` | `wqalerian` | 10-09 02:08:43 | [R:40](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L40) | 待重新登录 |
| 12 | `tzpatric` | `tzpatrick@utexas.edu` | `tzpatric` | 10-09 02:02:57 | [R:39](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L39) | 待重新登录 |
| 13 | `laohan18` | `BiHankiro13@asu.edu` | `linkueisz18` | 10-09 01:55:05 | [R:38](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L38) | 待重新登录 |
| 14 | `laohan17` | `BiHankiro10@asu.edu` | `linkueisz17` | 10-09 01:43:18 | [R:37](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L37) | 待重新登录 |
| 15 | `laohan16` | `BiHankiro9@asu.edu` | `linkueisz16` | 10-09 01:41:33 | [R:36](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L36) | 待重新登录 |
| 16 | `laohan15` | `BiHankiro8@asu.edu` | `linkueisz15` | 10-09 01:34:48 | [R:35](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L35) | 待重新登录 |
| 17 | `laohan14` | `BiHankiro7@asu.edu` | `linkueisz14` | 10-09 01:21:44 | [R:34](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L34) | 待重新登录 |
| 18 | `laohan13` | `BiHankiro6@asu.edu` | `linkueisz13` | 10-09 01:19:58 | [R:33](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L33) | 待重新登录 |
| 19 | `laohan12` | `BiHankiro5@asu.edu` | `linkueisz12` | 10-09 01:14:48 | [R:32](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L32) | 待重新登录 |
| 20 | `laohan11` | `BiHankiro4@asu.edu` | `linkueisz11` | 10-09 01:12:18 | [R:31](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L31) | 待重新登录 |
| 21 | `laohan10` | `BiHankiro3@asu.edu` | `linkueisz10` | 10-09 01:11:37 | [R:30](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L30) | 待重新登录 |
| 22 | `linkuei_3_laohan7` | `BiHankiro2@asu.edu` | `linkueisz9` | 10-09 01:02:33 | [R:29](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L29) | 待重新登录 |
| 23 | `laohan9` | `Jinbo.Pan@asu.edu` | `linkueisz8` | 10-09 00:56:45 | [R:28](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L28) | 待重新登录 |
| 24 | `laohan5` | `Calebkl2lis@asu.edu` | `linkueisz5` | 10-09 00:52:18 | [R:27](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L27) | 待重新登录 |
| 25 | `laohan6` | `Calebkl1lis@asu.edu` | `linkueisz4` | 10-09 00:47:20 | [R:26](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L26) | 待重新登录 |
| 26 | `laohan4` | `Nolan.arlisle@asu.edu` | `linkueisz2` | 10-09 00:45:09 | [R:25](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L25) | 待重新登录 |
| 27 | `kiro-linkueisz3-github-social` | `Adrian.Carlisle@asu.edu` | `linkueisz3` | 10-09 00:35:49 | [R:24](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L24) | 待重新登录 |
| 37 | `afanchard` | `afanchard@utexas.edu` | 未记录 | 10-05 23:48:23 | [G:71](github-kiro-account-onboarding-tracker.md#L71) | 待重新登录 |
| 38 | `allahaneas` | `allahaneas@utexas.edu` | 未记录 | 10-05 23:33:30 | [G:70](github-kiro-account-onboarding-tracker.md#L70) | 待重新登录 |
| 39 | `ennoxfs` | `ennoxfs@utexas.edu` | 未记录 | 10-05 23:30:51 | [G:69](github-kiro-account-onboarding-tracker.md#L69) | 待重新登录 |
| 40 | `bbonsrfds` | `bbonsrfds@utexas.edu` | 未记录 | 10-05 23:25:24 | [G:68](github-kiro-account-onboarding-tracker.md#L68) | 待重新登录 |
| 41 | `ulligandw` | `ulligandw@utexas.edu` | 未记录 | 10-05 23:20:11 | [G:66](github-kiro-account-onboarding-tracker.md#L66) | 待重新登录 |
| 42 | `terhilder` | `terhilder@utexas.edu` | 未记录 | 10-05 23:19:24 | [G:67](github-kiro-account-onboarding-tracker.md#L67) | 待重新登录 |
| 43 | `adenrtheds` | `adenrtheds@utexas.edu` | 未记录 | 10-05 23:10:02 | [G:65](github-kiro-account-onboarding-tracker.md#L65) | 待重新登录 |
| 44 | `endrickgd` | `endrick@utexas.edu` | 未记录 | 10-05 23:07:53 | [G:64](github-kiro-account-onboarding-tracker.md#L64) | 待重新登录 |
| 45 | `onovansdf` | `onovansdf@utexas.edu` | 未记录 | 10-05 23:02:23 | [G:63](github-kiro-account-onboarding-tracker.md#L63) | 待重新登录 |
| 61 | `usmore` | `unsmore@utexas.edu` | `usmore` | 09-30 04:02:12 | [S:35](github-account-suspension-appeal-tracker.md#L35) | 待重新登录 |

其中 15 个账号的网关名称与已记录的 GitHub 登录名不同，表中已列出准确映射：
`kiro-linkueisz3-github-social`、`laohan4`、`laohan6`、`laohan5`、`laohan9`、
`linkuei_3_laohan7`、`laohan10` 至 `laohan18`。
`linkueisz3` 当时以新名称保留，文档明确未能确认它与 `KitWilliam` / `laohan_MarleyMacdo` 的关系，不能合并认定。
`agnoliaqwd` 当时凭据恢复成功但额度已经用尽；这仍属于成功上号记录。

## 近期清单：36 个尚未找到明确成功记录

以下账号也都为 `401 Bad credentials` 且已停用。“未找到”仅指本次检查的项目文档未提供明确成功记录。
邮箱为空时保留“未记录”，不从网关名称推测邮箱或登录用户名。

| CSV 序号 | 网关账号 | 邮箱 | 登录方式 | 最后成功检查 | 本次恢复进度 |
| ---: | --- | --- | --- | --- | --- |
| 28 | `atterfield` | `atterfield@utexas.edu` | GitHub | 10-08 00:10:49 | 待重新登录 |
| 29 | `herwoodwe` | `herwood@utexas.edu` | GitHub | 10-07 23:46:51 | 待重新登录 |
| 30 | `tratton` | `tratton@utexas.edu` | GitHub | 10-07 23:43:17 | 待重新登录 |
| 31 | `allowaysfdf` | `allowaysfdf@utexas.edu` | GitHub | 10-07 23:38:44 | 待重新登录 |
| 32 | `errickds` | `errick@utexas.edu` | GitHub | 10-07 01:23:37 | 待重新登录 |
| 33 | `orsythe` | `orsythe@utexas.edu` | GitHub | 10-07 01:18:24 | 待重新登录 |
| 34 | `ennysondgw` | `ennyson@utexas.edu` | GitHub | 10-07 01:16:21 | 待重新登录 |
| 35 | `rowtherfdh` | `rowther@utexas.edu` | GitHub | 10-07 01:10:29 | 待重新登录 |
| 36 | `nderwood` | `nderwood@utexas.edu` | GitHub | 10-07 00:53:44 | 待重新登录 |
| 46 | `sadvass` | `sadvass@utexas.edu` | GitHub | 10-03 23:39:54 | 待重新登录 |
| 47 | `liupverg` | `liup44@utexas.edu` | GitHub | 10-03 23:37:36 | 待重新登录 |
| 48 | `uincey` | `mshawpl@utexas.edu` | GitHub | 10-03 23:36:07 | 待重新登录 |
| 49 | `finslow` | `inslow@utexas.edu` | GitHub | 10-03 23:18:33 | 待重新登录 |
| 50 | `enroseop` | `enrose@utexas.edu` | GitHub | 10-03 23:17:50 | 待重新登录 |
| 51 | `aveleyol` | `aveley@utexas.edu` | GitHub | 10-03 23:12:44 | 待重新登录 |
| 52 | `vanderto` | `vander@utexas.edu` | GitHub | 10-03 23:08:00 | 待重新登录 |
| 53 | `phyrurwe` | `phyrurwe@utexas.edu` | GitHub | 10-03 23:04:18 | 待重新登录 |
| 54 | `anthamt` | `anthamt@utexas.edu` | GitHub | 10-03 23:01:10 | 待重新登录 |
| 55 | `reshnderam` | `reshnderam@utexas.edu` | GitHub | 10-02 01:03:07 | 待重新登录 |
| 56 | `enwicks` | `enwick@utexas.edu` | GitHub | 09-30 23:15:27 | 待重新登录 |
| 57 | `kiro-spare-1` | `ttdevb@uchicago.edu` | GitHub | 09-30 23:11:24 | 待重新登录 |
| 58 | `fhhfdss` | `derrtf@uchicago.edu` | GitHub | 09-30 23:02:40 | 待重新登录 |
| 59 | `default` | `gerjjhs22@utexas.edu` | GitHub | 09-30 22:59:00 | 待重新登录 |
| 60 | `avenshades` | `avenshade@utexas.edu` | GitHub | 09-30 04:26:50 | 待重新登录 |
| 62 | `ockcharts` | 未记录 | GitHub | 09-29 21:45:33 | 待重新登录 |
| 63 | `hdsgvghh` | 未记录 | GitHub | 09-29 21:28:13 | 待重新登录 |
| 64 | `dfadgfgg` | 未记录 | GitHub | 09-29 21:21:47 | 待重新登录 |
| 65 | `fdaerwqrqr` | 未记录 | GitHub | 09-29 21:09:28 | 待重新登录 |
| 66 | `gghytr` | 未记录 | GitHub | 09-29 12:00:36 | 待重新登录 |
| 67 | `gdhsbvf` | 未记录 | GitHub | 09-29 11:55:23 | 待重新登录 |
| 68 | `gderghu` | 未记录 | GitHub | 09-29 11:53:59 | 待重新登录 |
| 69 | `gdergh` | 未记录 | GitHub | 09-29 11:50:00 | 待重新登录 |
| 70 | `jfderth` | 未记录 | GitHub | 09-29 11:31:34 | 待重新登录 |
| 71 | `ntagueik` | 未记录 | GitHub | 09-29 11:18:10 | 待重新登录 |
| 72 | `kiro-and163-google-social` | 未记录 | Google | 09-24 00:45:24 | 待重新登录 |
| 73 | `kiro-web-github` | 未记录 | GitHub | 09-20 15:53:34 | 待重新登录 |

## 较早异常或时间待确认的 90 个账号

完整名单见 [较早异常或时间待确认 CSV](kiro-disabled-older-or-undated-2026-10-11.csv)。
这一组没有 9 月 20 日起成功检查的记录，包含 30 个 `401 Bad credentials`、58 个仅有停用标记、2 个 IDC refresh `invalid_grant`。
不能仅凭停用标记认定是凭据失效或封禁。

### 有成功恢复或未完成操作记录的账号

| 网关账号 | 历史 GitHub 登录名 | 历史处理结果 | 项目证据 |
| --- | --- | --- | --- |
| `laohan185` | `Padbergsss` | 曾成功恢复 token 和邮箱（`Padbergsss@asu.edu`） | [R:217](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L217) |
| `laohan192` | 未记录 | 旁注仅说明该账号流程未完成并保持停用；该行成功恢复的是 laohan193，不将其 GitHub 登录名套用到此账号。 | [R:224](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L224) |
| `laohan146` | `Windlerghhgh` | GitHub 登录/验证未完成或用户跳过；未完成 Kiro token 刷新。 | [R:181](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L181) |
| `laohan144` | `Torphyhjjj` | GitHub 登录/验证未完成或用户跳过；未完成 Kiro token 刷新。 | [R:179](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L179) |
| `laohan143` | `Reillyhhh` | GitHub 登录/验证未完成或用户跳过；未完成 Kiro token 刷新。 | [R:178](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L178) |
| `laohan91` | `Bartonzzzx` | GitHub 登录/验证未完成或用户跳过；未完成 Kiro token 刷新。 | [R:127](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L127) |
| `laohan88` | `Nienowz` | GitHub 登录/验证未完成或用户跳过；未完成 Kiro token 刷新。 | [R:124](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L124) |

`KitWilliam`、`laohan_MarleyMacdo`、`laohan3` 在 R 中只有恢复前异常快照，未找到对应网关账号的明确恢复成功记录。
`laohan76` 只在 `laohan77` 的旁注中被列为当时缺失的账号，不能把同一行的恢复成功结果套用到它。

### 25 个有历史 GitHub 受限记录的账号

以下是项目文档在此前操作中记录的 suspended/disabled 状态，**不是本轮重新登录后的判断**。
历史受限或尝试登录，不计入“曾成功上号”的 38 个账号。

| 网关账号 | 历史记录 | 项目证据 |
| --- | --- | --- |
| `hydgjj` | GitHub suspended/disabled；尚无后续恢复成功记录 | [G:87](github-kiro-account-onboarding-tracker.md#L87) |
| `ghrdfh` | GitHub suspended/disabled；尚无后续恢复成功记录 | [G:86](github-kiro-account-onboarding-tracker.md#L86) |
| `drfgyyhj` | GitHub suspended/disabled；尚无后续恢复成功记录 | [G:85](github-kiro-account-onboarding-tracker.md#L85) |
| `gyhjf` | GitHub suspended/disabled；尚无后续恢复成功记录 | [G:84](github-kiro-account-onboarding-tracker.md#L84) |
| `fgredb` | GitHub suspended/disabled；尚无后续恢复成功记录 | [G:83](github-kiro-account-onboarding-tracker.md#L83) |
| `ghdgb` | GitHub suspended/disabled；尚无后续恢复成功记录 | [G:82](github-kiro-account-onboarding-tracker.md#L82) |
| `jurdgh` | GitHub suspended/disabled；尚无后续恢复成功记录 | [G:81](github-kiro-account-onboarding-tracker.md#L81) |
| `gsetynm` | GitHub suspended/disabled；尚无后续恢复成功记录 | [G:80](github-kiro-account-onboarding-tracker.md#L80) |
| `ghrdds` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:22](github-account-suspension-appeal-tracker.md#L22) |
| `ffgthn` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:21](github-account-suspension-appeal-tracker.md#L21) |
| `sefgyjh` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:23](github-account-suspension-appeal-tracker.md#L23) |
| `hfdegv` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:24](github-account-suspension-appeal-tracker.md#L24) |
| `fgddefc` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:25](github-account-suspension-appeal-tracker.md#L25) |
| `khsytgb` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:26](github-account-suspension-appeal-tracker.md#L26) |
| `gddegn` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:27](github-account-suspension-appeal-tracker.md#L27) |
| `kdhhrj` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:14](github-account-suspension-appeal-tracker.md#L14) |
| `hfdery` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:15](github-account-suspension-appeal-tracker.md#L15) |
| `ljhyugg` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:16](github-account-suspension-appeal-tracker.md#L16) |
| `hgfdsw` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:28](github-account-suspension-appeal-tracker.md#L28) |
| `hfdryhy` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:17](github-account-suspension-appeal-tracker.md#L17) |
| `jhfded` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:29](github-account-suspension-appeal-tracker.md#L29) |
| `hfdvbgt` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:18](github-account-suspension-appeal-tracker.md#L18) |
| `hfdegh` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:19](github-account-suspension-appeal-tracker.md#L19) |
| `zjhferw` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:20](github-account-suspension-appeal-tracker.md#L20) |
| `gfryuuu` | GitHub suspended/disabled；尚无后续恢复成功记录 | [S:13](github-account-suspension-appeal-tracker.md#L13) |

## 上次登录与本次失效是否相隔约 90 天

**时间分布高度吻合 90 天周期。** 2026-10-11 再次核对已有项目文档的 Git 历史，
将每条成功记录对应的提交时间与账号快照的 `cache.last_success_at` 比较，
37 个有历史成功记录的账号全部落在 **89.9014～89.9988 天**。
提交时间只是操作完成后的记录时间，批量提交可能晚于实际登录，不能当作精确的登录时刻。

| 成功记录提交日期 | 账号数 | 本次异常前最后成功检查日期 | 与记录提交时间相隔 |
| --- | ---: | --- | --- |
| 2026-07-02 | 1 | 2026-09-30 | 89.961 天 |
| 2026-07-08 | 9 | 2026-10-05 | 89.901～89.933 天 |
| 2026-07-11 | 19 | 2026-10-09 | 89.920～89.992 天 |
| 2026-07-13 | 8 | 2026-10-10 至 2026-10-11 | 89.973～89.999 天 |

对应提交可用 `git blame --line-porcelain -- <原文档路径>` 逐行核对。
前 3 组分别对应 `9284ce9d`、`e808dfe8`、`9ca6aa6b`；7 月 13 日一组对应
`52a993f4`、`92769964`、`f89d72fe`、`5ca7d3a8`、`b209255c`、`5209701e`。

进一步交叉核对同一账号状态快照中的 `last_imported_at`：

- 全部 **73 个近期异常账号**，从上次导入到最后成功检查，间隔为 **89.9701～89.9996 天**，均接近 90 天。
- 其中有项目文档成功记录的 **37 个**，间隔更集中于 **89.9958～89.9996 天**，距 90 天只差约 35 秒至 6 分 3 秒。
- 其余 36 个的导入时间只能佐证周期，不能替代项目文档对“以前由我们上过号”的确认。

两个可以具体核对的例子（全部为 UTC+08:00）：

| 账号 | 成功记录提交 | 上次导入时间 | 最后成功检查 | 快照记录的最近失败检查 |
| --- | --- | --- | --- | --- |
| `embrookew` | 07-13 00:53:10，`b209255c` | 07-13 00:52:16.942 | 10-11 00:51:22 | 10-11 00:56:19 |
| `haedradw` | 07-13 00:40:05，`92769964` | 07-13 00:39:20.381 | 10-11 00:37:42 | 10-11 00:42:17 |

两个例子的成功与失败检查都夹在“导入后满 90 天”的前后几分钟内。
据此，**约 90 天的凭据或授权周期到期**是很强的排查方向；这仍是从时间分布作出的推断，
目前没有核实具体是哪类 token/授权到期，也没有确认官方固定有效期规则。
最近失败检查仍不等于首次失败时间，这个周期也不能直接作为账号被封禁的证据。

## 后续逐个恢复时记录什么

从上面的 37 个已有成功记录的近期账号开始较容易核对身份；其余账号仍可逐个尝试。
先按表中登录方式和已确认的 GitHub 登录名登录，再核对邮箱与网关账号映射。
每次完成后更新 CSV 的“恢复进度”，并在本文追加操作日期、真实登录结果、最终网关账号名和请求验证结果。
如果被登录平台拦住，记录实际页面结果，不把所有失败统一写为封禁。

- [近期优先重登 CSV：73 个](kiro-relogin-priority-2026-10-11.csv)
- [较早异常或时间待确认 CSV：90 个](kiro-disabled-older-or-undated-2026-10-11.csv)

两个 CSV 已增加“历史上号结论、历史 GitHub 用户名、历史记录邮箱、项目文档证据、历史备注”字段，
保留原始快照的账号顺序、状态、代理和检查时间，便于筛选及逐项回填。
本次只整理与核对文档，未重新登录这些异常账号，也未修改它们的停用状态或凭据。
文档与 CSV 不保存密码、TOTP 密钥、一次性验证码或会话令牌。
