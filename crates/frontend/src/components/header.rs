use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use gloo_utils::{document, window};
use serde::Deserialize;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlInputElement, KeyboardEvent, Node};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{search_suggest::SearchSuggest, theme_toggle::ThemeToggle},
    i18n::current::header as t,
    router::Route,
};

const NAV_GALLERY: &str = "图库";
const NAV_STUDIO: &str = "共建台";
const CTA_STUDIO_ARTICLE: &str = "投一篇文章";
const CTA_STUDIO_MUSIC: &str = "点一首歌";

fn is_nav_active(current: Option<&Route>, target: &Route) -> bool {
    let Some(current) = current else {
        return false;
    };
    match target {
        Route::Posts => matches!(
            current,
            Route::Posts
                | Route::Categories
                | Route::CategoryDetail { .. }
                | Route::ArticleDetail { .. }
                | Route::ArticleInteractive { .. }
                | Route::ArticleRaw { .. }
        ),
        Route::LatestArticles => matches!(current, Route::LatestArticles),
        Route::Tags => matches!(current, Route::Tags | Route::TagDetail { .. }),
        Route::Categories => {
            matches!(current, Route::Categories | Route::CategoryDetail { .. })
        },
        Route::MediaAudio => matches!(current, Route::MediaAudio | Route::MusicPlayer { .. }),
        Route::MediaImage => matches!(current, Route::MediaImage | Route::MediaVideo),
        _ => current == target,
    }
}

fn is_editable_target(target: Option<web_sys::EventTarget>) -> bool {
    let Some(el) = target.and_then(|t| t.dyn_into::<Element>().ok()) else {
        return false;
    };
    let tag = el.tag_name().to_ascii_uppercase();
    if matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT") {
        return true;
    }
    el.dyn_into::<HtmlElement>()
        .map(|html_el| html_el.is_content_editable())
        .unwrap_or(false)
}

fn focus_search_input_in(container: &NodeRef) {
    let container = container.clone();
    Timeout::new(16, move || {
        if let Some(root) = container.cast::<Element>() {
            if let Ok(Some(input)) = root.query_selector("input") {
                if let Ok(input_el) = input.dyn_into::<HtmlInputElement>() {
                    let _ = input_el.focus();
                    input_el.select();
                }
            }
        }
    })
    .forget();
}

fn navigate_to_studio(target_subpath: &'static str) {
    let target_url = crate::config::route_path(target_subpath);
    if !crate::navigation_context::navigate_spa_to(&target_url) {
        let _ = window().location().set_href(&target_url);
        return;
    }
    let win = window();
    if let Ok(ev) = web_sys::Event::new("hashchange") {
        let _ = win.dispatch_event(&ev);
    }
    if !target_subpath.contains('#') {
        win.scroll_to_with_x_and_y(0.0, 0.0);
    }
}

