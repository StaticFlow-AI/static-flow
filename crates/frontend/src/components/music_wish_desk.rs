//! Self-service music wish submission form, live queue, AI execution log, and
//! inline playback desk.

use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    api,
    components::{
        icons::{Icon, IconName},
        pagination::Pagination,
        toast::use_toast,
    },
    i18n::current::{music_wish as wish_t, studio_page as st_t},
    music_context::{MusicAction, MusicPlayerContext},
    router::Route,
};

const WISH_PAGE_SIZE: usize = 10;

fn smooth_scroll_to_id(target_id: &str) {
    let Some(win) = web_sys::window() else {
        return;
    };
    let Some(doc) = win.document() else {
        return;
    };
    let Some(el) = doc.get_element_by_id(target_id) else {
        return;
    };
    let rect = el.get_bounding_client_rect();
    let scroll_y = win.scroll_y().unwrap_or(0.0);
    let top = (scroll_y + rect.top() - 76.0).max(0.0);
    let opts = web_sys::ScrollToOptions::new();
    opts.set_top(top);
    opts.set_behavior(web_sys::ScrollBehavior::Smooth);
    win.scroll_to_with_scroll_to_options(&opts);
}

fn format_ts_ms(ms: i64) -> String {
    if ms <= 0 {
        return String::new();
    }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64));
    let y = date.get_full_year();
    let m = date.get_month() + 1;
    let d = date.get_date();
    let h = date.get_hours();
    let min = date.get_minutes();
    format!("{y:04}-{m:02}-{d:02} {h:02}:{min:02}")
}

fn wish_status_class_and_text(status: &str) -> (&'static str, &str) {
    match status {
        "done" => ("sf-status--done", wish_t::STATUS_DONE),
        "running" => ("sf-status--running", wish_t::STATUS_RUNNING),
        "approved" => ("sf-status--approved", wish_t::STATUS_APPROVED),
        "failed" => ("sf-status--failed", wish_t::STATUS_FAILED),
        "pending" => ("sf-status--pending", wish_t::STATUS_PENDING),
        other => ("sf-status--pending", other),
    }
}

#[derive(Properties, PartialEq)]
struct WishQueueRowProps {
    pub wish: api::MusicWishItem,
    pub on_fill_form: Callback<api::MusicWishItem>,
    pub on_play_song: Callback<String>,
    pub is_playing: bool,
    pub is_loading_play: bool,
}

