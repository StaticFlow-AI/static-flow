//! Dedicated `/studio` co-creation workspace hosting article ingestion and
//! music wish desks.

use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use wasm_bindgen::JsValue;
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::use_location;

use crate::{
    components::{
        article_request_desk::ArticleRequestDesk, music_wish_desk::MusicWishDesk,
        scroll_to_top_button::ScrollToTopButton,
    },
    i18n::current::studio_page as st_t,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StudioTab {
    Article,
    Music,
}

impl StudioTab {
    fn from_hash(raw_hash: &str) -> Option<Self> {
        match raw_hash.trim() {
            "#article" | "#article-request-section" | "#article-request-form" => {
                Some(Self::Article)
            },
            "#music" | "#music-wish-section" | "#music-wish-form" => Some(Self::Music),
            _ => None,
        }
    }

    fn current_or_default() -> (Self, bool) {
        let hash = window()
            .and_then(|w| w.location().hash().ok())
            .unwrap_or_default();
        match Self::from_hash(&hash) {
            Some(tab) => (tab, true),
            None => (Self::Article, false),
        }
    }

    const fn hash_str(self) -> &'static str {
        match self {
            Self::Article => "#article",
            Self::Music => "#music",
        }
    }

    const fn section_id(self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::Music => "music",
        }
    }
}

fn smooth_scroll_to_section(id: &str) {
    let Some(win) = window() else {
        return;
    };
    let Some(doc) = win.document() else {
        return;
    };
    let Some(el) = doc.get_element_by_id(id) else {
        return;
    };
    let rect = el.get_bounding_client_rect();
    let scroll_y = win.scroll_y().unwrap_or(0.0);
    let target_top = (rect.top() + scroll_y - 88.0).max(0.0);
    let opts = web_sys::ScrollToOptions::new();
    opts.set_top(target_top);
    opts.set_behavior(web_sys::ScrollBehavior::Smooth);
    win.scroll_with_scroll_to_options(&opts);
}

fn replace_url_hash(hash: &str) {
    let Some(win) = window() else {
        return;
    };
    let Ok(pathname) = win.location().pathname() else {
        return;
    };
    let search = win.location().search().unwrap_or_default();
    let next_url = format!("{pathname}{search}{hash}");
    if let Ok(history) = win.history() {
        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&next_url));
    }
}

