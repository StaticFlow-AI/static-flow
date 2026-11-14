use wasm_bindgen::JsCast;
use web_sys::{window, Event, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    components::{
        icons::{Icon, IconName},
        image_with_loading::ImageWithLoading,
        persistent_audio::{preview_next_song, resolve_next_song},
        synced_lyrics::SyncedLyrics,
    },
    music_context::{MusicAction, MusicPlayerContext, NextSongMode},
    router::Route,
};

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(MusicPlayerPage)]
pub fn music_player_page(props: &Props) -> Html {
    let id = props.id.clone();
    let song = use_state(|| None::<api::SongDetail>);
    let lyrics = use_state(|| None::<api::SongLyrics>);
    let comments = use_state(Vec::<api::MusicCommentItem>::new);
    let song_loading = use_state(|| true);
    let lyrics_loading = use_state(|| true);
    let comments_loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let nickname = use_state(String::new);
    let comment_text = use_state(String::new);
    let submitting = use_state(|| false);
    let submit_error = use_state(|| None::<String>);
    let next_hint = use_state(|| None::<api::SongDetail>);
    let next_hint_loading = use_state(|| false);
    let bottom_recommendations = use_state(Vec::<api::SongSearchResult>::new);
    let bottom_recs_loading = use_state(|| false);
    let bottom_recs_ready = use_state(|| false);
    let player_ctx = use_context::<MusicPlayerContext>();
    let navigator = use_navigator();
    let current_time = player_ctx.as_ref().map(|c| c.current_time).unwrap_or(0.0);
    let lyrics_offset = player_ctx.as_ref().map(|c| c.lyrics_offset).unwrap_or(0.0);

    // Navigate when global song_id changes (prev/next/auto-next)
    {
        let navigator = navigator.clone();
        let id = id.clone();
        let ctx_song_id = player_ctx.as_ref().and_then(|c| c.song_id.clone());
        use_effect_with(ctx_song_id, move |ctx_song_id| {
            if let Some(ref new_id) = ctx_song_id {
                if *new_id != id {
                    if let Some(ref nav) = navigator {
                        nav.replace(&Route::MusicPlayer {
                            id: new_id.clone(),
                        });
                    }
                }
            }
            || ()
        });
    }

    // Fetch song detail (highest priority)
    {
        let id = id.clone();
        let song = song.clone();
        let song_loading = song_loading.clone();
        let error = error.clone();
        let player_ctx = player_ctx.clone();
        use_effect_with(id.clone(), move |id| {
            let id = id.clone();
            error.set(None);
            let ctx_hit = player_ctx.as_ref().and_then(|c| {
                if c.song_id.as_deref() == Some(id.as_str()) {
                    c.current_song.clone()
                } else {
                    None
                }
            });
            if let Some(cached) = ctx_hit {
                if let Some(ref ctx) = player_ctx {
                    ctx.dispatch(MusicAction::Expand);
                }
                song.set(Some(cached));
                song_loading.set(false);
                let id2 = id.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = api::track_song_play(&id2).await;
                });
            } else {
                song_loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_song_detail(&id).await {
                        Ok(Some(d)) => {
                            if let Some(ref ctx) = player_ctx {
                                ctx.dispatch(MusicAction::PlaySong {
                                    song: d.clone(),
                                    id: id.clone(),
                                });
                            }
                            song.set(Some(d));
                        },
                        Ok(None) => {
                            error.set(Some("Song not found".to_string()));
                        },
                        Err(e) => {
                            error.set(Some(e));
                        },
                    }
                    song_loading.set(false);
                    let id2 = id.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = api::track_song_play(&id2).await;
                    });
                });
            }
            || ()
        });
    }

    // Fetch lyrics (independent)
    {
        let id = id.clone();
        let lyrics = lyrics.clone();
        let lyrics_loading = lyrics_loading.clone();
        use_effect_with(id.clone(), move |id| {
            let id = id.clone();
            lyrics_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(Some(l)) = api::fetch_song_lyrics(&id).await {
                    lyrics.set(Some(l));
                } else {
                    lyrics.set(None);
                }
                lyrics_loading.set(false);
            });
            || ()
        });
    }

    // Fetch comments (independent)
    {
        let id = id.clone();
        let comments = comments.clone();
        let comments_loading = comments_loading.clone();
        use_effect_with(id.clone(), move |id| {
            let id = id.clone();
            comments_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(c) = api::fetch_music_comments(&id, Some(50), None).await {
                    comments.set(c);
                }
                comments_loading.set(false);
            });
            || ()
        });
    }

    // Prefetch next-song hint when playback context changes.
    {
        let next_hint = next_hint.clone();
        let next_hint_loading = next_hint_loading.clone();
        let player_ctx = player_ctx.clone();
        let deps = (
            player_ctx.as_ref().and_then(|ctx| ctx.song_id.clone()),
            player_ctx.as_ref().map(|ctx| ctx.next_mode.clone()),
            player_ctx
                .as_ref()
                .map(|ctx| {
                    ctx.history
                        .iter()
                        .rev()
                        .take(10)
                        .map(|(song_id, _)| song_id.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        );
        use_effect_with(deps, move |_| {
            if let Some(ctx) = player_ctx.clone() {
                if ctx.song_id.is_none() {
                    next_hint.set(None);
                    next_hint_loading.set(false);
                } else {
                    next_hint_loading.set(true);
                    wasm_bindgen_futures::spawn_local(async move {
                        let next = preview_next_song(&ctx).await.map(|(song, _)| song);
                        next_hint.set(next);
                        next_hint_loading.set(false);
                    });
                }
            } else {
                next_hint.set(None);
                next_hint_loading.set(false);
            }
            || ()
        });
    }

    // Bottom recommendations are lazy-loaded when user scrolls near page end.
    {
        let id = id.clone();
        let bottom_recommendations = bottom_recommendations.clone();
        let bottom_recs_loading = bottom_recs_loading.clone();
        let bottom_recs_ready = bottom_recs_ready.clone();
        use_effect_with(id, move |_| {
            bottom_recommendations.set(Vec::new());
            bottom_recs_loading.set(false);
            bottom_recs_ready.set(false);
            || ()
        });
    }
    {
        let bottom_recs_ready = bottom_recs_ready.clone();
        let ready = *bottom_recs_ready;
        let id = id.clone();
        use_effect_with((id, ready), move |_| {
            let mut on_scroll_opt: Option<wasm_bindgen::closure::Closure<dyn FnMut(Event)>> = None;
            if !ready {
                if should_load_bottom_recommendations() {
                    bottom_recs_ready.set(true);
                } else if let Some(win) = window() {
                    let ready_setter = bottom_recs_ready.clone();
                    let on_scroll =
                        wasm_bindgen::closure::Closure::wrap(Box::new(move |_: Event| {
                            if should_load_bottom_recommendations() {
                                ready_setter.set(true);
                            }
                        })
                            as Box<dyn FnMut(_)>);
                    let _ = win.add_event_listener_with_callback(
                        "scroll",
                        on_scroll.as_ref().unchecked_ref(),
                    );
                    let _ = win.add_event_listener_with_callback(
                        "resize",
                        on_scroll.as_ref().unchecked_ref(),
                    );
                    on_scroll_opt = Some(on_scroll);
                } else {
                    bottom_recs_ready.set(true);
                }
            }
            move || {
                if let Some(on_scroll) = on_scroll_opt {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "scroll",
                            on_scroll.as_ref().unchecked_ref(),
                        );
                        let _ = win.remove_event_listener_with_callback(
                            "resize",
                            on_scroll.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }
    {
        let id = id.clone();
        let bottom_recommendations = bottom_recommendations.clone();
        let bottom_recs_loading = bottom_recs_loading.clone();
        let ready = *bottom_recs_ready;
        use_effect_with((id, ready), move |(id, ready)| {
            if *ready {
                let id = id.clone();
                bottom_recs_loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let items = api::fetch_related_songs(&id).await.unwrap_or_default();
                    let filtered = items
                        .into_iter()
                        .filter(|item| item.id != id)
                        .take(4)
                        .collect::<Vec<_>>();
                    bottom_recommendations.set(filtered);
                    bottom_recs_loading.set(false);
                });
            }
            || ()
        });
    }

    let on_submit_comment = {
        let id = id.clone();
        let nickname = nickname.clone();
        let comment_text = comment_text.clone();
        let comments = comments.clone();
        let submitting = submitting.clone();
        let submit_error = submit_error.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let nick = (*nickname).clone();
            let text = (*comment_text).clone();
            if text.trim().is_empty() {
                return;
            }
            let nick_opt = if nick.trim().is_empty() { None } else { Some(nick.clone()) };
            let id = id.clone();
            let nickname = nickname.clone();
            let comment_text = comment_text.clone();
            let comments = comments.clone();
            let submitting = submitting.clone();
            let submit_error = submit_error.clone();
            submitting.set(true);
            submit_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match api::submit_music_comment(&id, nick_opt.as_deref(), &text).await {
                    Ok(nc) => {
                        let mut l = (*comments).clone();
                        l.insert(0, nc);
                        comments.set(l);
                        nickname.set(String::new());
                        comment_text.set(String::new());
                    },
                    Err(e) => {
                        submit_error.set(Some(e));
                    },
                }
                submitting.set(false);
            });
        })
    };
    let on_nickname_input = {
        let nickname = nickname.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                nickname.set(input.value());
            }
        })
    };
    let on_comment_input = {
        let comment_text = comment_text.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                comment_text.set(input.value());
            }
        })
    };

    let on_toggle_play = {
        let ctx = player_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::TogglePlay);
            }
        })
    };
    let can_prev = player_ctx
        .as_ref()
        .map(|c| c.history_index.map(|i| i > 0).unwrap_or(false))
        .unwrap_or(false);
    let on_prev = {
        let ctx = player_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::PlayPrev);
            }
        })
    };
    let on_next = {
        let ctx = player_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                let c2 = c.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fallback = resolve_next_song(&c2).await;
                    c2.dispatch(MusicAction::PlayNext {
                        fallback,
                    });
                });
            }
        })
    };
    let next_mode = player_ctx
        .as_ref()
        .map(|c| c.next_mode.clone())
        .unwrap_or(NextSongMode::Random);
    let on_set_random_mode = {
        let ctx = player_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::SetNextMode(NextSongMode::Random));
            }
        })
    };
    let on_set_playlist_mode = {
        let ctx = player_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::SetNextMode(NextSongMode::PlaylistSequential));
            }
        })
    };
    let on_set_semantic_mode = {
        let ctx = player_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::SetNextMode(NextSongMode::Semantic));
            }
        })
    };
    let is_random_mode = matches!(next_mode, NextSongMode::Random);
    let is_playlist_mode = matches!(next_mode, NextSongMode::PlaylistSequential);
    let is_semantic_mode = matches!(next_mode, NextSongMode::Semantic);
    let on_seek = {
        let ctx = player_ctx.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                if let Ok(v) = input.value().parse::<f64>() {
                    if let Some(ref c) = ctx {
                        c.dispatch(MusicAction::SetTime(v));
                    }
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        if let Ok(Some(el)) = doc.query_selector("audio") {
                            if let Some(audio) = el.dyn_ref::<web_sys::HtmlAudioElement>() {
                                audio.set_current_time(v);
                            }
                        }
                    }
                }
            }
        })
    };
    let on_volume = {
        let ctx = player_ctx.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                if let Ok(v) = input.value().parse::<f64>() {
                    if let Some(ref c) = ctx {
                        c.dispatch(MusicAction::SetVolume(v));
                    }
                }
            }
        })
    };
    let on_toggle_mute = {
        let ctx = player_ctx.clone();
        let cv = player_ctx.as_ref().map(|c| c.volume).unwrap_or(1.0);
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::SetVolume(if cv > 0.0 { 0.0 } else { 1.0 }));
            }
        })
    };
    let on_minimize = {
        let ctx = player_ctx.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::Minimize);
            }
            if let Some(ref nav) = navigator {
                nav.push(&Route::MediaAudio);
            }
        })
    };

    let on_back = {
        let ctx = player_ctx.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                if c.playing {
                    c.dispatch(MusicAction::Minimize);
                }
            }
            if let Some(w) = web_sys::window() {
                if w.history().ok().and_then(|h| h.length().ok()).unwrap_or(0) > 1 {
                    let _ = w.history().map(|h| h.back());
                    return;
                }
            }
            if let Some(ref nav) = navigator {
                nav.push(&Route::MediaAudio);
            }
        })
    };

    let on_offset_dec = {
        let ctx = player_ctx.clone();
        let off = lyrics_offset;
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::SetLyricsOffset(off - 0.5));
            }
        })
    };
    let on_offset_inc = {
        let ctx = player_ctx.clone();
        let off = lyrics_offset;
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::SetLyricsOffset(off + 0.5));
            }
        })
    };
    let on_offset_reset = {
        let ctx = player_ctx.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ref c) = ctx {
                c.dispatch(MusicAction::SetLyricsOffset(0.0));
            }
        })
    };
    let offset_display = if lyrics_offset > 0.0 {
        format!("+{:.1}s", lyrics_offset)
    } else if lyrics_offset < 0.0 {
        format!("{:.1}s", lyrics_offset)
    } else {
        "0.0s".to_string()
    };

    if *song_loading {
        return html! {
            <div class="mx-auto max-w-[1080px] px-4 py-12 sm:px-6 lg:px-8">
                <div class="grid grid-cols-1 gap-10 lg:grid-cols-[minmax(0,400px)_1fr] lg:gap-14">
                    <div class="aspect-square w-full max-w-[400px] animate-pulse rounded-[12px] border border-[var(--border)] bg-[var(--surface-alt)]" />
                    <div class="space-y-4 pt-2">
                        <div class="h-3 w-28 animate-pulse rounded bg-[var(--surface-alt)]" />
                        <div class="h-9 w-2/3 animate-pulse rounded bg-[var(--surface-alt)]" />
                        <div class="h-4 w-44 animate-pulse rounded bg-[var(--surface-alt)]" />
                        <div class="h-56 w-full animate-pulse rounded-[12px] border border-[var(--border)] bg-[var(--surface)]" />
                    </div>
                </div>
            </div>
        };
    }
    if let Some(err) = (*error).as_ref() {
        return html! {
            <div class="mx-auto max-w-xl px-4 py-20 text-center">
                <p class="font-serif text-xl font-normal text-[var(--err,#B8443A)]">{format!("无法加载曲目：{err}")}</p>
                <div class="mt-6">
                    <Link<Route>
                        to={Route::MediaAudio}
                        classes="sf-btn sf-btn--secondary inline-flex items-center gap-2 rounded-[8px] border border-[var(--border-strong,var(--border))] px-4 py-2 text-sm text-[var(--text)] no-underline hover:bg-[var(--surface-alt)]"
                    >
                        {"← 返回声学曲库"}
                    </Link<Route>>
                </div>
            </div>
        };
    }
    let detail = match (*song).as_ref() {
        Some(d) => d,
        None => {
            return html! {
                <div class="py-20 text-center text-sm text-[var(--muted)]">{"Song not found"}</div>
            };
        },
    };

    let cover_url = api::song_cover_url(detail.cover_image.as_deref());
    let audio_url = api::song_audio_url(&id);
    let duration_str = format_duration(detail.duration_ms);
    let lyrics_lrc = (*lyrics).as_ref().and_then(|l| l.lyrics_lrc.clone());
    let lyrics_trans = (*lyrics)
        .as_ref()
        .and_then(|l| l.lyrics_translation.clone());
    let playing = player_ctx.as_ref().map(|c| c.playing).unwrap_or(false);
    let duration = player_ctx
        .as_ref()
        .map(|c| c.duration)
        .filter(|d| *d > 0.0 && d.is_finite())
        .unwrap_or(detail.duration_ms as f64 / 1000.0);
    let volume = player_ctx.as_ref().map(|c| c.volume).unwrap_or(1.0);
    let progress_pct = if duration > 0.0 && duration.is_finite() {
        ((current_time / duration) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let artist_name = detail.artist.clone();
    let album_name = detail.album.clone();
    let on_artist_nav = {
        let navigator = navigator.clone();
        let artist = artist_name.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(ref nav) = navigator {
                let q = std::collections::HashMap::from([("artist", artist.as_str())]);
                let _ = nav.push_with_query(&Route::MediaAudio, &q);
            }
        })
    };
    let on_album_nav = {
        let navigator = navigator.clone();
        let album = album_name.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(ref nav) = navigator {
                let q = std::collections::HashMap::from([("album", album.as_str())]);
                let _ = nav.push_with_query(&Route::MediaAudio, &q);
            }
        })
    };

    let mode_btn_class = |active: bool| {
        classes!(
            "inline-flex",
            "h-8",
            "items-center",
            "gap-1.5",
            "rounded-full",
            "border",
            "px-2.5",
            "font-mono",
            "text-[11px]",
            "transition-colors",
            if active {
                "border-[var(--primary)]/40 bg-[var(--primary-soft,rgba(201,100,66,0.12))] \
                 text-[var(--primary)]"
            } else {
                "border-[var(--border)] bg-transparent text-[var(--muted)] \
                 hover:border-[var(--border-strong,var(--primary))] hover:text-[var(--text)]"
            }
        )
    };

    html! {
        <div class="mx-auto max-w-[1080px] px-4 py-8 sm:px-6 md:py-12 lg:px-8">
            // Top editorial navigation bar
            <div class="mb-8 flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border)] pb-4">
                <button
                    onclick={on_back}
                    type="button"
                    class="inline-flex items-center gap-2 font-mono text-xs text-[var(--muted)] transition-colors hover:text-[var(--text)]"
                >
                    <Icon name={IconName::ArrowLeft} size={14} />
                    <span>{"返回声学曲库"}</span>
                </button>
                <button
                    type="button"
                    onclick={Callback::from(|_: MouseEvent| {
                        crate::navigation_context::navigate_spa_to(
                            &crate::config::route_path("/studio#music")
                        );
                    })}
                    class="font-mono text-xs text-[var(--primary)] transition-colors hover:underline"
                >
                    {"想听别的歌？去共建台点歌 →"}
                </button>
            </div>

            // Two-column Editorial Player Layout
            <div class="grid grid-cols-1 gap-10 lg:grid-cols-[minmax(0,400px)_1fr] lg:items-start lg:gap-14">
                // Left column: Static square cover (no shadow, no vinyl spin) + Up Next
                <div class="mx-auto w-full max-w-[400px] lg:sticky lg:top-24 lg:mx-0">
                    <div class="aspect-square w-full overflow-hidden rounded-[12px] border border-[var(--border)] bg-[var(--surface-alt)]">
                        if cover_url.is_empty() {
                            <div class="flex h-full w-full items-center justify-center text-[var(--muted)]">
                                <Icon name={IconName::Music} size={64} class={classes!("opacity-30")} />
                            </div>
                        } else {
                            <ImageWithLoading
                                src={cover_url}
                                alt={detail.title.clone()}
                                referrerpolicy={Some(AttrValue::from("no-referrer"))}
                                loading={Some(AttrValue::from("eager"))}
                                class="h-full w-full object-cover"
                                container_class={classes!("w-full", "h-full")}
                            />
                        }
                    </div>

                    // Up Next card beneath cover
                    <div class="mt-5 rounded-[10px] border border-[var(--border)] bg-[var(--surface)] p-4">
                        if *next_hint_loading {
                            <p class="font-mono text-xs text-[var(--muted)]">{"正在预取下一首曲目..."}</p>
                        } else if let Some(next) = (*next_hint).as_ref() {
                            <div class="flex items-center justify-between gap-3">
                                <div class="min-w-0">
                                    <p class="font-mono text-[10px] uppercase tracking-[0.12em] text-[var(--muted)]">
                                        {"UP NEXT · 即将播放"}
                                    </p>
                                    <p class="mt-1 truncate font-serif text-[0.975rem] font-normal text-[var(--text)]">
                                        {&next.title}
                                    </p>
                                    <p class="mt-0.5 truncate text-xs text-[var(--muted)]">
                                        {if next.album.is_empty() {
                                            next.artist.clone()
                                        } else {
                                            format!("{} · {}", next.artist, next.album)
                                        }}
                                    </p>
                                </div>
                                <Link<Route>
                                    to={Route::MusicPlayer { id: next.id.clone() }}
                                    classes="shrink-0 rounded-[8px] border border-[var(--border-strong,var(--border))] bg-[var(--surface-alt)] px-3 py-1.5 font-mono text-xs text-[var(--text)] no-underline transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)]"
                                >
                                    {"下一首 →"}
                                </Link<Route>>
                            </div>
                        } else {
                            <p class="font-mono text-xs text-[var(--muted)]">
                                {"当前播放模式下暂无预设下一首。"}
                            </p>
                        }
                    </div>
                </div>

                // Right column: Serif header + slim 2px terracotta progress + synced lyrics + comments
                <div class="min-w-0">
                    <div class="sf-eyebrow mb-2 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--muted)]">
                        {"NOW PLAYING · AUDIO ARCHIVE"}
                    </div>
                    <h1 class="font-serif text-[clamp(1.875rem,3.4vw,2.625rem)] font-normal leading-[1.15] tracking-[-0.02em] text-[var(--text)]">
                        {&detail.title}
                    </h1>
                    <div class="mt-2.5 flex flex-wrap items-center gap-x-2 gap-y-1 text-[0.95rem] text-[var(--muted)]">
                        <a
                            href="#"
                            onclick={on_artist_nav}
                            class="cursor-pointer text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
                        >
                            {&detail.artist}
                        </a>
                        if !detail.album.is_empty() {
                            <span class="text-[var(--border-strong,var(--border))]">{"·"}</span>
                            <a
                                href="#"
                                onclick={on_album_nav}
                                class="cursor-pointer text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
                            >
                                {&detail.album}
                            </a>
                        }
                    </div>
                    <div class="mt-2 flex flex-wrap items-center gap-2 font-mono text-xs tabular-nums text-[var(--muted)]">
                        <span>{&duration_str}</span>
                        <span class="text-[var(--border-strong,var(--border))]">{"·"}</span>
                        <span class="uppercase">{&detail.format}</span>
                        if detail.bitrate > 0 {
                            <span class="text-[var(--border-strong,var(--border))]">{"·"}</span>
                            <span>{format!("{} kbps", detail.bitrate / 1000)}</span>
                        }
                    </div>

                    // Slim 2px (hover 4px + 10px circular thumb) terracotta progress bar & controls
                    <div class="mt-6 border-y border-[var(--border)] py-5">
                        <div class="group relative flex h-4 w-full items-center">
                            <div class="relative h-[2px] w-full overflow-hidden rounded-full bg-[var(--border-strong,var(--border))] transition-[height] duration-150 group-hover:h-1">
                                <div
                                    class="h-full bg-[var(--primary)]"
                                    style={format!("width: {progress_pct:.2}%")}
                                />
                            </div>
                            <div
                                class="pointer-events-none absolute top-1/2 h-2.5 w-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-[var(--primary)] opacity-0 transition-opacity duration-150 group-hover:opacity-100"
                                style={format!("left: {progress_pct:.2}%")}
                                aria-hidden="true"
                            />
                            <input
                                type="range"
                                min="0"
                                max={if duration.is_finite() && duration > 0.0 { duration.to_string() } else { "0".to_string() }}
                                step="0.1"
                                value={current_time.to_string()}
                                oninput={on_seek}
                                class="absolute inset-0 h-full w-full cursor-pointer opacity-0"
                                aria-label="Seek"
                            />
                        </div>

                        <div class="mt-1.5 flex items-center justify-between font-mono text-xs tabular-nums text-[var(--muted)]">
                            <span>{format_time(current_time)}</span>
                            <span>{format_time(duration)}</span>
                        </div>

                        <div class="mt-4 flex flex-wrap items-center justify-between gap-4">
                            // Transport controls
                            <div class="flex items-center gap-2.5">
                                <button
                                    onclick={on_prev}
                                    type="button"
                                    disabled={!can_prev}
                                    class="flex h-9 w-9 items-center justify-center rounded-full border border-[var(--border)] text-[var(--muted)] transition-colors hover:border-[var(--border-strong,var(--primary))] hover:text-[var(--text)] disabled:cursor-not-allowed disabled:opacity-30"
                                    aria-label="Previous song"
                                >
                                    <Icon name={IconName::SkipBack} size={15} />
                                </button>
                                <button
                                    onclick={on_toggle_play}
                                    type="button"
                                    class="flex h-11 w-11 shrink-0 items-center justify-center rounded-full bg-[var(--primary)] text-[var(--on-primary,#FAF9F5)] transition-colors hover:bg-[var(--primary-hover,#B85735)]"
                                    aria-label={if playing { "Pause" } else { "Play" }}
                                >
                                    <Icon name={if playing { IconName::Pause } else { IconName::Play }} size={18} />
                                </button>
                                <button
                                    onclick={on_next}
                                    type="button"
                                    class="flex h-9 w-9 items-center justify-center rounded-full border border-[var(--border)] text-[var(--muted)] transition-colors hover:border-[var(--border-strong,var(--primary))] hover:text-[var(--text)]"
                                    aria-label="Next song"
                                >
                                    <Icon name={IconName::SkipForward} size={15} />
                                </button>
                            </div>

                            // Playback mode chips (warm terracotta active state)
                            <div class="flex flex-wrap items-center gap-1.5">
                                <button
                                    onclick={on_set_random_mode}
                                    type="button"
                                    class={mode_btn_class(is_random_mode)}
                                    aria-label="Random mode"
                                    title="随机播放"
                                >
                                    <Icon name={IconName::Shuffle} size={13} />
                                    <span>{"随机"}</span>
                                </button>
                                <button
                                    onclick={on_set_playlist_mode}
                                    type="button"
                                    class={mode_btn_class(is_playlist_mode)}
                                    aria-label="Playlist sequential mode"
                                    title="列表顺序播放"
                                >
                                    <Icon name={IconName::List} size={13} />
                                    <span>{"顺序"}</span>
                                </button>
                                <button
                                    onclick={on_set_semantic_mode}
                                    type="button"
                                    class={mode_btn_class(is_semantic_mode)}
                                    aria-label="Semantic mode"
                                    title="语义相似推荐"
                                >
                                    <Icon name={IconName::Heart} size={13} />
                                    <span>{"语义"}</span>
                                </button>
                            </div>

                            // Volume + Download + Minimize
                            <div class="flex items-center gap-2.5">
                                <button
                                    onclick={on_toggle_mute.clone()}
                                    type="button"
                                    class="text-[var(--muted)] transition-colors hover:text-[var(--text)]"
                                    aria-label="Toggle mute"
                                >
                                    <Icon name={if volume > 0.0 { IconName::Volume2 } else { IconName::VolumeX }} size={16} />
                                </button>
                                <div class="w-20 max-sm:hidden">
                                    <input
                                        type="range"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        value={volume.to_string()}
                                        oninput={on_volume.clone()}
                                        class="h-[2px] w-full cursor-pointer appearance-none rounded-full bg-[var(--border-strong,var(--border))] accent-[var(--primary)]"
                                        aria-label="Volume"
                                    />
                                </div>
                                <a
                                    href={audio_url}
                                    download={format!("{}.{}", detail.title, detail.format)}
                                    class="flex h-8 w-8 items-center justify-center rounded-full border border-[var(--border)] text-[var(--muted)] transition-colors hover:border-[var(--border-strong,var(--primary))] hover:text-[var(--text)]"
                                    aria-label="Download"
                                    title="下载音频"
                                    onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                                >
                                    <Icon name={IconName::Download} size={14} />
                                </a>
                                <button
                                    onclick={on_minimize}
                                    type="button"
                                    class="flex h-8 w-8 items-center justify-center rounded-full border border-[var(--border)] text-[var(--muted)] transition-colors hover:border-[var(--border-strong,var(--primary))] hover:text-[var(--text)]"
                                    aria-label="Minimize player"
                                    title="收起至悬浮迷你播放器"
                                >
                                    <Icon name={IconName::Minimize2} size={14} />
                                </button>
                            </div>
                        </div>

                        // Mobile volume bar
                        <div class="mt-3 flex items-center gap-2 sm:hidden">
                            <button
                                onclick={on_toggle_mute}
                                type="button"
                                class="shrink-0 text-[var(--muted)] transition-colors hover:text-[var(--text)]"
                                aria-label="Toggle mute"
                            >
                                <Icon name={if volume > 0.0 { IconName::Volume2 } else { IconName::VolumeX }} size={15} />
                            </button>
                            <input
                                type="range"
                                min="0"
                                max="1"
                                step="0.01"
                                value={volume.to_string()}
                                oninput={on_volume}
                                class="h-[2px] flex-1 cursor-pointer appearance-none rounded-full bg-[var(--border-strong,var(--border))] accent-[var(--primary)]"
                                aria-label="Volume"
                            />
                            <span class="w-9 text-right font-mono text-xs tabular-nums text-[var(--muted)]">
                                {format!("{:.0}%", volume * 100.0)}
                            </span>
                        </div>
                    </div>

                    // Serif Synced Lyrics Panel
                    if *lyrics_loading {
                        <div class="mt-7 space-y-2.5 rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-5">
                            <div class="h-3.5 w-24 animate-pulse rounded bg-[var(--surface-alt)]" />
                            <div class="h-3.5 w-3/4 animate-pulse rounded bg-[var(--surface-alt)]" />
                            <div class="h-3.5 w-2/3 animate-pulse rounded bg-[var(--surface-alt)]" />
                            <div class="h-3.5 w-4/5 animate-pulse rounded bg-[var(--surface-alt)]" />
                        </div>
                    } else if lyrics_lrc.is_some() {
                        <div class="mt-7 overflow-hidden rounded-[12px] border border-[var(--border)] bg-[var(--surface)]">
                            <div class="flex items-center justify-between border-b border-[var(--border)] px-4 py-3">
                                <h2 class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">
                                    {"LYRICS · 同步歌词"}
                                </h2>
                                <div class="flex items-center gap-1.5 font-mono text-xs">
                                    <button
                                        onclick={on_offset_dec}
                                        type="button"
                                        class="flex h-6 w-6 items-center justify-center rounded border border-[var(--border)] bg-[var(--surface-alt)] text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--text)]"
                                        aria-label="Lyrics offset -0.5s"
                                    >
                                        {"−"}
                                    </button>
                                    <span class="min-w-[3.5rem] text-center tabular-nums text-[var(--muted)]">
                                        {&offset_display}
                                    </span>
                                    <button
                                        onclick={on_offset_inc}
                                        type="button"
                                        class="flex h-6 w-6 items-center justify-center rounded border border-[var(--border)] bg-[var(--surface-alt)] text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--text)]"
                                        aria-label="Lyrics offset +0.5s"
                                    >
                                        {"+"}
                                    </button>
                                    if lyrics_offset != 0.0 {
                                        <button
                                            onclick={on_offset_reset}
                                            type="button"
                                            class="ml-1 h-6 rounded border border-[var(--border)] bg-[var(--surface-alt)] px-1.5 text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--text)]"
                                            aria-label="Reset lyrics offset"
                                        >
                                            {"↺"}
                                        </button>
                                    }
                                </div>
                            </div>
                            <div class="font-serif">
                                <SyncedLyrics
                                    lyrics_lrc={lyrics_lrc.map(AttrValue::from)}
                                    lyrics_translation={lyrics_trans.map(AttrValue::from)}
                                    current_time={current_time}
                                    lyrics_offset={lyrics_offset}
                                />
                            </div>
                        </div>
                    }

                    // Warm Editorial Comments Section
                    <section class="mt-9">
                        if *comments_loading {
                            <div class="space-y-3">
                                <div class="mb-4 h-5 w-32 animate-pulse rounded bg-[var(--surface-alt)]" />
                                <div class="space-y-2 rounded-[10px] border border-[var(--border)] bg-[var(--surface)] p-4">
                                    <div class="h-3 w-24 animate-pulse rounded bg-[var(--surface-alt)]" />
                                    <div class="h-3 w-3/4 animate-pulse rounded bg-[var(--surface-alt)]" />
                                </div>
                            </div>
                        } else {
                            <div class="mb-4 flex items-baseline justify-between">
                                <h2 class="font-serif text-xl font-normal text-[var(--text)]">
                                    {format!("听友评论 ({})", comments.len())}
                                </h2>
                                <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">
                                    {"LISTENER NOTES"}
                                </span>
                            </div>

                            <form
                                onsubmit={on_submit_comment}
                                class="sf-card mb-6 rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-4 sm:p-5"
                            >
                                <div class="mb-3 flex gap-3">
                                    <input
                                        type="text"
                                        placeholder="昵称（可选）"
                                        value={(*nickname).clone()}
                                        oninput={on_nickname_input}
                                        class="sf-input w-full rounded-[8px] border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-sm text-[var(--text)] placeholder-[var(--muted)] transition-colors focus:border-[var(--primary)] focus:outline-none sm:max-w-xs"
                                    />
                                </div>
                                <div class="flex flex-col gap-2.5 sm:flex-row">
                                    <input
                                        type="text"
                                        placeholder="写下此刻听歌的感受..."
                                        value={(*comment_text).clone()}
                                        oninput={on_comment_input}
                                        class="sf-input flex-1 rounded-[8px] border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-sm text-[var(--text)] placeholder-[var(--muted)] transition-colors focus:border-[var(--primary)] focus:outline-none"
                                    />
                                    <button
                                        type="submit"
                                        disabled={*submitting}
                                        class="sf-btn sf-btn--primary shrink-0 rounded-[8px] bg-[var(--primary)] px-4 py-2 text-sm font-medium text-[var(--on-primary,#FAF9F5)] transition-colors hover:bg-[var(--primary-hover,#B85735)] disabled:opacity-50"
                                    >
                                        if *submitting { {"发送中..."} } else { {"发送评论"} }
                                    </button>
                                </div>
                                if let Some(err) = (*submit_error).as_ref() {
                                    <p class="mt-2 text-xs text-[var(--err,#B8443A)]">{err}</p>
                                }
                            </form>

                            if comments.is_empty() {
                                <div class="rounded-[12px] border border-[var(--border)] bg-[var(--surface)] px-4 py-8 text-center text-sm text-[var(--muted)]">
                                    {"暂无评论，留下第一条听歌笔记吧。"}
                                </div>
                            } else {
                                <div class="divide-y divide-[var(--border)] rounded-[12px] border border-[var(--border)] bg-[var(--surface)]">
                                    { for comments.iter().map(|c| {
                                        let ts = format_timestamp(c.created_at);
                                        html! {
                                            <div class="p-4">
                                                <div class="mb-1.5 flex items-center gap-2">
                                                    <span class="text-sm font-medium text-[var(--text)]">{&c.nickname}</span>
                                                    if let Some(region) = c.ip_region.as_ref() {
                                                        if !region.trim().is_empty() {
                                                            <span class="font-mono text-[11px] text-[var(--muted)]">{region}</span>
                                                        }
                                                    }
                                                    <span class="ml-auto font-mono text-[11px] tabular-nums text-[var(--muted)]">{ts}</span>
                                                </div>
                                                <p class="text-sm leading-relaxed text-[var(--text-2,var(--muted))]">{&c.comment_text}</p>
                                            </div>
                                        }
                                    }) }
                                </div>
                            }
                        }
                    </section>
                </div>
            </div>

            // Bottom Related Tracks (static shadowless cards)
            <div class="mt-14 border-t border-[var(--border)] pt-10">
                if *bottom_recs_loading {
                    <div class="mb-4 flex items-baseline justify-between">
                        <h2 class="font-serif text-xl font-normal text-[var(--text)]">{"相似曲目推荐"}</h2>
                        <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">{"RELATED TRACKS"}</span>
                    </div>
                    <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
                        { for (0..4).map(|_| html! {
                            <div class="aspect-[3/4] animate-pulse rounded-[8px] border border-[var(--border)] bg-[var(--surface)]" />
                        }) }
                    </div>
                } else if !bottom_recommendations.is_empty() {
                    <div class="mb-4 flex items-baseline justify-between">
                        <h2 class="font-serif text-xl font-normal text-[var(--text)]">{"相似曲目推荐"}</h2>
                        <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">{"RELATED TRACKS"}</span>
                    </div>
                    <div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
                        { for bottom_recommendations.iter().map(render_candidate_card) }
                    </div>
                }
            </div>
        </div>
    }
}

