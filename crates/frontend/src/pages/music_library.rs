use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    components::{
        form::{FormField, TextArea, TextInput},
        icons::{Icon, IconName},
        image_with_loading::ImageWithLoading,
        pagination::Pagination,
        toast::use_toast,
    },
    i18n::current::{header as header_t, music_wish as wish_t},
    music_context::{MusicAction, MusicPlayerContext},
    router::Route,
};

const PAGE_SIZE: usize = 20;
const RANDOM_RECOMMEND_LIMIT: usize = 10;
const WISH_PAGE_SIZE: usize = 10;

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

    // Wish board state
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

    // Auto-scroll smoothly when URL hash targets #music-wish-section (on mount and
    // hashchange)
    {
        let query_dep = query_string.clone();
        use_effect_with(query_dep, move |_| {
            let check_and_scroll = || {
                let hash = web_sys::window()
                    .and_then(|w| w.location().hash().ok())
                    .unwrap_or_default();
                if hash == "#music-wish-section" || hash == "#music-wish-form" {
                    let target = hash.trim_start_matches('#').to_string();
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(80).await;
                        smooth_scroll_to_id(&target);
                    });
                }
            };
            check_and_scroll();

            let mut listener_opt: Option<gloo_events::EventListener> = None;
            if let Some(win) = web_sys::window() {
                listener_opt =
                    Some(gloo_events::EventListener::new(&win, "hashchange", move |_| {
                        check_and_scroll();
                    }));
            }
            move || {
                drop(listener_opt);
            }
        });
    }

    let on_jump_to_wish = Callback::from(|_: MouseEvent| {
        smooth_scroll_to_id("music-wish-section");
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(el) = doc.get_element_by_id("wish-song") {
                if let Ok(input) = el.dyn_into::<web_sys::HtmlElement>() {
                    let _ = input.focus();
                }
            }
        }
    });

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
    let wish_total_val = *wish_total;
    let wish_total_pages =
        if wish_total_val == 0 { 1 } else { wish_total_val.div_ceil(WISH_PAGE_SIZE) };

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

    let on_wish_page_change = {
        let wish_page = wish_page.clone();
        Callback::from(move |page: usize| {
            wish_page.set(page);
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

    // Fetch wishes when page changes or manual refresh is triggered.
    {
        let wishes = wishes.clone();
        let wish_loading = wish_loading.clone();
        let wish_list_error = wish_list_error.clone();
        let wish_total = wish_total.clone();
        let wish_request_seq = wish_request_seq.clone();
        let deps = (*wish_page, *wish_refresh_tick);
        use_effect_with(deps, move |(page, _)| {
            let request_id = {
                let mut seq = wish_request_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            let wishes = wishes.clone();
            let wish_loading = wish_loading.clone();
            let wish_list_error = wish_list_error.clone();
            let wish_total = wish_total.clone();
            let wish_request_seq = wish_request_seq.clone();
            let offset = ((*page).saturating_sub(1)) * WISH_PAGE_SIZE;
            wish_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_music_wishes(Some(WISH_PAGE_SIZE), Some(offset)).await {
                    Ok(resp) => {
                        if *wish_request_seq.borrow() != request_id {
                            return;
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

    // Manual refresh wishes callback
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
        let wish_submit_banner = wish_submit_banner.clone();
        Callback::from(move |w: api::MusicWishItem| {
            wish_form_song.set(w.song_name.clone());
            wish_form_artist.set(w.artist_hint.unwrap_or_default());
            wish_form_message.set(w.wish_message.clone());
            wish_submit_banner.set(None);
            smooth_scroll_to_id("music-wish-form");
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
                        wish_refresh_tick.set(*wish_refresh_tick + 1);
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

    let tab_class = |active: bool| {
        classes!(
            "inline-flex",
            "items-center",
            "gap-2",
            "border-b-[1.5px]",
            "pb-2.5",
            "pt-1",
            "text-sm",
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
            "h-8",
            "items-center",
            "gap-1.5",
            "rounded-[6px]",
            "px-2.5",
            "font-mono",
            "text-xs",
            "transition-colors",
            if active {
                "bg-[var(--surface)] text-[var(--text)] border \
                 border-[var(--border-strong,var(--border))]"
            } else {
                "text-[var(--muted)] hover:text-[var(--text)] border border-transparent"
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

    html! {
        <div class="w-full pb-20">
            // 1. Anthropic Editorial Header
            <header class="mx-auto max-w-[1200px] px-4 pb-8 pt-10 sm:px-6 md:pt-14 lg:px-8">
                <div class="flex flex-col justify-between gap-6 md:flex-row md:items-end">
                    <div class="max-w-2xl">
                        <div class="sf-eyebrow mb-3 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--muted)]">
                            {"AUDIO LIBRARY · READER-WISHED ARCHIVE"}
                        </div>
                        <h1 class="font-serif text-[clamp(2.125rem,4vw,3.125rem)] font-normal leading-[1.1] tracking-[-0.02em] text-[var(--text)]">
                            {"声学曲库与点歌工坊"}
                        </h1>
                        <p class="mt-3 text-[0.975rem] leading-[1.65] text-[var(--text-2,var(--muted))]">
                            {"收录读者共创的本地无损与高码率曲目。想听的旋律尚未入库？直接在下方点歌工坊许愿，AI 将自动寻源、对齐双语歌词并写入曲库。"}
                        </p>
                        <div class="mt-4 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-xs tabular-nums text-[var(--muted)]">
                            <span>{format!("{header_song_count} 首馆藏曲目")}</span>
                            <span class="text-[var(--border-strong,var(--border))]">{"·"}</span>
                            <span>{format!("{wish_total_val} 条点歌心愿")}</span>
                            if has_active_wish_runner {
                                <span class="text-[var(--border-strong,var(--border))]">{"·"}</span>
                                <span class="inline-flex items-center gap-1.5 text-[var(--primary)]">
                                    <span class="sf-status sf-status--running inline-block h-1.5 w-1.5 rounded-full bg-[var(--primary)] animate-pulse" />
                                    {"AI 正在寻源入库"}
                                </span>
                            }
                        </div>
                    </div>

                    <div class="flex shrink-0 flex-wrap items-center gap-2.5">
                        <button
                            type="button"
                            onclick={on_jump_to_wish}
                            class="sf-btn sf-btn--primary inline-flex items-center gap-2 rounded-[8px] bg-[var(--primary)] px-4 py-2.5 text-sm font-medium text-[var(--on-primary,#FAF9F5)] transition-colors hover:bg-[var(--primary-hover,#B85735)]"
                        >
                            <span>{"许下歌愿 ↓"}</span>
                        </button>
                        <Link<Route>
                            to={Route::MediaImage}
                            classes="sf-btn sf-btn--secondary inline-flex items-center gap-2 rounded-[8px] border border-[var(--border-strong,var(--border))] bg-transparent px-3.5 py-2.5 text-sm font-medium text-[var(--text)] no-underline transition-colors hover:bg-[var(--surface-alt)]"
                        >
                            <span>{ header_t::IMAGE_LIBRARY_TITLE }</span>
                            <span class="font-mono text-xs text-[var(--muted)]">{"→"}</span>
                        </Link<Route>>
                    </div>
                </div>
            </header>

            // 2. Prominent Full-Bleed Self-Service Music Wish Workshop (#music-wish-section)
            <section
                id="music-wish-section"
                class="w-full scroll-mt-16 border-y border-[var(--border)] bg-[var(--surface-alt)] py-10 sm:py-14"
            >
                <div class="mx-auto max-w-[1200px] px-4 sm:px-6 lg:px-8">
                    <div class="mb-7 flex flex-col justify-between gap-3 sm:flex-row sm:items-end">
                        <div>
                            <div class="sf-eyebrow mb-2 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--primary)]">
                                {"SELF-SERVICE JUKEBOX · 自助点歌入库工坊"}
                            </div>
                            <h2 class="font-serif text-[clamp(1.5rem,2.6vw,2rem)] font-normal leading-[1.2] tracking-[-0.015em] text-[var(--text)]">
                                {wish_t::SECTION_TITLE}
                            </h2>
                            <p class="mt-1.5 max-w-2xl text-sm leading-relaxed text-[var(--muted)]">
                                {wish_t::SECTION_SUBTITLE}
                            </p>
                        </div>
                        <div class="font-mono text-[11px] tracking-[0.03em] text-[var(--muted)]">
                            {"submit → review → AI fetch & LRC → LanceDB"}
                        </div>
                    </div>

                    <div class="grid grid-cols-1 gap-7 lg:grid-cols-12 lg:items-start">
                        // Left: Editorial Wish Form Card
                        <div class="lg:col-span-5">
                            <form
                                id="music-wish-form"
                                onsubmit={on_wish_submit}
                                class="sf-card scroll-mt-24 rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-5 sm:p-6"
                            >
                                <div class="mb-4 flex items-center justify-between border-b border-[var(--border)] pb-3">
                                    <span class="font-serif text-base font-normal text-[var(--text)]">
                                        {"填写歌愿单"}
                                    </span>
                                    <span class="font-mono text-[11px] uppercase tracking-[0.1em] text-[var(--muted)]">
                                        {"WISH REQUEST"}
                                    </span>
                                </div>

                                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                                    <FormField
                                        label={wish_t::SONG_NAME_LABEL}
                                        id="wish-song"
                                        required=true
                                        error={(*wish_song_error).clone()}
                                    >
                                        <TextInput
                                            id="wish-song"
                                            value={(*wish_form_song).clone()}
                                            placeholder={wish_t::SONG_NAME_PLACEHOLDER}
                                            invalid={wish_song_error.is_some()}
                                            on_change={let s = wish_form_song.clone(); Callback::from(move |v| s.set(v))}
                                        />
                                    </FormField>
                                    <FormField label={wish_t::ARTIST_LABEL} id="wish-artist">
                                        <TextInput
                                            id="wish-artist"
                                            value={(*wish_form_artist).clone()}
                                            placeholder={wish_t::ARTIST_PLACEHOLDER}
                                            on_change={let s = wish_form_artist.clone(); Callback::from(move |v| s.set(v))}
                                        />
                                    </FormField>
                                    <div class="sm:col-span-2">
                                        <FormField
                                            label={wish_t::MESSAGE_LABEL}
                                            id="wish-message"
                                            required=true
                                            error={(*wish_message_error).clone()}
                                        >
                                            <TextArea
                                                id="wish-message"
                                                value={(*wish_form_message).clone()}
                                                placeholder={wish_t::MESSAGE_PLACEHOLDER}
                                                rows={3}
                                                invalid={wish_message_error.is_some()}
                                                on_change={let s = wish_form_message.clone(); Callback::from(move |v| s.set(v))}
                                            />
                                        </FormField>
                                    </div>
                                    <FormField label={wish_t::NICKNAME_LABEL} id="wish-nickname">
                                        <TextInput
                                            id="wish-nickname"
                                            value={(*wish_form_nickname).clone()}
                                            placeholder={wish_t::NICKNAME_PLACEHOLDER}
                                            on_change={let s = wish_form_nickname.clone(); Callback::from(move |v| s.set(v))}
                                        />
                                    </FormField>
                                    <FormField
                                        label={wish_t::EMAIL_LABEL}
                                        id="wish-email"
                                        hint={wish_t::EMAIL_HELP_TEXT}
                                    >
                                        <TextInput
                                            id="wish-email"
                                            kind="email"
                                            value={(*wish_form_email).clone()}
                                            placeholder={wish_t::EMAIL_PLACEHOLDER}
                                            on_change={let s = wish_form_email.clone(); Callback::from(move |v| s.set(v))}
                                        />
                                    </FormField>
                                </div>

                                if let Some((ok, ref msg)) = *wish_submit_banner {
                                    <div class={classes!(
                                        "mt-4",
                                        "rounded-[8px]",
                                        "border",
                                        "px-3.5",
                                        "py-2.5",
                                        "text-xs",
                                        "leading-relaxed",
                                        if ok {
                                            "border-[var(--ok,#4E7A51)]/35 bg-[var(--ok-soft,rgba(78,122,81,0.10))] text-[var(--ok,#4E7A51)]"
                                        } else {
                                            "border-[var(--err,#B8443A)]/35 bg-[var(--err-soft,rgba(184,68,58,0.10))] text-[var(--err,#B8443A)]"
                                        }
                                    )}>
                                        {msg}
                                    </div>
                                }

                                <div class="mt-5 flex items-center justify-between gap-3 border-t border-[var(--border)] pt-4">
                                    <span class="font-mono text-[11px] text-[var(--muted)]">
                                        {"审核通过后自动入库并邮件通知"}
                                    </span>
                                    <button
                                        type="submit"
                                        disabled={*wish_submitting}
                                        class="sf-btn sf-btn--primary inline-flex shrink-0 items-center gap-2 rounded-[8px] bg-[var(--primary)] px-4 py-2 text-sm font-medium text-[var(--on-primary,#FAF9F5)] transition-colors hover:bg-[var(--primary-hover,#B85735)] disabled:cursor-not-allowed disabled:opacity-50"
                                    >
                                        if *wish_submitting {
                                            <span
                                                class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-[var(--on-primary,#FAF9F5)]/40 border-t-[var(--on-primary,#FAF9F5)] motion-reduce:animate-none"
                                                aria-hidden="true"
                                            />
                                        }
                                        <span>{if *wish_submitting { wish_t::SUBMITTING } else { wish_t::SUBMIT_BTN }}</span>
                                    </button>
                                </div>
                            </form>
                        </div>

                        // Right: Live Music Wish Queue
                        <div class="lg:col-span-7">
                            <div class="sf-card overflow-hidden rounded-[12px] border border-[var(--border)] bg-[var(--surface)]">
                                <div class="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border)] px-5 py-3.5">
                                    <div class="flex items-center gap-2.5">
                                        <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">
                                            {"WISH QUEUE · 实时心愿队列"}
                                        </span>
                                        <span class="rounded-full border border-[var(--border)] bg-[var(--surface-alt)] px-2 py-0.5 font-mono text-[11px] tabular-nums text-[var(--muted)]">
                                            {wish_total_val}
                                        </span>
                                        if has_active_wish_runner {
                                            <span class="inline-flex items-center gap-1.5 rounded-full bg-[var(--primary-soft,rgba(201,100,66,0.12))] px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.1em] text-[var(--primary)]">
                                                <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-[var(--primary)]" />
                                                {"LIVE"}
                                            </span>
                                        }
                                    </div>
                                    <button
                                        type="button"
                                        onclick={on_refresh_wishes}
                                        disabled={*wish_loading}
                                        class="sf-btn sf-btn--ghost sf-btn--sm inline-flex items-center gap-1.5 rounded-[6px] border border-[var(--border)] px-2.5 py-1 font-mono text-xs text-[var(--muted)] transition-colors hover:border-[var(--border-strong,var(--primary))] hover:text-[var(--text)] disabled:cursor-not-allowed disabled:opacity-50"
                                    >
                                        <svg
                                            class={if *wish_loading { "h-3.5 w-3.5 animate-spin motion-reduce:animate-none" } else { "h-3.5 w-3.5" }}
                                            xmlns="http://www.w3.org/2000/svg"
                                            fill="none"
                                            viewBox="0 0 24 24"
                                            stroke-width="1.8"
                                            stroke="currentColor"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182"
                                            />
                                        </svg>
                                        <span>{if *wish_loading { wish_t::REFRESHING } else { wish_t::REFRESH_BTN }}</span>
                                    </button>
                                </div>

                                if *wish_loading && wishes.is_empty() {
                                    <div class="space-y-3 p-5">
                                        { for (0..4).map(|_| html! {
                                            <div class="h-14 animate-pulse rounded-[8px] bg-[var(--surface-alt)]" />
                                        }) }
                                    </div>
                                } else if let Some(err) = (*wish_list_error).clone() {
                                    <div class="p-6 text-sm text-[var(--err,#B8443A)]">
                                        {format!("加载心愿队列失败：{err}")}
                                    </div>
                                } else if wishes.is_empty() {
                                    <div class="px-6 py-12 text-center text-sm text-[var(--muted)]">
                                        {wish_t::EMPTY_LIST}
                                    </div>
                                } else {
                                    <div class="divide-y divide-[var(--border)]">
                                        { for wishes.iter().map(|w| {
                                            let is_wish_song_playing = w.ingested_song_id.as_ref().is_some_and(|sid| {
                                                current_playing_id.as_deref() == Some(sid.as_str()) && is_global_playing
                                            });
                                            html! {
                                                <WishQueueRow
                                                    key={w.wish_id.clone()}
                                                    wish={w.clone()}
                                                    on_fill_form={on_wish_fill_form.clone()}
                                                    on_play_song={on_inline_play.clone()}
                                                    is_playing={is_wish_song_playing}
                                                />
                                            }
                                        }) }
                                    </div>
                                }

                                if wish_total_pages > 1 {
                                    <div class="flex justify-center border-t border-[var(--border)] px-4 py-3.5">
                                        <Pagination
                                            current_page={*wish_page}
                                            total_pages={wish_total_pages}
                                            on_page_change={on_wish_page_change}
                                        />
                                    </div>
                                }
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            // 3. Acoustic Track Archive Section
            <section class="mx-auto max-w-[1200px] px-4 pt-10 sm:px-6 sm:pt-12 lg:px-8">
                // Toolbar: Editorial Text Tabs + Search + Layout Switch
                <div class="flex flex-col justify-between gap-4 border-b border-[var(--border)] pb-3 md:flex-row md:items-end">
                    <div class="flex flex-wrap items-center gap-5">
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
                                class="sf-chip mb-1 inline-flex cursor-pointer items-center gap-1.5 rounded-full border border-[var(--border)] bg-[var(--surface)] px-3 py-1 font-mono text-xs text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)]"
                            >
                                <Icon name={IconName::Shuffle} size={12} />
                                <span>{"换一批"}</span>
                            </button>
                        }
                    </div>

                    <div class="flex flex-wrap items-center gap-2.5">
                        <div class="relative flex min-w-[220px] flex-1 items-center sm:w-64 sm:flex-initial">
                            <input
                                type="text"
                                placeholder="搜索歌曲、歌手或专辑..."
                                value={(*hero_query).clone()}
                                oninput={on_hero_input}
                                onkeypress={on_hero_keypress}
                                class="sf-input h-9 w-full rounded-[8px] border border-[var(--border)] bg-[var(--surface)] pl-3 pr-8 text-sm text-[var(--text)] placeholder-[var(--muted)] transition-colors focus:border-[var(--primary)] focus:outline-none"
                            />
                            <button
                                type="button"
                                onclick={on_hero_search}
                                aria-label="Search music"
                                class="absolute right-2 inline-flex h-6 w-6 items-center justify-center text-[var(--muted)] transition-colors hover:text-[var(--primary)]"
                            >
                                <Icon name={IconName::Search} size={14} />
                            </button>
                        </div>

                        <div class="inline-flex items-center rounded-[8px] border border-[var(--border)] bg-[var(--surface-alt)] p-0.5">
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
                            <span class="sf-chip sf-chip--active inline-flex items-center gap-1.5 rounded-full border border-[var(--primary)]/35 bg-[var(--primary-soft,rgba(201,100,66,0.12))] py-1 pl-3 pr-1.5 text-xs font-medium text-[var(--primary)]">
                                <span>{format!("歌手：{artist}")}</span>
                                <button
                                    onclick={clear_artist}
                                    type="button"
                                    aria-label="清除歌手筛选"
                                    class="inline-flex h-4 w-4 items-center justify-center rounded-full transition-colors hover:bg-[var(--primary)]/20"
                                >
                                    <Icon name={IconName::X} size={11} />
                                </button>
                            </span>
                        }
                        if let Some(ref album) = *active_album {
                            <span class="sf-chip sf-chip--active inline-flex items-center gap-1.5 rounded-full border border-[var(--primary)]/35 bg-[var(--primary-soft,rgba(201,100,66,0.12))] py-1 pl-3 pr-1.5 text-xs font-medium text-[var(--primary)]">
                                <span>{format!("专辑：{album}")}</span>
                                <button
                                    onclick={clear_album}
                                    type="button"
                                    aria-label="清除专辑筛选"
                                    class="inline-flex h-4 w-4 items-center justify-center rounded-full transition-colors hover:bg-[var(--primary)]/20"
                                >
                                    <Icon name={IconName::X} size={11} />
                                </button>
                            </span>
                        }
                    </div>
                }

                if *display_mode == LibraryDisplayMode::RandomRecommended && !tracks_loading && tracks_error.is_none() && !active_songs.is_empty() {
                    <p class="mt-4 font-mono text-xs tracking-[0.02em] text-[var(--muted)]">
                        {"随机甄选 10 首馆藏曲目 · 点击右侧圆形按钮可直接唤起悬浮播放器试听，点击歌名进入歌词阅读页。"}
                    </p>
                }

                // Track List / Grid Body
                <div class="mt-4">
                    if tracks_loading {
                        <div class="divide-y divide-[var(--border)] border-y border-[var(--border)]">
                            { for (0..8).map(|_| html! {
                                <div class="flex h-16 items-center gap-4 px-2">
                                    <div class="h-12 w-12 shrink-0 animate-pulse rounded-[6px] bg-[var(--surface-alt)]" />
                                    <div class="flex-1 space-y-2">
                                        <div class="h-3.5 w-48 animate-pulse rounded bg-[var(--surface-alt)]" />
                                        <div class="h-3 w-32 animate-pulse rounded bg-[var(--surface-alt)]" />
                                    </div>
                                    <div class="h-3 w-12 animate-pulse rounded bg-[var(--surface-alt)]" />
                                    <div class="h-8 w-8 shrink-0 animate-pulse rounded-full bg-[var(--surface-alt)]" />
                                </div>
                            }) }
                        </div>
                    } else if let Some(err) = tracks_error {
                        <div class="rounded-[12px] border border-[var(--err,#B8443A)]/30 bg-[var(--err-soft,rgba(184,68,58,0.08))] p-6 text-sm text-[var(--err,#B8443A)]">
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
                        <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:gap-5 lg:grid-cols-5">
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
        "gap-3.5",
        "px-2",
        "py-2.5",
        "transition-colors",
        "sm:px-3",
        if is_current {
            "border-l-2 border-l-[var(--primary)] bg-[var(--primary-soft,rgba(201,100,66,0.08))]"
        } else {
            "hover:bg-[var(--surface-alt)]/75"
        }
    );

    html! {
        <div class={row_class}>
            // Index or active marker (static/subtle, zero vinyl spin)
            <div class="w-6 shrink-0 text-right font-mono text-xs tabular-nums text-[var(--faint,var(--muted))] max-sm:hidden">
                if is_current {
                    <span class="text-[var(--primary)]">{"◆"}</span>
                } else {
                    {format!("{index:02}")}
                }
            </div>

            // 48px static square cover (rounded-[6px])
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

            // Serif song title + muted artist / album
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
                <div class="mt-0.5 flex items-center gap-1.5 truncate text-xs text-[var(--muted)]">
                    <a
                        href="#"
                        onclick={artist_cb}
                        class="cursor-pointer truncate text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
                    >
                        {&song.artist}
                    </a>
                    if !song.album.is_empty() {
                        <span class="text-[var(--border-strong,var(--border))]">{"·"}</span>
                        <a
                            href="#"
                            onclick={album_cb}
                            class="cursor-pointer truncate text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
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

            // Monospace duration
            <span class="w-12 shrink-0 text-right font-mono text-xs tabular-nums text-[var(--muted)]">
                {&duration}
            </span>

            // 32px hairline circular play button (hover terracotta)
            <button
                type="button"
                onclick={on_play_click}
                aria-label={if is_playing { "暂停" } else { "播放" }}
                class={classes!(
                    "inline-flex",
                    "h-8",
                    "w-8",
                    "shrink-0",
                    "items-center",
                    "justify-center",
                    "rounded-full",
                    "border",
                    "transition-colors",
                    if is_playing {
                        "border-[var(--primary)] bg-[var(--primary)] text-[var(--on-primary,#FAF9F5)]"
                    } else {
                        "border-[var(--border-strong,var(--border))] bg-transparent text-[var(--text)] hover:border-[var(--primary)] hover:bg-[var(--primary)] hover:text-[var(--on-primary,#FAF9F5)]"
                    }
                )}
            >
                if is_loading {
                    <span class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
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
            "transition-colors",
            if is_current {
                "border-[var(--primary)]"
            } else {
                "border-[var(--border)] hover:border-[var(--border-strong,var(--primary))]"
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
            <div class="flex flex-1 flex-col justify-between gap-2 border-t border-[var(--border)] p-3">
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
                <div class="flex items-center justify-between gap-2 pt-1">
                    if !song.album.is_empty() {
                        <a
                            href="#"
                            onclick={album_cb}
                            class="max-w-[75%] truncate text-[11px] text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
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
                            "h-7",
                            "w-7",
                            "shrink-0",
                            "items-center",
                            "justify-center",
                            "rounded-full",
                            "border",
                            "transition-colors",
                            if is_playing {
                                "border-[var(--primary)] bg-[var(--primary)] text-[var(--on-primary,#FAF9F5)]"
                            } else {
                                "border-[var(--border-strong,var(--border))] text-[var(--text)] hover:border-[var(--primary)] hover:bg-[var(--primary)] hover:text-[var(--on-primary,#FAF9F5)]"
                            }
                        )}
                    >
                        if is_loading {
                            <span class="h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
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

#[derive(Properties, PartialEq)]
struct WishQueueRowProps {
    pub wish: api::MusicWishItem,
    pub on_fill_form: Callback<api::MusicWishItem>,
    pub on_play_song: Callback<String>,
    pub is_playing: bool,
}

#[function_component(WishQueueRow)]
fn wish_queue_row(props: &WishQueueRowProps) -> Html {
    let w = &props.wish;
    let log_open = use_state(|| false);
    let ai_log_loading = use_state(|| false);
    let ai_log_fetched = use_state(|| false);
    let ai_log_text = use_state(|| None::<String>);

    let (status_dot_class, status_badge_class, status_text) = match w.status.as_str() {
        "pending" => (
            "sf-status sf-status--pending bg-[var(--muted)]",
            "border-[var(--border)] bg-[var(--surface-alt)] text-[var(--muted)]",
            wish_t::STATUS_PENDING,
        ),
        "approved" => (
            "sf-status sf-status--running animate-pulse bg-[var(--primary)]",
            "border-[var(--primary)]/30 bg-[var(--primary-soft,rgba(201,100,66,0.12))] \
             text-[var(--primary)]",
            wish_t::STATUS_APPROVED,
        ),
        "running" => (
            "sf-status sf-status--running animate-pulse bg-[var(--primary)]",
            "border-[var(--primary)]/30 bg-[var(--primary-soft,rgba(201,100,66,0.12))] \
             text-[var(--primary)]",
            wish_t::STATUS_RUNNING,
        ),
        "done" => (
            "sf-status sf-status--done bg-[var(--ok,#4E7A51)]",
            "border-[var(--ok,#4E7A51)]/30 bg-[var(--ok-soft,rgba(78,122,81,0.10))] \
             text-[var(--ok,#4E7A51)]",
            wish_t::STATUS_DONE,
        ),
        "failed" => (
            "sf-status sf-status--failed bg-[var(--err,#B8443A)]",
            "border-[var(--err,#B8443A)]/30 bg-[var(--err-soft,rgba(184,68,58,0.10))] \
             text-[var(--err,#B8443A)]",
            wish_t::STATUS_FAILED,
        ),
        other => (
            "sf-status bg-[var(--muted)]",
            "border-[var(--border)] bg-[var(--surface-alt)] text-[var(--muted)]",
            other,
        ),
    };

    let ts = format_ts_ms(w.created_at);

    let on_fill = {
        let cb = props.on_fill_form.clone();
        let wish = w.clone();
        Callback::from(move |_: MouseEvent| cb.emit(wish.clone()))
    };

    let on_toggle_log = {
        let log_open = log_open.clone();
        let ai_log_loading = ai_log_loading.clone();
        let ai_log_fetched = ai_log_fetched.clone();
        let ai_log_text = ai_log_text.clone();
        let wish_id = w.wish_id.clone();
        Callback::from(move |_: MouseEvent| {
            let next_open = !*log_open;
            log_open.set(next_open);
            if next_open && !*ai_log_fetched && !*ai_log_loading {
                ai_log_loading.set(true);
                let ai_log_loading = ai_log_loading.clone();
                let ai_log_fetched = ai_log_fetched.clone();
                let ai_log_text = ai_log_text.clone();
                let wish_id = wish_id.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(output) = api::fetch_admin_music_wish_ai_output(&wish_id).await {
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
            lines.push(format!("ingested_song_id={sid}"));
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

    let done_song_actions = if w.status == "done" {
        if let Some(ref song_id) = w.ingested_song_id {
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
                        class="inline-flex items-center gap-1 rounded-full border border-[var(--primary)]/40 bg-[var(--primary-soft,rgba(201,100,66,0.12))] px-2.5 py-0.5 font-mono text-[11px] font-medium text-[var(--primary)] transition-colors hover:bg-[var(--primary)] hover:text-[var(--on-primary,#FAF9F5)]"
                    >
                        <Icon name={if props.is_playing { IconName::Pause } else { IconName::Play }} size={10} />
                        <span>{if props.is_playing { "暂停" } else { "播放" }}</span>
                    </button>
                    <Link<Route>
                        to={Route::MusicPlayer { id: song_id.clone() }}
                        classes="font-mono text-[11px] text-[var(--primary)] no-underline hover:underline"
                    >
                        {wish_t::LISTEN_NOW}
                    </Link<Route>>
                </>
            }
        } else {
            Html::default()
        }
    } else {
        Html::default()
    };

    html! {
        <div class="px-5 py-3.5 transition-colors hover:bg-[var(--surface-alt)]/50">
            <div class="flex flex-wrap items-start justify-between gap-2">
                <div class="min-w-0 flex-1">
                    <div class="flex flex-wrap items-center gap-2">
                        <span class={classes!(
                            "inline-flex",
                            "items-center",
                            "gap-1.5",
                            "rounded-full",
                            "border",
                            "px-2",
                            "py-0.5",
                            "font-mono",
                            "text-[10px]",
                            "leading-tight",
                            status_badge_class
                        )}>
                            <span class={classes!("h-1.5", "w-1.5", "rounded-full", status_dot_class)} />
                            <span>{status_text}</span>
                        </span>

                        <h3 class="truncate font-serif text-[1rem] font-normal text-[var(--text)]">
                            {&w.song_name}
                        </h3>

                        if let Some(ref artist) = w.artist_hint {
                            if !artist.trim().is_empty() {
                                <span class="truncate text-xs text-[var(--muted)]">
                                    {format!("— {artist}")}
                                </span>
                            }
                        }
                    </div>
                </div>

                <div class="flex shrink-0 flex-wrap items-center gap-1.5">
                    {done_song_actions}

                    <button
                        type="button"
                        onclick={on_toggle_log}
                        class="rounded-[6px] border border-[var(--border)] px-2 py-0.5 font-mono text-[11px] text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)]"
                    >
                        {if *log_open { "收起日志" } else { "AI 日志" }}
                    </button>

                    <button
                        type="button"
                        onclick={on_fill}
                        title={wish_t::FILL_FORM}
                        class="rounded-[6px] px-1.5 py-0.5 font-mono text-[11px] text-[var(--muted)] transition-colors hover:bg-[var(--surface-alt)] hover:text-[var(--primary)]"
                    >
                        {wish_t::FILL_FORM}
                    </button>
                </div>
            </div>

            <p class="mt-1.5 line-clamp-2 text-xs leading-relaxed text-[var(--text-2,var(--text))]">
                {&w.wish_message}
            </p>

            <div class="mt-2 flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] tabular-nums text-[var(--muted)]">
                <span class="truncate">
                    {if w.ip_region.trim().is_empty() {
                        w.nickname.clone()
                    } else {
                        format!("{} · {}", w.nickname, w.ip_region)
                    }}
                </span>
                <span>{&ts}</span>
            </div>

            if *log_open {
                <div class="mt-2.5 max-h-64 overflow-auto whitespace-pre-wrap rounded-[8px] border border-[var(--border)] bg-[var(--surface-sunk,var(--bg))] p-3.5 font-mono text-[0.8125rem] leading-[1.6] text-[var(--text-2,var(--text))]">
                    <div class="mb-1.5 flex items-center justify-between border-b border-[var(--border)] pb-1 text-[10px] uppercase tracking-[0.12em] text-[var(--muted)]">
                        <span>{"◆ AI EXECUTION LOG"}</span>
                        if *ai_log_loading {
                            <span>{"loading..."}</span>
                        }
                    </div>
                    {
                        if let Some(ref full_log) = *ai_log_text {
                            full_log.as_str()
                        } else {
                            fallback_log.as_str()
                        }
                    }
                </div>
            }
        </div>
    }
}

fn format_ts_ms(ms: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64));
    let y = date.get_full_year();
    let m = date.get_month() + 1;
    let d = date.get_date();
    let h = date.get_hours();
    let min = date.get_minutes();
    format!("{y:04}-{m:02}-{d:02} {h:02}:{min:02}")
}