#[function_component(WishQueueRow)]
fn wish_queue_row(props: &WishQueueRowProps) -> Html {
    let w = &props.wish;
    let log_open = use_state(|| w.status == "running");
    let ai_log_loading = use_state(|| false);
    let ai_log_fetched = use_state(|| false);
    let ai_log_text = use_state(|| None::<String>);

    let (status_modifier, status_text) = wish_status_class_and_text(&w.status);
    let ts = format_ts_ms(w.created_at);

    // Fetch AI execution log when expanded
    {
        let should_fetch = *log_open && !*ai_log_fetched && !*ai_log_loading;
        let ai_log_loading = ai_log_loading.clone();
        let ai_log_fetched = ai_log_fetched.clone();
        let ai_log_text = ai_log_text.clone();
        let wish_id = w.wish_id.clone();
        use_effect_with((wish_id, should_fetch), move |(wid, do_fetch)| {
            if *do_fetch {
                let wid = wid.clone();
                ai_log_loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(output) = api::fetch_admin_music_wish_ai_output(&wid).await {
                        let mut buf = String::new();
                        for run in &output.runs {
                            buf.push_str(&format!(
                                "[run {}] status={} runner={} exit={:?}\n",
                                run.run_id, run.status, run.runner_program, run.exit_code
                            ));
                            if let Some(ref reason) = run.failure_reason {
                                if !reason.trim().is_empty() {
                                    buf.push_str(&format!("failure: {reason}\n"));
                                }
                            }
                        }
                        for chunk in &output.chunks {
                            buf.push_str(&chunk.content);
                            if !chunk.content.ends_with('\n') {
                                buf.push('\n');
                            }
                        }
                        if !buf.trim().is_empty() {
                            ai_log_text.set(Some(buf));
                        }
                    }
                    ai_log_fetched.set(true);
                    ai_log_loading.set(false);
                });
            }
            || ()
        });
    }

    let on_fill = {
        let cb = props.on_fill_form.clone();
        let wish = w.clone();
        Callback::from(move |_: MouseEvent| cb.emit(wish.clone()))
    };

    let on_toggle_log = {
        let log_open = log_open.clone();
        Callback::from(move |_: MouseEvent| {
            log_open.set(!*log_open);
        })
    };

    let fallback_log = {
        let mut lines = Vec::new();
        lines.push(format!(
            "wish_id={} · status={} · attempts={}",
            w.wish_id, w.status, w.attempt_count
        ));
        lines.push(format!(
            "created={} · updated={}",
            format_ts_ms(w.created_at),
            format_ts_ms(w.updated_at)
        ));
        if let Some(ref sid) = w.ingested_song_id {
            if !sid.trim().is_empty() {
                lines.push(format!("ingested_song_id={sid}"));
            }
        }
        if let Some(ref err) = w.failure_reason {
            if !err.trim().is_empty() {
                lines.push(format!("failure_reason: {err}"));
            }
        }
        if let Some(ref reply) = w.ai_reply {
            if !reply.trim().is_empty() {
                lines.push(String::new());
                lines.push(reply.clone());
            }
        }
        lines.join("\n")
    };

    let ingested_sid = w
        .ingested_song_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let done_song_actions = if w.status == "done" {
        if let Some(ref song_id) = ingested_sid {
            let play_cb = {
                let cb = props.on_play_song.clone();
                let sid = song_id.clone();
                Callback::from(move |_: MouseEvent| cb.emit(sid.clone()))
            };
            html! {
                <>
                    <button
                        type="button"
                        onclick={play_cb}
                        aria-label={if props.is_playing { "暂停" } else { "播放" }}
                        title={if props.is_playing { "暂停" } else { "播放" }}
                        class={classes!(
                            "inline-flex",
                            "h-10",
                            "w-10",
                            "shrink-0",
                            "items-center",
                            "justify-center",
                            "rounded-full",
                            "border",
                            "transition-colors",
                            "cursor-pointer",
                            if props.is_playing {
                                "border-[var(--primary)] bg-[var(--primary)] text-[var(--on-primary)]"
                            } else {
                                "border-[var(--border-strong)] bg-transparent text-[var(--text-2)] hover:border-[var(--primary)] hover:bg-[var(--primary)] hover:text-[var(--on-primary)]"
                            }
                        )}
                    >
                        if props.is_loading_play {
                            <span class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent motion-reduce:animate-none" />
                        } else {
                            <Icon name={if props.is_playing { IconName::Pause } else { IconName::Play }} size={13} />
                        }
                    </button>
                    <Link<Route>
                        to={Route::MusicPlayer { id: song_id.clone() }}
                        classes="sf-btn sf-btn--ghost !h-10 !px-2.5 font-mono !text-xs !text-[var(--primary)]"
                    >
                        { "详情 →" }
                    </Link<Route>>
                </>
            }
        } else {
            Html::default()
        }
    } else {
        Html::default()
    };

    let artist_display = w
        .artist_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    html! {
        <li class="py-4 border-b border-[var(--border)] last:border-b-0 min-w-0">
            // Row 1: Status badge + wrappable song title & artist (never squeezed by action buttons)
            <div class="flex flex-wrap items-start gap-2 min-w-0">
                <span class={classes!("sf-status", "shrink-0", "mt-1", status_modifier)}>
                    { status_text }
                </span>

                <div class="min-w-0 flex-1">
                    if let (true, Some(ref song_id)) = (w.status == "done", &ingested_sid) {
                        <Link<Route>
                            to={Route::MusicPlayer { id: song_id.clone() }}
                            classes="min-h-[40px] inline-flex flex-wrap items-center gap-x-2 font-serif font-normal text-[1rem] leading-[1.45] text-[var(--text)] hover:text-[var(--primary)] no-underline [overflow-wrap:anywhere] transition-colors"
                        >
                            <span>{ &w.song_name }</span>
                            if let Some(artist) = artist_display {
                                <span class="font-sans text-xs text-[var(--muted)]">
                                    { format!("— {artist}") }
                                </span>
                            }
                        </Link<Route>>
                    } else {
                        <button
                            type="button"
                            onclick={on_toggle_log.clone()}
                            class="min-h-[40px] inline-flex flex-wrap items-center gap-x-2 font-serif font-normal text-[1rem] leading-[1.45] text-left text-[var(--text)] hover:text-[var(--primary)] bg-transparent border-0 p-0 cursor-pointer whitespace-normal [overflow-wrap:anywhere] transition-colors"
                        >
                            <span>{ &w.song_name }</span>
                            if let Some(artist) = artist_display {
                                <span class="font-sans text-xs text-[var(--muted)]">
                                    { format!("— {artist}") }
                                </span>
                            }
                        </button>
                    }
                </div>
            </div>

            if !w.wish_message.trim().is_empty() {
                <p class="mt-1 mb-0 line-clamp-2 text-xs leading-relaxed text-[var(--text-2)] [overflow-wrap:anywhere]">
                    { &w.wish_message }
                </p>
            }

            // Row 2: Metadata + Actions (>= 40px tap targets)
            <div class="mt-1.5 flex flex-wrap items-center justify-between gap-x-3 gap-y-1.5 min-w-0">
                <div class="flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-xs tabular-nums text-[var(--faint)] min-w-0">
                    <span class="truncate max-w-[16ch] sm:max-w-none">{ &w.nickname }</span>
                    if !ts.is_empty() {
                        <span aria-hidden="true">{ "·" }</span>
                        <span>{ &ts }</span>
                    }
                    if !w.ip_region.trim().is_empty() {
                        <span aria-hidden="true">{ "·" }</span>
                        <span>{ &w.ip_region }</span>
                    }
                </div>

                <div class="flex flex-wrap items-center gap-1 shrink-0">
                    { done_song_actions }
                    <button
                        type="button"
                        onclick={on_fill}
                        title={wish_t::FILL_FORM}
                        class="sf-btn sf-btn--ghost !h-10 !px-2.5 font-mono !text-xs"
                    >
                        { wish_t::FILL_FORM }
                    </button>
                    <button
                        type="button"
                        onclick={on_toggle_log}
                        aria-expanded={(*log_open).to_string()}
                        class="sf-btn sf-btn--ghost !h-10 !px-2.5 font-mono !text-xs"
                    >
                        { if *log_open { "收起 ↑" } else { "AI 日志 ↓" } }
                    </button>
                </div>
            </div>

            if *log_open {
                <div class="mt-3 space-y-2 min-w-0">
                    <div class="flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] text-[var(--faint)]">
                        <span>{ format!("AI EXECUTION LOG · status={}", w.status) }</span>
                        if *ai_log_loading {
                            <span>{ "loading…" }</span>
                        }
                    </div>
                    <div class="max-h-72 overflow-y-auto whitespace-pre-wrap rounded-[10px] border border-[var(--border)] bg-[var(--surface-sunk)] p-4 font-mono text-[0.8125rem] leading-[1.6] text-[var(--text-2)] [overflow-wrap:anywhere]">
                        {
                            if let Some(ref full_log) = *ai_log_text {
                                full_log.as_str()
                            } else {
                                fallback_log.as_str()
                            }
                        }
                    </div>
                </div>
            }
        </li>
    }
}