fn render_candidate_card(c: &api::SongSearchResult) -> Html {
    let cover = api::song_cover_url(c.cover_image.as_deref());
    let cid = c.id.clone();
    html! {
        <Link<Route>
            to={Route::MusicPlayer { id: cid }}
            classes="group flex flex-col overflow-hidden rounded-[8px] border border-[var(--border)] bg-[var(--surface)] no-underline transition-colors hover:border-[var(--border-strong,var(--primary))]"
        >
            <div class="relative aspect-square overflow-hidden bg-[var(--surface-alt)]">
                if cover.is_empty() {
                    <div class="flex h-full w-full items-center justify-center text-[var(--muted)]">
                        <Icon name={IconName::Music} size={32} class={classes!("opacity-30")} />
                    </div>
                } else {
                    <ImageWithLoading
                        src={cover}
                        alt={c.title.clone()}
                        loading={Some(AttrValue::from("lazy"))}
                        referrerpolicy={Some(AttrValue::from("no-referrer"))}
                        class="h-full w-full object-cover"
                        container_class={classes!("w-full", "h-full")}
                    />
                }
            </div>
            <div class="border-t border-[var(--border)] p-3">
                <p class="truncate font-serif text-[0.95rem] font-normal text-[var(--text)] transition-colors group-hover:text-[var(--primary)]">
                    {&c.title}
                </p>
                <p class="mt-0.5 truncate text-xs text-[var(--muted)]">{&c.artist}</p>
            </div>
        </Link<Route>>
    }
}

