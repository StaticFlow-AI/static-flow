use wasm_bindgen::JsCast;
use web_sys::console;
use yew::prelude::*;
use yew_router::prelude::{use_location, Link};

use crate::{
    api::{self, SongListItem},
    components::{article_card::ArticleCard, image_with_loading::ImageWithLoading},
    i18n::current::home as t,
    models::{ArticleKind, ArticleListItem},
    music_context::{MusicAction, MusicPlayerContext},
    pages::{home_hero::PersonalHero, worldlines::WorldlinesExhibit},
    router::Route,
    utils::image_url,
};

fn format_duration_short(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

fn song_detail_to_list_item(detail: &api::SongDetail) -> SongListItem {
    SongListItem {
        id: detail.id.clone(),
        title: detail.title.clone(),
        artist: detail.artist.clone(),
        album: detail.album.clone(),
        cover_image: detail.cover_image.clone(),
        duration_ms: detail.duration_ms,
        format: detail.format.clone(),
        tags: detail.tags.clone(),
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
    let wish_song_ids = use_state(Vec::<String>::new);

    // Recent content
    let recent_articles = use_state(Vec::<ArticleListItem>::new);
    let articles_loaded = use_state(|| false);
    let recent_songs = use_state(Vec::<SongListItem>::new);
    let songs_loaded = use_state(|| false);
    let loading_play_id = use_state(|| None::<String>);

    // Browser fragment navigation can run before WASM mounts the home sections.
    {
        let loc_dep = location.clone();
        use_effect_with(loc_dep, move |_| {
            let follow_home_fragment = || {
                let hash = web_sys::window()
                    .and_then(|w| w.location().hash().ok())
                    .unwrap_or_default();
                if hash == "#workshop-music" {
                    let target = crate::config::route_path("/studio#music");
                    let _ = crate::navigation_context::navigate_spa_to(&target);
                } else if hash == "#workshop" {
                    let target = crate::config::route_path("/studio#article");
                    let _ = crate::navigation_context::navigate_spa_to(&target);
                } else if let Some(section) = hash
                    .strip_prefix('#')
                    .and_then(|id| gloo_utils::document().get_element_by_id(id))
                {
                    section.scroll_into_view();
                }
            };

            follow_home_fragment();

            let on_hashchange =
                wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                    follow_home_fragment();
                });

            if let Some(win) = web_sys::window() {
                let _ = win.add_event_listener_with_callback(
                    "hashchange",
                    on_hashchange.as_ref().unchecked_ref(),
                );
            }

            move || {
                if let Some(win) = web_sys::window() {
                    let _ = win.remove_event_listener_with_callback(
                        "hashchange",
                        on_hashchange.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    // Initial site stats, articles (7), songs (5), and reader-ingestion totals
    {
        let total_articles = total_articles.clone();
        let total_music = total_music.clone();
        let ar_total = ar_total.clone();
        let mw_total = mw_total.clone();
        let wish_song_ids = wish_song_ids.clone();
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
            {
                let ar_total = ar_total.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(resp) = api::fetch_article_requests(Some(1), Some(0)).await {
                        ar_total.set(Some(resp.total));
                    }
                });
            }
            {
                let mw_total = mw_total.clone();
                let wish_song_ids = wish_song_ids.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(resp) = api::fetch_music_wishes(Some(8), Some(0)).await {
                        mw_total.set(Some(resp.total));
                        let ids: Vec<String> = resp
                            .wishes
                            .iter()
                            .filter_map(|w| w.ingested_song_id.as_deref())
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string)
                            .collect();
                        wish_song_ids.set(ids);
                    }
                });
            }
            || ()
        });
    }

    let reader_total: Option<usize> = match (*ar_total, *mw_total) {
        (Some(a), Some(b)) => Some(a + b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    // Featured song in PersonalHero reflects active player song if present, else
    // first recommended song
    let featured_song: Option<SongListItem> = player_ctx
        .as_ref()
        .and_then(|c| c.current_song.as_ref().map(song_detail_to_list_item))
        .or_else(|| recent_songs.first().cloned());

    let featured_playing = featured_song.as_ref().is_some_and(|song| {
        player_ctx.as_ref().and_then(|c| c.song_id.as_deref()) == Some(song.id.as_str())
            && player_ctx.as_ref().is_some_and(|c| c.playing)
    });

    let featured_loading = featured_song
        .as_ref()
        .is_some_and(|song| (*loading_play_id).as_deref() == Some(song.id.as_str()));

    let on_play_featured = {
        let player_ctx = player_ctx.clone();
        let loading_play_id = loading_play_id.clone();
        let featured_song = featured_song.clone();
        let playlist_ids: Vec<String> = recent_songs.iter().map(|s| s.id.clone()).collect();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            let Some(song) = featured_song.clone() else {
                return;
            };
            let song_id = song.id;
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
            let playlist_ids = playlist_ids.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if !playlist_ids.is_empty() {
                    ctx.dispatch(MusicAction::SetPlaylist {
                        source: "home:hero-jukebox".to_string(),
                        ids: playlist_ids,
                    });
                }
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

    let studio_article_href = crate::config::route_path("/studio#article");
    let go_studio_article = {
        let href = studio_article_href.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&href);
        })
    };

    let studio_music_href = crate::config::route_path("/studio#music");
    let go_studio_music = {
        let href = studio_music_href.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&href);
        })
    };

    let studio_href = crate::config::route_path("/studio");
    let go_studio = {
        let href = studio_href.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&href);
        })
    };

    html! {
        <main class="w-full min-h-screen bg-[var(--bg)] text-[var(--text)] overflow-x-clip">
            // ① Personal Split-Screen Hero (Knowledge Base + Jukebox + World-Line & sf-cli)
            <PersonalHero
                total_articles={*total_articles}
                total_music={*total_music}
                reader_total={reader_total}
                latest={(*recent_articles).clone()}
                featured_song={featured_song}
                featured_playing={featured_playing}
                featured_loading={featured_loading}
                on_play_featured={on_play_featured}
            />

            <WorldlinesExhibit />

            // ② Selected & Recent Writing (Anthropic Research / Editorial Magazine Layout)
            <section class="max-w-[1200px] mx-auto px-5 sm:px-6 md:px-10 pt-16 md:pt-24 pb-16 md:pb-24">
                <div class="flex flex-wrap items-baseline justify-between gap-4 pb-6 border-b border-[var(--border)]">
                    <div>
                        <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                            { "SELECTED WRITING · 知识库近作" }
                        </div>
                        <h2 class="mt-2 mb-0 font-serif font-normal text-[clamp(1.5rem,1.25rem+1vw,2rem)] leading-[1.2] tracking-[-0.015em] text-[var(--text)]">
                            { "精选与最新文章" }
                        </h2>
                    </div>

                    <div class="flex flex-wrap items-center gap-x-5 gap-y-2 text-sm">
                        <Link<Route>
                            to={Route::LatestArticles}
                            classes="text-[var(--muted)] hover:text-[var(--primary)] no-underline transition-colors"
                        >
                            { "最新时间线 →" }
                        </Link<Route>>
                        <Link<Route>
                            to={Route::Posts}
                            classes="font-medium text-[var(--text)] hover:text-[var(--primary)] no-underline transition-colors"
                        >
                            { "浏览全部文章 →" }
                        </Link<Route>>
                    </div>
                </div>

                if !*articles_loaded {
                    <div class="mt-10 grid grid-cols-1 lg:grid-cols-[1.25fr_1fr] gap-10" aria-hidden="true">
                        <div class="space-y-4">
                            <span class="skeleton block w-full aspect-[16/10] rounded-[10px]" />
                            <span class="skeleton block h-4 w-32 rounded" />
                            <span class="skeleton block h-8 w-5/6 rounded" />
                            <span class="skeleton block h-4 w-full rounded" />
                            <span class="skeleton block h-4 w-3/4 rounded" />
                        </div>
                        <div class="space-y-6 lg:border-l lg:border-[var(--border)] lg:pl-10">
                            { for (0..3).map(|idx| html! {
                                <div key={idx} class="space-y-2.5 pb-5 border-b border-[var(--border)]">
                                    <span class="skeleton block h-3.5 w-24 rounded" />
                                    <span class="skeleton block h-6 w-11/12 rounded" />
                                    <span class="skeleton block h-4 w-4/5 rounded" />
                                </div>
                            }) }
                        </div>
                    </div>
                } else if recent_articles.is_empty() {
                    <p class="mt-10 font-serif text-base text-[var(--muted)]">
                        { "知识库正在整理中——你也可以在 " }
                        <a href={studio_article_href.clone()} onclick={go_studio_article.clone()} class="text-[var(--primary)] underline underline-offset-4">
                            { "共建台" }
                        </a>
                        { " 推荐第一篇想读的好文。" }
                    </p>
                } else {
                    if let Some(lead_article) = recent_articles.first() {
                        <div class="mt-8 sm:mt-10 grid grid-cols-1 lg:grid-cols-[1.25fr_1fr] gap-8 lg:gap-10 items-start">
                            // Lead Essay
                            <article class="group space-y-4 min-w-0">
                                if let Some(ref img) = lead_article.featured_image {
                                    <Link<Route>
                                        to={Route::ArticleDetail { id: lead_article.id.clone() }}
                                        classes="block rounded-[10px] overflow-hidden border border-[var(--border)] bg-[var(--surface-alt)] aspect-[16/10] no-underline"
                                    >
                                        <ImageWithLoading
                                            src={image_url(img)}
                                            alt={lead_article.title.clone()}
                                            loading={Some(AttrValue::from("lazy"))}
                                            class={classes!(
                                                "w-full",
                                                "h-full",
                                                "object-cover",
                                                "transition-transform",
                                                "duration-300",
                                                "ease-out",
                                                "group-hover:scale-[1.015]",
                                                "motion-reduce:transform-none"
                                            )}
                                            container_class={classes!("w-full", "h-full")}
                                        />
                                    </Link<Route>>
                                }

                                <div class="flex items-center justify-between gap-2 flex-wrap pt-1">
                                    <div class="flex items-center gap-2 min-w-0">
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

                                <h3 class="m-0 font-serif font-normal text-[clamp(1.4rem,1.2rem+1vw,2.05rem)] leading-[1.22] tracking-[-0.015em] break-words">
                                    <Link<Route>
                                        to={Route::ArticleDetail { id: lead_article.id.clone() }}
                                        classes="text-[var(--text)] no-underline group-hover:underline decoration-[var(--primary)] underline-offset-4 decoration-1"
                                    >
                                        { &lead_article.title }
                                    </Link<Route>>
                                </h3>

                                if !lead_article.summary.trim().is_empty() {
                                    <p class="m-0 text-[0.95rem] sm:text-base leading-[1.68] text-[var(--muted)] line-clamp-3 break-words">
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
                            <div class="divide-y divide-[var(--border)] border-t border-[var(--border)] pt-2 lg:border-t-0 lg:pt-0 lg:border-l lg:pl-10 min-w-0">
                                { for recent_articles.iter().skip(1).take(3).enumerate().map(|(idx, item)| {
                                    let is_interactive = matches!(item.article_kind, ArticleKind::InteractiveRepost);
                                    html! {
                                        <article
                                            key={item.id.clone()}
                                            class={classes!(
                                                "group",
                                                "py-5",
                                                "min-w-0",
                                                (idx == 0).then_some("lg:pt-0")
                                            )}
                                        >
                                            <div class="flex items-center justify-between gap-2 flex-wrap">
                                                <div class="flex items-center gap-2 min-w-0">
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

                                            <h3 class="mt-2 mb-0 font-serif font-normal text-[1.18rem] sm:text-[1.225rem] leading-[1.32] tracking-[-0.01em] break-words">
                                                <Link<Route>
                                                    to={Route::ArticleDetail { id: item.id.clone() }}
                                                    classes="text-[var(--text)] no-underline group-hover:underline decoration-[var(--primary)] underline-offset-4 decoration-1 line-clamp-2"
                                                >
                                                    { &item.title }
                                                </Link<Route>>
                                            </h3>

                                            if !item.summary.trim().is_empty() {
                                                <p class="mt-1.5 mb-0 text-sm leading-[1.6] text-[var(--muted)] line-clamp-2 break-words">
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
                        <div class="mt-12 sm:mt-14 pt-10 sm:pt-12 border-t border-[var(--border)] grid grid-cols-1 md:grid-cols-3 gap-6 sm:gap-8">
                            { for recent_articles.iter().skip(4).take(3).map(|article| html! {
                                <ArticleCard key={article.id.clone()} article={article.clone()} />
                            }) }
                        </div>
                    }

                    <div class="mt-8 flex flex-wrap items-center justify-between gap-3 pt-5 border-t border-[var(--border)] text-xs text-[var(--muted)]">
                        <span>{ "读到值得深挖的外部长文，也可以丢进共建台让 AI 调研精译入库。" }</span>
                        <a
                            href={studio_article_href}
                            onclick={go_studio_article}
                            class="font-medium text-[var(--primary)] hover:underline underline-offset-4 no-underline shrink-0"
                        >
                            { "推荐文章入库 →" }
                        </a>
                    </div>
                }
            </section>

            // ③ Curated Jukebox (Pure Listening Experience — Responsive Hairline Track Rows)
            <section class="max-w-[1200px] mx-auto px-5 sm:px-6 md:px-10 pb-20 md:pb-28">
                <div class="flex flex-wrap items-baseline justify-between gap-4 pb-6 border-b border-[var(--border)]">
                    <div>
                        <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                            { "ACOUSTIC ARCHIVE · 私人曲库" }
                        </div>
                        <h2 class="mt-2 mb-0 font-serif font-normal text-[clamp(1.5rem,1.25rem+1vw,2rem)] leading-[1.2] tracking-[-0.015em] text-[var(--text)]">
                            { "写代码时循环的歌" }
                        </h2>
                    </div>

                    <div class="flex flex-wrap items-center gap-x-5 gap-y-2 text-sm">
                        <a
                            href={studio_music_href.clone()}
                            onclick={go_studio_music.clone()}
                            class="text-[var(--muted)] hover:text-[var(--primary)] no-underline transition-colors"
                        >
                            { "点一首歌 →" }
                        </a>
                        <Link<Route>
                            to={Route::MediaAudio}
                            classes="font-medium text-[var(--text)] hover:text-[var(--primary)] no-underline transition-colors"
                        >
                            { "进入完整曲库 →" }
                        </Link<Route>>
                    </div>
                </div>

                if !*songs_loaded {
                    <div class="divide-y divide-[var(--border)]" aria-hidden="true">
                        { for (0..4).map(|idx| html! {
                            <div key={idx} class="h-16 grid grid-cols-[44px_minmax(0,1fr)_auto] items-center gap-3.5">
                                <span class="skeleton w-11 h-11 rounded-[6px]" />
                                <div class="space-y-2 min-w-0">
                                    <span class="skeleton block h-4 w-44 max-w-full rounded" />
                                    <span class="skeleton block h-3 w-28 max-w-full rounded" />
                                </div>
                                <span class="skeleton w-9 h-9 rounded-full" />
                            </div>
                        }) }
                    </div>
                } else if recent_songs.is_empty() {
                    <p class="mt-8 font-serif text-base text-[var(--muted)]">
                        { "曲库暂无曲目——前往 " }
                        <a href={studio_music_href} onclick={go_studio_music} class="text-[var(--primary)] underline underline-offset-4">
                            { "共建台" }
                        </a>
                        { " 点下第一首歌吧。" }
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
                                    class="group relative py-3 min-h-[64px] grid grid-cols-[44px_minmax(0,1fr)_auto] sm:grid-cols-[48px_minmax(0,1fr)_auto_auto] items-center gap-3 sm:gap-4 px-1.5 sm:px-2 transition-colors duration-150 hover:bg-[var(--surface-alt)]/60"
                                >
                                    if is_current {
                                        <span
                                            class="absolute left-0 top-2.5 bottom-2.5 w-[2px] bg-[var(--primary)]"
                                            aria-hidden="true"
                                        />
                                    }

                                    <Link<Route>
                                        to={Route::MusicPlayer { id: song.id.clone() }}
                                        classes="relative w-11 h-11 sm:w-12 sm:h-12 rounded-[6px] overflow-hidden border border-[var(--border)] bg-[var(--surface-alt)] flex items-center justify-center shrink-0 no-underline"
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
                                                <span class="sf-eq shrink-0" aria-label="正在播放">
                                                    <i /><i /><i /><i />
                                                </span>
                                            }
                                            <Link<Route>
                                                to={Route::MusicPlayer { id: song.id.clone() }}
                                                classes="font-serif font-normal text-[0.96rem] sm:text-[0.98rem] text-[var(--text)] hover:text-[var(--primary)] truncate no-underline transition-colors"
                                            >
                                                { &song.title }
                                            </Link<Route>>
                                            if is_reader_wish {
                                                <span class="hidden sm:inline-flex sf-chip !h-5 !px-2 !text-[10px] !bg-[var(--primary-soft)] !text-[var(--primary)] shrink-0">
                                                    { "读者点歌" }
                                                </span>
                                            }
                                        </div>
                                        <div class="text-xs text-[var(--muted)] truncate mt-0.5">
                                            { &song.artist }
                                            if !song.album.trim().is_empty() {
                                                <span class="hidden md:inline text-[var(--faint)]">{ format!(" · {}", song.album) }</span>
                                            }
                                        </div>
                                    </div>

                                    <span class="hidden sm:inline font-mono text-xs tabular-nums text-[var(--faint)]">
                                        { duration }
                                    </span>

                                    <button
                                        type="button"
                                        onclick={on_play_click}
                                        aria-label={if is_playing { format!("暂停 {}", song.title) } else { format!("播放 {}", song.title) }}
                                        class={classes!(
                                            "w-9",
                                            "h-9",
                                            "rounded-full",
                                            "border",
                                            "inline-flex",
                                            "items-center",
                                            "justify-center",
                                            "cursor-pointer",
                                            "shrink-0",
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
                                            <span class="w-3.5 h-3.5 rounded-full border border-current border-t-transparent animate-spin motion-reduce:animate-none" aria-hidden="true" />
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

            // ④ Open Source & Personal Craft Strip
            <section class="w-full border-t border-[var(--border)] py-14 md:py-16">
                <div class="max-w-[1200px] mx-auto px-5 sm:px-6 md:px-10 flex flex-col lg:flex-row lg:items-center lg:justify-between gap-8">
                    <div class="max-w-[56ch]">
                        <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                            { "OPEN SOURCE · FULL-STACK RUST & WASM" }
                        </div>
                        <h3 class="mt-2.5 mb-0 font-serif font-normal text-2xl sm:text-[1.65rem] leading-[1.25] tracking-[-0.015em] text-[var(--text)]">
                            { "全栈原生 Rust 与 WebAssembly 构建。" }
                        </h3>
                        <p class="mt-2.5 mb-0 text-sm sm:text-[0.95rem] leading-[1.65] text-[var(--muted)]">
                            { "从 Axum 异步后端、LanceDB 本地多模态向量检索，到 Yew 0.23 原生 WASM 前端与 AI 自助入库工作流，代码完整开源。" }
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
                            <a
                                href={studio_href}
                                onclick={go_studio}
                                class="sf-link font-mono text-xs text-[var(--muted)] hover:text-[var(--primary)] no-underline hover:underline"
                            >
                                { "共建台 Studio →" }
                            </a>
                            <span class="text-[var(--border-strong)]" aria-hidden="true">{ "·" }</span>
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
