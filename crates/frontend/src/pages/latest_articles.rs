use std::collections::BTreeMap;

use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use static_flow_shared::ArticleListItem;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, Event};
use yew::prelude::*;
use yew_router::prelude::use_location;

use crate::{
    components::{
        article_card::{ArticleCard, ArticleCardVariant},
        pagination::Pagination,
        scroll_to_top_button::ScrollToTopButton,
    },
    i18n::{current::latest_articles_page as t, fill_one},
    pages::posts::render_archive_skeleton,
};

const PAGE_SIZE: usize = 12;

fn smooth_scroll_to_id(id: &str) {
    let Some(win) = window() else {
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

#[function_component(LatestArticlesPage)]
pub fn latest_articles_page() -> Html {
    let route_location = use_location();
    let articles = use_state(Vec::<ArticleListItem>::new);
    let loading = use_state(|| true);
    let current_page = use_state(|| 1_usize);
    let total = use_state(|| 0_usize);
    let fetch_seq = use_mut_ref(|| 0_u64);
    let load_error = use_state(|| None::<String>);
    let grid_refresh_tick = use_state(|| 0_u32);

    // Redirect legacy #article-request-section bookmarks to /studio#article
    {
        let loc_dep = route_location.clone();
        use_effect_with(loc_dep, move |_| {
            let check_legacy_hash = || {
                let hash = window()
                    .and_then(|w| w.location().hash().ok())
                    .unwrap_or_default();
                if hash == "#article-request-section" || hash == "#article-request-form" {
                    let studio_url = crate::config::route_path("/studio#article");
                    let _ = crate::navigation_context::navigate_spa_to(&studio_url);
                }
            };
            check_legacy_hash();

            let mut listener_opt: Option<EventListener> = None;
            if let Some(win) = window() {
                listener_opt = Some(EventListener::new(&win, "hashchange", move |_| {
                    check_legacy_hash();
                }));
            }
            move || drop(listener_opt)
        });
    }

    let total_pages = {
        let t = *total;
        if t == 0 {
            1
        } else {
            t.div_ceil(PAGE_SIZE)
        }
    };
    let current_page_num = *current_page;

    // Fetch paginated articles
    {
        let articles = articles.clone();
        let loading = loading.clone();
        let total = total.clone();
        let fetch_seq = fetch_seq.clone();
        let load_error = load_error.clone();
        let page = *current_page;
        let tick = *grid_refresh_tick;
        use_effect_with((page, tick), move |(page, _)| {
            let offset = (*page - 1) * PAGE_SIZE;
            let request_id = {
                let mut seq = fetch_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            loading.set(true);
            load_error.set(None);
            let articles = articles.clone();
            let loading = loading.clone();
            let total = total.clone();
            let fetch_seq = fetch_seq.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::fetch_articles(None, None, Some(PAGE_SIZE), Some(offset)).await {
                    Ok(data) => {
                        if *fetch_seq.borrow() != request_id {
                            return;
                        }
                        total.set(data.total);
                        articles.set(data.articles);
                        load_error.set(None);
                    },
                    Err(e) => {
                        if *fetch_seq.borrow() != request_id {
                            return;
                        }
                        web_sys::console::error_1(
                            &format!("Failed to fetch articles: {}", e).into(),
                        );
                        load_error.set(Some(e.to_string()));
                    },
                }
                if *fetch_seq.borrow() != request_id {
                    return;
                }
                loading.set(false);
            });
            || ()
        });
    }

    let save_scroll_position = {
        let location = route_location.clone();
        Callback::from(move |_| {
            if crate::navigation_context::is_return_armed() {
                return;
            }
            let mut state = BTreeMap::new();
            state.insert("page".to_string(), current_page_num.to_string());
            if let Some(loc) = location.as_ref() {
                state.insert("location".to_string(), loc.path().to_string());
            }
            crate::navigation_context::save_context_for_current_page(state);
        })
    };

    {
        let location = route_location.clone();
        use_effect_with((location, current_page_num), move |_| {
            let mut on_scroll_opt: Option<Closure<dyn FnMut(Event)>> = None;

            if !crate::navigation_context::is_return_armed() {
                let persist = move || {
                    let mut state = BTreeMap::new();
                    state.insert("page".to_string(), current_page_num.to_string());
                    crate::navigation_context::save_context_for_current_page(state);
                };

                persist();

                let on_scroll = Closure::wrap(Box::new(move |_: Event| {
                    if crate::navigation_context::is_return_armed() {
                        return;
                    }
                    let mut state = BTreeMap::new();
                    state.insert("page".to_string(), current_page_num.to_string());
                    crate::navigation_context::save_context_for_current_page(state);
                }) as Box<dyn FnMut(_)>);

                if let Some(win) = window() {
                    let _ = win.add_event_listener_with_callback(
                        "scroll",
                        on_scroll.as_ref().unchecked_ref(),
                    );
                }

                on_scroll_opt = Some(on_scroll);
            }

            move || {
                if let Some(on_scroll) = on_scroll_opt {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "scroll",
                            on_scroll.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }

    {
        let current_page = current_page.clone();
        let location_dep = route_location.clone();
        let article_len = articles.len();
        use_effect_with((location_dep, article_len), move |_| {
            if article_len > 0 {
                if let Some(context) =
                    crate::navigation_context::pop_context_if_armed_for_current_page()
                {
                    if let Some(page_num) = context
                        .page_state
                        .get("page")
                        .and_then(|raw| raw.parse::<usize>().ok())
                    {
                        current_page.set(page_num);
                    }

                    let scroll_y = context.scroll_y.max(0.0);
                    Timeout::new(140, move || {
                        if let Some(win) = window() {
                            win.scroll_to_with_x_and_y(0.0, scroll_y);
                        }
                    })
                    .forget();
                }
            }
            || ()
        });
    }

    let go_to_page = {
        let current_page = current_page.clone();
        Callback::from(move |page: usize| {
            current_page.set(page);
            smooth_scroll_to_id("latest-articles-stream");
        })
    };

    let pagination_controls = if total_pages > 1 {
        html! {
            <div class={classes!("mt-12", "flex", "justify-center")}>
                <Pagination
                    current_page={current_page_num}
                    total_pages={total_pages}
                    on_page_change={go_to_page}
                />
            </div>
        }
    } else {
        Html::default()
    };

    let studio_article_href = crate::config::route_path("/studio#article");
    let on_go_studio = {
        let target = studio_article_href.clone();
        Callback::from(move |e: MouseEvent| {
            if e.meta_key() || e.ctrl_key() || e.shift_key() || e.alt_key() || e.button() != 0 {
                return;
            }
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&target);
            if let Some(win) = window() {
                if let Ok(ev) = Event::new("hashchange") {
                    let _ = win.dispatch_event(&ev);
                }
            }
        })
    };

    html! {
        <main class={classes!("min-h-[65vh]", "pb-24", "min-w-0")}>
            // 1. Anthropic Editorial Page Header
            <div class={classes!("max-w-[1200px]", "mx-auto", "px-4", "sm:px-6", "md:px-10")}>
                <header class="pt-12 sm:pt-16 pb-8 sm:pb-10 border-b border-[var(--border)]">
                    <div class="sf-eyebrow">
                        { "LATEST DISPATCHES" }
                    </div>
                    <div class="mt-2 flex flex-col sm:flex-row sm:items-end justify-between gap-4">
                        <div class="min-w-0">
                            <h1 class="m-0 font-serif font-normal text-[clamp(2rem,1.5rem+2vw,3rem)] leading-[1.15] tracking-[-0.015em] text-[var(--text)]">
                                { t::HERO_TITLE }
                            </h1>
                            <p class="mt-3 mb-0 text-[0.95rem] leading-[1.65] text-[var(--muted)] max-w-[60ch]">
                                { t::HERO_DESC }
                            </p>
                        </div>

                        <div class="flex flex-wrap items-center gap-x-4 gap-y-2 shrink-0">
                            if *total > 0 {
                                <span class="font-mono text-xs tabular-nums text-[var(--faint)]">
                                    { format!("{} 篇文章", *total) }
                                </span>
                            }
                            <a
                                href={studio_article_href}
                                onclick={on_go_studio}
                                class="sf-link inline-flex min-h-[40px] items-center text-sm text-[var(--muted)] hover:text-[var(--text)]"
                            >
                                { t::STUDIO_CTA }
                            </a>
                        </div>
                    </div>
                </header>
            </div>

            // 2. Main Editorial Row Article Stream
            <section
                id="latest-articles-stream"
                class="max-w-[1200px] mx-auto px-4 sm:px-6 md:px-10 pt-8 sm:pt-10 scroll-mt-20 min-w-0"
            >
                {
                    if *loading {
                        render_archive_skeleton()
                    } else if let Some(err) = (*load_error).clone() {
                        let on_retry = {
                            let grid_refresh_tick = grid_refresh_tick.clone();
                            Callback::from(move |_| grid_refresh_tick.set(*grid_refresh_tick + 1))
                        };
                        html! {
                            <div class="py-16 max-w-[54ch]" role="alert">
                                <p class="m-0 font-serif text-xl text-[var(--err)]">{ t::LOAD_ERROR_TITLE }</p>
                                <p class="mt-2 mb-5 text-sm text-[var(--muted)] [overflow-wrap:anywhere]">{ err }</p>
                                <button
                                    type="button"
                                    class="sf-btn sf-btn--secondary min-h-[40px]"
                                    onclick={on_retry}
                                >
                                    { fill_one("{}", t::RETRY) }
                                </button>
                            </div>
                        }
                    } else if articles.is_empty() {
                        html! {
                            <div class="py-20">
                                <p class="m-0 font-serif italic text-lg text-[var(--muted)]">{ t::EMPTY }</p>
                            </div>
                        }
                    } else {
                        html! {
                            <>
                                <div class="min-w-0">
                                    { for articles.iter().map(|article| {
                                        html! {
                                            <ArticleCard
                                                key={article.id.clone()}
                                                article={article.clone()}
                                                variant={ArticleCardVariant::Row}
                                                on_before_navigate={Some(save_scroll_position.clone())}
                                            />
                                        }
                                    }) }
                                </div>
                                { pagination_controls }
                            </>
                        }
                    }
                }
            </section>

            <ScrollToTopButton />
        </main>
    }
}
