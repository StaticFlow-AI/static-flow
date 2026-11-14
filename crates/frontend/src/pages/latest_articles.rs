use std::collections::BTreeMap;

use gloo_timers::callback::Timeout;
use static_flow_shared::ArticleListItem;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    window, Event, EventSource, HtmlElement, HtmlInputElement, HtmlTextAreaElement, MessageEvent,
};
use yew::prelude::*;
use yew_router::prelude::{use_location, Link};

use crate::{
    api::{
        self, build_admin_article_request_ai_stream_url, fetch_admin_article_request_ai_output,
        ArticleRequestAiRunChunk, ArticleRequestAiRunRecord, ArticleRequestItem,
    },
    components::{
        article_card::{ArticleCard, ArticleCardVariant},
        pagination::Pagination,
        raw_html::RawHtml,
        scroll_to_top_button::ScrollToTopButton,
        stream_chunk_batcher::ChunkBatcher,
    },
    i18n::{
        current::{article_request as ar_t, latest_articles_page as t},
        fill_one,
    },
    pages::posts::render_archive_skeleton,
    router::Route,
    utils::markdown_to_html,
};

const PAGE_SIZE: usize = 12;
const REQUEST_PAGE_SIZE: usize = 8;

type NamedEventClosure = Closure<dyn FnMut(MessageEvent)>;
type ErrorClosure = Closure<dyn FnMut(Event)>;

struct RequestStreamHandle {
    source: EventSource,
    _on_chunk: NamedEventClosure,
    _on_done: NamedEventClosure,
    _on_stream_error: NamedEventClosure,
    _on_error: ErrorClosure,
    batcher: ChunkBatcher<ArticleRequestAiRunChunk, (i32, String)>,
}

impl Drop for RequestStreamHandle {
    fn drop(&mut self) {
        self.source.close();
        self.batcher.cancel();
    }
}

fn smooth_scroll_to_id(id: &str) {
    let Some(win) = window() else {
        return;
    };
    let Some(doc) = win.document() else {
        return;
    };
    let Some(el) = doc.get_element_by_id(id) else {
        return;
    };
    let rect = el.get_bounding_client_rect();
    let scroll_y = win.scroll_y().unwrap_or(0.0);
    let target_top = (rect.top() + scroll_y - 72.0).max(0.0);
    let opts = web_sys::ScrollToOptions::new();
    opts.set_top(target_top);
    opts.set_behavior(web_sys::ScrollBehavior::Smooth);
    win.scroll_with_scroll_to_options(&opts);
}

fn format_timestamp_short(ts_ms: i64) -> String {
    if ts_ms <= 0 {
        return String::new();
    }
    let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ts_ms as f64));
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        d.get_full_year(),
        d.get_month() + 1,
        d.get_date(),
        d.get_hours(),
        d.get_minutes()
    )
}

fn strip_url_display(raw: &str) -> String {
    let trimmed = raw
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_end_matches('/');
    if trimmed.is_empty() {
        raw.to_string()
    } else {
        trimmed.to_string()
    }
}

fn article_request_has_article(req: &ArticleRequestItem) -> bool {
    req.ingested_article_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|id| !id.is_empty())
}

fn article_request_done_without_article(req: &ArticleRequestItem) -> bool {
    req.status == "done" && !article_request_has_article(req)
}

fn request_status_class_and_text(req: &ArticleRequestItem) -> (&'static str, &str) {
    if article_request_done_without_article(req) {
        ("sf-status--pending", ar_t::STATUS_DONE_NO_ARTICLE)
    } else {
        match req.status.as_str() {
            "done" => ("sf-status--done", ar_t::STATUS_DONE),
            "running" => ("sf-status--running", ar_t::STATUS_RUNNING),
            "failed" => ("sf-status--failed", ar_t::STATUS_FAILED),
            "approved" => ("sf-status--pending", ar_t::STATUS_APPROVED),
            "pending" => ("sf-status--pending", ar_t::STATUS_PENDING),
            other => ("sf-status--pending", other),
        }
    }
}

#[derive(Properties, PartialEq)]
struct RequestAiStreamPanelProps {
    pub req: ArticleRequestItem,
}

