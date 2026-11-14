use std::collections::BTreeMap;

use gloo_timers::callback::Timeout;
use static_flow_shared::ArticleListItem;
use wasm_bindgen::JsCast;
use web_sys::{window, Event};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    components::{
        article_card::{ArticleCard, ArticleCardVariant},
        scroll_to_top_button::ScrollToTopButton,
    },
    i18n::{current::tag_detail_page as t, fill_one},
    pages::posts::render_archive_skeleton,
    router::Route,
};

#[derive(Properties, Clone, PartialEq)]
pub struct TagDetailProps {
    pub tag: String,
}

#[function_component(TagDetailPage)]
pub fn tag_detail_page(props: &TagDetailProps) -> Html {
    let normalized = props.tag.trim().to_string();
    let filter_value = if normalized.is_empty() { None } else { Some(normalized) };
    let display_tag = filter_value
        .clone()
        .unwrap_or_else(|| t::UNNAMED.to_string());

    let articles = use_state(Vec::<ArticleListItem>::new);
    let loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0_u64);

    {
        let articles = articles.clone();
        let tag = filter_value.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        let tick = *refresh_tick;
        use_effect_with((tag.clone(), tick), move |_| {
            loading.set(true);
            load_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let tag_ref = tag.as_deref();
                match crate::api::fetch_all_articles(tag_ref, None).await {
                    Ok(mut data) => {
                        data.sort_by(|a, b| b.date.cmp(&a.date));
                        articles.set(data);
                        loading.set(false);
                    },
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to fetch articles: {}", e).into(),
                        );
                        load_error.set(Some(e.to_string()));
                        loading.set(false);
                    },
                }
            });
            || ()
        });
    }

    let on_retry = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_: MouseEvent| {
            refresh_tick.set(*refresh_tick + 1);
        })
    };

    let filtered = (*articles).clone();
    let total_posts = filtered.len();

    let empty_message = if let Some(tag_value) = filter_value.as_ref() {
        fill_one(t::EMPTY_TEMPLATE, tag_value)
    } else {
        t::INVALID_NAME.to_string()
    };

    {
        let tag = filter_value.clone();
        use_effect_with((tag.clone(), total_posts), move |_| {
            let persist = move || {
                if crate::navigation_context::is_return_armed() {
                    return;
                }
                let mut state = BTreeMap::new();
                if let Some(tag) = tag.as_ref() {
                    state.insert("tag".to_string(), tag.clone());
                }
                crate::navigation_context::save_context_for_current_page(state);
            };

            persist();

            let on_scroll = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: Event| {
                persist();
            })
                as Box<dyn FnMut(_)>);

            if let Some(win) = window() {
                let _ = win
                    .add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());
            }

            move || {
                if let Some(win) = window() {
                    let _ = win.remove_event_listener_with_callback(
                        "scroll",
                        on_scroll.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    {
        let tag = filter_value.clone();
        use_effect_with((tag, total_posts), move |_| {
            if total_posts > 0 {
                if let Some(context) =
                    crate::navigation_context::pop_context_if_armed_for_current_page()
                {
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

    html! {
        <main class={classes!("min-h-[65vh]", "pb-24")}>
            <div class={classes!("max-w-[1200px]", "mx-auto", "px-6", "md:px-10")}>
                <header class="pt-16 pb-10 border-b border-[var(--border)]">
                    <div class="flex flex-wrap items-center gap-3 mb-3">
                        <Link<Route>
                            to={Route::Tags}
                            classes="sf-btn sf-btn--ghost sf-btn--sm !px-0 no-underline font-mono text-xs"
                        >
                            { format!("← {}", t::ARCHIVE_BADGE) }
                        </Link<Route>>
                    </div>
                    <h1 class="m-0 font-serif font-normal text-[clamp(2rem,1.5rem+2vw,3rem)] leading-[1.15] tracking-[-0.015em] text-[var(--text)]">
                        { format!("#{}", display_tag) }
                    </h1>
                    <p class="mt-3 mb-0 font-mono text-xs tabular-nums text-[var(--faint)]">
                        if total_posts > 0 {
                            { fill_one(t::COLLECTED_COUNT_TEMPLATE, total_posts) }
                        } else {
                            { t::NO_CONTENT }
                        }
                    </p>
                </header>

                {
                    if *loading {
                        render_archive_skeleton()
                    } else if let Some(err) = (*load_error).clone() {
                        html! {
                            <div class="py-14 max-w-[54ch]" role="alert">
                                <p class="m-0 font-serif text-xl text-[var(--err)]">{ t::LOAD_ERROR_TITLE }</p>
                                <p class="mt-2 mb-5 text-sm text-[var(--muted)]">{ err }</p>
                                <button
                                    type="button"
                                    onclick={on_retry}
                                    class="sf-btn sf-btn--secondary"
                                >
                                    { t::RETRY }
                                </button>
                            </div>
                        }
                    } else if filtered.is_empty() {
                        html! {
                            <div class="py-20">
                                <p class="m-0 font-serif italic text-lg text-[var(--muted)]">{ empty_message }</p>
                            </div>
                        }
                    } else {
                        html! {
                            <div>
                                { for filtered.iter().map(|article| html! {
                                    <ArticleCard
                                        key={article.id.clone()}
                                        article={article.clone()}
                                        variant={ArticleCardVariant::Row}
                                    />
                                }) }
                            </div>
                        }
                    }
                }
            </div>
            <ScrollToTopButton />
        </main>
    }
}
