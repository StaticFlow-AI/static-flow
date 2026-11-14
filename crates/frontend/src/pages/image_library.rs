use wasm_bindgen::JsCast;
use web_sys::{HtmlImageElement, HtmlInputElement, KeyboardEvent};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    components::{
        icons::{Icon, IconName},
        image_with_loading::ImageWithLoading,
        pagination::Pagination,
    },
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
    let catalog_total = use_state(|| 0usize);
    let current_page = use_state(|| 1usize);
    let display_mode = use_state(|| LibraryDisplayMode::RandomRecommended);
    let random_refresh_tick = use_state(|| 0_u64);
    let search_input = use_state(String::new);
    let active_query = use_state(|| None::<String>);
    let request_seq = use_mut_ref(|| 0_u64);
    let lightbox_index = use_state(|| None::<usize>);
    let lightbox_dims = use_state(|| None::<(u32, u32)>);

    // Fetch total image count once on mount for header counter
    {
        let catalog_total = catalog_total.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(page_data) = api::fetch_images_page(Some(1), Some(0)).await {
                    catalog_total.set(page_data.total);
                }
            });
            || ()
        });
    }

    // Lightbox keyboard navigation (Escape / ArrowLeft / ArrowRight)
    {
        let lightbox_index = lightbox_index.clone();
        let lightbox_dims = lightbox_dims.clone();
        let img_len = images.len();
        let is_open = lightbox_index.is_some();
        use_effect_with((is_open, img_len), move |(open, len)| {
            let mut listener_opt: Option<gloo_events::EventListener> = None;
            if *open {
                if let Some(win) = web_sys::window() {
                    let lb_idx = lightbox_index.clone();
                    let lb_dims = lightbox_dims.clone();
                    let count = *len;
                    listener_opt =
                        Some(gloo_events::EventListener::new(&win, "keydown", move |event| {
                            let Some(ke) = event.dyn_ref::<KeyboardEvent>() else {
                                return;
                            };
                            match ke.key().as_str() {
                                "Escape" => {
                                    lb_idx.set(None);
                                    lb_dims.set(None);
                                },
                                "ArrowLeft" if count > 1 => {
                                    if let Some(cur) = *lb_idx {
                                        let prev = if cur == 0 { count - 1 } else { cur - 1 };
                                        lb_dims.set(None);
                                        lb_idx.set(Some(prev));
                                    }
                                },
                                "ArrowRight" if count > 1 => {
                                    if let Some(cur) = *lb_idx {
                                        let next = (cur + 1) % count;
                                        lb_dims.set(None);
                                        lb_idx.set(Some(next));
                                    }
                                },
                                _ => {},
                            }
                        }));
                }
            }
            move || {
                drop(listener_opt);
            }
        });
    }

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
        let lightbox_index = lightbox_index.clone();
        move || {
            let query = search_input.trim().to_string();
            if query.is_empty() {
                active_query.set(None);
            } else {
                active_query.set(Some(query));
            }
            lightbox_index.set(None);
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

    let on_clear_search = {
        let search_input = search_input.clone();
        let active_query = active_query.clone();
        let current_page = current_page.clone();
        let lightbox_index = lightbox_index.clone();
        Callback::from(move |_: MouseEvent| {
            search_input.set(String::new());
            active_query.set(None);
            lightbox_index.set(None);
            current_page.set(1);
        })
    };

    let on_switch_random = {
        let display_mode = display_mode.clone();
        let current_page = current_page.clone();
        let lightbox_index = lightbox_index.clone();
        Callback::from(move |_: MouseEvent| {
            display_mode.set(LibraryDisplayMode::RandomRecommended);
            lightbox_index.set(None);
            current_page.set(1);
        })
    };

    let on_switch_all = {
        let display_mode = display_mode.clone();
        let current_page = current_page.clone();
        let lightbox_index = lightbox_index.clone();
        Callback::from(move |_: MouseEvent| {
            display_mode.set(LibraryDisplayMode::AllImages);
            lightbox_index.set(None);
            current_page.set(1);
        })
    };

    let on_refresh_random = {
        let random_refresh_tick = random_refresh_tick.clone();
        let lightbox_index = lightbox_index.clone();
        Callback::from(move |_: MouseEvent| {
            lightbox_index.set(None);
            random_refresh_tick.set(*random_refresh_tick + 1);
        })
    };

    {
        let images = images.clone();
        let loading = loading.clone();
        let error = error.clone();
        let total = total.clone();
        let catalog_total = catalog_total.clone();
        let request_seq = request_seq.clone();
        let deps = (*display_mode, (*active_query).clone(), *current_page, *random_refresh_tick);
        use_effect_with(deps, move |deps| {
            let request_id = {
                let mut seq = request_seq.borrow_mut();
                *seq += 1;
                *seq
            };

            let (mode, active_query, page, _refresh_tick) = deps.clone();
            let is_all_unfiltered = active_query.is_none() && mode == LibraryDisplayMode::AllImages;
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
                        if is_all_unfiltered {
                            catalog_total.set(page_data.total);
                        }
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
        let lightbox_index = lightbox_index.clone();
        Callback::from(move |page: usize| {
            lightbox_index.set(None);
            current_page.set(page);
        })
    };

    let on_open_lightbox = {
        let lightbox_index = lightbox_index.clone();
        let lightbox_dims = lightbox_dims.clone();
        Callback::from(move |idx: usize| {
            lightbox_dims.set(None);
            lightbox_index.set(Some(idx));
        })
    };

    let on_close_lightbox = {
        let lightbox_index = lightbox_index.clone();
        let lightbox_dims = lightbox_dims.clone();
        Callback::from(move |_: MouseEvent| {
            lightbox_index.set(None);
            lightbox_dims.set(None);
        })
    };

    let total_value = *total;
    let header_count = if *catalog_total > 0 { *catalog_total } else { total_value };
    let total_pages = if total_value == 0 { 1 } else { total_value.div_ceil(IMAGE_PAGE_SIZE) };
    let query_label = (*active_query).clone();
    let query_active = query_label.is_some();
    let should_show_pagination = query_active || *display_mode == LibraryDisplayMode::AllImages;

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

    let lightbox_node = if let Some(active_idx) = *lightbox_index {
        if let Some(active_img) = images.get(active_idx) {
            let full_src = image_url(&format!("images/{}", active_img.filename));
            let filename = active_img.filename.clone();
            let img_id = active_img.id.clone();
            let total_len = images.len();
            let on_prev_img = {
                let lb_idx = lightbox_index.clone();
                let lb_dims = lightbox_dims.clone();
                Callback::from(move |e: MouseEvent| {
                    e.stop_propagation();
                    if total_len > 1 {
                        let prev = if active_idx == 0 { total_len - 1 } else { active_idx - 1 };
                        lb_dims.set(None);
                        lb_idx.set(Some(prev));
                    }
                })
            };
            let on_next_img = {
                let lb_idx = lightbox_index.clone();
                let lb_dims = lightbox_dims.clone();
                Callback::from(move |e: MouseEvent| {
                    e.stop_propagation();
                    if total_len > 1 {
                        let next = (active_idx + 1) % total_len;
                        lb_dims.set(None);
                        lb_idx.set(Some(next));
                    }
                })
            };
            let on_img_load = {
                let lb_dims = lightbox_dims.clone();
                Callback::from(move |e: Event| {
                    if let Some(img_el) = e.target_dyn_into::<HtmlImageElement>() {
                        let w = img_el.natural_width();
                        let h = img_el.natural_height();
                        if w > 0 && h > 0 {
                            lb_dims.set(Some((w, h)));
                        }
                    }
                })
            };

            html! {
                <div
                    role="dialog"
                    aria-modal="true"
                    aria-label={filename.clone()}
                    onclick={on_close_lightbox.clone()}
                    class="fixed inset-0 z-[100] flex flex-col justify-between bg-[rgba(250,249,245,0.96)] p-4 backdrop-blur-[2px] dark:bg-[rgba(20,20,19,0.96)] sm:p-6"
                >
                    <div
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                        class="mx-auto flex w-full max-w-[1200px] items-center justify-between gap-4 border-b border-[var(--border)] pb-3"
                    >
                        <div class="min-w-0">
                            <p class="truncate font-mono text-xs text-[var(--text)]">{&filename}</p>
                            <p class="mt-0.5 truncate font-mono text-[11px] tabular-nums text-[var(--muted)]">
                                {
                                    if let Some((w, h)) = *lightbox_dims {
                                        format!("{w} × {h} px · id {}", img_id.chars().take(12).collect::<String>())
                                    } else {
                                        format!("id {}", img_id.chars().take(12).collect::<String>())
                                    }
                                }
                            </p>
                        </div>
                        <div class="flex shrink-0 items-center gap-2">
                            <a
                                href={full_src.clone()}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="rounded-[6px] border border-[var(--border-strong,var(--border))] bg-[var(--surface)] px-3 py-1.5 font-mono text-xs text-[var(--text)] no-underline transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)]"
                            >
                                {"打开原图 ↗"}
                            </a>
                            <button
                                type="button"
                                onclick={on_close_lightbox}
                                aria-label="Close preview"
                                class="flex h-8 w-8 items-center justify-center rounded-[6px] border border-[var(--border-strong,var(--border))] bg-[var(--surface)] text-[var(--text)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)]"
                            >
                                <Icon name={IconName::X} size={15} />
                            </button>
                        </div>
                    </div>

                    <div class="relative my-4 flex min-h-0 flex-1 items-center justify-center">
                        if total_len > 1 {
                            <button
                                type="button"
                                onclick={on_prev_img}
                                aria-label="Previous image"
                                class="absolute left-0 z-10 flex h-10 w-10 items-center justify-center rounded-full border border-[var(--border-strong,var(--border))] bg-[var(--surface)] text-[var(--text)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)] sm:left-2"
                            >
                                <Icon name={IconName::ChevronLeft} size={18} />
                            </button>
                        }

                        <img
                            src={full_src}
                            alt={filename}
                            onload={on_img_load}
                            onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                            class="max-h-[78vh] max-w-[88vw] rounded-[8px] border border-[var(--border)] bg-[var(--surface)] object-contain shadow-[var(--shadow-lg)]"
                        />

                        if total_len > 1 {
                            <button
                                type="button"
                                onclick={on_next_img}
                                aria-label="Next image"
                                class="absolute right-0 z-10 flex h-10 w-10 items-center justify-center rounded-full border border-[var(--border-strong,var(--border))] bg-[var(--surface)] text-[var(--text)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)] sm:right-2"
                            >
                                <Icon name={IconName::ChevronRight} size={18} />
                            </button>
                        }
                    </div>

                    <div class="text-center font-mono text-xs tabular-nums text-[var(--muted)]">
                        {format!("{} / {}", active_idx + 1, total_len)}
                    </div>
                </div>
            }
        } else {
            Html::default()
        }
    } else {
        Html::default()
    };

    html! {
        <div class="mx-auto max-w-[1200px] px-4 pb-20 pt-10 sm:px-6 md:pt-14 lg:px-8">
            // Editorial Page Header
            <header class="mb-8 flex flex-col justify-between gap-6 md:flex-row md:items-end">
                <div class="max-w-2xl">
                    <div class="sf-eyebrow mb-3 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--muted)]">
                        { "VISUAL ARCHIVE · 图像馆藏" }
                    </div>
                    <h1 class="font-serif text-[clamp(2.125rem,4vw,3.125rem)] font-normal leading-[1.1] tracking-[-0.02em] text-[var(--text)]">
                        { t::TITLE }
                    </h1>
                    <p class="mt-3 text-[0.975rem] leading-[1.65] text-[var(--text-2,var(--muted))]">
                        { t::SUBTITLE }
                    </p>
                    if header_count > 0 {
                        <div class="mt-3 font-mono text-xs tabular-nums text-[var(--muted)]">
                            { format!("{}: {header_count}", t::LABEL_TOTAL_IMAGES) }
                        </div>
                    }
                </div>
                <div class="flex shrink-0 flex-wrap items-center gap-2.5">
                    <Link<Route>
                        to={Route::MediaAudio}
                        classes="sf-btn sf-btn--secondary inline-flex items-center gap-2 rounded-[8px] border border-[var(--border-strong,var(--border))] bg-transparent px-3.5 py-2 text-sm font-medium text-[var(--text)] no-underline transition-colors hover:bg-[var(--surface-alt)]"
                    >
                        <span>{ t::BTN_AUDIO_LIBRARY }</span>
                        <span class="font-mono text-xs text-[var(--muted)]">{"→"}</span>
                    </Link<Route>>
                    <Link<Route>
                        to={Route::MediaVideo}
                        classes="sf-btn sf-btn--secondary inline-flex items-center gap-2 rounded-[8px] border border-[var(--border-strong,var(--border))] bg-transparent px-3.5 py-2 text-sm font-medium text-[var(--text)] no-underline transition-colors hover:bg-[var(--surface-alt)]"
                    >
                        <span>{ t::BTN_VIDEO_LIBRARY }</span>
                        <span class="font-mono text-xs text-[var(--muted)]">{"→"}</span>
                    </Link<Route>>
                </div>
            </header>

            // Toolbar: Text Tabs + Chips + Search Input
            <div class="mb-5 flex flex-col justify-between gap-4 border-b border-[var(--border)] pb-3 md:flex-row md:items-end">
                <div class="flex flex-wrap items-center gap-5">
                    <button
                        type="button"
                        onclick={on_switch_random}
                        class={tab_class(*display_mode == LibraryDisplayMode::RandomRecommended && !query_active)}
                    >
                        <span>{ t::MODE_RANDOM }</span>
                    </button>
                    <button
                        type="button"
                        onclick={on_switch_all}
                        class={tab_class(*display_mode == LibraryDisplayMode::AllImages && !query_active)}
                    >
                        <span>{ t::MODE_ALL }</span>
                        if header_count > 0 {
                            <span class="font-mono text-xs tabular-nums text-[var(--muted)]">{header_count}</span>
                        }
                    </button>
                    if *display_mode == LibraryDisplayMode::RandomRecommended && !query_active {
                        <button
                            type="button"
                            onclick={on_refresh_random}
                            class="sf-chip mb-1 inline-flex cursor-pointer items-center gap-1.5 rounded-full border border-[var(--border)] bg-[var(--surface)] px-3 py-1 font-mono text-xs text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)]"
                        >
                            <Icon name={IconName::Shuffle} size={12} />
                            <span>{ t::BTN_REFRESH_RANDOM }</span>
                        </button>
                    }
                    if query_active {
                        <button
                            type="button"
                            onclick={on_clear_search}
                            class="sf-chip sf-chip--active mb-1 inline-flex cursor-pointer items-center gap-1.5 rounded-full border border-[var(--primary)]/35 bg-[var(--primary-soft,rgba(201,100,66,0.12))] px-3 py-1 font-mono text-xs text-[var(--primary)]"
                        >
                            <Icon name={IconName::X} size={11} />
                            <span>{ t::BTN_CLEAR_SEARCH }</span>
                        </button>
                    }
                </div>

                <div class="relative flex w-full items-center md:w-80">
                    <input
                        type="text"
                        placeholder={t::SEARCH_PLACEHOLDER}
                        value={(*search_input).clone()}
                        oninput={on_search_input}
                        onkeypress={on_search_keypress}
                        class="sf-input h-9 w-full rounded-[8px] border border-[var(--border)] bg-[var(--surface)] pl-3 pr-8 text-sm text-[var(--text)] placeholder-[var(--muted)] transition-colors focus:border-[var(--primary)] focus:outline-none"
                    />
                    <button
                        onclick={on_search_click}
                        type="button"
                        aria-label="Search images"
                        class="absolute right-2 inline-flex h-6 w-6 items-center justify-center text-[var(--muted)] transition-colors hover:text-[var(--primary)]"
                    >
                        <Icon name={IconName::Search} size={14} />
                    </button>
                </div>
            </div>

            <div class="mb-5 font-mono text-xs tracking-[0.02em] text-[var(--muted)]">
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
                <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-5">
                    { for (0..10).map(|_| html! {
                        <div class="aspect-[4/3] animate-pulse rounded-[6px] border border-[var(--border)] bg-[var(--surface-alt)]" />
                    }) }
                </div>
            } else if let Some(ref err) = *error {
                <div class="rounded-[12px] border border-[var(--err,#B8443A)]/30 bg-[var(--err-soft,rgba(184,68,58,0.08))] p-6 text-sm text-[var(--err,#B8443A)]">
                    { format!("{}: {}", t::LOAD_ERROR_PREFIX, err) }
                </div>
            } else if images.is_empty() {
                <div class="rounded-[12px] border border-[var(--border)] bg-[var(--surface)] px-6 py-16 text-center text-sm text-[var(--muted)]">
                    {
                        if query_active {
                            t::EMPTY_SEARCH
                        } else {
                            t::EMPTY_LIBRARY
                        }
                    }
                </div>
            } else {
                // Clean masonry / equal-height grid: gap-3, rounded-[6px], zero card shadow
                <div class="columns-2 gap-3 space-y-3 sm:columns-3 md:columns-4 xl:columns-5">
                    { for images.iter().enumerate().map(|(idx, image)| {
                        render_image_tile(idx, image, &on_open_lightbox)
                    }) }
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

            // Warm semi-transparent Lightbox Preview (rgba(250,249,245,0.96) light / rgba(20,20,19,0.96) dark)
            {lightbox_node}
        </div>
    }
}

fn render_image_tile(index: usize, image: &api::ImageInfo, on_open: &Callback<usize>) -> Html {
    let src = image_url(&format!("images/{}", image.filename));
    let filename = image.filename.clone();
    let short_id: String = image.id.chars().take(8).collect();
    let on_click = {
        let cb = on_open.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            cb.emit(index);
        })
    };

    html! {
        <button
            type="button"
            onclick={on_click}
            title={filename.clone()}
            class="group relative block w-full break-inside-avoid overflow-hidden rounded-[6px] border border-[var(--border)] bg-[var(--surface-alt)] p-0 text-left transition-colors hover:border-[var(--border-strong,var(--primary))] focus:outline-none focus-visible:border-[var(--primary)]"
        >
            <ImageWithLoading
                src={src}
                alt={filename.clone()}
                loading={Some(AttrValue::from("lazy"))}
                class="block h-auto min-h-[120px] w-full object-cover"
                container_class={classes!("w-full")}
            />
            <div
                class="pointer-events-none absolute inset-x-0 bottom-0 px-2.5 pb-2 pt-8 opacity-0 transition-opacity duration-200 group-hover:opacity-100 group-focus-visible:opacity-100"
                style="background: linear-gradient(to top, rgba(20,20,19,0.68), rgba(20,20,19,0));"
            >
                <p class="truncate font-mono text-[11px] leading-tight text-[#FAF9F5]">
                    {filename}
                </p>
                <p class="mt-0.5 truncate font-mono text-[10px] tabular-nums text-[#FAF9F5]/75">
                    {format!("#{short_id}")}
                </p>
            </div>
        </button>
    }
}
