use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Event, EventSource, MessageEvent};
use yew::prelude::*;

use crate::{
    api::{
        build_admin_comment_ai_stream_url, fetch_admin_comment_task_ai_output,
        AdminCommentAiRunChunk, AdminCommentAiStreamEvent, AdminCommentTaskAiOutputResponse,
    },
    components::stream_chunk_batcher::ChunkBatcher,
    pages::{
        admin::shell::{
            admin_nav_href, navigate_to_admin_section, AdminNav, AdminPageHeader, AdminShell,
        },
        llm_access_shared::format_ms_iso,
    },
};

#[derive(Properties, Clone, PartialEq)]
pub struct AdminCommentRunsProps {
    pub task_id: String,
}

#[function_component(AdminCommentRunsPage)]
pub fn admin_comment_runs_page(props: &AdminCommentRunsProps) -> Html {
    let load_error = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let output = use_state(|| None::<AdminCommentTaskAiOutputResponse>);
    let selected_run_id = use_state(|| None::<String>);
    let stream_chunks = use_state(Vec::<AdminCommentAiRunChunk>::new);
    let stream_status = use_state(|| "idle".to_string());
    let stream_error = use_state(|| None::<String>);
    let stream_ref = use_mut_ref(|| {
        None::<(
            EventSource,
            Closure<dyn FnMut(MessageEvent)>,
            Closure<dyn FnMut(Event)>,
            // Batches chunks; dropping it cancels any pending flush.
            ChunkBatcher<AdminCommentAiRunChunk, String>,
        )>
    });

    let task_id = props.task_id.clone();

    {
        let task_id = task_id.clone();
        let load_error = load_error.clone();
        let loading = loading.clone();
        let output = output.clone();
        let selected_run_id = selected_run_id.clone();
        let stream_chunks = stream_chunks.clone();
        let stream_status = stream_status.clone();
        let stream_error = stream_error.clone();
        use_effect_with(task_id.clone(), move |id| {
            let id = id.clone();
            let load_error = load_error.clone();
            let loading = loading.clone();
            let output = output.clone();
            let selected_run_id = selected_run_id.clone();
            let stream_chunks = stream_chunks.clone();
            let stream_status = stream_status.clone();
            let stream_error = stream_error.clone();
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_comment_task_ai_output(&id, None, Some(2000)).await {
                    Ok(data) => {
                        selected_run_id.set(data.selected_run_id.clone());
                        stream_chunks.set(data.chunks.clone());
                        stream_status.set(
                            data.runs
                                .iter()
                                .find(|item| {
                                    Some(item.run_id.as_str()) == data.selected_run_id.as_deref()
                                })
                                .map(|item| item.status.clone())
                                .unwrap_or_else(|| "idle".to_string()),
                        );
                        stream_error.set(None);
                        output.set(Some(data));
                        load_error.set(None);
                    },
                    Err(err) => {
                        output.set(None);
                        stream_chunks.set(vec![]);
                        load_error.set(Some(format!("Failed to load AI runs: {}", err)));
                    },
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_select_run = {
        let task_id = task_id.clone();
        let output = output.clone();
        let selected_run_id = selected_run_id.clone();
        let stream_chunks = stream_chunks.clone();
        let stream_status = stream_status.clone();
        let stream_error = stream_error.clone();
        let load_error = load_error.clone();
        Callback::from(move |run_id: String| {
            let task_id = task_id.clone();
            let output = output.clone();
            let selected_run_id = selected_run_id.clone();
            let stream_chunks = stream_chunks.clone();
            let stream_status = stream_status.clone();
            let stream_error = stream_error.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_comment_task_ai_output(&task_id, Some(&run_id), Some(2000)).await
                {
                    Ok(data) => {
                        selected_run_id.set(Some(run_id.clone()));
                        stream_chunks.set(data.chunks.clone());
                        stream_status.set(
                            data.runs
                                .iter()
                                .find(|item| item.run_id == run_id)
                                .map(|item| item.status.clone())
                                .unwrap_or_else(|| "idle".to_string()),
                        );
                        stream_error.set(None);
                        output.set(Some(data));
                        load_error.set(None);
                    },
                    Err(err) => {
                        load_error.set(Some(format!("Failed to switch run: {}", err)));
                    },
                }
            });
        })
    };

    {
        let task_id = task_id.clone();
        let selected_run_id = selected_run_id.clone();
        let stream_chunks = stream_chunks.clone();
        let stream_status = stream_status.clone();
        let stream_error = stream_error.clone();
        let stream_ref = stream_ref.clone();
        use_effect_with((task_id.clone(), (*selected_run_id).clone()), move |(task_id, run_id)| {
            if let Some((source, _, _, batcher)) = stream_ref.borrow_mut().take() {
                source.close();
                batcher.cancel();
            }

            if let Some(run_id) = run_id.clone() {
                let stream_url = build_admin_comment_ai_stream_url(task_id, Some(&run_id), None);
                match EventSource::new(&stream_url) {
                    Ok(source) => {
                        stream_status.set("streaming".to_string());

                        // Batches token-level chunks so Yew renders at most ~10/sec.
                        let batcher = ChunkBatcher::new(
                            stream_chunks.clone(),
                            |c: &AdminCommentAiRunChunk| c.chunk_id.clone(),
                            |c: &AdminCommentAiRunChunk| c.batch_index,
                        );
                        let batcher_for_msg = batcher.clone();

                        let stream_status_setter = stream_status.clone();
                        let stream_error_setter = stream_error.clone();
                        let onmessage =
                            Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                                let Some(payload_text) = event.data().as_string() else {
                                    return;
                                };
                                let parsed = serde_json::from_str::<AdminCommentAiStreamEvent>(
                                    &payload_text,
                                );
                                let Ok(payload) = parsed else {
                                    return;
                                };
                                match payload.event_type.as_str() {
                                    "chunk" => {
                                        if let Some(chunk) = payload.chunk {
                                            batcher_for_msg.push(chunk);
                                        }
                                    },
                                    "done" => {
                                        stream_status_setter.set(
                                            payload
                                                .run_status
                                                .unwrap_or_else(|| "done".to_string()),
                                        );
                                    },
                                    "error" => {
                                        stream_status_setter.set("error".to_string());
                                        stream_error_setter
                                            .set(Some("Stream returned error event".to_string()));
                                    },
                                    _ => {},
                                }
                            });
                        source.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

                        let stream_status_setter = stream_status.clone();
                        let stream_error_setter = stream_error.clone();
                        let source_ref = source.clone();
                        let onerror = Closure::<dyn FnMut(Event)>::new(move |_| {
                            // Guard: ignore onerror if we already reached a terminal state
                            let current = (*stream_status_setter).clone();
                            if current == "done" || current == "success" || current == "error" {
                                return;
                            }
                            // readyState: 0=CONNECTING (browser auto-reconnecting), 2=CLOSED
                            if source_ref.ready_state() == 2 {
                                stream_status_setter.set("error".to_string());
                                stream_error_setter
                                    .set(Some("Stream connection closed".to_string()));
                            }
                            // CONNECTING (0) → browser is retrying, don't
                            // overwrite status
                        });
                        source.set_onerror(Some(onerror.as_ref().unchecked_ref()));

                        *stream_ref.borrow_mut() = Some((source, onmessage, onerror, batcher));
                    },
                    Err(err) => {
                        stream_status.set("error".to_string());
                        stream_error.set(Some(format!("Failed to open stream: {:?}", err)));
                    },
                }
            } else {
                stream_status.set("idle".to_string());
            }

            let stream_ref = stream_ref.clone();
            move || {
                if let Some((source, _, _, batcher)) = stream_ref.borrow_mut().take() {
                    source.close();
                    batcher.cancel();
                }
            }
        });
    }

    let runs = (*output)
        .as_ref()
        .map(|data| {
            data.runs
                .iter()
                .map(|run| RunEntry {
                    run_id: run.run_id.clone(),
                    status: run.status.clone(),
                    runner: run.runner_program.clone(),
                    exit_code: run.exit_code,
                    started_at: run.started_at,
                    completed_at: run.completed_at,
                    failure_reason: run.failure_reason.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let logs = log_entries(stream_chunks.iter().map(|chunk| {
        (chunk.stream.clone(), chunk.batch_index, chunk.created_at, chunk.content.clone())
    }));

    html! {
        <RunReader
            nav={AdminNav::CommentTasks}
            eyebrow="内容审核 / 评论任务"
            title="评论 AI 运行记录"
            description="查看该评论任务的每一次 AI 运行与实时输出；切换左侧运行可回看历史日志。"
            id_label="task_id"
            id_value={task_id}
            stream_status={(*stream_status).clone()}
            loading={*loading}
            load_error={(*load_error).clone()}
            stream_error={(*stream_error).clone()}
            runs={runs}
            selected_run_id={(*selected_run_id).clone()}
            on_select_run={Some(on_select_run)}
            empty_runs_hint="该评论任务还没有触发过 AI 运行。"
            logs={logs}
        />
    }
}

// ---------------------------------------------------------------------------
// Shared "run log reader" used by the comment / music-wish / article-request
// AI stream pages. Pure presentation: every page keeps its own fetch + SSE
// lifecycle and only hands normalized rows to this component.
// ---------------------------------------------------------------------------

/// One AI run as shown in the reader's run list.
#[derive(Clone, PartialEq)]
pub(crate) struct RunEntry {
    pub run_id: String,
    pub status: String,
    pub runner: String,
    pub exit_code: Option<i32>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub failure_reason: Option<String>,
}

/// One streamed output chunk as shown in the log view.
#[derive(Clone, PartialEq)]
pub(crate) struct LogEntry {
    pub stream: String,
    pub batch_index: i32,
    pub created_at: i64,
    pub content: String,
}

/// Normalizes `(stream, batch_index, created_at, content)` tuples into
/// log rows ordered by batch index (the order the runner emitted them).
pub(crate) fn log_entries(rows: impl Iterator<Item = (String, i32, i64, String)>) -> Vec<LogEntry> {
    let mut out = rows
        .map(|(stream, batch_index, created_at, content)| LogEntry {
            stream,
            batch_index,
            created_at,
            content,
        })
        .collect::<Vec<_>>();
    out.sort_by_key(|row| row.batch_index);
    out
}

/// Maps a raw run/stream status to an `.adm-pill` tone and a calm label.
pub(crate) fn run_status_pill(status: &str) -> (&'static str, String) {
    let lower = status.to_ascii_lowercase();
    let (tone, label) = match lower.as_str() {
        "streaming" => ("adm-pill--info", "接收中"),
        "running" | "processing" | "in_progress" => ("adm-pill--info", "运行中"),
        "pending" | "queued" | "waiting" => ("adm-pill--warn", "排队中"),
        "success" | "succeeded" | "done" | "completed" => ("adm-pill--ok", "已完成"),
        "failed" | "error" | "errored" => ("adm-pill--err", "失败"),
        "disconnected" => ("adm-pill--err", "连接断开"),
        "cancelled" | "canceled" => ("adm-pill--neutral", "已取消"),
        "idle" => ("adm-pill--neutral", "空闲"),
        _ => ("adm-pill--neutral", ""),
    };
    let label = if label.is_empty() { status.to_string() } else { label.to_string() };
    (tone, label)
}

fn status_pill(status: &str) -> Html {
    let (tone, label) = run_status_pill(status);
    html! {
        <span class={classes!("adm-pill", tone)} title={status.to_string()}>{ label }</span>
    }
}

fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

fn fmt_ts(ts: i64) -> String {
    if ts > 0 {
        format_ms_iso(ts)
    } else {
        "—".to_string()
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct RunReaderProps {
    /// Sidebar entry to highlight; the back link returns to its `/admin`
    /// hash section.
    pub nav: AdminNav,
    pub eyebrow: AttrValue,
    pub title: AttrValue,
    pub description: AttrValue,
    pub id_label: AttrValue,
    pub id_value: AttrValue,
    pub stream_status: AttrValue,
    pub loading: bool,
    pub load_error: Option<String>,
    pub stream_error: Option<String>,
    pub runs: Vec<RunEntry>,
    /// Run highlighted in the list; when absent the latest run is marked.
    pub selected_run_id: Option<String>,
    /// When present, run rows become buttons that switch the log view.
    #[prop_or_default]
    pub on_select_run: Option<Callback<String>>,
    pub empty_runs_hint: AttrValue,
    pub logs: Vec<LogEntry>,
}

#[function_component(RunReader)]
pub(crate) fn run_reader(props: &RunReaderProps) -> Html {
    let log_ref = use_node_ref();

    // Keep the log view pinned to the newest entry as tokens stream in.
    {
        let log_ref = log_ref.clone();
        use_effect_with(props.logs.len(), move |_| {
            if let Some(el) = log_ref.cast::<web_sys::HtmlElement>() {
                el.set_scroll_top(el.scroll_height());
            }
            || ()
        });
    }

    let on_back = {
        let nav = props.nav;
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            navigate_to_admin_section(nav, false);
        })
    };

    let highlighted = props
        .selected_run_id
        .clone()
        .or_else(|| props.runs.last().map(|run| run.run_id.clone()));
    let current_run = highlighted
        .as_ref()
        .and_then(|id| props.runs.iter().find(|run| &run.run_id == id))
        .cloned();
    let stream_status = props.stream_status.to_string();
    let is_live = matches!(stream_status.as_str(), "streaming" | "running");
    let stderr_count = props
        .logs
        .iter()
        .filter(|row| row.stream == "stderr")
        .count();

    let meta_item = |label: &'static str, value: Html| {
        html! {
            <div>
                <dt>{ label }</dt>
                <dd>{ value }</dd>
            </div>
        }
    };

    let summary = html! {
        <section class="adm-card stream-summary">
            <div class="adm-card__body">
                <div class="stream-summary__ident">
                    <span class="adm-eyebrow">{ props.id_label.clone() }</span>
                    <code class="stream-summary__id" title={props.id_value.clone()}>{ props.id_value.clone() }</code>
                    { status_pill(&stream_status) }
                </div>
                <dl class="adm-kv stream-summary__kv">
                    { meta_item("运行次数", html! { <span class="stream-mono">{ props.runs.len() }</span> }) }
                    { meta_item("日志片段", html! {
                        <span class="stream-mono">
                            { props.logs.len() }
                            if stderr_count > 0 {
                                <span class="stream-summary__err">{ format!(" · stderr {}", stderr_count) }</span>
                            }
                        </span>
                    }) }
                    { meta_item("当前运行", match current_run.as_ref() {
                        Some(run) => html! { <span class="stream-mono" title={run.run_id.clone()}>{ short_id(&run.run_id) }</span> },
                        None => html! { <span class="stream-faint">{ "—" }</span> },
                    }) }
                    { meta_item("执行器", match current_run.as_ref() {
                        Some(run) if !run.runner.is_empty() => html! { <span class="stream-mono stream-trunc" title={run.runner.clone()}>{ run.runner.clone() }</span> },
                        _ => html! { <span class="stream-faint">{ "—" }</span> },
                    }) }
                    { meta_item("开始时间", html! {
                        <span class="stream-mono">{ current_run.as_ref().map(|run| fmt_ts(run.started_at)).unwrap_or_else(|| "—".to_string()) }</span>
                    }) }
                    { meta_item("结束时间", html! {
                        <span class="stream-mono">{ current_run.as_ref().and_then(|run| run.completed_at).map(fmt_ts).unwrap_or_else(|| "—".to_string()) }</span>
                    }) }
                    { meta_item("退出码", match current_run.as_ref().and_then(|run| run.exit_code) {
                        Some(code) => html! {
                            <span class={classes!("stream-mono", (code != 0).then_some("stream-summary__err"))}>{ code }</span>
                        },
                        None => html! { <span class="stream-faint">{ "—" }</span> },
                    }) }
                </dl>
            </div>
        </section>
    };

    let banners = html! {
        <>
            if let Some(err) = props.load_error.clone() {
                <div class="adm-banner adm-banner--err" role="alert">{ err }</div>
            }
            if let Some(err) = props.stream_error.clone() {
                <div class="adm-banner adm-banner--err" role="alert">{ err }</div>
            }
            if let Some(reason) = current_run.as_ref().and_then(|run| run.failure_reason.clone()).filter(|r| !r.trim().is_empty()) {
                <div class="adm-banner adm-banner--warn" role="status">
                    <strong>{ "失败原因：" }</strong>{ reason }
                </div>
            }
        </>
    };

    let run_list = if props.loading && props.runs.is_empty() {
        html! {
            <div class="stream-runs__skel">
                <div class="adm-skel"></div>
                <div class="adm-skel"></div>
                <div class="adm-skel"></div>
            </div>
        }
    } else if props.runs.is_empty() {
        html! {
            <div class="stream-runs__empty">
                <i class="fas fa-inbox" aria-hidden="true"></i>
                <p class="stream-runs__empty-title">{ "暂无运行记录" }</p>
                <p class="stream-faint">{ props.empty_runs_hint.clone() }</p>
            </div>
        }
    } else {
        html! {
            <ul class="stream-runs">
                { for props.runs.iter().rev().map(|run| {
                    let active = highlighted.as_deref() == Some(run.run_id.as_str());
                    let body = html! {
                        <>
                            <span class="stream-run__top">
                                <code class="stream-run__id" title={run.run_id.clone()}>{ short_id(&run.run_id) }</code>
                                { status_pill(&run.status) }
                            </span>
                            <span class="stream-run__meta">
                                { fmt_ts(run.started_at) }
                                if let Some(code) = run.exit_code {
                                    { format!(" · exit {}", code) }
                                }
                            </span>
                        </>
                    };
                    let class = classes!("stream-run", active.then_some("is-active"));
                    html! {
                        <li key={run.run_id.clone()}>
                            if let Some(on_select) = props.on_select_run.clone() {
                                <button
                                    type="button"
                                    class={class}
                                    aria-pressed={active.to_string()}
                                    aria-label={format!("查看运行 {}", run.run_id)}
                                    onclick={{
                                        let run_id = run.run_id.clone();
                                        Callback::from(move |_| on_select.emit(run_id.clone()))
                                    }}
                                >
                                    { body }
                                </button>
                            } else {
                                <div class={class}>{ body }</div>
                            }
                        </li>
                    }
                }) }
            </ul>
        }
    };

    let log_view = if props.logs.is_empty() {
        html! {
            <div class="adm-code stream-log stream-log--empty">
                if props.loading {
                    <span>{ "正在加载日志…" }</span>
                } else if is_live {
                    <span class="stream-log__cursor">{ "等待输出" }</span>
                } else {
                    <span>{ "暂无输出。AI 运行开始后日志会实时出现在这里。" }</span>
                }
            </div>
        }
    } else {
        html! {
            <div ref={log_ref} class="adm-code stream-log" role="log" aria-live="polite">
                { for props.logs.iter().map(|row| {
                    let is_err = row.stream == "stderr";
                    html! {
                        <div class={classes!("stream-log__row", is_err.then_some("is-stderr"))}>
                            <span class="stream-log__gutter">
                                <span class="stream-log__idx">{ format!("#{}", row.batch_index) }</span>
                                <span class="stream-log__tag">{ row.stream.clone() }</span>
                                if row.created_at > 0 {
                                    <span class="stream-log__time">{ format_ms_iso(row.created_at) }</span>
                                }
                            </span>
                            <pre class="stream-log__text">{ row.content.clone() }</pre>
                        </div>
                    }
                }) }
                if is_live {
                    <div class="stream-log__cursor" aria-hidden="true"></div>
                }
            </div>
        }
    };

    html! {
        <AdminShell active={props.nav}>
            <AdminPageHeader
                eyebrow={props.eyebrow.clone()}
                title={props.title.clone()}
                description={props.description.clone()}
            >
                <a href={admin_nav_href(props.nav)} class="adm-btn adm-btn--ghost adm-btn--sm" onclick={on_back}>
                    <i class="fas fa-arrow-left" aria-hidden="true"></i>
                    <span>{ "返回队列" }</span>
                </a>
            </AdminPageHeader>
            <div class="stream-page">
                { summary }
                { banners }
                <div class="stream-reader">
                    <aside class="adm-card stream-reader__runs">
                        <div class="adm-card__header">
                            <h2 class="adm-card__title">{ "运行" }</h2>
                            <span class="stream-mono stream-faint">{ props.runs.len() }</span>
                        </div>
                        { run_list }
                    </aside>
                    <section class="adm-card stream-reader__log">
                        <div class="adm-card__header">
                            <h2 class="adm-card__title">{ "输出日志" }</h2>
                            <span class="stream-reader__live">
                                if is_live {
                                    <span class="stream-live-dot" aria-hidden="true"></span>
                                }
                                <span class="stream-mono stream-faint">{ format!("{} chunks", props.logs.len()) }</span>
                            </span>
                        </div>
                        { log_view }
                    </section>
                </div>
            </div>
        </AdminShell>
    }
}