#[function_component(Header)]
pub fn header() -> Html {
    let mobile_menu_open = use_state(|| false);
    let scrolled = use_state(|| window().scroll_y().unwrap_or(0.0) > 8.0);
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
    let search_expanded = use_state(|| !initial_query.is_empty());
    let search_query = use_state(|| initial_query);
    let search_wrap_ref = use_node_ref();

    // Sync query text from URL
    {
        let search_query = search_query.clone();
        let search_expanded = search_expanded.clone();
        let location = location.clone();
        use_effect_with(location_sync_key.clone(), move |_| {
            let next = location
                .as_ref()
                .and_then(|item| item.query::<HeaderSearchQuery>().ok())
                .and_then(|query| query.q)
                .unwrap_or_default();
            if !next.is_empty() {
                search_expanded.set(true);
            }
            if *search_query != next {
                search_query.set(next);
            }
            || ()
        });
    }

    // Toggle 1px bottom hairline only after scrolling > 8px
    {
        let scrolled = scrolled.clone();
        use_effect_with((), move |_| {
            let win = window();
            let sync = {
                let scrolled = scrolled.clone();
                let win = win.clone();
                move || {
                    let next = win.scroll_y().unwrap_or(0.0) > 8.0;
                    scrolled.set(next);
                }
            };
            sync();
            let listener = EventListener::new(&win, "scroll", move |_| sync());
            move || drop(listener)
        });
    }

    // Lock page scroll (both <html> and <body>) while the drawer is open
    {
        use_effect_with(*mobile_menu_open, move |open| {
            let open = *open;
            let doc = document();
            let targets: Vec<HtmlElement> = [
                doc.document_element()
                    .and_then(|el| el.dyn_into::<HtmlElement>().ok()),
                doc.body(),
            ]
            .into_iter()
            .flatten()
            .collect();
            let previous: Vec<String> = targets
                .iter()
                .map(|el| {
                    el.style()
                        .get_property_value("overflow")
                        .unwrap_or_default()
                })
                .collect();
            if open {
                for el in &targets {
                    let _ = el.style().set_property("overflow", "hidden");
                }
            }
            move || {
                if open {
                    for (el, prev) in targets.iter().zip(previous.iter()) {
                        if prev.is_empty() {
                            let _ = el.style().remove_property("overflow");
                        } else {
                            let _ = el.style().set_property("overflow", prev);
                        }
                    }
                }
            }
        });
    }

    // Close the drawer whenever the route changes (back/forward included)
    {
        let mobile_menu_open = mobile_menu_open.clone();
        use_effect_with(location_sync_key.clone(), move |_| {
            mobile_menu_open.set(false);
            || ()
        });
    }

    // `/` shortcut to expand/focus compact search, `Escape` to dismiss
    {
        let search_expanded = search_expanded.clone();
        let search_query = search_query.clone();
        let mobile_menu_open = mobile_menu_open.clone();
        let search_wrap_ref = search_wrap_ref.clone();
        use_effect_with(
            (*mobile_menu_open, *search_expanded, (*search_query).clone()),
            move |(menu_open, expanded, query)| {
                let menu_open = *menu_open;
                let expanded = *expanded;
                let query_empty = query.trim().is_empty();
                let doc = document();
                let listener = EventListener::new(&doc, "keydown", move |event| {
                    let Ok(ke) = event.clone().dyn_into::<KeyboardEvent>() else {
                        return;
                    };
                    let key = ke.key();
                    if key == "/"
                        && !ke.ctrl_key()
                        && !ke.meta_key()
                        && !ke.alt_key()
                        && !is_editable_target(ke.target())
                    {
                        ke.prevent_default();
                        let width = window()
                            .inner_width()
                            .ok()
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1024.0);
                        if width >= 768.0 {
                            search_expanded.set(true);
                            focus_search_input_in(&search_wrap_ref);
                        } else {
                            let _ = crate::navigation_context::navigate_spa_to(
                                &crate::config::route_path("/search"),
                            );
                        }
                    } else if key == "Escape" {
                        if menu_open {
                            mobile_menu_open.set(false);
                        } else if expanded && query_empty {
                            search_expanded.set(false);
                        }
                    }
                });
                move || drop(listener)
            },
        );
    }

    // Outside click collapses empty desktop search bar
    {
        let search_expanded = search_expanded.clone();
        let search_query = search_query.clone();
        let search_wrap_ref = search_wrap_ref.clone();
        use_effect_with((*search_expanded, (*search_query).clone()), move |(expanded, query)| {
            let listener = if *expanded && query.trim().is_empty() {
                let doc = document();
                let search_expanded = search_expanded.clone();
                let search_wrap_ref = search_wrap_ref.clone();
                Some(EventListener::new(&doc, "mousedown", move |event| {
                    let Some(root) = search_wrap_ref.cast::<Node>() else {
                        return;
                    };
                    let inside = event
                        .target()
                        .and_then(|t| t.dyn_into::<Node>().ok())
                        .is_some_and(|node| root.contains(Some(&node)));
                    if !inside {
                        search_expanded.set(false);
                    }
                }))
            } else {
                None
            };
            move || drop(listener)
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

    let open_desktop_search = {
        let search_expanded = search_expanded.clone();
        let search_wrap_ref = search_wrap_ref.clone();
        Callback::from(move |_: MouseEvent| {
            search_expanded.set(true);
            focus_search_input_in(&search_wrap_ref);
        })
    };

    let clear_or_collapse_search = {
        let search_query = search_query.clone();
        let search_expanded = search_expanded.clone();
        let search_wrap_ref = search_wrap_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if search_query.trim().is_empty() {
                search_expanded.set(false);
            } else {
                search_query.set(String::new());
                focus_search_input_in(&search_wrap_ref);
            }
        })
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
            } else {
                let _ = crate::navigation_context::navigate_spa_to(&crate::config::route_path(
                    "/search",
                ));
            }
        })
    };

    let on_studio_route = location.as_ref().is_some_and(|loc| {
        let path = loc.path();
        path == "/studio" || path.ends_with("/studio")
    });

    let go_studio = Callback::from(|e: MouseEvent| {
        if !e.meta_key() && !e.ctrl_key() && !e.shift_key() && !e.alt_key() && e.button() == 0 {
            e.prevent_default();
            navigate_to_studio("/studio");
        }
    });

    let go_studio_article_mobile = {
        let mobile_menu_open = mobile_menu_open.clone();
        Callback::from(move |e: MouseEvent| {
            if !e.meta_key() && !e.ctrl_key() && !e.shift_key() && !e.alt_key() && e.button() == 0 {
                e.prevent_default();
                mobile_menu_open.set(false);
                navigate_to_studio("/studio#article");
            }
        })
    };

    let go_studio_music_mobile = {
        let mobile_menu_open = mobile_menu_open.clone();
        Callback::from(move |e: MouseEvent| {
            if !e.meta_key() && !e.ctrl_key() && !e.shift_key() && !e.alt_key() && e.button() == 0 {
                e.prevent_default();
                mobile_menu_open.set(false);
                navigate_to_studio("/studio#music");
            }
        })
    };

    // Desktop center: strictly 5 content links (文章 · 最新 · 音乐 · 图库 · 标签)
    let desktop_nav_items = [
        (t::NAV_POSTS, Route::Posts),
        (t::NAV_LATEST, Route::LatestArticles),
        (t::NAV_MUSIC, Route::MediaAudio),
        (NAV_GALLERY, Route::MediaImage),
        (t::NAV_TAGS, Route::Tags),
    ];

    let mobile_nav_items = [
        (t::NAV_POSTS, Route::Posts),
        (t::NAV_LATEST, Route::LatestArticles),
        (t::NAV_MUSIC, Route::MediaAudio),
        (NAV_GALLERY, Route::MediaImage),
        (t::NAV_TAGS, Route::Tags),
        (t::NAV_CATEGORIES, Route::Categories),
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

    let mobile_clear_search = {
        let search_query = search_query.clone();
        Callback::from(move |_: MouseEvent| search_query.set(String::new()))
    };

    // The /search page owns a large in-page query field; mirroring `?q=` into
    // an expanded header box (plus its suggestion popover) would duplicate it
    // and occlude the result tabs, so the header stays a quiet icon there.
    let on_search_route = matches!(route.as_ref(), Some(Route::Search));
    let show_desktop_search_box =
        !on_search_route && (*search_expanded || !search_query.is_empty());

    let hamburger_line =
        classes!("block", "w-[18px]", "h-[1.5px]", "rounded-full", "bg-[var(--text)]");

    let mobile_icon_button = classes!(
        "w-10",
        "h-10",
        "shrink-0",
        "rounded-full",
        "bg-transparent",
        "text-[var(--muted)]",
        "inline-flex",
        "items-center",
        "justify-center",
        "cursor-pointer",
        "hover:text-[var(--text)]",
        "hover:bg-[var(--surface-alt)]",
        "transition-colors"
    );

    let open_mobile_search_page = {
        let search_query = search_query.clone();
        let route = route.clone();
        let location = location.clone();
        Callback::from(move |_: MouseEvent| {
            let query = (*search_query).trim();
            let url = if query.is_empty() {
                crate::config::route_path("/search")
            } else {
                build_search_url(route.clone(), location.clone(), &urlencoding::encode(query))
            };
            let _ = crate::navigation_context::navigate_spa_to(&url);
        })
    };

    html! {
        <>
            <header class={classes!(
                "header-minimal",
                "sticky", "top-0", "left-0", "right-0", "z-[80]", "w-full",
                (*scrolled).then_some("is-scrolled")
            )}>
                // Desktop header (>= md)
                <div class={classes!(
                    "desktop-header",
                    "items-center", "justify-between", "gap-4", "lg:gap-6",
                    "h-16",
                    "max-w-[1200px]", "mx-auto", "px-6", "md:px-10"
                )}>
                    <Link<Route> to={Route::Home} classes="brand-logo shrink-0">
                        <span class="brand-logo-diamond" aria-hidden="true"></span>
                        <span>{"StaticFlow"}</span>
                    </Link<Route>>

                    <nav
                        class={classes!("flex", "items-center", "gap-5", "lg:gap-7")}
                        aria-label={t::NAV_MAIN_ARIA}
                    >
                        { for desktop_nav_items.iter().map(|(label, target_route)| {
                            let active = is_nav_active(route.as_ref(), target_route);
                            html! {
                                <Link<Route>
                                    to={target_route.clone()}
                                    classes={classes!(
                                        "sf-nav-link",
                                        active.then_some("sf-nav-link--active")
                                    )}
                                >
                                    { *label }
                                </Link<Route>>
                            }
                        }) }
                    </nav>

                    <div class={classes!("flex", "items-center", "gap-2", "shrink-0")}>
                        <div ref={search_wrap_ref} class="relative flex items-center">
                            if show_desktop_search_box {
                                <div class={classes!(
                                    "flex", "items-center", "gap-1",
                                    "w-[180px]", "lg:w-[240px]", "h-9", "pl-3", "pr-1.5",
                                    "rounded-full",
                                    "border", "border-[var(--border-strong)]",
                                    "bg-[var(--surface)]",
                                    "transition-colors", "duration-150",
                                    "focus-within:border-[var(--primary)]"
                                )}>
                                    <button
                                        type="button"
                                        onclick={do_search}
                                        class={classes!(
                                            "w-6", "h-6", "shrink-0",
                                            "inline-flex", "items-center", "justify-center",
                                            "text-[var(--muted)]", "hover:text-[var(--primary)]",
                                            "transition-colors", "cursor-pointer"
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
                                            "w-full", "min-w-0", "px-1.5", "h-8",
                                            "bg-transparent", "text-[var(--text)]",
                                            "text-[0.875rem]", "border-0",
                                            "focus:outline-none"
                                        )}
                                    />
                                    <button
                                        type="button"
                                        onclick={clear_or_collapse_search}
                                        class={classes!(
                                            "w-6", "h-6", "shrink-0", "rounded-full",
                                            "inline-flex", "items-center", "justify-center",
                                            "text-[var(--muted)]", "hover:text-[var(--text)]",
                                            "hover:bg-[var(--surface-alt)]",
                                            "transition-colors", "cursor-pointer"
                                        )}
                                        aria-label={t::CLEAR_ARIA}
                                    >
                                        <i class="fas fa-times text-[11px]" aria-hidden="true"></i>
                                    </button>
                                </div>
                            } else {
                                <button
                                    type="button"
                                    onclick={open_desktop_search}
                                    class={classes!(
                                        "w-9", "h-9", "rounded-full",
                                        "inline-flex", "items-center", "justify-center",
                                        "text-[var(--muted)]", "hover:text-[var(--text)]",
                                        "hover:bg-[var(--surface-alt)]",
                                        "transition-colors", "duration-150", "cursor-pointer"
                                    )}
                                    aria-label={t::SEARCH_ARIA}
                                    title="搜索 ( / )"
                                >
                                    <i class="fas fa-search text-[13px]" aria-hidden="true"></i>
                                </button>
                            }
                        </div>

                        <ThemeToggle />

                        <a
                            href={crate::config::route_path("/studio")}
                            onclick={go_studio}
                            aria-current={on_studio_route.then_some("page")}
                            class={classes!(
                                "sf-btn",
                                "sf-btn--secondary",
                                "sf-btn--sm",
                                "ml-1",
                                "no-underline",
                                on_studio_route.then_some("!border-[var(--primary)] !text-[var(--primary)]")
                            )}
                        >
                            <span>{ NAV_STUDIO }</span>
                        </a>
                    </div>
                </div>

                // Mobile header (< md): brand / search / theme / menu.
                // Studio ingestion actions live at the bottom of the drawer
                // instead of competing for space in a 390px bar.
                <div class={classes!(
                    "mobile-header",
                    "items-center", "justify-between", "gap-3",
                    "h-16",
                    "max-w-[1200px]", "mx-auto", "px-4", "sm:px-5"
                )}>
                    <Link<Route> to={Route::Home} classes="brand-logo min-w-0">
                        <span class="brand-logo-diamond" aria-hidden="true"></span>
                        <span>{"StaticFlow"}</span>
                    </Link<Route>>

                    <div class="flex shrink-0 items-center gap-1">
                        <button
                            type="button"
                            onclick={open_mobile_search_page}
                            class={mobile_icon_button.clone()}
                            aria-label={t::SEARCH_ARIA}
                        >
                            <i class="fas fa-search text-[15px]" aria-hidden="true"></i>
                        </button>

                        <ThemeToggle class={classes!("!w-10", "!h-10")} />

                        <button
                            type="button"
                            class={classes!(
                                mobile_icon_button,
                                "flex-col", "gap-[4px]"
                            )}
                            aria-label={t::OPEN_MENU_ARIA}
                            aria-expanded={(*mobile_menu_open).to_string()}
                            aria-controls="sf-mobile-drawer"
                            onclick={toggle_mobile_menu}
                        >
                            <span class={hamburger_line.clone()} />
                            <span class={hamburger_line.clone()} />
                            <span class={hamburger_line} />
                        </button>
                    </div>
                </div>
            </header>

            // Full-screen opaque editorial drawer (< md only)
            <div
                id="sf-mobile-drawer"
                class={classes!(
                    "sf-mobile-drawer",
                    "md:hidden",
                    "fixed", "inset-0", "z-[120]",
                    "bg-[var(--bg)]", "text-[var(--text)]",
                    "flex", "flex-col",
                    "overflow-y-auto", "overscroll-contain",
                    (*mobile_menu_open).then_some("is-open")
                )}
                role="dialog"
                aria-modal="true"
                aria-label={t::MOBILE_NAV_ARIA}
                aria-hidden={(!*mobile_menu_open).to_string()}
            >
                <div class={classes!(
                    "sticky", "top-0", "z-[1]",
                    "h-16", "shrink-0", "px-4", "sm:px-5",
                    "flex", "items-center", "justify-between",
                    "bg-[var(--bg)]",
                    "border-b", "border-[var(--border)]"
                )}>
                    <div onclick={close_mobile_menu.clone()}>
                        <Link<Route> to={Route::Home} classes="brand-logo">
                            <span class="brand-logo-diamond" aria-hidden="true"></span>
                            <span>{"StaticFlow"}</span>
                        </Link<Route>>
                    </div>
                    <button
                        type="button"
                        onclick={close_mobile_menu.clone()}
                        aria-label={t::CLOSE_TOOLTIP}
                        class={classes!(
                            "w-10", "h-10", "rounded-full",
                            "border", "border-[var(--border-strong)]",
                            "bg-transparent", "text-[var(--text)]",
                            "inline-flex", "items-center", "justify-center",
                            "hover:bg-[var(--surface-alt)]",
                            "transition-colors", "cursor-pointer"
                        )}
                    >
                        <i class="fas fa-times text-[15px]" aria-hidden="true"></i>
                    </button>
                </div>

                <div class={classes!("flex", "flex-1", "flex-col", "px-6", "sm:px-8")}>
                    <div class={classes!("mobile-search-suggest", "mt-5", "flex", "gap-2", "items-center")}>
                        <SearchSuggest
                            value={(*search_query).clone()}
                            on_change={{
                                let search_query = search_query.clone();
                                Callback::from(move |value: String| search_query.set(value))
                            }}
                            on_submit={mobile_search_submit}
                            input_class={classes!("sf-input", "w-full", "min-w-0")}
                        />
                        if !search_query.is_empty() {
                            <button
                                type="button"
                                onclick={mobile_clear_search}
                                aria-label={t::CLEAR_ARIA}
                                class={classes!(
                                    "w-11", "h-11", "shrink-0",
                                    "rounded-[8px]",
                                    "border", "border-[var(--border-strong)]",
                                    "bg-[var(--surface)]",
                                    "text-[var(--muted)]",
                                    "hover:text-[var(--text)]",
                                    "cursor-pointer"
                                )}
                            >
                                <i class="fas fa-times text-xs" aria-hidden="true"></i>
                            </button>
                        }
                    </div>

                    <nav
                        class={classes!("py-6", "flex", "flex-col")}
                        aria-label={t::MOBILE_NAV_ARIA}
                    >
                        { for mobile_nav_items.iter().map(|(label, target_route)| {
                            let active = is_nav_active(route.as_ref(), target_route);
                            let close_cb = close_mobile_menu.clone();
                            html! {
                                <div onclick={close_cb} class="border-b border-[var(--border)]">
                                    <Link<Route>
                                        to={target_route.clone()}
                                        classes={classes!(
                                            "flex", "items-center", "justify-between", "gap-3",
                                            "min-h-[3.5rem]", "py-2",
                                            "font-serif", "font-normal",
                                            "text-[1.5rem]", "leading-[1.2]",
                                            "tracking-[-0.015em]", "no-underline",
                                            "hover:no-underline",
                                            "transition-colors",
                                            if active {
                                                "text-[var(--primary)]"
                                            } else {
                                                "text-[var(--text)] hover:text-[var(--primary)]"
                                            }
                                        )}
                                    >
                                        <span>{ *label }</span>
                                        <i
                                            class={classes!(
                                                "fas", "fa-arrow-right", "text-[13px]",
                                                if active { "text-[var(--primary)]" } else { "text-[var(--faint)]" }
                                            )}
                                            aria-hidden="true"
                                        ></i>
                                    </Link<Route>>
                                </div>
                            }
                        }) }
                    </nav>

                    <div class={classes!(
                        "mt-auto", "pt-4",
                        "pb-[calc(1.5rem+env(safe-area-inset-bottom,0px))]",
                        "grid", "grid-cols-2", "gap-3"
                    )}>
                        <a
                            href={crate::config::route_path("/studio#article")}
                            onclick={go_studio_article_mobile}
                            class={classes!("sf-btn", "sf-btn--primary", "w-full", "!h-12", "justify-center", "no-underline")}
                        >
                            { CTA_STUDIO_ARTICLE }
                        </a>
                        <a
                            href={crate::config::route_path("/studio#music")}
                            onclick={go_studio_music_mobile}
                            class={classes!("sf-btn", "sf-btn--secondary", "w-full", "!h-12", "justify-center", "no-underline")}
                        >
                            { CTA_STUDIO_MUSIC }
                        </a>
                    </div>
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
