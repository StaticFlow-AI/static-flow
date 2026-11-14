use wasm_bindgen::JsCast;
use web_sys::console;
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    api::{self, SongListItem},
    components::{
        article_card::ArticleCard,
        icons::{Icon, IconName},
        image_with_loading::ImageWithLoading,
        skeleton::SkeletonCard,
    },
    i18n::current::{common as common_text, home as t},
    models::ArticleListItem,
    router::Route,
};

fn format_duration_short(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let total_articles = use_state(|| 0usize);
    let total_tags = use_state(|| 0usize);
    let total_categories = use_state(|| 0usize);
    let total_music = use_state(|| 0usize);
    let total_images = use_state(|| 0usize);
    let stats_loaded = use_state(|| false);
    let recent_articles = use_state(Vec::<ArticleListItem>::new);
    let articles_loaded = use_state(|| false);
    let recent_songs = use_state(Vec::<SongListItem>::new);
    let songs_loaded = use_state(|| false);
    let avatar_loaded = use_state(|| false);

    {
        let total_articles = total_articles.clone();
        let total_tags = total_tags.clone();
        let total_categories = total_categories.clone();
        let total_music = total_music.clone();
        let total_images = total_images.clone();
        let stats_loaded = stats_loaded.clone();
        let recent_articles = recent_articles.clone();
        let articles_loaded = articles_loaded.clone();
        let recent_songs = recent_songs.clone();
        let songs_loaded = songs_loaded.clone();
        use_effect_with((), move |_| {
            {
                let total_articles = total_articles.clone();
                let total_tags = total_tags.clone();
                let total_categories = total_categories.clone();
                let stats_loaded = stats_loaded.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_site_stats().await {
                        Ok(stats) => {
                            total_articles.set(stats.total_articles);
                            total_tags.set(stats.total_tags);
                            total_categories.set(stats.total_categories);
                        },
                        Err(e) => {
                            console::error_1(&format!("Failed to fetch home stats: {e}").into());
                        },
                    }
                    stats_loaded.set(true);
                });
            }
            {
                let total_images = total_images.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_images_page(Some(1), Some(0)).await {
                        Ok(resp) => total_images.set(resp.total),
                        Err(e) => {
                            console::error_1(&format!("Failed to fetch image stats: {e}").into());
                        },
                    }
                });
            }
            {
                let total_music = total_music.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_songs(Some(1), Some(0), None, None, None).await {
                        Ok(resp) => total_music.set(resp.total),
                        Err(e) => {
                            console::error_1(&format!("Failed to fetch songs total: {e}").into());
                        },
                    }
                });
            }
            {
                let recent_songs = recent_songs.clone();
                let songs_loaded = songs_loaded.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_random_recommended_songs(Some(4), &[]).await {
                        Ok(songs) => recent_songs.set(songs),
                        Err(e) => {
                            console::error_1(&format!("Failed to fetch random songs: {e}").into());
                        },
                    }
                    songs_loaded.set(true);
                });
            }
            {
                let recent_articles = recent_articles.clone();
                let articles_loaded = articles_loaded.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::fetch_articles(None, None, Some(4), Some(0)).await {
                        Ok(page) => recent_articles.set(page.articles),
                        Err(e) => {
                            console::error_1(
                                &format!("Failed to fetch recent articles: {e}").into(),
                            );
                        },
                    }
                    articles_loaded.set(true);
                });
            }
            || ()
        });
    }

    let stats = [
        (IconName::FileText, *total_articles, t::STATS_ARTICLES, Route::Posts),
        (IconName::Hash, *total_tags, t::STATS_TAGS, Route::Tags),
        (IconName::Folder, *total_categories, t::STATS_CATEGORIES, Route::Categories),
        (IconName::Music, *total_music, t::STATS_MUSIC, Route::MediaAudio),
        (IconName::Folder, *total_images, t::STATS_IMAGES, Route::MediaImage),
    ];

    let staticflow_search_href = crate::config::route_path("/search?q=staticflow");
    let on_staticflow_search_click = {
        let staticflow_search_href = staticflow_search_href.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&staticflow_search_href);
        })
    };

    let tech_stack = [
        (
            crate::config::asset_path("static/logos/rust.svg"),
            "Rust",
            "https://doc.rust-lang.org/book",
        ),
        (
            crate::config::asset_path("static/logos/yew.svg"),
            "Yew 0.23",
            "https://yew.rs/docs/getting-started/introduction",
        ),
        (
            crate::config::asset_path("static/logos/tailwind.svg"),
            "Tailwind v4",
            "https://tailwindcss.com/docs",
        ),
        (
            crate::config::asset_path("static/logos/lancedb.png"),
            "LanceDB",
            "https://lancedb.com/docs/",
        ),
        (
            crate::config::asset_path("static/logos/wasm.ico"),
            "WebAssembly",
            "https://webassembly.org/getting-started/developers-guide",
        ),
    ];

    let on_avatar_load = {
        let avatar_loaded = avatar_loaded.clone();
        Callback::from(move |_: Event| avatar_loaded.set(true))
    };

    html! {
        <main class={classes!(
            "w-full",
            "min-h-screen",
            "bg-[var(--bg)]",
            "pb-16"
        )}>
            <div class={classes!(
                "max-w-6xl",
                "mx-auto",
                "px-4", "sm:px-6", "lg:px-8",
                "pt-8", "md:pt-12",
                "space-y-10", "md:space-y-14"
            )}>
                // 1. Editorial Studio Hero
                <section class={classes!(
                    "rounded-2xl",
                    "border", "border-[var(--border)]",
                    "bg-[var(--surface)]",
                    "p-6", "sm:p-8", "md:p-10",
                    "shadow-[var(--shadow-sm)]"
                )}>
                    <div class={classes!(
                        "flex", "flex-col-reverse", "md:flex-row",
                        "md:items-start", "md:justify-between",
                        "gap-6", "md:gap-10"
                    )}>
                        <div class="flex-1 min-w-0">
                            <div class="sf-eyebrow">
                                <span class="w-1.5 h-1.5 rounded-full bg-[var(--primary)]" aria-hidden="true"></span>
                                { "LOCAL-FIRST RUST KNOWLEDGE STUDIO" }
                            </div>

                            <div class="flex flex-wrap items-baseline gap-3">
                                <h1 class={classes!(
                                    "m-0",
                                    "text-3xl", "sm:text-4xl", "md:text-[2.65rem]",
                                    "font-extrabold", "tracking-[-0.035em]", "leading-[1.12]",
                                    "text-[var(--text)]"
                                )}>
                                    { "StaticFlow" }
                                </h1>
                                <span class="font-mono text-sm text-[var(--muted)]">
                                    { "by L_B__" }
                                </span>
                            </div>

                            <p class={classes!(
                                "mt-3", "mb-0",
                                "text-sm", "sm:text-base", "font-medium",
                                "text-[var(--text)]/90"
                            )}>
                                { t::MOTTO }
                            </p>

                            <p class={classes!(
                                "mt-2.5", "mb-0",
                                "text-sm", "sm:text-[0.95rem]", "leading-relaxed",
                                "text-[var(--muted)]", "max-w-2xl"
                            )}>
                                { t::INTRO }
                            </p>

                            // Open-source repo callout (pointing to acking-you/static_flow)
                            <div class={classes!(
                                "mt-4",
                                "inline-flex", "flex-wrap", "items-center", "gap-1.5",
                                "text-xs", "sm:text-[13px]", "text-[var(--muted)]",
                                "bg-[var(--surface-alt)]",
                                "border", "border-[var(--border)]",
                                "rounded-lg", "px-3", "py-1.5"
                            )}>
                                <span>{ t::OPEN_SOURCE_INLINE }</span>
                                <a
                                    href="https://github.com/acking-you/static_flow"
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="font-semibold text-[var(--primary)] hover:underline"
                                >
                                    { t::OPEN_SOURCE_GITHUB_CTA }
                                </a>
                            </div>

                            // Primary navigation actions
                            <div class={classes!("flex", "flex-wrap", "items-center", "gap-2.5", "mt-6")}>
                                <Link<Route>
                                    to={Route::LatestArticles}
                                    classes={classes!("btn-fluent-primary")}
                                >
                                    <i class="fas fa-arrow-right text-xs" aria-hidden="true"></i>
                                    { t::BTN_VIEW_ARTICLES }
                                </Link<Route>>
                                <Link<Route>
                                    to={Route::Posts}
                                    classes={classes!("btn-fluent-secondary")}
                                >
                                    <i class="fas fa-box-archive text-xs text-[var(--muted)]" aria-hidden="true"></i>
                                    { t::BTN_ARCHIVE }
                                </Link<Route>>
                                <a
                                    href={staticflow_search_href}
                                    onclick={on_staticflow_search_click}
                                    class={classes!("btn-fluent-search-hero")}
                                >
                                    <i class="fas fa-search" aria-hidden="true"></i>
                                    { t::BTN_SEARCH_STATICFLOW }
                                </a>
                                <Link<Route>
                                    to={Route::MediaAudio}
                                    classes={classes!("btn-terminal")}
                                >
                                    <i class="fas fa-headphones text-xs text-[var(--muted)]" aria-hidden="true"></i>
                                    { t::BTN_MEDIA_AUDIO }
                                </Link<Route>>
                                <Link<Route>
                                    to={Route::MediaImage}
                                    classes={classes!("btn-terminal")}
                                >
                                    <i class="fas fa-image text-xs text-[var(--muted)]" aria-hidden="true"></i>
                                    { t::BTN_IMAGE }
                                </Link<Route>>
                                <Link<Route>
                                    to={Route::Admin}
                                    classes={classes!("btn-terminal")}
                                >
                                    <i class="fas fa-sliders text-xs text-[var(--muted)]" aria-hidden="true"></i>
                                    { "Admin" }
                                </Link<Route>>
                            </div>

                            // Social & GitHub Wrapped row
                            <div class={classes!(
                                "mt-6", "pt-5",
                                "border-t", "border-[var(--border)]",
                                "flex", "flex-wrap", "items-center", "gap-3"
                            )}>
                                <div class="flex items-center gap-2">
                                    <a
                                        href="https://github.com/ACking-you"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        aria-label={common_text::GITHUB}
                                        class={classes!(
                                            "inline-flex", "h-9", "w-9",
                                            "items-center", "justify-center",
                                            "rounded-lg", "border", "border-[var(--border)]",
                                            "bg-[var(--surface-alt)]", "text-[var(--text)]",
                                            "hover:border-[var(--primary)]", "hover:text-[var(--primary)]",
                                            "transition-colors"
                                        )}
                                    >
                                        <i class="fa-brands fa-github text-base" aria-hidden="true"></i>
                                        <span class="sr-only">{ common_text::GITHUB }</span>
                                    </a>
                                    <a
                                        href="https://space.bilibili.com/24264499"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        aria-label={common_text::BILIBILI}
                                        class={classes!(
                                            "inline-flex", "h-9", "w-9",
                                            "items-center", "justify-center",
                                            "rounded-lg", "border", "border-[var(--border)]",
                                            "bg-[var(--surface-alt)]", "text-[var(--text)]",
                                            "hover:border-[var(--primary)]", "hover:text-[var(--primary)]",
                                            "transition-colors"
                                        )}
                                    >
                                        <svg viewBox="0 0 24 24" role="img" aria-hidden="true" focusable="false" width="18" height="18">
                                            <path
                                                fill="currentColor"
                                                d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.858 0 17.347v-7.36c.036-1.511.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.234 1.234 0 0 1-.373-.906c0-.356.124-.658.373-.907l.027-.027c.267-.249.573-.373.92-.373.347 0 .653.124.92.373L9.653 4.44c.071.071.134.142.187.213h4.267a.836.836 0 0 1 .16-.213l2.853-2.747c.267-.249.573-.373.92-.373.347 0 .662.151.929.4.267.249.391.551.391.907 0 .355-.124.657-.373.906zM5.333 7.24c-.746.018-1.373.276-1.88.773-.506.498-.769 1.13-.786 1.894v7.52c.017.764.28 1.395.786 1.893.507.498 1.134.756 1.88.773h13.334c.746-.017 1.373-.275 1.88-.773.506-.498.769-1.129.786-1.893v-7.52c-.017-.765-.28-1.396-.786-1.894-.507-.497-1.134-.755-1.88-.773zM8 11.107c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c0-.373.129-.689.386-.947.258-.257.574-.386.947-.386zm8 0c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c.017-.391.15-.711.4-.96.249-.249.56-.373.933-.373Z"
                                            />
                                        </svg>
                                        <span class="sr-only">{ common_text::BILIBILI }</span>
                                    </a>
                                </div>

                                <GithubWrappedSelector />
                            </div>
                        </div>

                        // Author Avatar
                        <div class="shrink-0">
                            <Link<Route>
                                to={Route::Posts}
                                classes={classes!(
                                    "hero-avatar-trigger",
                                    "relative",
                                    "inline-flex",
                                    "w-24", "h-24", "sm:w-28", "sm:h-28", "md:w-32", "md:h-32",
                                    "rounded-2xl",
                                    "border", "border-[var(--border)]",
                                    "bg-[var(--surface-alt)]",
                                    "overflow-hidden",
                                    "shadow-[var(--shadow-4)]",
                                    "no-underline"
                                )}
                            >
                                if !*avatar_loaded {
                                    <div class="absolute inset-0 skeleton" />
                                }
                                <img
                                    src={crate::config::asset_path("static/avatar.jpg")}
                                    alt={t::AVATAR_ALT}
                                    loading="eager"
                                    onload={on_avatar_load}
                                    class={classes!(
                                        "hero-avatar",
                                        "w-full", "h-full", "object-cover",
                                        "transition-opacity", "duration-300",
                                        if *avatar_loaded { "opacity-100" } else { "opacity-0" }
                                    )}
                                />
                                <span class="sr-only">{ t::AVATAR_LINK_SR }</span>
                            </Link<Route>>
                        </div>
                    </div>
                </section>

                // 2. Live Content Metrics Strip
                <section aria-label="站点数据概览">
                    <div class={classes!(
                        "grid", "gap-3",
                        "grid-cols-2", "sm:grid-cols-3", "lg:grid-cols-5"
                    )}>
                        { for stats.into_iter().map(|(icon, value, label, target_route)| {
                            html! {
                                <Link<Route> to={target_route} classes="no-underline group">
                                    <div class="system-panel-compact h-full">
                                        <div class="w-full flex items-center justify-between text-[var(--muted)]">
                                            <span class="text-xs font-medium tracking-wide">{ label }</span>
                                            <Icon name={icon} size={16} class={classes!("text-[var(--muted)]", "group-hover:text-[var(--primary)]", "transition-colors")} />
                                        </div>
                                        <div class="text-2xl sm:text-[1.65rem] font-bold tabular-nums leading-none text-[var(--text)] mt-1">
                                            if *stats_loaded {
                                                { value.to_string() }
                                            } else {
                                                <span class="skeleton h-6 w-12 inline-block rounded" />
                                            }
                                        </div>
                                    </div>
                                </Link<Route>>
                            }
                        }) }
                    </div>
                </section>

                // 3. LLM & Kiro Gateway Callout
                <section class={classes!(
                    "rounded-xl",
                    "border", "border-[var(--border)]",
                    "border-l-[3px]", "border-l-[var(--primary)]",
                    "bg-[var(--surface)]",
                    "px-5", "py-4",
                    "shadow-[var(--shadow-sm)]",
                    "flex", "flex-col", "md:flex-row",
                    "md:items-center", "md:justify-between", "gap-3"
                )}>
                    <p class="m-0 text-sm leading-relaxed text-[var(--muted)]">
                        { t::LLM_ACCESS_HINT }
                    </p>
                    <div class="flex flex-wrap gap-2 shrink-0">
                        <Link<Route>
                            to={Route::LlmAccess}
                            classes={classes!("btn-terminal", "btn-terminal-accent")}
                        >
                            <i class="fas fa-key text-xs" aria-hidden="true"></i>
                            { t::BTN_LLM_ACCESS }
                        </Link<Route>>
                        <Link<Route>
                            to={Route::KiroAccess}
                            classes={classes!("btn-terminal")}
                        >
                            <i class="fas fa-bolt text-xs text-[var(--muted)]" aria-hidden="true"></i>
                            { "Kiro Access" }
                        </Link<Route>>
                    </div>
                </section>

                // 4. Recent Articles Section
                <section>
                    <div class="sf-section-head">
                        <div>
                            <h2 class="sf-section-title">{ "最新文章" }</h2>
                            <p class="m-0 mt-0.5 text-xs text-[var(--muted)]">{ "近期技术深潜、源码剖析与工程记录" }</p>
                        </div>
                        <Link<Route>
                            to={Route::LatestArticles}
                            classes="text-sm font-semibold text-[var(--primary)] hover:underline no-underline shrink-0"
                        >
                            { t::BTN_VIEW_ALL_ARTICLES }
                        </Link<Route>>
                    </div>

                    if *articles_loaded {
                        <div class="grid gap-5 grid-cols-1 md:grid-cols-2">
                            { for recent_articles.iter().cloned().map(|article| html! {
                                <ArticleCard article={article} />
                            }) }
                        </div>
                    } else {
                        <div class="grid gap-5 grid-cols-1 md:grid-cols-2">
                            { for (0..4).map(|_| html! { <SkeletonCard /> }) }
                        </div>
                    }
                </section>

                // 5. Curated Music Section
                <section>
                    <div class="sf-section-head">
                        <div>
                            <h2 class="sf-section-title">{ "随机推荐曲目" }</h2>
                            <p class="m-0 mt-0.5 text-xs text-[var(--muted)]">{ "托管于本地 LanceDB 的无损与精选曲库" }</p>
                        </div>
                        <Link<Route>
                            to={Route::MediaAudio}
                            classes="text-sm font-semibold text-[var(--primary)] hover:underline no-underline shrink-0"
                        >
                            { t::BTN_VIEW_ALL_MUSIC }
                        </Link<Route>>
                    </div>

                    if *songs_loaded {
                        <div class="grid gap-3.5 grid-cols-1 sm:grid-cols-2">
                            { for recent_songs.iter().map(|song| {
                                let has_cover = song.cover_image.as_ref().is_some_and(|c| !c.is_empty());
                                let cover_url = api::song_cover_url(song.cover_image.as_deref());
                                let duration = format_duration_short(song.duration_ms);
                                html! {
                                    <Link<Route>
                                        to={Route::MusicPlayer { id: song.id.clone() }}
                                        classes={classes!(
                                            "no-underline", "group",
                                            "flex", "items-center", "gap-3.5",
                                            "p-3", "rounded-xl",
                                            "border", "border-[var(--border)]",
                                            "bg-[var(--surface)]",
                                            "shadow-[var(--shadow-sm)]",
                                            "transition-all", "duration-150",
                                            "hover:border-[rgba(var(--primary-rgb),0.5)]",
                                            "hover:shadow-[var(--shadow-4)]"
                                        )}
                                    >
                                        <div class={classes!(
                                            "w-14", "h-14", "rounded-lg", "shrink-0",
                                            "bg-[var(--surface-alt)]",
                                            "flex", "items-center", "justify-center",
                                            "overflow-hidden", "border", "border-[var(--border)]"
                                        )}>
                                            if has_cover {
                                                <ImageWithLoading
                                                    src={cover_url}
                                                    alt={song.title.clone()}
                                                    loading={Some(AttrValue::from("lazy"))}
                                                    class={classes!(
                                                        "w-full", "h-full", "object-cover",
                                                        "group-hover:scale-105",
                                                        "transition-transform", "duration-300"
                                                    )}
                                                    container_class={classes!("w-full", "h-full")}
                                                />
                                            } else {
                                                <Icon name={IconName::Music} size={22} class={classes!("text-[var(--muted)]")} />
                                            }
                                        </div>

                                        <div class="flex-1 min-w-0">
                                            <div class="text-sm font-semibold text-[var(--text)] truncate group-hover:text-[var(--primary)] transition-colors">
                                                { &song.title }
                                            </div>
                                            <div class="text-xs text-[var(--muted)] truncate mt-0.5">
                                                { &song.artist }
                                            </div>
                                        </div>

                                        <div class="flex items-center gap-2.5 shrink-0 pr-1">
                                            <span class="font-mono text-xs tabular-nums text-[var(--muted)]">
                                                { duration }
                                            </span>
                                            <span class={classes!(
                                                "w-7", "h-7", "rounded-full",
                                                "inline-flex", "items-center", "justify-center",
                                                "bg-[var(--surface-alt)]", "text-[var(--muted)]",
                                                "group-hover:bg-[var(--primary)]", "group-hover:text-white",
                                                "transition-colors"
                                            )}>
                                                <i class="fas fa-play text-[10px] ml-0.5" aria-hidden="true"></i>
                                            </span>
                                        </div>
                                    </Link<Route>>
                                }
                            }) }
                        </div>
                    } else {
                        <div class="grid gap-3.5 grid-cols-1 sm:grid-cols-2">
                            { for (0..4).map(|_| html! {
                                <div class="flex items-center gap-3.5 p-3 rounded-xl border border-[var(--border)] bg-[var(--surface)]">
                                    <div class="w-14 h-14 rounded-lg skeleton shrink-0" />
                                    <div class="flex-1 space-y-2">
                                        <div class="h-3.5 w-3/5 skeleton rounded" />
                                        <div class="h-3 w-2/5 skeleton rounded" />
                                    </div>
                                </div>
                            }) }
                        </div>
                    }
                </section>

                // 6. Tech Stack Strip
                <section class="pt-4 border-t border-[var(--border)] flex flex-wrap items-center justify-between gap-3">
                    <span class="font-mono text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--muted)]">
                        { t::POWERED_BY }
                    </span>
                    <div class="flex flex-wrap items-center gap-2">
                        { for tech_stack.iter().map(|(logo, name, href)| html! {
                            <a
                                href={(*href).to_string()}
                                target="_blank"
                                rel="noopener noreferrer"
                                title={*name}
                                aria-label={(*name).to_string()}
                                class={classes!(
                                    "inline-flex", "items-center", "gap-2",
                                    "rounded-full", "border", "border-[var(--border)]",
                                    "bg-[var(--surface)]", "px-3", "py-1",
                                    "text-xs", "font-medium", "text-[var(--muted)]",
                                    "no-underline",
                                    "transition-colors",
                                    "hover:text-[var(--text)]",
                                    "hover:border-[rgba(var(--primary-rgb),0.45)]"
                                )}
                            >
                                <ImageWithLoading
                                    src={logo.clone()}
                                    alt={*name}
                                    loading={Some(AttrValue::from("lazy"))}
                                    class={classes!("w-3.5", "h-3.5", "object-contain")}
                                    container_class={classes!("inline-flex")}
                                />
                                <span>{ *name }</span>
                            </a>
                        }) }
                    </div>
                </section>
            </div>
        </main>
    }
}

