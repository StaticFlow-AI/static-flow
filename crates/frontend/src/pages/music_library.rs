use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    components::{
        form::{FormField, TextArea, TextInput},
        icons::{Icon, IconName},
        image_with_loading::ImageWithLoading,
        pagination::Pagination,
        skeleton::SkeletonSongCard,
        toast::use_toast,
    },
    i18n::current::{header as header_t, music_wish as wish_t},
    music_context::{MusicAction, MusicPlayerContext},
    router::Route,
};

const PAGE_SIZE: usize = 20;
const RANDOM_RECOMMEND_LIMIT: usize = 10;
const WISH_PAGE_SIZE: usize = 12;

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

    let page_songs = use_state(Vec::<api::SongListItem>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let random_songs = use_state(Vec::<api::SongListItem>::new);
    let random_loading = use_state(|| true);
    let random_error = use_state(|| None::<String>);
    let random_refresh_tick = use_state(|| 0_u64);
    let display_mode = use_state(|| LibraryDisplayMode::RandomRecommended);
    let active_artist = use_state(|| initial_query.artist.clone());
    let active_album = use_state(|| initial_query.album.clone());
    let current_page = use_state(|| 1_usize);
    let total = use_state(|| 0_usize);
    let all_songs_request_seq = use_mut_ref(|| 0_u64);
    let random_request_seq = use_mut_ref(|| 0_u64);
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
    let toast = use_toast();

    // Hero search state
    let hero_query = use_state(String::new);
    let hero_focused = use_state(|| false);

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
            if let Some(window) = web_sys::window() {
                if let Ok(history) = window.history() {
                    let _ =
                        history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
                    if let Ok(event) = web_sys::Event::new("popstate") {
                        let _ = window.dispatch_event(&event);
                    }
                }
            }
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

    let on_hero_focus = {
        let hero_focused = hero_focused.clone();
        Callback::from(move |_: FocusEvent| hero_focused.set(true))
    };

    let on_hero_blur = {
        let hero_focused = hero_focused.clone();
        Callback::from(move |_: FocusEvent| hero_focused.set(false))
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
        Callback::from(move |w: api::MusicWishItem| {
            wish_form_song.set(w.song_name.clone());
            wish_form_artist.set(w.artist_hint.unwrap_or_default());
            wish_form_message.set(w.wish_message.clone());
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("music-wish-form") {
                    el.scroll_into_view();
                }
            }
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
            let toast = toast.clone();
            let wish_page = wish_page.clone();
            let wish_refresh_tick = wish_refresh_tick.clone();
            let wish_form_song = wish_form_song.clone();
            let wish_form_artist = wish_form_artist.clone();
            let wish_form_message = wish_form_message.clone();
            let wish_form_email = wish_form_email.clone();
            wish_submitting.set(true);
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
                    Ok(_) => {
                        toast.success(wish_t::SUBMIT_SUCCESS);
                        wish_form_song.set(String::new());
                        wish_form_artist.set(String::new());
                        wish_form_message.set(String::new());
                        wish_form_email.set(String::new());
                        wish_page.set(1);
                        wish_refresh_tick.set(*wish_refresh_tick + 1);
                    },
                    Err(e) => {
                        toast.error(e);
                    },
                }
                wish_submitting.set(false);
            });
        })
    };

    // Segmented mode switch: active segment lifts onto the base surface,
    // inactive segments stay quiet until hovered.
    let segment_class = |active: bool| {
        classes!(
            "inline-flex",
            "items-center",
            "rounded-[var(--radius-sm)]",
            "px-3.5",
            "py-1.5",
            "text-sm",
            "font-medium",
            "transition-colors",
            "duration-[var(--motion-fast)]",
            "ease-[var(--ease-snap)]",
            if active {
                "bg-[var(--surface)] text-[var(--text)] shadow-[var(--shadow-sm)]"
            } else {
                "text-[var(--muted)] hover:text-[var(--text)]"
            }
        )
    };
    let filter_chip_class = "inline-flex items-center gap-1.5 rounded-full border \
                             border-[var(--primary)]/25 bg-[var(--primary)]/10 py-1 pl-3 pr-1 \
                             text-xs font-medium text-[var(--primary)]";
    let filter_chip_clear_class = "inline-flex h-5 w-5 items-center justify-center rounded-full \
                                   transition-colors duration-[var(--motion-fast)] \
                                   ease-[var(--ease-snap)] hover:bg-[var(--primary)]/15";
    let song_grid_class =
        "grid grid-cols-2 gap-4 sm:grid-cols-3 md:gap-5 lg:grid-cols-4 xl:grid-cols-5";

    html! {
        <div class="mx-auto max-w-7xl px-4 pb-16 sm:px-6 lg:px-8">
            <header class="sf-page-hero flex flex-wrap items-end justify-between gap-4">
                <div>
                    <div class="sf-eyebrow">{ "AUDIO ARCHIVE" }</div>
                    <h1 class="sf-page-title">
                        {"Music Library"}
                    </h1>
                    <p class="sf-page-sub">
                        {"Explore and play the music collection"}
                    </p>
                </div>
                <Link<Route>
                    to={Route::MediaImage}
                    classes="btn-terminal"
                >
                    <i class="fas fa-image text-xs text-[var(--primary)]" aria-hidden="true"></i>
                    { header_t::IMAGE_LIBRARY_TITLE }
                </Link<Route>>
            </header>

            <div class="mb-4 flex flex-wrap items-center gap-2">
                <div class="inline-flex items-center gap-1 rounded-[var(--radius)] border border-[var(--border)] \
                            bg-[var(--surface-alt)] p-1">
                    <button type="button"
                        onclick={on_switch_random}
                        class={segment_class(*display_mode == LibraryDisplayMode::RandomRecommended)}>
                        {"Random 10 Picks"}
                    </button>
                    <button type="button"
                        onclick={on_switch_all}
                        class={segment_class(*display_mode == LibraryDisplayMode::AllSongs)}>
                        {"View All Songs"}
                    </button>
                </div>
                if *display_mode == LibraryDisplayMode::RandomRecommended {
                    <button type="button"
                        onclick={on_refresh_random}
                        class="sf-chip cursor-pointer">
                        <i class="fas fa-shuffle text-[10px]" aria-hidden="true"></i>
                        {"Refresh Random"}
                    </button>
                }
            </div>

            // Hero search box
            <div class={classes!(
                "music-search-hero",
                hero_focused.then_some("focused"),
            )}>
                <div class="music-search-hero-inner">
                    <i class="fas fa-music music-search-icon" />
                    <input type="text"
                        placeholder="搜索歌曲、歌手或专辑..."
                        class="music-search-input"
                        value={(*hero_query).clone()}
                        oninput={on_hero_input}
                        onkeypress={on_hero_keypress}
                        onfocus={on_hero_focus}
                        onblur={on_hero_blur}
                    />
                    <button class="music-search-btn" onclick={on_hero_search} type="button">
                        <i class="fas fa-search" />
                    </button>
                </div>
            </div>

            if *display_mode == LibraryDisplayMode::AllSongs && (active_artist.is_some() || active_album.is_some()) {
                <div class="mb-4 flex flex-wrap gap-2">
                    if let Some(ref artist) = *active_artist {
                        <span class={filter_chip_class}>
                            {format!("Artist: {}", artist)}
                            <button onclick={clear_artist.clone()} type="button"
                                class={filter_chip_clear_class}>
                                <Icon name={IconName::X} size={12} />
                            </button>
                        </span>
                    }
                    if let Some(ref album) = *active_album {
                        <span class={filter_chip_class}>
                            {format!("Album: {}", album)}
                            <button onclick={clear_album.clone()} type="button"
                                class={filter_chip_clear_class}>
                                <Icon name={IconName::X} size={12} />
                            </button>
                        </span>
                    }
                </div>
            }

            if *display_mode == LibraryDisplayMode::RandomRecommended {
                if *random_loading {
                    <div class={song_grid_class}>
                        { for (0..RANDOM_RECOMMEND_LIMIT).map(|_| html! { <SkeletonSongCard /> }) }
                    </div>
                } else if let Some(ref err) = *random_error {
                    <div class="sf-state sf-state--error">
                        <span class="sf-state__icon">
                            <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                        </span>
                        <p class="sf-state__body">{format!("Failed to load random picks: {}", err)}</p>
                    </div>
                } else if random_songs.is_empty() {
                    <div class="sf-state">
                        <span class="sf-state__icon">
                            <i class="fas fa-compact-disc" aria-hidden="true"></i>
                        </span>
                        <p class="sf-state__body">{"No random picks available"}</p>
                    </div>
                } else {
                    <p class="mb-4 font-mono text-xs tracking-[0.02em] text-[var(--muted)]">
                        {"Showing 10 random recommendations. Click \"Refresh Random\" for a different set."}
                    </p>
                    <div class={song_grid_class}>
                        { for random_songs.iter().map(|song| {
                            render_song_card(song, &on_artist_click, &on_album_click)
                        })}
                    </div>
                }
            } else {
                if *loading {
                    <div class={song_grid_class}>
                        { for (0..PAGE_SIZE).map(|_| html! { <SkeletonSongCard /> }) }
                    </div>
                } else if let Some(ref err) = *error {
                    <div class="sf-state sf-state--error">
                        <span class="sf-state__icon">
                            <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                        </span>
                        <p class="sf-state__body">{format!("Failed to load: {}", err)}</p>
                    </div>
                } else if page_songs.is_empty() {
                    <div class="sf-state">
                        <span class="sf-state__icon">
                            <i class="fas fa-music" aria-hidden="true"></i>
                        </span>
                        <p class="sf-state__body">{"No music found"}</p>
                    </div>
                } else {
                    <div class={song_grid_class}>
                        { for page_songs.iter().map(|song| {
                            render_song_card(song, &on_artist_click, &on_album_click)
                        })}
                    </div>
                    if total_pages > 1 {
                        <div class="mt-10 flex justify-center">
                            <Pagination
                                current_page={*current_page}
                                total_pages={total_pages}
                                on_page_change={on_page_change.clone()}
                            />
                        </div>
                    }
                }
            }

            // Wish board section
            <section class="mt-16 border-t border-[var(--border)] pt-10">
                <div class="mb-6">
                    <h2 class="text-xl font-bold tracking-[-0.02em] text-[var(--text)] md:text-2xl">
                        {wish_t::SECTION_TITLE}
                    </h2>
                    <p class="mt-1 max-w-2xl text-sm leading-relaxed text-[var(--muted)]">{wish_t::SECTION_SUBTITLE}</p>
                </div>

                <form id="music-wish-form" onsubmit={on_wish_submit}
                    class="music-wish-form mb-8 grid scroll-mt-24 grid-cols-1 gap-4 p-5 sm:grid-cols-2 sm:gap-5 sm:p-6">
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
                    <div class="flex items-end sm:justify-end">
                        <button type="submit" disabled={*wish_submitting}
                            class="btn-fluent-primary w-full disabled:cursor-not-allowed disabled:opacity-50 sm:w-auto">
                            if *wish_submitting {
                                <span class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-white/40 \
                                             border-t-white motion-reduce:animate-none" aria-hidden="true" />
                            } else {
                                <i class="fas fa-paper-plane text-xs" aria-hidden="true"></i>
                            }
                            {if *wish_submitting { wish_t::SUBMITTING } else { wish_t::SUBMIT_BTN }}
                        </button>
                    </div>
                </form>

                // Refresh button
                <div class="mb-4 flex justify-end">
                    <button
                        onclick={on_refresh_wishes}
                        disabled={*wish_loading}
                        class="btn-fluent-secondary disabled:cursor-not-allowed disabled:opacity-50"
                    >
                        <svg class={if *wish_loading { "h-4 w-4 animate-spin motion-reduce:animate-none" } else { "h-4 w-4" }}
                             xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
                             stroke-width="2" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round"
                                  d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 \
                                     3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 \
                                     13.803-3.7l3.181 3.182" />
                        </svg>
                        {if *wish_loading { wish_t::REFRESHING } else { wish_t::REFRESH_BTN }}
                    </button>
                </div>

                if *wish_loading && wishes.is_empty() {
                    <div class="flex justify-center py-10">
                        <div class="h-6 w-6 animate-spin rounded-full border-2 border-[var(--border)] \
                                    border-t-[var(--primary)] motion-reduce:animate-none" />
                    </div>
                } else if let Some(err) = (*wish_list_error).clone() {
                    <div class="sf-state sf-state--error">
                        <span class="sf-state__icon">
                            <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                        </span>
                        <p class="sf-state__body">{format!("Failed to load wishes: {err}")}</p>
                    </div>
                } else if wishes.is_empty() {
                    <div class="sf-state">
                        <span class="sf-state__icon">
                            <i class="fas fa-compact-disc" aria-hidden="true"></i>
                        </span>
                        <p class="sf-state__body">{wish_t::EMPTY_LIST}</p>
                    </div>
                } else {
                    <>
                        if *wish_loading {
                            <div class="mb-3 inline-flex items-center gap-2 text-xs text-[var(--muted)]">
                                <div class="h-4 w-4 animate-spin rounded-full border-2 border-[var(--border)] \
                                            border-t-[var(--primary)] motion-reduce:animate-none" />
                                <span>{"Loading wishes..."}</span>
                            </div>
                        }
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                            { for wishes.iter().map(|w| html! {
                                <WishCard key={w.wish_id.clone()} wish={w.clone()} on_fill_form={on_wish_fill_form.clone()} />
                            }) }
                        </div>
                    </>
                }
                if wish_total_pages > 1 {
                    <div class="mt-8 flex justify-center">
                        <Pagination
                            current_page={*wish_page}
                            total_pages={wish_total_pages}
                            on_page_change={on_wish_page_change.clone()}
                        />
                    </div>
                }
            </section>
        </div>
    }
}

