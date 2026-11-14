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

    html! {
        <main class={classes!("min-h-[65vh]", "pb-24")}>
            <div class={classes!("max-w-[1200px]", "mx-auto", "px-6", "md:px-10")}>
                <header class="pt-16 pb-10 border-b border-[var(--border)]">
                    <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                        { "TAXONOMY · TAG INDEX" }
                    </div>
                    <h1 class="mt-3 mb-0 font-serif font-normal text-[clamp(2rem,1.5rem+2vw,3rem)] leading-[1.15] tracking-[-0.015em] text-[var(--text)]">
                        { t::HERO_TITLE }
                    </h1>
                    <p class="mt-3 mb-0 text-[0.95rem] leading-[1.65] text-[var(--muted)]">
                        { fill_two(t::HERO_DESC_TEMPLATE, total_tags, total_articles) }
                    </p>
                    <div class={classes!("flex", "flex-wrap", "items-center", "gap-3", "mt-4", "font-mono", "text-xs", "tabular-nums", "text-[var(--faint)]")}>
                        <span>{ fill_one(t::TAG_COUNT_TEMPLATE, total_tags) }</span>
                        <span aria-hidden="true">{ "│" }</span>
                        <span>{ fill_one(t::ARTICLE_COUNT_TEMPLATE, total_articles) }</span>
                        <span aria-hidden="true">{ "│" }</span>
                        <Link<Route> to={Route::Categories} classes="sf-link no-underline hover:underline text-[var(--muted)]">
                            { "切换至分类目录 →" }
                        </Link<Route>>
                    </div>
                </header>

                <section class="py-12" aria-label={t::CLOUD_ARIA}>
                    {
                        if *loading {
                            html! {
                                <div class="flex flex-wrap items-center gap-3" aria-hidden="true">
                                    { for [5.5_f32, 7.0, 4.5, 8.5, 6.0, 5.0, 9.0, 4.0, 6.5, 7.5, 5.0, 8.0, 4.5, 6.0]
                                        .iter()
                                        .enumerate()
                                        .map(|(idx, w)| html! {
                                            <span
                                                key={idx}
                                                class="skeleton block h-7 rounded-full"
                                                style={format!("width: {w}rem")}
                                            />
                                        })
                                    }
                                </div>
                            }
                        } else if let Some(err) = (*load_error).clone() {
                            html! {
                                <div class="py-10 max-w-[54ch]" role="alert">
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
                        } else if tag_stats.is_empty() {
                            html! {
                                <div class="py-12">
                                    <p class="m-0 font-serif italic text-lg text-[var(--muted)]">{ t::EMPTY }</p>
                                </div>
                            }
                        } else {
                            html! {
                                <ul class="m-0 p-0 list-none flex flex-wrap items-center gap-x-4 gap-y-3.5">
                                    { for tag_stats.iter().map(|tag_info| {
                                        html! {
                                            <li key={tag_info.name.clone()} class="m-0 inline-flex items-center gap-1.5">
                                                <Link<Route>
                                                    to={Route::TagDetail { tag: tag_info.name.clone() }}
                                                    classes="sf-chip no-underline hover:!text-[var(--text)]"
                                                >
                                                    { format!("#{}", tag_info.name) }
                                                </Link<Route>>
                                                <span class="font-mono text-xs tabular-nums text-[var(--faint)]">
                                                    { tag_info.count }
                                                </span>
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
