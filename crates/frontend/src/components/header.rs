use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{
        icons::IconName,
        search_suggest::SearchSuggest,
        theme_toggle::ThemeToggle,
        tooltip::{TooltipIconButton, TooltipPosition},
    },
    i18n::current::header as t,
    router::Route,
};

fn is_nav_active(current: Option<&Route>, target: &Route) -> bool {
    let Some(current) = current else {
        return false;
    };
    match target {
        Route::LatestArticles => matches!(current, Route::LatestArticles),
        Route::Posts => matches!(
            current,
            Route::Posts
                | Route::ArticleDetail { .. }
                | Route::ArticleInteractive { .. }
                | Route::ArticleRaw { .. }
        ),
        Route::Tags => matches!(current, Route::Tags | Route::TagDetail { .. }),
        Route::Categories => {
            matches!(current, Route::Categories | Route::CategoryDetail { .. })
        },
        Route::MediaAudio => matches!(current, Route::MediaAudio | Route::MusicPlayer { .. }),
        Route::MediaImage => matches!(current, Route::MediaImage | Route::MediaVideo),
        Route::LlmAccess => matches!(
            current,
            Route::LlmAccess | Route::LlmAccessGuide | Route::LlmAccessUsage | Route::KiroAccess
        ),
        _ => current == target,
    }
}