/// Reader-driven ingestion studio page (`/studio`).
#[function_component(StudioPage)]
pub fn studio_page() -> Html {
    let location = use_location();
    let initial = StudioTab::current_or_default();
    let active_tab = use_state(|| initial.0);
    let focused_tab = use_state(|| initial.1.then_some(initial.0));
    let article_total = use_state(|| None::<usize>);
    let music_total = use_state(|| None::<usize>);

    // Sync active tab and scroll target on mount, route transition, hashchange, and
    // popstate
    {
        let active_tab = active_tab.clone();
        let focused_tab = focused_tab.clone();
        let loc_dep = location.clone();
        use_effect_with(loc_dep, move |_| {
            let sync_from_hash = {
                let active_tab = active_tab.clone();
                let focused_tab = focused_tab.clone();
                move |allow_scroll: bool| {
                    let hash = window()
                        .and_then(|w| w.location().hash().ok())
                        .unwrap_or_default();
                    if let Some(tab) = StudioTab::from_hash(&hash) {
                        active_tab.set(tab);
                        focused_tab.set(Some(tab));
                        if allow_scroll {
                            let target_id = tab.section_id();
                            Timeout::new(60, move || {
                                // Only scroll on desktop or when section is below header
                                if let Some(win) = window() {
                                    let width = win
                                        .inner_width()
                                        .ok()
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or(1024.0);
                                    if width >= 1024.0 {
                                        smooth_scroll_to_section(target_id);
                                    } else {
                                        smooth_scroll_to_section("studio-tabs");
                                    }
                                }
                            })
                            .forget();
                        }
                    } else {
                        focused_tab.set(None);
                    }
                }
            };

            sync_from_hash(true);

            let mut hash_listener: Option<EventListener> = None;
            let mut pop_listener: Option<EventListener> = None;
            if let Some(win) = window() {
                let sync_on_hash = sync_from_hash.clone();
                hash_listener = Some(EventListener::new(&win, "hashchange", move |_| {
                    sync_on_hash(true);
                }));
                pop_listener = Some(EventListener::new(&win, "popstate", move |_| {
                    sync_from_hash(true);
                }));
            }

            move || {
                drop(hash_listener);
                drop(pop_listener);
            }
        });
    }

    let select_article_tab = {
        let active_tab = active_tab.clone();
        let focused_tab = focused_tab.clone();
        Callback::from(move |_: MouseEvent| {
            active_tab.set(StudioTab::Article);
            focused_tab.set(Some(StudioTab::Article));
            replace_url_hash(StudioTab::Article.hash_str());
        })
    };

    let select_music_tab = {
        let active_tab = active_tab.clone();
        let focused_tab = focused_tab.clone();
        Callback::from(move |_: MouseEvent| {
            active_tab.set(StudioTab::Music);
            focused_tab.set(Some(StudioTab::Music));
            replace_url_hash(StudioTab::Music.hash_str());
        })
    };

    let focus_article_desktop = {
        let active_tab = active_tab.clone();
        let focused_tab = focused_tab.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            active_tab.set(StudioTab::Article);
            focused_tab.set(Some(StudioTab::Article));
            replace_url_hash(StudioTab::Article.hash_str());
            smooth_scroll_to_section("article");
        })
    };

    let focus_music_desktop = {
        let active_tab = active_tab.clone();
        let focused_tab = focused_tab.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            active_tab.set(StudioTab::Music);
            focused_tab.set(Some(StudioTab::Music));
            replace_url_hash(StudioTab::Music.hash_str());
            smooth_scroll_to_section("music");
        })
    };

    let on_article_total = {
        let article_total = article_total.clone();
        Callback::from(move |count: usize| article_total.set(Some(count)))
    };

    let on_music_total = {
        let music_total = music_total.clone();
        Callback::from(move |count: usize| music_total.set(Some(count)))
    };

    let mobile_tab_class = |is_active: bool| {
        classes!(
            "inline-flex",
            "min-h-[44px]",
            "items-center",
            "gap-2",
            "border-b-[1.5px]",
            "px-1",
            "py-2.5",
            "text-sm",
            "bg-transparent",
            "border-t-0",
            "border-x-0",
            "cursor-pointer",
            "transition-colors",
            if is_active {
                "border-[var(--primary)] font-medium text-[var(--text)]"
            } else {
                "border-transparent text-[var(--muted)] hover:text-[var(--text)]"
            }
        )
    };

    let is_article_active = *active_tab == StudioTab::Article;
    let is_music_active = *active_tab == StudioTab::Music;

    html! {
        <main class="min-h-[70vh] pb-24 min-w-0">
            // Editorial Page Hero
            <div class="max-w-[1200px] mx-auto px-4 sm:px-6 md:px-10">
                <header class="pt-12 sm:pt-16 pb-8 sm:pb-10 border-b border-[var(--border)]">
                    <div class="sf-eyebrow">
                        { st_t::EYEBROW }
                    </div>
                    <div class="mt-2 flex flex-col lg:flex-row lg:items-end lg:justify-between gap-5">
                        <div class="max-w-[64ch] min-w-0">
                            <h1 class="m-0 font-serif font-normal text-[clamp(2rem,1.5rem+2vw,3rem)] leading-[1.15] tracking-[-0.015em] text-[var(--text)]">
                                { st_t::TITLE }
                            </h1>
                            <p class="mt-3 mb-0 text-[0.95rem] leading-[1.65] text-[var(--text-2)]">
                                { st_t::SUBTITLE }
                            </p>
                            <div class="mt-4 font-mono text-xs tabular-nums tracking-[0.03em] text-[var(--faint)] [overflow-wrap:anywhere]">
                                { st_t::PIPELINE }
                            </div>
                        </div>

                        // Desktop quick anchors
                        <nav
                            aria-label="共建台分区定位"
                            class="hidden lg:flex items-center gap-2 shrink-0"
                        >
                            <a
                                href="#article"
                                onclick={focus_article_desktop}
                                class={classes!(
                                    "sf-btn",
                                    "sf-btn--ghost",
                                    "min-h-[40px]",
                                    "!px-3",
                                    "font-mono",
                                    "!text-xs",
                                    (*focused_tab == Some(StudioTab::Article)).then_some("!text-[var(--primary)]")
                                )}
                            >
                                <span>{ format!("# {}", st_t::TAB_ARTICLE) }</span>
                                if let Some(c) = *article_total {
                                    <span class="tabular-nums text-[var(--faint)]">{ c }</span>
                                }
                            </a>
                            <span class="text-[var(--border-strong)]" aria-hidden="true">{ "·" }</span>
                            <a
                                href="#music"
                                onclick={focus_music_desktop}
                                class={classes!(
                                    "sf-btn",
                                    "sf-btn--ghost",
                                    "min-h-[40px]",
                                    "!px-3",
                                    "font-mono",
                                    "!text-xs",
                                    (*focused_tab == Some(StudioTab::Music)).then_some("!text-[var(--primary)]")
                                )}
                            >
                                <span>{ format!("# {}", st_t::TAB_MUSIC) }</span>
                                if let Some(c) = *music_total {
                                    <span class="tabular-nums text-[var(--faint)]">{ c }</span>
                                }
                            </a>
                        </nav>
                    </div>
                </header>

                // Mobile / Tablet (< lg) Top Editorial Text Tabs (1.5px terracotta underline)
                <div
                    id="studio-tabs"
                    role="tablist"
                    aria-label="共建台类型切换"
                    class="lg:hidden flex items-center gap-6 border-b border-[var(--border)] scroll-mt-20"
                >
                    <button
                        type="button"
                        role="tab"
                        id="studio-tab-article"
                        aria-selected={is_article_active.to_string()}
                        aria-controls="article"
                        onclick={select_article_tab}
                        class={mobile_tab_class(is_article_active)}
                    >
                        <span>{ st_t::TAB_ARTICLE }</span>
                        if let Some(c) = *article_total {
                            <span class="font-mono text-xs tabular-nums text-[var(--faint)]">{ c }</span>
                        }
                    </button>
                    <button
                        type="button"
                        role="tab"
                        id="studio-tab-music"
                        aria-selected={is_music_active.to_string()}
                        aria-controls="music"
                        onclick={select_music_tab}
                        class={mobile_tab_class(is_music_active)}
                    >
                        <span>{ st_t::TAB_MUSIC }</span>
                        if let Some(c) = *music_total {
                            <span class="font-mono text-xs tabular-nums text-[var(--faint)]">{ c }</span>
                        }
                    </button>
                </div>

                // Two-Desk Workspace: side-by-side on >= lg, active tab on < lg
                <div class="pt-8 sm:pt-10 lg:pt-12 grid grid-cols-1 lg:grid-cols-2 gap-10 lg:gap-12 items-start min-w-0">
                    <section
                        id="article"
                        aria-labelledby="ar-desk-heading"
                        class={classes!(
                            "min-w-0",
                            "scroll-mt-24",
                            (!is_article_active).then_some("hidden"),
                            "lg:block"
                        )}
                    >
                        <ArticleRequestDesk
                            on_total_loaded={Some(on_article_total)}
                            highlighted={*focused_tab == Some(StudioTab::Article)}
                        />
                    </section>

                    <section
                        id="music"
                        aria-labelledby="mw-desk-heading"
                        class={classes!(
                            "min-w-0",
                            "scroll-mt-24",
                            "lg:border-l",
                            "lg:border-[var(--border)]",
                            "lg:pl-12",
                            (!is_music_active).then_some("hidden"),
                            "lg:block"
                        )}
                    >
                        <MusicWishDesk
                            on_total_loaded={Some(on_music_total)}
                            highlighted={*focused_tab == Some(StudioTab::Music)}
                        />
                    </section>
                </div>
            </div>

            <ScrollToTopButton />
        </main>
    }
}
