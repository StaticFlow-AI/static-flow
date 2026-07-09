//! Usage-journal page (`/admin/llm-gateway/journal`).
//!
//! Standalone `.admin-shell` page showing the journal worker/import status and
//! a live preview of the current producer file, extracted from the mega llm
//! gateway panel. Polls status every 5s while mounted.

use gloo_timers::callback::Interval;
use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_llm_gateway::{
    copy_icon_button, format_optional_bytes, format_optional_duration_ms,
    usage_journal_preview_has_full_message, usage_journal_preview_message,
};
use crate::{
    api::{
        fetch_admin_usage_journal_preview, fetch_admin_usage_journal_status,
        AdminUsageJournalFileView, AdminUsageJournalPreviewResponse, AdminUsageJournalStatusView,
        ProcessMemoryRuntimeStats,
    },
    components::{copy_button::copy_to_clipboard, pagination::Pagination},
    pages::llm_access_shared::{
        first_token_latency_color, format_latency_ms, format_ms, format_number_u64,
        token_usage_missing_label, total_latency_color,
    },
    router::Route,
};

const JOURNAL_PREVIEW_PAGE_SIZE: usize = 20;

fn render_usage_journal_file_list(
    title: &str,
    files: &[AdminUsageJournalFileView],
    empty_label: &str,
) -> Html {
    html! {
        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--bg)]", "p-3")}>
            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>
                { title }
            </div>
            if files.is_empty() {
                <div class={classes!("mt-2", "text-xs", "text-[var(--muted)]")}>{ empty_label }</div>
            } else {
                <div class={classes!("mt-2", "space-y-2")}>
                    { for files.iter().map(|file| html! {
                        <div class={classes!("rounded-md", "border", "border-[var(--border)]", "px-2.5", "py-2")}>
                            <div class={classes!("flex", "items-center", "justify-between", "gap-2", "font-mono", "text-xs", "text-[var(--text)]")}>
                                <span>{ file.sequence.map(|seq| format!("#{seq}")).unwrap_or_else(|| file.file_name.clone()) }</span>
                                <span class={classes!("text-[var(--muted)]")}>{ format_optional_bytes(Some(file.bytes)) }</span>
                            </div>
                            <div class={classes!("mt-1", "break-all", "text-[11px]", "text-[var(--muted)]")}>{ file.path.clone() }</div>
                            <div class={classes!("mt-1", "text-[10px]", "text-[var(--muted)]")}>
                                { format!("age {}", format_optional_duration_ms(file.age_ms)) }
                            </div>
                        </div>
                    }) }
                </div>
            }
        </div>
    }
}

fn render_usage_journal_current_file_card(
    title: &str,
    file: Option<&AdminUsageJournalFileView>,
    empty_label: &str,
) -> Html {
    html! {
        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--bg)]", "p-3")}>
            <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ title }</div>
            if let Some(file) = file {
                <div class={classes!("mt-1", "font-mono", "text-lg", "font-bold")}>
                    { file.sequence.map(|seq| format!("#{seq}")).unwrap_or_else(|| file.file_name.clone()) }
                </div>
                <div class={classes!("text-xs", "text-[var(--muted)]")}>
                    { format_optional_bytes(Some(file.bytes)) }
                </div>
                <div class={classes!("mt-1", "break-all", "text-[10px]", "text-[var(--muted)]")}>
                    { file.path.clone() }
                </div>
            } else {
                <div class={classes!("mt-2", "text-xs", "text-[var(--muted)]")}>{ empty_label }</div>
            }
        </div>
    }
}

fn usage_worker_state_tone(state: &str) -> &'static str {
    match state {
        "idle" => "bg-emerald-500/12 text-emerald-700 dark:text-emerald-200",
        "importing" => "bg-sky-500/12 text-sky-700 dark:text-sky-200",
        "unreachable" => "bg-red-500/12 text-red-700 dark:text-red-200",
        _ => "bg-slate-500/12 text-slate-700 dark:text-slate-200",
    }
}

fn format_cgroup_memory_usage(memory: &ProcessMemoryRuntimeStats) -> String {
    match (memory.cgroup_current_bytes, memory.cgroup_max_bytes) {
        (Some(current), Some(max)) if max > 0 => {
            let percent = (current as f64 / max as f64 * 100.0).clamp(0.0, 999.0);
            format!(
                "{} / {} ({percent:.1}%)",
                format_optional_bytes(Some(current)),
                format_optional_bytes(Some(max))
            )
        },
        (Some(current), Some(max)) => format!(
            "{} / {}",
            format_optional_bytes(Some(current)),
            format_optional_bytes(Some(max))
        ),
        (Some(current), None) => format!("{} / limit -", format_optional_bytes(Some(current))),
        (None, Some(max)) => format!("- / {}", format_optional_bytes(Some(max))),
        (None, None) => "-".to_string(),
    }
}

