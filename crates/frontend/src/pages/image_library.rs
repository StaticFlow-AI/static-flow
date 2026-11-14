use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    components::{image_with_loading::ImageWithLoading, pagination::Pagination},
    i18n::current::image_library_page as t,
    router::Route,
    utils::image_url,
};

const IMAGE_PAGE_SIZE: usize = 24;
const RANDOM_RECOMMEND_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryDisplayMode {
    RandomRecommended,
    AllImages,
}

#[function_component(ImageLibraryPage)]
pub fn image_library_page() -> Html {
    let images = use_state(Vec::<api::ImageInfo>::new);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let total = use_state(|| 0usize);
    let current_page = use_state(|| 1usize);
    let display_mode = use_state(|| LibraryDisplayMode::RandomRecommended);
    let random_refresh_tick = use_state(|| 0_u64);
    let search_input = use_state(String::new);
    let active_query = use_state(|| None::<String>);
    let request_seq = use_mut_ref(|| 0_u64);
    let search_focused = use_state(|| false);

    let on_search_input = {
        let search_input = search_input.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                search_input.set(target.value());
            }
        })
    };

    let apply_search = {
        let search_input = search_input.clone();
        let active_query = active_query.clone();
        let current_page = current_page.clone();
        move || {
            let query = search_input.trim().to_string();
            if query.is_empty() {
                active_query.set(None);
            } else {
                active_query.set(Some(query));
            }
            current_page.set(1);
        }
    };

    let on_search_click = {
        let apply_search = apply_search.clone();
        Callback::from(move |_: MouseEvent| apply_search())
    };

    let on_search_keypress = {
        let apply_search = apply_search.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                apply_search();
            }
        })
    };

    let on_search_focus = {
        let search_focused = search_focused.clone();
        Callback::from(move |_: FocusEvent| search_focused.set(true))
    };

    let on_search_blur = {
        let search_focused = search_focused.clone();
        Callback::from(move |_: FocusEvent| search_focused.set(false))
    };

    let on_clear_search = {
        let search_input = search_input.clone();
        let active_query = active_query.clone();
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| {
            search_input.set(String::new());
            active_query.set(None);
            current_page.set(1);
        })
    };

    let on_switch_random = {
        let display_mode = display_mode.clone();
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| {
            display_mode.set(LibraryDisplayMode::RandomRecommended);
            current_page.set(1);
        })
    };

    let on_switch_all = {
        let display_mode = display_mode.clone();
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| {
            display_mode.set(LibraryDisplayMode::AllImages);
            current_page.set(1);
        })
    };

    let on_refresh_random = {
        let random_refresh_tick = random_refresh_tick.clone();
        Callback::from(move |_: MouseEvent| {
            random_refresh_tick.set(*random_refresh_tick + 1);
        })
    };

    {
        let images = images.clone();
        let loading = loading.clone();
        let error = error.clone();
        let total = total.clone();
        let request_seq = request_seq.clone();
        let deps = (*display_mode, (*active_query).clone(), *current_page, *random_refresh_tick);
        use_effect_with(deps, move |deps| {
            let request_id = {
                let mut seq = request_seq.borrow_mut();
                *seq += 1;
                *seq
            };

            let (mode, active_query, page, _refresh_tick) = deps.clone();
            loading.set(true);
            error.set(None);
            let request_seq = request_seq.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let response = if let Some(query) = active_query {
                    let offset = page.saturating_sub(1) * IMAGE_PAGE_SIZE;
                    api::search_images_by_text_page(
                        &query,
                        Some(IMAGE_PAGE_SIZE),
                        Some(offset),
                        None,
                    )
                    .await
                } else if mode == LibraryDisplayMode::RandomRecommended {
                    api::fetch_random_images_page(Some(RANDOM_RECOMMEND_LIMIT)).await
                } else {
                    let offset = page.saturating_sub(1) * IMAGE_PAGE_SIZE;
                    api::fetch_images_page(Some(IMAGE_PAGE_SIZE), Some(offset)).await
                };

                match response {
                    Ok(page_data) => {
                        if *request_seq.borrow() != request_id {
                            return;
                        }
                        total.set(page_data.total);
                        images.set(page_data.images);
                    },
                    Err(err) => {
                        if *request_seq.borrow() != request_id {
                            return;
                        }
                        error.set(Some(err));
                    },
                }

                if *request_seq.borrow() != request_id {
                    return;
                }
                loading.set(false);
            });

            || ()
        });
    }

    let on_page_change = {
        let current_page = current_page.clone();
        Callback::from(move |page: usize| current_page.set(page))
    };

    let total_value = *total;
    let total_pages = if total_value == 0 { 1 } else { total_value.div_ceil(IMAGE_PAGE_SIZE) };
    let query_label = (*active_query).clone();
    let query_active = query_label.is_some();
    let should_show_pagination = query_active || *display_mode == LibraryDisplayMode::AllImages;

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

    html! {
        <div class="mx-auto max-w-7xl px-4 pb-16 sm:px-6 lg:px-8">
            <header class="sf-page-hero flex flex-wrap items-end justify-between gap-4">
                <div>
                    <div class="sf-eyebrow">{ "VISUAL ARCHIVE" }</div>
                    <h1 class="sf-page-title">
                        { t::TITLE }
                    </h1>
                    <p class="sf-page-sub">
                        { t::SUBTITLE }
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-2">
                    <Link<Route>
                        to={Route::MediaAudio}
                        classes="btn-terminal"
                    >
                        <i class="fas fa-music text-xs text-[var(--primary)]" aria-hidden="true"></i>
                        { t::BTN_AUDIO_LIBRARY }
                    </Link<Route>>
                    <Link<Route>
                        to={Route::MediaVideo}
                        classes="btn-terminal"
                    >
                        <i class="fas fa-video text-xs text-[var(--primary)]" aria-hidden="true"></i>
                        { t::BTN_VIDEO_LIBRARY }
                    </Link<Route>>
                </div>
            </header>

            <div class="mb-4 flex flex-wrap items-center gap-2">
                <div class="inline-flex items-center gap-1 rounded-[var(--radius)] border border-[var(--border)] \
                            bg-[var(--surface-alt)] p-1">
                    <button
                        type="button"
                        onclick={on_switch_random}
                        class={segment_class(*display_mode == LibraryDisplayMode::RandomRecommended)}
                    >
                        { t::MODE_RANDOM }
                    </button>
                    <button
                        type="button"
                        onclick={on_switch_all}
                        class={segment_class(*display_mode == LibraryDisplayMode::AllImages)}
                    >
                        { t::MODE_ALL }
                    </button>
                </div>
                if *display_mode == LibraryDisplayMode::RandomRecommended && !query_active {
                    <button
                        type="button"
                        onclick={on_refresh_random}
                        class="sf-chip cursor-pointer"
                    >
                        <i class="fas fa-shuffle text-[10px]" aria-hidden="true"></i>
                        { t::BTN_REFRESH_RANDOM }
                    </button>
                }
                if query_active {
                    <button
                        type="button"
                        onclick={on_clear_search}
                        class="sf-chip cursor-pointer"
                    >
                        <i class="fas fa-xmark text-[10px]" aria-hidden="true"></i>
                        { t::BTN_CLEAR_SEARCH }
                    </button>
                }
            </div>

            <div class={classes!("music-search-hero", search_focused.then_some("focused"))}>
                <div class="music-search-hero-inner">
                    <i class="fas fa-image music-search-icon" />
                    <input
                        type="text"
                        placeholder={t::SEARCH_PLACEHOLDER}
                        class="music-search-input"
                        value={(*search_input).clone()}
                        oninput={on_search_input}
                        onkeypress={on_search_keypress}
                        onfocus={on_search_focus}
                        onblur={on_search_blur}
                    />
                    <button class="music-search-btn" onclick={on_search_click} type="button">
                        <i class="fas fa-search" />
                    </button>
                </div>
            </div>

            <div class="mb-4 font-mono text-xs tracking-[0.02em] text-[var(--muted)]">
                {
                    if let Some(query) = query_label {
                        format!("{}: {}（{}）", t::LABEL_SEARCH_RESULTS, total_value, query)
                    } else if *display_mode == LibraryDisplayMode::RandomRecommended {
                        t::LABEL_RANDOM_HINT.to_string()
                    } else {
                        format!("{}: {}", t::LABEL_TOTAL_IMAGES, total_value)
                    }
                }
            </div>

            if *loading {
                <div class="flex justify-center py-20">
                    <div class="h-8 w-8 animate-spin rounded-full border-2 border-[var(--border)] \
                                border-t-[var(--primary)] motion-reduce:animate-none" />
                </div>
            } else if let Some(ref err) = *error {
                <div class="sf-state sf-state--error">
                    <span class="sf-state__icon">
                        <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                    </span>
                    <p class="sf-state__body">{ format!("{}: {}", t::LOAD_ERROR_PREFIX, err) }</p>
                </div>
            } else if images.is_empty() {
                <div class="sf-state">
                    <span class="sf-state__icon">
                        <i
                            class={if query_active { "fas fa-magnifying-glass" } else { "fas fa-images" }}
                            aria-hidden="true"
                        ></i>
                    </span>
                    <p class="sf-state__body">
                        {
                            if query_active {
                                t::EMPTY_SEARCH
                            } else {
                                t::EMPTY_LIBRARY
                            }
                        }
                    </p>
                </div>
            } else {
                <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:gap-5 lg:grid-cols-4 xl:grid-cols-5">
                    { for images.iter().map(render_image_card) }
                </div>
            }

            if should_show_pagination && total_pages > 1 {
                <div class="mt-10 flex justify-center">
                    <Pagination
                        current_page={*current_page}
                        total_pages={total_pages}
                        on_page_change={on_page_change}
                    />
                </div>
            }
        </div>
    }
}