#[function_component(Header)]
pub fn header() -> Html {
    let mobile_menu_open = use_state(|| false);
    let location = use_location();
    let route = use_route::<Route>();
    let location_sync_key = location
        .as_ref()
        .map(|loc| format!("{}{}", loc.path(), loc.query_str()))
        .unwrap_or_default();
    let initial_query = location
        .as_ref()
        .and_then(|loc| loc.query::<HeaderSearchQuery>().ok())
        .and_then(|query| query.q)
        .unwrap_or_default();
    let search_query = use_state(|| initial_query);

    {
        let search_query = search_query.clone();
        let location = location.clone();
        use_effect_with(location_sync_key, move |_| {
            let next = location
                .as_ref()
                .and_then(|item| item.query::<HeaderSearchQuery>().ok())
                .and_then(|query| query.q)
                .unwrap_or_default();
            if *search_query != next {
                search_query.set(next);
            }
            || ()
        });
    }

    let toggle_mobile_menu = {
        let mobile_menu_open = mobile_menu_open.clone();
        Callback::from(move |_| mobile_menu_open.set(!*mobile_menu_open))
    };

    let close_mobile_menu = {
        let mobile_menu_open = mobile_menu_open.clone();
        Callback::from(move |_| mobile_menu_open.set(false))
    };

    let clear_search = {
        let search_query = search_query.clone();
        Callback::from(move |_| search_query.set(String::new()))
    };

    let do_search = {
        let search_query = search_query.clone();
        let route = route.clone();
        let location = location.clone();
        Callback::from(move |_: MouseEvent| {
            let query = (*search_query).trim();
            if !query.is_empty() {
                let encoded_query = urlencoding::encode(query);
                let search_url = build_search_url(route.clone(), location.clone(), &encoded_query);
                let _ = crate::navigation_context::navigate_spa_to(&search_url);
            }
        })
    };

    let nav_items = [
        (t::NAV_LATEST, Route::LatestArticles, "fa-bolt"),
        (t::NAV_POSTS, Route::Posts, "fa-book-open"),
        (t::NAV_CATEGORIES, Route::Categories, "fa-folder-open"),
        (t::NAV_TAGS, Route::Tags, "fa-hashtag"),
        (t::NAV_MUSIC, Route::MediaAudio, "fa-headphones"),
        (t::IMAGE_LIBRARY_TITLE, Route::MediaImage, "fa-image"),
        (t::NAV_LLM, Route::LlmAccess, "fa-key"),
    ];

    let mobile_search_submit = {
        let route = route.clone();
        let location = location.clone();
        let mobile_menu_open = mobile_menu_open.clone();
        Callback::from(move |query: String| {
            let query = query.trim();
            if !query.is_empty() {
                let encoded_query = urlencoding::encode(query);
                let search_url = build_search_url(route.clone(), location.clone(), &encoded_query);
                let _ = crate::navigation_context::navigate_spa_to(&search_url);
                mobile_menu_open.set(false);
            }
        })
    };
    let mobile_clear_search = clear_search.clone();

    let mobile_menu_classes = classes!(
        "fixed",
        "inset-0",
        "z-[120]",
        "transition-opacity",
        "duration-200",
        if *mobile_menu_open {
            "opacity-100 pointer-events-auto"
        } else {
            "opacity-0 pointer-events-none"
        }
    );

    let mobile_panel_classes = classes!(
        "absolute",
        "top-0",
        "right-0",
        "bottom-0",
        "w-full",
        "max-w-sm",
        "bg-[var(--surface)]",
        "border-l",
        "border-[var(--border)]",
        "text-[var(--text)]",
        "px-5",
        "pb-8",
        "flex",
        "flex-col",
        "gap-4",
        "overflow-y-auto",
        "shadow-[var(--shadow-16)]",
        "transition-transform",
        "duration-250",
        "ease-[var(--ease-spring)]",
        if *mobile_menu_open { "translate-x-0" } else { "translate-x-full" }
    );

    let hamburger_line = classes!(
        "block",
        "w-4",
        "h-[1.75px]",
        "rounded-full",
        "bg-[var(--text)]",
        "transition-all",
        "duration-200"
    );

    html! {
        <>
            <header class={classes!(
                "header-minimal",
                "sticky", "top-0", "left-0", "right-0", "z-[80]", "w-full"
            )}>
                // Desktop header
                <div class={classes!(
                    "desktop-header",
                    "items-center", "gap-3",
                    "min-h-[var(--header-height-mobile)]", "md:min-h-[var(--header-height-desktop)]",
                    "max-w-7xl", "mx-auto", "px-4", "sm:px-6", "lg:px-8"
                )}>
                    <Link<Route> to={Route::Home} classes="brand-logo shrink-0">
                        <span class="brand-logo-dot" aria-hidden="true"></span>
                        <span>{"StaticFlow"}</span>
                        <span class="brand-logo-sub hidden xl:inline">{t::BRAND_NAME}</span>
                    </Link<Route>>

                    <nav class={classes!("flex", "items-center", "gap-0.5", "ml-2")} aria-label={t::NAV_MAIN_ARIA}>
                        { for nav_items.iter().map(|(label, target_route, _icon)| {
                            let active = is_nav_active(route.as_ref(), target_route);
                            html! {
                                <Link<Route>
                                    to={target_route.clone()}
                                    classes={classes!(
                                        "sf-nav-pill",
                                        active.then_some("sf-nav-pill--active")
                                    )}
                                >
                                    { *label }
                                </Link<Route>>
                            }
                        }) }
                    </nav>

                    <div class={classes!("ml-auto", "flex", "items-center", "gap-2")}>
                        // Fixed-width search capsule (never reflows nav on focus)
                        <div class={classes!(
                            "flex", "items-center",
                            "h-9", "pl-3", "pr-1.5",
                            "rounded-full",
                            "border", "border-[var(--border)]",
                            "bg-[var(--surface)]",
                            "transition-colors", "duration-150",
                            "focus-within:border-[var(--primary)]"
                        )}>
                            <button
                                type="button"
                                onclick={do_search.clone()}
                                class={classes!(
                                    "w-6", "h-6",
                                    "inline-flex", "items-center", "justify-center",
                                    "text-[var(--muted)]", "hover:text-[var(--primary)]",
                                    "transition-colors", "shrink-0"
                                )}
                                aria-label={t::SEARCH_ARIA}
                            >
                                <i class="fas fa-search text-xs" aria-hidden="true"></i>
                            </button>
                            <SearchSuggest
                                value={(*search_query).clone()}
                                on_change={{
                                    let search_query = search_query.clone();
                                    Callback::from(move |value: String| search_query.set(value))
                                }}
                                on_submit={{
                                    let route = route.clone();
                                    let location = location.clone();
                                    Callback::from(move |query: String| {
                                        let encoded_query = urlencoding::encode(&query);
                                        let search_url = build_search_url(
                                            route.clone(),
                                            location.clone(),
                                            &encoded_query,
                                        );
                                        let _ = crate::navigation_context::navigate_spa_to(&search_url);
                                    })
                                }}
                                input_class={classes!(
                                    "w-[140px]", "lg:w-[175px]", "xl:w-[205px]",
                                    "px-2", "h-8",
                                    "bg-transparent", "text-[var(--text)]",
                                    "text-sm", "border-0",
                                    "focus:outline-none"
                                )}
                            />
                            <button
                                type="button"
                                onclick={clear_search.clone()}
                                disabled={search_query.is_empty()}
                                class={classes!(
                                    "w-6", "h-6", "rounded-full",
                                    "inline-flex", "items-center", "justify-center",
                                    "text-[var(--muted)]", "hover:text-[var(--text)]",
                                    "hover:bg-[var(--surface-alt)]",
                                    "transition-opacity",
                                    if search_query.is_empty() {
                                        "opacity-0 pointer-events-none"
                                    } else {
                                        "opacity-100"
                                    }
                                )}
                                aria-label={t::CLEAR_ARIA}
                            >
                                <i class="fas fa-times text-[11px]" aria-hidden="true"></i>
                            </button>
                        </div>

                        <ThemeToggle />
                    </div>
                </div>

                // Mobile header
                <div class={classes!(
                    "mobile-header",
                    "items-center",
                    "justify-between",
                    "gap-3",
                    "min-h-[var(--header-height-mobile)]",
                    "max-w-7xl",
                    "mx-auto",
                    "px-4",
                    "sm:px-6"
                )}>
                    <Link<Route> to={Route::Home} classes="brand-logo">
                        <span class="brand-logo-dot" aria-hidden="true"></span>
                        <span>{"StaticFlow"}</span>
                        <span class="brand-logo-sub">{t::BRAND_NAME}</span>
                    </Link<Route>>

                    <div class="flex items-center gap-2">
                        <ThemeToggle />
                        <button
                            type="button"
                            class={classes!(
                                "w-10", "h-10",
                                "border", "border-[var(--border)]",
                                "rounded-lg",
                                "bg-[var(--surface)]",
                                "text-[var(--text)]",
                                "flex", "flex-col", "justify-center", "items-center",
                                "gap-[4px]",
                                "cursor-pointer",
                                "hover:bg-[var(--surface-alt)]"
                            )}
                            aria-label={t::OPEN_MENU_ARIA}
                            aria-expanded={(*mobile_menu_open).to_string()}
                            onclick={toggle_mobile_menu.clone()}
                        >
                            <span
                                class={classes!(
                                    hamburger_line.clone(),
                                    if *mobile_menu_open { "translate-y-[5.75px] rotate-45" } else { "" }
                                )}
                            />
                            <span
                                class={classes!(
                                    hamburger_line.clone(),
                                    if *mobile_menu_open { "opacity-0" } else { "opacity-100" }
                                )}
                            />
                            <span
                                class={classes!(
                                    hamburger_line,
                                    if *mobile_menu_open { "-translate-y-[5.75px] -rotate-45" } else { "" }
                                )}
                            />
                        </button>
                    </div>
                </div>
            </header>

            // Mobile slide-over drawer
            <div class={mobile_menu_classes}>
                <div
                    class={classes!(
                        "absolute",
                        "inset-0",
                        "bg-black/40",
                        "backdrop-blur-[4px]",
                        "transition-opacity",
                        "duration-200",
                        if *mobile_menu_open { "opacity-100" } else { "opacity-0" }
                    )}
                    onclick={close_mobile_menu.clone()}
                />

                <div
                    class={mobile_panel_classes}
                    role="dialog"
                    aria-modal="true"
                >
                    <div class={classes!(
                        "sticky", "top-0", "z-10",
                        "flex", "items-center", "justify-between",
                        "pt-4", "pb-3",
                        "bg-[var(--surface)]",
                        "border-b", "border-[var(--border)]"
                    )}>
                        <span class="text-sm font-semibold text-[var(--muted)]">{t::MOBILE_NAV_ARIA}</span>
                        <TooltipIconButton
                            icon={IconName::ArrowLeft}
                            tooltip={t::CLOSE_TOOLTIP}
                            position={TooltipPosition::Bottom}
                            onclick={close_mobile_menu.clone()}
                            size={18}
                            class={classes!(
                                "!bg-[var(--surface-alt)]",
                                "!border",
                                "!border-[var(--border)]",
                                "!rounded-lg"
                            )}
                        />
                    </div>

                    <div class={classes!("mobile-search-suggest", "flex", "gap-2", "items-center")}>
                        <SearchSuggest
                            value={(*search_query).clone()}
                            on_change={{
                                let search_query = search_query.clone();
                                Callback::from(move |value: String| search_query.set(value))
                            }}
                            on_submit={mobile_search_submit}
                            input_class={classes!(
                                "w-full",
                                "border", "border-[var(--border)]", "rounded-lg",
                                "px-3.5", "h-10", "text-sm",
                                "bg-[var(--surface-alt)]",
                                "text-[var(--text)]",
                                "focus:outline-none",
                                "focus:bg-[var(--surface)]",
                                "focus:border-[var(--primary)]"
                            )}
                        />
                        if !search_query.is_empty() {
                            <button
                                type="button"
                                onclick={mobile_clear_search}
                                aria-label={t::CLEAR_ARIA}
                                class={classes!(
                                    "w-10", "h-10", "shrink-0",
                                    "rounded-lg",
                                    "border", "border-[var(--border)]",
                                    "bg-[var(--surface-alt)]",
                                    "text-[var(--muted)]",
                                    "hover:text-[var(--text)]"
                                )}
                            >
                                <i class="fas fa-times text-xs" aria-hidden="true"></i>
                            </button>
                        }
                    </div>

                    <nav class={classes!("flex", "flex-col", "gap-1.5")} aria-label={t::MOBILE_NAV_ARIA}>
                        { for nav_items.iter().map(|(label, target_route, icon)| {
                            let active = is_nav_active(route.as_ref(), target_route);
                            let close_cb = close_mobile_menu.clone();
                            html! {
                                <div onclick={close_cb}>
                                    <Link<Route> to={target_route.clone()} classes={classes!(
                                        "flex", "items-center", "gap-3",
                                        "py-2.5", "px-3.5", "rounded-lg",
                                        "text-sm", "font-medium", "no-underline",
                                        "transition-colors",
                                        if active {
                                            "bg-[rgba(var(--primary-rgb),0.1)] text-[var(--primary)] font-semibold"
                                        } else {
                                            "text-[var(--text)] hover:bg-[var(--surface-alt)]"
                                        }
                                    )}>
                                        <i class={classes!(
                                            "fas", *icon, "w-4", "text-center",
                                            if active { "text-[var(--primary)]" } else { "text-[var(--muted)]" }
                                        )} aria-hidden="true"></i>
                                        <span>{ *label }</span>
                                    </Link<Route>>
                                </div>
                            }
                        }) }
                    </nav>
                </div>
            </div>
        </>
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct HeaderSearchQuery {
    q: Option<String>,
    mode: Option<String>,
    enhanced_highlight: Option<bool>,
    hybrid: Option<bool>,
    hybrid_rrf_k: Option<f32>,
    hybrid_vector_limit: Option<usize>,
    hybrid_fts_limit: Option<usize>,
    limit: Option<usize>,
    all: Option<bool>,
    max_distance: Option<f32>,
}

fn build_search_url(
    route: Option<Route>,
    location: Option<Location>,
    encoded_query: &str,
) -> String {
    let mut params = vec![format!("q={encoded_query}")];

    let is_music_context =
        matches!(route, Some(Route::MediaAudio) | Some(Route::MusicPlayer { .. }));
    let is_image_context = matches!(route, Some(Route::MediaImage));

    if matches!(route, Some(Route::Search)) {
        if let Some(current) = location.and_then(|loc| loc.query::<HeaderSearchQuery>().ok()) {
            if let Some(mode) = current.mode.filter(|value| {
                matches!(value.as_str(), "keyword" | "semantic" | "image" | "music")
            }) {
                if mode != "keyword" {
                    params.push(format!("mode={}", urlencoding::encode(&mode)));
                }
            }
            if current.enhanced_highlight.unwrap_or(false) {
                params.push("enhanced_highlight=true".to_string());
            }
            if let Some(limit) = current.limit.filter(|value| *value > 0) {
                params.push(format!("limit={limit}"));
            }
            if current.all.unwrap_or(false) {
                params.push("all=true".to_string());
            }
            if let Some(max_distance) = current
                .max_distance
                .filter(|value| value.is_finite() && *value >= 0.0)
            {
                params.push(format!("max_distance={max_distance}"));
            }
            if current.hybrid.unwrap_or(false) {
                params.push("hybrid=true".to_string());
                if let Some(rrf_k) = current
                    .hybrid_rrf_k
                    .filter(|value| value.is_finite() && *value > 0.0)
                {
                    params.push(format!("hybrid_rrf_k={rrf_k}"));
                }
                if let Some(vector_limit) = current.hybrid_vector_limit.filter(|value| *value > 0) {
                    params.push(format!("hybrid_vector_limit={vector_limit}"));
                }
                if let Some(fts_limit) = current.hybrid_fts_limit.filter(|value| *value > 0) {
                    params.push(format!("hybrid_fts_limit={fts_limit}"));
                }
            }
        }
    } else if is_music_context {
        params.push("mode=music".to_string());
    } else if is_image_context {
        params.push("mode=image".to_string());
    }
    crate::config::route_path(&format!("/search?{}", params.join("&")))
}
