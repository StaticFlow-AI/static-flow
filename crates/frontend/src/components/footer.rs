use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    i18n::current::{common as common_text, footer as t},
    router::Route,
};

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer class={classes!(
            "border-t", "border-[var(--border)]",
            "bg-[var(--surface)]",
            "mt-16",
            "py-10",
            "text-sm",
            "text-[var(--muted)]"
        )}>
            <div class={classes!(
                "max-w-7xl",
                "mx-auto",
                "px-4", "sm:px-6", "lg:px-8",
                "flex", "flex-col", "sm:flex-row",
                "items-center", "justify-between",
                "gap-4"
            )}>
                <div class={classes!("flex", "flex-col", "sm:flex-row", "items-center", "gap-2", "sm:gap-3")}>
                    <span class={classes!("font-semibold", "text-[var(--text)]")}>
                        {"StaticFlow"}
                    </span>
                    <span class="hidden sm:inline text-[var(--border)]">{"·"}</span>
                    <span>{t::COPYRIGHT}</span>
                </div>

                <div class={classes!("flex", "flex-wrap", "items-center", "justify-center", "gap-4")}>
                    <Link<Route> to={Route::LatestArticles} classes="text-[var(--muted)] hover:text-[var(--primary)] no-underline">
                        {"最新文章"}
                    </Link<Route>>
                    <Link<Route> to={Route::Posts} classes="text-[var(--muted)] hover:text-[var(--primary)] no-underline">
                        {"文章归档"}
                    </Link<Route>>
                    <Link<Route> to={Route::MediaAudio} classes="text-[var(--muted)] hover:text-[var(--primary)] no-underline">
                        {"音乐库"}
                    </Link<Route>>
                    <a
                        href="https://github.com/acking-you/static_flow"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-[var(--muted)] hover:text-[var(--primary)] no-underline"
                    >
                        {"GitHub 源码"}
                    </a>

                    <span class="h-3.5 w-[1px] bg-[var(--border)] hidden sm:inline-block" aria-hidden="true"></span>

                    <div class={classes!("flex", "items-center", "gap-1.5")} aria-label={t::SOCIAL_ARIA}>
                        <a
                            class={classes!(
                                "inline-flex", "h-8", "w-8",
                                "items-center", "justify-center",
                                "rounded-lg", "text-[var(--muted)]",
                                "transition-colors",
                                "hover:bg-[var(--surface-alt)]",
                                "hover:text-[var(--text)]"
                            )}
                            href="https://github.com/ACking-you"
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label={common_text::GITHUB}
                        >
                            <i class="fa-brands fa-github text-base" aria-hidden="true"></i>
                            <span class={classes!("sr-only")}>{ common_text::GITHUB }</span>
                        </a>
                        <a
                            class={classes!(
                                "inline-flex", "h-8", "w-8",
                                "items-center", "justify-center",
                                "rounded-lg", "text-[var(--muted)]",
                                "transition-colors",
                                "hover:bg-[var(--surface-alt)]",
                                "hover:text-[var(--text)]"
                            )}
                            href="https://space.bilibili.com/24264499"
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label={common_text::BILIBILI}
                        >
                            <svg
                                viewBox="0 0 24 24"
                                role="img"
                                aria-hidden="true"
                                focusable="false"
                                width="17"
                                height="17"
                            >
                                <path
                                    fill="currentColor"
                                    d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.858 0 17.347v-7.36c.036-1.511.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.234 1.234 0 0 1-.373-.906c0-.356.124-.658.373-.907l.027-.027c.267-.249.573-.373.92-.373.347 0 .653.124.92.373L9.653 4.44c.071.071.134.142.187.213h4.267a.836.836 0 0 1 .16-.213l2.853-2.747c.267-.249.573-.373.92-.373.347 0 .662.151.929.4.267.249.391.551.391.907 0 .355-.124.657-.373.906zM5.333 7.24c-.746.018-1.373.276-1.88.773-.506.498-.769 1.13-.786 1.894v7.52c.017.764.28 1.395.786 1.893.507.498 1.134.756 1.88.773h13.334c.746-.017 1.373-.275 1.88-.773.506-.498.769-1.129.786-1.893v-7.52c-.017-.765-.28-1.396-.786-1.894-.507-.497-1.134-.755-1.88-.773zM8 11.107c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c0-.373.129-.689.386-.947.258-.257.574-.386.947-.386zm8 0c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c.017-.391.15-.711.4-.96.249-.249.56-.373.933-.373Z"
                                />
                            </svg>
                            <span class={classes!("sr-only")}>{ common_text::BILIBILI }</span>
                        </a>
                    </div>
                </div>
            </div>
        </footer>
    }
}
