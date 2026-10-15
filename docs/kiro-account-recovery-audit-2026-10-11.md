# Kiro 异常账号与历史上号记录核对（更新至 2026-10-15）

本轮重登前的账号状态快照：**2026-10-12 22:38:57，UTC+08:00**。
后续候选核查快照：**2026-10-15 01:49:16，UTC+08:00**；本次补入的账号使用此快照，既有恢复记录保留原快照。
初始快照与历史上号核对日期：2026-10-11。本文件与两个 CSV 保留初建日期的文件名，快照时间以内容为准。
本文依据本项目已有文档确认“以前上过号”，不以聊天记录或账号存在于数据库作为成功上号的证据。

## 本轮恢复进度（重登前快照之后）

2026-10-12 按优先重登清单开始恢复。目前 **85 个重登完成，17 个待完成，2 个按用户要求忽略**，清单共 104 项。后续补入的 31 项已完成 14 项，尚待处理 17 项。
2026-10-15 用户要求先不处理第 101–103 项 GitHub 经 Google 登录的账号，三项已标记“按用户要求暂缓”。页首待完成数包含这三项；其余账号继续依次恢复。
2026-10-15 首批 73 项处理结束：`kiro-and163-google-social` 按用户要求跳过；`kiro-web-github` 按用户要求标记为“失联”，停止身份追查及重登，真实 GitHub 用户名与邮箱仍未确认，本轮余额未核验。该账号原导入时间为 2026-06-22 16:19:42.026（UTC+08:00）；“失联”是本轮处理结论，不代表已确认 GitHub 封禁。
首批 71 个完成账号中，65 个已通过账号独占公网模型请求及用量归属核验；6 个本轮余额为 0（`haedradw`、`shfordsdf`、`laohan11`、`laohan4`、`atterfield`、`ennoxfs`），模型请求待额度恢复后核验。2026-10-15 再次完整分页检查 354 个账号，71 个完成账号均已启用、无账号异常、余额缓存为 `ready`。
本次续跑从第三项 `embrookew` 开始，2026-10-13 按用户要求在第 36 项 `nderwood` 完成后暂停。
2026-10-14 按用户要求，从第 37 项 `afanchard`（GitHub 登录邮箱 `afanchard@utexas.edu`）继续。
前 35 项按原要求执行代理登录，首次切换到 `http://127.0.0.1:11115` 在第 5 个成功账号之后。
2026-10-13 用户改为要求直连；从第 36 项 `nderwood` 开始，浏览器使用 `--no-proxy-server`，登录请求清除代理环境变量并直接访问。
2026-10-14 用户要求按累计每 5 个重登完成账号交替切换：第 36–40 个直连，第 41–45 个使用 `http://127.0.0.1:11115`，第 46–50 个再直连，之后循环。额度为 0 的已重登账号计入完成数，按要求跳过的 Google 账号不计入。
本轮按用户要求恢复 zellij `proxy` 会话，并启动其中 `Tab #6` 保存的 11115 代理命令；已验证监听和 GitHub 登录页 HTTP 200。可用 `zellij attach proxy` 接入该会话。
用户确认后续未记录邮箱的 GitHub 账号使用同名登录；`kiro-and163-google-social` 按要求跳过。
“重登完成”指授权、身份、凭据写回和余额核验完成；额度耗尽账号单独注明模型请求状态。
CSV 增加“本轮剩余积分”字段，记录每个账号本轮余额刷新时的剩余 credits。
CSV 第 1–73 项的账号状态与检查时间仍对应上述重登前快照；第 74–104 项使用后续候选核查快照。“恢复进度”记录后续处理结果，筛选“待重新登录”可查看待执行任务；另有三项标记“按用户要求暂缓”。完成恢复的账号不应再按旧快照认定为当前异常。

