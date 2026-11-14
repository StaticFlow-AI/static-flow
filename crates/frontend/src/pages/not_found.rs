use yew::prelude::*;
use yew_router::prelude::*;

use crate::{i18n::current::not_found_page as t, router::Route};

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <main class={classes!(
            "container",
            "mx-auto",
            "px-4",
            "py-16",
            "flex",
            "justify-center",
            "items-center",
            "min-h-[65vh]"
        )}>
            <div class={classes!(
                "max-w-xl",
                "w-full",
                "rounded-2xl",
                "border",
                "border-[var(--border)]",
                "bg-[var(--surface)]",
                "p-8",
                "sm:p-10",
                "shadow-[var(--shadow-4)]"
            )}>
                <div class="sf-eyebrow">{ "404 · NOT FOUND" }</div>
                <h1 class="text-3xl sm:text-4xl font-bold tracking-[-0.03em] text-[var(--text)] m-0">
                    { t::ERROR_CODE }
                </h1>
                <p class="mt-3 mb-0 text-sm sm:text-base leading-relaxed text-[var(--muted)]">
                    { t::ERROR_DETAIL }
                </p>

                <ul class="mt-4 mb-0 pl-5 space-y-1.5 text-sm text-[var(--muted)]">
                    <li>{ t::SUGGESTION_1 }</li>
                    <li>{ t::SUGGESTION_2 }</li>
                </ul>

                <div class={classes!("flex", "flex-wrap", "gap-2.5", "mt-7")}>
                    <Link<Route>
                        to={Route::Home}
                        classes={classes!("btn-fluent-primary")}
                    >
                        <i class="fas fa-home text-xs" aria-hidden="true"></i>
                        { t::BTN_HOME }
                    </Link<Route>>
                    <Link<Route>
                        to={Route::LatestArticles}
                        classes={classes!("btn-fluent-secondary")}
                    >
                        <i class="fas fa-newspaper text-xs text-[var(--muted)]" aria-hidden="true"></i>
                        { t::BTN_LATEST }
                    </Link<Route>>
                    <Link<Route>
                        to={Route::Posts}
                        classes={classes!("btn-fluent-secondary")}
                    >
                        <i class="fas fa-box-archive text-xs text-[var(--muted)]" aria-hidden="true"></i>
                        { t::BTN_ARCHIVE }
                    </Link<Route>>
                </div>
            </div>
        </main>
    }
}
