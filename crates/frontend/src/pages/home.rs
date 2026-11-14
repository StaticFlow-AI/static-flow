use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use web_sys::{console, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_router::prelude::{use_location, Link};

use crate::{
    api::{self, ArticleRequestItem, MusicWishItem, SongListItem},
    components::{article_card::ArticleCard, image_with_loading::ImageWithLoading},
    i18n::current::{article_request as ar_t, home as t, music_wish as mw_t},
    models::{ArticleKind, ArticleListItem},
    music_context::{MusicAction, MusicPlayerContext},
    router::Route,
    utils::image_url,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkshopTab {
    Article,
    Music,
}

fn format_duration_short(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

fn format_relative_time_zh(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return String::new();
    }
    let now_ms = js_sys::Date::now() as i64;
    let diff_secs = ((now_ms - timestamp_ms) / 1000).max(0);
    if diff_secs < 60 {
        "刚刚".to_string()
    } else if diff_secs < 3600 {
        format!("{} 分钟前", diff_secs / 60)
    } else if diff_secs < 86_400 {
        format!("{} 小时前", diff_secs / 3600)
    } else if diff_secs < 86_400 * 30 {
        format!("{} 天前", diff_secs / 86_400)
    } else {
        let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(timestamp_ms as f64));
        format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
    }
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

fn smooth_scroll_to_id(id: &str) {
    let Some(win) = web_sys::window() else {
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

fn status_badge_view(status: &str, label: &str) -> Html {
    let modifier = match status {
        "running" => "sf-status--running",
        "done" => "sf-status--done",
        "failed" => "sf-status--failed",
        _ => "sf-status--pending",
    };
    html! {
        <span class={classes!("sf-status", modifier, "shrink-0")}>
            { label }
        </span>
    }
}

fn article_request_status_label(req: &ArticleRequestItem) -> (&'static str, &'static str) {
    let has_article = req
        .ingested_article_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|id| !id.is_empty());
    if req.status == "done" && !has_article {
        ("pending", "已回复")
    } else {
        match req.status.as_str() {
            "running" => ("running", "AI 入库中"),
            "done" => ("done", "已入库"),
            "failed" => ("failed", "失败"),
            "approved" => ("pending", "排队中"),
            _ => ("pending", "待处理"),
        }
    }
}

fn music_wish_status_label(wish: &MusicWishItem) -> (&'static str, &'static str) {
    match wish.status.as_str() {
        "running" => ("running", "抓取中"),
        "done" => ("done", "已入库"),
        "failed" => ("failed", "未命中"),
        "approved" => ("pending", "排队中"),
        _ => ("pending", "待处理"),
    }
}

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let location = use_location();
    let player_ctx = use_context::<MusicPlayerContext>();

    // Real metrics (None until fetched — never display fake numbers)
    let total_articles = use_state(|| None::<usize>);
    let total_music = use_state(|| None::<usize>);
    let ar_total = use_state(|| None::<usize>);
    let mw_total = use_state(|| None::<usize>);

    // Recent content & queues
    let recent_articles = use_state(Vec::<ArticleListItem>::new);
    let articles_loaded = use_state(|| false);
    let recent_songs = use_state(Vec::<SongListItem>::new);
    let songs_loaded = use_state(|| false);
    let loading_play_id = use_state(|| None::<String>);

    let ar_items = use_state(Vec::<ArticleRequestItem>::new);
    let ar_loaded = use_state(|| false);
    let ar_refresh_seq = use_state(|| 0_u32);

    let mw_items = use_state(Vec::<MusicWishItem>::new);
    let mw_loaded = use_state(|| false);
    let mw_refresh_seq = use_state(|| 0_u32);

    // Workshop tabs & forms
    let active_tab = use_state(|| WorkshopTab::Article);

    // Article request form state
    let ar_url = use_state(String::new);
    let ar_title = use_state(String::new);
    let ar_message = use_state(String::new);
    let ar_nickname = use_state(String::new);
    let ar_email = use_state(String::new);
    let ar_submitting = use_state(|| false);
    let ar_error = use_state(|| None::<String>);
    let ar_submitted_id = use_state(|| None::<String>);

    // Music wish form state
    let mw_song = use_state(String::new);
    let mw_artist = use_state(String::new);
    let mw_message = use_state(String::new);
    let mw_nickname = use_state(String::new);
    let mw_email = use_state(String::new);
    let mw_submitting = use_state(|| false);
    let mw_error = use_state(|| None::<String>);
    let mw_submitted_id = use_state(|| None::<String>);

    // Handle URL hash (#workshop & #workshop-music) on mount, route change, and
    // hashchange
    {
        let active_tab = active_tab.clone();
        let loc_dep = location.clone();
        use_effect_with(loc_dep, move |_| {
            let apply_hash = {
                let active_tab = active_tab.clone();
                move || {
                    let hash = web_sys::window()
                        .and_then(|w| w.location().hash().ok())
                        .unwrap_or_default();
                    if hash == "#workshop-music" {
                        active_tab.set(WorkshopTab::Music);
                        smooth_scroll_to_id("workshop");
                        Timeout::new(90, || smooth_scroll_to_id("workshop")).forget();
                    } else if hash == "#workshop" {
                        active_tab.set(WorkshopTab::Article);
                        smooth_scroll_to_id("workshop");
                        Timeout::new(90, || smooth_scroll_to_id("workshop")).forget();
                    }
                }
            };

            apply_hash();

            let apply_on_hash = apply_hash.clone();
            let on_hashchange =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                    apply_on_hash();
                });
            let apply_on_pop = apply_hash;
            let on_popstate =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                    apply_on_pop();
                });

            if let Some(win) = web_sys::window() {
                let _ = win.add_event_listener_with_callback(
                    "hashchange",
                    on_hashchange.as_ref().unchecked_ref(),
                );
                let _ = win.add_event_listener_with_callback(
                    "popstate",
                    on_popstate.as_ref().unchecked_ref(),
                );
            }

            move || {
                if let Some(win) = web_sys::window() {
                    let _ = win.remove_event_listener_with_callback(
                        "hashchange",
                        on_hashchange.as_ref().unchecked_ref(),
                    );
                    let _ = win.remove_event_listener_with_callback(
                        "popstate",
                        on_popstate.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    // Initial site stats, articles (7), and songs (5)
    {
        let total_articles = total_articles.clone();
        let total_music = total_music.clone();
        let recent_articles = recent_articles.clone();
        let articles_loaded = articles_loaded.clone();
        let recent_songs = recent_songs.clone();
        let songs_loaded = songs_loaded.clone();
        use_effect_with((), move |_| {
            {
                let total_articles = total_articles.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_site_stats().await {
                        Ok(stats) => total_articles.set(Some(stats.total_articles)),
                        Err(e) => {
                            console::error_1(&format!("Failed to fetch home stats: {e}").into());
                        },
                    }
                });
            }
            {
                let total_music = total_music.clone();
                let recent_songs = recent_songs.clone();
                let songs_loaded = songs_loaded.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut fallback_songs = Vec::new();
                    match api::fetch_songs(Some(5), Some(0), None, None, None).await {
                        Ok(resp) => {
                            total_music.set(Some(resp.total));
                            fallback_songs = resp.songs;
                        },
                        Err(e) => {
                            console::error_1(&format!("Failed to fetch songs total: {e}").into());
                        },
                    }
                    match api::fetch_random_recommended_songs(Some(5), &[]).await {
                        Ok(songs) if !songs.is_empty() => recent_songs.set(songs),
                        Ok(_) => recent_songs.set(fallback_songs),
                        Err(e) => {
                            console::error_1(&format!("Failed to fetch random songs: {e}").into());
                            recent_songs.set(fallback_songs);
                        },
                    }
                    songs_loaded.set(true);
                });
            }
            {
                let recent_articles = recent_articles.clone();
                let articles_loaded = articles_loaded.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_articles(None, None, Some(7), Some(0)).await {
                        Ok(page) => recent_articles.set(page.articles),
                        Err(e) => {
                            console::error_1(
                                &format!("Failed to fetch recent articles: {e}").into(),
                            );
                        },
                    }
                    articles_loaded.set(true);
                });
            }
            || ()
        });
    }

    // Fetch latest 4 article requests (refreshed after submit)
    {
        let ar_items = ar_items.clone();
        let ar_total = ar_total.clone();
        let ar_loaded = ar_loaded.clone();
        let seq = *ar_refresh_seq;
        use_effect_with(seq, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_article_requests(Some(4), Some(0)).await {
                    Ok(resp) => {
                        ar_total.set(Some(resp.total));
                        ar_items.set(resp.requests);
                    },
                    Err(e) => {
                        console::error_1(&format!("Failed to fetch article requests: {e}").into());
                    },
                }
                ar_loaded.set(true);
            });
            || ()
        });
    }

    // Fetch latest 4 music wishes (refreshed after submit)
    {
        let mw_items = mw_items.clone();
        let mw_total = mw_total.clone();
        let mw_loaded = mw_loaded.clone();
        let seq = *mw_refresh_seq;
        use_effect_with(seq, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_music_wishes(Some(4), Some(0)).await {
                    Ok(resp) => {
                        mw_total.set(Some(resp.total));
                        mw_items.set(resp.wishes);
                    },
                    Err(e) => {
                        console::error_1(&format!("Failed to fetch music wishes: {e}").into());
                    },
                }
                mw_loaded.set(true);
            });
            || ()
        });
    }

    let reader_ingestion_total: Option<usize> = match (*ar_total, *mw_total) {
        (Some(a), Some(b)) => Some(a + b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    let on_hero_cta = {
        let active_tab = active_tab.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            active_tab.set(WorkshopTab::Article);
            if let Some(win) = web_sys::window() {
                if let Ok(history) = win.history() {
                    let _ = history.replace_state_with_url(
                        &wasm_bindgen::JsValue::NULL,
                        "",
                        Some("#workshop"),
                    );
                }
            }
            smooth_scroll_to_id("workshop");
        })
    };

    let search_href = crate::config::route_path("/search");
    let on_search_click = {
        let search_href = search_href.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&search_href);
        })
    };

    let full_ar_queue_href = crate::config::route_path("/latest#article-request-section");
    let go_full_ar_queue = {
        let href = full_ar_queue_href.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&href);
        })
    };

    let full_mw_queue_href = crate::config::route_path("/media/audio#music-wish-section");
    let go_full_mw_queue = {
        let href = full_mw_queue_href.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&href);
        })
    };

    // Article Request submission handler
    let on_submit_article = {
        let ar_url = ar_url.clone();
        let ar_title = ar_title.clone();
        let ar_message = ar_message.clone();
        let ar_nickname = ar_nickname.clone();
        let ar_email = ar_email.clone();
        let ar_submitting = ar_submitting.clone();
        let ar_error = ar_error.clone();
        let ar_submitted_id = ar_submitted_id.clone();
        let ar_refresh_seq = ar_refresh_seq.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if *ar_submitting {
                return;
            }
            let url = (*ar_url).trim().to_string();
            let title = (*ar_title).trim().to_string();
            let message = (*ar_message).trim().to_string();
            let nickname = (*ar_nickname).trim().to_string();
            let email = (*ar_email).trim().to_string();

            if url.is_empty() || (!url.starts_with("http://") && !url.starts_with("https://")) {
                ar_error.set(Some(
                    "请填写完整的外部文章链接（需以 https:// 或 http:// 开头）。".to_string(),
                ));
                return;
            }
            if message.is_empty() {
                ar_error.set(Some("请填写简短说明或希望关注的解读 / 翻译角度。".to_string()));
                return;
            }

            ar_error.set(None);
            ar_submitting.set(true);
            let frontend_page_url = web_sys::window().and_then(|w| w.location().href().ok());
            let ar_submitting = ar_submitting.clone();
            let ar_error = ar_error.clone();
            let ar_submitted_id = ar_submitted_id.clone();
            let ar_url = ar_url.clone();
            let ar_title = ar_title.clone();
            let ar_message = ar_message.clone();
            let ar_refresh_seq = ar_refresh_seq.clone();
            let next_seq = (*ar_refresh_seq).wrapping_add(1);

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
                    None,
                )
                .await
                {
                    Ok(resp) => {
                        ar_url.set(String::new());
                        ar_title.set(String::new());
                        ar_message.set(String::new());
                        ar_submitted_id.set(Some(resp.request_id));
                        ar_refresh_seq.set(next_seq);
                    },
                    Err(err) => {
                        ar_error.set(Some(err));
                    },
                }
                ar_submitting.set(false);
            });
        })
    };

    // Music Wish submission handler
    let on_submit_music = {
        let mw_song = mw_song.clone();
        let mw_artist = mw_artist.clone();
        let mw_message = mw_message.clone();
        let mw_nickname = mw_nickname.clone();
        let mw_email = mw_email.clone();
        let mw_submitting = mw_submitting.clone();
        let mw_error = mw_error.clone();
        let mw_submitted_id = mw_submitted_id.clone();
        let mw_refresh_seq = mw_refresh_seq.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if *mw_submitting {
                return;
            }
            let song = (*mw_song).trim().to_string();
            let artist = (*mw_artist).trim().to_string();
            let message = (*mw_message).trim().to_string();
            let nickname = (*mw_nickname).trim().to_string();
            let email = (*mw_email).trim().to_string();

            if song.is_empty() {
                mw_error.set(Some(mw_t::SONG_NAME_REQUIRED.to_string()));
                return;
            }
            if message.is_empty() {
                mw_error
                    .set(Some("请填写一句点歌留言或版本偏好，方便 AI 准确检索曲源。".to_string()));
                return;
            }

            mw_error.set(None);
            mw_submitting.set(true);
            let frontend_page_url = web_sys::window().and_then(|w| w.location().href().ok());
            let mw_submitting = mw_submitting.clone();
            let mw_error = mw_error.clone();
            let mw_submitted_id = mw_submitted_id.clone();
            let mw_song = mw_song.clone();
            let mw_artist = mw_artist.clone();
            let mw_message = mw_message.clone();
            let mw_refresh_seq = mw_refresh_seq.clone();
            let next_seq = (*mw_refresh_seq).wrapping_add(1);

            wasm_bindgen_futures::spawn_local(async move {
                let artist_opt = if artist.is_empty() { None } else { Some(artist.as_str()) };
                let nick_opt = if nickname.is_empty() { None } else { Some(nickname.as_str()) };
                let email_opt = if email.is_empty() { None } else { Some(email.as_str()) };
                match api::submit_music_wish(
                    &song,
                    artist_opt,
                    &message,
                    nick_opt,
                    email_opt,
                    frontend_page_url.as_deref(),
                )
                .await
                {
                    Ok(resp) => {
                        mw_song.set(String::new());
                        mw_artist.set(String::new());
                        mw_message.set(String::new());
                        mw_submitted_id.set(Some(resp.wish_id));
                        mw_refresh_seq.set(next_seq);
                    },
                    Err(err) => {
                        mw_error.set(Some(err));
                    },
                }
                mw_submitting.set(false);
            });
        })
    };

    let ar_has_running = ar_items.iter().any(|r| r.status == "running");
    let mw_has_running = mw_items.iter().any(|w| w.status == "running");

    // Collect fulfilled wish song IDs so acoustic track rows can badge reader
    // wishes
    let wish_song_ids: Vec<String> = mw_items
        .iter()
        .filter_map(|w| w.ingested_song_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    // Build real metric segments (only display items whose real API returned)
    let mut metric_nodes: Vec<Html> = Vec::new();
    if let Some(count) = *total_articles {
        metric_nodes.push(html! {
            <Link<Route> to={Route::Posts} classes="no-underline text-[var(--muted)] hover:text-[var(--text)] transition-colors">
                <span class="text-[var(--text)] font-medium">{ count }</span>
                { " 篇文章" }
            </Link<Route>>
        });
    }
    if let Some(count) = *total_music {
        metric_nodes.push(html! {
            <Link<Route> to={Route::MediaAudio} classes="no-underline text-[var(--muted)] hover:text-[var(--text)] transition-colors">
                <span class="text-[var(--text)] font-medium">{ count }</span>
                { " 首曲目" }
            </Link<Route>>
        });
    }
    if let Some(count) = reader_ingestion_total {
        let scroll_workshop = on_hero_cta.clone();
        metric_nodes.push(html! {
            <a href="#workshop" onclick={scroll_workshop} class="no-underline text-[var(--muted)] hover:text-[var(--text)] transition-colors">
                <span class="text-[var(--text)] font-medium">{ count }</span>
                { " 条读者入库" }
            </a>
        });
    }

    html! {
        <main class="w-full min-h-screen bg-[var(--bg)] text-[var(--text)]">
            // ① Anthropic Editorial Left-Aligned Hero
            <section class="max-w-[1200px] mx-auto px-6 md:px-10 pt-16 md:pt-24 pb-16 md:pb-20">
                <div class="max-w-[880px]">
                    <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                        { "LOCAL-FIRST · AI-ASSISTED · READER-DRIVEN" }
                    </div>

                    <h1 class="mt-5 mb-0 font-serif font-normal text-[clamp(2.4rem,1.6rem+3.5vw,4.25rem)] leading-[1.08] tracking-[-0.02em] text-[var(--text)]">
                        { "A library that readers help write." }
                        <span class="block mt-2.5 text-[var(--muted)] text-[clamp(1.65rem,1.25rem+1.9vw,2.65rem)] leading-[1.2] tracking-[-0.015em]">
                            { "一座由读者共同生长的技术文库。" }
                        </span>
                    </h1>

                    <p class="mt-6 mb-0 text-base sm:text-[1.125rem] leading-[1.68] text-[var(--text-2)] max-w-[60ch]">
                        { "任何访客皆可自助提交外部技术好文或心仪曲目，由本地 AI 工作流自动调研、双语精译、抓取音频歌词并持久入库 LanceDB。" }
                    </p>

                    <div class="mt-8 flex flex-wrap items-center gap-3">
                        <a
                            href="#workshop"
                            onclick={on_hero_cta.clone()}
                            class="sf-btn sf-btn--primary no-underline"
                        >
                            { "提交入库 / 点歌" }
                            <span aria-hidden="true">{ "→" }</span>
                        </a>
                        <Link<Route>
                            to={Route::Posts}
                            classes="sf-btn sf-btn--secondary no-underline"
                        >
                            { "浏览全部文章" }
                        </Link<Route>>
                        <a
                            href={search_href}
                            onclick={on_search_click}
                            class="sf-btn sf-btn--ghost no-underline"
                        >
                            <i class="fas fa-magnifying-glass text-xs opacity-70" aria-hidden="true"></i>
                            { "全站检索" }
                        </a>
                    </div>

                    <div
                        class="mt-12 pt-6 border-t border-[var(--border)] flex flex-wrap items-center gap-x-4 gap-y-2 font-mono text-xs sm:text-[0.8125rem] tabular-nums text-[var(--faint)]"
                        aria-label="文库实时指标"
                    >
                        if metric_nodes.is_empty() {
                            <span class="skeleton inline-block h-4 w-56 rounded" aria-hidden="true" />
                        } else {
                            { for metric_nodes.into_iter().enumerate().map(|(idx, node)| html! {
                                <>
                                    if idx > 0 {
                                        <span class="text-[var(--border-strong)] select-none" aria-hidden="true">{ "│" }</span>
                                    }
                                    { node }
                                </>
                            }) }
                        }
                    </div>
                </div>
            </section>

            // ② Reader-Driven AI Ingestion Workshop (#workshop)
            <section
                id="workshop"
                class="w-full bg-[var(--surface-alt)] border-y border-[var(--border)] py-16 md:py-20 scroll-mt-16"
                aria-labelledby="workshop-heading"
            >
                <div class="max-w-[1200px] mx-auto px-6 md:px-10">
                    <div class="max-w-[68ch]">
                        <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                            { "READER-DRIVEN AI INGESTION" }
                        </div>
                        <h2
                            id="workshop-heading"
                            class="mt-2.5 mb-0 font-serif font-normal text-[clamp(1.5rem,1.25rem+1vw,2rem)] leading-[1.2] tracking-[-0.015em] text-[var(--text)]"
                        >
                            { "自助入库工坊" }
                        </h2>
                        <p class="mt-2.5 mb-0 text-[0.95rem] sm:text-base leading-[1.65] text-[var(--text-2)]">
                            { "你提交，AI 调研 · 翻译 · 抓取 · 入库，全程公开可追踪。" }
                        </p>
                    </div>

                    <div class="mt-10 grid grid-cols-1 lg:grid-cols-[1.1fr_0.9fr] gap-10 items-start">
                        // Left: In-place submission panel
                        <div class="sf-card p-6 sm:p-8 bg-[var(--surface)] border border-[var(--border)] rounded-[12px]">
                            // Clean text segmented tabs
                            <div
                                class="flex items-center gap-7 border-b border-[var(--border)] mb-6"
                                role="tablist"
                                aria-label="入库类型"
                            >
                                {
                                    for [
                                        (WorkshopTab::Article, "文章入库"),
                                        (WorkshopTab::Music, "点歌心愿"),
                                    ]
                                    .into_iter()
                                    .map(|(tab, label)| {
                                        let is_active = *active_tab == tab;
                                        let active_tab = active_tab.clone();
                                        let onclick = Callback::from(move |_: MouseEvent| {
                                            active_tab.set(tab);
                                        });
                                        html! {
                                            <button
                                                type="button"
                                                role="tab"
                                                aria-selected={is_active.to_string()}
                                                {onclick}
                                                class={classes!(
                                                    "pb-3",
                                                    "-mb-px",
                                                    "text-sm",
                                                    "font-medium",
                                                    "bg-transparent",
                                                    "border-0",
                                                    "border-b-[1.5px]",
                                                    "cursor-pointer",
                                                    "transition-colors",
                                                    "duration-150",
                                                    if is_active {
                                                        "border-[var(--primary)] text-[var(--text)]"
                                                    } else {
                                                        "border-transparent text-[var(--muted)] hover:text-[var(--text)]"
                                                    }
                                                )}
                                            >
                                                { label }
                                            </button>
                                        }
                                    })
                                }
                            </div>

                            {
                                match *active_tab {
                                    WorkshopTab::Article => {
                                        if let Some(req_id) = (*ar_submitted_id).clone() {
                                            let reset_ar = {
                                                let ar_submitted_id = ar_submitted_id.clone();
                                                Callback::from(move |_: MouseEvent| ar_submitted_id.set(None))
                                            };
                                            let queue_pos = ar_total.unwrap_or(1);
                                            html! {
                                                <div class="py-6 space-y-5">
                                                    <div class="flex items-start gap-3.5">
                                                        <span
                                                            class="w-9 h-9 rounded-full bg-[var(--ok-soft)] text-[var(--ok)] inline-flex items-center justify-center shrink-0 font-mono text-sm"
                                                            aria-hidden="true"
                                                        >
                                                            { "✓" }
                                                        </span>
                                                        <div class="space-y-1.5">
                                                            <h3 class="m-0 font-serif font-normal text-xl text-[var(--text)]">
                                                                { format!("已进入入库队列（当前累计第 {queue_pos} 条请求）。") }
                                                            </h3>
                                                            <p class="m-0 text-sm leading-relaxed text-[var(--muted)]">
                                                                { "任务通过校验后将自动拉取原文、双语精译并写入本地 LanceDB。" }
                                                            </p>
                                                            <p class="m-0 font-mono text-xs text-[var(--faint)]">
                                                                { format!("request_id: {req_id}") }
                                                            </p>
                                                        </div>
                                                    </div>

                                                    <div class="flex flex-wrap items-center gap-3 pt-2">
                                                        <a
                                                            href={full_ar_queue_href.clone()}
                                                            onclick={go_full_ar_queue.clone()}
                                                            class="sf-btn sf-btn--primary sf-btn--sm no-underline"
                                                        >
                                                            { "前往查看完整队列与 AI 实时执行流 →" }
                                                        </a>
                                                        <button
                                                            type="button"
                                                            onclick={reset_ar}
                                                            class="sf-btn sf-btn--secondary sf-btn--sm"
                                                        >
                                                            { "继续提交" }
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            let presets = [
                                                "双语精译并梳理核心架构机制",
                                                "保留源码细节与工程权衡对比",
                                                "提炼可复现结论与关键数据",
                                            ];
                                            html! {
                                                <form onsubmit={on_submit_article} class="space-y-4">
                                                    <div class="flex flex-col gap-1.5">
                                                        <label for="home-ar-url" class="text-xs font-medium text-[var(--text-2)]">
                                                            { format!("{} *", ar_t::URL_LABEL) }
                                                        </label>
                                                        <input
                                                            id="home-ar-url"
                                                            type="url"
                                                            placeholder="https://… 粘贴想读的外部技术好文链接"
                                                            value={(*ar_url).clone()}
                                                            oninput={
                                                                let ar_url = ar_url.clone();
                                                                let ar_error = ar_error.clone();
                                                                Callback::from(move |e: InputEvent| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    ar_url.set(input.value());
                                                                    ar_error.set(None);
                                                                })
                                                            }
                                                            class="sf-input w-full"
                                                            required=true
                                                        />
                                                    </div>

                                                    <div class="flex flex-col gap-1.5">
                                                        <div class="flex flex-wrap items-center justify-between gap-2">
                                                            <label for="home-ar-msg" class="text-xs font-medium text-[var(--text-2)]">
                                                                { "关注角度 / 推荐说明 *" }
                                                            </label>
                                                            <div class="flex flex-wrap gap-1.5">
                                                                { for presets.into_iter().map(|preset| {
                                                                    let ar_message = ar_message.clone();
                                                                    let ar_error = ar_error.clone();
                                                                    let onclick = Callback::from(move |_: MouseEvent| {
                                                                        ar_message.set(preset.to_string());
                                                                        ar_error.set(None);
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
                                                            id="home-ar-msg"
                                                            rows="3"
                                                            placeholder="希望 AI 重点剖析的章节、翻译要求或推荐理由…"
                                                            value={(*ar_message).clone()}
                                                            oninput={
                                                                let ar_message = ar_message.clone();
                                                                let ar_error = ar_error.clone();
                                                                Callback::from(move |e: InputEvent| {
                                                                    let input: HtmlTextAreaElement = e.target_unchecked_into();
                                                                    ar_message.set(input.value());
                                                                    ar_error.set(None);
                                                                })
                                                            }
                                                            class="sf-textarea w-full min-h-[88px] py-2.5 resize-y"
                                                            required=true
                                                        />
                                                    </div>

                                                    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                                                        <div class="flex flex-col gap-1">
                                                            <label for="home-ar-title" class="text-xs text-[var(--muted)]">
                                                                { "标题提示（可选）" }
                                                            </label>
                                                            <input
                                                                id="home-ar-title"
                                                                type="text"
                                                                placeholder="原文标题"
                                                                value={(*ar_title).clone()}
                                                                oninput={
                                                                    let s = ar_title.clone();
                                                                    Callback::from(move |e: InputEvent| {
                                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                                        s.set(input.value());
                                                                    })
                                                                }
                                                                class="sf-input w-full"
                                                            />
                                                        </div>
                                                        <div class="flex flex-col gap-1">
                                                            <label for="home-ar-nick" class="text-xs text-[var(--muted)]">
                                                                { ar_t::NICKNAME_LABEL }
                                                            </label>
                                                            <input
                                                                id="home-ar-nick"
                                                                type="text"
                                                                placeholder="匿名读者"
                                                                value={(*ar_nickname).clone()}
                                                                oninput={
                                                                    let s = ar_nickname.clone();
                                                                    Callback::from(move |e: InputEvent| {
                                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                                        s.set(input.value());
                                                                    })
                                                                }
                                                                class="sf-input w-full"
                                                            />
                                                        </div>
                                                        <div class="flex flex-col gap-1">
                                                            <label for="home-ar-email" class="text-xs text-[var(--muted)]">
                                                                { ar_t::EMAIL_LABEL }
                                                            </label>
                                                            <input
                                                                id="home-ar-email"
                                                                type="email"
                                                                placeholder="完成通知邮箱"
                                                                value={(*ar_email).clone()}
                                                                oninput={
                                                                    let s = ar_email.clone();
                                                                    Callback::from(move |e: InputEvent| {
                                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                                        s.set(input.value());
                                                                    })
                                                                }
                                                                class="sf-input w-full"
                                                            />
                                                        </div>
                                                    </div>

                                                    if let Some(err) = (*ar_error).clone() {
                                                        <div
                                                            class="rounded-[8px] bg-[var(--err-soft)] text-[var(--err)] px-3.5 py-2.5 text-xs leading-relaxed"
                                                            role="alert"
                                                        >
                                                            { err }
                                                        </div>
                                                    }

                                                    <div class="pt-1 flex flex-wrap items-center justify-between gap-3">
                                                        <span class="text-xs text-[var(--faint)]">
                                                            { ar_t::EMAIL_HELP_TEXT }
                                                        </span>
                                                        <button
                                                            type="submit"
                                                            disabled={*ar_submitting}
                                                            class="sf-btn sf-btn--primary disabled:opacity-50 disabled:cursor-not-allowed"
                                                        >
                                                            { if *ar_submitting { ar_t::SUBMITTING } else { "提交入库申请 →" } }
                                                        </button>
                                                    </div>
                                                </form>
                                            }
                                        }
                                    },
                                    WorkshopTab::Music => {
                                        if let Some(wish_id) = (*mw_submitted_id).clone() {
                                            let reset_mw = {
                                                let mw_submitted_id = mw_submitted_id.clone();
                                                Callback::from(move |_: MouseEvent| mw_submitted_id.set(None))
                                            };
                                            let queue_pos = mw_total.unwrap_or(1);
                                            html! {
                                                <div class="py-6 space-y-5">
                                                    <div class="flex items-start gap-3.5">
                                                        <span
                                                            class="w-9 h-9 rounded-full bg-[var(--ok-soft)] text-[var(--ok)] inline-flex items-center justify-center shrink-0 font-mono text-sm"
                                                            aria-hidden="true"
                                                        >
                                                            { "✓" }
                                                        </span>
                                                        <div class="space-y-1.5">
                                                            <h3 class="m-0 font-serif font-normal text-xl text-[var(--text)]">
                                                                { format!("歌愿已收入队列（当前累计第 {queue_pos} 条心愿）。") }
                                                            </h3>
                                                            <p class="m-0 text-sm leading-relaxed text-[var(--muted)]">
                                                                { "AI 将自动检索音源、提取封面与时间轴歌词并入库本地曲库。" }
                                                            </p>
                                                            <p class="m-0 font-mono text-xs text-[var(--faint)]">
                                                                { format!("wish_id: {wish_id}") }
                                                            </p>
                                                        </div>
                                                    </div>

                                                    <div class="flex flex-wrap items-center gap-3 pt-2">
                                                        <a
                                                            href={full_mw_queue_href.clone()}
                                                            onclick={go_full_mw_queue.clone()}
                                                            class="sf-btn sf-btn--primary sf-btn--sm no-underline"
                                                        >
                                                            { "前往曲库查看心愿进度 →" }
                                                        </a>
                                                        <button
                                                            type="button"
                                                            onclick={reset_mw}
                                                            class="sf-btn sf-btn--secondary sf-btn--sm"
                                                        >
                                                            { "继续许愿" }
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            let wish_presets = [
                                                "希望收录高音质与双语歌词",
                                                "写代码时常听的安静曲目",
                                                "推荐原版录音室专辑版本",
                                            ];
                                            html! {
                                                <form onsubmit={on_submit_music} class="space-y-4">
                                                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
                                                        <div class="flex flex-col gap-1.5">
                                                            <label for="home-mw-song" class="text-xs font-medium text-[var(--text-2)]">
                                                                { format!("{} *", mw_t::SONG_NAME_LABEL) }
                                                            </label>
                                                            <input
                                                                id="home-mw-song"
                                                                type="text"
                                                                placeholder={mw_t::SONG_NAME_PLACEHOLDER}
                                                                value={(*mw_song).clone()}
                                                                oninput={
                                                                    let mw_song = mw_song.clone();
                                                                    let mw_error = mw_error.clone();
                                                                    Callback::from(move |e: InputEvent| {
                                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                                        mw_song.set(input.value());
                                                                        mw_error.set(None);
                                                                    })
                                                                }
                                                                class="sf-input w-full"
                                                                required=true
                                                            />
                                                        </div>
                                                        <div class="flex flex-col gap-1.5">
                                                            <label for="home-mw-artist" class="text-xs font-medium text-[var(--text-2)]">
                                                                { mw_t::ARTIST_LABEL }
                                                            </label>
                                                            <input
                                                                id="home-mw-artist"
                                                                type="text"
                                                                placeholder={mw_t::ARTIST_PLACEHOLDER}
                                                                value={(*mw_artist).clone()}
                                                                oninput={
                                                                    let s = mw_artist.clone();
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
                                                        <div class="flex flex-wrap items-center justify-between gap-2">
                                                            <label for="home-mw-msg" class="text-xs font-medium text-[var(--text-2)]">
                                                                { format!("{} *", mw_t::MESSAGE_LABEL) }
                                                            </label>
                                                            <div class="flex flex-wrap gap-1.5">
                                                                { for wish_presets.into_iter().map(|preset| {
                                                                    let mw_message = mw_message.clone();
                                                                    let mw_error = mw_error.clone();
                                                                    let onclick = Callback::from(move |_: MouseEvent| {
                                                                        mw_message.set(preset.to_string());
                                                                        mw_error.set(None);
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
                                                            id="home-mw-msg"
                                                            rows="3"
                                                            placeholder={mw_t::MESSAGE_PLACEHOLDER}
                                                            value={(*mw_message).clone()}
                                                            oninput={
                                                                let mw_message = mw_message.clone();
                                                                let mw_error = mw_error.clone();
                                                                Callback::from(move |e: InputEvent| {
                                                                    let input: HtmlTextAreaElement = e.target_unchecked_into();
                                                                    mw_message.set(input.value());
                                                                    mw_error.set(None);
                                                                })
                                                            }
                                                            class="sf-textarea w-full min-h-[88px] py-2.5 resize-y"
                                                            required=true
                                                        />
                                                    </div>

                                                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
                                                        <div class="flex flex-col gap-1">
                                                            <label for="home-mw-nick" class="text-xs text-[var(--muted)]">
                                                                { mw_t::NICKNAME_LABEL }
                                                            </label>
                                                            <input
                                                                id="home-mw-nick"
                                                                type="text"
                                                                placeholder="匿名听众"
                                                                value={(*mw_nickname).clone()}
                                                                oninput={
                                                                    let s = mw_nickname.clone();
                                                                    Callback::from(move |e: InputEvent| {
                                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                                        s.set(input.value());
                                                                    })
                                                                }
                                                                class="sf-input w-full"
                                                            />
                                                        </div>
                                                        <div class="flex flex-col gap-1">
                                                            <label for="home-mw-email" class="text-xs text-[var(--muted)]">
                                                                { mw_t::EMAIL_LABEL }
                                                            </label>
                                                            <input
                                                                id="home-mw-email"
                                                                type="email"
                                                                placeholder="完成通知邮箱"
                                                                value={(*mw_email).clone()}
                                                                oninput={
                                                                    let s = mw_email.clone();
                                                                    Callback::from(move |e: InputEvent| {
                                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                                        s.set(input.value());
                                                                    })
                                                                }
                                                                class="sf-input w-full"
                                                            />
                                                        </div>
                                                    </div>

                                                    if let Some(err) = (*mw_error).clone() {
                                                        <div
                                                            class="rounded-[8px] bg-[var(--err-soft)] text-[var(--err)] px-3.5 py-2.5 text-xs leading-relaxed"
                                                            role="alert"
                                                        >
                                                            { err }
                                                        </div>
                                                    }

                                                    <div class="pt-1 flex flex-wrap items-center justify-between gap-3">
                                                        <span class="text-xs text-[var(--faint)]">
                                                            { mw_t::EMAIL_HELP_TEXT }
                                                        </span>
                                                        <button
                                                            type="submit"
                                                            disabled={*mw_submitting}
                                                            class="sf-btn sf-btn--primary disabled:opacity-50 disabled:cursor-not-allowed"
                                                        >
                                                            { if *mw_submitting { mw_t::SUBMITTING } else { "许下歌愿 →" } }
                                                        </button>
                                                    </div>
                                                </form>
                                            }
                                        }
                                    },
                                }
                            }

                            <div class="mt-6 pt-4 border-t border-[var(--border)] font-mono text-[0.6875rem] tracking-[0.04em] text-[var(--faint)]">
                                { "submit → queue → AI research & ingest → LanceDB publish" }
                            </div>
                        </div>

                        // Right: Live Co-Creation Queues
                        <div class="flex flex-col gap-8">
                            // Queue 1: Article Ingestion Activity
                            <div>
                                <div class="flex items-center justify-between gap-3 pb-3 border-b border-[var(--border-strong)]">
                                    <div class="flex items-center gap-2.5">
                                        <span class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                                            { "文章入库动态" }
                                        </span>
                                        if ar_has_running {
                                            <span class="inline-flex items-center gap-1.5 font-mono text-[11px] font-semibold uppercase tracking-[0.08em] text-[var(--primary)]">
                                                <span class="w-1.5 h-1.5 rounded-full bg-[var(--primary)] animate-pulse motion-reduce:animate-none" aria-hidden="true" />
                                                { "LIVE" }
                                            </span>
                                        }
                                    </div>
                                    <a
                                        href={full_ar_queue_href}
                                        onclick={go_full_ar_queue}
                                        class="font-mono text-xs text-[var(--muted)] hover:text-[var(--primary)] no-underline transition-colors"
                                    >
                                        { "完整队列 →" }
                                    </a>
                                </div>

                                if !*ar_loaded {
                                    <div class="divide-y divide-[var(--border)]" aria-hidden="true">
                                        { for (0..3).map(|i| html! {
                                            <div key={i} class="py-3.5 flex items-center justify-between gap-3">
                                                <span class="skeleton h-4 w-3/4 rounded" />
                                                <span class="skeleton h-4 w-14 rounded-full" />
                                            </div>
                                        }) }
                                    </div>
                                } else if ar_items.is_empty() {
                                    <p class="my-6 font-serif italic text-[0.95rem] text-[var(--muted)]">
                                        { "队列很安静——来提交第一篇吧。" }
                                    </p>
                                } else {
                                    <ul class="m-0 p-0 list-none divide-y divide-[var(--border)] border-b border-[var(--border)]">
                                        { for ar_items.iter().take(4).map(|req| {
                                            let (status_key, status_txt) = article_request_status_label(req);
                                            let display_title = req
                                                .title_hint
                                                .as_deref()
                                                .map(str::trim)
                                                .filter(|s| !s.is_empty())
                                                .map(str::to_string)
                                                .unwrap_or_else(|| strip_url_display(&req.article_url));
                                            let rel_time = format_relative_time_zh(req.created_at);
                                            let article_id = req
                                                .ingested_article_id
                                                .as_deref()
                                                .map(str::trim)
                                                .filter(|s| !s.is_empty())
                                                .map(str::to_string);

                                            html! {
                                                <li key={req.request_id.clone()} class="py-3.5 flex items-center justify-between gap-3">
                                                    <div class="min-w-0 flex-1">
                                                        <div class="flex items-center gap-2.5 min-w-0">
                                                            { status_badge_view(status_key, status_txt) }
                                                            <span class="font-serif text-[0.9375rem] text-[var(--text)] truncate">
                                                                { display_title }
                                                            </span>
                                                        </div>
                                                        <div class="mt-1 flex items-center gap-2 font-mono text-[11px] tabular-nums text-[var(--faint)]">
                                                            <span class="truncate max-w-[140px]">{ &req.nickname }</span>
                                                            if !rel_time.is_empty() {
                                                                <span aria-hidden="true">{ "·" }</span>
                                                                <span>{ rel_time }</span>
                                                            }
                                                        </div>
                                                    </div>

                                                    if req.status == "done" {
                                                        if let Some(aid) = article_id {
                                                            <Link<Route>
                                                                to={Route::ArticleDetail { id: aid }}
                                                                classes="shrink-0 font-mono text-xs text-[var(--primary)] hover:underline no-underline"
                                                            >
                                                                { "阅读 →" }
                                                            </Link<Route>>
                                                        }
                                                    }
                                                </li>
                                            }
                                        }) }
                                    </ul>
                                }
                            </div>

                            // Queue 2: Music Wish Activity
                            <div>
                                <div class="flex items-center justify-between gap-3 pb-3 border-b border-[var(--border-strong)]">
                                    <div class="flex items-center gap-2.5">
                                        <span class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                                            { "点歌心愿动态" }
                                        </span>
                                        if mw_has_running {
                                            <span class="inline-flex items-center gap-1.5 font-mono text-[11px] font-semibold uppercase tracking-[0.08em] text-[var(--primary)]">
                                                <span class="w-1.5 h-1.5 rounded-full bg-[var(--primary)] animate-pulse motion-reduce:animate-none" aria-hidden="true" />
                                                { "LIVE" }
                                            </span>
                                        }
                                    </div>
                                    <a
                                        href={full_mw_queue_href}
                                        onclick={go_full_mw_queue}
                                        class="font-mono text-xs text-[var(--muted)] hover:text-[var(--primary)] no-underline transition-colors"
                                    >
                                        { "完整队列 →" }
                                    </a>
                                </div>

                                if !*mw_loaded {
                                    <div class="divide-y divide-[var(--border)]" aria-hidden="true">
                                        { for (0..3).map(|i| html! {
                                            <div key={i} class="py-3.5 flex items-center justify-between gap-3">
                                                <span class="skeleton h-4 w-2/3 rounded" />
                                                <span class="skeleton h-4 w-14 rounded-full" />
                                            </div>
                                        }) }
                                    </div>
                                } else if mw_items.is_empty() {
                                    <p class="my-6 font-serif italic text-[0.95rem] text-[var(--muted)]">
                                        { "队列很安静——来许下第一首歌愿吧。" }
                                    </p>
                                } else {
                                    <ul class="m-0 p-0 list-none divide-y divide-[var(--border)] border-b border-[var(--border)]">
                                        { for mw_items.iter().take(4).map(|wish| {
                                            let (status_key, status_txt) = music_wish_status_label(wish);
                                            let title_line = match wish.artist_hint.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                                                Some(artist) => format!("{} — {}", wish.song_name, artist),
                                                None => wish.song_name.clone(),
                                            };
                                            let rel_time = format_relative_time_zh(wish.created_at);
                                            let song_id = wish
                                                .ingested_song_id
                                                .as_deref()
                                                .map(str::trim)
                                                .filter(|s| !s.is_empty())
                                                .map(str::to_string);

                                            html! {
                                                <li key={wish.wish_id.clone()} class="py-3.5 flex items-center justify-between gap-3">
                                                    <div class="min-w-0 flex-1">
                                                        <div class="flex items-center gap-2.5 min-w-0">
                                                            { status_badge_view(status_key, status_txt) }
                                                            <span class="font-serif text-[0.9375rem] text-[var(--text)] truncate">
                                                                { title_line }
                                                            </span>
                                                        </div>
                                                        <div class="mt-1 flex items-center gap-2 font-mono text-[11px] tabular-nums text-[var(--faint)]">
                                                            <span class="truncate max-w-[140px]">{ &wish.nickname }</span>
                                                            if !rel_time.is_empty() {
                                                                <span aria-hidden="true">{ "·" }</span>
                                                                <span>{ rel_time }</span>
                                                            }
                                                        </div>
                                                    </div>

                                                    if wish.status == "done" {
                                                        if let Some(sid) = song_id {
                                                            <Link<Route>
                                                                to={Route::MusicPlayer { id: sid }}
                                                                classes="shrink-0 font-mono text-xs text-[var(--primary)] hover:underline no-underline"
                                                            >
                                                                { "播放 →" }
                                                            </Link<Route>>
                                                        }
                                                    }
                                                </li>
                                            }
                                        }) }
                                    </ul>
                                }
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            // ③ Recent Articles (Magazine Lead + Journal List + 3-Col Grid)
            <section class="max-w-[1200px] mx-auto px-6 md:px-10 py-20 md:py-24">
                <div class="flex items-baseline justify-between gap-4 pb-6 border-b border-[var(--border)]">
                    <div>
                        <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                            { "EDITORIAL DISPATCHES" }
                        </div>
                        <h2 class="mt-2 mb-0 font-serif font-normal text-[clamp(1.5rem,1.25rem+1vw,2rem)] leading-[1.2] tracking-[-0.015em] text-[var(--text)]">
                            { "最近的文章" }
                        </h2>
                    </div>
                    <Link<Route>
                        to={Route::LatestArticles}
                        classes="sf-btn sf-btn--ghost sf-btn--sm !px-0 no-underline shrink-0"
                    >
                        { "全部 →" }
                    </Link<Route>>
                </div>

                if !*articles_loaded {
                    <div class="mt-10 grid grid-cols-1 lg:grid-cols-[1.4fr_1fr] gap-10" aria-hidden="true">
                        <div class="space-y-4">
                            <div class="skeleton aspect-[16/10] w-full rounded-[10px]" />
                            <div class="skeleton h-4 w-32 rounded" />
                            <div class="skeleton h-8 w-4/5 rounded" />
                            <div class="skeleton h-4 w-full rounded" />
                        </div>
                        <div class="divide-y divide-[var(--border)]">
                            { for (0..3).map(|idx| html! {
                                <div key={idx} class="py-5 first:pt-0 space-y-2.5">
                                    <div class="skeleton h-3 w-28 rounded" />
                                    <div class="skeleton h-6 w-5/6 rounded" />
                                    <div class="skeleton h-4 w-full rounded" />
                                </div>
                            }) }
                        </div>
                    </div>
                } else if recent_articles.is_empty() {
                    <p class="mt-10 font-serif italic text-lg text-[var(--muted)]">
                        { "暂无已发布文章。" }
                    </p>
                } else {
                    if let Some(lead_article) = recent_articles.first() {
                        <div class="mt-10 grid grid-cols-1 lg:grid-cols-[1.4fr_1fr] gap-10 lg:gap-12 items-start">
                            // Lead Feature Story
                            <article class="group flex flex-col gap-3.5">
                                if let Some(img) = lead_article
                                    .featured_image
                                    .as_deref()
                                    .map(str::trim)
                                    .filter(|s| !s.is_empty())
                                {
                                    <Link<Route>
                                        to={Route::ArticleDetail { id: lead_article.id.clone() }}
                                        classes="block aspect-[16/10] rounded-[10px] overflow-hidden border border-[var(--border)] bg-[var(--surface-alt)] hover:border-[var(--border-strong)] transition-colors no-underline"
                                    >
                                        <ImageWithLoading
                                            src={image_url(img)}
                                            alt={lead_article.title.clone()}
                                            loading={Some(AttrValue::from("eager"))}
                                            class={classes!("w-full", "h-full", "object-cover")}
                                            container_class={classes!("w-full", "h-full")}
                                        />
                                    </Link<Route>>
                                }

                                <div class="flex items-center justify-between gap-2 flex-wrap pt-1">
                                    <div class="flex items-center gap-2">
                                        if !lead_article.category.trim().is_empty() {
                                            <Link<Route>
                                                to={Route::CategoryDetail { category: lead_article.category.clone() }}
                                                classes="text-xs font-semibold uppercase tracking-[0.08em] text-[var(--muted)] hover:text-[var(--primary)] no-underline transition-colors"
                                            >
                                                { &lead_article.category }
                                            </Link<Route>>
                                            <span class="text-[var(--faint)] text-xs" aria-hidden="true">{ "·" }</span>
                                        }
                                        <time class="font-mono text-xs tabular-nums text-[var(--faint)]" datetime={lead_article.date.clone()}>
                                            { &lead_article.date }
                                        </time>
                                    </div>

                                    if matches!(lead_article.article_kind, ArticleKind::InteractiveRepost) {
                                        <span class="sf-chip !bg-[var(--primary-soft)] !text-[var(--primary)] !border-transparent font-mono !text-[11px]">
                                            { "◆ Interactive" }
                                        </span>
                                    }
                                </div>

                                <h3 class="m-0 font-serif font-normal text-[clamp(1.5rem,1.25rem+1vw,2.1rem)] leading-[1.2] tracking-[-0.015em]">
                                    <Link<Route>
                                        to={Route::ArticleDetail { id: lead_article.id.clone() }}
                                        classes="text-[var(--text)] no-underline group-hover:underline decoration-[var(--primary)] underline-offset-4 decoration-1"
                                    >
                                        { &lead_article.title }
                                    </Link<Route>>
                                </h3>

                                if !lead_article.summary.trim().is_empty() {
                                    <p class="m-0 text-base leading-[1.65] text-[var(--muted)] line-clamp-3">
                                        { crate::utils::plain_excerpt(&lead_article.summary) }
                                    </p>
                                }

                                if !lead_article.tags.is_empty() {
                                    <div class="pt-1 flex flex-wrap gap-3">
                                        { for lead_article.tags.iter().take(4).map(|tag| html! {
                                            <Link<Route>
                                                key={tag.clone()}
                                                to={Route::TagDetail { tag: tag.clone() }}
                                                classes="font-mono text-xs text-[var(--faint)] hover:text-[var(--primary)] no-underline transition-colors"
                                            >
                                                { format!("#{tag}") }
                                            </Link<Route>>
                                        }) }
                                    </div>
                                }
                            </article>

                            // Right 3 Pure-Text Journal Entries
                            <div class="divide-y divide-[var(--border)] lg:border-l lg:border-[var(--border)] lg:pl-10">
                                { for recent_articles.iter().skip(1).take(3).enumerate().map(|(idx, item)| {
                                    let is_interactive = matches!(item.article_kind, ArticleKind::InteractiveRepost);
                                    html! {
                                        <article
                                            key={item.id.clone()}
                                            class={classes!(
                                                "group",
                                                "py-5",
                                                (idx == 0).then_some("first:pt-0")
                                            )}
                                        >
                                            <div class="flex items-center justify-between gap-2 flex-wrap">
                                                <div class="flex items-center gap-2">
                                                    if !item.category.trim().is_empty() {
                                                        <Link<Route>
                                                            to={Route::CategoryDetail { category: item.category.clone() }}
                                                            classes="text-[11px] font-semibold uppercase tracking-[0.08em] text-[var(--muted)] hover:text-[var(--primary)] no-underline transition-colors"
                                                        >
                                                            { &item.category }
                                                        </Link<Route>>
                                                        <span class="text-[var(--faint)] text-xs" aria-hidden="true">{ "·" }</span>
                                                    }
                                                    <time class="font-mono text-xs tabular-nums text-[var(--faint)]" datetime={item.date.clone()}>
                                                        { &item.date }
                                                    </time>
                                                </div>
                                                if is_interactive {
                                                    <span class="sf-chip !bg-[var(--primary-soft)] !text-[var(--primary)] !border-transparent font-mono !text-[10px] !h-5 !px-2">
                                                        { "◆ Interactive" }
                                                    </span>
                                                }
                                            </div>

                                            <h3 class="mt-2 mb-0 font-serif font-normal text-[1.225rem] leading-[1.3] tracking-[-0.01em]">
                                                <Link<Route>
                                                    to={Route::ArticleDetail { id: item.id.clone() }}
                                                    classes="text-[var(--text)] no-underline group-hover:underline decoration-[var(--primary)] underline-offset-4 decoration-1 line-clamp-2"
                                                >
                                                    { &item.title }
                                                </Link<Route>>
                                            </h3>

                                            if !item.summary.trim().is_empty() {
                                                <p class="mt-1.5 mb-0 text-sm leading-[1.6] text-[var(--muted)] line-clamp-2">
                                                    { crate::utils::plain_excerpt(&item.summary) }
                                                </p>
                                            }
                                        </article>
                                    }
                                }) }
                            </div>
                        </div>
                    }

                    if recent_articles.len() > 4 {
                        <div class="mt-14 pt-12 border-t border-[var(--border)] grid grid-cols-1 md:grid-cols-3 gap-8">
                            { for recent_articles.iter().skip(4).take(3).map(|article| html! {
                                <ArticleCard key={article.id.clone()} article={article.clone()} />
                            }) }
                        </div>
                    }
                }
            </section>

            // ④ Recently Ingested Sounds (Acoustic Track Rows — 64px hairline rows, no vinyl rotation)
            <section class="max-w-[1200px] mx-auto px-6 md:px-10 pb-20 md:pb-28">
                <div class="flex items-baseline justify-between gap-4 pb-6 border-b border-[var(--border)]">
                    <div>
                        <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                            { "ACOUSTIC ARCHIVE" }
                        </div>
                        <h2 class="mt-2 mb-0 font-serif font-normal text-[clamp(1.5rem,1.25rem+1vw,2rem)] leading-[1.2] tracking-[-0.015em] text-[var(--text)]">
                            { "最近入库的声音" }
                        </h2>
                    </div>
                    <Link<Route>
                        to={Route::MediaAudio}
                        classes="sf-btn sf-btn--ghost sf-btn--sm !px-0 no-underline shrink-0"
                    >
                        { "曲库与点歌 →" }
                    </Link<Route>>
                </div>

                if !*songs_loaded {
                    <div class="divide-y divide-[var(--border)]" aria-hidden="true">
                        { for (0..4).map(|idx| html! {
                            <div key={idx} class="h-16 grid grid-cols-[48px_1fr_auto] items-center gap-4">
                                <span class="skeleton w-12 h-12 rounded-[6px]" />
                                <div class="space-y-2">
                                    <span class="skeleton block h-4 w-48 rounded" />
                                    <span class="skeleton block h-3 w-28 rounded" />
                                </div>
                                <span class="skeleton w-8 h-8 rounded-full" />
                            </div>
                        }) }
                    </div>
                } else if recent_songs.is_empty() {
                    <p class="mt-8 font-serif italic text-base text-[var(--muted)]">
                        { "曲库暂无曲目——前往工坊许下第一首歌愿吧。" }
                    </p>
                } else {
                    <div class="divide-y divide-[var(--border)] border-b border-[var(--border)]">
                        { for recent_songs.iter().take(5).map(|song| {
                            let has_cover = song.cover_image.as_ref().is_some_and(|c| !c.is_empty());
                            let cover_url = api::song_cover_url(song.cover_image.as_deref());
                            let duration = format_duration_short(song.duration_ms);
                            let is_current = player_ctx
                                .as_ref()
                                .and_then(|c| c.song_id.as_deref())
                                == Some(song.id.as_str());
                            let is_playing = is_current && player_ctx.as_ref().is_some_and(|c| c.playing);
                            let is_loading_this = (*loading_play_id).as_deref() == Some(song.id.as_str());
                            let is_reader_wish = wish_song_ids.iter().any(|id| id == &song.id)
                                || song.tags.to_lowercase().contains("wish")
                                || song.tags.contains("点歌");

                            let on_play_click = {
                                let player_ctx = player_ctx.clone();
                                let loading_play_id = loading_play_id.clone();
                                let song_id = song.id.clone();
                                let playlist_ids: Vec<String> =
                                    recent_songs.iter().map(|s| s.id.clone()).collect();
                                Callback::from(move |e: MouseEvent| {
                                    e.prevent_default();
                                    e.stop_propagation();
                                    let Some(ctx) = player_ctx.clone() else {
                                        let href = crate::config::route_path(&format!("/media/audio/{song_id}"));
                                        let _ = crate::navigation_context::navigate_spa_to(&href);
                                        return;
                                    };
                                    if ctx.song_id.as_deref() == Some(song_id.as_str()) {
                                        ctx.dispatch(MusicAction::TogglePlay);
                                        return;
                                    }
                                    loading_play_id.set(Some(song_id.clone()));
                                    let loading_play_id = loading_play_id.clone();
                                    let song_id = song_id.clone();
                                    let playlist_ids = playlist_ids.clone();
                                    wasm_bindgen_futures::spawn_local(async move {
                                        ctx.dispatch(MusicAction::SetPlaylist {
                                            source: "home:recent-sounds".to_string(),
                                            ids: playlist_ids,
                                        });
                                        match api::fetch_song_detail(&song_id).await {
                                            Ok(Some(detail)) => {
                                                ctx.dispatch(MusicAction::PlaySong {
                                                    song: detail,
                                                    id: song_id.clone(),
                                                });
                                                let _ = api::track_song_play(&song_id).await;
                                            },
                                            _ => {
                                                let href = crate::config::route_path(&format!("/media/audio/{song_id}"));
                                                let _ = crate::navigation_context::navigate_spa_to(&href);
                                            },
                                        }
                                        loading_play_id.set(None);
                                    });
                                })
                            };

                            html! {
                                <div
                                    key={song.id.clone()}
                                    class={classes!(
                                        "group",
                                        "relative",
                                        "h-16",
                                        "grid",
                                        "grid-cols-[48px_1fr_auto_auto]",
                                        "items-center",
                                        "gap-4",
                                        "px-2",
                                        "transition-colors",
                                        "duration-150",
                                        "hover:bg-[var(--surface-alt)]/60"
                                    )}
                                >
                                    if is_current {
                                        <span
                                            class="absolute left-0 top-2.5 bottom-2.5 w-[2px] bg-[var(--primary)]"
                                            aria-hidden="true"
                                        />
                                    }

                                    <Link<Route>
                                        to={Route::MusicPlayer { id: song.id.clone() }}
                                        classes="relative w-12 h-12 rounded-[6px] overflow-hidden border border-[var(--border)] bg-[var(--surface-alt)] flex items-center justify-center shrink-0 no-underline"
                                    >
                                        if has_cover {
                                            <ImageWithLoading
                                                src={cover_url}
                                                alt={song.title.clone()}
                                                loading={Some(AttrValue::from("lazy"))}
                                                class={classes!("w-full", "h-full", "object-cover")}
                                                container_class={classes!("w-full", "h-full")}
                                            />
                                        } else {
                                            <span class="font-serif text-sm text-[var(--faint)]">{ "♪" }</span>
                                        }
                                        <span
                                            class="absolute inset-0 bg-[rgba(20,20,19,0.42)] text-[#FAF9F5] opacity-0 group-hover:opacity-100 transition-opacity duration-150 flex items-center justify-center text-xs"
                                            aria-hidden="true"
                                        >
                                            { "▶" }
                                        </span>
                                    </Link<Route>>

                                    <div class="min-w-0">
                                        <div class="flex items-center gap-2 min-w-0">
                                            if is_playing {
                                                <span
                                                    class="font-mono text-[10px] tracking-tighter text-[var(--primary)] animate-pulse motion-reduce:animate-none shrink-0"
                                                    aria-label="正在播放"
                                                >
                                                    { "▍▊▌" }
                                                </span>
                                            }
                                            <Link<Route>
                                                to={Route::MusicPlayer { id: song.id.clone() }}
                                                classes="font-serif font-normal text-[0.98rem] text-[var(--text)] hover:text-[var(--primary)] truncate no-underline transition-colors"
                                            >
                                                { &song.title }
                                            </Link<Route>>
                                            if is_reader_wish {
                                                <span class="sf-chip !h-5 !px-2 !text-[10px] !bg-[var(--primary-soft)] !text-[var(--primary)] shrink-0">
                                                    { "读者点歌" }
                                                </span>
                                            }
                                        </div>
                                        <div class="text-xs text-[var(--muted)] truncate mt-0.5">
                                            { &song.artist }
                                        </div>
                                    </div>

                                    <span class="font-mono text-xs tabular-nums text-[var(--faint)]">
                                        { duration }
                                    </span>

                                    <button
                                        type="button"
                                        onclick={on_play_click}
                                        aria-label={if is_playing { format!("暂停 {}", song.title) } else { format!("播放 {}", song.title) }}
                                        class={classes!(
                                            "w-8",
                                            "h-8",
                                            "rounded-full",
                                            "border",
                                            "inline-flex",
                                            "items-center",
                                            "justify-center",
                                            "cursor-pointer",
                                            "transition-colors",
                                            "duration-150",
                                            if is_playing {
                                                "border-[var(--primary)] bg-[var(--primary)] text-[var(--on-primary)]"
                                            } else {
                                                "border-[var(--border-strong)] bg-transparent text-[var(--text)] hover:border-[var(--primary)] hover:bg-[var(--primary)] hover:text-[var(--on-primary)]"
                                            }
                                        )}
                                    >
                                        if is_loading_this {
                                            <span class="w-3 h-3 rounded-full border border-current border-t-transparent animate-spin motion-reduce:animate-none" aria-hidden="true" />
                                        } else if is_playing {
                                            <i class="fas fa-pause text-[10px]" aria-hidden="true"></i>
                                        } else {
                                            <i class="fas fa-play text-[10px] ml-0.5" aria-hidden="true"></i>
                                        }
                                    </button>
                                </div>
                            }
                        }) }
                    </div>
                }
            </section>

            // ⑤ Open Source & Engineering Architecture Strip
            <section class="w-full border-t border-[var(--border)] pt-16">
                <div class="max-w-[1200px] mx-auto px-6 md:px-10 flex flex-col lg:flex-row lg:items-center lg:justify-between gap-8">
                    <div class="max-w-[56ch]">
                        <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                            { "OPEN SOURCE · FULL-STACK RUST" }
                        </div>
                        <h3 class="mt-2.5 mb-0 font-serif font-normal text-2xl sm:text-[1.65rem] leading-[1.25] tracking-[-0.015em] text-[var(--text)]">
                            { "这个网站本身是开源的。" }
                        </h3>
                        <p class="mt-2.5 mb-0 text-sm sm:text-[0.95rem] leading-[1.65] text-[var(--muted)]">
                            { "从 Axum 后端、LanceDB 多模态向量存储到 Yew 原生 WebAssembly 前端，全站皆由本地优先的 Rust 工程构建。" }
                        </p>
                    </div>

                    <div class="flex flex-col lg:items-end gap-4">
                        <div class="flex flex-wrap items-center gap-2.5">
                            <a
                                href="https://github.com/acking-you/static_flow"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="sf-btn sf-btn--secondary no-underline font-mono text-xs"
                            >
                                <i class="fa-brands fa-github text-sm" aria-hidden="true"></i>
                                { "GitHub · acking-you/static_flow" }
                            </a>

                            <GithubWrappedSelector />

                        </div>

                        <div class="flex flex-wrap items-center gap-x-4 gap-y-1.5 text-xs">
                            <Link<Route>
                                to={Route::LlmAccess}
                                classes="sf-link font-mono text-xs text-[var(--muted)] hover:text-[var(--primary)] no-underline hover:underline"
                            >
                                { "LLM Gateway →" }
                            </Link<Route>>
                            <span class="text-[var(--border-strong)]" aria-hidden="true">{ "·" }</span>
                            <Link<Route>
                                to={Route::KiroAccess}
                                classes="sf-link font-mono text-xs text-[var(--muted)] hover:text-[var(--primary)] no-underline hover:underline"
                            >
                                { "Kiro Gateway →" }
                            </Link<Route>>
                            <span class="text-[var(--border-strong)] hidden sm:inline" aria-hidden="true">{ "│" }</span>
                            <span class="font-mono text-[0.75rem] text-[var(--faint)]">
                                { "Rust · Yew 0.23 · Axum · LanceDB · WebAssembly" }
                            </span>
                        </div>
                    </div>
                </div>
            </section>
        </main>
    }
}

/// GitHub Wrapped year entry.
#[derive(Clone)]
struct WrappedYear {
    year: u16,
    is_latest: bool,
}

impl WrappedYear {
    fn url(&self) -> String {
        crate::config::asset_path(&format!("standalone/github-wrapped-{}.html", self.year))
    }
}

fn get_wrapped_years() -> Vec<WrappedYear> {
    vec![
        WrappedYear {
            year: 2025,
            is_latest: true,
        },
        WrappedYear {
            year: 2024,
            is_latest: false,
        },
    ]
}

#[function_component(GithubWrappedSelector)]
fn github_wrapped_selector() -> Html {
    let expanded = use_state(|| false);
    let years = get_wrapped_years();
    let Some(latest) = years.first().cloned() else {
        return Html::default();
    };

    let toggle_expand = {
        let expanded = expanded.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            expanded.set(!*expanded);
        })
    };

    let close_dropdown = {
        let expanded = expanded.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            expanded.set(false);
        })
    };

    {
        let expanded = expanded.clone();
        use_effect_with(*expanded, move |is_expanded| {
            let cleanup: Box<dyn FnOnce()> = if *is_expanded {
                let expanded = expanded.clone();
                let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
                    move |_: web_sys::Event| {
                        expanded.set(false);
                    },
                );

                if let Some(window) = web_sys::window() {
                    let _ = window.add_event_listener_with_callback(
                        "click",
                        closure.as_ref().unchecked_ref(),
                    );
                    let window_clone = window.clone();
                    Box::new(move || {
                        let _ = window_clone.remove_event_listener_with_callback(
                            "click",
                            closure.as_ref().unchecked_ref(),
                        );
                    })
                } else {
                    Box::new(|| {})
                }
            } else {
                Box::new(|| {})
            };
            cleanup
        });
    }

    let has_multiple_years = years.len() > 1;

    html! {
        <div class="relative inline-flex items-center">
            <div class="inline-flex items-center rounded-full border border-[var(--border-strong)] bg-[var(--surface)] overflow-hidden">
                <a
                    href={latest.url()}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="inline-flex items-center gap-2 h-10 px-4 text-xs font-mono text-[var(--text)] hover:bg-[var(--surface-alt)] no-underline transition-colors"
                >
                    <span class="text-[var(--primary)]">{ "◆" }</span>
                    <span>{ format!("{} Wrapped", latest.year) }</span>
                </a>

                if has_multiple_years {
                    <button
                        type="button"
                        class="inline-flex items-center justify-center h-10 w-8 border-0 border-l border-[var(--border)] bg-transparent text-[var(--muted)] hover:bg-[var(--surface-alt)] hover:text-[var(--text)] cursor-pointer transition-colors"
                        onclick={toggle_expand}
                        aria-label={t::WRAPPED_MORE_YEARS_ARIA}
                        aria-expanded={(*expanded).to_string()}
                    >
                        <i
                            class={classes!(
                                "fas",
                                "fa-chevron-down",
                                "text-[10px]",
                                "transition-transform",
                                "duration-150",
                                (*expanded).then_some("rotate-180")
                            )}
                            aria-hidden="true"
                        ></i>
                    </button>
                }
            </div>

            if has_multiple_years && *expanded {
                <div
                    class="absolute right-0 bottom-full mb-2 z-50 min-w-[180px] rounded-[10px] border border-[var(--border-strong)] bg-[var(--surface)] p-1.5 shadow-[var(--shadow-8)]"
                    onclick={close_dropdown}
                >
                    <div class="px-2.5 py-1.5 font-mono text-[10px] uppercase tracking-[0.08em] text-[var(--faint)]">
                        { t::WRAPPED_SELECT_YEAR }
                    </div>
                    { for years.iter().map(|y| html! {
                        <a
                            key={y.year}
                            href={y.url()}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="flex items-center justify-between gap-3 rounded-[6px] px-2.5 py-2 text-xs font-mono text-[var(--text)] hover:bg-[var(--surface-alt)] no-underline transition-colors"
                        >
                            <span>{ format!("{} Wrapped", y.year) }</span>
                            if y.is_latest {
                                <span class="sf-chip !h-5 !px-2 !text-[10px] !bg-[var(--primary-soft)] !text-[var(--primary)]">
                                    { t::WRAPPED_LATEST_TAG }
                                </span>
                            }
                        </a>
                    }) }
                </div>
            }
        </div>
    }
}