| CSV 序号 | 网关账号 / GitHub 用户名或本次登录标识 | 本轮验证结果 | 额度与原有设置 |
| ---: | --- | --- | --- |
| 1 | `elestineaw` / `elestineaw` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `elestineaw@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占此账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；usage journal 确认归属该账号，输入 10 / 输出 2 tokens，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 118.27 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 1122 ms、最低剩余额度 3。 |
| 2 | `unfieldgt` / `unfieldgt` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `unfield@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占此账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；usage journal 确认归属该账号，输入 10 / 输出 2 tokens，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 705 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 1121 ms、最低剩余额度 3。 |
| 3 | `embrookew` / `embrookew` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `embrooke@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1.88 credits；保留 `aws_us_east1`、并发 3、RPM 10、间隔 1109 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:21115`。 |
| 4 | `verharta` / `verharta` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `verharta@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 553.31 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1108 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:21115`。 |
| 5 | `bbottadq` / `bbottadq` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `bbottadq@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 388.20 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 1107 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:21115`。 |
| 6 | `haedradw` / `haedradw` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `haedra@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。本轮公网请求核验未通过；额度已耗尽，待额度恢复后核验。临时验证 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 **0 credits**；保留 `aws_us3_east1`、并发 3、RPM 10、间隔 1106 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 7 | `agnoliaqwd` / `agnoliaqwd` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `agnolia@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 388.27 credits；保留 `aws_us_east1`、并发 3、RPM 10、间隔 1105 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 8 | `ddggte` / `ddggte` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `reuiiisa@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 180.34 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 1200 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 9 | `shfordsdf` / `shfordsdf` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `shfordsdf@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `aws_us3_east1`、并发 3、RPM 10、间隔 1104 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 10 | `oorcrofta` / `oorcrofta` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `oorcrofta@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 105.16 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1103 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 11 | `wqalerian` / `wqalerian` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `wqalerian@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 722.43 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1102 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 12 | `tzpatric` / `tzpatric` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `tzpatrick@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 446.10 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1101 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 13 | `laohan18` / `linkueisz18` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro13@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 273.74 credits；保留 `do-us-6`、并发 3、RPM 10、间隔 1011 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 14 | `laohan17` / `linkueisz17` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro10@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 683.81 credits；保留 `do-us-6`、并发 3、RPM 10、间隔 1010 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 15 | `laohan16` / `linkueisz16` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro9@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 344.42 credits；保留 `do-us-5`、并发 3、RPM 10、间隔 1009 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 16 | `laohan15` / `linkueisz15` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro8@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 513.52 credits；保留 `do-us-5`、并发 3、RPM 10、间隔 1008 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 17 | `laohan14` / `linkueisz14` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro7@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 313.90 credits；保留 `do-us-11`、并发 3、RPM 10、间隔 1007 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 18 | `laohan13` / `linkueisz13` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro6@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 350.23 credits；保留 `do-us-4`、并发 3、RPM 10、间隔 1006 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 19 | `laohan12` / `linkueisz12` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro5@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 627.55 credits；保留 `do-us-4`、并发 3、RPM 10、间隔 1012 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 20 | `laohan11` / `linkueisz11` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro4@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `do-us-11`、并发 3、RPM 10、间隔 1004 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 21 | `laohan10` / `linkueisz10` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro3@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 400.10 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 1003 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 22 | `linkuei_3_laohan7` / `linkueisz9` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `BiHankiro2@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 774.08 credits；保留 `do-us-7`、并发 3、RPM 10、间隔 1101 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 23 | `laohan9` / `linkueisz8` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Jinbo.Pan@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 642.05 credits；保留 `do-us-7`、并发 3、RPM 10、间隔 1002 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 24 | `laohan5` / `linkueisz5` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Calebkl2lis@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 469.33 credits；保留 `do-us-7`、并发 3、RPM 10、间隔 1005 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 25 | `laohan6` / `linkueisz4` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Calebkl1lis@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 649.14 credits；保留 `do-us-1`、并发 3、RPM 10、间隔 1006 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 26 | `laohan4` / `linkueisz2` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Nolan.arlisle@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `do-us-8`、并发 3、RPM 10、间隔 1004 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 27 | `kiro-linkueisz3-github-social` / `linkueisz3` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Adrian.Carlisle@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 747.67 credits；保留 `do-us-1`、并发 3、RPM 10、间隔 594 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 28 | `atterfield` / `atterfield@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `atterfield@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `aws_us3_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 29 | `herwoodwe` / `herwood@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `herwood@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 707.47 credits；保留 `aws_us3_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 30 | `tratton` / `tratton@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `tratton@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 672.99 credits；保留 `aws_us3_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 31 | `allowaysfdf` / `allowaysfdf@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `allowaysfdf@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 344.19 credits；保留 `aws_us3_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 32 | `errickds` / `errick@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `errick@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 635.08 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 1023 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 33 | `orsythe` / `orsythe@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `orsythe@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 853.60 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1013 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 34 | `ennysondgw` / `ennyson@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ennyson@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 370.87 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 1003 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 35 | `rowtherfdh` / `rowther@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `rowther@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 728.61 credits；保留 `do-us-9`、并发 3、RPM 10、间隔 1203 ms、最低剩余额度 3；浏览器登录代理 `http://127.0.0.1:11115`。 |
| 36 | `nderwood` / `nderwood@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `nderwood@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 767.16 credits；保留 `do-us-8`、并发 3、RPM 10、间隔 1201 ms、最低剩余额度 3；浏览器登录代理 `直连（--no-proxy-server）`。 |
| 37 | `afanchard` / `afanchard@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `afanchard@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 291.20 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录为直连。 |
| 38 | `allahaneas` / `allahaneas@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `allahaneas@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 647.13 credits；保留 `aws_us_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录为直连。 |
| 39 | `ennoxfs` / `ennoxfs@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ennoxfs@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `aws_us3_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录为直连。 |
| 40 | `bbonsrfds` / `bbonsrfds@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `bbonsrfds@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 378.05 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 41 | `ulligandw` / `ulligandw@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ulligandw@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 931.71 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 42 | `terhilder` / `terhilder@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `terhilder@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 550.43 credits；保留 `aws_us_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 43 | `adenrtheds` / `adenrtheds@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `adenrtheds@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 708.64 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 44 | `endrickgd` / `endrick@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `endrick@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 363.04 credits；保留 `aws_us_east1`、并发 3、RPM 10、间隔 594 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 45 | `onovansdf` / `onovansdf@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `onovansdf@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 590.04 credits；保留 `aws_us3_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 46 | `sadvass` / `sadvass@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `sadvass@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 661.34 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1043 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 47 | `liupverg` / `liup44@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `liup44@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 854.06 credits；保留 `do-us-9`、并发 3、RPM 10、间隔 1041 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 48 | `uincey` / `mshawpl@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `mshawpl@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 686.47 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1037 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 49 | `finslow` / `inslow@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `inslow@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 839.95 credits；保留 `do-us-4`、并发 3、RPM 10、间隔 1033 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 50 | `enroseop` / `enrose@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `enrose@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 881.43 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 1031 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 51 | `aveleyol` / `aveley@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `aveley@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 833.34 credits；保留 `do-us-1`、并发 3、RPM 10、间隔 1029 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 52 | `vanderto` / `vander@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `vander@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 734.47 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1027 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 53 | `phyrurwe` / `phyrurwe@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `phyrurwe@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 785.20 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1023 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 54 | `anthamt` / `anthamt@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `anthamt@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 409.61 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 1021 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 55 | `reshnderam` / `reshnderam@utexas.edu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `reshnderam@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 988.89 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 233 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 56 | `enwicks` / `enwicks` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `enwick@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 200 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 57 | `kiro-spare-1` / `hfdeeh` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ttdevb@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 200 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 58 | `fhhfdss` / `fhhfdss` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `derrtf@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-3`、并发 4、RPM 10、间隔 200 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 59 | `default` / `hgdshf` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `gerjjhs22@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-7`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 60 | `avenshades` / `avenshades` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `avenshade@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-2`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 61 | `usmore` / `unsmore` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `unsmore@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 999.99 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 425 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 63 | `hdsgvghh` / `hdsgvghh` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `retyusa@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-1`、并发 3、RPM 10、间隔 1150 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 62 | `ockcharts` / `ockharts` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ockhart@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-3`、并发 4、RPM 10、间隔 200 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 64 | `dfadgfgg` / `dfadgfgg` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `reyuias@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-10`、并发 3、RPM 10、间隔 1100 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 65 | `fdaerwqrqr` / `fdaerwqrqr` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `reqwes@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1000 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 66 | `gghytr` / `gghytr` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `reuiiisga@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-10`、并发 3、RPM 10、间隔 920 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 67 | `gdhsbvf` / `gdhsbvf` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `scured@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-10`、并发 3、RPM 10、间隔 999 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 68 | `gderghu` / `gderghu` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `emouiue@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-10`、并发 3、RPM 10、间隔 905 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 69 | `gdergh` / `gdergh` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ttyeassd@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-10`、并发 3、RPM 10、间隔 1050 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 70 | `jfderth` / `jfderth` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `poojhsa@uchicago.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 1250 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 71 | `ntagueik` / `ntagueik` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ntagueik@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 1000.00 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1111 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 74 | `allowayw` / `allowayw` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `allowayw@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 1123 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 75 | `ghtingaleaw` / `ghtingaleaw` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ghtingaleaw@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 503.73 credits；保留 `do-us-10`、并发 3、RPM 10、间隔 1204 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 76 | `orvalede` / `orvalede` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `orvalede@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 50.94 credits；保留 `do-us-3`、并发 3、RPM 10、间隔 1125 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 77 | `arlowew` / `arlowew` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `arlowew@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `do-us-1`、并发 3、RPM 10、间隔 1126 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 78 | `deoqew` / `deoqew` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `deoqew@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 362.58 credits；保留 `do-us-9`、并发 3、RPM 10、间隔 1127 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 79 | `tellanfs` / `tellanfs` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `tellanfs@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 166.79 credits；保留 `aws_us2_east1`、并发 3、RPM 10、间隔 1128 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 80 | `ghtingalew` / `ghtingalew` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `ghtingalew@utexas.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `do-us-7`、并发 3、RPM 10、间隔 1129 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 81 | `laohan19` / `Christiansenx` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Christiansen@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `do-us-7`、并发 3、RPM 10、间隔 1101 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 82 | `laohan20` / `Thompsonx` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Thompsonx@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.60 credits；保留 `do-us-7`、并发 3、RPM 10、间隔 1102 ms、最低剩余额度 3；浏览器登录方式 `直连（--no-proxy-server）`。 |
| 83 | `laohan21` / `Julianazzzx` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Julianaz@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 454.16 credits；保留 `do-us-7`、并发 3、RPM 10、间隔 1003 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 84 | `laohan22` / `Leslieex` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Leslieex@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。额度已耗尽；本轮未进行公网模型请求核验，待额度恢复后核验。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 0.00 credits；保留 `do-us-8`、并发 3、RPM 10、间隔 1004 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 85 | `laohan23` / `Yazminz` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Yazmin@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 364.31 credits；保留 `do-us-8`、并发 3、RPM 10、间隔 1005 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 86 | `laohan24` / `Hartmannxz` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Hartmannx@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 632.97 credits；保留 `do-us-8`、并发 3、RPM 10、间隔 1006 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |
| 87 | `laohan25` / `Carmellaz` | GitHub/Kiro 重新授权完成；`whoami` 邮箱为 `Carmellaz@asu.edu`；原账号已启用，`issue_kind` 已清除，余额缓存为 `ready`。独占账号的临时 Key 公网请求 `claude-sonnet-5.5` 返回 HTTP 200、`KIRO_OK`、`end_turn`；用量归属已确认，`usage_missing=false`；临时 Key 已删除并验证不存在。 | `KIRO STUDENT`，上限 1000，余额刷新时剩余 60.76 credits；保留 `do-us-8`、并发 3、RPM 10、间隔 1007 ms、最低剩余额度 3；浏览器登录方式 `http://127.0.0.1:11115`。 |

