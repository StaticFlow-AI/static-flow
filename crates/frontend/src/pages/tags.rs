use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    components::scroll_to_top_button::ScrollToTopButton,
    i18n::{current::tags_page as t, fill_one, fill_two},
    router::Route,
};

#[function_component(TagsPage)]
pub fn tags_page() -> Html {
    let tag_stats = use_state(Vec::<crate::api::TagInfo>::new);
    let loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0_u64);

    {
        let tag_stats = tag_stats.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        use_effect_with(*refresh_tick, move |_| {
            loading.set(true);
            load_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::fetch_tags().await {
                    Ok(data) => {
                        tag_stats.set(data);
                        loading.set(false);
                    },
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to fetch tags: {}", e).into());
                        load_error.set(Some(e));
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

    let total_tags = tag_stats.len();
    let total_articles: usize = tag_stats.iter().map(|t| t.count).sum();
    let max_count = tag_stats.iter().map(|t| t.count as f32).fold(1.0, f32::max);

    html! {
        <main class={classes!("pb-20")}>
            <div class={classes!("container")}>
                <header class="sf-page-hero">
                    <div class="sf-eyebrow">{ t::HERO_INDEX }</div>
                    <h1 class="sf-page-title">{ t::HERO_TITLE }</h1>
                    <p class="sf-page-sub">
                        { fill_two(t::HERO_DESC_TEMPLATE, total_tags, total_articles) }
                    </p>
                    <div class={classes!("flex", "flex-wrap", "items-center", "gap-2.5", "mt-4")}>
                        <span class="sf-chip">
                            <i class={classes!("fas", "fa-tags", "text-[var(--primary)]")} aria-hidden="true"></i>
                            <span>{ fill_one(t::TAG_COUNT_TEMPLATE, total_tags) }</span>
                        </span>
                        <span class="sf-chip">
                            <i class={classes!("fas", "fa-book", "text-[var(--primary)]")} aria-hidden="true"></i>
                            <span>{ fill_one(t::ARTICLE_COUNT_TEMPLATE, total_articles) }</span>
                        </span>
                    </div>
                </header>

                <section class="mb-16" aria-label={t::CLOUD_ARIA}>
                    {
                        if *loading {
                            html! {
                                <div class="sf-tag-cloud" aria-hidden="true">
                                    { for [5.5_f32, 7.0, 4.5, 8.5, 6.0, 5.0, 9.0, 4.0, 6.5, 7.5, 5.0, 8.0, 4.5, 6.0]
                                        .iter()
                                        .map(|w| html! {
                                            <span
                                                class="skeleton block h-9 rounded-full"
                                                style={format!("width: {w}rem")}
                                            />
                                        })
                                    }
                                </div>
                            }
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
                        } else if tag_stats.is_empty() {
                            html! {
                                <div class="sf-state">
                                    <span class="sf-state__icon">
                                        <i class="fas fa-tags" aria-hidden="true"></i>
                                    </span>
                                    <p class="sf-state__title">{ t::EMPTY }</p>
                                </div>
                            }
                        } else {
                            html! {
                                <ul class="sf-tag-cloud">
                                    { for tag_stats.iter().map(|tag_info| {
                                        let weight = (tag_info.count as f32 / max_count).clamp(0.0, 1.0);
                                        let style = format!("--tag-weight: {weight:.2}");
                                        html! {
                                            <li class="m-0" {style}>
                                                <Link<Route>
                                                    to={Route::TagDetail { tag: tag_info.name.clone() }}
                                                    classes="sf-tag"
                                                >
                                                    <span class="sf-tag__name">
                                                        <span class="sf-tag__hash" aria-hidden="true">{ "#" }</span>
                                                        { &tag_info.name }
                                                    </span>
                                                    <span class="sf-tag__count">{ tag_info.count }</span>
                                                </Link<Route>>
                                            </li>
                                        }
                                    }) }
                                </ul>
                            }
                        }
                    }
                </section>
            </div>
            <ScrollToTopButton />
        </main>
    }
}
