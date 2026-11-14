use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    components::scroll_to_top_button::ScrollToTopButton,
    i18n::{current::categories_page as t, fill_one, fill_two},
    router::Route,
};

#[function_component(CategoriesPage)]
pub fn categories_page() -> Html {
    let categories = use_state(Vec::<crate::api::CategoryInfo>::new);
    let loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0_u64);

    {
        let categories = categories.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        use_effect_with(*refresh_tick, move |_| {
            loading.set(true);
            load_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::fetch_categories().await {
                    Ok(data) => {
                        categories.set(data);
                        loading.set(false);
                    },
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to fetch categories: {}", e).into(),
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

    let total_categories = categories.len();
    let total_articles: usize = categories.iter().map(|c| c.count).sum();

    html! {
        <main class={classes!("pb-20")}>
            <div class={classes!("container")}>
                <header class="sf-page-hero">
                    <div class="sf-eyebrow">{ t::HERO_INDEX }</div>
                    <h1 class="sf-page-title">{ t::HERO_TITLE }</h1>
                    <p class="sf-page-sub">
                        { fill_two(t::HERO_DESC_TEMPLATE, total_categories, total_articles) }
                    </p>
                    <div class={classes!("flex", "flex-wrap", "items-center", "gap-2.5", "mt-4")}>
                        <span class="sf-chip">
                            <i class={classes!("fas", "fa-folder-open", "text-[var(--primary)]")} aria-hidden="true"></i>
                            <span>{ fill_one(t::HERO_BADGE_TEMPLATE, total_categories) }</span>
                        </span>
                    </div>
                </header>

                {
                    if *loading {
                        html! {
                            <div class="sf-category-grid" aria-hidden="true">
                                { for (0..6).map(|_| html! {
                                    <div class="sf-category-card pointer-events-none">
                                        <div class="sf-category-card__top">
                                            <span class="skeleton block h-3 w-6 rounded" />
                                            <span class="skeleton block h-5 w-16 rounded-full" />
                                        </div>
                                        <span class="skeleton block h-5 w-2/5 rounded mt-1" />
                                        <span class="skeleton block h-3 w-full rounded" />
                                        <span class="skeleton block h-3 w-4/5 rounded" />
                                    </div>
                                }) }
                            </div>
                        }
                    } else if let Some(err) = (*load_error).clone() {
                        html! {
                            <div class="sf-state sf-state--error" role="alert">
                                <span class="sf-state__icon">
                                    <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                                </span>
                                <p class="sf-state__title">{ "Couldn't load" }</p>
                                <p class="sf-state__body">{ err }</p>
                                <div class="sf-state__action">
                                    <button
                                        onclick={on_retry.clone()}
                                        class="btn-fluent-secondary"
                                        type="button"
                                    >
                                        <i class="fas fa-rotate-right text-xs" aria-hidden="true"></i>
                                        { "Retry" }
                                    </button>
                                </div>
                            </div>
                        }
                    } else if categories.is_empty() {
                        html! {
                            <div class="sf-state">
                                <span class="sf-state__icon">
                                    <i class="fas fa-folder-open" aria-hidden="true"></i>
                                </span>
                                <p class="sf-state__title">{ t::EMPTY }</p>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="sf-category-grid mb-16">
                                { for categories.iter().enumerate().map(|(idx, category)| {
                                    html! {
                                        <Link<Route>
                                            to={Route::CategoryDetail { category: category.name.clone() }}
                                            classes="sf-category-card"
                                        >
                                            <div class="sf-category-card__top">
                                                <span class="sf-category-card__index">
                                                    { format!("{:02}", idx + 1) }
                                                </span>
                                                <span class="sf-category-card__count">
                                                    <i class="far fa-file-alt" aria-hidden="true"></i>
                                                    { fill_one(t::COUNT_TEMPLATE, category.count) }
                                                </span>
                                            </div>
                                            <h3 class="sf-category-card__name">{ &category.name }</h3>
                                            <p class="sf-category-card__desc">{ &category.description }</p>
                                            <span class="sf-category-card__foot" aria-hidden="true">
                                                { "浏览分类" }
                                                <i class="fas fa-arrow-right"></i>
                                            </span>
                                        </Link<Route>>
                                    }
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