此次使用独立 Kiro CLI 数据目录完成登录，没有改动默认 CLI 账号。
第 48 项 `uincey` 的首次公网核验在客户端等待 60 秒后读超时；持久用量记录显示服务端最终 HTTP 200、耗时约 74 秒。将响应等待延长至 180 秒后补做完整响应核验，HTTP 200、`KIRO_OK`、`end_turn` 和用量归属均通过，两次临时 Key 均已删除。
核对发现技能原有刷新代码遗漏 `kiro_channel_rpm_limit`，使首个账号的 RPM 被重置为默认值 5；
已恢复该账号原值 10，并修正技能与设置保留测试，25 个 Python 测试通过。
后续重登使用修正后的技能保留该字段。

第 61 项首次把网关名 `usmore` 当作 GitHub 用户名登录，页面提示不支持密码登录，未完成授权或写回。核对历史会话后，仅能确认原网关名 `usmore` 和 Kiro 邮箱 `unsmore@utexas.edu`；当时是用户手动登录，未明确记录真实 GitHub 用户名。用户指出应为 `unsmore`；本轮改用该登录名后密码及验证码验证通过，并已从认证浏览器确认真实 GitHub 用户名 `unsmore`、Kiro 邮箱 `unsmore@utexas.edu`。原快照中的 `usmore` 登录名映射需结合本轮核对结果使用。

第 62 项首次使用网关名 `ockcharts` 登录时，页面提示用户名或密码错误，未完成授权或写回。用户纠正真实 GitHub 用户名为 `ockharts`；本轮密码及验证码验证通过，并从认证浏览器确认用户名 `ockharts`、Kiro 邮箱 `ockhart@utexas.edu`。每五个切换网络按实际重登完成数计算，未完成项不计入。

追查末项时发现，现有刷新流程经 `create_admin_kiro_account` / `upsert_kiro_account` 写回已有账号，会将 `created_at_ms` 更新为本次导入时间；因此当前数据库的创建时间排序不能直接视为最初注册顺序。本轮未修改该实现或回写历史时间。第 56–71 项已在下表记录实际核验的 GitHub 用户名；此前仅记录登录邮箱的项保留原证据，不据邮箱推断用户名。


第 83 项 `laohan21` / `Julianazzzx` 首次通过 GitHub 验证后，Kiro 回调页面返回 `invalid_request`，当次未写回网关凭据。保留并核对已登录的独立 GitHub 浏览器会话后，使用新的 Kiro 设备授权复用该会话，完成身份、余额与账号独占公网请求核验；全程沿用 11115 代理。

## 本轮补充的真实 GitHub 用户名（2026-10-14 起）

按用户要求，后续恢复记录同时保留真实 GitHub 用户名、Kiro 身份邮箱和网关账号。登录邮箱不等同于 GitHub 用户名。完成认证后从本次独立浏览器核对用户名；已有明确映射时同时核对该映射。