fn format_relative_age_from_ms(now_ms: i64, timestamp_ms: Option<i64>) -> String {
    let age_ms = timestamp_ms.map(|timestamp| now_ms.saturating_sub(timestamp));
    format_optional_duration_ms(age_ms)
}

async fn tokio_like_join_usage_journal(
    preview_offset: usize,
) -> Result<(AdminUsageJournalStatusView, AdminUsageJournalPreviewResponse), String> {
    let status_fut = fetch_admin_usage_journal_status();
    let preview_fut =
        fetch_admin_usage_journal_preview(Some(JOURNAL_PREVIEW_PAGE_SIZE), Some(preview_offset));
    let (status, preview) = futures::future::join(status_fut, preview_fut).await;
    Ok((status?, preview?))
}

#[function_component(AdminLlmGatewayJournalPage)]
pub fn admin_llm_gateway_journal_page() -> Html {
    let usage_journal_status = use_state(|| None::<AdminUsageJournalStatusView>);
    let usage_journal_preview = use_state(|| None::<AdminUsageJournalPreviewResponse>);
    let usage_journal_preview_page = use_state(|| 1_usize);
    let selected_usage_journal_message = use_state(|| None::<(String, String, String, String)>);
    let usage_journal_loading = use_state(|| false);
    let usage_journal_error = use_state(|| None::<String>);
    let journal_filter_model = use_state(String::new);
    let journal_filter_account = use_state(String::new);
    let journal_filter_key = use_state(String::new);
    let journal_filter_status = use_state(String::new);
    let toast = use_state(|| None::<(String, bool)>);

    let notify = {
        let toast = toast.clone();
        Callback::from(move |(message, is_error): (String, bool)| {
            toast.set(Some((message, is_error)));
        })
    };
    let on_copy = {
        let notify = notify.clone();
        Callback::from(move |(label, value): (String, String)| {
            copy_to_clipboard(&value);
            notify.emit((format!("Copied {label} to clipboard."), false));
        })
    };

    let reload_usage_journal_status = {
        let usage_journal_status = usage_journal_status.clone();
        let usage_journal_preview = usage_journal_preview.clone();
        let usage_journal_preview_page = usage_journal_preview_page.clone();
        let usage_journal_loading = usage_journal_loading.clone();
        let usage_journal_error = usage_journal_error.clone();
        Callback::from(move |requested_page: Option<usize>| {
            let usage_journal_status = usage_journal_status.clone();
            let usage_journal_preview = usage_journal_preview.clone();
            let usage_journal_preview_page = usage_journal_preview_page.clone();
            let usage_journal_loading = usage_journal_loading.clone();
            let usage_journal_error = usage_journal_error.clone();
            let page = requested_page.unwrap_or(*usage_journal_preview_page).max(1);
            let offset = (page - 1) * JOURNAL_PREVIEW_PAGE_SIZE;
            usage_journal_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match tokio_like_join_usage_journal(offset).await {
                    Ok((status, preview)) => {
                        let actual_page = (preview.offset / preview.limit.max(1))
                            .saturating_add(1)
                            .max(1);
                        usage_journal_status.set(Some(status));
                        usage_journal_preview.set(Some(preview));
                        usage_journal_preview_page.set(actual_page);
                        usage_journal_error.set(None);
                    },
                    Err(err) => usage_journal_error.set(Some(err)),
                }
                usage_journal_loading.set(false);
            });
        })
    };

    // Poll journal status every 5s while this page is mounted; the interval is
    // dropped on unmount (navigating away).
    {
        let reload_usage_journal_status = reload_usage_journal_status.clone();
        use_effect_with((), move |_| {
            reload_usage_journal_status.emit(None);
            let interval = Interval::new(5_000, move || {
                reload_usage_journal_status.emit(None);
            });
            move || drop(interval)
        });
    }

    let on_usage_journal_preview_page_change = {
        let usage_journal_preview_page = usage_journal_preview_page.clone();
        let reload_usage_journal_status = reload_usage_journal_status.clone();
        Callback::from(move |page: usize| {
            usage_journal_preview_page.set(page);
            reload_usage_journal_status.emit(Some(page));
        })
    };

    let usage_journal_preview_total_pages = (*usage_journal_preview)
        .as_ref()
        .map(|resp| resp.total.max(1).div_ceil(resp.limit.max(1)))
        .unwrap_or(1);

    let usage_journal_message_modal = (*selected_usage_journal_message)
        .clone()
        .map(|(event_id, created_at, key_name, full_message)| {
            html! {
                <div
                    class={classes!(
                        "fixed",
                        "inset-0",
                        "z-[90]",
                        "flex",
                        "items-start",
                        "sm:items-center",
                        "justify-center",
                        "overflow-y-auto",
                        "bg-slate-950/58",
                        "backdrop-blur-sm",
                        "px-4",
                        "py-8"
                    )}
                    onclick={{
                        let selected_usage_journal_message = selected_usage_journal_message.clone();
                        Callback::from(move |_| selected_usage_journal_message.set(None))
                    }}
                >
                    <div
                        class={classes!(
                            "w-full",
                            "mx-auto",
                            "flex",
                            "max-h-[88vh]",
                            "max-w-3xl",
                            "flex-col",
                            "overflow-y-auto",
                            "rounded-xl",
                            "border",
                            "border-[var(--border)]",
                            "bg-[var(--surface)]",
                            "p-5",
                            "shadow-[0_16px_48px_rgba(0,0,0,0.2)]"
                        )}
                        onclick={Callback::from(|event: MouseEvent| event.stop_propagation())}
                    >
                        <div class={classes!("flex", "items-start", "justify-between", "gap-4", "flex-wrap", "shrink-0")}>
                            <div class={classes!("max-w-2xl")}>
                                <p class={classes!("m-0", "text-xs", "uppercase", "tracking-[0.18em]", "text-[var(--muted)]")}>{ "Journal Last Message" }</p>
                                <h2 class={classes!("mt-3", "text-xl", "font-black", "tracking-[-0.03em]")}>{ key_name.clone() }</h2>
                                <p class={classes!("mt-2", "m-0", "break-all", "text-sm", "leading-6", "text-[var(--muted)]")}>
                                    { format!("{created_at} · {event_id}") }
                                </p>
                            </div>
                            <div class={classes!("flex", "gap-2", "flex-wrap")}>
                                <button
                                    class={classes!("ghost")}
                                    onclick={{
                                        let on_copy = on_copy.clone();
                                        let full_message = full_message.clone();
                                        Callback::from(move |_| on_copy.emit(("Journal Last Message".to_string(), full_message.clone())))
                                    }}
                                >
                                    { "复制全文" }
                                </button>
                                <button
                                    class={classes!("primary")}
                                    onclick={{
                                        let selected_usage_journal_message = selected_usage_journal_message.clone();
                                        Callback::from(move |_| selected_usage_journal_message.set(None))
                                    }}
                                >
                                    { "关闭" }
                                </button>
                            </div>
                        </div>

                        <div class={classes!("mt-4")}>
                            <pre class={classes!(
                                "max-h-[62vh]",
                                "overflow-x-auto",
                                "overflow-y-auto",
                                "rounded-lg",
                                "bg-slate-950",
                                "p-3",
                                "text-xs",
                                "leading-6",
                                "text-amber-100",
                                "whitespace-pre-wrap",
                                "break-words"
                            )}>
                                { full_message }
                            </pre>
                        </div>
                    </div>
                </div>
            }
        });


    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "LLM Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Usage Journal" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminLlmGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                    </div>
                </header>

                <section class={classes!("grid", "gap-4", "min-w-0")}>
                    <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5", "min-w-0")}>
                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                            <div>
                                <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Usage Journal" }</h2>
                                <p class={classes!("mt-2", "mb-0", "text-sm", "text-[var(--muted)]")}>
                                    { "API writes active journal blocks locally; the worker seals and imports completed files into DuckDB. Live Preview only reads already-complete blocks from the current producer file." }
                                </p>
                            </div>
                            <button
                                class={classes!("primary")}
                                onclick={{
                                    let reload_usage_journal_status = reload_usage_journal_status.clone();
                                    Callback::from(move |_| reload_usage_journal_status.emit(None))
                                }}
                                disabled={*usage_journal_loading}
                            >
                                { if *usage_journal_loading { "刷新中..." } else { "刷新状态" } }
                            </button>
                        </div>

                        if let Some(status) = (*usage_journal_status).clone() {
                            <div class={classes!("mt-4", "grid", "gap-3", "md:grid-cols-2", "xl:grid-cols-6")}>
                                <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--bg)]", "p-3")}>
                                    <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "worker" }</div>
                                    <div class={classes!("mt-2", "flex", "items-center", "gap-2", "flex-wrap")}>
                                        <span class={classes!("rounded-full", "px-2.5", "py-1", "font-mono", "text-[11px]", "font-semibold", usage_worker_state_tone(&status.worker.state))}>
                                            { status.worker.state.clone() }
                                        </span>
                                        <span class={classes!("text-xs", "text-[var(--muted)]")}>
                                            { format!("heartbeat {}", format_optional_duration_ms(status.worker.heartbeat_age_ms)) }
                                        </span>
                                    </div>
                                </div>
                                <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--bg)]", "p-3")}>
                                    <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "worker memory" }</div>
                                    <div class={classes!("mt-1", "font-mono", "text-lg", "font-bold")}>
                                        { format_optional_bytes(status.worker.process_memory.rss_bytes) }
                                    </div>
                                    <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                        { format!("cgroup {}", format_cgroup_memory_usage(&status.worker.process_memory)) }
                                    </div>
                                    <div class={classes!("mt-1", "text-[10px]", "text-[var(--muted)]")}>
                                        { format!(
                                            "peak {} · swap {} / {}",
                                            format_optional_bytes(status.worker.process_memory.cgroup_peak_bytes),
                                            format_optional_bytes(status.worker.process_memory.cgroup_swap_current_bytes),
                                            format_optional_bytes(status.worker.process_memory.cgroup_swap_max_bytes),
                                        ) }
                                    </div>
                                </div>
                                <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--bg)]", "p-3")}>
                                    <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "sealed backlog" }</div>
                                    <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black", if status.sealed_file_count > 0 { "text-amber-600" } else { "text-emerald-600" })}>
                                        { status.sealed_file_count }
                                    </div>
                                    <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                        { format!("{} · oldest {}", format_optional_bytes(Some(status.sealed_bytes)), format_optional_duration_ms(status.oldest_sealed_age_ms)) }
                                    </div>
                                </div>
                                { render_usage_journal_current_file_card(
                                    "producer file",
                                    status.producer_current_file.as_ref(),
                                    "producer is not holding an active file",
                                ) }
                                { render_usage_journal_current_file_card(
                                    "worker file",
                                    status.current_consuming_file.as_ref(),
                                    "worker is not holding a consuming file",
                                ) }
                                <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--bg)]", "p-3")}>
                                    <div class={classes!("font-mono", "text-[11px]", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "import progress" }</div>
                                    <div class={classes!("mt-1", "font-mono", "text-lg", "font-bold")}>
                                        { format!("{:.1}%", status.worker.progress_percent) }
                                    </div>
                                    <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                        { format!(
                                            "{} / {} events · {} / {}",
                                            format_number_u64(status.worker.processed_events),
                                            format_number_u64(status.worker.total_events),
                                            format_optional_bytes(Some(status.worker.processed_compressed_bytes)),
                                            format_optional_bytes(Some(status.worker.total_compressed_bytes)),
                                        ) }
                                    </div>
                                </div>
                            </div>
                            <div class={classes!("mt-3", "grid", "gap-2", "text-xs", "text-[var(--muted)]", "xl:grid-cols-2")}>
                                <p class={classes!("m-0")}>
                                    { format!(
                                        "last_successful_import: {} · file {}",
                                        format_relative_age_from_ms(
                                            status.generated_at,
                                            status.worker.last_successful_import_at_ms,
                                        ),
                                        status
                                            .worker
                                            .last_successful_file_sequence
                                            .map(|seq| format!("#{seq}"))
                                            .unwrap_or_else(|| "-".to_string())
                                    ) }
                                </p>
                                <p class={classes!("m-0", "break-all")}>
                                    { format!("journal_root: {}", status.journal_root) }
                                </p>
                                <p class={classes!("m-0", "break-all")}>
                                    { format!("usage_query_base_url: {}", status.usage_query_base_url) }
                                </p>
                                if let Some(cluster) = status.cluster.as_ref() {
                                    <p class={classes!("m-0", "break-all")}>
                                        { format!(
                                            "cluster: node {} · class {} · role {} · usage {}",
                                            cluster.node_id,
                                            cluster.node_class,
                                            cluster.runtime_role,
                                            cluster.usage_query_mode,
                                        ) }
                                    </p>
                                    if let Some(primary_node_id) = cluster.primary_node_id.as_deref() {
                                        <p class={classes!("m-0", "break-all")}>
                                            { format!("primary_node_id: {primary_node_id}") }
                                        </p>
                                    }
                                    if let Some(primary_worker_base_url) = cluster.primary_worker_base_url.as_deref() {
                                        <p class={classes!("m-0", "break-all")}>
                                            { format!("primary_worker_base_url: {primary_worker_base_url}") }
                                        </p>
                                    }
                                }
                                if let Some(path) = status.worker.current_file_path.as_deref() {
                                    <p class={classes!("m-0", "break-all")}>
                                        { format!("current_file: {path}") }
                                    </p>
                                }
                                if let Some(error) = status.worker.last_error.as_deref() {
                                    <p class={classes!("m-0", "break-all", "text-red-600", "dark:text-red-300")}>
                                        { format!("worker_error: {error}") }
                                    </p>
                                }
                            </div>
                            <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-2")}>
                                { render_usage_journal_file_list("sealed files", &status.sealed_files, "no sealed backlog") }
                                { render_usage_journal_file_list("orphan consuming files", &status.orphan_consuming_files, "no orphan consuming files") }
                                { render_usage_journal_file_list("bad files", &status.bad_files, "no quarantined files") }
                                { render_usage_journal_file_list("orphan active files", &status.orphan_active_files, "no orphan active files") }
                            </div>
                        } else if let Some(error) = (*usage_journal_error).clone() {
                            <div class={classes!("mt-4", "rounded-lg", "border", "border-red-500/30", "bg-red-500/10", "p-3", "text-sm", "text-red-700", "dark:text-red-200")}>
                                { error }
                            </div>
                        } else {
                            <div class={classes!("mt-4", "text-sm", "text-[var(--muted)]")}>
                                { "尚未加载 usage journal 状态。" }
                            </div>
                        }
                    </section>

                    <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5", "min-w-0")}>
                        <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                            <div>
                                <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Live Preview" }</h2>
                                <p class={classes!("mt-2", "mb-0", "text-sm", "text-[var(--muted)]")}>
                                    { "Only the current producer file is previewed. Trailing partial writes are ignored until the next full block is flushed." }
                                </p>
                            </div>
                            <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                if let Some(status) = (*usage_journal_status).as_ref() {
                                    <span class={classes!("rounded-full", "border", "border-[var(--border)]", "px-3", "py-1", "text-xs", "font-semibold", "text-[var(--muted)]")}>
                                        { format!("RPM {}", status.current_rpm) }
                                    </span>
                                    <span class={classes!("rounded-full", "border", "border-[var(--border)]", "px-3", "py-1", "text-xs", "font-semibold", "text-[var(--muted)]")}>
                                        { format!("In Flight {}", status.current_in_flight) }
                                    </span>
                                }
                                if let Some(preview) = (*usage_journal_preview).as_ref().and_then(|view| view.preview.as_ref()) {
                                    <span class={classes!("rounded-full", "border", "border-[var(--border)]", "px-3", "py-1", "text-xs", "font-semibold", "text-[var(--muted)]")}>
                                        { format!("blocks {} · scanned {}", preview.complete_blocks, format_optional_bytes(Some(preview.bytes_scanned))) }
                                    </span>
                                }
                                <button
                                    class={classes!("ghost")}
                                    title="刷新预览"
                                    aria-label="刷新预览"
                                    onclick={{
                                        let reload_usage_journal_status = reload_usage_journal_status.clone();
                                        Callback::from(move |_| reload_usage_journal_status.emit(None))
                                    }}
                                    disabled={*usage_journal_loading}
                                >
                                    <i class={classes!("fas", if *usage_journal_loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                                </button>
                            </div>
                        </div>

                        if let Some(preview_response) = (*usage_journal_preview).clone() {
                            if let Some(preview) = preview_response.preview {
                                <div class={classes!("mt-3", "grid", "gap-2", "text-xs", "text-[var(--muted)]", "xl:grid-cols-2")}>
                                    <p class={classes!("m-0", "break-all")}>
                                        { format!("producer_current_file: {}", preview_response.producer_current_file.as_ref().map(|file| file.path.clone()).unwrap_or_else(|| "-".to_string())) }
                                    </p>
                                    <p class={classes!("m-0")}>
                                        { format!("truncated_tail: {}", if preview.truncated_tail { "yes" } else { "no" }) }
                                    </p>
                                </div>
                                <div class={classes!("mt-4", "flex", "flex-wrap", "items-center", "gap-2")}>
                                    <input
                                        type="text"
                                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "text-xs", "text-[var(--text)]", "placeholder:text-[var(--muted)]", "w-28")}
                                        placeholder="model"
                                        value={(*journal_filter_model).clone()}
                                        oninput={{
                                            let journal_filter_model = journal_filter_model.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                journal_filter_model.set(input.value());
                                            })
                                        }}
                                    />
                                    <input
                                        type="text"
                                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "text-xs", "text-[var(--text)]", "placeholder:text-[var(--muted)]", "w-28")}
                                        placeholder="account"
                                        value={(*journal_filter_account).clone()}
                                        oninput={{
                                            let journal_filter_account = journal_filter_account.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                journal_filter_account.set(input.value());
                                            })
                                        }}
                                    />
                                    <input
                                        type="text"
                                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "text-xs", "text-[var(--text)]", "placeholder:text-[var(--muted)]", "w-28")}
                                        placeholder="key"
                                        value={(*journal_filter_key).clone()}
                                        oninput={{
                                            let journal_filter_key = journal_filter_key.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                journal_filter_key.set(input.value());
                                            })
                                        }}
                                    />
                                    <select
                                        class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "text-xs", "text-[var(--text)]", "w-20")}
                                        onchange={{
                                            let journal_filter_status = journal_filter_status.clone();
                                            Callback::from(move |e: Event| {
                                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                journal_filter_status.set(select.value());
                                            })
                                        }}
                                    >
                                        <option value="" selected={journal_filter_status.is_empty()}>{ "All" }</option>
                                        <option value="2xx" selected={&**journal_filter_status == "2xx"}>{ "2xx" }</option>
                                        <option value="4xx" selected={&**journal_filter_status == "4xx"}>{ "4xx" }</option>
                                        <option value="5xx" selected={&**journal_filter_status == "5xx"}>{ "5xx" }</option>
                                    </select>
                                    {{
                                        let total = preview.events.len();
                                        let filtered_count = preview.events.iter().filter(|e| {
                                            (journal_filter_model.is_empty() || e.model.as_deref().unwrap_or("").contains(&**journal_filter_model))
                                            && (journal_filter_account.is_empty() || e.account_name.as_deref().unwrap_or("").contains(&**journal_filter_account))
                                            && (journal_filter_key.is_empty() || e.key_name.contains(&**journal_filter_key))
                                            && (journal_filter_status.is_empty() || match journal_filter_status.as_str() {
                                                "2xx" => e.status_code >= 200 && e.status_code < 300,
                                                "4xx" => e.status_code >= 400 && e.status_code < 500,
                                                "5xx" => e.status_code >= 500,
                                                _ => true,
                                            })
                                        }).count();
                                        html! {
                                            <span class={classes!("rounded-full", "border", "border-[var(--border)]", "px-2.5", "py-1", "text-[11px]", "font-semibold", "text-[var(--muted)]")}>
                                                { format!("{}/{}", filtered_count, total) }
                                            </span>
                                        }
                                    }}
                                </div>
                                <div class={classes!("mt-3", "min-w-0") }>
                                    <div class={classes!("overflow-x-auto", "max-w-full", "rounded-xl", "border", "border-[var(--border)]")}>
                                    <table class={classes!("min-w-[64rem]", "w-full", "text-sm")}>
                                        <thead>
                                            <tr class={classes!("text-left", "text-[var(--muted)]")}>
                                                <th class={classes!("py-2", "pl-3", "pr-3")}>{ "时间" }</th>
                                                <th class={classes!("py-2", "pr-3")}>{ "Key" }</th>
                                                <th class={classes!("py-2", "pr-3")}>{ "号池" }</th>
                                                <th class={classes!("py-2", "pr-3")}>{ "Model" }</th>
                                                <th class={classes!("py-2", "pr-3")}>{ "Status" }</th>
                                                <th class={classes!("py-2", "pr-3")}>{ "Latency" }</th>
                                                <th class={classes!("py-2", "pr-3")}>{ "Tokens" }</th>
                                                <th class={classes!("py-2", "pr-3")}>{ "" }</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {{
                                                let filtered_events: Vec<_> = preview.events.iter().filter(|e| {
                                                    (journal_filter_model.is_empty() || e.model.as_deref().unwrap_or("").contains(&**journal_filter_model))
                                                    && (journal_filter_account.is_empty() || e.account_name.as_deref().unwrap_or("").contains(&**journal_filter_account))
                                                    && (journal_filter_key.is_empty() || e.key_name.contains(&**journal_filter_key))
                                                    && (journal_filter_status.is_empty() || match journal_filter_status.as_str() {
                                                        "2xx" => e.status_code >= 200 && e.status_code < 300,
                                                        "4xx" => e.status_code >= 400 && e.status_code < 500,
                                                        "5xx" => e.status_code >= 500,
                                                        _ => true,
                                                    })
                                                }).collect();
                                                if filtered_events.is_empty() {
                                                    html! {
                                                        <tr class={classes!("border-t", "border-[var(--border)]")}>
                                                            <td colspan="8" class={classes!("py-8", "text-center", "text-[var(--muted)]")}>{ "当前 producer file 里还没有完整 block 可预览" }</td>
                                                        </tr>
                                                    }
                                                } else {
                                                    html! { { for filtered_events.into_iter().map(|event| {
                                                        let account_label = event.account_name.clone().unwrap_or_else(|| "not captured".to_string());
                                                        let last_message_full = usage_journal_preview_message(event);
                                                        let has_full_message = usage_journal_preview_has_full_message(event);
                                                        let open_preview_message = {
                                                            let selected_usage_journal_message = selected_usage_journal_message.clone();
                                                            let event_id = event.event_id.clone();
                                                            let created_at = format_ms(event.created_at_ms);
                                                            let key_name = event.key_name.clone();
                                                            let full_message = last_message_full.clone();
                                                            Callback::from(move |_| {
                                                                selected_usage_journal_message.set(Some((
                                                                    event_id.clone(),
                                                                    created_at.clone(),
                                                                    key_name.clone(),
                                                                    full_message.clone(),
                                                                )))
                                                            })
                                                        };
                                                        let latency_ms_val = event.latency_ms.unwrap_or(0) as i32;
                                                        let latency_color = total_latency_color(latency_ms_val);
                                                        let first_token = event.first_sse_write_ms.map(|first_ms| {
                                                            let first_ms = first_ms.clamp(0, i32::MAX as i64) as i32;
                                                            (first_ms, first_token_latency_color(first_ms))
                                                        });
                                                        let status_ok = event.status_code >= 200 && event.status_code < 300;
                                                        html! {
                                                            <tr class={classes!("border-t", "border-[var(--border)]", "align-top")}>
                                                                <td class={classes!("py-2.5", "pl-3", "pr-3", "whitespace-nowrap")}>
                                                                    <div class={classes!("text-xs")}>{ format_ms(event.created_at_ms) }</div>
                                                                    <div class={classes!("mt-0.5", "flex", "items-center", "gap-1")}>
                                                                        <span class={classes!("max-w-[7rem]", "truncate", "font-mono", "text-[10px]", "text-[var(--muted)]")} title={event.event_id.clone()}>
                                                                            { event.event_id.clone() }
                                                                        </span>
                                                                        { copy_icon_button(&event.event_id, &on_copy) }
                                                                    </div>
                                                                </td>
                                                                <td class={classes!("py-2.5", "pr-3")}>
                                                                    <div class={classes!("text-xs", "font-semibold", "text-[var(--text)]", "truncate", "max-w-[10rem]")} title={event.key_name.clone()}>{ event.key_name.clone() }</div>
                                                                    <div class={classes!("font-mono", "text-[10px]", "text-[var(--muted)]")}>{ event.key_id.clone() }</div>
                                                                </td>
                                                                <td class={classes!("py-2.5", "pr-3")}>
                                                                    <span class={classes!("inline-flex", "rounded-full", "border", "border-emerald-500/20", "bg-emerald-500/10", "px-2", "py-0.5", "text-[11px]", "font-semibold", "text-emerald-700", "dark:text-emerald-200")}>
                                                                        { account_label }
                                                                    </span>
                                                                </td>
                                                                <td class={classes!("py-2.5", "pr-3")}>
                                                                    <div class={classes!("text-xs", "truncate", "max-w-[10rem]")} title={event.model.clone().unwrap_or_default()}>
                                                                        { event.model.clone().unwrap_or_else(|| "-".to_string()) }
                                                                    </div>
                                                                    if event.usage_missing {
                                                                        <span class={classes!("inline-flex", "rounded-full", "border", "border-amber-500/20", "bg-amber-500/10", "px-1.5", "py-0.5", "text-[10px]", "font-semibold", "text-amber-700", "dark:text-amber-200")}>
                                                                            { token_usage_missing_label() }
                                                                        </span>
                                                                    }
                                                                </td>
                                                                <td class={classes!("py-2.5", "pr-3", "whitespace-nowrap")}>
                                                                    <span class={classes!(
                                                                        "inline-flex", "h-5", "w-5", "items-center", "justify-center", "rounded-full", "text-[10px]", "font-bold",
                                                                        if status_ok { "bg-emerald-500/15" } else { "bg-red-500/15" },
                                                                        if status_ok { "text-emerald-700" } else { "text-red-700" },
                                                                        if status_ok { "dark:text-emerald-200" } else { "dark:text-red-200" },
                                                                    )} title={format!("{}", event.status_code)}>
                                                                        { if status_ok { "" } else { "!" } }
                                                                    </span>
                                                                    <span class={classes!("ml-1", "text-xs", "font-mono")}>{ event.status_code }</span>
                                                                </td>
                                                                <td class={classes!("py-2.5", "pr-3", "whitespace-nowrap")}>
                                                                    if event.latency_ms.is_some() {
                                                                        <span class={classes!("inline-flex", "rounded-full", "border", "px-2", "py-0.5", "text-[11px]", "font-semibold", latency_color.0, latency_color.1, latency_color.2, latency_color.3)}>
                                                                            { format_latency_ms(latency_ms_val) }
                                                                        </span>
                                                                        <div class={classes!("mt-0.5")}>
                                                                            if let Some((first_ms, first_color)) = first_token {
                                                                                <span class={classes!("inline-flex", "rounded-full", "border", "px-1.5", "py-0.5", "text-[10px]", "font-semibold", first_color.0, first_color.1, first_color.2, first_color.3)}>
                                                                                    { format!("首字 {}", format_latency_ms(first_ms)) }
                                                                                </span>
                                                                            } else {
                                                                                <span class={classes!("text-[10px]", "text-[var(--muted)]")}>{ "首字 -" }</span>
                                                                            }
                                                                        </div>
                                                                    } else {
                                                                        <span class={classes!("text-xs", "text-[var(--muted)]")}>{ "-" }</span>
                                                                    }
                                                                </td>
                                                                <td class={classes!("py-2.5", "pr-3", "whitespace-nowrap", "font-mono", "text-[11px]")}>
                                                                    <span class={classes!("text-[var(--muted)]")}>
                                                                        { format!("{}/{}/{}", format_number_u64(event.input_uncached_tokens), format_number_u64(event.input_cached_tokens), format_number_u64(event.output_tokens)) }
                                                                    </span>
                                                                </td>
                                                                <td class={classes!("py-2.5", "pr-3")}>
                                                                    if has_full_message {
                                                                        <button
                                                                            type="button"
                                                                            class={classes!(
                                                                                "inline-flex",
                                                                                "h-7",
                                                                                "w-7",
                                                                                "items-center",
                                                                                "justify-center",
                                                                                "rounded-lg",
                                                                                "border",
                                                                                "border-[var(--border)]",
                                                                                "bg-[var(--surface)]",
                                                                                "text-[var(--muted)]",
                                                                                "transition-colors",
                                                                                "hover:text-[var(--primary)]",
                                                                                "hover:bg-[var(--surface-alt)]"
                                                                            )}
                                                                            title="查看最后一条内容"
                                                                            aria-label="查看最后一条内容"
                                                                            onclick={open_preview_message}
                                                                        >
                                                                            <i class={classes!("fas", "fa-bars-staggered", "text-xs")}></i>
                                                                        </button>
                                                                    }
                                                                </td>
                                                            </tr>
                                                        }
                                                    }) }}
                                                }
                                            }}
                                        </tbody>
                                    </table>
                                    </div>
                                </div>
                                <div class={classes!("mt-5", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                    <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                        { format!("第 {} 页 · {} 条", *usage_journal_preview_page, preview.total_events) }
                                    </div>
                                    <Pagination
                                        current_page={*usage_journal_preview_page}
                                        total_pages={usage_journal_preview_total_pages}
                                        on_page_change={on_usage_journal_preview_page_change.clone()}
                                    />
                                </div>
                            } else {
                                <div class={classes!("mt-4", "text-sm", "text-[var(--muted)]")}>
                                    { "当前还没有 producer file 可预览。" }
                                </div>
                            }
                        } else if let Some(error) = (*usage_journal_error).clone() {
                            <div class={classes!("mt-4", "rounded-lg", "border", "border-red-500/30", "bg-red-500/10", "p-3", "text-sm", "text-red-700", "dark:text-red-200")}>
                                { error }
                            </div>
                        } else {
                            <div class={classes!("mt-4", "text-sm", "text-[var(--muted)]")}>
                                { "尚未加载实时预览。" }
                            </div>
                        }
                    </section>
                </section>

            </div>
            { usage_journal_message_modal.unwrap_or_default() }
            if let Some((message, is_error)) = (*toast).clone() {
                <div class={classes!("toasts")}>
                    <div class={classes!("toast", if is_error { "error" } else { "ok" })}>
                        { message }
                    </div>
                </div>
            }
        </main>
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn journal_preview_layout_uses_compact_table_width_and_toolbar_badges() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/pages/admin_llm_gateway_journal.rs"
        ));
        let journal_start = source
            .find("\"Live Preview\"")
            .expect("journal preview header");
        let journal_slice = &source[journal_start..source.len().min(journal_start + 12000)];

        assert!(journal_slice.contains("min-w-[64rem]"));
        assert!(journal_slice.contains("RPM {}"));
        assert!(journal_slice.contains("In Flight {}"));
        assert!(journal_slice.contains("刷新预览"));
    }
}
