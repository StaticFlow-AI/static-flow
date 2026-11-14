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
        <main class={classes!("min-h-[65vh]", "pb-24")}>
            <div class={classes!("max-w-[1200px]", "mx-auto", "px-6", "md:px-10")}>
                <header class="pt-16 pb-10 border-b border-[var(--border)]">
                    <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                        { "TABLE OF CONTENTS · CATEGORIES" }
                    </div>
                    <h1 class="mt-3 mb-0 font-serif font-normal text-[clamp(2rem,1.5rem+2vw,3rem)] leading-[1.15] tracking-[-0.015em] text-[var(--text)]">
                        { t::HERO_TITLE }
                    </h1>
                    <p class="mt-3 mb-0 text-[0.95rem] leading-[1.65] text-[var(--muted)]">
                        { fill_two(t::HERO_DESC_TEMPLATE, total_categories, total_articles) }
                    </p>
                    <div class={classes!("flex", "flex-wrap", "items-center", "gap-3", "mt-4", "font-mono", "text-xs", "tabular-nums", "text-[var(--faint)]")}>
                        <span>{ fill_one(t::HERO_BADGE_TEMPLATE, total_categories) }</span>
                        <span aria-hidden="true">{ "│" }</span>
                        <Link<Route> to={Route::Tags} classes="sf-link no-underline hover:underline text-[var(--muted)]">
                            { "切换至标签索引 →" }
                        </Link<Route>>
                    </div>
                </header>

                <section class="py-12">
                    {
                        if *loading {
                            html! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-x-14 gap-y-6" aria-hidden="true">
                                    { for (0..6).map(|idx| html! {
                                        <div key={idx} class="py-2">
                                            <div class="flex items-baseline gap-3">
                                                <span class="skeleton block h-3 w-5 rounded" />
                                                <span class="skeleton block h-6 w-36 rounded" />
                                                <span class="flex-1 border-b border-dotted border-[var(--border)] mb-1" />
                                                <span class="skeleton block h-3.5 w-10 rounded" />
                                            </div>
                                            <span class="skeleton block h-3 w-2/3 rounded mt-2 ml-8" />
                                        </div>
                                    }) }
                                </div>
                            }
                        } else if let Some(err) = (*load_error).clone() {
                            html! {
                                <div class="py-10 max-w-[54ch]" role="alert">
                                    <p class="m-0 font-serif text-xl text-[var(--err)]">{ "分类加载失败" }</p>
                                    <p class="mt-2 mb-5 text-sm text-[var(--muted)]">{ err }</p>
                                    <button
                                        onclick={on_retry}
                                        class="sf-btn sf-btn--secondary"
                                        type="button"
                                    >
                                        { "重试" }
                                    </button>
                                </div>
                            }
                        } else if categories.is_empty() {
                            html! {
                                <div class="py-12">
                                    <p class="m-0 font-serif italic text-lg text-[var(--muted)]">{ t::EMPTY }</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-x-14 gap-y-5">
                                    { for categories.iter().enumerate().map(|(idx, category)| {
                                        html! {
                                            <Link<Route>
                                                key={category.name.clone()}
                                                to={Route::CategoryDetail { category: category.name.clone() }}
                                                classes="group block py-2 no-underline"
                                            >
                                                <div class="flex items-baseline gap-3">
                                                    <span class="font-mono text-xs tabular-nums text-[var(--faint)] shrink-0">
                                                        { format!("{:02}", idx + 1) }
                                                    </span>
                                                    <h2 class="m-0 font-serif font-normal text-[1.35rem] leading-[1.25] tracking-[-0.01em] text-[var(--text)] group-hover:text-[var(--primary)] transition-colors duration-150 shrink-0">
                                                        { &category.name }
                                                    </h2>
                                                    <span
                                                        class="flex-1 min-w-[24px] border-b border-dotted border-[var(--border-strong)] mb-1.5"
                                                        aria-hidden="true"
                                                    />
                                                    <span class="font-mono text-xs tabular-nums text-[var(--faint)] group-hover:text-[var(--primary)] transition-colors duration-150 shrink-0">
                                                        { fill_one(t::COUNT_TEMPLATE, category.count) }
                                                    </span>
                                                </div>
                                                if !category.description.trim().is_empty() {
                                                    <p class="m-0 mt-1.5 pl-7 text-xs leading-relaxed text-[var(--muted)] line-clamp-1">
                                                        { &category.description }
                                                    </p>
                                                }
                                            </Link<Route>>
                                        }
                                    }) }
                                </div>
                            }
                        }
                    }
                </section>
            </div>
            <ScrollToTopButton />
        </main>
    }
}