以下身份映射来自本地历史会话 `019f1dd3-1a6f-7772-a13f-609bab9bd4f0` 中的明确 `--github-login` / `--account-name` 参数，以及同次操作的 `whoami` / 导入邮箱；第 57 项另经用户确认，第 57–60 项在本轮独立浏览器及 Kiro 身份中再次核对。这里只补充身份映射，原快照的“历史上号结论”仍保留原项目文档核对口径。

| CSV 序号 | 网关账号 | 真实 GitHub 用户名 | Kiro 身份邮箱 | 历史会话证据 |
| ---: | --- | --- | --- | --- |
| 56 | `enwicks` | `enwicks` | `enwick@utexas.edu` | 2026-07-02 23:14 的登录参数、23:16 的身份与导入结果（UTC+08:00） |
| 57 | `kiro-spare-1` | `hfdeeh` | `ttdevb@uchicago.edu` | 2026-07-02 23:10 的登录参数、23:12 的身份与导入结果；用户确认 |
| 58 | `fhhfdss` | `fhhfdss` | `derrtf@uchicago.edu` | 2026-07-02 23:04 的登录参数、23:06 的身份与导入结果 |
| 59 | `default` | `hgdshf` | `gerjjhs22@utexas.edu` | 2026-07-02 22:57 的登录参数、23:01 的身份与导入结果 |
| 60 | `avenshades` | `avenshades` | `avenshade@utexas.edu` | 2026-07-02 04:21 的登录参数；本轮浏览器用户名与 `whoami` 邮箱已核对 |
| 61 | `usmore` | `unsmore` | `unsmore@utexas.edu` | 用户纠正；本轮密码登录、认证后浏览器用户名及 Kiro 身份邮箱均核对通过 |
| 62 | `ockcharts` | `ockharts` | `ockhart@utexas.edu` | 用户纠正；本轮认证后浏览器用户名、Kiro 身份邮箱均核对通过 |
| 63 | `hdsgvghh` | `hdsgvghh` | `retyusa@uchicago.edu` | 本轮密码及验证码登录、认证后浏览器用户名、Kiro 身份邮箱核对通过 |
| 64 | `dfadgfgg` | `dfadgfgg` | `reyuias@uchicago.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 65 | `fdaerwqrqr` | `fdaerwqrqr` | `reqwes@uchicago.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 66 | `gghytr` | `gghytr` | `reuiiisga@uchicago.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 67 | `gdhsbvf` | `gdhsbvf` | `scured@uchicago.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 68 | `gderghu` | `gderghu` | `emouiue@uchicago.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 69 | `gdergh` | `gdergh` | `ttyeassd@uchicago.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 70 | `jfderth` | `jfderth` | `poojhsa@uchicago.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 71 | `ntagueik` | `ntagueik` | `ntagueik@utexas.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 74 | `allowayw` | `allowayw` | `allowayw@utexas.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 75 | `ghtingaleaw` | `ghtingaleaw` | `ghtingaleaw@utexas.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 76 | `orvalede` | `orvalede` | `orvalede@utexas.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 77 | `arlowew` | `arlowew` | `arlowew@utexas.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 78 | `deoqew` | `deoqew` | `deoqew@utexas.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 79 | `tellanfs` | `tellanfs` | `tellanfs@utexas.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 80 | `ghtingalew` | `ghtingalew` | `ghtingalew@utexas.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 81 | `laohan19` | `Christiansenx` | `Christiansen@asu.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 82 | `laohan20` | `Thompsonx` | `Thompsonx@asu.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 83 | `laohan21` | `Julianazzzx` | `Julianaz@asu.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 84 | `laohan22` | `Leslieex` | `Leslieex@asu.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 85 | `laohan23` | `Yazminz` | `Yazmin@asu.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 86 | `laohan24` | `Hartmannxz` | `Hartmannx@asu.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |
| 87 | `laohan25` | `Carmellaz` | `Carmellaz@asu.edu` | 本轮认证后 GitHub 用户名与 Kiro 身份邮箱核对通过 |

第 61 项公网请求已返回 HTTP 200、`KIRO_OK`、`end_turn`；初次用量查询未命中，随后已从持久用量确认唯一对应的临时 Key 和账号归属、`usage_missing=false`，未重复发起模型请求。

第 57 项首次等待用户验证码时，Kiro 设备授权等待超时，未写回网关账号；重新发起直连登录。此前提供的验证码没有提交。

## 2026-10-15 后续 401 核查与候选补入

本次按**上次登录/凭据导入或刷新时间自 2026-07-01 起（UTC+08:00）**筛选尚未列入优先队列的当前 `auth_401` 账号。当前创建时间会随刷新写回变化，故不直接用 `created_at_ms` 判定“7 月以后上过号”；28 个补入账号在保留回滚快照中的创建时间为 4 月，7–8 月属于后续登录/刷新，另 3 个在此前数据库核查快照中创建于 7 月 15 日。

- 管理接口完整读取 200 + 154 条，共 354 个唯一账号；当前停用 122 个，其中 62 个 `auth_401`、58 个仅停用、2 个其他错误。
- 对比 2026-10-12 22:38:57 的 103 个 401：71 个首批重登账号已清除异常，32 个仍为 401；此外出现 30 个此前未列为 401 的账号。71 个已完成账号本次均已启用、无账号异常、余额缓存为 `ready`。
- 30 个新增 401 的上次导入/刷新时间均为 7 月 14–15 日，最后成功检查为 10 月 12–13 日。两者相隔 **89.995138–89.999516 天**，延续此前约 90 天到期的时间特征；这是时间分布推断，不是已确认的供应商有效期规则或封禁结论。
- 另将旧异常清单中的 `laohan185`（历史 GitHub 用户名 `Padbergsss`）补入队列：8 月 3 日曾刷新，最后成功检查为 8 月 4 日；这是已有异常，并非本次新增或本轮约 90 天到期的证据。原较早异常 CSV 保留该记录，并注明移交优先清单第 104 项，恢复工作只在优先清单回填。
- 新增队列第 74–104 项共 31 个：28 个已有项目文档明确记录的历史 GitHub 用户名及成功恢复证据，3 个 Gmail 关联的 GitHub 账号待确认实际用户名与登录入口。31 个均未在本次核查中重登或刷新余额，“本轮剩余积分”保持空白。
- `kiro-and163-google-social` 与失联的 `kiro-web-github` 继续按用户要求忽略；其余 29 个现有 401 不满足本次 7 月起导入/刷新的范围，仍留在较早异常清单。仅有停用标记的 58 个账号不作为新增 401。

下表时间为 2026 年、UTC+08:00。最近检查时间不代表首次 401 时间；完整邮箱、检查时间、代理和证据见 CSV。用户名来自历史明确映射，后续重登仍需核对浏览器实际用户名与 Kiro 身份邮箱。

| CSV 序号 | 网关账号 | 历史 GitHub 用户名 | 上次导入/刷新 | 最后成功检查 | 项目文档证据 / 待确认事项 |
| ---: | --- | --- | --- | --- | --- |
| 74 | `allowayw` | `allowayw` | 07-14 23:06:18 | 10-12 23:01:52 | [R:51](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L51) |
| 75 | `ghtingaleaw` | `ghtingaleaw` | 07-14 23:09:58 | 10-12 23:05:53 | [R:52](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L52) |
| 76 | `orvalede` | `orvalede` | 07-14 23:15:08 | 10-12 23:12:50 | [R:53](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L53) |
| 77 | `arlowew` | `arlowew` | 07-14 23:21:05 | 10-12 23:18:23 | [R:54](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L54) |
| 78 | `deoqew` | `deoqew` | 07-14 23:27:11 | 10-12 23:23:13 | [R:55](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L55) |
| 79 | `tellanfs` | `tellanfs` | 07-14 23:31:16 | 10-12 23:26:15 | [R:56](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L56) |
| 80 | `ghtingalew` | `ghtingalew` | 07-14 23:37:57 | 10-12 23:34:06 | [R:57](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L57) |
| 81 | `laohan19` | `Christiansenx` | 07-14 23:45:35 | 10-12 23:40:58 | [R:58](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L58) |
| 82 | `laohan20` | `Thompsonx` | 07-14 23:50:52 | 10-12 23:47:15 | [R:59](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L59) |
| 83 | `laohan21` | `Julianazzzx` | 07-14 23:54:30 | 10-12 23:52:11 | [R:60](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L60) |
| 84 | `laohan22` | `Leslieex` | 07-14 23:58:55 | 10-12 23:57:04 | [R:61](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L61) |
| 85 | `laohan23` | `Yazminz` | 07-15 00:03:51 | 10-13 00:00:33 | [R:62](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L62) |
| 86 | `laohan24` | `Hartmannxz` | 07-15 00:07:11 | 10-13 00:05:01 | [R:63](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L63) |
| 87 | `laohan25` | `Carmellaz` | 07-15 00:11:47 | 10-13 00:08:04 | [R:64](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L64) |
| 88 | `laohan26` | `Jacklynzz` | 07-15 00:16:10 | 10-13 00:12:07 | [R:65](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L65) |
| 89 | `laohan27` | `Bradtkez` | 07-15 00:20:45 | 10-13 00:19:24 | [R:66](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L66) |
| 90 | `laohan28` | `Naderxxz` | 07-15 00:23:32 | 10-13 00:18:57 | [R:67](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L67) |
| 91 | `laohan29` | `deckoww` | 07-15 00:26:09 | 10-13 00:23:03 | [R:68](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L68) |
| 92 | `laohan30` | `kertzmann1` | 07-15 00:29:49 | 10-13 00:25:03 | [R:69](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L69) |
| 93 | `laohan31` | `sschiller1` | 07-15 00:32:46 | 10-13 00:31:16 | [R:70](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L70) |
| 94 | `laohan32` | `ddenesikk` | 07-15 00:36:53 | 10-13 00:34:46 | [R:71](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L71) |
| 95 | `laohan33` | `Dooleyyy` | 07-15 00:41:08 | 10-13 00:38:54 | [R:72](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L72) |
| 96 | `laohan34` | `Bergstromx` | 07-15 00:43:22 | 10-13 00:38:56 | [R:73](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L73) |
| 97 | `laohan35` | `Martinexxxz` | 07-15 00:45:45 | 10-13 00:43:16 | [R:74](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L74) |
| 98 | `laohan36` | `Kassulkee` | 07-15 00:48:38 | 10-13 00:46:39 | [R:75](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L75) |
| 99 | `laohan37` | `Scottieeex` | 07-15 00:50:57 | 10-13 00:47:21 | [R:76](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L76) |
| 100 | `laohan38` | `Francescoxz` | 07-15 00:54:40 | 10-13 00:51:34 | [R:77](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L77) |
| 101 | `kiro-leisingriester-github-social` | 未记录（待确认） | 07-15 01:57:01 | 10-13 01:50:01 | 邮箱 `leisingriester@gmail.com`；已从历史会话确认 GitHub 经 Google 登录；按用户要求暂缓，真实 GitHub 用户名待核实 |
| 102 | `kiro-smithdanielpavmb8172-github-social` | 未记录（待确认） | 07-15 02:04:51 | 10-13 02:04:09 | 邮箱 `smithdanielpavmb8172@gmail.com`；已从历史会话确认 GitHub 经 Google 登录；按用户要求暂缓，真实 GitHub 用户名待核实 |
| 103 | `kiro-francescagiuseppecominetti-github-social` | 未记录（待确认） | 07-15 02:12:03 | 10-13 02:10:58 | 邮箱 `francescagiuseppecominetti@gmail.com`；已从历史会话确认 GitHub 经 Google 登录；按用户要求暂缓，真实 GitHub 用户名待核实 |
| 104 | `laohan185` | `Padbergsss` | 08-03 01:35:37 | 08-04 11:49:29 | [R:217](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L217) |

2026-10-15 续跑补查：本地历史会话 `019f4cdb-9b51-7a72-a0ca-167b1b6bb3d7` 第 7400/7718、7832、8078 行，分别明确调用 `skills/kiro-github-google-onboarder/scripts/onboard_kiro_github_via_google.py`，并将上述三个 Google 邮箱与同名网关逐一绑定。这三项应通过 GitHub 的 **Continue with Google** 入口进入，后续需提供 Google 登录凭据并在认证后核实真实 GitHub 用户名；本批统一 GitHub 密码不能直接视为 Google 密码。用户已要求先不处理这三项，CSV 改为“按用户要求暂缓”，尚未尝试登录。此处仅补充登录入口证据，不改写原快照按项目文档判定的历史成功结论。

## 核对结果（重登前快照）

| 范围 | 数量 | 说明 |
| --- | ---: | --- |
| Kiro 账号总数 | 354 | 本轮管理接口完整分页读取 200 + 154 条，账号名无重复，两页总数一致 |
| 快照中的停用账号 | 163 | 103 个 `401 Bad credentials`、58 个仅有停用标记、2 个 IDC refresh `invalid_grant` |
| 近期优先重登清单 | 73 | 2026-09-20 起仍有成功检查记录，快照中均为 `401 Bad credentials`；72 个 GitHub、1 个 Google |
| 其中：项目文档明确记录曾成功上号或恢复 | **37** | 恢复快照 27 个、上号追踪表 9 个、申诉追踪表已恢复项 1 个 |
| 其中：项目文档未找到明确成功记录 | **36** | 不能据此判断“从未上过号” |
| 较早异常或时间待确认清单 | 90 | 其中 `laohan185` 也有成功恢复记录；连同近期清单共确认 38 个曾上号成功 |

## 2026-10-12 核查与清单更新

本轮通过本地 AWS 订阅 `http://127.0.0.1:19182` 读取
`GET /admin/kiro-gateway/accounts?limit=200&offset=0` 和 `offset=200`，
两页接口生成时间为 22:38:56～22:38:57。管理接口汇总为 191 个启用、163 个停用；
191 个启用账号的状态缓存均为 `ready`，最新检查时间为 22:38:54。
本轮没有主动触发账号刷新或重新登录。

