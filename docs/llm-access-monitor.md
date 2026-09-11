# llm-access 资源监视器

`llm-access-monitor` 是一个独立的 Linux binary。它不依赖 llm-access API、Neon 或 usage worker，直接读取 `/proc`、cgroup v2、`/sys` 和受限的本地文件系统统计，并将采样快照写入自己的 SQLite WAL 数据库。API 服务停止时，监视器仍能保留最后一次快照和历史。

默认监听 `127.0.0.1:19092`，默认每 5 秒采样并保留 24 小时。前端控制台的“可观测性 → 主机与进程”通过 Vite 同源代理访问它；`LLM_ACCESS_MONITOR_TARGET` 可将代理指向另一台主机的安全隧道。配置 `LLM_ACCESS_ADMIN_TOKEN` 后，API 要求 `x-admin-token`；未配置 token 时只允许 loopback 连接，非 loopback 监听会拒绝启动。

采集内容包括：整机 CPU 各状态、load、内存与 swap、上下文切换、fork、major fault、swap I/O、OOM、运行队列、CPU/内存/I/O PSI；llm-access 服务及其子进程的 CPU、RSS/HWM/PSS/USS、虚拟内存、线程、FD、I/O、fault、上下文切换、cgroup 限制和 throttling；物理块设备 I/O、网络接口收发包、白名单本地文件系统容量和进程出现/消失事件。计数器的首个采样点、回退或 PID 重用会返回 `null`，不会伪造速率。

## 构建与运行

```bash
export CARGO_TARGET_DIR=/mnt/wsl/data4tb/static-flow-data/cargo-target/llm-access
cargo build -p llm-access-monitor --release
sudo install -Dm755 "$CARGO_TARGET_DIR/release/llm-access-monitor" /opt/llm-access/bin/llm-access-monitor
sudo install -Dm644 deployment-examples/systemd/llm-access-monitor.service.template /etc/systemd/system/llm-access-monitor.service
sudo systemctl daemon-reload && sudo systemctl enable --now llm-access-monitor
```

开发时可用 `cargo run -p llm-access-monitor -- --once` 输出单个 JSON 快照。前端在 `apps/llm-access-frontend` 运行 `pnpm typecheck && pnpm build` 后，打开 `/console/system/resources`。
