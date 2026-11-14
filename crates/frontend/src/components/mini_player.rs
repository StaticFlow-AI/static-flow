use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    components::{
        icons::{Icon, IconName},
        image_with_loading::ImageWithLoading,
        persistent_audio::resolve_next_song,
    },
    music_context::{MusicAction, MusicPlayerContext},
    router::Route,
};

#[function_component(MiniPlayer)]
pub fn mini_player() -> Html {
    let ctx = use_context::<MusicPlayerContext>();
    let navigator = use_navigator();

    let ctx = match ctx {
        Some(c) => c,
        None => return html! {},
    };

    if !ctx.visible || ctx.current_song.is_none() {
        return html! {};
    }

    let show = ctx.minimized;
    let song = ctx
        .current_song
        .as_ref()
        .expect("guarded by is_none check above");

    let cover_url = api::song_cover_url(song.cover_image.as_deref());
    let title = song.title.clone();
    let artist = song.artist.clone();
    let playing = ctx.playing;
    let progress_pct = if ctx.duration > 0.0 && ctx.duration.is_finite() {
        ((ctx.current_time / ctx.duration) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let on_toggle_play = {
        let ctx = ctx.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            ctx.dispatch(MusicAction::TogglePlay);
        })
    };

    let can_prev = ctx.history_index.map(|i| i > 0).unwrap_or(false);
    let on_prev = {
        let ctx = ctx.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            ctx.dispatch(MusicAction::PlayPrev);
        })
    };

    let on_next = {
        let ctx = ctx.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            let c2 = ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fallback = resolve_next_song(&c2).await;
                c2.dispatch(MusicAction::PlayNext {
                    fallback,
                });
            });
        })
    };

    let on_close = {
        let ctx = ctx.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            ctx.dispatch(MusicAction::Close);
        })
    };

    let on_expand = {
        let ctx = ctx.clone();
        let navigator = navigator.clone();
        let song_id = ctx.song_id.clone();
        Callback::from(move |_: MouseEvent| {
            ctx.dispatch(MusicAction::Expand);
            if let (Some(nav), Some(id)) = (navigator.as_ref(), song_id.as_ref()) {
                nav.push(&Route::MusicPlayer {
                    id: id.clone(),
                });
            }
        })
    };

    let vis_class = if show {
        "translate-y-0 opacity-100 pointer-events-auto"
    } else {
        "translate-y-2 opacity-0 pointer-events-none"
    };

    let container_class = format!(
        "fixed bottom-4 right-4 max-sm:inset-x-3 max-sm:bottom-3 sm:w-[360px] h-16 z-[70] \
         overflow-hidden rounded-[12px] border border-[var(--border-strong,var(--border))] \
         bg-[var(--surface)] shadow-[var(--shadow-lg)] px-3 cursor-pointer flex items-center \
         gap-2.5 select-none transition-[transform,opacity,border-color] duration-[240ms] \
         ease-out hover:border-[var(--primary)]/55 motion-reduce:transition-none {vis_class}"
    );

    html! {
        <div onclick={on_expand} class={container_class}>
            // Top 2px terracotta playback progress hairline
            <div
                class="pointer-events-none absolute inset-x-0 top-0 h-[2px] bg-[var(--border)]"
                aria-hidden="true"
            >
                <div
                    class="h-full bg-[var(--primary)] transition-[width] duration-200 ease-linear"
                    style={format!("width: {progress_pct:.2}%")}
                />
            </div>

            // 40px static square cover
            <div class="h-10 w-10 shrink-0 overflow-hidden rounded-[6px] border border-[var(--border)] bg-[var(--surface-alt)]">
                if cover_url.is_empty() {
                    <div class="flex h-full w-full items-center justify-center text-[var(--muted)]">
                        <Icon name={IconName::Music} size={16} class={classes!("opacity-40")} />
                    </div>
                } else {
                    <ImageWithLoading
                        src={cover_url}
                        alt={title.clone()}
                        referrerpolicy={Some(AttrValue::from("no-referrer"))}
                        loading={Some(AttrValue::from("eager"))}
                        class="h-full w-full object-cover"
                        container_class={classes!("w-full", "h-full")}
                    />
                }
            </div>

            // Serif song title + monospace artist
            <div class="min-w-0 flex-1 pl-0.5">
                <p class="truncate font-serif text-[0.875rem] font-normal leading-tight text-[var(--text)]">
                    {title}
                </p>
                <p class="mt-0.5 truncate font-mono text-[0.6875rem] leading-tight text-[var(--muted)]">
                    {artist}
                </p>
            </div>

            // Restrained playback controls
            <div class="flex shrink-0 items-center gap-0.5">
                <button
                    onclick={on_prev}
                    type="button"
                    disabled={!can_prev}
                    class="flex h-7 w-7 items-center justify-center rounded-full text-[var(--muted)] transition-colors hover:bg-[var(--surface-alt)] hover:text-[var(--text)] disabled:cursor-not-allowed disabled:opacity-30 disabled:hover:bg-transparent disabled:hover:text-[var(--muted)]"
                    aria-label="Previous song"
                >
                    <Icon name={IconName::SkipBack} size={13} />
                </button>

                <button
                    onclick={on_toggle_play}
                    type="button"
                    class="mx-0.5 flex h-8 w-8 items-center justify-center rounded-full bg-[var(--primary)] text-[var(--on-primary,#FAF9F5)] transition-colors hover:bg-[var(--primary-hover,#B85735)]"
                    aria-label={if playing { "Pause" } else { "Play" }}
                >
                    <Icon name={if playing { IconName::Pause } else { IconName::Play }} size={13} />
                </button>

                <button
                    onclick={on_next}
                    type="button"
                    class="flex h-7 w-7 items-center justify-center rounded-full text-[var(--muted)] transition-colors hover:bg-[var(--surface-alt)] hover:text-[var(--text)]"
                    aria-label="Next song"
                >
                    <Icon name={IconName::SkipForward} size={13} />
                </button>

                <button
                    onclick={on_close}
                    type="button"
                    class="ml-0.5 flex h-7 w-7 items-center justify-center rounded-full text-[var(--muted)] transition-colors hover:bg-[var(--surface-alt)] hover:text-[var(--text)]"
                    aria-label="Close player"
                >
                    <Icon name={IconName::X} size={12} />
                </button>
            </div>
        </div>
    }
}