与 2026-10-11 02:16:07 的清单逐项比较：

- **新增 401 账号：0；新增其他异常账号：0；移出异常清单：0。**
- 103 个 401 账号的名单及问题分类一致，优先重登清单仍为 73 个，较早异常清单仍为 90 个。
- 两个 CSV 中有 **125 个账号的最近检查时间更新**，其中 71 个原已列为 401 的账号。
  优先重登清单更新 45 个账号的检查时间，较早异常清单更新 80 个。
  部分账号的缓存状态为 `disabled`，最近检查时间推进不能证明重新调用上游后再次收到 401，
  也不能当作首次异常时间。
- 已按最新快照更新两个 CSV 的最近检查时间和快照时间，保留原顺序、恢复进度、历史上号结论与身份映射。

另外查询了从初始快照到本轮快照的 Kiro 保留用量记录：
`GET /admin/llm-gateway/usage?provider_type=kiro&status_kind=non_ok&start_ms=1791656167000&end_ms=1791815937312&limit=200&offset=0`。
共返回 9 条失败请求，`has_more=false`，其中 3 条 HTTP 503、6 条 HTTP 499；这些记录的状态码和错误信息未见 401。
用量查询只覆盖已保留的请求记录，不包含全部账号状态/余额检查或每一次内部上游尝试，不能据此断言期间从未发生 401。
本轮确认的是：**当前账号管理接口没有比旧清单增加新的 401 账号。**

