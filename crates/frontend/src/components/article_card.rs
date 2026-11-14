use yew::prelude::*;
use yew_router::prelude::{use_navigator, Link, Routable};

use crate::{
    components::image_with_loading::ImageWithLoading,
    models::{ArticleKind, ArticleListItem},
    router::Route,
    utils::image_url,
};

#[derive(Properties, PartialEq, Clone)]
pub struct ArticleCardProps {
    pub article: ArticleListItem,
    #[prop_or_default]
    pub on_before_navigate: Option<Callback<()>>,
}

#[function_component(ArticleCard)]
pub fn article_card(props: &ArticleCardProps) -> Html {
    let article = props.article.clone();
    let is_interactive = matches!(article.article_kind, ArticleKind::InteractiveRepost);
    let detail_route = Route::ArticleDetail {
        id: article.id.clone(),
    };
    let detail_href = detail_route.to_path();
    let has_image = article.featured_image.is_some();

    let navigator = use_navigator();
    let on_before_navigate = props.on_before_navigate.clone();

    let handle_title_click = {
        let navigator = navigator.clone();
        let route = detail_route.clone();
        let on_before_navigate = on_before_navigate.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(callback) = on_before_navigate.as_ref() {
                callback.emit(());
            }
            if let Some(nav) = navigator.as_ref() {
                nav.push(&route);
            }
        })
    };

    let handle_image_click = handle_title_click.clone();

    html! {
        <article
            class={classes!(
                "editorial-card",
                is_interactive.then_some("editorial-card--interactive"),
                "bg-[var(--surface)]",
                "border",
                "border-[var(--border)]",
                "rounded-[var(--radius-lg)]",
                "overflow-hidden",
                "flex",
                "flex-col",
                "h-full",
                "hover:border-[rgba(var(--primary-rgb),0.5)]",
                "group"
            )}
        >
            {
                if let Some(image) = article.featured_image.as_ref() {
                    let image_url_val = image_url(image);
                    let title = article.title.clone();
                    html! {
                        <a
                            href={detail_href.clone()}
                            class={classes!(
                                "block",
                                "aspect-[16/9]",
                                "overflow-hidden",
                                "relative",
                                "border-b",
                                "border-[var(--border)]"
                            )}
                            onclick={handle_image_click}
                        >
                            <ImageWithLoading
                                src={image_url_val}
                                alt={title}
                                loading={Some(AttrValue::from("lazy"))}
                                decoding={Some(AttrValue::from("async"))}
                                class={classes!(
                                    "w-full",
                                    "h-full",
                                    "object-cover",
                                    "transition-transform",
                                    "duration-500",
                                    "ease-out",
                                    "group-hover:scale-[1.03]"
                                )}
                                container_class={classes!("w-full", "h-full")}
                            />
                            if is_interactive {
                                <span class="interactive-card-seal" aria-hidden="true">
                                    <span class="interactive-card-seal__icon">
                                        <span class="interactive-card-seal__core"></span>
                                    </span>
                                    <span class="interactive-card-seal__label">{ "Interactive" }</span>
                                </span>
                            }
                        </a>
                    }
                } else {
                    html! {}
                }
            }

            <div class={classes!("p-5", "sm:p-6", "flex", "flex-col", "gap-2.5", "flex-1")}>
                // Eyebrow: Category + Date (+ Interactive badge)
                <div class={classes!("flex", "items-center", "justify-between", "gap-2", "flex-wrap")}>
                    <div class={classes!("flex", "items-center", "gap-2", "text-xs", "text-[var(--muted)]")}>
                        <Link<Route>
                            to={Route::CategoryDetail { category: article.category.clone() }}
                            classes={classes!(
                                "font-semibold",
                                "text-[var(--primary)]",
                                "hover:underline",
                                "no-underline"
                            )}
                        >
                            { &article.category }
                        </Link<Route>>
                        <span class="text-[var(--border)]">{"·"}</span>
                        <time class="tabular-nums font-mono text-[11px] text-[var(--muted)]">
                            { &article.date }
                        </time>
                    </div>

                    if is_interactive && !has_image {
                        <div
                            class="interactive-card-badge"
                            role="note"
                            aria-label="交互式文章，包含可运行演示"
                            title="交互式文章：进入后可打开独立交互页"
                        >
                            <span class="interactive-card-badge__icon" aria-hidden="true">
                                <span class="interactive-card-badge__core"></span>
                            </span>
                            <span class="interactive-card-badge__copy">
                                <span class="interactive-card-badge__eyebrow">{ "Interactive" }</span>
                                <span class="interactive-card-badge__label">{ "可运行演示" }</span>
                            </span>
                        </div>
                    }
                </div>

                // Title
                <h3 class={classes!("m-0", "leading-snug")}>
                    <a
                        href={detail_href}
                        class={classes!(
                            "text-lg",
                            "sm:text-xl",
                            "font-bold",
                            "tracking-[-0.02em]",
                            "text-[var(--text)]",
                            "no-underline",
                            "transition-colors",
                            "duration-150",
                            "group-hover:text-[var(--primary)]",
                            "line-clamp-2"
                        )}
                        onclick={handle_title_click}
                    >
                        { &article.title }
                        if is_interactive && has_image {
                            <span class="sr-only">{ "（交互式文章，包含可运行演示）" }</span>
                        }
                    </a>
                </h3>

                // Summary
                <p class={classes!(
                    "m-0",
                    "text-sm",
                    "leading-relaxed",
                    "text-[var(--muted)]",
                    "line-clamp-3",
                    "flex-1"
                )}>
                    { &article.summary }
                </p>

                // Footer: Tags + Author
                <div class={classes!(
                    "mt-auto", "pt-3",
                    "border-t", "border-[var(--border)]",
                    "flex", "items-center", "justify-between", "gap-2", "flex-wrap"
                )}>
                    <ul class={classes!("m-0", "flex", "list-none", "flex-wrap", "gap-1.5", "p-0")}>
                        { for article.tags.iter().take(3).map(|tag| {
                            let tag_route = Route::TagDetail { tag: tag.clone() };
                            let tag_label = format!("#{}", tag);
                            html! {
                                <li class="m-0">
                                    <Link<Route>
                                        to={tag_route}
                                        classes="sf-chip"
                                    >
                                        { tag_label }
                                    </Link<Route>>
                                </li>
                            }
                        }) }
                    </ul>

                    <span class="text-xs text-[var(--muted)] shrink-0">
                        { &article.author }
                    </span>
                </div>
            </div>
        </article>
    }
}
