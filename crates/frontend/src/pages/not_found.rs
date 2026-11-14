use yew::prelude::*;
use yew_router::prelude::*;

use crate::{i18n::current::not_found_page as t, router::Route};

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <main class={classes!(
            "max-w-[1200px]",
            "mx-auto",
            "px-6",
            "md:px-10",
            "py-24",
            "md:py-32",
            "min-h-[65vh]",
            "flex",
            "items-center"
        )}>
            <div class="max-w-[60ch]">
                <div class="sf-eyebrow text-xs font-semibold uppercase tracking-[0.08em] text-[var(--faint)]">
                    { "404 · PAGE NOT FOUND" }
                </div>
                <h1 class="mt-4 mb-0 font-serif font-normal text-[clamp(2.25rem,1.6rem+2.5vw,3.5rem)] leading-[1.12] tracking-[-0.02em] text-[var(--text)]">
                    { "这一页不在文库目录中。" }
                </h1>
                <p class="mt-4 mb-0 font-serif italic text-lg text-[var(--muted)]">
                    { t::ERROR_DETAIL }
                </p>
                <p class="mt-4 mb-0 text-[0.95rem] leading-[1.65] text-[var(--text-2)]">
                    { "链接可能已变更或输入有误。你可以返回首页探索读者共创文库，或直接浏览最新与归档目录。" }
                </p>

                <hr class="sf-rule my-8 border-0 border-t border-[var(--border)]" />

                <div class={classes!("flex", "flex-wrap", "items-center", "gap-3")}>
                    <Link<Route>
                        to={Route::Home}
                        classes={classes!("sf-btn", "sf-btn--primary", "no-underline")}
                    >
                        { t::BTN_HOME }
                        <span aria-hidden="true">{ "→" }</span>
                    </Link<Route>>
                    <Link<Route>
                        to={Route::LatestArticles}
                        classes={classes!("sf-btn", "sf-btn--secondary", "no-underline")}
                    >
                        { t::BTN_LATEST }
                    </Link<Route>>
                    <Link<Route>
                        to={Route::Posts}
                        classes={classes!("sf-btn", "sf-btn--ghost", "no-underline")}
                    >
                        { t::BTN_ARCHIVE }
                    </Link<Route>>
                </div>
            </div>
        </main>
    }
}