fn should_load_bottom_recommendations() -> bool {
    let Some(win) = window() else {
        return true;
    };
    let scroll_y = win.scroll_y().ok().unwrap_or(0.0);
    let viewport_h = win
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let doc_h = win
        .document()
        .and_then(|doc| doc.document_element())
        .map(|el| el.scroll_height() as f64)
        .unwrap_or(0.0);

    if doc_h <= 0.0 || viewport_h <= 0.0 {
        return true;
    }
    scroll_y + viewport_h + 320.0 >= doc_h
}

fn format_duration(ms: u64) -> String {
    let s = ms / 1000;
    format!("{:02}:{:02}", s / 60, s % 60)
}

fn format_time(secs: f64) -> String {
    if secs.is_nan() || secs.is_infinite() {
        return "00:00".to_string();
    }
    let t = secs as u64;
    format!("{:02}:{:02}", t / 60, t % 60)
}

fn format_timestamp(epoch_ms: i64) -> String {
    let secs = epoch_ms / 1000;
    let hours_ago = (js_sys::Date::now() as i64 / 1000 - secs) / 3600;
    if hours_ago < 1 {
        "just now".to_string()
    } else if hours_ago < 24 {
        format!("{}h ago", hours_ago)
    } else {
        let d = hours_ago / 24;
        if d < 30 {
            format!("{}d ago", d)
        } else {
            format!("day {}", secs / 86400)
        }
    }
}
