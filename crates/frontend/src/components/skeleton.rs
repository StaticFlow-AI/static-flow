//! Loading skeletons that reserve final layout space (zero CLS).
//!
//! Aligned with the warm parchment / obsidian editorial design system:
//! borderless article footprints, hairline-only surfaces, zero drop shadows,
//! and reduced-motion-safe shimmer defined by `.skeleton` in `input.css`.

use yew::prelude::*;

/// Props for [`SkeletonBlock`].
#[allow(
    dead_code,
    reason = "The standalone block is part of the skeleton kit's public surface; page-specific \
              skeletons compose it during the feedback-adoption phase."
)]
#[derive(Properties, PartialEq)]
pub struct SkeletonBlockProps {
    /// Extra classes controlling the block's size/shape (width, height,
    /// rounding); the base `.skeleton` class supplies warm color and shimmer.
    #[prop_or_default]
    pub class: Classes,
}

/// One shimmering placeholder rectangle.
#[function_component(SkeletonBlock)]
pub fn skeleton_block(props: &SkeletonBlockProps) -> Html {
    html! { <div class={classes!("skeleton", props.class.clone())} aria-hidden="true"></div> }
}

/// Props for [`SkeletonText`].
#[derive(Properties, PartialEq)]
pub struct SkeletonTextProps {
    /// Number of text lines to render; the last line is shortened.
    #[prop_or(3)]
    pub lines: u8,
}

/// A paragraph-shaped stack of shimmer lines.
#[function_component(SkeletonText)]
pub fn skeleton_text(props: &SkeletonTextProps) -> Html {
    let lines = props.lines.max(1);
    html! {
        <div class={classes!("flex", "flex-col", "gap-2")} aria-hidden="true">
            { for (0..lines).map(|i| {
                let width = if i + 1 == lines { "w-3/5" } else { "w-full" };
                html! { <div class={classes!("skeleton", "h-3.5", "rounded-[4px]", width)}></div> }
            }) }
        </div>
    }
}

/// Editorial article placeholder matching the borderless `ArticleCard`:
/// 16:10 framed cover, eyebrow line, serif title bar, and two summary lines.
#[function_component(SkeletonCard)]
pub fn skeleton_card() -> Html {
    html! {
        <div class={classes!("flex", "flex-col", "gap-3")} aria-hidden="true">
            <div class={classes!(
                "skeleton", "aspect-[16/10]", "w-full",
                "rounded-[10px]", "border", "border-[var(--border)]"
            )}></div>
            <div class={classes!("skeleton", "h-3", "w-28", "rounded-[4px]")}></div>
            <div class={classes!("skeleton", "h-5", "w-4/5", "rounded-[4px]")}></div>
            <SkeletonText lines={2} />
        </div>
    }
}

/// Music song placeholder: square 1:1 cover with hairline border and two text
/// lines.
#[function_component(SkeletonSongCard)]
pub fn skeleton_song_card() -> Html {
    html! {
        <div class={classes!("flex", "flex-col", "gap-2.5")} aria-hidden="true">
            <div class={classes!(
                "skeleton", "aspect-square", "w-full",
                "rounded-[8px]", "border", "border-[var(--border)]"
            )}></div>
            <div class={classes!("skeleton", "h-4", "w-4/5", "rounded-[4px]")}></div>
            <div class={classes!("skeleton", "h-3", "w-1/2", "rounded-[4px]")}></div>
        </div>
    }
}

/// Reading-column skeleton for the article detail page: eyebrow, display title,
/// italic summary, hairline divider, cover frame, and prose paragraphs.
#[function_component(SkeletonArticle)]
pub fn skeleton_article() -> Html {
    html! {
        <div class={classes!("flex", "flex-col", "gap-5", "w-full", "max-w-[760px]", "mx-auto")} aria-hidden="true">
            <div class={classes!("skeleton", "h-3", "w-36", "rounded-[4px]")}></div>
            <div class={classes!("skeleton", "h-9", "w-4/5", "rounded-[6px]")}></div>
            <div class={classes!("skeleton", "h-5", "w-2/3", "rounded-[4px]")}></div>
            <hr class="sf-rule" />
            <div class={classes!(
                "skeleton", "aspect-[16/10]", "w-full",
                "rounded-[12px]", "border", "border-[var(--border)]"
            )}></div>
            <SkeletonText lines={4} />
            <SkeletonText lines={5} />
            <SkeletonText lines={3} />
        </div>
    }
}