fn render_image_card(image: &api::ImageInfo) -> Html {
    let src = image_url(&format!("images/{}", image.filename));
    let filename = image.filename.clone();
    let id = image.id.clone();

    html! {
        <a
            href={src.clone()}
            target="_blank"
            rel="noopener noreferrer"
            class="group flex flex-col overflow-hidden rounded-[var(--radius-lg)] border border-[var(--border)] \
                   bg-[var(--surface)] no-underline shadow-[var(--shadow-sm)] \
                   transition-[transform,box-shadow,border-color] duration-[var(--motion-base)] ease-[var(--ease-spring)] \
                   hover:-translate-y-1 hover:border-[var(--primary)]/50 hover:shadow-[var(--shadow-8)] \
                   motion-reduce:transition-none motion-reduce:hover:translate-y-0"
            title={filename.clone()}
        >
            <div class="relative aspect-square overflow-hidden bg-[var(--surface-alt)]">
                <ImageWithLoading
                    src={src}
                    alt={filename.clone()}
                    loading={Some(AttrValue::from("lazy"))}
                    class="h-full w-full object-cover transition-transform duration-500 ease-[var(--ease-spring)] \
                           group-hover:scale-[1.04] motion-reduce:transition-none motion-reduce:group-hover:scale-100"
                    container_class={classes!("w-full", "h-full")}
                />
            </div>
            <div class="flex flex-col gap-1 border-t border-[var(--border)] px-3 py-2.5">
                <p class="truncate text-xs font-medium text-[var(--text)]">{ filename }</p>
                <p class="truncate font-mono text-[10px] text-[var(--muted)]">{ id }</p>
            </div>
        </a>
    }
}
