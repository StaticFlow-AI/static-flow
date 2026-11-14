use gloo_timers::{callback::Timeout, future::TimeoutFuture};
use static_flow_shared::{Article, ArticleKind, ArticleListItem};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    window, Element, HtmlImageElement, HtmlSelectElement, HtmlTextAreaElement, KeyboardEvent, Node,
    TouchEvent,
};
use yew::{prelude::*, virtual_dom::AttrValue};
use yew_router::prelude::{use_navigator, use_route, Link};

use crate::{
    api::{
        fetch_all_articles, fetch_article_comment_stats, fetch_article_comments,
        fetch_article_view_trend, fetch_related_articles, submit_article_comment,
        track_article_view, ArticleComment, ArticleViewPoint, SubmitCommentRequest,
    },
    components::{
        article_card::ArticleCard,
        icons::IconName,
        image_with_loading::ImageWithLoading,
        loading_spinner::{LoadingSpinner, SpinnerSize},
        raw_html::RawHtml,
        reading_progress::ReadingProgress,
        scroll_to_top_button::ScrollToTopButton,
        skeleton::SkeletonArticle,
        toast::use_toast,
        toc_button::TocButton,
        tooltip::{TooltipIconButton, TooltipPosition},
        view_trend_chart::ViewTrendChart,
    },
    i18n::{current::article_detail_page as t, fill_one},
    router::Route,
    seo,
    utils::{image_url, markdown_for_external_export, markdown_to_html},
};

#[derive(Properties, Clone, PartialEq)]
pub struct ArticleDetailProps {
    #[prop_or_default]
    pub id: String,
}

type ImageClickListener =
    (web_sys::Element, wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>);

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArticleContentLanguage {
    Zh,
    En,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TrendGranularity {
    Day,
    Hour,
}

const LIGHTBOX_MIN_ZOOM: f64 = 0.5;
const LIGHTBOX_MAX_ZOOM: f64 = 3.0;
const LIGHTBOX_ZOOM_STEP: f64 = 0.25;
const COMMENT_SECTION_ID: &str = "article-comments-section";

#[derive(Clone, PartialEq)]
struct SelectionCommentDraft {
    selected_text: String,
    anchor_block_id: Option<String>,
    anchor_context_before: Option<String>,
    anchor_context_after: Option<String>,
}

#[derive(Clone, PartialEq)]
struct CommentReplyDraft {
    comment_id: String,
    author_name: String,
    comment_text: String,
    ai_reply_markdown: Option<String>,
}

fn normalize_excerpt(value: &str, max_chars: usize) -> Option<String> {
    let compact = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if compact.is_empty() {
        return None;
    }
    Some(compact.chars().take(max_chars).collect::<String>())
}

const INTERACTIVE_CALLOUT_HINT_ZH: &str = "本文的图示、滑杆与演示可直接操作，交互版阅读更完整。";
const INTERACTIVE_CALLOUT_HINT_EN: &str =
    "Diagrams, sliders and demos here are live — best read in the interactive view.";
const INTERACTIVE_CALLOUT_CTA_ZH: &str = "进入交互版";
const INTERACTIVE_CALLOUT_CTA_EN: &str = "Open interactive";
const INTERACTIVE_CALLOUT_DISMISS_ZH: &str = "隐藏提示";
const INTERACTIVE_CALLOUT_DISMISS_EN: &str = "Dismiss";

fn interactive_page_url(page_id: &str, language: ArticleContentLanguage) -> String {
    let lang = match language {
        ArticleContentLanguage::Zh => "zh",
        ArticleContentLanguage::En => "en",
    };
    crate::config::route_path(&format!("/interactive-pages/{page_id}?lang={lang}"))
}

fn extract_anchor_context(
    block_text: Option<String>,
    selected_text: &str,
) -> (Option<String>, Option<String>) {
    let Some(block_text) = block_text else {
        return (None, None);
    };
    let selected = selected_text.trim();
    if selected.is_empty() {
        return (None, None);
    }

    if let Some(found_at) = block_text.find(selected) {
        let before = block_text[..found_at]
            .chars()
            .rev()
            .take(120)
            .collect::<String>();
        let before = before.chars().rev().collect::<String>();
        let after_start = found_at.saturating_add(selected.len());
        let after = block_text[after_start..]
            .chars()
            .take(120)
            .collect::<String>();
        return (normalize_excerpt(&before, 120), normalize_excerpt(&after, 120));
    }

    (None, None)
}

fn node_in_article(node: &Node, article_root: &Element) -> bool {
    let mut cursor = if node.node_type() == Node::ELEMENT_NODE {
        node.clone().dyn_into::<Element>().ok()
    } else {
        node.parent_element()
    };

    while let Some(el) = cursor {
        if el.is_same_node(Some(article_root)) {
            return true;
        }
        cursor = el.parent_element();
    }
    false
}

fn find_anchor_block(
    common_node: &Node,
    article_root: &Element,
) -> (Option<String>, Option<String>) {
    let mut cursor = if common_node.node_type() == Node::ELEMENT_NODE {
        common_node.clone().dyn_into::<Element>().ok()
    } else {
        common_node.parent_element()
    };

    while let Some(el) = cursor {
        if let Some(block_id) = el
            .get_attribute("data-sf-block-id")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return (Some(block_id), el.text_content());
        }
        if el.is_same_node(Some(article_root)) {
            break;
        }
        cursor = el.parent_element();
    }

    (None, None)
}

fn capture_selection_draft() -> Option<(SelectionCommentDraft, (f64, f64))> {
    let win = window()?;
    let selection = win.get_selection().ok().flatten()?;
    let selected_text: String = selection.to_string().into();
    let selected_text = selected_text.trim().to_string();
    if selected_text.chars().count() < 2 {
        return None;
    }

    let range = selection.get_range_at(0).ok()?;
    if range.collapsed() {
        return None;
    }

    let document = win.document()?;
    let article_root = document.query_selector(".article-content").ok().flatten()?;
    let common_node = range.common_ancestor_container().ok()?;
    if !node_in_article(&common_node, &article_root) {
        return None;
    }

    let (anchor_block_id, block_text) = find_anchor_block(&common_node, &article_root);
    let (before, after) = extract_anchor_context(block_text, &selected_text);

    let rect = range.get_bounding_client_rect();
    let viewport_w = win
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(1280.0);
    let viewport_h = win
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(720.0);

    let mut left = rect.x() + (rect.width() / 2.0) - 68.0;
    let mut top = rect.y() - 48.0;
    if !left.is_finite() || !top.is_finite() {
        left = 24.0;
        top = 24.0;
    }

    let max_left = (viewport_w - 136.0).max(12.0);
    let max_top = (viewport_h - 54.0).max(12.0);
    left = left.clamp(12.0, max_left);
    top = top.clamp(12.0, max_top);

    Some((
        SelectionCommentDraft {
            selected_text,
            anchor_block_id,
            anchor_context_before: before,
            anchor_context_after: after,
        },
        (left, top),
    ))
}

fn scroll_to_element_id(id: &str) {
    if let Some(target) = window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id(id))
    {
        target.scroll_into_view();
    }
}

fn scroll_to_anchor_block(block_id: &str) {
    let target = window().and_then(|win| win.document()).and_then(|doc| {
        let selector =
            format!(".article-content [data-sf-block-id=\"{}\"]", block_id.replace('"', "\\\""));
        doc.query_selector(&selector).ok().flatten()
    });

    let Some(target) = target else {
        return;
    };

    target.scroll_into_view();
    let _ = target.class_list().add_1("sf-comment-anchor-flash");
    let target_clone = target.clone();
    Timeout::new(1600, move || {
        let _ = target_clone
            .class_list()
            .remove_1("sf-comment-anchor-flash");
    })
    .forget();
}

fn scroll_to_comment_card(comment_id: &str) {
    let target = window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id(&format!("comment-item-{}", comment_id)));

    let Some(target) = target else {
        return;
    };
    target.scroll_into_view();
    let _ = target.class_list().add_1("sf-comment-anchor-flash");
    let target_clone = target.clone();
    Timeout::new(1600, move || {
        let _ = target_clone
            .class_list()
            .remove_1("sf-comment-anchor-flash");
    })
    .forget();
}

fn avatar_hue(seed: &str) -> u32 {
    let mut hash: u32 = 0;
    for byte in seed.bytes() {
        hash = hash.wrapping_mul(16777619) ^ (byte as u32);
    }
    hash % 360
}

fn comment_avatar_initials(author_name: &str, avatar_seed: &str) -> String {
    let trimmed = author_name.trim();
    if let Some(suffix) = trimmed
        .strip_prefix("Reader-")
        .or_else(|| trimmed.strip_prefix("reader-"))
    {
        let value = suffix
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .take(2)
            .collect::<String>();
        if !value.is_empty() {
            return value.to_ascii_uppercase();
        }
    }

    let parts = trimmed
        .split_whitespace()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
        if parts.len() >= 2 {
            let mut value = String::new();
            if let Some(ch) = first.chars().find(|ch| ch.is_alphanumeric()) {
                value.push(ch);
            }
            if let Some(ch) = last.chars().find(|ch| ch.is_alphanumeric()) {
                value.push(ch);
            }
            let value = value.trim().to_string();
            if !value.is_empty() {
                return value.to_uppercase();
            }
        }
    }

    let single = trimmed
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .take(2)
        .collect::<String>();
    if !single.is_empty() {
        return single.to_uppercase();
    }

    let fallback = avatar_seed
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(2)
        .collect::<String>();
    if !fallback.is_empty() {
        return fallback.to_ascii_uppercase();
    }

    "RD".to_string()
}

fn comment_avatar_style(seed: &str) -> String {
    // Warm editorial earth & terracotta hues (12°..46°) so avatars stay calm on
    // parchment/obsidian.
    let warm_hue = 12 + (avatar_hue(seed) % 35);
    let lightness = 36 + (avatar_hue(seed) % 10);
    format!(
        "background-color: hsl({warm_hue} 34% {lightness}%); border: 1px solid \
         var(--border-strong);"
    )
}

fn display_comment_region(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("unknown")
        || trimmed.eq_ignore_ascii_case("lan")
    {
        return "-".to_string();
    }

    let segments = trimmed
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return "-".to_string();
    }

    segments.join("/")
}

fn format_published_time(ts_ms: i64) -> String {
    let value = js_sys::Date::new(&JsValue::from_f64(ts_ms as f64))
        .to_iso_string()
        .as_string()
        .unwrap_or_else(|| ts_ms.to_string());
    value.replace('T', " ").trim_end_matches('Z').to_string()
}

