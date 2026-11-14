use gloo_events::EventListener;
use web_sys::window;
use yew::prelude::*;

use crate::{
    components::{
        icons::{Icon, IconName},
        tooltip::{Tooltip, TooltipPosition},
    },
    i18n::current::scroll_to_top as t,
};

#[function_component(ScrollToTopButton)]
pub fn scroll_to_top_button() -> Html {
    let show = use_state(|| false);

    {
        let show = show.clone();
        use_effect_with((), move |_| {
            let listener = window().map(|win| {
                let sync = {
                    let show = show.clone();
                    let win = win.clone();
                    move || {
                        show.set(win.scroll_y().unwrap_or(0.0) > 400.0);
                    }
                };
                sync();
                EventListener::new(&win, "scroll", move |_| sync())
            });
            move || drop(listener)
        });
    }

    let onclick = Callback::from(|_: MouseEvent| {
        if let Some(win) = window() {
            let options = web_sys::ScrollToOptions::new();
            options.set_behavior(web_sys::ScrollBehavior::Smooth);
            options.set_top(0.0);
            options.set_left(0.0);
            win.scroll_with_scroll_to_options(&options);
        }
    });

    let visibility =
        if *show { "opacity-100 pointer-events-auto" } else { "opacity-0 pointer-events-none" };

    html! {
        <div class={classes!(
            "fixed", "right-8", "bottom-8", "z-50",
            "max-md:bottom-6", "max-md:right-6",
            "transition-opacity", "duration-200",
            visibility
        )}>
            <Tooltip text={t::TOOLTIP.to_string()} position={TooltipPosition::Top}>
                <button
                    type="button"
                    {onclick}
                    aria-label={t::TOOLTIP}
                    class={classes!(
                        "w-10", "h-10", "rounded-full",
                        "border", "border-[var(--border-strong)]",
                        "bg-[var(--surface)]",
                        "text-[var(--muted)]",
                        "inline-flex", "items-center", "justify-center",
                        "hover:border-[var(--primary)]",
                        "hover:text-[var(--primary)]",
                        "transition-colors", "duration-150",
                        "cursor-pointer"
                    )}
                >
                    <Icon name={IconName::ArrowUp} size={18} />
                </button>
            </Tooltip>
        </div>
    }
}
