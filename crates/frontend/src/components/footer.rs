use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    i18n::current::{common as common_text, footer as t, header as header_text},
    router::Route,
};

#[function_component(Footer)]
pub fn footer() -> Html {
    let nav_links = [
        (header_text::NAV_POSTS, Route::Posts),
        (header_text::NAV_LATEST, Route::LatestArticles),
        (header_text::NAV_TAGS, Route::Tags),
        (header_text::NAV_CATEGORIES, Route::Categories),
        (header_text::NAV_MUSIC, Route::MediaAudio),
        ("图库", Route::MediaImage),
    ];

    html! {
        <footer class={classes!(
            "border-t", "border-[var(--border)]",
            "bg-[var(--bg)]",
            "mt-20",
            "py-16",
            "text-[var(--muted)]"
        )}>
            <div class={classes!(
                "max-w-[1200px]",
                "mx-auto",
                "px-6", "md:px-10"
            )}>
                <div class={classes!(
                    "grid", "grid-cols-1",
                    "md:grid-cols-[1.35fr_1fr_1.15fr]",
                    "gap-10", "md:gap-12"
                )}>
                    // Column 1: Brand wordmark + editorial motto + author links
                    <div class="flex flex-col items-start">
                        <Link<Route> to={Route::Home} classes="brand-logo">
                            <span class="brand-logo-diamond" aria-hidden="true"></span>
                            <span>{"StaticFlow"}</span>
                        </Link<Route>>
                        <p class={classes!(
                            "mt-3", "mb-0",
                            "font-serif", "italic", "font-normal",
                            "text-[0.95rem]", "leading-relaxed",
                            "text-[var(--muted)]"
                        )}>
                            {"Written locally. Read anywhere. Grown by readers."}
                        </p>
                        <div
                            class={classes!("mt-4", "flex", "items-center", "gap-1.5")}
                            aria-label={t::SOCIAL_ARIA}
                        >
                            <a
                                class={classes!(
                                    "inline-flex", "h-8", "w-8",
                                    "items-center", "justify-center",
                                    "rounded-full", "text-[var(--muted)]",
                                    "transition-colors",
                                    "hover:bg-[var(--surface-alt)]",
                                    "hover:text-[var(--text)]",
                                    "no-underline"
                                )}
                                href="https://github.com/ACking-you"
                                target="_blank"
                                rel="noopener noreferrer"
                                aria-label={common_text::GITHUB}
                                title={common_text::GITHUB}
                            >
                                <i class="fa-brands fa-github text-[15px]" aria-hidden="true"></i>
                                <span class="sr-only">{ common_text::GITHUB }</span>
                            </a>
                            <a
                                class={classes!(
                                    "inline-flex", "h-8", "w-8",
                                    "items-center", "justify-center",
                                    "rounded-full", "text-[var(--muted)]",
                                    "transition-colors",
                                    "hover:bg-[var(--surface-alt)]",
                                    "hover:text-[var(--text)]",
                                    "no-underline"
                                )}
                                href="https://space.bilibili.com/24264499"
                                target="_blank"
                                rel="noopener noreferrer"
                                aria-label={common_text::BILIBILI}
                                title={common_text::BILIBILI}
                            >
                                <svg
                                    viewBox="0 0 24 24"
                                    role="img"
                                    aria-hidden="true"
                                    focusable="false"
                                    width="16"
                                    height="16"
                                >
                                    <path
                                        fill="currentColor"
                                        d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.858 0 17.347v-7.36c.036-1.511.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.234 1.234 0 0 1-.373-.906c0-.356.124-.658.373-.907l.027-.027c.267-.249.573-.373.92-.373.347 0 .653.124.92.373L9.653 4.44c.071.071.134.142.187.213h4.267a.836.836 0 0 1 .16-.213l2.853-2.747c.267-.249.573-.373.92-.373.347 0 .662.151.929.4.267.249.391.551.391.907 0 .355-.124.657-.373.906zM5.333 7.24c-.746.018-1.373.276-1.88.773-.506.498-.769 1.13-.786 1.894v7.52c.017.764.28 1.395.786 1.893.507.498 1.134.756 1.88.773h13.334c.746-.017 1.373-.275 1.88-.773.506-.498.769-1.129.786-1.893v-7.52c-.017-.765-.28-1.396-.786-1.894-.507-.497-1.134-.755-1.88-.773zM8 11.107c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c0-.373.129-.689.386-.947.258-.257.574-.386.947-.386zm8 0c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c.017-.391.15-.711.4-.96.249-.249.56-.373.933-.373Z"
                                    />
                                </svg>
                                <span class="sr-only">{ common_text::BILIBILI }</span>
                            </a>
                        </div>
                    </div>

                    // Column 2: Editorial navigation
                    <div class="flex flex-col items-start">
                        <span class="sf-eyebrow">{"导航"}</span>
                        <nav
                            class={classes!("grid", "grid-cols-2", "gap-x-8", "gap-y-2.5")}
                            aria-label="页脚导航"
                        >
                            { for nav_links.iter().map(|(label, target_route)| html! {
                                <Link<Route>
                                    to={target_route.clone()}
                                    classes={classes!(
                                        "text-[0.875rem]",
                                        "text-[var(--muted)]",
                                        "hover:text-[var(--primary)]",
                                        "transition-colors",
                                        "no-underline"
                                    )}
                                >
                                    { *label }
                                </Link<Route>>
                            }) }
                        </nav>
                    </div>

                    // Column 3: Open-source repository & stack nameplate
                    <div class="flex flex-col items-start">
                        <span class="sf-eyebrow">{"开源"}</span>
                        <a
                            href="https://github.com/acking-you/static_flow"
                            target="_blank"
                            rel="noopener noreferrer"
                            class={classes!(
                                "inline-flex", "items-center", "gap-2",
                                "font-mono", "text-[0.8125rem]",
                                "text-[var(--text)]", "hover:text-[var(--primary)]",
                                "transition-colors", "no-underline"
                            )}
                        >
                            <i class="fa-brands fa-github text-sm" aria-hidden="true"></i>
                            <span>{"acking-you/static_flow"}</span>
                        </a>
                        <p class={classes!(
                            "mt-3", "mb-0",
                            "font-mono", "text-[0.75rem]", "leading-relaxed",
                            "text-[var(--faint)]"
                        )}>
                            {"Built with Rust · Yew · WebAssembly · LanceDB"}
                        </p>
                    </div>
                </div>

                // Bottom hairline metadata row
                <div class={classes!(
                    "mt-12", "pt-6",
                    "border-t", "border-[var(--border)]",
                    "flex", "flex-col", "sm:flex-row",
                    "items-start", "sm:items-center", "justify-between",
                    "gap-2",
                    "font-mono", "text-[0.75rem]", "tabular-nums",
                    "text-[var(--faint)]"
                )}>
                    <span>{ t::COPYRIGHT }</span>
                    <span>{"rendered by wasm32-unknown-unknown"}</span>
                </div>
            </div>
        </footer>
    }
}
