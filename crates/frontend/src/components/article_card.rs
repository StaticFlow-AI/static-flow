use yew::prelude::*;
use yew_router::prelude::{use_navigator, Link, Routable};

use crate::{
    components::image_with_loading::ImageWithLoading,
    models::{ArticleKind, ArticleListItem},
    router::Route,
    utils::image_url,
};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ArticleCardVariant {
    #[default]
    Grid,
    Row,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ArticleCardProps {
    pub article: ArticleListItem,
    #[prop_or_default]
    pub variant: ArticleCardVariant,
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
    let featured_image = article
        .featured_image
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

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

    let interactive_chip = if is_interactive {
        html! {
            <span
                class={classes!(
                    "sf-chip",
                    "!bg-[var(--primary-soft)]",
                    "!text-[var(--primary)]",
                    "!border-transparent",
                    "font-mono",
                    "!text-[11px]",
                    "tracking-[0.02em]"
                )}
                role="note"
                aria-label="交互式文章，包含可运行演示"
                title="交互式文章：进入后可打开独立交互页"
            >
                { "◆ Interactive" }
            </span>
        }
    } else {
        Html::default()
    };

    let render_tags = || {
        if article.tags.is_empty() {
            return Html::default();
        }
        html! {
            <ul class={classes!("m-0", "p-0", "list-none", "flex", "flex-wrap", "items-center", "gap-x-3", "gap-y-1")}>
                { for article.tags.iter().take(3).map(|tag| {
                    let tag_route = Route::TagDetail { tag: tag.clone() };
                    let tag_label = format!("#{tag}");
                    html! {
                        <li class="m-0">
                            <Link<Route>
                                to={tag_route}
                                classes={classes!(
                                    "font-mono",
                                    "text-xs",
                                    "text-[var(--faint)]",
                                    "hover:text-[var(--primary)]",
                                    "no-underline",
                                    "transition-colors",
                                    "duration-150"
                                )}
                            >
                                { tag_label }
                            </Link<Route>>
                        </li>
                    }
                }) }
            </ul>
        }
    };

    match props.variant {
        ArticleCardVariant::Grid => html! {
            <article class={classes!("flex", "flex-col", "gap-3", "h-full", "bg-transparent", "group")}>
                {
                    if let Some(image) = featured_image.as_deref() {
                        let image_url_val = image_url(image);
                        let title = article.title.clone();
                        html! {
                            <a
                                href={detail_href.clone()}
                                class={classes!(
                                    "block",
                                    "aspect-[16/10]",
                                    "rounded-[10px]",
                                    "overflow-hidden",
                                    "border",
                                    "border-[var(--border)]",
                                    "bg-[var(--surface-alt)]",
                                    "hover:border-[var(--border-strong)]",
                                    "transition-colors",
                                    "duration-150"
                                )}
                                onclick={handle_image_click}
                            >
                                <ImageWithLoading
                                    src={image_url_val}
                                    alt={title}
                                    loading={Some(AttrValue::from("lazy"))}
                                    decoding={Some(AttrValue::from("async"))}
                                    class={classes!("w-full", "h-full", "object-cover")}
                                    container_class={classes!("w-full", "h-full")}
                                />
                            </a>
                        }
                    } else {
                        Html::default()
                    }
                }

                <div class={classes!("flex", "items-center", "justify-between", "gap-2", "flex-wrap")}>
                    <div class={classes!("flex", "items-center", "gap-2", "min-w-0")}>
                        if !article.category.trim().is_empty() {
                            <Link<Route>
                                to={Route::CategoryDetail { category: article.category.clone() }}
                                classes={classes!(
                                    "text-xs",
                                    "font-semibold",
                                    "uppercase",
                                    "tracking-[0.08em]",
                                    "text-[var(--muted)]",
                                    "hover:text-[var(--primary)]",
                                    "no-underline",
                                    "transition-colors",
                                    "duration-150"
                                )}
                            >
                                { &article.category }
                            </Link<Route>>
                            <span class="text-[var(--faint)] text-xs" aria-hidden="true">{ "·" }</span>
                        }
                        <time
                            class="font-mono text-xs tabular-nums text-[var(--faint)]"
                            datetime={article.date.clone()}
                        >
                            { &article.date }
                        </time>
                    </div>

                    { interactive_chip }
                </div>

                <h3 class={classes!(
                    "m-0",
                    "font-serif",
                    "font-normal",
                    "text-[1.375rem]",
                    "leading-[1.25]",
                    "tracking-[-0.015em]"
                )}>
                    <a
                        href={detail_href}
                        class={classes!(
                            "text-[var(--text)]",
                            "no-underline",
                            "group-hover:underline",
                            "decoration-[var(--primary)]",
                            "underline-offset-4",
                            "decoration-1",
                            "line-clamp-2"
                        )}
                        onclick={handle_title_click}
                    >
                        { &article.title }
                    </a>
                </h3>

                if !article.summary.trim().is_empty() {
                    <p class={classes!(
                        "m-0",
                        "text-[0.9375rem]",
                        "leading-[1.6]",
                        "text-[var(--muted)]",
                        "line-clamp-3"
                    )}>
                        { &article.summary }
                    </p>
                }

                if !article.tags.is_empty() {
                    <div class="mt-auto pt-1">
                        { render_tags() }
                    </div>
                }
            </article>
        },
        ArticleCardVariant::Row => html! {
            <article class={classes!(
                "grid",
                "grid-cols-1",
                "md:grid-cols-[130px_1fr_180px]",
                "gap-3",
                "md:gap-6",
                "py-7",
                "border-b",
                "border-[var(--border)]",
                "items-start",
                "bg-transparent",
                "group"
            )}>
                <div class={classes!("flex", "md:flex-col", "items-center", "md:items-start", "gap-2", "md:gap-1.5", "md:pt-1")}>
                    <time
                        class="font-mono text-xs tabular-nums text-[var(--faint)]"
                        datetime={article.date.clone()}
                    >
                        { &article.date }
                    </time>
                    if !article.category.trim().is_empty() {
                        <span class="md:hidden text-[var(--faint)] text-xs" aria-hidden="true">{ "·" }</span>
                        <Link<Route>
                            to={Route::CategoryDetail { category: article.category.clone() }}
                            classes={classes!(
                                "text-[11px]",
                                "font-semibold",
                                "uppercase",
                                "tracking-[0.08em]",
                                "text-[var(--muted)]",
                                "hover:text-[var(--primary)]",
                                "no-underline",
                                "transition-colors",
                                "duration-150"
                            )}
                        >
                            { &article.category }
                        </Link<Route>>
                    }
                </div>

                <div class={classes!("min-w-0", "flex", "flex-col", "gap-2")}>
                    if is_interactive {
                        <div class="flex items-center">
                            { interactive_chip }
                        </div>
                    }

                    <h3 class={classes!(
                        "m-0",
                        "font-serif",
                        "font-normal",
                        "text-[1.3rem]",
                        "sm:text-[1.375rem]",
                        "leading-[1.26]",
                        "tracking-[-0.015em]"
                    )}>
                        <a
                            href={detail_href.clone()}
                            class={classes!(
                                "text-[var(--text)]",
                                "no-underline",
                                "group-hover:underline",
                                "decoration-[var(--primary)]",
                                "underline-offset-4",
                                "decoration-1"
                            )}
                            onclick={handle_title_click}
                        >
                            { &article.title }
                        </a>
                    </h3>

                    if !article.summary.trim().is_empty() {
                        <p class={classes!(
                            "m-0",
                            "text-[0.9375rem]",
                            "leading-[1.6]",
                            "text-[var(--muted)]",
                            "line-clamp-2"
                        )}>
                            { &article.summary }
                        </p>
                    }

                    if !article.tags.is_empty() {
                        <div class="pt-1">
                            { render_tags() }
                        </div>
                    }
                </div>

                {
                    if let Some(image) = featured_image.as_deref() {
                        let image_url_val = image_url(image);
                        let title = article.title.clone();
                        html! {
                            <a
                                href={detail_href}
                                class={classes!(
                                    "block",
                                    "aspect-[4/3]",
                                    "w-full",
                                    "max-w-[220px]",
                                    "md:max-w-none",
                                    "rounded-[8px]",
                                    "overflow-hidden",
                                    "border",
                                    "border-[var(--border)]",
                                    "bg-[var(--surface-alt)]",
                                    "hover:border-[var(--border-strong)]",
                                    "transition-colors",
                                    "duration-150"
                                )}
                                onclick={handle_image_click}
                            >
                                <ImageWithLoading
                                    src={image_url_val}
                                    alt={title}
                                    loading={Some(AttrValue::from("lazy"))}
                                    decoding={Some(AttrValue::from("async"))}
                                    class={classes!("w-full", "h-full", "object-cover")}
                                    container_class={classes!("w-full", "h-full")}
                                />
                            </a>
                        }
                    } else {
                        html! { <div class="hidden md:block" aria-hidden="true" /> }
                    }
                }
            </article>
        },
    }
}