/// GitHub Wrapped year entry
#[derive(Clone)]
struct WrappedYear {
    year: u16,
    is_latest: bool,
}

impl WrappedYear {
    fn url(&self) -> String {
        format!("/standalone/github-wrapped-{}.html", self.year)
    }
}

/// Available GitHub Wrapped years (newest first)
fn get_wrapped_years() -> Vec<WrappedYear> {
    vec![
        WrappedYear {
            year: 2025,
            is_latest: true,
        },
        WrappedYear {
            year: 2024,
            is_latest: false,
        },
    ]
}

#[function_component(GithubWrappedSelector)]
fn github_wrapped_selector() -> Html {
    let expanded = use_state(|| false);
    let years = get_wrapped_years();
    let latest = years.first().cloned();

    let toggle_expand = {
        let expanded = expanded.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            expanded.set(!*expanded);
        })
    };

    let close_dropdown = {
        let expanded = expanded.clone();
        Callback::from(move |_| expanded.set(false))
    };

    {
        let expanded = expanded.clone();
        use_effect_with(*expanded, move |is_expanded| {
            let cleanup: Box<dyn FnOnce()> = if *is_expanded {
                let expanded = expanded.clone();
                let closure = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
                    move |_: web_sys::Event| {
                        expanded.set(false);
                    },
                );

                if let Some(window) = web_sys::window() {
                    let _ = window.add_event_listener_with_callback(
                        "click",
                        closure.as_ref().unchecked_ref(),
                    );
                    let window_clone = window.clone();
                    Box::new(move || {
                        let _ = window_clone.remove_event_listener_with_callback(
                            "click",
                            closure.as_ref().unchecked_ref(),
                        );
                    })
                } else {
                    Box::new(|| {})
                }
            } else {
                Box::new(|| {})
            };
            cleanup
        });
    }

    let Some(latest) = latest else {
        return html! {};
    };

    let has_multiple_years = years.len() > 1;
    let group_ref = use_node_ref();
    let dropdown_style = use_state(String::new);

    {
        let group_ref = group_ref.clone();
        let dropdown_style = dropdown_style.clone();
        use_effect_with(*expanded, move |is_expanded| {
            if *is_expanded {
                if let Some(el) = group_ref.cast::<web_sys::HtmlElement>() {
                    let rect = el.get_bounding_client_rect();
                    let top = rect.bottom() + 8.0;
                    let left = rect.left();
                    dropdown_style.set(format!("top: {}px; left: {}px;", top, left));
                }
            }
            || ()
        });
    }

    html! {
        <div class="github-wrapped-container">
            <div class="github-wrapped-group" ref={group_ref}>
                <a
                    href={latest.url()}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="github-wrapped-btn"
                >
                    <span class="github-wrapped-badge">{ t::GITHUB_WRAPPED_BADGE }</span>
                    <i class={classes!("fa-brands", "fa-github", "text-lg")} aria-hidden="true"></i>
                    <span class="github-wrapped-text">
                        <span class="github-wrapped-title">{ format!("{} GitHub Wrapped", latest.year) }</span>
                        <span class="github-wrapped-subtitle">{ t::GITHUB_WRAPPED_SUBTITLE }</span>
                    </span>
                </a>

                if has_multiple_years {
                    <button
                        type="button"
                        class={classes!(
                            "github-wrapped-expand",
                            if *expanded { "expanded" } else { "" }
                        )}
                        onclick={toggle_expand}
                        aria-label={t::WRAPPED_MORE_YEARS_ARIA}
                        aria-expanded={(*expanded).to_string()}
                    >
                        <i class="fas fa-chevron-down" aria-hidden="true"></i>
                    </button>
                }
            </div>

            if has_multiple_years && *expanded {
                <div
                    class="github-wrapped-dropdown"
                    style={(*dropdown_style).clone()}
                    onclick={close_dropdown.reform(|e: MouseEvent| e.stop_propagation())}
                >
                    <div class="github-wrapped-dropdown-header">
                        { t::WRAPPED_SELECT_YEAR }
                    </div>
                    { for years.iter().map(|y| html! {
                        <a
                            href={y.url()}
                            target="_blank"
                            rel="noopener noreferrer"
                            class={classes!(
                                "github-wrapped-dropdown-item",
                                if y.is_latest { "latest" } else { "" }
                            )}
                        >
                            <i class="fa-brands fa-github" aria-hidden="true"></i>
                            <span>{ format!("{} Wrapped", y.year) }</span>
                            if y.is_latest {
                                <span class="github-wrapped-latest-tag">{ t::WRAPPED_LATEST_TAG }</span>
                            }
                        </a>
                    }) }
                </div>
            }
        </div>
    }
}
