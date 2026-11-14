use std::collections::BTreeMap;

use gloo_timers::callback::Timeout;
use static_flow_shared::ArticleListItem;
use wasm_bindgen::JsCast;
use web_sys::{window, Event};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    components::scroll_to_top_button::ScrollToTopButton,
    i18n::{current::tag_detail_page as t, fill_one},
    pages::posts::{group_articles_by_year, render_archive_skeleton, render_archive_years},
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
                    Ok(data) => {
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
    let grouped_by_year = group_articles_by_year(&filtered);

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
        <main class={classes!("tag-detail-page", "min-h-[60vh]", "pb-20")}>
            <div class={classes!("container")}>
                <header class="sf-page-hero">
                    <div class="flex items-center gap-3 mb-3">
                        <Link<Route>
                            to={Route::Tags}
                            classes="sf-chip"
                        >
                            <i class="fas fa-arrow-left text-[10px]" aria-hidden="true"></i>
                            <span>{ t::ARCHIVE_BADGE }</span>
                        </Link<Route>>
                    </div>
                    <h1 class="sf-page-title">{ format!("#{}", display_tag) }</h1>
                    <p class="sf-page-sub">
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
                            <div class="sf-state sf-state--error" role="alert">
                                <span class="sf-state__icon">
                                    <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                                </span>
                                <p class="sf-state__title">{ t::LOAD_ERROR_TITLE }</p>
                                <p class="sf-state__body">{ err }</p>
                                <div class="sf-state__action">
                                    <button
                                        type="button"
                                        onclick={on_retry.clone()}
                                        class="btn-fluent-secondary"
                                    >
                                        <i class="fas fa-rotate-right text-xs" aria-hidden="true"></i>
                                        { t::RETRY }
                                    </button>
                                </div>
                            </div>
                        }
                    } else if grouped_by_year.is_empty() {
                        html! {
                            <div class="sf-state">
                                <span class="sf-state__icon">
                                    <i class="fas fa-hashtag" aria-hidden="true"></i>
                                </span>
                                <p class="sf-state__title">{ empty_message }</p>
                            </div>
                        }
                    } else {
                        render_archive_years(&grouped_by_year, true)
                    }
                }
            </div>
            <ScrollToTopButton />
        </main>
    }
}