fn render_song_card(
    song: &api::SongListItem,
    on_artist_click: &dyn Fn(String) -> Callback<MouseEvent>,
    on_album_click: &dyn Fn(String) -> Callback<MouseEvent>,
) -> Html {
    let cover_url = api::song_cover_url(song.cover_image.as_deref());
    let duration = format_duration(song.duration_ms);
    let id = song.id.clone();
    let artist = song.artist.clone();
    let album = song.album.clone();
    let artist_cb = on_artist_click(artist.clone());
    let album_cb = on_album_click(album.clone());

    html! {
        <div class="group flex flex-col overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border)] \
                    bg-[var(--surface)] shadow-[var(--shadow-sm)] \
                    transition-[transform,box-shadow,border-color] duration-[var(--motion-base)] ease-[var(--ease-spring)] \
                    hover:-translate-y-1 hover:border-[var(--primary)]/50 hover:shadow-[var(--shadow-8)] \
                    motion-reduce:transition-none motion-reduce:hover:translate-y-0">
            <Link<Route> to={Route::MusicPlayer { id }}>
                <div class="relative aspect-square overflow-hidden bg-[var(--surface-alt)]">
                    if cover_url.is_empty() {
                        <div class="flex h-full w-full items-center justify-center text-[var(--muted)]">
                            <Icon name={IconName::Music} size={48} class={classes!("opacity-30")} />
                        </div>
                    } else {
                        <ImageWithLoading
                            src={cover_url}
                            alt={song.title.clone()}
                            loading={Some(AttrValue::from("lazy"))}
                            referrerpolicy={Some(AttrValue::from("no-referrer"))}
                            class="h-full w-full object-cover transition-transform duration-500 ease-[var(--ease-spring)] \
                                   group-hover:scale-[1.04] motion-reduce:transition-none motion-reduce:group-hover:scale-100"
                            container_class={classes!("w-full", "h-full")}
                        />
                    }
                    <div class="absolute inset-0 flex items-center justify-center bg-black/30 opacity-0 \
                                transition-opacity duration-[var(--motion-base)] ease-[var(--ease-snap)] \
                                group-hover:opacity-100 group-focus-within:opacity-100">
                        <div class="flex h-12 w-12 scale-90 items-center justify-center rounded-full bg-white/90 \
                                    shadow-[var(--shadow-8)] transition-transform duration-[var(--motion-base)] \
                                    ease-[var(--ease-spring)] group-hover:scale-100 motion-reduce:scale-100">
                            <Icon name={IconName::Play} size={20} color="#000" />
                        </div>
                    </div>
                    <div class="absolute bottom-2 right-2 rounded-md bg-black/60 px-1.5 py-0.5 font-mono \
                                text-[11px] tabular-nums text-white backdrop-blur-sm">
                        {&duration}
                    </div>
                </div>
            </Link<Route>>
            <div class="flex flex-col gap-1.5 border-t border-[var(--border)] px-3 py-2.5">
                <h3 class="truncate font-sans text-sm font-semibold leading-tight text-[var(--text)]">
                    {&song.title}
                </h3>
                <a href="#" onclick={artist_cb}
                    class="cursor-pointer truncate text-xs text-[var(--muted)] no-underline transition-colors \
                           duration-[var(--motion-fast)] ease-[var(--ease-snap)] hover:text-[var(--primary)]">
                    {&song.artist}
                </a>
                if !song.album.is_empty() {
                    <a href="#" onclick={album_cb}
                        class="block max-w-full self-start truncate rounded-full border border-[var(--border)] \
                               bg-[var(--surface-alt)] px-2 py-0.5 text-[10px] text-[var(--muted)] no-underline \
                               transition-colors duration-[var(--motion-fast)] ease-[var(--ease-snap)] \
                               hover:border-[var(--primary)]/50 hover:text-[var(--primary)]">
                        {&song.album}
                    </a>
                }
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
struct WishCardProps {
    pub wish: api::MusicWishItem,
    pub on_fill_form: Callback<api::MusicWishItem>,
}

#[function_component(WishCard)]
fn wish_card(props: &WishCardProps) -> Html {
    let w = &props.wish;

    let status_class = match w.status.as_str() {
        "pending" => "wish-status wish-status--pending",
        "approved" | "running" => "wish-status wish-status--running",
        "done" => "wish-status wish-status--done",
        "failed" => "wish-status wish-status--failed",
        _ => "wish-status wish-status--default",
    };
    let status_text = match w.status.as_str() {
        "pending" => wish_t::STATUS_PENDING,
        "approved" => wish_t::STATUS_APPROVED,
        "running" => wish_t::STATUS_RUNNING,
        "done" => wish_t::STATUS_DONE,
        "failed" => wish_t::STATUS_FAILED,
        other => other,
    };

    let ts = format_ts_ms(w.created_at);

    let on_fill = {
        let cb = props.on_fill_form.clone();
        let wish = w.clone();
        Callback::from(move |_: MouseEvent| cb.emit(wish.clone()))
    };

    html! {
        <div class="sf-panel flex flex-col gap-2.5 p-4 transition-[transform,box-shadow,border-color] \
                    duration-[var(--motion-base)] ease-[var(--ease-spring)] \
                    hover:-translate-y-0.5 hover:border-[var(--primary)]/40 hover:shadow-[var(--shadow-4)] \
                    motion-reduce:transition-none motion-reduce:hover:translate-y-0">
            <div class="flex items-start justify-between gap-2">
                <h3 class="truncate text-sm font-semibold text-[var(--text)]">{&w.song_name}</h3>
                <span class={classes!("text-[10px]", "font-medium", "leading-tight", "px-2", "py-0.5",
                    "rounded-full", "border", "whitespace-nowrap", "shrink-0", status_class)}>
                    {status_text}
                </span>
            </div>
            if let Some(ref artist) = w.artist_hint {
                <p class="text-xs text-[var(--muted)]">{format!("🎤 {}", artist)}</p>
            }
            <p class="line-clamp-3 text-xs leading-relaxed text-[var(--text)]">{&w.wish_message}</p>
            <div class="mt-auto flex items-center justify-between gap-2 border-t border-[var(--border)] \
                        pt-2.5 text-[10px] text-[var(--muted)]">
                <span class="truncate">{format!("{} · {}", w.nickname, w.ip_region)}</span>
                <div class="flex shrink-0 items-center gap-2">
                    <button onclick={on_fill}
                        class="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[var(--muted)] \
                               transition-colors duration-[var(--motion-fast)] ease-[var(--ease-snap)] \
                               hover:bg-[var(--primary)]/10 hover:text-[var(--primary)]"
                        title={wish_t::FILL_FORM}>
                        <svg class="h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none"
                             viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round"
                                  d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10" />
                        </svg>
                        <span>{wish_t::FILL_FORM}</span>
                    </button>
                    <span class="font-mono tabular-nums">{&ts}</span>
                </div>
            </div>
            if w.status == "done" {
                if let Some(ref song_id) = w.ingested_song_id {
                    <Link<Route> to={Route::MusicPlayer { id: song_id.clone() }}
                        classes="inline-flex items-center gap-1 self-start text-xs font-medium \
                                 text-[var(--primary)] underline-offset-2 hover:underline">
                        <i class="fas fa-play text-[9px]" aria-hidden="true"></i>
                        {wish_t::LISTEN_NOW}
                    </Link<Route>>
                }
            }
            if let Some(ref reply) = w.ai_reply {
                <div class="mt-1 whitespace-pre-wrap rounded-[var(--radius)] border border-[var(--border)] \
                            bg-[var(--surface-alt)] p-2.5 text-xs leading-relaxed text-[var(--text)]">
                    <span class="mb-1 block text-[10px] font-medium uppercase tracking-[0.12em] text-[var(--muted)]">{"🤖 AI"}</span>
                    {reply}
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
