use std::collections::{BTreeMap, HashMap};

use gloo_timers::callback::Timeout;
use serde::{Deserialize, Serialize};
use static_flow_shared::ArticleListItem;
use wasm_bindgen::JsCast;
use web_sys::{window, Event};
use yew::prelude::*;
use yew_router::prelude::{use_location, Link};

use crate::{
    components::{
        article_card::{ArticleCard, ArticleCardVariant},
        scroll_to_top_button::ScrollToTopButton,
    },
    i18n::{current::posts_page as t, fill_one},
    router::Route,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PostsQuery {
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

impl PostsQuery {
    fn trim_field(field: Option<String>) -> Option<String> {
        field.and_then(|raw| {
            let trimmed = raw.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
    }

    pub fn normalized(mut self) -> Self {
        self.tag = Self::trim_field(self.tag.take());
        self.category = Self::trim_field(self.category.take());
        self
    }

    pub fn has_filters(&self) -> bool {
        self.tag.is_some() || self.category.is_some()
    }
}

#[function_component(PostsPage)]
pub fn posts_page() -> Html {
    let location = use_location();
    let query = location
        .as_ref()
        .and_then(|loc| loc.query::<PostsQuery>().ok())
        .unwrap_or_default()
        .normalized();

    let articles = use_state(Vec::<ArticleListItem>::new);
    let loading = use_state(|| true);
    let load_error = use_state(|| None::<String>);
    let refresh = use_state(|| 0u32);
    let expanded_years = use_state(HashMap::<i32, bool>::new);

    {
        let articles = articles.clone();
        let loading = loading.clone();
        let load_error = load_error.clone();
        let tag = query.tag.clone();
        let category = query.category.clone();

        use_effect_with((tag.clone(), category.clone(), *refresh), move |_| {
            loading.set(true);
            load_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let tag_ref = tag.as_deref();
                let category_ref = category.as_deref();

                match crate::api::fetch_all_articles(tag_ref, category_ref).await {
                    Ok(data) => articles.set(data),
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to fetch articles: {}", e).into(),
                        );
                        load_error.set(Some(e.to_string()));
                    },
                }
                loading.set(false);
            });
            || ()
        });
    }

    let filtered = (*articles).clone();
    let total_posts = filtered.len();
    let grouped_by_year = group_articles_by_year(&filtered);

    let filter_label = match (&query.tag, &query.category) {
        (Some(tag), Some(category)) => format!("#{tag} · {category}"),
        (Some(tag), None) => format!("#{tag}"),
        (None, Some(category)) => category.clone(),
        (None, None) => String::new(),
    };

    let description = if total_posts == 0 {
        if query.has_filters() {
            t::DESC_EMPTY_FILTERED.to_string()
        } else {
            t::DESC_EMPTY_ALL.to_string()
        }
    } else if query.has_filters() {
        fill_one(t::DESC_FILTERED_TEMPLATE, total_posts)
    } else {
        fill_one(t::DESC_ALL_TEMPLATE, total_posts)
    };

    let toggle_year = {
        let expanded_years = expanded_years.clone();
        Callback::from(move |year: i32| {
            expanded_years.set({
                let mut map = (*expanded_years).clone();
                let next = !map.get(&year).copied().unwrap_or(false);
                map.insert(year, next);
                map
            });
        })
    };

    let expanded_years_serialized = {
        let mut years = (*expanded_years)
            .iter()
            .filter_map(|(year, expanded)| if *expanded { Some(*year) } else { None })
            .collect::<Vec<_>>();
        years.sort_unstable();
        years
            .into_iter()
            .map(|year| year.to_string())
            .collect::<Vec<_>>()
            .join(",")
    };

    {
        let location_dep = location.clone();
        let tag = query.tag.clone();
        let category = query.category.clone();
        let expanded_years_serialized = expanded_years_serialized.clone();
        use_effect_with(
            (
                location_dep.clone(),
                tag.clone(),
                category.clone(),
                expanded_years_serialized.clone(),
                total_posts,
            ),
            move |_| {
                let persist = move || {
                    if crate::navigation_context::is_return_armed() {
                        return;
                    }
                    let mut state = BTreeMap::new();
                    if let Some(value) = tag.as_ref() {
                        state.insert("tag".to_string(), value.clone());
                    }
                    if let Some(value) = category.as_ref() {
                        state.insert("category".to_string(), value.clone());
                    }
                    if !expanded_years_serialized.is_empty() {
                        state.insert(
                            "expanded_years".to_string(),
                            expanded_years_serialized.clone(),
                        );
                    }
                    crate::navigation_context::save_context_for_current_page(state);
                };

                persist();

                let on_scroll = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: Event| {
                    persist();
                })
                    as Box<dyn FnMut(_)>);

                if let Some(win) = window() {
                    let _ = win.add_event_listener_with_callback(
                        "scroll",
                        on_scroll.as_ref().unchecked_ref(),
                    );
                }

                move || {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "scroll",
                            on_scroll.as_ref().unchecked_ref(),
                        );
                    }
                }
            },
        );
    }

    {
        let location_dep = location.clone();
        let expanded_years = expanded_years.clone();
        use_effect_with((location_dep, total_posts), move |_| {
            if total_posts > 0 {
                if let Some(context) =
                    crate::navigation_context::pop_context_if_armed_for_current_page()
                {
                    if let Some(raw) = context.page_state.get("expanded_years") {
                        let mut restored = HashMap::<i32, bool>::new();
                        for value in raw.split(',') {
                            if let Ok(year) = value.parse::<i32>() {
                                restored.insert(year, true);
                            }
                        }
                        if !restored.is_empty() {
                            expanded_years.set(restored);
                        }
                    }
                    let scroll_y = context.scroll_y.max(0.0);
                    Timeout::new(150, move || {
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
                    <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                        { "ARCHIVE · CHRONOLOGICAL INDEX" }
                    </div>
                    <h1 class="mt-3 mb-0 font-serif font-normal text-[clamp(2rem,1.5rem+2vw,3rem)] leading-[1.15] tracking-[-0.015em] text-[var(--text)]">
                        { t::HERO_TITLE }
                    </h1>
                    <p class="mt-3 mb-0 text-[0.95rem] leading-[1.65] text-[var(--muted)] max-w-[62ch]">
                        { description }
                    </p>

                    <div class="mt-5 flex flex-wrap items-center gap-4 text-xs font-mono tabular-nums text-[var(--faint)]">
                        <Link<Route> to={Route::Tags} classes="sf-link no-underline hover:underline text-[var(--muted)]">
                            { "按标签索引 →" }
                        </Link<Route>>
                        <span aria-hidden="true">{ "·" }</span>
                        <Link<Route> to={Route::Categories} classes="sf-link no-underline hover:underline text-[var(--muted)]">
                            { "按分类目录 →" }
                        </Link<Route>>
                    </div>

                    if query.has_filters() {
                        <div class={classes!("flex", "flex-wrap", "gap-2.5", "items-center", "mt-5")}>
                            <span class="sf-chip !bg-[var(--primary-soft)] !text-[var(--primary)]">
                                { filter_label }
                            </span>
                            <Link<Route> to={Route::Posts} classes="sf-btn sf-btn--ghost sf-btn--sm no-underline">
                                { format!("× {}", t::FILTER_CLEAR) }
                            </Link<Route>>
                        </div>
                    }
                </header>

                {
                    if *loading {
                        render_archive_skeleton()
                    } else if let Some(err) = (*load_error).clone() {
                        let on_retry = {
                            let refresh = refresh.clone();
                            Callback::from(move |_| refresh.set(*refresh + 1))
                        };
                        html! {
                            <div class="py-16 max-w-[54ch]" role="alert">
                                <p class="m-0 font-serif text-xl text-[var(--err)]">{ t::ERROR_TITLE }</p>
                                <p class="mt-2 mb-5 text-sm text-[var(--muted)]">{ err }</p>
                                <button
                                    type="button"
                                    class="sf-btn sf-btn--secondary"
                                    onclick={on_retry}
                                >
                                    { t::ERROR_RETRY }
                                </button>
                            </div>
                        }
                    } else if grouped_by_year.is_empty() {
                        html! {
                            <div class="py-20">
                                <p class="m-0 font-serif italic text-lg text-[var(--muted)]">{ t::EMPTY }</p>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="divide-y divide-[var(--border)]">
                                { for grouped_by_year.iter().map(|(year, posts)| {
                                    let year_value = *year;
                                    let total_count = posts.len();
                                    let should_collapse = total_count > 20;
                                    let is_expanded = (*expanded_years).get(&year_value).copied().unwrap_or(false);
                                    let visible_count = if should_collapse && !is_expanded {
                                        total_count.min(10)
                                    } else {
                                        total_count
                                    };
                                    let remaining = total_count.saturating_sub(visible_count);
                                    let heading_id = format!("archive-year-{year_value}");

                                    html! {
                                        <section
                                            key={year_value}
                                            class="grid grid-cols-1 lg:grid-cols-[120px_1fr] gap-4 lg:gap-12 py-10"
                                            aria-labelledby={heading_id.clone()}
                                        >
                                            <div class="lg:sticky lg:top-24 self-start flex lg:flex-col items-baseline gap-3 lg:gap-1.5 pt-1">
                                                <h2
                                                    class="m-0 font-serif font-normal text-3xl text-[var(--faint)] tabular-nums leading-none"
                                                    id={heading_id}
                                                >
                                                    { year_value }
                                                </h2>
                                                <span class="font-mono text-xs tabular-nums text-[var(--faint)]">
                                                    { fill_one(t::YEAR_COUNT_TEMPLATE, total_count) }
                                                </span>
                                            </div>

                                            <div class="min-w-0">
                                                <div class="[&>article:first-child]:pt-0 [&>article:last-child]:border-b-0">
                                                    { for posts.iter().take(visible_count).map(|article| html! {
                                                        <ArticleCard
                                                            key={article.id.clone()}
                                                            article={article.clone()}
                                                            variant={ArticleCardVariant::Row}
                                                        />
                                                    }) }
                                                </div>

                                                {
                                                    if should_collapse {
                                                        let button_label = if is_expanded {
                                                            t::COLLAPSE.to_string()
                                                        } else {
                                                            fill_one(t::EXPAND_REMAINING_TEMPLATE, remaining)
                                                        };
                                                        let toggle_cb = toggle_year.clone();
                                                        let year_for_toggle = year_value;
                                                        let onclick = Callback::from(move |_| toggle_cb.emit(year_for_toggle));
                                                        html! {
                                                            <div class="pt-4 border-t border-[var(--border)]">
                                                                <button
                                                                    type="button"
                                                                    class="sf-btn sf-btn--ghost sf-btn--sm"
                                                                    {onclick}
                                                                    aria-expanded={is_expanded.to_string()}
                                                                    aria-label={fill_one(t::YEAR_TOGGLE_ARIA_TEMPLATE, year_value)}
                                                                >
                                                                    { button_label }
                                                                    <span aria-hidden="true">
                                                                        { if is_expanded { "↑" } else { "↓" } }
                                                                    </span>
                                                                </button>
                                                            </div>
                                                        }
                                                    } else {
                                                        Html::default()
                                                    }
                                                }
                                            </div>
                                        </section>
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

/// One editorial archive row helper retained for callers using year-grouped
/// rendering.
#[allow(
    dead_code,
    reason = "Retained as a shared archive row helper alongside ArticleCardVariant::Row."
)]
pub(crate) fn render_archive_row(article: &ArticleListItem, _show_category: bool) -> Html {
    html! {
        <ArticleCard
            key={article.id.clone()}
            article={article.clone()}
            variant={ArticleCardVariant::Row}
        />
    }
}

/// Year-grouped archive without collapsing (tag / category detail pages).
#[allow(dead_code, reason = "Retained for callers that prefer year-grouped editorial rows.")]
pub(crate) fn render_archive_years(
    grouped_by_year: &[(i32, Vec<ArticleListItem>)],
    _show_category: bool,
) -> Html {
    html! {
        <div class="divide-y divide-[var(--border)]">
            { for grouped_by_year.iter().map(|(year, posts)| {
                let heading_id = format!("archive-year-{year}");
                html! {
                    <section
                        class="grid grid-cols-1 lg:grid-cols-[120px_1fr] gap-4 lg:gap-12 py-10"
                        key={*year}
                        aria-labelledby={heading_id.clone()}
                    >
                        <div class="lg:sticky lg:top-24 self-start flex lg:flex-col items-baseline gap-3 lg:gap-1.5 pt-1">
                            <h2 class="m-0 font-serif font-normal text-3xl text-[var(--faint)] tabular-nums leading-none" id={heading_id}>
                                { *year }
                            </h2>
                            <span class="font-mono text-xs tabular-nums text-[var(--faint)]">
                                { fill_one(t::YEAR_COUNT_TEMPLATE, posts.len()) }
                            </span>
                        </div>
                        <div class="min-w-0 [&>article:first-child]:pt-0 [&>article:last-child]:border-b-0">
                            { for posts.iter().map(|article| html! {
                                <ArticleCard
                                    key={article.id.clone()}
                                    article={article.clone()}
                                    variant={ArticleCardVariant::Row}
                                />
                            }) }
                        </div>
                    </section>
                }
            }) }
        </div>
    }
}

/// Loading placeholder that mirrors the editorial row layout.
pub(crate) fn render_archive_skeleton() -> Html {
    html! {
        <div class="divide-y divide-[var(--border)]" aria-hidden="true">
            { for (0..5).map(|idx| html! {
                <div key={idx} class="grid grid-cols-1 md:grid-cols-[130px_1fr_180px] gap-4 md:gap-6 py-7 items-start">
                    <div class="space-y-2">
                        <span class="skeleton block h-3.5 w-24 rounded" />
                        <span class="skeleton block h-3 w-14 rounded" />
                    </div>
                    <div class="space-y-2.5">
                        <span class="skeleton block h-6 w-3/4 rounded" />
                        <span class="skeleton block h-4 w-full rounded" />
                        <span class="skeleton block h-4 w-2/3 rounded" />
                    </div>
                    <span class="hidden md:block skeleton aspect-[4/3] w-full rounded-[8px]" />
                </div>
            }) }
        </div>
    }
}

#[allow(
    dead_code,
    reason = "These render helpers are exercised by alternate timeline layouts and tests, even \
              when a specific build path does not call them."
)]
pub(crate) fn render_timeline(grouped_by_year: &[(i32, Vec<ArticleListItem>)]) -> Html {
    render_timeline_with_state(grouped_by_year, None, None)
}

#[allow(
    dead_code,
    reason = "These render helpers are exercised by alternate timeline layouts and tests, even \
              when a specific build path does not call them."
)]
pub(crate) fn render_expandable_timeline(
    grouped_by_year: &[(i32, Vec<ArticleListItem>)],
    expanded_years: &HashMap<i32, bool>,
    toggle_year: &Callback<i32>,
) -> Html {
    render_timeline_with_state(grouped_by_year, Some(expanded_years), Some(toggle_year))
}

#[allow(
    dead_code,
    reason = "The shared rendering implementation is retained for alternate timeline layouts."
)]
fn render_timeline_with_state(
    grouped_by_year: &[(i32, Vec<ArticleListItem>)],
    expanded_years: Option<&HashMap<i32, bool>>,
    toggle_year: Option<&Callback<i32>>,
) -> Html {
    html! {
        <>
            { for grouped_by_year.iter().map(|(year, posts)| {
                let year_value = *year;
                let total_count = posts.len();
                let collapse_enabled = expanded_years.is_some() && toggle_year.is_some();
                let should_collapse = collapse_enabled && total_count > 20;
                let is_expanded = if collapse_enabled {
                    expanded_years
                        .and_then(|map| map.get(&year_value).copied())
                        .unwrap_or(false)
                } else {
                    true
                };
                let visible_count = if should_collapse && !is_expanded {
                    total_count.min(10)
                } else {
                    total_count
                };
                let remaining = total_count.saturating_sub(visible_count);
                html! {
                    <>
                        <h3 class={classes!(
                            "mt-10",
                            "mb-5",
                            "font-serif",
                            "font-normal",
                            "text-2xl",
                            "text-[var(--text)]"
                        )}>{ year_value }</h3>
                        <div class={classes!("timeline")}>
                            { for posts.iter().take(visible_count).map(|article| {
                                let detail_route = Route::ArticleDetail { id: article.id.clone() };
                                html! {
                                    <div class={classes!("circle")}>
                                        <div class={classes!(
                                            "m-0",
                                            "leading-relaxed",
                                            "pl-[calc(var(--timeline-offset)+1rem)]"
                                        )}>
                                            <Link<Route> to={detail_route} classes={classes!(
                                                "font-serif",
                                                "font-normal",
                                                "text-[1.1rem]",
                                                "text-[var(--text)]",
                                                "transition-colors",
                                                "duration-150",
                                                "hover:text-[var(--primary)]"
                                            )}>
                                                { article.title.clone() }
                                            </Link<Route>>
                                        </div>
                                        <div class={classes!(
                                            "m-0",
                                            "leading-relaxed",
                                            "pl-[calc(var(--timeline-offset)+1rem)]"
                                        )}>
                                            <span class={classes!(
                                                "inline-block",
                                                "mt-1",
                                                "font-mono",
                                                "text-xs",
                                                "tabular-nums",
                                                "text-[var(--faint)]"
                                            )}>
                                                { fill_one(t::PUBLISHED_ON_TEMPLATE, format_month_day(&article.date)) }
                                            </span>
                                        </div>
                                    </div>
                                }
                            }) }
                        </div>
                        {
                            if should_collapse {
                                let button_label = if is_expanded {
                                    t::COLLAPSE.to_string()
                                } else {
                                    fill_one(t::EXPAND_REMAINING_TEMPLATE, remaining)
                                };
                                if let Some(toggle_cb) = toggle_year {
                                    let toggle_cb = toggle_cb.clone();
                                    let year_for_toggle = year_value;
                                    let onclick = Callback::from(move |_| toggle_cb.emit(year_for_toggle));
                                    html! {
                                        <button
                                            type="button"
                                            class={classes!("sf-btn", "sf-btn--ghost", "sf-btn--sm", "mt-3")}
                                            {onclick}
                                            aria-expanded={is_expanded.to_string()}
                                            aria-label={fill_one(t::YEAR_TOGGLE_ARIA_TEMPLATE, year_value)}
                                        >
                                            { button_label }
                                        </button>
                                    }
                                } else {
                                    Html::default()
                                }
                            } else {
                                Html::default()
                            }
                        }
                    </>
                }
            }) }
        </>
    }
}

pub(crate) fn group_articles_by_year(
    articles: &[ArticleListItem],
) -> Vec<(i32, Vec<ArticleListItem>)> {
    let mut map: BTreeMap<i32, Vec<ArticleListItem>> = BTreeMap::new();
    for article in articles {
        if let Some(year) = extract_year(&article.date) {
            map.entry(year).or_default().push(article.clone());
        }
    }

    for posts in map.values_mut() {
        posts.sort_by(|a, b| b.date.cmp(&a.date));
    }

    map.into_iter().rev().collect()
}

fn extract_year(date: &str) -> Option<i32> {
    date.split('-').next()?.parse().ok()
}

pub(crate) fn format_month_day(date: &str) -> String {
    let mut parts = date.split('-');
    let _ = parts.next();
    match (parts.next(), parts.next()) {
        (Some(month), Some(day)) => format!("{month}-{day}"),
        _ => date.to_string(),
    }
}