#[function_component(RequestAiStreamPanel)]
fn request_ai_stream_panel(props: &RequestAiStreamPanelProps) -> Html {
    let req = &props.req;
    let request_id = req.request_id.clone();
    let is_running = req.status == "running";

    let loading = use_state(|| true);
    let runs = use_state(Vec::<ArticleRequestAiRunRecord>::new);
    let chunks = use_state(Vec::<ArticleRequestAiRunChunk>::new);
    let stream_status = use_state(|| req.status.clone());
    let access_restricted = use_state(|| false);
    let stream_ref = use_mut_ref(|| None::<RequestStreamHandle>);
    let chunk_scroll_ref = use_node_ref();

    // Fetch existing AI runs & chunks when expanded
    {
        let request_id = request_id.clone();
        let loading = loading.clone();
        let runs = runs.clone();
        let chunks = chunks.clone();
        let stream_status = stream_status.clone();
        let access_restricted = access_restricted.clone();
        use_effect_with(request_id, move |id| {
            let id = id.clone();
            loading.set(true);
            access_restricted.set(false);
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_article_request_ai_output(&id).await {
                    Ok(data) => {
                        if let Some(latest) = data.runs.last() {
                            stream_status.set(latest.status.clone());
                        }
                        chunks.set(data.chunks);
                        runs.set(data.runs);
                    },
                    Err(_) => {
                        access_restricted.set(true);
                    },
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Open SSE live chunk stream when running and accessible
    {
        let request_id = request_id.clone();
        let chunks = chunks.clone();
        let stream_status = stream_status.clone();
        let stream_ref = stream_ref.clone();
        let should_stream = is_running && !*access_restricted;
        use_effect_with((request_id, should_stream), move |(wid, active)| {
            if let Some(prev) = stream_ref.borrow_mut().take() {
                drop(prev);
            }

            if *active {
                let stream_url = build_admin_article_request_ai_stream_url(wid);
                if let Ok(source) = EventSource::new(&stream_url) {
                    stream_status.set("running".to_string());
                    let batcher = ChunkBatcher::new(
                        chunks.clone(),
                        |c: &ArticleRequestAiRunChunk| (c.batch_index, c.stream.clone()),
                        |c: &ArticleRequestAiRunChunk| c.batch_index,
                    );
                    let batcher_for_msg = batcher.clone();

                    let on_chunk =
                        Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                            let Some(text) = event.data().as_string() else {
                                return;
                            };
                            let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) else {
                                return;
                            };
                            let stream = val["stream"].as_str().unwrap_or("stdout").to_string();
                            let batch_index = val["batch_index"].as_i64().unwrap_or(0) as i32;
                            let content = val["content"].as_str().unwrap_or("").to_string();
                            batcher_for_msg.push(ArticleRequestAiRunChunk {
                                chunk_id: format!("live-{stream}-{batch_index}"),
                                run_id: String::new(),
                                request_id: String::new(),
                                stream,
                                batch_index,
                                content,
                                created_at: 0,
                            });
                        });

                    let on_done = {
                        let stream_status = stream_status.clone();
                        Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                            let status = event
                                .data()
                                .as_string()
                                .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                                .and_then(|v| v["status"].as_str().map(String::from))
                                .unwrap_or_else(|| "done".to_string());
                            stream_status.set(status);
                        })
                    };

                    let on_stream_error = {
                        let stream_status = stream_status.clone();
                        Closure::<dyn FnMut(MessageEvent)>::new(move |_: MessageEvent| {
                            stream_status.set("failed".to_string());
                        })
                    };

                    let on_error = {
                        let stream_status = stream_status.clone();
                        let source_ref = source.clone();
                        Closure::<dyn FnMut(Event)>::new(move |_| {
                            let current = (*stream_status).clone();
                            if current == "success" || current == "done" || current == "failed" {
                                return;
                            }
                            if source_ref.ready_state() == 2 {
                                stream_status.set("closed".to_string());
                            }
                        })
                    };

                    let _ = source.add_event_listener_with_callback(
                        "chunk",
                        on_chunk.as_ref().unchecked_ref(),
                    );
                    let _ = source
                        .add_event_listener_with_callback("done", on_done.as_ref().unchecked_ref());
                    let _ = source.add_event_listener_with_callback(
                        "error",
                        on_stream_error.as_ref().unchecked_ref(),
                    );
                    source.set_onerror(Some(on_error.as_ref().unchecked_ref()));

                    *stream_ref.borrow_mut() = Some(RequestStreamHandle {
                        source,
                        _on_chunk: on_chunk,
                        _on_done: on_done,
                        _on_stream_error: on_stream_error,
                        _on_error: on_error,
                        batcher,
                    });
                }
            }

            let stream_ref = stream_ref.clone();
            move || {
                stream_ref.borrow_mut().take();
            }
        });
    }

    let sorted_chunks = {
        let mut rows = (*chunks).clone();
        rows.sort_by_key(|c| c.batch_index);
        rows
    };

    // Pin scroll container to newest chunk
    {
        let chunk_scroll_ref = chunk_scroll_ref.clone();
        use_effect_with(sorted_chunks.len(), move |_| {
            if let Some(el) = chunk_scroll_ref.cast::<HtmlElement>() {
                el.set_scroll_top(el.scroll_height());
            }
            || ()
        });
    }

    html! {
        <div class="space-y-3">
            <div class="flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] text-[var(--faint)]">
                <span>
                    { format!("AI RUN LOG · status={}", *stream_status) }
                    if !runs.is_empty() {
                        { format!(" · runs={}", runs.len()) }
                    }
                </span>
                if !sorted_chunks.is_empty() {
                    <span>{ format!("{} chunks", sorted_chunks.len()) }</span>
                }
            </div>

            <div
                ref={chunk_scroll_ref}
                class="rounded-[10px] border border-[var(--border)] bg-[var(--surface-sunk)] p-4 font-mono text-[0.8125rem] leading-[1.6] text-[var(--text-2)] max-h-80 overflow-y-auto"
            >
                if *loading {
                    <div class="text-xs text-[var(--faint)]">{ "正在读取 AI 执行日志…" }</div>
                } else if !sorted_chunks.is_empty() {
                    <div class="space-y-1.5">
                        { for sorted_chunks.into_iter().map(|chunk| {
                            let is_err = chunk.stream == "stderr";
                            html! {
                                <div key={chunk.chunk_id.clone()} class="whitespace-pre-wrap break-words">
                                    if is_err {
                                        <span class="text-[var(--err)] mr-1.5">{ "[stderr]" }</span>
                                    }
                                    <span class={classes!(is_err.then_some("text-[var(--err)]"))}>
                                        { chunk.content }
                                    </span>
                                </div>
                            }
                        }) }
                    </div>
                } else {
                    <div class="space-y-1 text-xs text-[var(--muted)]">
                        <div>{ format!("[status] {}", req.status) }</div>
                        if req.created_at > 0 {
                            <div>{ format!("[queued_at] {}", format_timestamp_short(req.created_at)) }</div>
                        }
                        if req.updated_at > 0 {
                            <div>{ format!("[updated_at] {}", format_timestamp_short(req.updated_at)) }</div>
                        }
                        if let Some(reason) = req.failure_reason.as_deref().filter(|s| !s.trim().is_empty()) {
                            <div class="text-[var(--err)]">{ format!("[failure] {reason}") }</div>
                        } else if *access_restricted {
                            <div class="text-[var(--faint)]">
                                { "[stream] 实时原始终端流仅对本地 / 管理员会话开放；公开执行摘要与 AI 回复见下方。" }
                            </div>
                        } else {
                            <div class="text-[var(--faint)]">
                                { "[stream] 尚无流式分片输出，任务启动后将在此处滚动展示。" }
                            </div>
                        }
                    </div>
                }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct RequestRowProps {
    pub req: ArticleRequestItem,
    pub on_follow_up: Callback<ArticleRequestItem>,
    #[prop_or_default]
    pub parent_summary: Option<AttrValue>,
}

#[function_component(RequestRow)]
fn request_row(props: &RequestRowProps) -> Html {
    let req = &props.req;
    let expanded = use_state(|| req.status == "running");

    let (status_modifier, status_text) = request_status_class_and_text(req);

    let toggle_expand = {
        let expanded = expanded.clone();
        Callback::from(move |_: MouseEvent| expanded.set(!*expanded))
    };

    let on_follow_up_click = {
        let cb = props.on_follow_up.clone();
        let req = req.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            cb.emit(req.clone());
        })
    };

    let display_title = req
        .title_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| strip_url_display(&req.article_url));

    let ai_rendered = req.ai_reply.as_deref().map(markdown_to_html);

    html! {
        <li class="py-4 border-b border-[var(--border)] last:border-b-0">
            <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-center gap-2">
                        <span class={classes!("sf-status", status_modifier)}>
                            { status_text }
                        </span>
                        if req.parent_request_id.is_some() {
                            <span class="sf-chip !h-5 !px-2 !text-[10px] !bg-[var(--primary-soft)] !text-[var(--primary)]">
                                { ar_t::FOLLOW_UP_BADGE }
                            </span>
                        }
                        <button
                            type="button"
                            onclick={toggle_expand.clone()}
                            class="font-serif text-[0.98rem] text-left text-[var(--text)] hover:text-[var(--primary)] bg-transparent border-0 p-0 cursor-pointer truncate max-w-full transition-colors"
                        >
                            { display_title }
                        </button>
                    </div>

                    <div class="mt-1.5 flex flex-wrap items-center gap-x-2.5 gap-y-1 font-mono text-[11px] tabular-nums text-[var(--faint)]">
                        <span>{ &req.nickname }</span>
                        if req.created_at > 0 {
                            <span aria-hidden="true">{ "·" }</span>
                            <span>{ format_timestamp_short(req.created_at) }</span>
                        }
                        if !req.ip_region.trim().is_empty() {
                            <span aria-hidden="true">{ "·" }</span>
                            <span>{ &req.ip_region }</span>
                        }
                    </div>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                    if req.status == "done" {
                        if let Some(aid) = req
                            .ingested_article_id
                            .as_deref()
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                        {
                            <Link<Route>
                                to={Route::ArticleDetail { id: aid.to_string() }}
                                classes="font-mono text-xs text-[var(--primary)] hover:underline no-underline"
                            >
                                { ar_t::VIEW_ARTICLE }
                            </Link<Route>>
                        }
                    }

                    <button
                        type="button"
                        onclick={toggle_expand}
                        aria-expanded={(*expanded).to_string()}
                        class="sf-btn sf-btn--ghost sf-btn--sm !px-2 font-mono !text-xs"
                    >
                        { if *expanded { "收起 ↑" } else { "AI 流 / 详情 ↓" } }
                    </button>
                </div>
            </div>

            if *expanded {
                <div class="mt-4 pl-0 sm:pl-2 space-y-4">
                    if let Some(ref summary) = props.parent_summary {
                        <div class="inline-flex items-center gap-2 rounded-[6px] bg-[var(--primary-soft)] px-2.5 py-1 text-xs text-[var(--primary)]">
                            <span>{ "↳" }</span>
                            <span>{ summary }</span>
                        </div>
                    }

                    if article_request_done_without_article(req) {
                        <div class="rounded-[8px] bg-[var(--warn-soft)] px-3.5 py-2.5 text-xs leading-relaxed text-[var(--warn)]">
                            { ar_t::NO_ARTICLE_NOTICE }
                        </div>
                    }

                    <div class="space-y-1.5 text-xs">
                        <div class="font-mono uppercase tracking-[0.08em] text-[var(--faint)]">
                            { ar_t::LABEL_URL }
                        </div>
                        <a
                            href={req.article_url.clone()}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="sf-link break-all text-sm text-[var(--link)]"
                        >
                            { &req.article_url }
                        </a>
                    </div>

                    if !req.request_message.trim().is_empty() {
                        <div class="space-y-1 text-xs">
                            <div class="font-mono uppercase tracking-[0.08em] text-[var(--faint)]">
                                { ar_t::LABEL_MESSAGE }
                            </div>
                            <p class="m-0 whitespace-pre-wrap text-sm leading-relaxed text-[var(--text-2)]">
                                { &req.request_message }
                            </p>
                        </div>
                    }

                    // Live / Historical AI Run Chunks Log
                    <RequestAiStreamPanel req={req.clone()} />

                    // AI Reply Markdown (if completed)
                    if let Some(rendered) = ai_rendered {
                        <div class="space-y-1.5">
                            <div class="font-mono text-xs uppercase tracking-[0.08em] text-[var(--faint)]">
                                { ar_t::LABEL_AI_REPLY }
                            </div>
                            <div class="prose prose-sm max-w-none rounded-[10px] border border-[var(--border)] bg-[var(--surface)] p-4 text-sm text-[var(--text)]">
                                <RawHtml html={AttrValue::from(rendered)} />
                            </div>
                        </div>
                    }

                    if req.status == "done" {
                        <div class="pt-1 flex items-center justify-end">
                            <button
                                type="button"
                                onclick={on_follow_up_click}
                                class="sf-btn sf-btn--secondary sf-btn--sm"
                            >
                                { ar_t::FOLLOW_UP_BTN }
                            </button>
                        </div>
                    }
                </div>
            }
        </li>
    }
}

