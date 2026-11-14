use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    components::{scroll_to_top_button::ScrollToTopButton, skeleton::SkeletonCard},
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

                // Editorial Timeline Section
                <div class={classes!("editorial-timeline", "mb-16")}>
                    {
                        if *loading {
                            html! {
                                <div class={classes!("grid", "grid-cols-1", "md:grid-cols-2", "lg:grid-cols-3", "gap-6")}>
                                    { for (0..6).map(|_| html! { <SkeletonCard /> }) }
                                </div>
                            }
                        } else if let Some(err) = (*load_error).clone() {
                            html! {
                                <div class={classes!(
                                    "empty-state",
                                    "text-center",
                                    "py-20",
                                    "px-4",
                                    "bg-[var(--surface)]",
                                    "rounded-2xl",
                                    "border",
                                    "border-[var(--border)]"
                                )}>
                                    <i class={classes!(
                                        "fas",
                                        "fa-triangle-exclamation",
                                        "text-6xl",
                                        "text-amber-500",
                                        "mb-6"
                                    )}></i>
                                    <p class={classes!(
                                        "text-xl",
                                        "font-semibold",
                                        "text-[var(--text)]",
                                        "mb-2"
                                    )}>
                                        { t::LOAD_ERROR_TITLE }
                                    </p>
                                    <p class={classes!("text-sm", "text-[var(--muted)]", "mb-6")}>
                                        { err }
                                    </p>
                                    <button
                                        onclick={on_retry.clone()}
                                        class={classes!("btn-fluent-secondary")}
                                    >
                                        <i class={classes!("fas", "fa-rotate-right", "mr-1")}></i>
                                        { t::RETRY }
                                    </button>
                                </div>
                            }
                        } else if tag_stats.is_empty() {
                            html! {
                                <div class={classes!(
                                    "empty-state",
                                    "text-center",
                                    "py-20",
                                    "px-4",
                                    "bg-[var(--surface)]",
                                    "rounded-2xl",
                                    "border",
                                    "border-[var(--border)]"
                                )}>
                                    <i class={classes!(
                                        "fas",
                                        "fa-tags",
                                        "text-6xl",
                                        "text-[var(--muted)]",
                                        "mb-6"
                                    )}></i>
                                    <p class={classes!("text-xl", "text-[var(--muted)]")}>
                                        { t::EMPTY }
                                    </p>
                                </div>
                            }
                        } else {
                            html! {
                                <div
                                    class={classes!(
                                        "tag-cloud",
                                        "flex",
                                        "flex-wrap",
                                        "justify-center",
                                        "gap-3",
                                        "px-4",
                                        "max-w-5xl",
                                        "mx-auto"
                                    )}
                                    role="list"
                                    aria-label={t::CLOUD_ARIA}
                                >
                                    { for tag_stats.iter().enumerate().map(|(idx, tag_info)| {
                                        let weight = (tag_info.count as f32 / max_count).max(0.35);
                                        let style = format!("--tag-weight: {:.2}; animation-delay: {}ms", weight, idx * 50);
                                        html! {
                                            <div {style}>
                                                <Link<Route>
                                                    to={Route::TagDetail { tag: tag_info.name.clone() }}
                                                    classes={classes!(
                                                        "tag-pill",
                                                        "inline-flex",
                                                        "items-center",
                                                        "gap-2",
                                                        "px-5",
                                                        "py-3",
                                                        "border",
                                                        "border-[var(--border)]",
                                                        "rounded-full",
                                                        "bg-[var(--surface)]",
                                                        "text-[var(--text)]",
                                                        "font-medium",
                                                        "transition-all",
                                                        "duration-300",
                                                        "ease-[cubic-bezier(0.34,1.56,0.64,1)]",
                                                        "hover:-translate-y-1",
                                                        "hover:scale-105",
                                                        "hover:border-[var(--primary)]",
                                                        "hover:shadow-[var(--shadow-8)]",
                                                        "hover:bg-gradient-to-br",
                                                        "hover:from-[var(--primary)]/10",
                                                        "hover:to-purple-500/10",
                                                        "group"
                                                    )}
                                                >
                                                    <span
                                                        class={classes!(
                                                            "text-[calc(1rem+var(--tag-weight,0.4)*0.35rem)]",
                                                            "font-semibold",
                                                            "transition-colors",
                                                            "duration-300",
                                                            "group-hover:text-[var(--primary)]"
                                                        )}
                                                    >
                                                        { format!("#{}", &tag_info.name) }
                                                    </span>
                                                    <span class={classes!(
                                                        "text-sm",
                                                        "text-[var(--muted)]",
                                                        "px-2",
                                                        "py-0.5",
                                                        "bg-[var(--surface-alt)]",
                                                        "rounded-full",
                                                        "transition-all",
                                                        "duration-300",
                                                        "group-hover:bg-[var(--primary)]/20",
                                                        "group-hover:text-[var(--primary)]"
                                                    )}>
                                                        { tag_info.count }
                                                    </span>
                                                </Link<Route>>
                                            </div>
                                        }
                                    }) }
                                </div>
                            }
                        }
                    }
                </div>
            </div>
            <ScrollToTopButton />
        </main>
    }
}
