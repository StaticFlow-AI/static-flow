use gloo_events::EventListener;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    components::{
        icons::{Icon, IconName},
        image_with_loading::ImageWithLoading,
        pagination::Pagination,
        toast::use_toast,
    },
    i18n::current::studio_page as st_t,
    music_context::{MusicAction, MusicPlayerContext},
    router::Route,
};

const PAGE_SIZE: usize = 20;
const RANDOM_RECOMMEND_LIMIT: usize = 10;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
struct MusicLibraryQuery {
    artist: Option<String>,
    album: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryDisplayMode {
    RandomRecommended,
    AllSongs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackLayoutMode {
    AcousticList,
    MinimalGrid,
}

#[function_component(MusicLibraryPage)]
pub fn music_library_page() -> Html {
    let location = use_location();
    let query_string = location
        .as_ref()
        .map(|l| l.query_str().to_string())
        .unwrap_or_default();

    // Initialize filter state directly from URL to avoid double-fetch on mount
    let initial_query = location
        .as_ref()
        .and_then(|loc| loc.query::<MusicLibraryQuery>().ok())
        .unwrap_or(MusicLibraryQuery {
            artist: None,
            album: None,
        });
    let initial_has_filter = initial_query.artist.is_some() || initial_query.album.is_some();

    let page_songs = use_state(Vec::<api::SongListItem>::new);
    let loading = use_state(|| initial_has_filter);
    let error = use_state(|| None::<String>);
    let random_songs = use_state(Vec::<api::SongListItem>::new);
    let random_loading = use_state(|| !initial_has_filter);
    let random_error = use_state(|| None::<String>);
    let random_refresh_tick = use_state(|| 0_u64);
    let display_mode = use_state(|| {
        if initial_has_filter {
            LibraryDisplayMode::AllSongs
        } else {
            LibraryDisplayMode::RandomRecommended
        }
    });
    let layout_mode = use_state(|| TrackLayoutMode::AcousticList);
    let active_artist = use_state(|| initial_query.artist.clone());
    let active_album = use_state(|| initial_query.album.clone());
    let current_page = use_state(|| 1_usize);
    let total = use_state(|| 0_usize);
    let catalog_total = use_state(|| 0_usize);
    let all_songs_request_seq = use_mut_ref(|| 0_u64);
    let random_request_seq = use_mut_ref(|| 0_u64);
    let inline_play_loading_id = use_state(|| None::<String>);
    let player_ctx = use_context::<MusicPlayerContext>();
    let toast = use_toast();

    // Search state
    let hero_query = use_state(String::new);

    // Fetch catalog total count once on mount so the editorial header always shows
    // accurate archive count even before switching to AllSongs.
    {
        let catalog_total = catalog_total.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = api::fetch_songs(Some(1), Some(0), None, None, None).await {
                    catalog_total.set(resp.total);
                }
            });
            || ()
        });
    }

    // Redirect legacy #music-wish-section / #music-wish-form bookmarks to
    // /studio#music
    {
        let query_dep = query_string.clone();
        use_effect_with(query_dep, move |_| {
            let check_legacy_hash = || {
                let hash = web_sys::window()
                    .and_then(|w| w.location().hash().ok())
                    .unwrap_or_default();
                if hash == "#music-wish-section" || hash == "#music-wish-form" {
                    let studio_url = crate::config::route_path("/studio#music");
                    let _ = crate::navigation_context::navigate_spa_to(&studio_url);
                }
            };
            check_legacy_hash();

            let mut listener_opt: Option<EventListener> = None;
            if let Some(win) = web_sys::window() {
                listener_opt = Some(EventListener::new(&win, "hashchange", move |_| {
                    check_legacy_hash();
                }));
            }
            move || {
                drop(listener_opt);
            }
        });
    }

    let on_hero_input = {
        let hero_query = hero_query.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                hero_query.set(input.value());
            }
        })
    };

    let do_hero_search = {
        let hero_query = hero_query.clone();
        move || {
            let q = hero_query.trim().to_string();
            if q.is_empty() {
                return;
            }
            let encoded = urlencoding::encode(&q);
            let url = crate::config::route_path(&format!("/search?q={encoded}&mode=music"));
            crate::navigation_context::navigate_spa_to(&url);
        }
    };

    let on_hero_search = {
        let do_search = do_hero_search.clone();
        Callback::from(move |_: MouseEvent| do_search())
    };

    let on_hero_keypress = {
        let do_search = do_hero_search;
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                do_search();
            }
        })
    };

    // Sync URL query params → state on subsequent navigation
    {
        let active_artist = active_artist.clone();
        let active_album = active_album.clone();
        let current_page = current_page.clone();
        let display_mode = display_mode.clone();
        let location = location.clone();
        use_effect_with(query_string, move |_| {
            if let Some(ref loc) = location {
                if let Ok(q) = loc.query::<MusicLibraryQuery>() {
                    let has_filter = q.artist.is_some() || q.album.is_some();
                    active_artist.set(q.artist);
                    active_album.set(q.album);
                    if has_filter {
                        display_mode.set(LibraryDisplayMode::AllSongs);
                    }
                    current_page.set(1);
                }
            }
            || ()
        });
    }

    // Fetch one page of songs when filter or page changes
    {
        let page_songs = page_songs.clone();
        let loading = loading.clone();
        let error = error.clone();
        let total = total.clone();
        let catalog_total = catalog_total.clone();
        let all_songs_request_seq = all_songs_request_seq.clone();
        let deps =
            (*display_mode, (*active_artist).clone(), (*active_album).clone(), *current_page);
        use_effect_with(deps, move |deps| {
            let request_id = {
                let mut seq = all_songs_request_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            let (mode, artist, album, page) = deps.clone();
            if mode != LibraryDisplayMode::AllSongs {
                loading.set(false);
                error.set(None);
            } else {
                let offset = (page - 1) * PAGE_SIZE;
                let unfiltered = artist.is_none() && album.is_none();
                loading.set(true);
                error.set(None);
                let all_songs_request_seq = all_songs_request_seq.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_songs(
                        Some(PAGE_SIZE),
                        Some(offset),
                        artist.as_deref(),
                        album.as_deref(),
                        None,
                    )
                    .await
                    {
                        Ok(resp) => {
                            if *all_songs_request_seq.borrow() != request_id {
                                return;
                            }
                            total.set(resp.total);
                            if unfiltered {
                                catalog_total.set(resp.total);
                            }
                            page_songs.set(resp.songs);
                        },
                        Err(e) => {
                            if *all_songs_request_seq.borrow() != request_id {
                                return;
                            }
                            error.set(Some(e));
                        },
                    }
                    if *all_songs_request_seq.borrow() != request_id {
                        return;
                    }
                    loading.set(false);
                });
            }
            || ()
        });
    }

    // Fetch default random recommendations.
    {
        let display_mode = *display_mode;
        let random_songs = random_songs.clone();
        let random_loading = random_loading.clone();
        let random_error = random_error.clone();
        let random_request_seq = random_request_seq.clone();
        let refresh_tick = *random_refresh_tick;
        use_effect_with((display_mode, refresh_tick), move |(mode, _)| {
            let request_id = {
                let mut seq = random_request_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            if *mode != LibraryDisplayMode::RandomRecommended {
                random_loading.set(false);
                random_error.set(None);
            } else {
                random_loading.set(true);
                random_error.set(None);
                let random_request_seq = random_request_seq.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_random_recommended_songs(Some(RANDOM_RECOMMEND_LIMIT), &[])
                        .await
                    {
                        Ok(songs) => {
                            if *random_request_seq.borrow() != request_id {
                                return;
                            }
                            random_songs.set(songs);
                        },
                        Err(err) => {
                            if *random_request_seq.borrow() != request_id {
                                return;
                            }
                            random_error.set(Some(err));
                        },
                    }
                    if *random_request_seq.borrow() != request_id {
                        return;
                    }
                    random_loading.set(false);
                });
            }
            || ()
        });
    }

    let total_val = *total;
    let header_song_count = if *catalog_total > 0 {
        *catalog_total
    } else if total_val > 0 {
        total_val
    } else {
        random_songs.len()
    };
    let total_pages = if total_val == 0 { 1 } else { total_val.div_ceil(PAGE_SIZE) };

    let on_artist_click = {
        let active_artist = active_artist.clone();
        let current_page = current_page.clone();
        let display_mode = display_mode.clone();
        move |artist: String| {
            let active_artist = active_artist.clone();
            let current_page = current_page.clone();
            let display_mode = display_mode.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                active_artist.set(Some(artist.clone()));
                current_page.set(1);
                display_mode.set(LibraryDisplayMode::AllSongs);
            })
        }
    };

    let on_album_click = {
        let active_album = active_album.clone();
        let current_page = current_page.clone();
        let display_mode = display_mode.clone();
        move |album: String| {
            let active_album = active_album.clone();
            let current_page = current_page.clone();
            let display_mode = display_mode.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                active_album.set(Some(album.clone()));
                current_page.set(1);
                display_mode.set(LibraryDisplayMode::AllSongs);
            })
        }
    };

    let clear_artist = {
        let active_artist = active_artist.clone();
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| {
            active_artist.set(None);
            current_page.set(1);
        })
    };

    let clear_album = {
        let active_album = active_album.clone();
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| {
            active_album.set(None);
            current_page.set(1);
        })
    };

    let on_page_change = {
        let current_page = current_page.clone();
        Callback::from(move |page: usize| {
            current_page.set(page);
        })
    };

    let on_switch_random = {
        let display_mode = display_mode.clone();
        Callback::from(move |_: MouseEvent| {
            display_mode.set(LibraryDisplayMode::RandomRecommended);
        })
    };

    let on_switch_all = {
        let display_mode = display_mode.clone();
        Callback::from(move |_: MouseEvent| {
            display_mode.set(LibraryDisplayMode::AllSongs);
        })
    };

    let on_refresh_random = {
        let random_refresh_tick = random_refresh_tick.clone();
        Callback::from(move |_: MouseEvent| {
            random_refresh_tick.set(*random_refresh_tick + 1);
        })
    };

    let on_set_list_layout = {
        let layout_mode = layout_mode.clone();
        Callback::from(move |_: MouseEvent| {
            layout_mode.set(TrackLayoutMode::AcousticList);
        })
    };

    let on_set_grid_layout = {
        let layout_mode = layout_mode.clone();
        Callback::from(move |_: MouseEvent| {
            layout_mode.set(TrackLayoutMode::MinimalGrid);
        })
    };

    // Inline playback handler: toggles play/pause if the song is already loaded,
    // or fetches SongDetail and starts playback in the floating MiniPlayer.
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

    // Keep global playlist synced with current Music Library page results.
    {
        let player_ctx = player_ctx.clone();
        let mode = *display_mode;
        let active_artist = (*active_artist).clone();
        let active_album = (*active_album).clone();
        let current_page = *current_page;
        let random_tick = *random_refresh_tick;
        let ids: Vec<String> = if mode == LibraryDisplayMode::RandomRecommended {
            random_songs.iter().map(|song| song.id.clone()).collect()
        } else {
            page_songs.iter().map(|song| song.id.clone()).collect()
        };
        use_effect_with(
            (
                ids.clone(),
                mode,
                active_artist.clone(),
                active_album.clone(),
                current_page,
                random_tick,
            ),
            move |(ids, mode, artist, album, page, tick)| {
                if let Some(ctx) = player_ctx.as_ref() {
                    let source = if *mode == LibraryDisplayMode::RandomRecommended {
                        format!("music-library:random:tick={tick}")
                    } else {
                        format!(
                            "music-library:all:artist={}:album={}:page={}",
                            artist.clone().unwrap_or_default(),
                            album.clone().unwrap_or_default(),
                            page
                        )
                    };
                    ctx.dispatch(MusicAction::SetPlaylist {
                        source,
                        ids: ids.clone(),
                    });
                }
                || ()
            },
        );
    }

    let current_playing_id = player_ctx.as_ref().and_then(|c| c.song_id.clone());
    let is_global_playing = player_ctx.as_ref().map(|c| c.playing).unwrap_or(false);

    let tab_class = |active: bool| {
        classes!(
            "inline-flex",
            "min-h-[40px]",
            "items-center",
            "gap-2",
            "border-b-[1.5px]",
            "pb-2.5",
            "pt-1",
            "text-sm",
            "bg-transparent",
            "border-t-0",
            "border-x-0",
            "cursor-pointer",
            "transition-colors",
            if active {
                "border-[var(--primary)] font-medium text-[var(--text)]"
            } else {
                "border-transparent text-[var(--muted)] hover:text-[var(--text)]"
            }
        )
    };

    let layout_btn_class = |active: bool| {
        classes!(
            "inline-flex",
            "h-9",
            "items-center",
            "gap-1.5",
            "rounded-[6px]",
            "px-2.5",
            "font-mono",
            "text-xs",
            "cursor-pointer",
            "transition-colors",
            if active {
                "bg-[var(--surface)] text-[var(--text)] border border-[var(--border-strong)]"
            } else {
                "bg-transparent text-[var(--muted)] hover:text-[var(--text)] border \
                 border-transparent"
            }
        )
    };

    let active_songs: &[api::SongListItem] =
        if *display_mode == LibraryDisplayMode::RandomRecommended {
            &random_songs
        } else {
            &page_songs
        };
    let tracks_loading = if *display_mode == LibraryDisplayMode::RandomRecommended {
        *random_loading
    } else {
        *loading
    };
    let tracks_error = if *display_mode == LibraryDisplayMode::RandomRecommended {
        (*random_error).clone()
    } else {
        (*error).clone()
    };

    let studio_music_href = crate::config::route_path("/studio#music");
    let on_go_studio = {
        let target = studio_music_href.clone();
        Callback::from(move |e: MouseEvent| {
            if e.meta_key() || e.ctrl_key() || e.shift_key() || e.alt_key() || e.button() != 0 {
                return;
            }
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&target);
            if let Some(win) = web_sys::window() {
                if let Ok(ev) = web_sys::Event::new("hashchange") {
                    let _ = win.dispatch_event(&ev);
                }
            }
        })
    };

    html! {
        <div class="w-full pb-20 min-w-0">
            // 1. Anthropic Editorial Header
            <header class="mx-auto max-w-[1200px] px-4 sm:px-6 md:px-10 pt-12 sm:pt-16 pb-8 sm:pb-10">
                <div class="border-b border-[var(--border)] pb-8 sm:pb-10 flex flex-col justify-between gap-5 md:flex-row md:items-end">
                    <div class="max-w-2xl min-w-0">
                        <div class="sf-eyebrow">
                            {"AUDIO LIBRARY · ACOUSTIC ARCHIVE"}
                        </div>
                        <h1 class="m-0 font-serif text-[clamp(2rem,1.5rem+2vw,3rem)] font-normal leading-[1.15] tracking-[-0.015em] text-[var(--text)]">
                            {"声学曲库"}
                        </h1>
                        <p class="mt-3 mb-0 text-[0.95rem] leading-[1.65] text-[var(--muted)]">
                            {"收录读者共创的本地无损与高码率曲目，支持在线试听、双语同步歌词与按歌手专辑检索。"}
                        </p>
                    </div>

                    <div class="flex flex-wrap items-center gap-x-4 gap-y-2 shrink-0">
                        if header_song_count > 0 {
                            <span class="font-mono text-xs tabular-nums text-[var(--faint)]">
                                {format!("{header_song_count} 首馆藏曲目")}
                            </span>
                        }
                        <a
                            href={studio_music_href}
                            onclick={on_go_studio}
                            class="sf-link inline-flex min-h-[40px] items-center text-sm text-[var(--muted)] hover:text-[var(--text)]"
                        >
                            { st_t::MUSIC_LIBRARY_STUDIO_CTA }
                        </a>
                    </div>
                </div>
            </header>

            // 2. Acoustic Track Archive Section
            <section class="mx-auto max-w-[1200px] px-4 sm:px-6 md:px-10 min-w-0">
                // Toolbar: Editorial Text Tabs + Search + Layout Switch
                <div class="flex flex-col justify-between gap-4 border-b border-[var(--border)] pb-3 md:flex-row md:items-end">
                    <div class="flex flex-wrap items-center gap-4 sm:gap-5">
                        <button
                            type="button"
                            onclick={on_switch_random}
                            class={tab_class(*display_mode == LibraryDisplayMode::RandomRecommended)}
                        >
                            <span>{"随机推荐"}</span>
                            <span class="font-mono text-xs tabular-nums text-[var(--muted)]">{"10"}</span>
                        </button>
                        <button
                            type="button"
                            onclick={on_switch_all}
                            class={tab_class(*display_mode == LibraryDisplayMode::AllSongs)}
                        >
                            <span>{"全部曲目"}</span>
                            if header_song_count > 0 {
                                <span class="font-mono text-xs tabular-nums text-[var(--muted)]">
                                    {header_song_count}
                                </span>
                            }
                        </button>
                        if *display_mode == LibraryDisplayMode::RandomRecommended {
                            <button
                                type="button"
                                onclick={on_refresh_random}
                                class="sf-chip mb-1 !min-h-[36px] inline-flex cursor-pointer items-center gap-1.5 rounded-full border border-[var(--border)] bg-[var(--surface)] px-3 py-1 font-mono text-xs text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)]"
                            >
                                <Icon name={IconName::Shuffle} size={12} />
                                <span>{"换一批"}</span>
                            </button>
                        }
                    </div>

                    <div class="flex flex-wrap items-center gap-2.5 min-w-0">
                        <div class="relative flex min-w-0 flex-1 items-center sm:w-64 sm:flex-initial">
                            <input
                                type="text"
                                placeholder="搜索歌曲、歌手或专辑..."
                                value={(*hero_query).clone()}
                                oninput={on_hero_input}
                                onkeypress={on_hero_keypress}
                                class="sf-input !h-10 w-full rounded-[8px] pl-3 pr-9 text-sm"
                            />
                            <button
                                type="button"
                                onclick={on_hero_search}
                                aria-label="Search music"
                                class="absolute right-1 inline-flex h-8 w-8 items-center justify-center bg-transparent border-0 cursor-pointer text-[var(--muted)] transition-colors hover:text-[var(--primary)]"
                            >
                                <Icon name={IconName::Search} size={14} />
                            </button>
                        </div>

                        <div class="inline-flex shrink-0 items-center rounded-[8px] border border-[var(--border)] bg-[var(--surface-alt)] p-0.5">
                            <button
                                type="button"
                                onclick={on_set_list_layout}
                                class={layout_btn_class(*layout_mode == TrackLayoutMode::AcousticList)}
                                aria-label="列表视图"
                            >
                                <Icon name={IconName::List} size={13} />
                                <span>{"列表"}</span>
                            </button>
                            <button
                                type="button"
                                onclick={on_set_grid_layout}
                                class={layout_btn_class(*layout_mode == TrackLayoutMode::MinimalGrid)}
                                aria-label="网格视图"
                            >
                                <span>{"网格"}</span>
                            </button>
                        </div>
                    </div>
                </div>

                // Active artist / album filter chips
                if *display_mode == LibraryDisplayMode::AllSongs && (active_artist.is_some() || active_album.is_some()) {
                    <div class="mt-4 flex flex-wrap items-center gap-2">
                        <span class="font-mono text-xs text-[var(--muted)]">{"筛选条件："}</span>
                        if let Some(ref artist) = *active_artist {
                            <span class="sf-chip sf-chip--active !h-auto !py-1 inline-flex items-center gap-1.5 rounded-full pl-3 pr-1.5 text-xs font-medium">
                                <span class="[overflow-wrap:anywhere]">{format!("歌手：{artist}")}</span>
                                <button
                                    onclick={clear_artist}
                                    type="button"
                                    aria-label="清除歌手筛选"
                                    class="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-transparent border-0 cursor-pointer text-current transition-colors hover:bg-[var(--primary)]/20"
                                >
                                    <Icon name={IconName::X} size={12} />
                                </button>
                            </span>
                        }
                        if let Some(ref album) = *active_album {
                            <span class="sf-chip sf-chip--active !h-auto !py-1 inline-flex items-center gap-1.5 rounded-full pl-3 pr-1.5 text-xs font-medium">
                                <span class="[overflow-wrap:anywhere]">{format!("专辑：{album}")}</span>
                                <button
                                    onclick={clear_album}
                                    type="button"
                                    aria-label="清除专辑筛选"
                                    class="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-transparent border-0 cursor-pointer text-current transition-colors hover:bg-[var(--primary)]/20"
                                >
                                    <Icon name={IconName::X} size={12} />
                                </button>
                            </span>
                        }
                    </div>
                }

                if *display_mode == LibraryDisplayMode::RandomRecommended && !tracks_loading && tracks_error.is_none() && !active_songs.is_empty() {
                    <p class="mt-4 mb-0 font-mono text-xs leading-relaxed tracking-[0.02em] text-[var(--muted)]">
                        {"随机甄选 10 首馆藏曲目 · 点击右侧播放按钮试听，点击歌名阅读双语歌词。"}
                    </p>
                }

                // Track List / Grid Body
                <div class="mt-4 min-w-0">
                    if tracks_loading {
                        <div class="divide-y divide-[var(--border)] border-y border-[var(--border)]">
                            { for (0..8).map(|_| html! {
                                <div class="flex h-16 items-center gap-3.5 px-2">
                                    <div class="h-12 w-12 shrink-0 animate-pulse rounded-[6px] bg-[var(--surface-alt)]" />
                                    <div class="min-w-0 flex-1 space-y-2">
                                        <div class="h-3.5 w-40 max-w-full animate-pulse rounded bg-[var(--surface-alt)]" />
                                        <div class="h-3 w-28 max-w-full animate-pulse rounded bg-[var(--surface-alt)]" />
                                    </div>
                                    <div class="h-3 w-12 animate-pulse rounded bg-[var(--surface-alt)] max-sm:hidden" />
                                    <div class="h-10 w-10 shrink-0 animate-pulse rounded-full bg-[var(--surface-alt)]" />
                                </div>
                            }) }
                        </div>
                    } else if let Some(err) = tracks_error {
                        <div class="rounded-[12px] border border-[var(--err)]/30 bg-[var(--err-soft)] p-6 text-sm text-[var(--err)] [overflow-wrap:anywhere]">
                            {format!("加载曲目列表失败：{err}")}
                        </div>
                    } else if active_songs.is_empty() {
                        <div class="rounded-[12px] border border-[var(--border)] bg-[var(--surface)] px-6 py-14 text-center text-sm text-[var(--muted)]">
                            {"当前列表暂无曲目。"}
                        </div>
                    } else if *layout_mode == TrackLayoutMode::AcousticList {
                        <div class="divide-y divide-[var(--border)] border-y border-[var(--border)]">
                            { for active_songs.iter().enumerate().map(|(idx, song)| {
                                let display_idx = if *display_mode == LibraryDisplayMode::AllSongs {
                                    (*current_page - 1) * PAGE_SIZE + idx + 1
                                } else {
                                    idx + 1
                                };
                                let is_current = current_playing_id.as_deref() == Some(song.id.as_str());
                                let is_row_playing = is_current && is_global_playing;
                                let is_row_loading = inline_play_loading_id.as_deref() == Some(song.id.as_str());
                                render_acoustic_track_row(
                                    display_idx,
                                    song,
                                    is_current,
                                    is_row_playing,
                                    is_row_loading,
                                    &on_artist_click,
                                    &on_album_click,
                                    &on_inline_play,
                                )
                            }) }
                        </div>
                    } else {
                        <div class="grid grid-cols-2 gap-3.5 sm:grid-cols-3 md:gap-5 lg:grid-cols-5">
                            { for active_songs.iter().map(|song| {
                                let is_current = current_playing_id.as_deref() == Some(song.id.as_str());
                                let is_card_playing = is_current && is_global_playing;
                                let is_card_loading = inline_play_loading_id.as_deref() == Some(song.id.as_str());
                                render_minimal_song_card(
                                    song,
                                    is_current,
                                    is_card_playing,
                                    is_card_loading,
                                    &on_artist_click,
                                    &on_album_click,
                                    &on_inline_play,
                                )
                            }) }
                        </div>
                    }
                </div>

                if *display_mode == LibraryDisplayMode::AllSongs && total_pages > 1 {
                    <div class="mt-10 flex justify-center">
                        <Pagination
                            current_page={*current_page}
                            total_pages={total_pages}
                            on_page_change={on_page_change}
                        />
                    </div>
                }
            </section>
        </div>
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Track row helper renders playback, filter, and loading state together without an \
              intermediate wrapper component."
)]
fn render_acoustic_track_row(
    index: usize,
    song: &api::SongListItem,
    is_current: bool,
    is_playing: bool,
    is_loading: bool,
    on_artist_click: &dyn Fn(String) -> Callback<MouseEvent>,
    on_album_click: &dyn Fn(String) -> Callback<MouseEvent>,
    on_inline_play: &Callback<String>,
) -> Html {
    let cover_url = api::song_cover_url(song.cover_image.as_deref());
    let duration = format_duration(song.duration_ms);
    let id = song.id.clone();
    let play_id = song.id.clone();
    let artist_cb = on_artist_click(song.artist.clone());
    let album_cb = on_album_click(song.album.clone());
    let on_play_click = {
        let cb = on_inline_play.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            cb.emit(play_id.clone());
        })
    };

    let row_class = classes!(
        "group",
        "flex",
        "min-h-[64px]",
        "items-center",
        "gap-3",
        "sm:gap-3.5",
        "px-1.5",
        "sm:px-3",
        "py-2.5",
        "min-w-0",
        "transition-colors",
        if is_current {
            "border-l-2 border-l-[var(--primary)] bg-[var(--primary-soft)]"
        } else {
            "hover:bg-[var(--surface-alt)]/75"
        }
    );

    html! {
        <div class={row_class}>
            // Index or active marker (hidden on mobile)
            <div class="w-6 shrink-0 text-right font-mono text-xs tabular-nums text-[var(--faint)] max-sm:hidden">
                if is_current {
                    <span class="text-[var(--primary)]">{"◆"}</span>
                } else {
                    {format!("{index:02}")}
                }
            </div>

            // 48px static square cover
            <Link<Route>
                to={Route::MusicPlayer { id: id.clone() }}
                classes="relative h-12 w-12 shrink-0 overflow-hidden rounded-[6px] border border-[var(--border)] bg-[var(--surface-alt)]"
            >
                if cover_url.is_empty() {
                    <div class="flex h-full w-full items-center justify-center text-[var(--muted)]">
                        <Icon name={IconName::Music} size={20} class={classes!("opacity-40")} />
                    </div>
                } else {
                    <ImageWithLoading
                        src={cover_url}
                        alt={song.title.clone()}
                        loading={Some(AttrValue::from("lazy"))}
                        referrerpolicy={Some(AttrValue::from("no-referrer"))}
                        class="h-full w-full object-cover"
                        container_class={classes!("w-full", "h-full")}
                    />
                }
            </Link<Route>>

            // Serif song title + muted artist / album (truncates cleanly at 390px)
            <div class="min-w-0 flex-1">
                <Link<Route>
                    to={Route::MusicPlayer { id }}
                    classes={classes!(
                        "block",
                        "truncate",
                        "font-serif",
                        "text-[1.02rem]",
                        "font-normal",
                        "leading-snug",
                        "no-underline",
                        "transition-colors",
                        if is_current {
                            "text-[var(--primary)]"
                        } else {
                            "text-[var(--text)] hover:text-[var(--primary)]"
                        }
                    )}
                >
                    {&song.title}
                </Link<Route>>
                <div class="mt-0.5 flex items-center gap-1.5 min-w-0 text-xs text-[var(--muted)]">
                    <a
                        href="#"
                        onclick={artist_cb}
                        class="cursor-pointer truncate text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
                    >
                        {&song.artist}
                    </a>
                    if !song.album.is_empty() {
                        <span class="text-[var(--border-strong)] shrink-0 max-sm:hidden">{"·"}</span>
                        <a
                            href="#"
                            onclick={album_cb}
                            class="cursor-pointer truncate text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)] max-sm:hidden"
                        >
                            {&song.album}
                        </a>
                    }
                </div>
            </div>

            // Audio format tag on desktop
            if !song.format.is_empty() {
                <span class="shrink-0 rounded border border-[var(--border)] px-1.5 py-0.5 font-mono text-[10px] uppercase tracking-[0.06em] text-[var(--muted)] max-md:hidden">
                    {&song.format}
                </span>
            }

            // Monospace duration (hidden on mobile so cover + title/artist + play button breathe)
            <span class="w-12 shrink-0 text-right font-mono text-xs tabular-nums text-[var(--muted)] max-sm:hidden">
                {&duration}
            </span>

            // 40px circular hairline play button
            <button
                type="button"
                onclick={on_play_click}
                aria-label={if is_playing { "暂停" } else { "播放" }}
                class={classes!(
                    "inline-flex",
                    "h-10",
                    "w-10",
                    "sm:h-9",
                    "sm:w-9",
                    "shrink-0",
                    "items-center",
                    "justify-center",
                    "rounded-full",
                    "border",
                    "cursor-pointer",
                    "transition-colors",
                    if is_playing {
                        "border-[var(--primary)] bg-[var(--primary)] text-[var(--on-primary)]"
                    } else {
                        "border-[var(--border-strong)] bg-transparent text-[var(--text)] hover:border-[var(--primary)] hover:bg-[var(--primary)] hover:text-[var(--on-primary)]"
                    }
                )}
            >
                if is_loading {
                    <span class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent motion-reduce:animate-none" />
                } else {
                    <Icon name={if is_playing { IconName::Pause } else { IconName::Play }} size={13} />
                }
            </button>
        </div>
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Minimal grid card helper takes playback, filter, and loading callbacks directly."
)]
fn render_minimal_song_card(
    song: &api::SongListItem,
    is_current: bool,
    is_playing: bool,
    is_loading: bool,
    on_artist_click: &dyn Fn(String) -> Callback<MouseEvent>,
    on_album_click: &dyn Fn(String) -> Callback<MouseEvent>,
    on_inline_play: &Callback<String>,
) -> Html {
    let cover_url = api::song_cover_url(song.cover_image.as_deref());
    let duration = format_duration(song.duration_ms);
    let id = song.id.clone();
    let play_id = song.id.clone();
    let artist_cb = on_artist_click(song.artist.clone());
    let album_cb = on_album_click(song.album.clone());
    let on_play_click = {
        let cb = on_inline_play.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            cb.emit(play_id.clone());
        })
    };

    html! {
        <div class={classes!(
            "group",
            "flex",
            "flex-col",
            "overflow-hidden",
            "rounded-[8px]",
            "border",
            "bg-[var(--surface)]",
            "min-w-0",
            "transition-colors",
            if is_current {
                "border-[var(--primary)]"
            } else {
                "border-[var(--border)] hover:border-[var(--border-strong)]"
            }
        )}>
            <Link<Route> to={Route::MusicPlayer { id: id.clone() }} classes="block">
                <div class="relative aspect-square overflow-hidden bg-[var(--surface-alt)]">
                    if cover_url.is_empty() {
                        <div class="flex h-full w-full items-center justify-center text-[var(--muted)]">
                            <Icon name={IconName::Music} size={40} class={classes!("opacity-30")} />
                        </div>
                    } else {
                        <ImageWithLoading
                            src={cover_url}
                            alt={song.title.clone()}
                            loading={Some(AttrValue::from("lazy"))}
                            referrerpolicy={Some(AttrValue::from("no-referrer"))}
                            class="h-full w-full object-cover"
                            container_class={classes!("w-full", "h-full")}
                        />
                    }
                    <div class="absolute bottom-2 right-2 rounded-[4px] bg-[rgba(20,20,19,0.72)] px-1.5 py-0.5 font-mono text-[11px] tabular-nums text-[#FAF9F5]">
                        {&duration}
                    </div>
                </div>
            </Link<Route>>
            <div class="flex flex-1 flex-col justify-between gap-2 border-t border-[var(--border)] p-3 min-w-0">
                <div class="min-w-0">
                    <Link<Route>
                        to={Route::MusicPlayer { id }}
                        classes="block truncate font-serif text-[0.95rem] font-normal leading-snug text-[var(--text)] no-underline transition-colors hover:text-[var(--primary)]"
                    >
                        {&song.title}
                    </Link<Route>>
                    <a
                        href="#"
                        onclick={artist_cb}
                        class="mt-0.5 block cursor-pointer truncate text-xs text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
                    >
                        {&song.artist}
                    </a>
                </div>
                <div class="flex items-center justify-between gap-2 pt-1 min-w-0">
                    if !song.album.is_empty() {
                        <a
                            href="#"
                            onclick={album_cb}
                            class="min-w-0 flex-1 truncate text-[11px] text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
                        >
                            {&song.album}
                        </a>
                    } else {
                        <span />
                    }
                    <button
                        type="button"
                        onclick={on_play_click}
                        aria-label={if is_playing { "暂停" } else { "播放" }}
                        class={classes!(
                            "inline-flex",
                            "h-9",
                            "w-9",
                            "sm:h-8",
                            "sm:w-8",
                            "shrink-0",
                            "items-center",
                            "justify-center",
                            "rounded-full",
                            "border",
                            "cursor-pointer",
                            "transition-colors",
                            if is_playing {
                                "border-[var(--primary)] bg-[var(--primary)] text-[var(--on-primary)]"
                            } else {
                                "border-[var(--border-strong)] bg-transparent text-[var(--text)] hover:border-[var(--primary)] hover:bg-[var(--primary)] hover:text-[var(--on-primary)]"
                            }
                        )}
                    >
                        if is_loading {
                            <span class="h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent motion-reduce:animate-none" />
                        } else {
                            <Icon name={if is_playing { IconName::Pause } else { IconName::Play }} size={12} />
                        }
                    </button>
                </div>
            </div>
        </div>
    }
}

fn format_duration(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