#[function_component(LatestArticlesPage)]
pub fn latest_articles_page() -> Html {
    let route_location = use_location();
    let articles = use_state(Vec::<ArticleListItem>::new);
    let loading = use_state(|| true);
    let current_page = use_state(|| 1_usize);
    let total = use_state(|| 0_usize);
    let fetch_seq = use_mut_ref(|| 0_u64);
    let load_error = use_state(|| None::<String>);
    let grid_refresh_tick = use_state(|| 0_u32);

    // Article request workshop state (placed at top of /latest)
    let workshop_open = use_state(|| true);
    let ar_requests = use_state(Vec::<ArticleRequestItem>::new);
    let ar_loading = use_state(|| false);
    let ar_page = use_state(|| 1_usize);
    let ar_total = use_state(|| 0_usize);
    let ar_list_error = use_state(|| None::<String>);
    let ar_form_url = use_state(String::new);
    let ar_form_title = use_state(String::new);
    let ar_form_message = use_state(String::new);
    let ar_form_nickname = use_state(String::new);
    let ar_form_email = use_state(String::new);
    let ar_submitting = use_state(|| false);
    let ar_submit_error = use_state(|| None::<String>);
    let ar_submit_ok = use_state(|| None::<String>);
    let ar_refresh_seq = use_state(|| 0_u32);
    let ar_parent_request = use_state(|| None::<ArticleRequestItem>);

    // Auto-scroll when URL hash is #article-request-section
    {
        let workshop_open = workshop_open.clone();
        let loc_dep = route_location.clone();
        use_effect_with(loc_dep, move |_| {
            let check_hash = {
                let workshop_open = workshop_open.clone();
                move || {
                    let hash = window()
                        .and_then(|w| w.location().hash().ok())
                        .unwrap_or_default();
                    if hash == "#article-request-section" {
                        workshop_open.set(true);
                        smooth_scroll_to_id("article-request-section");
                        Timeout::new(90, || smooth_scroll_to_id("article-request-section"))
                            .forget();
                    }
                }
            };
            check_hash();

            let check_on_hash = check_hash.clone();
            let on_hash = Closure::<dyn FnMut(Event)>::new(move |_| check_on_hash());
            let on_pop = Closure::<dyn FnMut(Event)>::new(move |_| check_hash());

            if let Some(win) = window() {
                let _ = win.add_event_listener_with_callback(
                    "hashchange",
                    on_hash.as_ref().unchecked_ref(),
                );
                let _ = win
                    .add_event_listener_with_callback("popstate", on_pop.as_ref().unchecked_ref());
            }

            move || {
                if let Some(win) = window() {
                    let _ = win.remove_event_listener_with_callback(
                        "hashchange",
                        on_hash.as_ref().unchecked_ref(),
                    );
                    let _ = win.remove_event_listener_with_callback(
                        "popstate",
                        on_pop.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    let total_pages = {
        let t = *total;
        if t == 0 {
            1
        } else {
            t.div_ceil(PAGE_SIZE)
        }
    };
    let current_page_num = *current_page;

    // Fetch paginated articles
    {
        let articles = articles.clone();
        let loading = loading.clone();
        let total = total.clone();
        let fetch_seq = fetch_seq.clone();
        let load_error = load_error.clone();
        let page = *current_page;
        let tick = *grid_refresh_tick;
        use_effect_with((page, tick), move |(page, _)| {
            let offset = (*page - 1) * PAGE_SIZE;
            let request_id = {
                let mut seq = fetch_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            loading.set(true);
            load_error.set(None);
            let articles = articles.clone();
            let loading = loading.clone();
            let total = total.clone();
            let fetch_seq = fetch_seq.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::fetch_articles(None, None, Some(PAGE_SIZE), Some(offset)).await {
                    Ok(data) => {
                        if *fetch_seq.borrow() != request_id {
                            return;
                        }
                        total.set(data.total);
                        articles.set(data.articles);
                        load_error.set(None);
                    },
                    Err(e) => {
                        if *fetch_seq.borrow() != request_id {
                            return;
                        }
                        web_sys::console::error_1(
                            &format!("Failed to fetch articles: {}", e).into(),
                        );
                        load_error.set(Some(e.to_string()));
                    },
                }
                if *fetch_seq.borrow() != request_id {
                    return;
                }
                loading.set(false);
            });
            || ()
        });
    }

    let save_scroll_position = {
        let location = route_location.clone();
        Callback::from(move |_| {
            if crate::navigation_context::is_return_armed() {
                return;
            }
            let mut state = BTreeMap::new();
            state.insert("page".to_string(), current_page_num.to_string());
            if let Some(loc) = location.as_ref() {
                state.insert("location".to_string(), loc.path().to_string());
            }
            crate::navigation_context::save_context_for_current_page(state);
        })
    };

    {
        let location = route_location.clone();
        use_effect_with((location, current_page_num), move |_| {
            let mut on_scroll_opt: Option<Closure<dyn FnMut(Event)>> = None;

            if !crate::navigation_context::is_return_armed() {
                let persist = move || {
                    let mut state = BTreeMap::new();
                    state.insert("page".to_string(), current_page_num.to_string());
                    crate::navigation_context::save_context_for_current_page(state);
                };

                persist();

                let on_scroll = Closure::wrap(Box::new(move |_: Event| {
                    if crate::navigation_context::is_return_armed() {
                        return;
                    }
                    let mut state = BTreeMap::new();
                    state.insert("page".to_string(), current_page_num.to_string());
                    crate::navigation_context::save_context_for_current_page(state);
                }) as Box<dyn FnMut(_)>);

                if let Some(win) = window() {
                    let _ = win.add_event_listener_with_callback(
                        "scroll",
                        on_scroll.as_ref().unchecked_ref(),
                    );
                }

                on_scroll_opt = Some(on_scroll);
            }

            move || {
                if let Some(on_scroll) = on_scroll_opt {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "scroll",
                            on_scroll.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }

    {
        let current_page = current_page.clone();
        let location_dep = route_location.clone();
        let article_len = articles.len();
        use_effect_with((location_dep, article_len), move |_| {
            if article_len > 0 {
                if let Some(context) =
                    crate::navigation_context::pop_context_if_armed_for_current_page()
                {
                    if let Some(page_num) = context
                        .page_state
                        .get("page")
                        .and_then(|raw| raw.parse::<usize>().ok())
                    {
                        current_page.set(page_num);
                    }

                    let scroll_y = context.scroll_y.max(0.0);
                    Timeout::new(140, move || {
                        if let Some(win) = window() {
                            win.scroll_to_with_x_and_y(0.0, scroll_y);
                        }
                    })
                    .forget();
                }
            }
            || ()
        });
    }

    let go_to_page = {
        let current_page = current_page.clone();
        Callback::from(move |page: usize| {
            current_page.set(page);
            smooth_scroll_to_id("latest-articles-stream");
        })
    };

    let pagination_controls = if total_pages > 1 {
        html! {
            <div class={classes!("mt-12", "flex", "justify-center")}>
                <Pagination
                    current_page={current_page_num}
                    total_pages={total_pages}
                    on_page_change={go_to_page}
                />
            </div>
        }
    } else {
        Html::default()
    };

    // Fetch article requests queue
    let ar_total_pages = {
        let t = *ar_total;
        if t == 0 {
            1
        } else {
            t.div_ceil(REQUEST_PAGE_SIZE)
        }
    };
    {
        let ar_requests = ar_requests.clone();
        let ar_loading = ar_loading.clone();
        let ar_total = ar_total.clone();
        let ar_list_error = ar_list_error.clone();
        let page = *ar_page;
        let seq = *ar_refresh_seq;
        use_effect_with((page, seq), move |(page, _seq)| {
            let offset = (*page - 1) * REQUEST_PAGE_SIZE;
            ar_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_article_requests(Some(REQUEST_PAGE_SIZE), Some(offset)).await {
                    Ok(data) => {
                        ar_total.set(data.total);
                        ar_requests.set(data.requests);
                        ar_list_error.set(None);
                    },
                    Err(e) => {
                        ar_list_error.set(Some(e));
                    },
                }
                ar_loading.set(false);
            });
            || ()
        });
    }

    let on_ar_page_change = {
        let ar_page = ar_page.clone();
        Callback::from(move |page: usize| ar_page.set(page))
    };

    let on_ar_refresh = {
        let ar_refresh_seq = ar_refresh_seq.clone();
        Callback::from(move |_: MouseEvent| {
            ar_refresh_seq.set(ar_refresh_seq.wrapping_add(1));
        })
    };

    let on_follow_up = {
        let ar_parent_request = ar_parent_request.clone();
        let ar_form_url = ar_form_url.clone();
        let workshop_open = workshop_open.clone();
        let ar_submit_ok = ar_submit_ok.clone();
        let ar_submit_error = ar_submit_error.clone();
        Callback::from(move |req: ArticleRequestItem| {
            ar_form_url.set(req.article_url.clone());
            ar_parent_request.set(Some(req));
            ar_submit_ok.set(None);
            ar_submit_error.set(None);
            workshop_open.set(true);
            smooth_scroll_to_id("article-request-section");
        })
    };

    let on_cancel_follow_up = {
        let ar_parent_request = ar_parent_request.clone();
        let ar_form_url = ar_form_url.clone();
        Callback::from(move |_: MouseEvent| {
            ar_parent_request.set(None);
            ar_form_url.set(String::new());
        })
    };

    let has_parent = ar_parent_request.is_some();

    let on_ar_submit = {
        let ar_form_url = ar_form_url.clone();
        let ar_form_title = ar_form_title.clone();
        let ar_form_message = ar_form_message.clone();
        let ar_form_nickname = ar_form_nickname.clone();
        let ar_form_email = ar_form_email.clone();
        let ar_submitting = ar_submitting.clone();
        let ar_submit_error = ar_submit_error.clone();
        let ar_submit_ok = ar_submit_ok.clone();
        let ar_page = ar_page.clone();
        let ar_refresh_seq = ar_refresh_seq.clone();
        let ar_parent_request = ar_parent_request.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if *ar_submitting {
                return;
            }
            let url = (*ar_form_url).trim().to_string();
            let title = (*ar_form_title).trim().to_string();
            let message = (*ar_form_message).trim().to_string();
            let nickname = (*ar_form_nickname).trim().to_string();
            let email = (*ar_form_email).trim().to_string();
            let parent_id = (*ar_parent_request).as_ref().map(|r| r.request_id.clone());

            if parent_id.is_none()
                && (url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")))
            {
                ar_submit_error.set(Some(
                    "请填写完整的文章链接（需以 https:// 或 http:// 开头）。".to_string(),
                ));
                return;
            }
            if message.is_empty() {
                ar_submit_error.set(Some("请填写推荐理由或希望 AI 关注的解读角度。".to_string()));
                return;
            }

            ar_submit_error.set(None);
            ar_submit_ok.set(None);
            ar_submitting.set(true);

            let ar_submitting = ar_submitting.clone();
            let ar_submit_error = ar_submit_error.clone();
            let ar_submit_ok = ar_submit_ok.clone();
            let ar_form_url = ar_form_url.clone();
            let ar_form_title = ar_form_title.clone();
            let ar_form_message = ar_form_message.clone();
            let ar_page = ar_page.clone();
            let ar_refresh_seq = ar_refresh_seq.clone();
            let ar_parent_request = ar_parent_request.clone();
            let next_seq = ar_refresh_seq.wrapping_add(1);
            let frontend_page_url = window().and_then(|w| w.location().href().ok());

            wasm_bindgen_futures::spawn_local(async move {
                let title_opt = if title.is_empty() { None } else { Some(title.as_str()) };
                let nick_opt = if nickname.is_empty() { None } else { Some(nickname.as_str()) };
                let email_opt = if email.is_empty() { None } else { Some(email.as_str()) };
                match api::submit_article_request(
                    &url,
                    title_opt,
                    &message,
                    nick_opt,
                    email_opt,
                    frontend_page_url.as_deref(),
                    parent_id.as_deref(),
                )
                .await
                {
                    Ok(resp) => {
                        ar_submit_ok.set(Some(format!(
                            "{}（request_id: {}）",
                            ar_t::SUBMIT_SUCCESS,
                            resp.request_id
                        )));
                        ar_form_url.set(String::new());
                        ar_form_title.set(String::new());
                        ar_form_message.set(String::new());
                        ar_page.set(1);
                        ar_refresh_seq.set(next_seq);
                        ar_parent_request.set(None);
                    },
                    Err(err) => {
                        ar_submit_error.set(Some(err));
                    },
                }
                ar_submitting.set(false);
            });
        })
    };

    let toggle_workshop = {
        let workshop_open = workshop_open.clone();
        Callback::from(move |_: MouseEvent| {
            workshop_open.set(!*workshop_open);
        })
    };

    let jump_to_workshop = {
        let workshop_open = workshop_open.clone();
        Callback::from(move |_: MouseEvent| {
            workshop_open.set(true);
            smooth_scroll_to_id("article-request-section");
        })
    };

    let has_running_req = ar_requests.iter().any(|r| r.status == "running");
    let presets =
        ["双语精译并梳理核心架构机制", "保留源码细节与工程权衡对比", "提炼可复现结论与关键数据"];

    html! {
        <main class={classes!("min-h-[65vh]", "pb-24")}>
            // 1. Anthropic Editorial Page Header
            <div class={classes!("max-w-[1200px]", "mx-auto", "px-6", "md:px-10")}>
                <header class="pt-16 pb-10">
                    <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                        { "LATEST DISPATCHES" }
                    </div>
                    <div class="mt-3 flex flex-wrap items-end justify-between gap-4">
                        <div>
                            <h1 class="m-0 font-serif font-normal text-[clamp(2rem,1.5rem+2vw,3rem)] leading-[1.15] tracking-[-0.015em] text-[var(--text)]">
                                { t::HERO_TITLE }
                            </h1>
                            <p class="mt-3 mb-0 text-[0.95rem] leading-[1.65] text-[var(--muted)] max-w-[60ch]">
                                { t::HERO_DESC }
                            </p>
                        </div>

                        <div class="flex flex-wrap items-center gap-3">
                            <span class="font-mono text-xs tabular-nums text-[var(--faint)]">
                                if *total > 0 {
                                    { format!("{} 篇文章 · {} 条入库队列", *total, *ar_total) }
                                }
                            </span>
                            <button
                                type="button"
                                onclick={jump_to_workshop}
                                class="sf-btn sf-btn--secondary sf-btn--sm"
                            >
                                { ar_t::SECTION_TITLE }
                                <span aria-hidden="true">{ "↓" }</span>
                            </button>
                        </div>
                    </div>
                </header>
            </div>

            // 2. Top Article Request Ingestion Band (#article-request-section)
            <section
                id="article-request-section"
                class="w-full bg-[var(--surface-alt)] border-y border-[var(--border)] py-10 md:py-14 scroll-mt-16"
                aria-labelledby="ar-section-heading"
            >
                <div class="max-w-[1200px] mx-auto px-6 md:px-10">
                    <div class="flex flex-wrap items-center justify-between gap-4">
                        <div class="flex flex-wrap items-center gap-3">
                            <div>
                                <div class="flex items-center gap-2.5">
                                    <span class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                                        { "READER INGESTION QUEUE & AI STREAM" }
                                    </span>
                                    if has_running_req {
                                        <span class="inline-flex items-center gap-1.5 font-mono text-[11px] font-semibold uppercase tracking-[0.08em] text-[var(--primary)]">
                                            <span class="w-1.5 h-1.5 rounded-full bg-[var(--primary)] animate-pulse motion-reduce:animate-none" aria-hidden="true" />
                                            { "LIVE" }
                                        </span>
                                    }
                                </div>
                                <h2
                                    id="ar-section-heading"
                                    class="mt-1.5 mb-0 font-serif font-normal text-[clamp(1.4rem,1.2rem+0.8vw,1.85rem)] leading-[1.2] tracking-[-0.015em] text-[var(--text)]"
                                >
                                    { ar_t::SECTION_TITLE }
                                </h2>
                                <p class="mt-1 mb-0 text-sm leading-relaxed text-[var(--muted)]">
                                    { ar_t::SECTION_SUBTITLE }
                                </p>
                            </div>
                        </div>

                        <div class="flex items-center gap-2">
                            <button
                                type="button"
                                onclick={on_ar_refresh}
                                disabled={*ar_loading}
                                class="sf-btn sf-btn--secondary sf-btn--sm disabled:opacity-50"
                            >
                                { if *ar_loading { ar_t::REFRESHING } else { ar_t::REFRESH_BTN } }
                            </button>
                            <button
                                type="button"
                                onclick={toggle_workshop}
                                aria-expanded={(*workshop_open).to_string()}
                                class="sf-btn sf-btn--ghost sf-btn--sm"
                            >
                                { if *workshop_open { "收起面板 ↑" } else { "展开投稿与队列 ↓" } }
                            </button>
                        </div>
                    </div>

                    if *workshop_open {
                        <div class="mt-8 grid grid-cols-1 lg:grid-cols-[0.95fr_1.05fr] gap-8 lg:gap-10 items-start">
                            // Left: Editorial Submission Card (matching HomePage Workshop)
                            <div class="sf-card p-6 sm:p-7 bg-[var(--surface)] border border-[var(--border)] rounded-[12px]">
                                <form onsubmit={on_ar_submit} class="space-y-4">
                                    if has_parent {
                                        <div class="flex items-center justify-between gap-3 rounded-[8px] bg-[var(--primary-soft)] px-3.5 py-2.5 text-xs text-[var(--primary)]">
                                            <span class="font-medium">
                                                { format!("↳ {}", ar_t::FOLLOW_UP_INDICATOR) }
                                            </span>
                                            <button
                                                type="button"
                                                onclick={on_cancel_follow_up}
                                                class="sf-btn sf-btn--ghost sf-btn--sm !h-6 !px-2 !text-xs"
                                            >
                                                { ar_t::CANCEL_FOLLOW_UP }
                                            </button>
                                        </div>
                                    }

                                    <div class="flex flex-col gap-1.5">
                                        <label for="ar-url" class="text-xs font-medium text-[var(--text-2)]">
                                            { ar_t::URL_LABEL }
                                            if !has_parent {
                                                { " *" }
                                            }
                                        </label>
                                        <input
                                            id="ar-url"
                                            type="url"
                                            placeholder={ar_t::URL_PLACEHOLDER}
                                            value={(*ar_form_url).clone()}
                                            oninput={
                                                let s = ar_form_url.clone();
                                                let err = ar_submit_error.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    s.set(input.value());
                                                    err.set(None);
                                                })
                                            }
                                            class="sf-input w-full"
                                            required={!has_parent}
                                        />
                                    </div>

                                    <div class="flex flex-col gap-1.5">
                                        <div class="flex flex-wrap items-center justify-between gap-2">
                                            <label for="ar-message" class="text-xs font-medium text-[var(--text-2)]">
                                                { format!("{} *", ar_t::MESSAGE_LABEL) }
                                            </label>
                                            <div class="flex flex-wrap gap-1.5">
                                                { for presets.into_iter().map(|preset| {
                                                    let ar_form_message = ar_form_message.clone();
                                                    let ar_submit_error = ar_submit_error.clone();
                                                    let onclick = Callback::from(move |_: MouseEvent| {
                                                        ar_form_message.set(preset.to_string());
                                                        ar_submit_error.set(None);
                                                    });
                                                    html! {
                                                        <button
                                                            type="button"
                                                            {onclick}
                                                            class="sf-chip !h-6 !px-2.5 !text-[11px] cursor-pointer border-0"
                                                        >
                                                            { format!("+ {preset}") }
                                                        </button>
                                                    }
                                                }) }
                                            </div>
                                        </div>
                                        <textarea
                                            id="ar-message"
                                            placeholder={ar_t::MESSAGE_PLACEHOLDER}
                                            value={(*ar_form_message).clone()}
                                            oninput={
                                                let s = ar_form_message.clone();
                                                let err = ar_submit_error.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: HtmlTextAreaElement = e.target_unchecked_into();
                                                    s.set(input.value());
                                                    err.set(None);
                                                })
                                            }
                                            rows="3"
                                            class="sf-textarea w-full min-h-[88px] py-2.5 resize-y"
                                            required=true
                                        />
                                    </div>

                                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                                        <div class="flex flex-col gap-1">
                                            <label for="ar-title" class="text-xs text-[var(--muted)]">
                                                { ar_t::TITLE_HINT_LABEL }
                                            </label>
                                            <input
                                                id="ar-title"
                                                type="text"
                                                placeholder={ar_t::TITLE_HINT_PLACEHOLDER}
                                                value={(*ar_form_title).clone()}
                                                oninput={
                                                    let s = ar_form_title.clone();
                                                    Callback::from(move |e: InputEvent| {
                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                        s.set(input.value());
                                                    })
                                                }
                                                class="sf-input w-full"
                                            />
                                        </div>
                                        <div class="flex flex-col gap-1">
                                            <label for="ar-nickname" class="text-xs text-[var(--muted)]">
                                                { ar_t::NICKNAME_LABEL }
                                            </label>
                                            <input
                                                id="ar-nickname"
                                                type="text"
                                                placeholder="匿名读者"
                                                value={(*ar_form_nickname).clone()}
                                                oninput={
                                                    let s = ar_form_nickname.clone();
                                                    Callback::from(move |e: InputEvent| {
                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                        s.set(input.value());
                                                    })
                                                }
                                                class="sf-input w-full"
                                            />
                                        </div>
                                        <div class="flex flex-col gap-1">
                                            <label for="ar-email" class="text-xs text-[var(--muted)]">
                                                { ar_t::EMAIL_LABEL }
                                            </label>
                                            <input
                                                id="ar-email"
                                                type="email"
                                                placeholder="完成通知邮箱"
                                                value={(*ar_form_email).clone()}
                                                oninput={
                                                    let s = ar_form_email.clone();
                                                    Callback::from(move |e: InputEvent| {
                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                        s.set(input.value());
                                                    })
                                                }
                                                class="sf-input w-full"
                                            />
                                        </div>
                                    </div>

                                    if let Some(ok_msg) = (*ar_submit_ok).clone() {
                                        <div class="rounded-[8px] bg-[var(--ok-soft)] text-[var(--ok)] px-3.5 py-2.5 text-xs leading-relaxed">
                                            { format!("✓ {ok_msg}") }
                                        </div>
                                    }

                                    if let Some(err_msg) = (*ar_submit_error).clone() {
                                        <div
                                            class="rounded-[8px] bg-[var(--err-soft)] text-[var(--err)] px-3.5 py-2.5 text-xs leading-relaxed"
                                            role="alert"
                                        >
                                            { err_msg }
                                        </div>
                                    }

                                    <div class="pt-1 flex flex-wrap items-center justify-between gap-3">
                                        <span class="text-[11px] text-[var(--faint)]">
                                            { ar_t::EMAIL_HELP_TEXT }
                                        </span>
                                        <button
                                            type="submit"
                                            disabled={*ar_submitting}
                                            class="sf-btn sf-btn--primary disabled:cursor-not-allowed disabled:opacity-50"
                                        >
                                            { if *ar_submitting { ar_t::SUBMITTING } else { ar_t::SUBMIT_BTN } }
                                        </button>
                                    </div>
                                </form>

                                <div class="mt-5 pt-4 border-t border-[var(--border)] font-mono text-[0.6875rem] tracking-[0.04em] text-[var(--faint)]">
                                    { "submit → queue → AI research & ingest → LanceDB publish" }
                                </div>
                            </div>

                            // Right: Full Queue + Expandable AI Run Chunks
                            <div class="min-w-0">
                                <div class="flex items-center justify-between gap-3 pb-3 border-b border-[var(--border-strong)]">
                                    <span class="font-mono text-xs uppercase tracking-[0.08em] text-[var(--faint)]">
                                        { format!("入库队列（共 {} 条 · 点击任意条目展开 AI 执行流）", *ar_total) }
                                    </span>
                                </div>

                                if *ar_loading && ar_requests.is_empty() {
                                    <div class="divide-y divide-[var(--border)]" aria-hidden="true">
                                        { for (0..4).map(|i| html! {
                                            <div key={i} class="py-4 space-y-2">
                                                <span class="skeleton block h-4 w-3/4 rounded" />
                                                <span class="skeleton block h-3 w-1/3 rounded" />
                                            </div>
                                        }) }
                                    </div>
                                } else if let Some(err) = (*ar_list_error).clone() {
                                    <div class="py-6 text-sm text-[var(--err)]" role="alert">
                                        { format!("队列加载失败：{err}") }
                                    </div>
                                } else if ar_requests.is_empty() {
                                    <p class="py-8 font-serif italic text-base text-[var(--muted)]">
                                        { ar_t::EMPTY_LIST }
                                    </p>
                                } else {
                                    <>
                                        <ul class="m-0 p-0 list-none border-b border-[var(--border)]">
                                            { for ar_requests.iter().map(|req| {
                                                let key = req.request_id.clone();
                                                let parent_summary = req.parent_request_id.as_ref().and_then(|pid| {
                                                    ar_requests.iter().find(|r| r.request_id == *pid).map(|parent| {
                                                        let label = parent
                                                            .title_hint
                                                            .as_deref()
                                                            .filter(|s| !s.is_empty())
                                                            .unwrap_or(&parent.article_url);
                                                        let display: String = if label.chars().count() > 40 {
                                                            format!("{}…", label.chars().take(39).collect::<String>())
                                                        } else {
                                                            label.to_string()
                                                        };
                                                        AttrValue::from(format!("{} {}", ar_t::FOLLOW_UP_REF_PREFIX, display))
                                                    })
                                                });
                                                html! {
                                                    <RequestRow
                                                        key={key}
                                                        req={req.clone()}
                                                        on_follow_up={on_follow_up.clone()}
                                                        parent_summary={parent_summary}
                                                    />
                                                }
                                            }) }
                                        </ul>

                                        if ar_total_pages > 1 {
                                            <div class="mt-5 flex justify-center">
                                                <Pagination
                                                    current_page={*ar_page}
                                                    total_pages={ar_total_pages}
                                                    on_page_change={on_ar_page_change}
                                                />
                                            </div>
                                        }
                                    </>
                                }
                            </div>
                        </div>
                    }
                </div>
            </section>

            // 3. Main Editorial Row Article Stream
            <section
                id="latest-articles-stream"
                class="max-w-[1200px] mx-auto px-6 md:px-10 pt-12 scroll-mt-20"
            >
                {
                    if *loading {
                        render_archive_skeleton()
                    } else if let Some(err) = (*load_error).clone() {
                        let on_retry = {
                            let grid_refresh_tick = grid_refresh_tick.clone();
                            Callback::from(move |_| grid_refresh_tick.set(*grid_refresh_tick + 1))
                        };
                        html! {
                            <div class="py-16 max-w-[54ch]" role="alert">
                                <p class="m-0 font-serif text-xl text-[var(--err)]">{ t::LOAD_ERROR_TITLE }</p>
                                <p class="mt-2 mb-5 text-sm text-[var(--muted)]">{ err }</p>
                                <button
                                    type="button"
                                    class="sf-btn sf-btn--secondary"
                                    onclick={on_retry}
                                >
                                    { fill_one("{}", t::RETRY) }
                                </button>
                            </div>
                        }
                    } else if articles.is_empty() {
                        html! {
                            <div class="py-20">
                                <p class="m-0 font-serif italic text-lg text-[var(--muted)]">{ t::EMPTY }</p>
                            </div>
                        }
                    } else {
                        html! {
                            <>
                                <div>
                                    { for articles.iter().map(|article| {
                                        html! {
                                            <ArticleCard
                                                key={article.id.clone()}
                                                article={article.clone()}
                                                variant={ArticleCardVariant::Row}
                                                on_before_navigate={Some(save_scroll_position.clone())}
                                            />
                                        }
                                    }) }
                                </div>
                                { pagination_controls }
                            </>
                        }
                    }
                }
            </section>

            <ScrollToTopButton />
        </main>
    }
}