/// Props for [`MusicWishDesk`].
#[derive(Properties, PartialEq)]
pub struct MusicWishDeskProps {
    /// Optional callback notified whenever total wish count is fetched.
    #[prop_or_default]
    pub on_total_loaded: Option<Callback<usize>>,
    /// Highlights the form card border when focused via URL hash.
    #[prop_or(false)]
    pub highlighted: bool,
}

/// Self-service music wish submission form and live queue component.
#[function_component(MusicWishDesk)]
pub fn music_wish_desk(props: &MusicWishDeskProps) -> Html {
    let wishes = use_state(Vec::<api::MusicWishItem>::new);
    let wish_loading = use_state(|| false);
    let wish_list_error = use_state(|| None::<String>);
    let wish_page = use_state(|| 1_usize);
    let wish_total = use_state(|| 0_usize);
    let wish_refresh_tick = use_state(|| 0_u64);
    let wish_request_seq = use_mut_ref(|| 0_u64);

    let wish_form_song = use_state(String::new);
    let wish_form_artist = use_state(String::new);
    let wish_form_message = use_state(String::new);
    let wish_form_nickname = use_state(String::new);
    let wish_form_email = use_state(String::new);
    let wish_submitting = use_state(|| false);
    let wish_song_error = use_state(|| None::<String>);
    let wish_message_error = use_state(|| None::<String>);
    let wish_submit_banner = use_state(|| None::<(bool, String)>);

    let inline_play_loading_id = use_state(|| None::<String>);
    let player_ctx = use_context::<MusicPlayerContext>();
    let toast = use_toast();

    let wish_total_val = *wish_total;
    let wish_total_pages =
        if wish_total_val == 0 { 1 } else { wish_total_val.div_ceil(WISH_PAGE_SIZE) };

    // Fetch wishes when page changes or manual refresh is triggered
    {
        let wishes = wishes.clone();
        let wish_loading = wish_loading.clone();
        let wish_list_error = wish_list_error.clone();
        let wish_total = wish_total.clone();
        let wish_request_seq = wish_request_seq.clone();
        let on_total_loaded = props.on_total_loaded.clone();
        let deps = (*wish_page, *wish_refresh_tick);
        use_effect_with(deps, move |(page, _)| {
            let request_id = {
                let mut seq = wish_request_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            let offset = ((*page).saturating_sub(1)) * WISH_PAGE_SIZE;
            wish_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_music_wishes(Some(WISH_PAGE_SIZE), Some(offset)).await {
                    Ok(resp) => {
                        if *wish_request_seq.borrow() != request_id {
                            return;
                        }
                        if let Some(cb) = on_total_loaded {
                            cb.emit(resp.total);
                        }
                        wishes.set(resp.wishes);
                        wish_total.set(resp.total);
                        wish_list_error.set(None);
                    },
                    Err(err) => {
                        if *wish_request_seq.borrow() != request_id {
                            return;
                        }
                        wish_list_error.set(Some(err));
                    },
                }
                if *wish_request_seq.borrow() != request_id {
                    return;
                }
                wish_loading.set(false);
            });
            || ()
        });
    }

    let on_wish_page_change = {
        let wish_page = wish_page.clone();
        Callback::from(move |page: usize| {
            wish_page.set(page);
        })
    };

    let on_refresh_wishes = {
        let wish_refresh_tick = wish_refresh_tick.clone();
        Callback::from(move |_: MouseEvent| {
            wish_refresh_tick.set(*wish_refresh_tick + 1);
        })
    };

    let on_wish_fill_form = {
        let wish_form_song = wish_form_song.clone();
        let wish_form_artist = wish_form_artist.clone();
        let wish_form_message = wish_form_message.clone();
        let wish_song_error = wish_song_error.clone();
        let wish_message_error = wish_message_error.clone();
        let wish_submit_banner = wish_submit_banner.clone();
        Callback::from(move |w: api::MusicWishItem| {
            wish_form_song.set(w.song_name.clone());
            wish_form_artist.set(w.artist_hint.unwrap_or_default());
            wish_form_message.set(w.wish_message.clone());
            wish_song_error.set(None);
            wish_message_error.set(None);
            wish_submit_banner.set(None);
            smooth_scroll_to_id("music-wish-form");
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("wish-song") {
                    if let Ok(html_el) = el.dyn_into::<HtmlElement>() {
                        let _ = html_el.focus();
                    }
                }
            }
        })
    };

    // Inline playback handler for ingested wish tracks
    let on_inline_play = {
        let player_ctx = player_ctx.clone();
        let inline_play_loading_id = inline_play_loading_id.clone();
        let toast = toast.clone();
        Callback::from(move |song_id: String| {
            let Some(ctx) = player_ctx.clone() else {
                return;
            };
            if ctx.song_id.as_deref() == Some(song_id.as_str()) && ctx.current_song.is_some() {
                ctx.dispatch(MusicAction::TogglePlay);
                ctx.dispatch(MusicAction::Minimize);
                return;
            }
            if inline_play_loading_id.as_deref() == Some(song_id.as_str()) {
                return;
            }
            inline_play_loading_id.set(Some(song_id.clone()));
            let inline_play_loading_id = inline_play_loading_id.clone();
            let toast = toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_song_detail(&song_id).await {
                    Ok(Some(detail)) => {
                        ctx.dispatch(MusicAction::PlaySong {
                            song: detail,
                            id: song_id.clone(),
                        });
                        ctx.dispatch(MusicAction::Minimize);
                        let id2 = song_id.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let _ = api::track_song_play(&id2).await;
                        });
                    },
                    Ok(None) => {
                        toast.error("未找到该曲目音频数据");
                    },
                    Err(err) => {
                        toast.error(format!("加载曲目失败：{err}"));
                    },
                }
                inline_play_loading_id.set(None);
            });
        })
    };

    let on_wish_submit = {
        let wish_form_song = wish_form_song.clone();
        let wish_form_artist = wish_form_artist.clone();
        let wish_form_message = wish_form_message.clone();
        let wish_form_nickname = wish_form_nickname.clone();
        let wish_form_email = wish_form_email.clone();
        let wish_submitting = wish_submitting.clone();
        let wish_song_error = wish_song_error.clone();
        let wish_message_error = wish_message_error.clone();
        let wish_submit_banner = wish_submit_banner.clone();
        let toast = toast.clone();
        let wish_page = wish_page.clone();
        let wish_refresh_tick = wish_refresh_tick.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if *wish_submitting {
                return;
            }
            let song = (*wish_form_song).trim().to_string();
            let artist = (*wish_form_artist).trim().to_string();
            let message = (*wish_form_message).trim().to_string();
            let nickname = (*wish_form_nickname).trim().to_string();
            let email = (*wish_form_email).trim().to_string();
            let song_missing = song.is_empty();
            let message_missing = message.is_empty();
            wish_song_error.set(song_missing.then(|| wish_t::SONG_NAME_REQUIRED.to_string()));
            wish_message_error.set(message_missing.then(|| wish_t::MESSAGE_REQUIRED.to_string()));
            if song_missing || message_missing {
                return;
            }
            let artist_opt = if artist.is_empty() { None } else { Some(artist.clone()) };
            let nickname_opt = if nickname.is_empty() { None } else { Some(nickname.clone()) };
            let email_opt = if email.is_empty() { None } else { Some(email.clone()) };
            let frontend_page_url = web_sys::window().and_then(|w| w.location().href().ok());
            let wish_submitting = wish_submitting.clone();
            let wish_submit_banner = wish_submit_banner.clone();
            let toast = toast.clone();
            let wish_page = wish_page.clone();
            let wish_refresh_tick = wish_refresh_tick.clone();
            let wish_form_song = wish_form_song.clone();
            let wish_form_artist = wish_form_artist.clone();
            let wish_form_message = wish_form_message.clone();
            let wish_form_email = wish_form_email.clone();
            let next_tick = wish_refresh_tick.wrapping_add(1);
            wish_submitting.set(true);
            wish_submit_banner.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match api::submit_music_wish(
                    &song,
                    artist_opt.as_deref(),
                    &message,
                    nickname_opt.as_deref(),
                    email_opt.as_deref(),
                    frontend_page_url.as_deref(),
                )
                .await
                {
                    Ok(resp) => {
                        toast.success(wish_t::SUBMIT_SUCCESS);
                        wish_submit_banner.set(Some((
                            true,
                            format!(
                                "{}（#{}）",
                                wish_t::SUBMIT_SUCCESS,
                                resp.wish_id.chars().take(8).collect::<String>()
                            ),
                        )));
                        wish_form_song.set(String::new());
                        wish_form_artist.set(String::new());
                        wish_form_message.set(String::new());
                        wish_form_email.set(String::new());
                        wish_page.set(1);
                        wish_refresh_tick.set(next_tick);
                    },
                    Err(e) => {
                        toast.error(e.clone());
                        wish_submit_banner.set(Some((false, e)));
                    },
                }
                wish_submitting.set(false);
            });
        })
    };

    let has_active_wish_runner = wishes
        .iter()
        .any(|w| matches!(w.status.as_str(), "running" | "approved"));
    let current_playing_id = player_ctx.as_ref().and_then(|c| c.song_id.clone());
    let is_global_playing = player_ctx.as_ref().map(|c| c.playing).unwrap_or(false);

    html! {
        <div class="min-w-0 space-y-7">
            // Desk Header
            <div class="flex items-start justify-between gap-4">
                <div class="flex-1 min-w-0">
                    <div class="flex flex-wrap items-center gap-2.5">
                        <span class="sf-eyebrow !mb-0">
                            { st_t::MUSIC_DESK_EYEBROW }
                        </span>
                        if has_active_wish_runner {
                            <span class="inline-flex items-center gap-1.5 font-mono text-[11px] font-semibold uppercase tracking-[0.08em] text-[var(--primary)]">
                                <span class="w-1.5 h-1.5 rounded-full bg-[var(--primary)] animate-pulse motion-reduce:animate-none" aria-hidden="true" />
                                { "LIVE" }
                            </span>
                        }
                    </div>
                    <h2
                        id="mw-desk-heading"
                        class="mt-2 mb-0 font-serif font-normal text-[clamp(1.375rem,1.2rem+0.7vw,1.75rem)] leading-[1.2] tracking-[-0.015em] text-[var(--text)]"
                    >
                        { st_t::TAB_MUSIC }
                    </h2>
                    <p class="mt-1.5 mb-0 text-sm leading-relaxed text-[var(--muted)] max-w-[54ch]">
                        { st_t::MUSIC_DESK_DESC }
                    </p>
                </div>

                <button
                    type="button"
                    onclick={on_refresh_wishes}
                    disabled={*wish_loading}
                    class="sf-btn sf-btn--secondary min-h-[40px] !px-3.5 !text-xs font-mono shrink-0"
                >
                    { if *wish_loading { wish_t::REFRESHING } else { wish_t::REFRESH_BTN } }
                </button>
            </div>

            // Top: Editorial Wish Form Card (.sf-card)
            <div
                id="music-wish-form"
                class={classes!(
                    "sf-card",
                    "p-5",
                    "sm:p-6",
                    "scroll-mt-24",
                    props.highlighted.then_some("!border-[var(--border-strong)]")
                )}
            >
                <form onsubmit={on_wish_submit} class="space-y-4">
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
                        <div class="flex flex-col gap-1.5">
                            <label for="wish-song" class="text-xs font-medium text-[var(--text-2)]">
                                { format!("{} *", wish_t::SONG_NAME_LABEL) }
                            </label>
                            <input
                                id="wish-song"
                                type="text"
                                placeholder={wish_t::SONG_NAME_PLACEHOLDER}
                                value={(*wish_form_song).clone()}
                                aria-invalid={wish_song_error.is_some().to_string()}
                                oninput={
                                    let s = wish_form_song.clone();
                                    let err = wish_song_error.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        s.set(input.value());
                                        err.set(None);
                                    })
                                }
                                class={classes!(
                                    "sf-input",
                                    "w-full",
                                    wish_song_error.is_some().then_some("!border-[var(--err)]")
                                )}
                                required=true
                            />
                            if let Some(ref err_msg) = *wish_song_error {
                                <p class="m-0 text-xs text-[var(--err)]" role="alert">{ err_msg }</p>
                            }
                        </div>

                        <div class="flex flex-col gap-1.5">
                            <label for="wish-artist" class="text-xs font-medium text-[var(--text-2)]">
                                { wish_t::ARTIST_LABEL }
                            </label>
                            <input
                                id="wish-artist"
                                type="text"
                                placeholder={wish_t::ARTIST_PLACEHOLDER}
                                value={(*wish_form_artist).clone()}
                                oninput={
                                    let s = wish_form_artist.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        s.set(input.value());
                                    })
                                }
                                class="sf-input w-full"
                            />
                        </div>
                    </div>

                    <div class="flex flex-col gap-1.5">
                        <label for="wish-message" class="text-xs font-medium text-[var(--text-2)]">
                            { format!("{} *", wish_t::MESSAGE_LABEL) }
                        </label>
                        <textarea
                            id="wish-message"
                            placeholder={wish_t::MESSAGE_PLACEHOLDER}
                            value={(*wish_form_message).clone()}
                            aria-invalid={wish_message_error.is_some().to_string()}
                            oninput={
                                let s = wish_form_message.clone();
                                let err = wish_message_error.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: HtmlTextAreaElement = e.target_unchecked_into();
                                    s.set(input.value());
                                    err.set(None);
                                })
                            }
                            rows="3"
                            class={classes!(
                                "sf-textarea",
                                "w-full",
                                "min-h-[92px]",
                                "py-2.5",
                                "resize-y",
                                wish_message_error.is_some().then_some("!border-[var(--err)]")
                            )}
                            required=true
                        />
                        if let Some(ref err_msg) = *wish_message_error {
                            <p class="m-0 text-xs text-[var(--err)]" role="alert">{ err_msg }</p>
                        }
                    </div>

                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
                        <div class="flex flex-col gap-1.5">
                            <label for="wish-nickname" class="text-xs text-[var(--muted)]">
                                { wish_t::NICKNAME_LABEL }
                            </label>
                            <input
                                id="wish-nickname"
                                type="text"
                                placeholder={wish_t::NICKNAME_PLACEHOLDER}
                                value={(*wish_form_nickname).clone()}
                                oninput={
                                    let s = wish_form_nickname.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        s.set(input.value());
                                    })
                                }
                                class="sf-input w-full"
                            />
                        </div>
                        <div class="flex flex-col gap-1.5">
                            <label for="wish-email" class="text-xs text-[var(--muted)]">
                                { wish_t::EMAIL_LABEL }
                            </label>
                            <input
                                id="wish-email"
                                type="email"
                                placeholder={wish_t::EMAIL_PLACEHOLDER}
                                value={(*wish_form_email).clone()}
                                oninput={
                                    let s = wish_form_email.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        s.set(input.value());
                                    })
                                }
                                class="sf-input w-full"
                            />
                        </div>
                    </div>

                    if let Some((ok, ref msg)) = *wish_submit_banner {
                        <div
                            class={classes!(
                                "rounded-[8px]",
                                "px-3.5",
                                "py-2.5",
                                "text-xs",
                                "leading-relaxed",
                                "[overflow-wrap:anywhere]",
                                if ok {
                                    "bg-[var(--ok-soft)] text-[var(--ok)]"
                                } else {
                                    "bg-[var(--err-soft)] text-[var(--err)]"
                                }
                            )}
                            role={(!ok).then_some("alert")}
                        >
                            { if ok { format!("✓ {msg}") } else { msg.clone() } }
                        </div>
                    }

                    <div class="pt-3 border-t border-[var(--border)] flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                        <span class="text-xs text-[var(--faint)]">
                            { wish_t::EMAIL_HELP_TEXT }
                        </span>
                        <button
                            type="submit"
                            disabled={*wish_submitting}
                            class="sf-btn sf-btn--primary min-h-[40px] w-full sm:w-auto"
                        >
                            { if *wish_submitting { wish_t::SUBMITTING } else { wish_t::SUBMIT_BTN } }
                        </button>
                    </div>
                </form>
            </div>

            // Bottom: Hairline-divided Live Wish Queue
            <div class="min-w-0">
                <div class="flex flex-wrap items-center justify-between gap-2 pb-3 border-b border-[var(--border-strong)]">
                    <span class="font-mono text-xs uppercase tracking-[0.08em] text-[var(--faint)]">
                        { "WISH QUEUE · 实时心愿队列" }
                    </span>
                    <span class="font-mono text-xs tabular-nums text-[var(--muted)]">
                        { format!("共 {} 条", wish_total_val) }
                    </span>
                </div>

                if *wish_loading && wishes.is_empty() {
                    <div class="divide-y divide-[var(--border)]" aria-hidden="true">
                        { for (0..4).map(|i| html! {
                            <div key={i} class="py-4 space-y-2">
                                <span class="skeleton block h-4 w-3/4 rounded" />
                                <span class="skeleton block h-3 w-1/3 rounded" />
                            </div>
                        }) }
                    </div>
                } else if let Some(err) = (*wish_list_error).clone() {
                    <div class="py-6 text-sm text-[var(--err)] [overflow-wrap:anywhere]" role="alert">
                        { format!("加载心愿队列失败：{err}") }
                    </div>
                } else if wishes.is_empty() {
                    <p class="py-8 font-serif italic text-base text-[var(--muted)]">
                        { wish_t::EMPTY_LIST }
                    </p>
                } else {
                    <>
                        <ul class="m-0 p-0 list-none border-b border-[var(--border)]">
                            { for wishes.iter().map(|w| {
                                let is_wish_song_playing = w.ingested_song_id.as_ref().is_some_and(|sid| {
                                    current_playing_id.as_deref() == Some(sid.as_str()) && is_global_playing
                                });
                                let is_wish_song_loading = w.ingested_song_id.as_ref().is_some_and(|sid| {
                                    inline_play_loading_id.as_deref() == Some(sid.as_str())
                                });
                                html! {
                                    <WishQueueRow
                                        key={w.wish_id.clone()}
                                        wish={w.clone()}
                                        on_fill_form={on_wish_fill_form.clone()}
                                        on_play_song={on_inline_play.clone()}
                                        is_playing={is_wish_song_playing}
                                        is_loading_play={is_wish_song_loading}
                                    />
                                }
                            }) }
                        </ul>

                        if wish_total_pages > 1 {
                            <div class="mt-5 flex justify-center">
                                <Pagination
                                    current_page={*wish_page}
                                    total_pages={wish_total_pages}
                                    on_page_change={on_wish_page_change}
                                />
                            </div>
                        }
                    </>
                }
            </div>
        </div>
    }
}