## 统计口径

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

重登前快照中的问题均为 `401 Bad credentials`，且 `disabled=true`；恢复后的结果见本次恢复进度及页首恢复记录。
序号沿用完整 CSV，时间均为 2026 年 UTC+08:00。GitHub 登录名“未记录”表示原项目文档没有单独确认该字段。

| CSV 序号 | 网关账号 | 邮箱 | 历史 GitHub 登录名 | 最后成功检查 | 历史证据 | 本次恢复进度 |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | `elestineaw` | `elestineaw@utexas.edu` | `elestineaw` | 10-11 01:05:10 | [R:50](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L50) | 已恢复，2026-10-12；公网独占请求验证通过 |
| 2 | `unfieldgt` | `unfield@utexas.edu` | `unfieldgt` | 10-11 01:03:25 | [R:49](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L49) | 已恢复，2026-10-12；公网独占请求验证通过 |
| 3 | `embrookew` | `embrooke@utexas.edu` | `embrookew` | 10-11 00:51:22 | [R:48](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L48) | 已恢复，2026-10-12；公网独占请求验证通过 |
| 4 | `verharta` | `verharta@utexas.edu` | `verharta` | 10-11 00:42:36 | [R:47](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L47) | 已恢复，2026-10-12；公网独占请求验证通过 |
| 5 | `bbottadq` | `bbottadq@utexas.edu` | `bbottadq` | 10-11 00:39:33 | [R:46](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L46) | 已恢复，2026-10-12；公网独占请求验证通过 |
| 6 | `haedradw` | `haedra@utexas.edu` | `haedradw` | 10-11 00:37:42 | [R:45](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L45) | 已重登，2026-10-12；剩余 0 credits，模型请求待核验 |
| 7 | `agnoliaqwd` | `agnolia@utexas.edu` | `agnoliaqwd` | 10-11 00:27:55 | [R:44](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L44) | 已重登，2026-10-12；公网独占请求验证通过 |
| 8 | `ddggte` | `reuiiisa@uchicago.edu` | `ddggte` | 10-10 23:55:51 | [R:43](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L43) | 已重登，2026-10-12；公网独占请求验证通过 |
| 9 | `shfordsdf` | `shfordsdf@utexas.edu` | `shfordsdf` | 10-09 02:20:21 | [R:42](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L42) | 已重登，2026-10-12；额度用尽，模型请求待核验 |
| 10 | `oorcrofta` | `oorcrofta@utexas.edu` | `oorcrofta` | 10-09 02:17:45 | [R:41](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L41) | 已重登，2026-10-12；公网独占请求验证通过 |
| 11 | `wqalerian` | `wqalerian@utexas.edu` | `wqalerian` | 10-09 02:08:43 | [R:40](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L40) | 已重登，2026-10-12；公网独占请求验证通过 |
| 12 | `tzpatric` | `tzpatrick@utexas.edu` | `tzpatric` | 10-09 02:02:57 | [R:39](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L39) | 已重登，2026-10-12；公网独占请求验证通过 |
| 13 | `laohan18` | `BiHankiro13@asu.edu` | `linkueisz18` | 10-09 01:55:05 | [R:38](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L38) | 已重登，2026-10-12；公网独占请求验证通过 |
| 14 | `laohan17` | `BiHankiro10@asu.edu` | `linkueisz17` | 10-09 01:43:18 | [R:37](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L37) | 已重登，2026-10-12；公网独占请求验证通过 |
| 15 | `laohan16` | `BiHankiro9@asu.edu` | `linkueisz16` | 10-09 01:41:33 | [R:36](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L36) | 已重登，2026-10-12；公网独占请求验证通过 |
| 16 | `laohan15` | `BiHankiro8@asu.edu` | `linkueisz15` | 10-09 01:34:48 | [R:35](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L35) | 已重登，2026-10-13；公网独占请求验证通过 |
| 17 | `laohan14` | `BiHankiro7@asu.edu` | `linkueisz14` | 10-09 01:21:44 | [R:34](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L34) | 已重登，2026-10-13；公网独占请求验证通过 |
| 18 | `laohan13` | `BiHankiro6@asu.edu` | `linkueisz13` | 10-09 01:19:58 | [R:33](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L33) | 已重登，2026-10-13；公网独占请求验证通过 |
| 19 | `laohan12` | `BiHankiro5@asu.edu` | `linkueisz12` | 10-09 01:14:48 | [R:32](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L32) | 已重登，2026-10-13；公网独占请求验证通过 |
| 20 | `laohan11` | `BiHankiro4@asu.edu` | `linkueisz11` | 10-09 01:12:18 | [R:31](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L31) | 已重登，2026-10-13；额度用尽，模型请求待核验 |
| 21 | `laohan10` | `BiHankiro3@asu.edu` | `linkueisz10` | 10-09 01:11:37 | [R:30](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L30) | 已重登，2026-10-13；公网独占请求验证通过 |
| 22 | `linkuei_3_laohan7` | `BiHankiro2@asu.edu` | `linkueisz9` | 10-09 01:02:33 | [R:29](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L29) | 已重登，2026-10-13；公网独占请求验证通过 |
| 23 | `laohan9` | `Jinbo.Pan@asu.edu` | `linkueisz8` | 10-09 00:56:45 | [R:28](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L28) | 已重登，2026-10-13；公网独占请求验证通过 |
| 24 | `laohan5` | `Calebkl2lis@asu.edu` | `linkueisz5` | 10-09 00:52:18 | [R:27](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L27) | 已重登，2026-10-13；公网独占请求验证通过 |
| 25 | `laohan6` | `Calebkl1lis@asu.edu` | `linkueisz4` | 10-09 00:47:20 | [R:26](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L26) | 已重登，2026-10-13；公网独占请求验证通过 |
| 26 | `laohan4` | `Nolan.arlisle@asu.edu` | `linkueisz2` | 10-09 00:45:09 | [R:25](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L25) | 已重登，2026-10-13；额度用尽，模型请求待核验 |
| 27 | `kiro-linkueisz3-github-social` | `Adrian.Carlisle@asu.edu` | `linkueisz3` | 10-09 00:35:49 | [R:24](../tmp/kiro-account-recovery/quota-snapshot-2026-07-11.md#L24) | 已重登，2026-10-13；公网独占请求验证通过 |
| 37 | `afanchard` | `afanchard@utexas.edu` | 未记录 | 10-05 23:48:23 | [G:71](github-kiro-account-onboarding-tracker.md#L71) | 已重登，2026-10-14；公网独占请求验证通过 |
| 38 | `allahaneas` | `allahaneas@utexas.edu` | 未记录 | 10-05 23:33:30 | [G:70](github-kiro-account-onboarding-tracker.md#L70) | 已重登，2026-10-14；公网独占请求验证通过 |
| 39 | `ennoxfs` | `ennoxfs@utexas.edu` | 未记录 | 10-05 23:30:51 | [G:69](github-kiro-account-onboarding-tracker.md#L69) | 已重登，2026-10-14；额度用尽，模型请求待核验 |
| 40 | `bbonsrfds` | `bbonsrfds@utexas.edu` | 未记录 | 10-05 23:25:24 | [G:68](github-kiro-account-onboarding-tracker.md#L68) | 已重登，2026-10-14；公网独占请求验证通过 |
| 41 | `ulligandw` | `ulligandw@utexas.edu` | 未记录 | 10-05 23:20:11 | [G:66](github-kiro-account-onboarding-tracker.md#L66) | 已重登，2026-10-14；公网独占请求验证通过 |
| 42 | `terhilder` | `terhilder@utexas.edu` | 未记录 | 10-05 23:19:24 | [G:67](github-kiro-account-onboarding-tracker.md#L67) | 已重登，2026-10-14；公网独占请求验证通过 |
| 43 | `adenrtheds` | `adenrtheds@utexas.edu` | 未记录 | 10-05 23:10:02 | [G:65](github-kiro-account-onboarding-tracker.md#L65) | 已重登，2026-10-14；公网独占请求验证通过 |
| 44 | `endrickgd` | `endrick@utexas.edu` | 未记录 | 10-05 23:07:53 | [G:64](github-kiro-account-onboarding-tracker.md#L64) | 已重登，2026-10-14；公网独占请求验证通过 |
| 45 | `onovansdf` | `onovansdf@utexas.edu` | 未记录 | 10-05 23:02:23 | [G:63](github-kiro-account-onboarding-tracker.md#L63) | 已重登，2026-10-14；公网独占请求验证通过 |
| 61 | `usmore` | `unsmore@utexas.edu` | `usmore` | 09-30 04:02:12 | [S:35](github-account-suspension-appeal-tracker.md#L35) | 已重登，2026-10-14；公网独占请求验证通过 |

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
| 28 | `atterfield` | `atterfield@utexas.edu` | GitHub | 10-08 00:10:49 | 已重登，2026-10-13；额度用尽，模型请求待核验 |
| 29 | `herwoodwe` | `herwood@utexas.edu` | GitHub | 10-07 23:46:51 | 已重登，2026-10-13；公网独占请求验证通过 |
| 30 | `tratton` | `tratton@utexas.edu` | GitHub | 10-07 23:43:17 | 已重登，2026-10-13；公网独占请求验证通过 |
| 31 | `allowaysfdf` | `allowaysfdf@utexas.edu` | GitHub | 10-07 23:38:44 | 已重登，2026-10-13；公网独占请求验证通过 |
| 32 | `errickds` | `errick@utexas.edu` | GitHub | 10-07 01:23:37 | 已重登，2026-10-13；公网独占请求验证通过 |
| 33 | `orsythe` | `orsythe@utexas.edu` | GitHub | 10-07 01:18:24 | 已重登，2026-10-13；公网独占请求验证通过 |
| 34 | `ennysondgw` | `ennyson@utexas.edu` | GitHub | 10-07 01:16:21 | 已重登，2026-10-13；公网独占请求验证通过 |
| 35 | `rowtherfdh` | `rowther@utexas.edu` | GitHub | 10-07 01:10:29 | 已重登，2026-10-13；公网独占请求验证通过 |
| 36 | `nderwood` | `nderwood@utexas.edu` | GitHub | 10-07 00:53:44 | 已重登，2026-10-13；公网独占请求验证通过 |
| 46 | `sadvass` | `sadvass@utexas.edu` | GitHub | 10-03 23:39:54 | 已重登，2026-10-14；公网独占请求验证通过 |
| 47 | `liupverg` | `liup44@utexas.edu` | GitHub | 10-03 23:37:36 | 已重登，2026-10-14；公网独占请求验证通过 |
| 48 | `uincey` | `mshawpl@utexas.edu` | GitHub | 10-03 23:36:07 | 已重登，2026-10-14；公网独占请求验证通过 |
| 49 | `finslow` | `inslow@utexas.edu` | GitHub | 10-03 23:18:33 | 已重登，2026-10-14；公网独占请求验证通过 |
| 50 | `enroseop` | `enrose@utexas.edu` | GitHub | 10-03 23:17:50 | 已重登，2026-10-14；公网独占请求验证通过 |
| 51 | `aveleyol` | `aveley@utexas.edu` | GitHub | 10-03 23:12:44 | 已重登，2026-10-14；公网独占请求验证通过 |
| 52 | `vanderto` | `vander@utexas.edu` | GitHub | 10-03 23:08:00 | 已重登，2026-10-14；公网独占请求验证通过 |
| 53 | `phyrurwe` | `phyrurwe@utexas.edu` | GitHub | 10-03 23:04:18 | 已重登，2026-10-14；公网独占请求验证通过 |
| 54 | `anthamt` | `anthamt@utexas.edu` | GitHub | 10-03 23:01:10 | 已重登，2026-10-14；公网独占请求验证通过 |
| 55 | `reshnderam` | `reshnderam@utexas.edu` | GitHub | 10-02 01:03:07 | 已重登，2026-10-14；公网独占请求验证通过 |
| 56 | `enwicks` | `enwick@utexas.edu` | GitHub | 09-30 23:15:27 | 已重登，2026-10-14；公网独占请求验证通过 |
| 57 | `kiro-spare-1` | `ttdevb@uchicago.edu` | GitHub | 09-30 23:11:24 | 已重登，2026-10-14；公网独占请求验证通过 |
| 58 | `fhhfdss` | `derrtf@uchicago.edu` | GitHub | 09-30 23:02:40 | 已重登，2026-10-14；公网独占请求验证通过 |
| 59 | `default` | `gerjjhs22@utexas.edu` | GitHub | 09-30 22:59:00 | 已重登，2026-10-14；公网独占请求验证通过 |
| 60 | `avenshades` | `avenshade@utexas.edu` | GitHub | 09-30 04:26:50 | 已重登，2026-10-14；公网独占请求验证通过 |
| 62 | `ockcharts` | 未记录 | GitHub | 09-29 21:45:33 | 已重登，2026-10-14；公网独占请求验证通过 |
| 63 | `hdsgvghh` | 未记录 | GitHub | 09-29 21:28:13 | 已重登，2026-10-14；公网独占请求验证通过 |
| 64 | `dfadgfgg` | 未记录 | GitHub | 09-29 21:21:47 | 已重登，2026-10-14；公网独占请求验证通过 |
| 65 | `fdaerwqrqr` | 未记录 | GitHub | 09-29 21:09:28 | 已重登，2026-10-14；公网独占请求验证通过 |
| 66 | `gghytr` | 未记录 | GitHub | 09-29 12:00:36 | 已重登，2026-10-14；公网独占请求验证通过 |
| 67 | `gdhsbvf` | 未记录 | GitHub | 09-29 11:55:23 | 已重登，2026-10-14；公网独占请求验证通过 |
| 68 | `gderghu` | 未记录 | GitHub | 09-29 11:53:59 | 已重登，2026-10-15；公网独占请求验证通过 |
| 69 | `gdergh` | 未记录 | GitHub | 09-29 11:50:00 | 已重登，2026-10-15；公网独占请求验证通过 |
| 70 | `jfderth` | 未记录 | GitHub | 09-29 11:31:34 | 已重登，2026-10-15；公网独占请求验证通过 |
| 71 | `ntagueik` | 未记录 | GitHub | 09-29 11:18:10 | 已重登，2026-10-15；公网独占请求验证通过 |
| 72 | `kiro-and163-google-social` | 未记录 | Google | 09-24 00:45:24 | 按用户要求忽略，2026-10-12 |
| 73 | `kiro-web-github` | 未记录 | GitHub | 09-20 15:53:34 | 失联，2026-10-15 按用户要求停止追查及重登；本轮余额未核验 |

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

| 账号 | 成功记录提交 | 上次导入时间 | 最后成功检查 | 2026-10-11 快照记录的最近失败检查 |
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

- [优先重登 CSV：104 项，31 项待处理](kiro-relogin-priority-2026-10-11.csv)
- [较早异常或时间待确认 CSV：90 个](kiro-disabled-older-or-undated-2026-10-11.csv)

两个 CSV 已增加“历史上号结论、历史 GitHub 用户名、历史记录邮箱、项目文档证据、历史备注”字段，
保留账号顺序、状态、代理及历史证据，检查时间按最新快照更新，便于筛选及逐项回填。
前两次快照核查只整理文档，没有修改账号；之后本轮实际恢复结果见页首“本轮恢复进度”。
2026-10-12 再次计算 73 个近期账号的导入至最后成功检查间隔，仍为 89.9701～89.9996 天；
上面的 90 天分析保留初始快照的失败检查时间，后续检查时间以更新后的 CSV 为准。
文档与 CSV 不保存密码、TOTP 密钥、一次性验证码或会话令牌。