#[function_component(ArticleDetailPage)]
pub fn article_detail_page(props: &ArticleDetailProps) -> Html {
    let route = use_route::<Route>();
    let navigator = use_navigator();

    let article_id = route
        .as_ref()
        .and_then(|r| match r {
            Route::ArticleDetail {
                id,
            } => Some(id.clone()),
            _ => None,
        })
        .unwrap_or_else(|| props.id.clone());

    let article = use_state(|| None::<Article>);
    let loading = use_state(|| true);
    let related_articles = use_state(Vec::<ArticleListItem>::new);
    let related_loading = use_state(|| false);
    let adjacent_articles = use_state(|| (None::<ArticleListItem>, None::<ArticleListItem>));
    let view_total = use_state(|| None::<usize>);
    let view_today = use_state(|| None::<u32>);
    let trend_points = use_state(Vec::<ArticleViewPoint>::new);
    let trend_day_options = use_state(Vec::<String>::new);
    let trend_selected_day = use_state(|| None::<String>);
    let trend_loading = use_state(|| false);
    let trend_error = use_state(|| None::<String>);
    let trend_granularity = use_state(|| TrendGranularity::Day);
    let comments = use_state(Vec::<ArticleComment>::new);
    let comments_total = use_state(|| 0usize);
    let comments_loading = use_state(|| false);
    let comments_error = use_state(|| None::<String>);
    let comments_refresh_key = use_state(|| 0u64);
    let footer_comment_input = use_state(String::new);
    let footer_reply_target = use_state(|| None::<CommentReplyDraft>);
    let footer_submit_loading = use_state(|| false);
    let footer_submit_feedback = use_state(|| None::<(bool, String)>);
    let selection_draft = use_state(|| None::<SelectionCommentDraft>);
    let selection_button_pos = use_state(|| None::<(f64, f64)>);
    let selection_modal_open = use_state(|| false);
    let selection_comment_input = use_state(String::new);
    let selection_submit_loading = use_state(|| false);
    let selection_submit_feedback = use_state(|| None::<(bool, String)>);
    let more_menu_open = use_state(|| false);
    let toast = use_toast();

    // Back to where user came from (non-article route) with robust fallback.
    let handle_back = {
        let navigator = navigator.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            if let Some(context) = crate::navigation_context::peek_context() {
                crate::navigation_context::arm_context_for_return();
                if crate::navigation_context::navigate_spa_to(&context.source_url) {
                    return;
                }
                if let Some(win) = window() {
                    if win.location().set_href(&context.source_url).is_ok() {
                        return;
                    }
                }
            }
            crate::navigation_context::clear_context();

            if let Some(nav) = navigator.as_ref() {
                nav.push(&Route::Posts);
            } else if let Some(win) = window() {
                let _ = win
                    .location()
                    .set_href(&crate::config::route_path("/posts"));
            }
        })
    };

    {
        let article = article.clone();
        let article_id = article_id.clone();
        let loading = loading.clone();
        let view_total = view_total.clone();
        let view_today = view_today.clone();
        let trend_points = trend_points.clone();
        let trend_day_options = trend_day_options.clone();
        let trend_selected_day = trend_selected_day.clone();
        let trend_error = trend_error.clone();
        use_effect_with(article_id.clone(), move |id| {
            let id = id.clone();
            let article = article.clone();
            let loading = loading.clone();
            let view_total = view_total.clone();
            let view_today = view_today.clone();
            let trend_points = trend_points.clone();
            let trend_day_options = trend_day_options.clone();
            let trend_selected_day = trend_selected_day.clone();
            let trend_error = trend_error.clone();
            loading.set(true);
            view_total.set(None);
            view_today.set(None);
            trend_points.set(vec![]);
            trend_day_options.set(vec![]);
            trend_selected_day.set(None);
            trend_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::fetch_article_detail(&id).await {
                    Ok(data) => {
                        let has_article = data.is_some();
                        article.set(data);
                        loading.set(false);
                        if has_article {
                            match track_article_view(&id).await {
                                Ok(metrics) => {
                                    let daily_points = metrics.daily_points.clone();
                                    let mut days = daily_points
                                        .iter()
                                        .map(|item| item.key.clone())
                                        .collect::<Vec<_>>();
                                    days.sort();
                                    days.dedup();
                                    let selected_day = days.last().cloned();

                                    view_total.set(Some(metrics.total_views));
                                    view_today.set(Some(metrics.today_views));
                                    trend_points.set(daily_points);
                                    trend_day_options.set(days);
                                    trend_selected_day.set(selected_day);
                                },
                                Err(e) => {
                                    web_sys::console::warn_1(
                                        &format!("Failed to track article view: {}", e).into(),
                                    );
                                    trend_error.set(Some(e));
                                },
                            }
                        }
                    },
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to fetch article: {}", e).into(),
                        );
                        article.set(None);
                        loading.set(false);
                    },
                }
            });
            || ()
        });
    }

    {
        let related_articles = related_articles.clone();
        let related_loading = related_loading.clone();
        let article_id = article_id.clone();
        use_effect_with(article_id.clone(), move |id| {
            let id = id.clone();
            related_loading.set(true);
            related_articles.set(vec![]);
            let related_articles = related_articles.clone();
            let related_loading = related_loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_related_articles(&id).await {
                    Ok(data) => {
                        related_articles.set(data);
                        related_loading.set(false);
                    },
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to fetch related articles: {}", e).into(),
                        );
                        related_loading.set(false);
                    },
                }
            });
            || ()
        });
    }

    {
        let adjacent_articles = adjacent_articles.clone();
        let article_id = article_id.clone();
        use_effect_with(article_id.clone(), move |id| {
            let id = id.clone();
            let adjacent_articles = adjacent_articles.clone();
            adjacent_articles.set((None, None));
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(all) = fetch_all_articles(None, None).await {
                    if let Some(pos) = all.iter().position(|item| item.id == id) {
                        // `fetch_all_articles` is newest-first: `pos + 1` is older (previous),
                        // `pos - 1` is newer (next).
                        let prev_item = all.get(pos + 1).cloned();
                        let next_item = pos.checked_sub(1).and_then(|idx| all.get(idx).cloned());
                        adjacent_articles.set((prev_item, next_item));
                    }
                }
            });
            || ()
        });
    }

    {
        let article_id = article_id.clone();
        let comments = comments.clone();
        let comments_total = comments_total.clone();
        let comments_loading = comments_loading.clone();
        let comments_error = comments_error.clone();
        let comments_refresh_key = comments_refresh_key.clone();
        let footer_reply_target = footer_reply_target.clone();
        use_effect_with((article_id.clone(), *comments_refresh_key), move |(id, _refresh)| {
            let id = id.clone();
            let comments = comments.clone();
            let comments_total = comments_total.clone();
            let comments_loading = comments_loading.clone();
            let comments_error = comments_error.clone();
            let footer_reply_target = footer_reply_target.clone();
            comments_loading.set(true);
            comments_error.set(None);
            if *_refresh == 0 {
                footer_reply_target.set(None);
            }

            wasm_bindgen_futures::spawn_local(async move {
                let list_result = fetch_article_comments(&id, Some(80)).await;
                let stats_result = fetch_article_comment_stats(&id).await;

                match (list_result, stats_result) {
                    (Ok(list), Ok(stats)) => {
                        comments.set(list.comments);
                        comments_total.set(stats.total);
                        comments_error.set(None);
                    },
                    (Ok(list), Err(err)) => {
                        let list_len = list.comments.len();
                        comments.set(list.comments);
                        comments_total.set(list_len);
                        comments_error.set(Some(format!("Failed to load comment stats: {}", err)));
                    },
                    (Err(err), _) => {
                        comments.set(vec![]);
                        comments_total.set(0);
                        comments_error.set(Some(format!("Failed to load comments: {}", err)));
                    },
                }
                comments_loading.set(false);
            });
            || ()
        });
    }

    let article_data = (*article).clone();
    let content_language = use_state(|| ArticleContentLanguage::Zh);
    let article_content_ref = use_node_ref();
    let is_lightbox_open = use_state(|| false);
    let is_brief_open = use_state(|| false);
    let is_trend_open = use_state(|| false);
    let interactive_callout_dismissed = use_state(|| false);
    let markdown_copied = use_state(|| false);
    let preview_image_url = use_state_eq(|| None::<String>);
    let preview_image_failed = use_state(|| false);
    let preview_zoom = use_state(|| 1.0_f64);
    let switch_to_zh = {
        let content_language = content_language.clone();
        Callback::from(move |_| content_language.set(ArticleContentLanguage::Zh))
    };
    let switch_to_en = {
        let content_language = content_language.clone();
        Callback::from(move |_| content_language.set(ArticleContentLanguage::En))
    };

    {
        let content_language = content_language.clone();
        use_effect_with(article_data.clone(), move |article_opt| {
            if let Some(article) = article_opt {
                let has_zh = !article.content.trim().is_empty();
                let has_en = article
                    .content_en
                    .as_deref()
                    .map(|value| !value.trim().is_empty())
                    .unwrap_or(false);
                let next_language = if has_zh {
                    ArticleContentLanguage::Zh
                } else if has_en {
                    ArticleContentLanguage::En
                } else {
                    ArticleContentLanguage::Zh
                };
                if *content_language != next_language {
                    content_language.set(next_language);
                }
            }
            || ()
        });
    }

    {
        let article_data = article_data.clone();
        let article_id = article_id.clone();
        use_effect_with(
            (article_data.clone(), article_id.clone(), *content_language),
            move |(article_opt, id, lang)| {
                if let Some(article) = article_opt.as_ref() {
                    let has_en = article
                        .content_en
                        .as_deref()
                        .map(|value| !value.trim().is_empty())
                        .unwrap_or(false);
                    let preferred_lang =
                        if *lang == ArticleContentLanguage::En && has_en { "en" } else { "zh" };
                    seo::apply_article_seo(article, id, preferred_lang);
                }
                || ()
            },
        );
    }

    {
        let is_brief_open = is_brief_open.clone();
        let is_trend_open = is_trend_open.clone();
        let interactive_callout_dismissed = interactive_callout_dismissed.clone();
        let trend_granularity = trend_granularity.clone();
        let footer_comment_input = footer_comment_input.clone();
        let footer_submit_feedback = footer_submit_feedback.clone();
        let selection_draft = selection_draft.clone();
        let selection_button_pos = selection_button_pos.clone();
        let selection_modal_open = selection_modal_open.clone();
        let selection_comment_input = selection_comment_input.clone();
        let selection_submit_feedback = selection_submit_feedback.clone();
        use_effect_with(article_id.clone(), move |_| {
            is_brief_open.set(false);
            is_trend_open.set(false);
            interactive_callout_dismissed.set(false);
            trend_granularity.set(TrendGranularity::Day);
            footer_comment_input.set(String::new());
            footer_submit_feedback.set(None);
            selection_draft.set(None);
            selection_button_pos.set(None);
            selection_modal_open.set(false);
            selection_comment_input.set(String::new());
            selection_submit_feedback.set(None);
            || ()
        });
    }

    let open_image_preview = {
        let is_lightbox_open = is_lightbox_open.clone();
        let preview_image_url = preview_image_url.clone();
        let preview_image_failed = preview_image_failed.clone();
        let preview_zoom = preview_zoom.clone();
        Callback::from(move |src: String| {
            preview_image_failed.set(false);
            preview_image_url.set(Some(src));
            preview_zoom.set(1.0);
            is_lightbox_open.set(true);
        })
    };

    let open_brief_click = {
        let is_brief_open = is_brief_open.clone();
        Callback::from(move |_| is_brief_open.set(true))
    };

    let close_brief_click = {
        let is_brief_open = is_brief_open.clone();
        Callback::from(move |_| is_brief_open.set(false))
    };

    let open_trend_click = {
        let is_trend_open = is_trend_open.clone();
        Callback::from(move |_| is_trend_open.set(true))
    };

    let close_trend_click = {
        let is_trend_open = is_trend_open.clone();
        Callback::from(move |_| is_trend_open.set(false))
    };

    let switch_trend_to_day = {
        let trend_granularity = trend_granularity.clone();
        Callback::from(move |_| trend_granularity.set(TrendGranularity::Day))
    };

    let switch_trend_to_hour = {
        let trend_granularity = trend_granularity.clone();
        Callback::from(move |_| trend_granularity.set(TrendGranularity::Hour))
    };

    let on_trend_day_change = {
        let trend_selected_day = trend_selected_day.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                trend_selected_day.set(Some(target.value()));
            }
        })
    };

    let close_lightbox_click = {
        let is_lightbox_open = is_lightbox_open.clone();
        let preview_image_url = preview_image_url.clone();
        let preview_image_failed = preview_image_failed.clone();
        let preview_zoom = preview_zoom.clone();
        Callback::from(move |_| {
            is_lightbox_open.set(false);
            preview_image_url.set(None);
            preview_image_failed.set(false);
            preview_zoom.set(1.0);
        })
    };

    let on_article_mouseup = {
        let selection_draft = selection_draft.clone();
        let selection_button_pos = selection_button_pos.clone();
        let selection_submit_feedback = selection_submit_feedback.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some((draft, pos)) = capture_selection_draft() {
                selection_draft.set(Some(draft));
                selection_button_pos.set(Some(pos));
                selection_submit_feedback.set(None);
            } else {
                selection_draft.set(None);
                selection_button_pos.set(None);
            }
        })
    };

    let open_selection_modal_click = {
        let selection_modal_open = selection_modal_open.clone();
        let selection_submit_feedback = selection_submit_feedback.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            selection_submit_feedback.set(None);
            selection_modal_open.set(true);
        })
    };

    let close_selection_modal_click = {
        let selection_modal_open = selection_modal_open.clone();
        Callback::from(move |_| {
            selection_modal_open.set(false);
        })
    };

    let on_selection_comment_input = {
        let selection_comment_input = selection_comment_input.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlTextAreaElement>() {
                selection_comment_input.set(target.value());
            }
        })
    };

    let on_footer_comment_input = {
        let footer_comment_input = footer_comment_input.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlTextAreaElement>() {
                footer_comment_input.set(target.value());
            }
        })
    };

    let clear_footer_reply_target = {
        let footer_reply_target = footer_reply_target.clone();
        Callback::from(move |_| {
            footer_reply_target.set(None);
        })
    };

    let submit_selection_comment_click = {
        let article_id = article_id.clone();
        let selection_draft = selection_draft.clone();
        let selection_comment_input = selection_comment_input.clone();
        let selection_submit_loading = selection_submit_loading.clone();
        let selection_submit_feedback = selection_submit_feedback.clone();
        let selection_modal_open = selection_modal_open.clone();
        let selection_button_pos = selection_button_pos.clone();
        let comments_refresh_key = comments_refresh_key.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            let Some(draft) = (*selection_draft).clone() else {
                selection_submit_feedback
                    .set(Some((false, "请先选中文章中的一段内容。".to_string())));
                return;
            };

            let comment_text = selection_comment_input.trim().to_string();
            if comment_text.is_empty() {
                selection_submit_feedback
                    .set(Some((false, "请输入你对选中内容的疑问或评价。".to_string())));
                return;
            }

            let request = SubmitCommentRequest {
                article_id: article_id.clone(),
                entry_type: "selection".to_string(),
                comment_text,
                selected_text: Some(draft.selected_text.clone()),
                anchor_block_id: draft.anchor_block_id.clone(),
                anchor_context_before: draft.anchor_context_before.clone(),
                anchor_context_after: draft.anchor_context_after.clone(),
                reply_to_comment_id: None,
                client_meta: None,
            };

            let selection_comment_input = selection_comment_input.clone();
            let selection_submit_loading = selection_submit_loading.clone();
            let selection_submit_feedback = selection_submit_feedback.clone();
            let selection_modal_open = selection_modal_open.clone();
            let selection_draft = selection_draft.clone();
            let selection_button_pos = selection_button_pos.clone();
            let comments_refresh_key = comments_refresh_key.clone();
            let toast = toast.clone();

            selection_submit_loading.set(true);
            selection_submit_feedback.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match submit_article_comment(request).await {
                    Ok(resp) => {
                        // The selection modal closes on success, so the
                        // confirmation must outlive it as a toast.
                        toast.success(format!("评论已提交，等待审核（任务 {}）。", resp.task_id));
                        selection_submit_feedback.set(None);
                        selection_comment_input.set(String::new());
                        selection_modal_open.set(false);
                        selection_draft.set(None);
                        selection_button_pos.set(None);
                        comments_refresh_key.set((*comments_refresh_key).saturating_add(1));
                    },
                    Err(err) => {
                        selection_submit_feedback.set(Some((false, format!("提交失败：{}", err))));
                    },
                }
                selection_submit_loading.set(false);
            });
        })
    };

    let submit_footer_comment_click = {
        let article_id = article_id.clone();
        let footer_comment_input = footer_comment_input.clone();
        let footer_reply_target = footer_reply_target.clone();
        let footer_submit_loading = footer_submit_loading.clone();
        let footer_submit_feedback = footer_submit_feedback.clone();
        let comments_refresh_key = comments_refresh_key.clone();
        Callback::from(move |_| {
            let comment_text = footer_comment_input.trim().to_string();
            if comment_text.is_empty() {
                footer_submit_feedback.set(Some((false, "请输入评论内容。".to_string())));
                return;
            }

            let request = SubmitCommentRequest {
                article_id: article_id.clone(),
                entry_type: "footer".to_string(),
                comment_text,
                selected_text: None,
                anchor_block_id: None,
                anchor_context_before: None,
                anchor_context_after: None,
                reply_to_comment_id: footer_reply_target
                    .as_ref()
                    .as_ref()
                    .map(|target| target.comment_id.clone()),
                client_meta: None,
            };

            let footer_comment_input = footer_comment_input.clone();
            let footer_reply_target = footer_reply_target.clone();
            let footer_submit_loading = footer_submit_loading.clone();
            let footer_submit_feedback = footer_submit_feedback.clone();
            let comments_refresh_key = comments_refresh_key.clone();
            footer_submit_loading.set(true);
            footer_submit_feedback.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match submit_article_comment(request).await {
                    Ok(resp) => {
                        footer_submit_feedback.set(Some((
                            true,
                            format!("评论已提交，等待审核（任务 {}）。", resp.task_id),
                        )));
                        footer_comment_input.set(String::new());
                        footer_reply_target.set(None);
                        comments_refresh_key.set((*comments_refresh_key).saturating_add(1));
                    },
                    Err(err) => {
                        footer_submit_feedback.set(Some((false, format!("提交失败：{}", err))));
                    },
                }
                footer_submit_loading.set(false);
            });
        })
    };

    let jump_to_comments_click = Callback::from(move |_| {
        scroll_to_element_id(COMMENT_SECTION_ID);
    });

    let refresh_comments_click = {
        let comments_refresh_key = comments_refresh_key.clone();
        Callback::from(move |_| {
            comments_refresh_key.set((*comments_refresh_key).saturating_add(1));
        })
    };

    {
        let article_id = article_id.clone();
        let is_trend_open = is_trend_open.clone();
        let trend_granularity = trend_granularity.clone();
        let trend_selected_day = trend_selected_day.clone();
        let trend_points = trend_points.clone();
        let trend_loading = trend_loading.clone();
        let trend_error = trend_error.clone();
        let view_total = view_total.clone();
        let trend_day_options = trend_day_options.clone();
        use_effect_with(
            (article_id.clone(), *is_trend_open, *trend_granularity, (*trend_selected_day).clone()),
            move |(id, is_open, granularity, selected_day)| {
                if *is_open {
                    let article_id = id.clone();
                    let trend_points = trend_points.clone();
                    let trend_loading = trend_loading.clone();
                    let trend_error = trend_error.clone();
                    let view_total = view_total.clone();
                    let trend_selected_day = trend_selected_day.clone();
                    let trend_day_options = trend_day_options.clone();
                    let selected_day = selected_day.clone();
                    let granularity = *granularity;

                    trend_loading.set(true);
                    trend_error.set(None);

                    wasm_bindgen_futures::spawn_local(async move {
                        let response = match granularity {
                            TrendGranularity::Day => {
                                fetch_article_view_trend(&article_id, "day", None, None).await
                            },
                            TrendGranularity::Hour => {
                                let day = selected_day.unwrap_or_default();
                                if day.trim().is_empty() {
                                    trend_loading.set(false);
                                    trend_error.set(Some("missing trend day".to_string()));
                                    return;
                                }
                                fetch_article_view_trend(
                                    &article_id,
                                    "hour",
                                    None,
                                    Some(day.as_str()),
                                )
                                .await
                            },
                        };

                        match response {
                            Ok(data) => {
                                trend_points.set(data.points.clone());
                                view_total.set(Some(data.total_views));
                                trend_loading.set(false);

                                if data.granularity == "day" {
                                    let mut days = data
                                        .points
                                        .iter()
                                        .map(|item| item.key.clone())
                                        .collect::<Vec<_>>();
                                    days.sort();
                                    days.dedup();
                                    let selected = days.last().cloned();
                                    trend_day_options.set(days);
                                    if selected.is_some() {
                                        trend_selected_day.set(selected);
                                    }
                                }
                            },
                            Err(error) => {
                                trend_loading.set(false);
                                trend_error.set(Some(error));
                            },
                        }
                    });
                }

                || ()
            },
        );
    }

    {
        let is_lightbox_open = is_lightbox_open.clone();
        let preview_image_url = preview_image_url.clone();
        let preview_image_failed = preview_image_failed.clone();
        let preview_zoom = preview_zoom.clone();
        use_effect_with(*is_lightbox_open, move |is_open| {
            let keydown_listener_opt = if *is_open {
                let handle = is_lightbox_open.clone();
                let preview_url = preview_image_url.clone();
                let failed = preview_image_failed.clone();
                let zoom = preview_zoom.clone();
                let listener =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |event: KeyboardEvent| {
                        match event.key().as_str() {
                            "Escape" => {
                                handle.set(false);
                                preview_url.set(None);
                                failed.set(false);
                                zoom.set(1.0);
                            },
                            "+" | "=" => {
                                zoom.set((*zoom + LIGHTBOX_ZOOM_STEP).min(LIGHTBOX_MAX_ZOOM));
                            },
                            "-" | "_" => {
                                zoom.set((*zoom - LIGHTBOX_ZOOM_STEP).max(LIGHTBOX_MIN_ZOOM));
                            },
                            "0" => {
                                zoom.set(1.0);
                            },
                            _ => {},
                        }
                    })
                        as Box<dyn FnMut(_)>);

                if let Some(win) = window() {
                    let _ = win.add_event_listener_with_callback(
                        "keydown",
                        listener.as_ref().unchecked_ref(),
                    );
                }
                Some(listener)
            } else {
                None
            };

            move || {
                if let Some(listener) = keydown_listener_opt {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "keydown",
                            listener.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }

    {
        let is_brief_open = is_brief_open.clone();
        use_effect_with(*is_brief_open, move |is_open| {
            let keydown_listener_opt = if *is_open {
                let handle = is_brief_open.clone();
                let listener =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |event: KeyboardEvent| {
                        if event.key() == "Escape" {
                            handle.set(false);
                        }
                    })
                        as Box<dyn FnMut(_)>);

                if let Some(win) = window() {
                    let _ = win.add_event_listener_with_callback(
                        "keydown",
                        listener.as_ref().unchecked_ref(),
                    );
                }
                Some(listener)
            } else {
                None
            };

            move || {
                if let Some(listener) = keydown_listener_opt {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "keydown",
                            listener.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }

    {
        let is_trend_open = is_trend_open.clone();
        use_effect_with(*is_trend_open, move |is_open| {
            let keydown_listener_opt = if *is_open {
                let handle = is_trend_open.clone();
                let listener =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |event: KeyboardEvent| {
                        if event.key() == "Escape" {
                            handle.set(false);
                        }
                    })
                        as Box<dyn FnMut(_)>);

                if let Some(win) = window() {
                    let _ = win.add_event_listener_with_callback(
                        "keydown",
                        listener.as_ref().unchecked_ref(),
                    );
                }
                Some(listener)
            } else {
                None
            };

            move || {
                if let Some(listener) = keydown_listener_opt {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "keydown",
                            listener.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }

    {
        let selection_modal_open = selection_modal_open.clone();
        use_effect_with(*selection_modal_open, move |is_open| {
            let keydown_listener_opt = if *is_open {
                let handle = selection_modal_open.clone();
                let listener =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |event: KeyboardEvent| {
                        if event.key() == "Escape" {
                            handle.set(false);
                        }
                    })
                        as Box<dyn FnMut(_)>);

                if let Some(win) = window() {
                    let _ = win.add_event_listener_with_callback(
                        "keydown",
                        listener.as_ref().unchecked_ref(),
                    );
                }
                Some(listener)
            } else {
                None
            };

            move || {
                if let Some(listener) = keydown_listener_opt {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "keydown",
                            listener.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }

    let stop_lightbox_bubble = Callback::from(|event: MouseEvent| event.stop_propagation());
    let stop_brief_bubble = Callback::from(|event: MouseEvent| event.stop_propagation());
    let stop_trend_bubble = Callback::from(|event: MouseEvent| event.stop_propagation());
    let stop_selection_modal_bubble = Callback::from(|event: MouseEvent| event.stop_propagation());
    let mark_preview_failed = {
        let preview_image_failed = preview_image_failed.clone();
        Callback::from(move |_: Event| preview_image_failed.set(true))
    };
    let mark_preview_loaded = {
        let preview_image_failed = preview_image_failed.clone();
        Callback::from(move |_: Event| preview_image_failed.set(false))
    };
    let zoom_in_click = {
        let preview_zoom = preview_zoom.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            preview_zoom.set((*preview_zoom + LIGHTBOX_ZOOM_STEP).min(LIGHTBOX_MAX_ZOOM));
        })
    };
    let zoom_out_click = {
        let preview_zoom = preview_zoom.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            preview_zoom.set((*preview_zoom - LIGHTBOX_ZOOM_STEP).max(LIGHTBOX_MIN_ZOOM));
        })
    };
    let zoom_reset_click = {
        let preview_zoom = preview_zoom.clone();
        Callback::from(move |event: MouseEvent| {
            event.stop_propagation();
            preview_zoom.set(1.0);
        })
    };

    // Touch gestures for the lightbox: two-finger pinch drives the zoom
    // state, a quick double-tap toggles 1x <-> 2x. One-finger panning stays
    // native (the container scrolls; touch-action pan-x pan-y suppresses the
    // browser's own pinch so ours wins).
    let pinch_state = use_mut_ref(|| None::<(f64, f64)>);
    let last_tap = use_mut_ref(|| None::<(f64, i32, i32)>);

    fn touch_distance(event: &TouchEvent) -> Option<f64> {
        let touches = event.touches();
        let (a, b) = (touches.get(0)?, touches.get(1)?);
        let dx = (a.client_x() - b.client_x()) as f64;
        let dy = (a.client_y() - b.client_y()) as f64;
        Some((dx * dx + dy * dy).sqrt())
    }

    let lightbox_touch_start = {
        let pinch_state = pinch_state.clone();
        let last_tap = last_tap.clone();
        let preview_zoom = preview_zoom.clone();
        Callback::from(move |event: TouchEvent| {
            let touches = event.touches();
            if touches.length() == 2 {
                if let Some(distance) = touch_distance(&event) {
                    *pinch_state.borrow_mut() = Some((distance, *preview_zoom));
                }
                return;
            }
            if touches.length() == 1 {
                let Some(touch) = touches.get(0) else {
                    return;
                };
                let now = js_sys::Date::now();
                let (x, y) = (touch.client_x(), touch.client_y());
                let previous_tap = *last_tap.borrow();
                let is_double = previous_tap.is_some_and(|(at, lx, ly)| {
                    now - at < 300.0 && (x - lx).abs() < 30 && (y - ly).abs() < 30
                });
                if is_double {
                    *last_tap.borrow_mut() = None;
                    preview_zoom.set(if (*preview_zoom - 1.0).abs() < 0.01 { 2.0 } else { 1.0 });
                } else {
                    *last_tap.borrow_mut() = Some((now, x, y));
                }
            }
        })
    };
    let lightbox_touch_move = {
        let pinch_state = pinch_state.clone();
        let preview_zoom = preview_zoom.clone();
        Callback::from(move |event: TouchEvent| {
            let state = *pinch_state.borrow();
            if let Some((start_distance, start_zoom)) = state {
                if event.touches().length() == 2 {
                    if let Some(distance) = touch_distance(&event) {
                        if start_distance > 0.0 {
                            let next = (start_zoom * distance / start_distance)
                                .clamp(LIGHTBOX_MIN_ZOOM, LIGHTBOX_MAX_ZOOM);
                            preview_zoom.set(next);
                        }
                    }
                }
            }
        })
    };
    let lightbox_touch_end = {
        let pinch_state = pinch_state.clone();
        Callback::from(move |event: TouchEvent| {
            if event.touches().length() < 2 {
                *pinch_state.borrow_mut() = None;
            }
        })
    };

    let markdown_render_key = if let Some(article) = article_data.as_ref() {
        let lang_key = if *content_language == ArticleContentLanguage::En { "en" } else { "zh" };
        format!("{}:{lang_key}", article.id)
    } else {
        String::new()
    };
    let article_body_html = article_data.as_ref().map(|article| {
        let has_en_content = article
            .content_en
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let active_content = if *content_language == ArticleContentLanguage::En && has_en_content {
            article
                .content_en
                .as_deref()
                .unwrap_or(article.content.as_str())
        } else {
            article.content.as_str()
        };
        AttrValue::from(markdown_to_html(active_content))
    });
    let comments_render_key = (*comments)
        .iter()
        .map(|comment| {
            format!(
                "{}:{}:{}",
                comment.comment_id,
                comment.published_at,
                comment
                    .ai_reply_markdown
                    .as_ref()
                    .map(|value| value.len())
                    .unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join("|");

    {
        let markdown_copied = markdown_copied.clone();
        use_effect_with(markdown_render_key.clone(), move |_| {
            markdown_copied.set(false);
            || ()
        });
    }

    {
        let article_content_ref = article_content_ref.clone();
        let article_body_html = article_body_html.clone();
        use_effect_with(
            (markdown_render_key.clone(), article_body_html.clone()),
            move |(_, html)| {
                if let Some(host) = article_content_ref.cast::<Element>() {
                    host.set_inner_html(html.as_deref().unwrap_or(""));
                }
                || ()
            },
        );
    }

    // Initialize markdown rendering after content/language is loaded
    use_effect_with(markdown_render_key.clone(), |render_key| {
        let timeout = if !render_key.is_empty() {
            Some(Timeout::new(100, move || {
                if let Some(win) = window() {
                    let article_root = win
                        .document()
                        .and_then(|doc| doc.query_selector(".article-content").ok())
                        .flatten();

                    if let Ok(init_fn) =
                        js_sys::Reflect::get(&win, &JsValue::from_str("initMarkdownRendering"))
                    {
                        if let Ok(func) = init_fn.dyn_into::<js_sys::Function>() {
                            if let Some(root) = article_root {
                                let _ = func.call1(&win, root.as_ref());
                            } else {
                                let _ = func.call0(&win);
                            }
                        }
                    }
                }
            }))
        } else {
            None
        };

        move || {
            drop(timeout);

            // Cleanup markdown/TOC/fullscreen state on unmount or content switch
            if let Some(win) = window() {
                if let Ok(cleanup_fn) =
                    js_sys::Reflect::get(&win, &JsValue::from_str("cleanupMarkdownRendering"))
                {
                    if let Ok(func) = cleanup_fn.dyn_into::<js_sys::Function>() {
                        let _ = func.call0(&win);
                    }
                }
            }
        }
    });

    // Re-run markdown enhancements for dynamic comment markdown fragments.
    // We intentionally avoid TOC regeneration here.
    {
        let article_id = article_id.clone();
        let comments_render_key = comments_render_key.clone();
        use_effect_with(
            (article_id.clone(), comments_render_key.clone()),
            move |(id, comments_key)| {
                let timeout = if !id.trim().is_empty() && !comments_key.is_empty() {
                    Some(Timeout::new(120, move || {
                        if let Some(win) = window() {
                            let Some(document) = win.document() else {
                                return;
                            };
                            let init_fn = js_sys::Reflect::get(
                                &win,
                                &JsValue::from_str("initMarkdownFragmentRendering"),
                            )
                            .ok()
                            .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
                            .or_else(|| {
                                js_sys::Reflect::get(
                                    &win,
                                    &JsValue::from_str("initMarkdownRendering"),
                                )
                                .ok()
                                .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
                            });
                            let Some(func) = init_fn else {
                                return;
                            };

                            if let Ok(node_list) =
                                document.query_selector_all(".comment-ai-markdown")
                            {
                                for idx in 0..node_list.length() {
                                    if let Some(node) = node_list.item(idx) {
                                        if let Ok(element) = node.dyn_into::<Element>() {
                                            let _ = func.call1(&win, element.as_ref());
                                        }
                                    }
                                }
                            }
                        }
                    }))
                } else {
                    None
                };

                move || {
                    drop(timeout);
                }
            },
        );
    }

    {
        let open_image_preview = open_image_preview.clone();
        use_effect_with(markdown_render_key.clone(), move |render_key| {
            let mut listeners: Vec<ImageClickListener> = Vec::new();

            if !render_key.is_empty() {
                if let Some(document) = window().and_then(|win| win.document()) {
                    if let Ok(node_list) = document.query_selector_all(".article-content img") {
                        for idx in 0..node_list.length() {
                            if let Some(node) = node_list.item(idx) {
                                if let Ok(element) = node.dyn_into::<web_sys::Element>() {
                                    let callback = open_image_preview.clone();
                                    let listener = wasm_bindgen::closure::Closure::wrap(Box::new(
                                        move |event: web_sys::Event| {
                                            if let Some(target) = event.current_target() {
                                                if let Ok(img) =
                                                    target.dyn_into::<HtmlImageElement>()
                                                {
                                                    if let Some(src) = img.get_attribute("src") {
                                                        callback.emit(src);
                                                    }
                                                }
                                            }
                                        },
                                    )
                                        as Box<dyn FnMut(_)>);

                                    if let Err(err) = element.add_event_listener_with_callback(
                                        "click",
                                        listener.as_ref().unchecked_ref(),
                                    ) {
                                        web_sys::console::error_1(&err);
                                    }

                                    listeners.push((element, listener));
                                }
                            }
                        }
                    }
                }
            }

            move || {
                for (element, listener) in listeners {
                    let _ = element.remove_event_listener_with_callback(
                        "click",
                        listener.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    let loading_view = html! {
        <div class={classes!("mx-auto", "w-full", "max-w-[760px]", "px-6", "py-14")}>
            <SkeletonArticle />
        </div>
    };

    let is_overlay_open =
        *is_lightbox_open || *is_brief_open || *is_trend_open || *selection_modal_open;

    {
        let more_menu_open = more_menu_open.clone();
        use_effect_with(article_id.clone(), move |_| {
            more_menu_open.set(false);
            || ()
        });
    }

    let navigate_to_studio = Callback::from(move |e: MouseEvent| {
        if !e.meta_key() && !e.ctrl_key() && !e.shift_key() && !e.alt_key() && e.button() == 0 {
            e.prevent_default();
            let studio_url = crate::config::route_path("/studio#article");
            if !crate::navigation_context::navigate_spa_to(&studio_url) {
                if let Some(win) = window() {
                    let _ = win.location().set_href(&studio_url);
                }
                return;
            }
            if let Some(win) = window() {
                if let Ok(ev) = web_sys::Event::new("hashchange") {
                    let _ = win.dispatch_event(&ev);
                }
            }
        }
    });

    let body = if *loading {
        loading_view
    } else if let Some(article) = article_data.clone() {
        let has_zh_content = !article.content.trim().is_empty();
        let has_en_content = article
            .content_en
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let show_language_toggle = has_zh_content && has_en_content;
        let active_content = if *content_language == ArticleContentLanguage::En && has_en_content {
            article
                .content_en
                .as_deref()
                .unwrap_or(article.content.as_str())
        } else {
            article.content.as_str()
        };
        let active_detailed_summary = article.detailed_summary.as_ref().and_then(|summary| {
            let preferred = if *content_language == ArticleContentLanguage::En {
                summary.en.as_ref().or(summary.zh.as_ref())
            } else {
                summary.zh.as_ref().or(summary.en.as_ref())
            };
            preferred
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        });
        let has_detailed_summary = active_detailed_summary.is_some();
        let word_count = active_content
            .chars()
            .filter(|c| !c.is_whitespace())
            .count();
        let detailed_summary_html = active_detailed_summary
            .as_ref()
            .map(|summary| AttrValue::from(markdown_to_html(summary)));
        let zh_button_class = if *content_language == ArticleContentLanguage::Zh {
            classes!(
                "rounded-full",
                "bg-[var(--primary)]",
                "px-2.5",
                "py-1.5",
                "font-mono",
                "text-[0.6875rem]",
                "font-medium",
                "uppercase",
                "tracking-[0.06em]",
                "text-[var(--on-primary)]",
                "transition-colors"
            )
        } else {
            classes!(
                "rounded-full",
                "px-2.5",
                "py-1.5",
                "font-mono",
                "text-[0.6875rem]",
                "font-medium",
                "uppercase",
                "tracking-[0.06em]",
                "text-[var(--muted)]",
                "transition-colors",
                "hover:text-[var(--text)]"
            )
        };
        let en_button_class = if *content_language == ArticleContentLanguage::En {
            classes!(
                "rounded-full",
                "bg-[var(--primary)]",
                "px-2.5",
                "py-1.5",
                "font-mono",
                "text-[0.6875rem]",
                "font-medium",
                "uppercase",
                "tracking-[0.06em]",
                "text-[var(--on-primary)]",
                "transition-colors"
            )
        } else {
            classes!(
                "rounded-full",
                "px-2.5",
                "py-1.5",
                "font-mono",
                "text-[0.6875rem]",
                "font-medium",
                "uppercase",
                "tracking-[0.06em]",
                "text-[var(--muted)]",
                "transition-colors",
                "hover:text-[var(--text)]"
            )
        };
        let summary_title = if *content_language == ArticleContentLanguage::En {
            t::DETAILED_SUMMARY_TITLE_EN
        } else {
            t::DETAILED_SUMMARY_TITLE_ZH
        };
        let brief_button_text = if *content_language == ArticleContentLanguage::En {
            t::OPEN_BRIEF_BUTTON_EN
        } else {
            t::OPEN_BRIEF_BUTTON_ZH
        };
        let raw_markdown_button_text = if *content_language == ArticleContentLanguage::En {
            t::OPEN_RAW_MARKDOWN_BUTTON_EN
        } else {
            t::OPEN_RAW_MARKDOWN_BUTTON_ZH
        };
        let interactive_button_text = if *content_language == ArticleContentLanguage::En {
            t::OPEN_INTERACTIVE_BUTTON_EN
        } else {
            t::OPEN_INTERACTIVE_BUTTON_ZH
        };
        let interactive_callout_hint = if *content_language == ArticleContentLanguage::En {
            INTERACTIVE_CALLOUT_HINT_EN
        } else {
            INTERACTIVE_CALLOUT_HINT_ZH
        };
        let interactive_callout_cta = if *content_language == ArticleContentLanguage::En {
            INTERACTIVE_CALLOUT_CTA_EN
        } else {
            INTERACTIVE_CALLOUT_CTA_ZH
        };
        let interactive_callout_dismiss = if *content_language == ArticleContentLanguage::En {
            INTERACTIVE_CALLOUT_DISMISS_EN
        } else {
            INTERACTIVE_CALLOUT_DISMISS_ZH
        };
        let has_interactive_mirror = article
            .interactive_page_id
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let show_interactive_callout = article.article_kind == ArticleKind::InteractiveRepost
            && has_interactive_mirror
            && !*interactive_callout_dismissed;
        let interactive_callout_href = article
            .interactive_page_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|page_id| interactive_page_url(page_id, *content_language));
        let can_export_markdown = !active_content.trim().is_empty();
        let show_article_actions = show_language_toggle
            || has_detailed_summary
            || can_export_markdown
            || has_interactive_mirror;
        let export_button_label = if *markdown_copied {
            if *content_language == ArticleContentLanguage::En {
                "Copied"
            } else {
                "已复制"
            }
        } else if *content_language == ArticleContentLanguage::En {
            "Copy Markdown"
        } else {
            "导出 Markdown"
        };
        let export_button_icon =
            if *markdown_copied { classes!("fas", "fa-check") } else { classes!("far", "fa-copy") };
        let copy_markdown_click = {
            let markdown_copied = markdown_copied.clone();
            let markdown_source = markdown_for_external_export(active_content);
            Callback::from(move |_| {
                let markdown_copied = markdown_copied.clone();
                let markdown_source = markdown_source.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut copied = false;

                    if let Some(win) = window() {
                        let navigator = win.navigator();
                        if let Ok(clipboard) =
                            js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard"))
                        {
                            if !clipboard.is_undefined() && !clipboard.is_null() {
                                if let Ok(write_text) = js_sys::Reflect::get(
                                    &clipboard,
                                    &JsValue::from_str("writeText"),
                                ) {
                                    if let Some(write_fn) = write_text.dyn_ref::<js_sys::Function>()
                                    {
                                        if let Ok(promise_value) = write_fn
                                            .call1(&clipboard, &JsValue::from_str(&markdown_source))
                                        {
                                            if let Ok(promise) =
                                                promise_value.dyn_into::<js_sys::Promise>()
                                            {
                                                copied =
                                                    wasm_bindgen_futures::JsFuture::from(promise)
                                                        .await
                                                        .is_ok();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if copied {
                        markdown_copied.set(true);
                        TimeoutFuture::new(1800).await;
                        markdown_copied.set(false);
                    } else {
                        web_sys::console::warn_1(&JsValue::from_str(
                            "Failed to copy markdown to clipboard.",
                        ));
                    }
                });
            })
        };
        let open_raw_markdown_click = {
            let navigator = navigator.clone();
            let article_id = article.id.clone();
            let lang = if *content_language == ArticleContentLanguage::En {
                "en".to_string()
            } else {
                "zh".to_string()
            };
            Callback::from(move |_| {
                if let Some(nav) = navigator.as_ref() {
                    nav.push(&Route::ArticleRaw {
                        id: article_id.clone(),
                        lang: lang.clone(),
                    });
                }
            })
        };
        let open_interactive_click = {
            let navigator = navigator.clone();
            let article_id = article.id.clone();
            let interactive_page_url = article
                .interactive_page_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|page_id| interactive_page_url(page_id, *content_language));
            Callback::from(move |_| {
                if let Some(url) = interactive_page_url.as_ref() {
                    if let Some(win) = window() {
                        if win.location().set_href(url).is_ok() {
                            return;
                        }
                    }
                }
                if let Some(nav) = navigator.as_ref() {
                    nav.push(&Route::ArticleInteractive {
                        id: article_id.clone(),
                    });
                }
            })
        };
        let dismiss_interactive_callout_click = {
            let interactive_callout_dismissed = interactive_callout_dismissed.clone();
            Callback::from(move |_: MouseEvent| interactive_callout_dismissed.set(true))
        };
        // Article toolbar: primary actions (back on < xl, language, interactive
        // mirror) stay visible; secondary actions are quiet ghost text buttons
        // on >= md and collapse into a "more" menu on phones. Trend/comments
        // only appear here below xl, where the floating left rail is hidden.
        let toolbar_icon_button = classes!(
            "inline-flex",
            "h-10",
            "w-10",
            "shrink-0",
            "items-center",
            "justify-center",
            "rounded-full",
            "text-[var(--muted)]",
            "transition-colors",
            "cursor-pointer",
            "hover:bg-[var(--surface-alt)]",
            "hover:text-[var(--text)]"
        );
        let toolbar_ghost_button = classes!("sf-btn", "sf-btn--ghost", "sf-btn--sm", "!px-2.5");
        let more_menu_item = classes!(
            "flex",
            "w-full",
            "min-h-[2.75rem]",
            "items-center",
            "gap-3",
            "px-4",
            "text-left",
            "text-[0.875rem]",
            "text-[var(--text)]",
            "transition-colors",
            "cursor-pointer",
            "hover:bg-[var(--surface-alt)]"
        );
        let more_label =
            if *content_language == ArticleContentLanguage::En { "More" } else { "更多操作" };
        let comments_label = if *content_language == ArticleContentLanguage::En {
            "Comments"
        } else {
            "定位到评论区"
        };
        let close_more_then = |cb: Callback<MouseEvent>| {
            let more_menu_open = more_menu_open.clone();
            Callback::from(move |e: MouseEvent| {
                more_menu_open.set(false);
                cb.emit(e);
            })
        };
        let toggle_more_menu = {
            let more_menu_open = more_menu_open.clone();
            Callback::from(move |_: MouseEvent| more_menu_open.set(!*more_menu_open))
        };
        let close_more_menu = {
            let more_menu_open = more_menu_open.clone();
            Callback::from(move |_: MouseEvent| more_menu_open.set(false))
        };
        let render_article_actions = || -> Html {
            html! {
                <div
                    class={classes!(
                        "article-toolbar",
                        "mt-6",
                        "flex",
                        "min-w-0",
                        "items-center",
                        "gap-1.5",
                        "sm:gap-2",
                        "border-y",
                        "border-[var(--border)]",
                        "py-2",
                        (!show_article_actions).then_some("xl:hidden")
                    )}
                    role="toolbar"
                    aria-label={t::ARTICLE_META_ARIA}
                >
                    <button
                        type="button"
                        class={classes!(toolbar_icon_button.clone(), "xl:hidden", "-ml-2")}
                        onclick={handle_back.clone()}
                        aria-label={t::BACK_TOOLTIP}
                        title={t::BACK_TOOLTIP}
                    >
                        <i class={classes!("fas", "fa-arrow-left", "text-[14px]")} aria-hidden="true"></i>
                    </button>
                    {
                        if show_language_toggle {
                            html! {
                                <div
                                    class={classes!(
                                        "inline-flex",
                                        "shrink-0",
                                        "items-center",
                                        "gap-0.5",
                                        "rounded-full",
                                        "border",
                                        "border-[var(--border-strong)]",
                                        "bg-[var(--surface)]",
                                        "p-0.5"
                                    )}
                                    role="group"
                                    aria-label={t::LANG_SWITCH_LABEL}
                                >
                                    <button
                                        type="button"
                                        class={zh_button_class.clone()}
                                        aria-pressed={if *content_language == ArticleContentLanguage::Zh {
                                            "true"
                                        } else {
                                            "false"
                                        }}
                                        onclick={switch_to_zh.clone()}
                                    >
                                        { t::LANG_SWITCH_ZH }
                                    </button>
                                    <button
                                        type="button"
                                        class={en_button_class.clone()}
                                        aria-pressed={if *content_language == ArticleContentLanguage::En {
                                            "true"
                                        } else {
                                            "false"
                                        }}
                                        onclick={switch_to_en.clone()}
                                    >
                                        { t::LANG_SWITCH_EN }
                                    </button>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                    {
                        if has_interactive_mirror {
                            html! {
                                <button
                                    type="button"
                                    class={classes!(
                                        "sf-btn",
                                        "sf-btn--sm",
                                        "min-w-0",
                                        "border",
                                        "border-[var(--primary)]/35",
                                        "bg-[var(--primary-soft)]",
                                        "text-[var(--primary)]",
                                        "hover:bg-[var(--primary)]",
                                        "hover:text-[var(--on-primary)]"
                                    )}
                                    onclick={open_interactive_click.clone()}
                                    title={interactive_button_text}
                                >
                                    <span class="font-mono text-[10px]" aria-hidden="true">{ "◆" }</span>
                                    <span class="min-w-0 truncate">{ interactive_button_text }</span>
                                </button>
                            }
                        } else {
                            html! {}
                        }
                    }

                    <div class={classes!(
                        "ml-auto",
                        (!show_language_toggle && !has_interactive_mirror).then_some("xl:ml-0"),
                        "flex",
                        "shrink-0",
                        "items-center",
                        "gap-0.5"
                    )}>
                        // >= md: quiet ghost text buttons
                        <div class={classes!("hidden", "md:flex", "items-center", "gap-0.5")}>
                            {
                                if has_detailed_summary {
                                    html! {
                                        <button
                                            type="button"
                                            class={toolbar_ghost_button.clone()}
                                            onclick={open_brief_click.clone()}
                                        >
                                            <i class={classes!("fas", "fa-list-check", "text-[11px]")} aria-hidden="true"></i>
                                            { brief_button_text }
                                        </button>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            {
                                if can_export_markdown {
                                    html! {
                                        <>
                                            <button
                                                type="button"
                                                class={toolbar_ghost_button.clone()}
                                                onclick={open_raw_markdown_click.clone()}
                                            >
                                                <i class={classes!("far", "fa-file-lines", "text-[11px]")} aria-hidden="true"></i>
                                                { raw_markdown_button_text }
                                            </button>
                                            <button
                                                type="button"
                                                class={classes!(
                                                    toolbar_ghost_button.clone(),
                                                    (*markdown_copied).then_some("!text-[var(--ok)]")
                                                )}
                                                onclick={copy_markdown_click.clone()}
                                                aria-live="polite"
                                            >
                                                <i class={classes!(export_button_icon.clone(), "text-[11px]")} aria-hidden="true"></i>
                                                { export_button_label }
                                            </button>
                                        </>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                        // md..xl: trend + comments (the left rail takes over at xl)
                        <button
                            type="button"
                            class={classes!(toolbar_icon_button.clone(), "hidden", "md:inline-flex", "xl:hidden")}
                            onclick={open_trend_click.clone()}
                            aria-label={t::TREND_TOOLTIP}
                            title={t::TREND_TOOLTIP}
                        >
                            <i class={classes!("fas", "fa-chart-line", "text-[13px]")} aria-hidden="true"></i>
                        </button>
                        <button
                            type="button"
                            class={classes!(toolbar_icon_button.clone(), "hidden", "md:inline-flex", "xl:hidden")}
                            onclick={jump_to_comments_click.clone()}
                            aria-label={comments_label}
                            title={comments_label}
                        >
                            <i class={classes!("far", "fa-comment", "text-[14px]")} aria-hidden="true"></i>
                        </button>
                        // < md: everything secondary lives in one menu
                        <div class={classes!("relative", "md:hidden")}>
                            <button
                                type="button"
                                class={classes!(
                                    toolbar_icon_button.clone(),
                                    "-mr-2",
                                    (*more_menu_open).then_some("bg-[var(--surface-alt)]")
                                )}
                                onclick={toggle_more_menu.clone()}
                                aria-label={more_label}
                                aria-haspopup="menu"
                                aria-expanded={(*more_menu_open).to_string()}
                            >
                                <i class={classes!("fas", "fa-ellipsis", "text-[15px]")} aria-hidden="true"></i>
                            </button>
                            if *more_menu_open {
                                <div
                                    class={classes!("fixed", "inset-0", "z-[55]", "cursor-default")}
                                    onclick={close_more_menu.clone()}
                                    aria-hidden="true"
                                />
                                <div
                                    class={classes!(
                                        "sf-pop-in",
                                        "absolute",
                                        "right-0",
                                        "top-[calc(100%+0.5rem)]",
                                        "z-[56]",
                                        "w-[min(16rem,calc(100vw-2.5rem))]",
                                        "overflow-hidden",
                                        "rounded-[12px]",
                                        "border",
                                        "border-[var(--border-strong)]",
                                        "bg-[var(--surface)]",
                                        "py-1.5"
                                    )}
                                    role="menu"
                                >
                                    {
                                        if has_detailed_summary {
                                            html! {
                                                <button
                                                    type="button"
                                                    role="menuitem"
                                                    class={more_menu_item.clone()}
                                                    onclick={close_more_then(open_brief_click.clone())}
                                                >
                                                    <i class={classes!("fas", "fa-list-check", "w-4", "text-[13px]", "text-[var(--muted)]")} aria-hidden="true"></i>
                                                    { brief_button_text }
                                                </button>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    {
                                        if can_export_markdown {
                                            html! {
                                                <>
                                                    <button
                                                        type="button"
                                                        role="menuitem"
                                                        class={more_menu_item.clone()}
                                                        onclick={close_more_then(open_raw_markdown_click.clone())}
                                                    >
                                                        <i class={classes!("far", "fa-file-lines", "w-4", "text-[13px]", "text-[var(--muted)]")} aria-hidden="true"></i>
                                                        { raw_markdown_button_text }
                                                    </button>
                                                    // Stays open so the "copied" confirmation is visible.
                                                    <button
                                                        type="button"
                                                        role="menuitem"
                                                        class={classes!(
                                                            more_menu_item.clone(),
                                                            (*markdown_copied).then_some("!text-[var(--ok)]")
                                                        )}
                                                        onclick={copy_markdown_click.clone()}
                                                        aria-live="polite"
                                                    >
                                                        <i class={classes!(export_button_icon.clone(), "w-4", "text-[13px]", "text-[var(--muted)]")} aria-hidden="true"></i>
                                                        { export_button_label }
                                                    </button>
                                                </>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <div class={classes!("my-1.5", "border-t", "border-[var(--border)]")} role="separator" />
                                    <button
                                        type="button"
                                        role="menuitem"
                                        class={more_menu_item.clone()}
                                        onclick={close_more_then(open_trend_click.clone())}
                                    >
                                        <i class={classes!("fas", "fa-chart-line", "w-4", "text-[13px]", "text-[var(--muted)]")} aria-hidden="true"></i>
                                        { t::TREND_TOOLTIP }
                                    </button>
                                    <button
                                        type="button"
                                        role="menuitem"
                                        class={more_menu_item.clone()}
                                        onclick={close_more_then(jump_to_comments_click.clone())}
                                    >
                                        <i class={classes!("far", "fa-comment", "w-4", "text-[13px]", "text-[var(--muted)]")} aria-hidden="true"></i>
                                        { comments_label }
                                    </button>
                                </div>
                            }
                        </div>
                    </div>
                </div>
            }
        };

        let (prev_article, next_article) = (*adjacent_articles).clone();
        let has_summary_lead = !article.summary.trim().is_empty();

        html! {
            <section class={classes!("article-layout-shell")}>
                <div class={classes!("article-main-column")}>
                    <article class={classes!(
                        "article-detail",
                        "mx-auto",
                        "w-full",
                        "pb-16",
                        "pt-10",
                        "md:pt-16"
                    )}>
                        // Anthropic Research Paper / Long-form Header
                        <header class={classes!(
                            "mx-auto",
                            "w-full",
                            "max-w-[760px]",
                            "fade-in"
                        )}>
                            <div
                                class={classes!(
                                    "flex",
                                    "flex-wrap",
                                    "items-center",
                                    "gap-x-2.5",
                                    "gap-y-1.5",
                                    "font-mono",
                                    "text-[0.75rem]",
                                    "tabular-nums",
                                    "uppercase",
                                    "tracking-[0.08em]",
                                    "text-[var(--faint)]"
                                )}
                                aria-label={t::ARTICLE_META_ARIA}
                            >
                                <Link<Route>
                                    to={Route::CategoryDetail { category: article.category.clone() }}
                                    classes={classes!(
                                        "font-semibold",
                                        "text-[var(--muted)]",
                                        "no-underline",
                                        "transition-colors",
                                        "hover:text-[var(--primary)]"
                                    )}
                                >
                                    { article.category.clone() }
                                </Link<Route>>
                                <span aria-hidden="true">{ "·" }</span>
                                <time>{ article.date.clone() }</time>
                                <span aria-hidden="true">{ "·" }</span>
                                <span>{ fill_one(t::READ_TIME_TEMPLATE, article.read_time) }</span>
                                <span aria-hidden="true">{ "·" }</span>
                                <span>{ fill_one(t::WORD_COUNT_TEMPLATE, word_count) }</span>
                                <span aria-hidden="true">{ "·" }</span>
                                <span>
                                    {
                                        if let Some(total) = *view_total {
                                            fill_one(t::VIEW_COUNT_TEMPLATE, total)
                                        } else {
                                            t::VIEW_COUNT_LOADING.to_string()
                                        }
                                    }
                                </span>
                                {
                                    if has_interactive_mirror {
                                        html! {
                                            <span class={classes!(
                                                "inline-flex",
                                                "items-center",
                                                "gap-1",
                                                "rounded-full",
                                                "bg-[var(--primary-soft)]",
                                                "px-2.5",
                                                "py-0.5",
                                                "font-mono",
                                                "text-[0.6875rem]",
                                                "font-medium",
                                                "uppercase",
                                                "tracking-[0.06em]",
                                                "text-[var(--primary)]"
                                            )}>
                                                { "◆ Interactive" }
                                            </span>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>

                            <h1 class={classes!(
                                "m-0",
                                "mt-4",
                                "font-serif",
                                "font-normal",
                                "text-[clamp(2rem,1.45rem+2.3vw,3.35rem)]",
                                "leading-[1.12]",
                                "tracking-[-0.02em]",
                                "text-[var(--text)]",
                                "[text-wrap:balance]"
                            )}>
                                { article.title.clone() }
                            </h1>

                            {
                                if has_summary_lead {
                                    html! {
                                        <p class={classes!(
                                            "m-0",
                                            "mt-5",
                                            "font-serif",
                                            "text-[1.125rem]",
                                            "sm:text-[1.25rem]",
                                            "leading-[1.62]",
                                            "text-[var(--muted)]"
                                        )}>
                                            { article.summary.clone() }
                                        </p>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                            <div class={classes!(
                                "mt-6",
                                "flex",
                                "flex-wrap",
                                "items-center",
                                "justify-between",
                                "gap-x-6",
                                "gap-y-3"
                            )}>
                                <div class={classes!(
                                    "flex",
                                    "flex-wrap",
                                    "items-center",
                                    "gap-x-3",
                                    "gap-y-1.5",
                                    "text-[0.875rem]",
                                    "text-[var(--muted)]"
                                )}>
                                    <span class={classes!("font-medium", "text-[var(--text-2)]")}>
                                        { article.author.clone() }
                                    </span>
                                    {
                                        if let Some(source_url) = article
                                            .source_url
                                            .clone()
                                            .filter(|value| !value.trim().is_empty())
                                        {
                                            html! {
                                                <>
                                                    <span class="text-[var(--faint)]" aria-hidden="true">{ "·" }</span>
                                                    <a
                                                        href={source_url}
                                                        target="_blank"
                                                        rel="noreferrer noopener"
                                                        class={classes!(
                                                            "sf-link",
                                                            "inline-flex",
                                                            "items-center",
                                                            "gap-1.5",
                                                            "font-mono",
                                                            "text-[0.75rem]",
                                                            "text-[var(--muted)]",
                                                            "hover:text-[var(--primary)]"
                                                        )}
                                                    >
                                                        { t::SOURCE_LINK_TEXT }
                                                        <i class={classes!("fas", "fa-arrow-up-right-from-square", "text-[10px]")} aria-hidden="true"></i>
                                                    </a>
                                                </>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    {
                                        if !article.tags.is_empty() {
                                            html! {
                                                <div class={classes!("flex", "flex-wrap", "items-center", "gap-2")}>
                                                    <span class="text-[var(--faint)]" aria-hidden="true">{ "·" }</span>
                                                    { for article.tags.iter().take(4).map(|tag| {
                                                        html! {
                                                            <Link<Route>
                                                                to={Route::TagDetail { tag: tag.to_string() }}
                                                                classes={classes!(
                                                                    "font-mono",
                                                                    "text-[0.75rem]",
                                                                    "text-[var(--faint)]",
                                                                    "no-underline",
                                                                    "transition-colors",
                                                                    "hover:text-[var(--primary)]"
                                                                )}
                                                            >
                                                                { format!("#{}", tag) }
                                                            </Link<Route>>
                                                        }
                                                    }) }
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>

                            </div>

                            { render_article_actions() }

                            <div class={classes!("h-8", "md:h-10")} aria-hidden="true" />
                        </header>

                        // Cover image placed cleanly below the 1px divider
                        {
                            if let Some(image) = article.featured_image.clone() {
                                let image_src = image_url(&image);
                                let open_featured_preview = {
                                    let open_image_preview = open_image_preview.clone();
                                    let image_src = image_src.clone();
                                    Callback::from(move |_| {
                                        open_image_preview.emit(image_src.clone());
                                    })
                                };
                                html! {
                                    <figure class={classes!(
                                        "mx-0",
                                        "mb-10",
                                        "md:mb-12",
                                        "mx-auto",
                                        "w-full",
                                        "max-w-[960px]",
                                        "overflow-hidden",
                                        "rounded-[12px]",
                                        "border",
                                        "border-[var(--border)]",
                                        "bg-[var(--surface-alt)]",
                                        "relative",
                                        "group"
                                    )}>
                                        <ImageWithLoading
                                            src={image_src.clone()}
                                            alt={article.title.clone()}
                                            loading={Some(AttrValue::from("lazy"))}
                                            onclick={Some(open_featured_preview.clone())}
                                            class={classes!(
                                                "w-full",
                                                "max-h-[520px]",
                                                "object-cover",
                                                "block",
                                                "cursor-zoom-in"
                                            )}
                                            container_class={classes!("w-full")}
                                        />
                                        <button
                                            type="button"
                                            class={classes!(
                                                "hidden",
                                                "md:inline-flex",
                                                "items-center",
                                                "gap-1.5",
                                                "absolute",
                                                "bottom-3.5",
                                                "right-3.5",
                                                "rounded-full",
                                                "border",
                                                "border-[var(--border-strong)]",
                                                "bg-[var(--surface)]/92",
                                                "px-3.5",
                                                "py-1.5",
                                                "font-mono",
                                                "text-[0.75rem]",
                                                "text-[var(--text)]",
                                                "opacity-0",
                                                "backdrop-blur-sm",
                                                "transition-opacity",
                                                "duration-150",
                                                "group-hover:opacity-100",
                                                "hover:border-[var(--primary)]",
                                                "hover:text-[var(--primary)]"
                                            )}
                                            onclick={open_featured_preview}
                                        >
                                            { t::VIEW_ORIGINAL_IMAGE }
                                        </button>
                                    </figure>
                                }
                            } else {
                                html! {}
                            }
                        }

                        // Reading Column: Inline Interactive Banner + Prose Body
                        <div class={classes!("mx-auto", "w-full", "max-w-[720px]")}>
                            {
                                if show_interactive_callout {
                                    html! {
                                        <aside
                                            class={classes!(
                                                "mb-8",
                                                "flex",
                                                "flex-wrap",
                                                "items-center",
                                                "justify-between",
                                                "gap-3",
                                                "rounded-[10px]",
                                                "border",
                                                "border-[var(--border-strong)]",
                                                "border-l-2",
                                                "border-l-[var(--primary)]",
                                                "bg-[var(--surface-alt)]",
                                                "px-4",
                                                "py-3.5"
                                            )}
                                            role="note"
                                        >
                                            <div class={classes!("flex", "flex-1", "flex-wrap", "items-center", "gap-2.5", "min-w-0", "sm:min-w-[220px]")}>
                                                <span class={classes!(
                                                    "inline-flex",
                                                    "items-center",
                                                    "gap-1",
                                                    "rounded-full",
                                                    "bg-[var(--primary-soft)]",
                                                    "px-2.5",
                                                    "py-0.5",
                                                    "font-mono",
                                                    "text-[0.6875rem]",
                                                    "font-medium",
                                                    "uppercase",
                                                    "tracking-[0.06em]",
                                                    "text-[var(--primary)]"
                                                )}>
                                                    { format!("◆ {}", t::INTERACTIVE_ALERT_BADGE) }
                                                </span>
                                                <span class={classes!("text-[0.875rem]", "leading-relaxed", "text-[var(--text-2)]")}>
                                                    { interactive_callout_hint }
                                                </span>
                                            </div>
                                            <div class={classes!("flex", "items-center", "gap-2")}>
                                                {
                                                    if let Some(href) = interactive_callout_href.clone() {
                                                        html! {
                                                            <a
                                                                class={classes!("sf-btn", "sf-btn--primary", "sf-btn--sm", "no-underline")}
                                                                href={href}
                                                            >
                                                                { interactive_callout_cta }
                                                                <span aria-hidden="true">{ "→" }</span>
                                                            </a>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                <button
                                                    type="button"
                                                    class={classes!(
                                                        "inline-flex",
                                                        "h-7",
                                                        "w-7",
                                                        "items-center",
                                                        "justify-center",
                                                        "rounded-full",
                                                        "text-[var(--faint)]",
                                                        "transition-colors",
                                                        "hover:bg-[var(--surface)]",
                                                        "hover:text-[var(--text)]"
                                                    )}
                                                    aria-label={interactive_callout_dismiss}
                                                    title={interactive_callout_dismiss}
                                                    onclick={dismiss_interactive_callout_click.clone()}
                                                >
                                                    <i class="fas fa-xmark text-xs" aria-hidden="true"></i>
                                                </button>
                                            </div>
                                        </aside>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                            <section
                                key={markdown_render_key.clone()}
                                ref={article_content_ref.clone()}
                                class={classes!("article-content", "prose")}
                                aria-label={t::ARTICLE_BODY_ARIA}
                                onmouseup={on_article_mouseup.clone()}
                            />
                        </div>

                        // End-of-article Footer, Prev/Next, Related, Comments & Workshop Link
                        <div class={classes!("mx-auto", "mt-14", "w-full", "max-w-[760px]")}>
                            {
                                if !article.tags.is_empty() {
                                    html! {
                                        <footer class={classes!("border-t", "border-[var(--border)]", "pt-6")}>
                                            <div class={classes!("flex", "flex-wrap", "items-center", "gap-2")}>
                                                <span class={classes!(
                                                    "mr-1",
                                                    "font-mono",
                                                    "text-[0.6875rem]",
                                                    "uppercase",
                                                    "tracking-[0.08em]",
                                                    "text-[var(--faint)]"
                                                )}>
                                                    { t::TAGS_TITLE }
                                                </span>
                                                { for article.tags.iter().map(|tag| {
                                                    html! {
                                                        <Link<Route>
                                                            to={Route::TagDetail { tag: tag.to_string() }}
                                                            classes={classes!("sf-chip", "no-underline")}
                                                        >
                                                            { format!("#{}", tag) }
                                                        </Link<Route>>
                                                    }
                                                }) }
                                            </div>
                                        </footer>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                            // Clean 1px Hairline Two-Column Previous / Next Navigation
                            {
                                if prev_article.is_some() || next_article.is_some() {
                                    html! {
                                        <nav
                                            class={classes!(
                                                "mt-8",
                                                "grid",
                                                "grid-cols-1",
                                                "gap-6",
                                                "border-t",
                                                "border-[var(--border)]",
                                                "pt-7",
                                                "sm:grid-cols-2"
                                            )}
                                            aria-label="Adjacent Articles"
                                        >
                                            <div>
                                                {
                                                    if let Some(prev) = prev_article {
                                                        html! {
                                                            <Link<Route>
                                                                to={Route::ArticleDetail { id: prev.id }}
                                                                classes={classes!(
                                                                    "group",
                                                                    "flex",
                                                                    "flex-col",
                                                                    "gap-1.5",
                                                                    "no-underline"
                                                                )}
                                                            >
                                                                <span class={classes!(
                                                                    "font-mono",
                                                                    "text-[0.6875rem]",
                                                                    "uppercase",
                                                                    "tracking-[0.08em]",
                                                                    "text-[var(--faint)]"
                                                                )}>
                                                                    { "← 上一篇 · PREVIOUS" }
                                                                </span>
                                                                <span class={classes!(
                                                                    "font-serif",
                                                                    "font-normal",
                                                                    "text-[1.125rem]",
                                                                    "leading-[1.35]",
                                                                    "text-[var(--text)]",
                                                                    "decoration-[var(--primary)]",
                                                                    "decoration-1",
                                                                    "underline-offset-4",
                                                                    "group-hover:underline"
                                                                )}>
                                                                    { prev.title }
                                                                </span>
                                                            </Link<Route>>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>
                                            <div class={classes!("sm:text-right")}>
                                                {
                                                    if let Some(next) = next_article {
                                                        html! {
                                                            <Link<Route>
                                                                to={Route::ArticleDetail { id: next.id }}
                                                                classes={classes!(
                                                                    "group",
                                                                    "flex",
                                                                    "flex-col",
                                                                    "gap-1.5",
                                                                    "no-underline",
                                                                    "sm:items-end"
                                                                )}
                                                            >
                                                                <span class={classes!(
                                                                    "font-mono",
                                                                    "text-[0.6875rem]",
                                                                    "uppercase",
                                                                    "tracking-[0.08em]",
                                                                    "text-[var(--faint)]"
                                                                )}>
                                                                    { "下一篇 · NEXT →" }
                                                                </span>
                                                                <span class={classes!(
                                                                    "font-serif",
                                                                    "font-normal",
                                                                    "text-[1.125rem]",
                                                                    "leading-[1.35]",
                                                                    "text-[var(--text)]",
                                                                    "decoration-[var(--primary)]",
                                                                    "decoration-1",
                                                                    "underline-offset-4",
                                                                    "group-hover:underline"
                                                                )}>
                                                                    { next.title }
                                                                </span>
                                                            </Link<Route>>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>
                                        </nav>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                            // Further Reading / Related Articles
                            <section class={classes!(
                                "mt-10",
                                "border-t",
                                "border-[var(--border)]",
                                "pt-8"
                            )}>
                                <div class="mb-5">
                                    <div class="sf-eyebrow">{ "FURTHER READING" }</div>
                                    <h2 class={classes!(
                                        "m-0",
                                        "font-serif",
                                        "font-normal",
                                        "text-[1.5rem]",
                                        "leading-[1.25]",
                                        "text-[var(--text)]"
                                    )}>{ t::RELATED_TITLE }</h2>
                                </div>
                                if *related_loading {
                                    <div class={classes!(
                                        "flex",
                                        "items-center",
                                        "gap-2",
                                        "text-sm",
                                        "text-[var(--muted)]"
                                    )}>
                                        <LoadingSpinner size={SpinnerSize::Small} />
                                        <span>{ t::RELATED_LOADING }</span>
                                    </div>
                                } else if related_articles.is_empty() {
                                    <p class={classes!("m-0", "font-serif", "italic", "text-[var(--muted)]")}>
                                        { t::NO_RELATED }
                                    </p>
                                } else {
                                    <div class={classes!(
                                        "grid",
                                        "gap-8",
                                        "md:grid-cols-2"
                                    )}>
                                        { for related_articles.iter().map(|article| {
                                            html! { <ArticleCard key={article.id.clone()} article={article.clone()} /> }
                                        }) }
                                    </div>
                                }
                            </section>

                            // Restrained `.sf-card` Comments Section with `◆ AI` Terracotta Chip
                            <section
                                id={COMMENT_SECTION_ID}
                                class={classes!(
                                    "sf-card",
                                    "mt-12",
                                    "p-6",
                                    "sm:p-8",
                                    "flex",
                                    "flex-col",
                                    "gap-6"
                                )}
                            >
                                <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                                    <div>
                                        <div class="sf-eyebrow">{ "READER DISCUSSION" }</div>
                                        <h2 class={classes!(
                                            "m-0",
                                            "font-serif",
                                            "font-normal",
                                            "text-[1.5rem]",
                                            "leading-[1.25]",
                                            "text-[var(--text)]"
                                        )}>{ "评论区" }</h2>
                                        <p class={classes!("m-0", "mt-1", "font-mono", "text-xs", "tabular-nums", "text-[var(--faint)]")}>
                                            { format!("当前评论 {} 条", *comments_total) }
                                        </p>
                                    </div>
                                    <button
                                        type="button"
                                        class={classes!("sf-btn", "sf-btn--secondary", "sf-btn--sm")}
                                        onclick={refresh_comments_click}
                                    >
                                        <i class={classes!("fas", "fa-rotate-right", "text-[11px]")} aria-hidden="true"></i>
                                        { "刷新评论" }
                                    </button>
                                </div>

                                {
                                    if let Some((is_success, message)) = (*selection_submit_feedback).clone() {
                                        let status_class = if is_success {
                                            classes!(
                                                "m-0",
                                                "rounded-[8px]",
                                                "border",
                                                "border-[var(--ok)]/35",
                                                "bg-[var(--ok-soft)]",
                                                "px-3.5",
                                                "py-2.5",
                                                "text-sm",
                                                "text-[var(--ok)]"
                                            )
                                        } else {
                                            classes!(
                                                "m-0",
                                                "rounded-[8px]",
                                                "border",
                                                "border-[var(--err)]/35",
                                                "bg-[var(--err-soft)]",
                                                "px-3.5",
                                                "py-2.5",
                                                "text-sm",
                                                "text-[var(--err)]"
                                            )
                                        };
                                        html! {
                                            <p class={status_class}>{ message }</p>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }

                                <div class={classes!(
                                    "rounded-[10px]",
                                    "border",
                                    "border-[var(--border)]",
                                    "bg-[var(--surface-alt)]",
                                    "p-4",
                                    "sm:p-5",
                                    "flex",
                                    "flex-col",
                                    "gap-3"
                                )}>
                                    <label class={classes!(
                                        "text-sm",
                                        "font-medium",
                                        "text-[var(--text)]"
                                    )}>
                                        { "发表评论" }
                                    </label>
                                    if let Some(reply_target) = (*footer_reply_target).clone() {
                                        <div class={classes!(
                                            "rounded-[8px]",
                                            "border",
                                            "border-[var(--border-strong)]",
                                            "border-l-2",
                                            "border-l-[var(--primary)]",
                                            "bg-[var(--surface)]",
                                            "px-3.5",
                                            "py-2.5",
                                            "text-sm",
                                            "flex",
                                            "flex-col",
                                            "gap-1.5"
                                        )}>
                                            <div class={classes!("flex", "items-center", "justify-between", "gap-2", "flex-wrap")}>
                                                <p class={classes!("m-0", "font-mono", "text-[0.6875rem]", "uppercase", "tracking-[0.08em]", "text-[var(--faint)]")}>
                                                    { format!("正在引用：{}", reply_target.author_name) }
                                                </p>
                                                <button
                                                    type="button"
                                                    class={classes!("sf-btn", "sf-btn--ghost", "sf-btn--sm")}
                                                    onclick={clear_footer_reply_target.clone()}
                                                >
                                                    { "取消引用" }
                                                </button>
                                            </div>
                                            <p class={classes!("m-0", "text-sm", "text-[var(--text-2)]", "break-words")}>{ reply_target.comment_text.clone() }</p>
                                            if let Some(ai_reply) = reply_target.ai_reply_markdown {
                                                <p class={classes!("m-0", "text-xs", "text-[var(--muted)]", "break-words")}>
                                                    { format!("引用 AI 回复：{}", ai_reply.chars().take(140).collect::<String>()) }
                                                </p>
                                            }
                                        </div>
                                    }
                                    <textarea
                                        class={classes!(
                                            "sf-textarea",
                                            "comment-compose-textarea",
                                            "min-h-[108px]",
                                            "w-full",
                                            "py-3"
                                        )}
                                        placeholder={"写下你对本文的疑问、勘误或补充建议..."}
                                        value={(*footer_comment_input).clone()}
                                        oninput={on_footer_comment_input}
                                    />
                                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                        {
                                            if let Some((is_success, message)) = (*footer_submit_feedback).clone() {
                                                let status_class = if is_success {
                                                    classes!("text-sm", "text-[var(--ok)]")
                                                } else {
                                                    classes!("text-sm", "text-[var(--err)]")
                                                };
                                                html! { <span class={status_class}>{ message }</span> }
                                            } else {
                                                html! { <span class={classes!("font-mono", "text-xs", "text-[var(--faint)]")}>{ "每个用户每分钟最多提交 1 条评论 · 支持选中正文段落直接提问" }</span> }
                                            }
                                        }
                                        <button
                                            type="button"
                                            class={classes!("sf-btn", "sf-btn--primary")}
                                            onclick={submit_footer_comment_click}
                                            disabled={*footer_submit_loading}
                                        >
                                            {
                                                if *footer_submit_loading {
                                                    "提交中..."
                                                } else {
                                                    "提交评论"
                                                }
                                            }
                                        </button>
                                    </div>
                                </div>

                                {
                                    if *comments_loading {
                                        html! {
                                            <div class={classes!(
                                                "py-6",
                                                "text-sm",
                                                "text-[var(--muted)]",
                                                "inline-flex",
                                                "items-center",
                                                "gap-2"
                                            )}>
                                                <LoadingSpinner size={SpinnerSize::Small} />
                                                <span>{ "评论加载中..." }</span>
                                            </div>
                                        }
                                    } else if let Some(error) = (*comments_error).clone() {
                                        html! {
                                            <p class={classes!(
                                                "m-0",
                                                "rounded-[8px]",
                                                "border",
                                                "border-[var(--err)]/35",
                                                "bg-[var(--err-soft)]",
                                                "px-3.5",
                                                "py-2.5",
                                                "text-sm",
                                                "text-[var(--err)]"
                                            )}>
                                                { error }
                                            </p>
                                        }
                                    } else if comments.is_empty() {
                                        html! {
                                            <p class={classes!("m-0", "font-serif", "italic", "text-[0.95rem]", "text-[var(--muted)]")}>
                                                { "暂无评论，欢迎成为第一个留言的人。" }
                                            </p>
                                        }
                                    } else {
                                        html! {
                                            <div class={classes!("flex", "flex-col", "divide-y", "divide-[var(--border)]")}>
                                                { for comments.iter().map(|comment| {
                                                    let anchor_block_id = comment.anchor_block_id.clone();
                                                    let selected_text = comment.selected_text.clone();
                                                    let reply_to_comment_id = comment.reply_to_comment_id.clone();
                                                    let reply_to_comment_text = comment.reply_to_comment_text.clone();
                                                    let reply_to_ai_reply_markdown = comment.reply_to_ai_reply_markdown.clone();
                                                    let jump_quote_click = {
                                                        let anchor_block_id = anchor_block_id.clone();
                                                        Callback::from(move |_| {
                                                            if let Some(block_id) = anchor_block_id.clone() {
                                                                scroll_to_anchor_block(&block_id);
                                                            }
                                                        })
                                                    };
                                                    let jump_reply_comment_click = {
                                                        let reply_to_comment_id = reply_to_comment_id.clone();
                                                        Callback::from(move |_| {
                                                            if let Some(comment_id) = reply_to_comment_id.clone() {
                                                                scroll_to_comment_card(&comment_id);
                                                            }
                                                        })
                                                    };
                                                    let quote_reply_click = {
                                                        let footer_reply_target = footer_reply_target.clone();
                                                        let comment_id = comment.comment_id.clone();
                                                        let author_name = comment.author_name.clone();
                                                        let comment_text = comment.comment_text.clone();
                                                        let ai_reply_markdown = comment.ai_reply_markdown.clone();
                                                        Callback::from(move |_| {
                                                            footer_reply_target.set(Some(CommentReplyDraft {
                                                                comment_id: comment_id.clone(),
                                                                author_name: author_name.clone(),
                                                                comment_text: comment_text.clone(),
                                                                ai_reply_markdown: ai_reply_markdown.clone(),
                                                            }));
                                                            scroll_to_element_id(COMMENT_SECTION_ID);
                                                        })
                                                    };
                                                    let avatar_initial =
                                                        comment_avatar_initials(
                                                            &comment.author_name,
                                                            &comment.author_avatar_seed,
                                                        );
                                                    let avatar_style =
                                                        comment_avatar_style(&comment.author_avatar_seed);
                                                    let region_label = display_comment_region(&comment.ip_region);
                                                    let ai_reply_html = comment
                                                        .ai_reply_markdown
                                                        .clone()
                                                        .filter(|value| !value.trim().is_empty())
                                                        .map(|value| {
                                                            AttrValue::from(markdown_to_html(&value))
                                                        });

                                                    html! {
                                                        <article
                                                            id={format!("comment-item-{}", comment.comment_id)}
                                                            class={classes!("py-6", "first:pt-2", "last:pb-1", "flex", "flex-col", "gap-3.5")}
                                                        >
                                                            <header class={classes!("flex", "items-center", "justify-between", "gap-3")}>
                                                                <div class={classes!("flex", "items-center", "gap-3", "min-w-0")}>
                                                                    <div
                                                                        class={classes!(
                                                                            "inline-flex",
                                                                            "h-8",
                                                                            "w-8",
                                                                            "shrink-0",
                                                                            "items-center",
                                                                            "justify-center",
                                                                            "rounded-full",
                                                                            "font-mono",
                                                                            "text-[0.7rem]",
                                                                            "font-semibold",
                                                                            "uppercase",
                                                                            "text-white"
                                                                        )}
                                                                        style={avatar_style}
                                                                    >
                                                                        { avatar_initial }
                                                                    </div>
                                                                    <div class={classes!("min-w-0")}>
                                                                        <p class={classes!("m-0", "truncate", "text-sm", "font-medium", "text-[var(--text)]")}>
                                                                            { comment.author_name.clone() }
                                                                        </p>
                                                                        <p class={classes!("m-0", "mt-0.5", "font-mono", "text-[0.7rem]", "tabular-nums", "text-[var(--faint)]")}>
                                                                            { format!("{} · {}", region_label, format_published_time(comment.published_at)) }
                                                                        </p>
                                                                    </div>
                                                                </div>
                                                                <button
                                                                    type="button"
                                                                    class={classes!("sf-btn", "sf-btn--ghost", "sf-btn--sm", "shrink-0")}
                                                                    onclick={quote_reply_click}
                                                                >
                                                                    { "引用并回复" }
                                                                </button>
                                                            </header>

                                                            if let Some(quote_text) = selected_text {
                                                                <div class={classes!(
                                                                    "rounded-r-[8px]",
                                                                    "border-l-2",
                                                                    "border-l-[var(--primary)]",
                                                                    "bg-[var(--surface-alt)]",
                                                                    "px-3.5",
                                                                    "py-2.5"
                                                                )}>
                                                                    <div class={classes!("flex", "items-center", "justify-between", "gap-2", "flex-wrap")}>
                                                                        <span class={classes!("font-mono", "text-[0.6875rem]", "uppercase", "tracking-[0.08em]", "text-[var(--faint)]")}>
                                                                            { "选中段落" }
                                                                        </span>
                                                                        {
                                                                            if anchor_block_id.is_some() {
                                                                                html! {
                                                                                    <button
                                                                                        type="button"
                                                                                        class={classes!(
                                                                                            "font-mono",
                                                                                            "text-[0.7rem]",
                                                                                            "text-[var(--primary)]",
                                                                                            "hover:underline"
                                                                                        )}
                                                                                        onclick={jump_quote_click}
                                                                                    >
                                                                                        { "定位到正文 →" }
                                                                                    </button>
                                                                                }
                                                                            } else {
                                                                                html! {}
                                                                            }
                                                                        }
                                                                    </div>
                                                                    <p class={classes!("m-0", "mt-1.5", "font-serif", "italic", "text-[0.9375rem]", "leading-relaxed", "text-[var(--muted)]", "whitespace-pre-wrap", "break-words")}>
                                                                        { quote_text }
                                                                    </p>
                                                                </div>
                                                            }

                                                            if let Some(reply_text) = reply_to_comment_text {
                                                                <div class={classes!(
                                                                    "rounded-[8px]",
                                                                    "border",
                                                                    "border-[var(--border)]",
                                                                    "bg-[var(--surface-alt)]",
                                                                    "px-3.5",
                                                                    "py-2.5"
                                                                )}>
                                                                    <div class={classes!("flex", "items-center", "justify-between", "gap-2", "flex-wrap")}>
                                                                        <span class={classes!("font-mono", "text-[0.6875rem]", "uppercase", "tracking-[0.08em]", "text-[var(--faint)]")}>
                                                                            { "引用评论" }
                                                                        </span>
                                                                        if reply_to_comment_id.is_some() {
                                                                            <button
                                                                                type="button"
                                                                                class={classes!("font-mono", "text-[0.7rem]", "text-[var(--primary)]", "hover:underline")}
                                                                                onclick={jump_reply_comment_click}
                                                                            >
                                                                                { "定位原评论 →" }
                                                                            </button>
                                                                        }
                                                                    </div>
                                                                    <p class={classes!("m-0", "mt-1", "text-sm", "leading-relaxed", "text-[var(--text-2)]", "whitespace-pre-wrap", "break-words")}>
                                                                        { reply_text }
                                                                    </p>
                                                                    if let Some(ai_reply) = reply_to_ai_reply_markdown {
                                                                        <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted)]", "break-words")}>
                                                                            { format!("被引用 AI 回复：{}", ai_reply.chars().take(140).collect::<String>()) }
                                                                        </p>
                                                                    }
                                                                </div>
                                                            }

                                                            <p class={classes!("m-0", "text-[0.95rem]", "leading-[1.7]", "text-[var(--text-2)]", "whitespace-pre-wrap", "break-words")}>
                                                                { comment.comment_text.clone() }
                                                            </p>

                                                            if let Some(ai_reply_html) = ai_reply_html {
                                                                <section class={classes!(
                                                                    "rounded-[10px]",
                                                                    "border",
                                                                    "border-[var(--border)]",
                                                                    "bg-[var(--surface-alt)]",
                                                                    "p-4"
                                                                )}>
                                                                    <div class="mb-2 flex items-center gap-2">
                                                                        <span class={classes!(
                                                                            "inline-flex",
                                                                            "items-center",
                                                                            "gap-1",
                                                                            "rounded-full",
                                                                            "bg-[var(--primary-soft)]",
                                                                            "px-2.5",
                                                                            "py-0.5",
                                                                            "font-mono",
                                                                            "text-[0.6875rem]",
                                                                            "font-medium",
                                                                            "uppercase",
                                                                            "tracking-[0.06em]",
                                                                            "text-[var(--primary)]"
                                                                        )}>
                                                                            { "◆ AI" }
                                                                        </span>
                                                                        <span class={classes!("font-mono", "text-[0.7rem]", "text-[var(--faint)]")}>
                                                                            { "研究与技术解答" }
                                                                        </span>
                                                                    </div>
                                                                    <RawHtml
                                                                        class={classes!("article-content", "comment-ai-markdown")}
                                                                        html={ai_reply_html}
                                                                    />
                                                                </section>
                                                            }
                                                        </article>
                                                    }
                                                }) }
                                            </div>
                                        }
                                    }
                                }
                            </section>

                            // Reader-driven ingestion studio prompt
                            <div class={classes!(
                                "mt-10",
                                "flex",
                                "flex-wrap",
                                "items-center",
                                "justify-between",
                                "gap-4",
                                "border-t",
                                "border-[var(--border)]",
                                "pt-7"
                            )}>
                                <a
                                    href={crate::config::route_path("/studio#article")}
                                    onclick={navigate_to_studio}
                                    class={classes!(
                                        "sf-link",
                                        "font-serif",
                                        "italic",
                                        "text-[1.0625rem]",
                                        "text-[var(--text-2)]",
                                        "hover:text-[var(--primary)]"
                                    )}
                                >
                                    { "想读相关的技术深潜？前往共建台提交 →" }
                                </a>
                            </div>
                        </div>

                        {
                            if *is_brief_open {
                                html! {
                                    <div
                                        class={classes!(
                                            "fixed",
                                            "inset-0",
                                            "z-[95]",
                                            "flex",
                                            "items-center",
                                            "justify-center",
                                            "bg-[rgba(20,20,19,0.45)]",
                                            "dark:bg-[rgba(0,0,0,0.65)]",
                                            "p-4",
                                            "backdrop-blur-[4px]"
                                        )}
                                        role="dialog"
                                        aria-modal="true"
                                        aria-label={t::DETAILED_SUMMARY_ARIA}
                                        onclick={close_brief_click.clone()}
                                    >
                                        <section
                                            class={classes!(
                                                "w-full",
                                                "max-w-[760px]",
                                                "max-h-[85vh]",
                                                "overflow-auto",
                                                "rounded-[12px]",
                                                "border",
                                                "border-[var(--border-strong)]",
                                                "bg-[var(--surface)]",
                                                "px-4",
                                                "py-4",
                                                "shadow-[var(--shadow-lg)]",
                                                "sm:px-6",
                                                "sm:py-5"
                                            )}
                                            onclick={stop_brief_bubble.clone()}
                                        >
                                            <div class={classes!(
                                                "mb-4",
                                                "flex",
                                                "items-center",
                                                "justify-between",
                                                "gap-3",
                                                "border-b",
                                                "border-[var(--border)]",
                                                "pb-3"
                                            )}>
                                                <p class={classes!(
                                                    "m-0",
                                                    "inline-flex",
                                                    "items-center",
                                                    "gap-2",
                                                    "font-serif",
                                                    "font-normal",
                                                    "text-[1.2rem]",
                                                    "text-[var(--text)]"
                                                )}>
                                                    <span class="font-mono text-xs text-[var(--primary)]" aria-hidden="true">{ "◆" }</span>
                                                    { summary_title }
                                                </p>
                                                <button
                                                    type="button"
                                                    class={classes!("sf-btn", "sf-btn--secondary", "sf-btn--sm")}
                                                    aria-label={t::CLOSE_BRIEF_ARIA}
                                                    onclick={close_brief_click.clone()}
                                                >
                                                    { t::CLOSE_BRIEF_BUTTON }
                                                </button>
                                            </div>
                                            {
                                                if let Some(summary_html) = detailed_summary_html.clone() {
                                                    html! {
                                                        <RawHtml class={classes!(
                                                            "article-content",
                                                            "prose",
                                                            "text-[0.97rem]",
                                                            "leading-[1.8]"
                                                        )}
                                                        html={summary_html}
                                                        />
                                                    }
                                                } else {
                                                    html! {
                                                        <p class={classes!("m-0", "text-[var(--muted)]")}>
                                                            { "No brief available." }
                                                        </p>
                                                    }
                                                }
                                            }
                                        </section>
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </article>
                </div>
            </section>
        }
    } else {
        html! {
            <section class={classes!(
                "sf-card",
                "p-6",
                "sm:p-10",
                "my-10",
                "sm:my-14",
                "mx-auto",
                "max-w-[760px]",
                "flex",
                "flex-col",
                "gap-4"
            )}>
                <div class={classes!("flex", "flex-col", "gap-3", "fade-in")}>
                    <p class={classes!("sf-eyebrow", "m-0")}>{ "404 · NOT FOUND" }</p>
                    <h1 class={classes!(
                        "m-0",
                        "font-serif",
                        "font-normal",
                        "text-[1.6rem]",
                        "leading-[1.2]",
                        "text-[var(--text)]",
                        "sm:text-[2rem]"
                    )}>{ t::NOT_FOUND_TITLE }</h1>
                    <p class={classes!("m-0", "text-[var(--muted)]")}>{ t::NOT_FOUND_DESC }</p>
                    <div class={classes!("pt-1", "xl:hidden")}>
                        <button
                            type="button"
                            class={classes!("sf-btn", "sf-btn--secondary", "sf-btn--sm")}
                            onclick={handle_back.clone()}
                        >
                            <i class={classes!("fas", "fa-arrow-left", "text-[11px]")} aria-hidden="true"></i>
                            { t::BACK_TOOLTIP }
                        </button>
                    </div>
                </div>
            </section>
        }
    };

    html! {
        <main class={classes!("main", "mt-[var(--space-lg)]")}>
            // Fixed back button - hide when any overlay is open
            if !is_overlay_open {
                // >= xl only: floats in the left margin, anchored to the reading
                // column (see `.article-float-rail`); below xl these actions
                // live in the toolbar under the title.
                <div class={classes!(
                    "article-float-rail",
                    "hidden",
                    "xl:flex",
                    "fixed",
                    "z-50",
                    "flex-col",
                    "gap-3"
                )}>
                    <div>
                        <TooltipIconButton
                            icon={IconName::ArrowLeft}
                            tooltip={t::BACK_TOOLTIP}
                            position={TooltipPosition::Right}
                            onclick={handle_back}
                            size={20}
                        />
                    </div>
                    {
                        if article_data.is_some() {
                            html! {
                                <>
                                    <div>
                                        <TooltipIconButton
                                            icon={IconName::TrendingUp}
                                            tooltip={t::TREND_TOOLTIP}
                                            position={TooltipPosition::Right}
                                            onclick={open_trend_click.clone()}
                                            size={20}
                                        />
                                    </div>
                                    <div>
                                        <TooltipIconButton
                                            icon={IconName::MessageSquare}
                                            tooltip={"定位到评论区".to_string()}
                                            position={TooltipPosition::Right}
                                            onclick={jump_to_comments_click.clone()}
                                            size={20}
                                        />
                                    </div>
                                </>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            }

            <div class={classes!("container", "article-page-container")}>
                { body }
            </div>
            {
                if !is_overlay_open {
                    if let (Some((left, top)), Some(_)) = (*selection_button_pos, (*selection_draft).clone()) {
                        html! {
                            <button
                                type="button"
                                class={classes!("selection-comment-fab")}
                                style={format!("left: {left:.1}px; top: {top:.1}px;")}
                                onclick={open_selection_modal_click.clone()}
                            >
                                <i class={classes!("fas", "fa-comment-dots")} aria-hidden="true"></i>
                                { "评论所选" }
                            </button>
                        }
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }
            }
            {
                if *selection_modal_open {
                    let draft_snapshot = (*selection_draft).clone();
                    let selected_excerpt = draft_snapshot
                        .as_ref()
                        .map(|draft| draft.selected_text.chars().take(240).collect::<String>())
                        .unwrap_or_default();
                    let has_ellipsis = draft_snapshot
                        .as_ref()
                        .map(|draft| draft.selected_text.chars().count() > 240)
                        .unwrap_or(false);

                    html! {
                        <div
                            class={classes!(
                                "fixed",
                                "inset-0",
                                "z-[97]",
                                "flex",
                                "items-center",
                                "justify-center",
                                "bg-[rgba(20,20,19,0.45)]",
                                "dark:bg-[rgba(0,0,0,0.65)]",
                                "p-4",
                                "backdrop-blur-[4px]"
                            )}
                            role="dialog"
                            aria-modal="true"
                            aria-label={"评论所选内容"}
                            onclick={close_selection_modal_click.clone()}
                        >
                            <section
                                class={classes!(
                                    "w-full",
                                    "max-w-[580px]",
                                    "rounded-[12px]",
                                    "border",
                                    "border-[var(--border-strong)]",
                                    "bg-[var(--surface)]",
                                    "p-4",
                                    "shadow-[var(--shadow-lg)]",
                                    "flex",
                                    "flex-col",
                                    "gap-4",
                                    "sm:p-6"
                                )}
                                onclick={stop_selection_modal_bubble.clone()}
                            >
                                <header class={classes!("flex", "items-start", "justify-between", "gap-3")}>
                                    <div>
                                        <div class="sf-eyebrow">{ "INLINE QUOTE COMMENT" }</div>
                                        <h3 class={classes!(
                                            "m-0",
                                            "font-serif",
                                            "font-normal",
                                            "text-[1.35rem]",
                                            "leading-[1.25]",
                                            "text-[var(--text)]"
                                        )}>{ "评论所选内容" }</h3>
                                        <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted)]")}>
                                            { "你可以基于选中的段落提交疑问、勘误或补充观点。" }
                                        </p>
                                    </div>
                                    <button
                                        type="button"
                                        class={classes!("sf-btn", "sf-btn--secondary", "sf-btn--sm")}
                                        aria-label={"关闭评论弹窗"}
                                        onclick={close_selection_modal_click.clone()}
                                    >
                                        { "关闭" }
                                    </button>
                                </header>

                                <div class={classes!(
                                    "rounded-r-[8px]",
                                    "border-l-2",
                                    "border-l-[var(--primary)]",
                                    "bg-[var(--surface-alt)]",
                                    "px-3.5",
                                    "py-3"
                                )}>
                                    <p class={classes!("m-0", "font-mono", "text-[0.6875rem]", "uppercase", "tracking-[0.08em]", "text-[var(--faint)]")}>{ "选中内容" }</p>
                                    <p class={classes!("m-0", "mt-1.5", "font-serif", "italic", "text-[0.9375rem]", "leading-relaxed", "text-[var(--muted)]", "whitespace-pre-wrap", "break-words")}>
                                        {
                                            if has_ellipsis {
                                                format!("{selected_excerpt}...")
                                            } else {
                                                selected_excerpt
                                            }
                                        }
                                    </p>
                                </div>

                                <textarea
                                    class={classes!("sf-textarea", "comment-compose-textarea", "min-h-[112px]", "w-full", "py-3")}
                                    placeholder={"请输入你的疑问或评论..."}
                                    value={(*selection_comment_input).clone()}
                                    oninput={on_selection_comment_input}
                                />

                                {
                                    if let Some((is_success, message)) = (*selection_submit_feedback).clone() {
                                        let feedback_class = if is_success {
                                            classes!("m-0", "text-sm", "text-[var(--ok)]")
                                        } else {
                                            classes!("m-0", "text-sm", "text-[var(--err)]")
                                        };
                                        html! {
                                            <p class={feedback_class}>{ message }</p>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }

                                <div class={classes!("flex", "items-center", "justify-end", "gap-2.5")}>
                                    <button
                                        type="button"
                                        class={classes!("sf-btn", "sf-btn--secondary")}
                                        onclick={close_selection_modal_click}
                                    >
                                        { "取消" }
                                    </button>
                                    <button
                                        type="button"
                                        class={classes!("sf-btn", "sf-btn--primary")}
                                        onclick={submit_selection_comment_click}
                                        disabled={*selection_submit_loading}
                                    >
                                        {
                                            if *selection_submit_loading {
                                                "提交中..."
                                            } else {
                                                "提交评论"
                                            }
                                        }
                                    </button>
                                </div>
                            </section>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            {
                if *is_lightbox_open {
                    html! {
                        <div
                            class={classes!(
                                "fixed",
                                "inset-0",
                                "z-[100]",
                                "flex",
                                "items-center",
                                "justify-center",
                                "bg-[rgba(250,249,245,0.96)]",
                                "dark:bg-[rgba(20,20,19,0.96)]",
                                "p-4",
                                "text-[var(--text)]",
                                "backdrop-blur-[6px]",
                                "transition-opacity"
                            )}
                            role="dialog"
                            aria-modal="true"
                            onclick={close_lightbox_click.clone()}
                        >
                            <div
                                class={classes!(
                                    "absolute",
                                    "left-4",
                                    "top-4",
                                    "z-[101]",
                                    "flex",
                                    "items-center",
                                    "gap-1.5"
                                )}
                                onclick={stop_lightbox_bubble.clone()}
                            >
                                <button
                                    type="button"
                                    class={classes!("sf-btn", "sf-btn--secondary", "sf-btn--sm", "font-mono")}
                                    aria-label={t::LIGHTBOX_ZOOM_OUT_ARIA}
                                    onclick={zoom_out_click.clone()}
                                >
                                    { "−" }
                                </button>
                                <button
                                    type="button"
                                    class={classes!("sf-btn", "sf-btn--secondary", "sf-btn--sm", "font-mono", "tabular-nums")}
                                    aria-label={t::LIGHTBOX_ZOOM_RESET_ARIA}
                                    onclick={zoom_reset_click.clone()}
                                >
                                    { format!("{:.0}%", *preview_zoom * 100.0) }
                                </button>
                                <button
                                    type="button"
                                    class={classes!("sf-btn", "sf-btn--secondary", "sf-btn--sm", "font-mono")}
                                    aria-label={t::LIGHTBOX_ZOOM_IN_ARIA}
                                    onclick={zoom_in_click.clone()}
                                >
                                    { "+" }
                                </button>
                            </div>
                            <button
                                type="button"
                                class={classes!(
                                    "sf-btn",
                                    "sf-btn--secondary",
                                    "sf-btn--sm",
                                    "absolute",
                                    "right-4",
                                    "top-4",
                                    "z-[101]"
                                )}
                                aria-label={t::CLOSE_IMAGE_ARIA}
                                onclick={close_lightbox_click.clone()}
                            >
                                <i class="fas fa-xmark text-xs" aria-hidden="true"></i>
                            </button>
                            <div
                                class={classes!(
                                    "max-h-full",
                                    "max-w-full",
                                    "rounded-[12px]",
                                    "border",
                                    "border-[var(--border)]",
                                    "bg-[var(--surface)]",
                                    "p-2",
                                    "overflow-auto",
                                    "shadow-[var(--shadow-lg)]"
                                )}
                                style="touch-action: pan-x pan-y;"
                                onclick={stop_lightbox_bubble.clone()}
                                ontouchstart={lightbox_touch_start.clone()}
                                ontouchmove={lightbox_touch_move.clone()}
                                ontouchend={lightbox_touch_end.clone()}
                            >
                                {
                                    if let Some(src) = (*preview_image_url).clone() {
                                        let alt_text = article_data
                                            .as_ref()
                                            .map(|article| article.title.clone())
                                            .unwrap_or_else(|| t::DEFAULT_IMAGE_ALT.to_string());
                                        html! {
                                            <>
                                                <img
                                                    src={src.clone()}
                                                    alt={alt_text}
                                                    class={classes!(
                                                        "block",
                                                        "max-h-[88vh]",
                                                        "max-w-[90vw]",
                                                        "h-auto",
                                                        "w-auto",
                                                        "object-contain",
                                                        "transition-transform",
                                                        "duration-150"
                                                    )}
                                                    style={format!("transform: scale({}); transform-origin: center center;", *preview_zoom)}
                                                    loading="eager"
                                                    decoding="async"
                                                    onerror={mark_preview_failed.clone()}
                                                    onload={mark_preview_loaded.clone()}
                                                />
                                                {
                                                    if *preview_image_failed {
                                                        html! {
                                                            <div class={classes!(
                                                                "mt-3",
                                                                "max-w-[90vw]",
                                                                "rounded-[8px]",
                                                                "border",
                                                                "border-[var(--err)]/35",
                                                                "bg-[var(--err-soft)]",
                                                                "px-3.5",
                                                                "py-2.5",
                                                                "text-sm",
                                                                "text-[var(--err)]"
                                                            )}>
                                                                { fill_one(t::IMAGE_PREVIEW_FAILED, &src) }
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            {
                if *is_trend_open {
                    html! {
                        <div
                            class={classes!(
                                "fixed",
                                "inset-0",
                                "z-[96]",
                                "flex",
                                "items-center",
                                "justify-center",
                                "bg-[rgba(20,20,19,0.45)]",
                                "dark:bg-[rgba(0,0,0,0.65)]",
                                "p-4",
                                "backdrop-blur-[4px]"
                            )}
                            role="dialog"
                            aria-modal="true"
                            aria-label={t::TREND_TITLE}
                            onclick={close_trend_click.clone()}
                        >
                            <section
                                class={classes!(
                                    "w-full",
                                    "max-w-[920px]",
                                    "max-h-[88vh]",
                                    "overflow-auto",
                                    "rounded-[12px]",
                                    "border",
                                    "border-[var(--border-strong)]",
                                    "bg-[var(--surface)]",
                                    "px-4",
                                    "py-4",
                                    "shadow-[var(--shadow-lg)]",
                                    "sm:px-6",
                                    "sm:py-5"
                                )}
                                onclick={stop_trend_bubble.clone()}
                            >
                                <div class={classes!(
                                    "mb-4",
                                    "flex",
                                    "items-start",
                                    "justify-between",
                                    "gap-3",
                                    "border-b",
                                    "border-[var(--border)]",
                                    "pb-3.5"
                                )}>
                                    <div class={classes!("flex", "flex-col", "gap-1")}>
                                        <div class="sf-eyebrow">{ "READERSHIP ANALYTICS" }</div>
                                        <h2 class={classes!("m-0", "font-serif", "font-normal", "text-[1.35rem]", "text-[var(--text)]")}>{ t::TREND_TITLE }</h2>
                                        <p class={classes!("m-0", "text-sm", "text-[var(--muted)]")}>{ t::TREND_SUBTITLE }</p>
                                        <p class={classes!("m-0", "font-mono", "text-xs", "tabular-nums", "text-[var(--faint)]")}>
                                            {
                                                if let Some(total) = *view_total {
                                                    fill_one(t::TREND_TOTAL_TEMPLATE, total)
                                                } else {
                                                    t::VIEW_COUNT_LOADING.to_string()
                                                }
                                            }
                                            {
                                                if let Some(today) = *view_today {
                                                    format!(" · 今日 {}", today)
                                                } else {
                                                    String::new()
                                                }
                                            }
                                        </p>
                                    </div>
                                    <button
                                        type="button"
                                        class={classes!("sf-btn", "sf-btn--secondary", "sf-btn--sm")}
                                        aria-label={t::TREND_CLOSE_ARIA}
                                        onclick={close_trend_click.clone()}
                                    >
                                        { t::CLOSE_BRIEF_BUTTON }
                                    </button>
                                </div>

                                <div class={classes!("mb-4", "flex", "items-center", "gap-2", "flex-wrap")}>
                                    <button
                                        type="button"
                                        class={classes!(
                                            "sf-btn",
                                            "sf-btn--sm",
                                            if *trend_granularity == TrendGranularity::Day {
                                                classes!("sf-btn--primary")
                                            } else {
                                                classes!("sf-btn--secondary")
                                            }
                                        )}
                                        onclick={switch_trend_to_day}
                                    >
                                        { t::TREND_TAB_DAY }
                                    </button>
                                    <button
                                        type="button"
                                        class={classes!(
                                            "sf-btn",
                                            "sf-btn--sm",
                                            if *trend_granularity == TrendGranularity::Hour {
                                                classes!("sf-btn--primary")
                                            } else {
                                                classes!("sf-btn--secondary")
                                            }
                                        )}
                                        onclick={switch_trend_to_hour}
                                    >
                                        { t::TREND_TAB_HOUR }
                                    </button>
                                </div>

                                if *trend_granularity == TrendGranularity::Hour {
                                    <div class={classes!("mb-4", "flex", "items-center", "gap-2.5")}>
                                        <label class={classes!("text-sm", "text-[var(--muted)]")}>{ t::TREND_SELECT_DAY }</label>
                                        <select
                                            class={classes!(
                                                "sf-input",
                                                "!h-9",
                                                "w-auto",
                                                "font-mono",
                                                "text-xs"
                                            )}
                                            onchange={on_trend_day_change}
                                            value={(*trend_selected_day).clone().unwrap_or_default()}
                                        >
                                            { for (*trend_day_options).iter().map(|day| {
                                                html! { <option value={day.clone()}>{ day.clone() }</option> }
                                            }) }
                                        </select>
                                    </div>
                                }

                                {
                                    if *trend_loading {
                                        html! {
                                            <div class={classes!(
                                                "rounded-[10px]",
                                                "border",
                                                "border-[var(--border)]",
                                                "bg-[var(--surface-alt)]",
                                                "px-4",
                                                "py-8",
                                                "text-center",
                                                "text-sm",
                                                "text-[var(--muted)]"
                                            )}>
                                                <i class={classes!("fas", "fa-spinner", "fa-spin", "mr-2")}></i>
                                                { t::TREND_LOADING }
                                            </div>
                                        }
                                    } else if let Some(error) = (*trend_error).clone() {
                                        html! {
                                            <div class={classes!(
                                                "rounded-[10px]",
                                                "border",
                                                "border-[var(--err)]/35",
                                                "bg-[var(--err-soft)]",
                                                "px-4",
                                                "py-3",
                                                "text-sm",
                                                "text-[var(--err)]"
                                            )}>
                                                { error }
                                            </div>
                                        }
                                    } else {
                                        html! {
                                            <ViewTrendChart
                                                points={(*trend_points).clone()}
                                                empty_text={t::TREND_EMPTY.to_string()}
                                            />
                                        }
                                    }
                                }
                            </section>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            // Hide scroll-to-top button and TOC button when overlay is open
            if !is_overlay_open {
                <ScrollToTopButton />
                <TocButton />
                <ReadingProgress />
            }
        </main>
    }
}
