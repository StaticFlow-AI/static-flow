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

    // Shares the bottom-right floating stack with the TOC button (see the
    // `.sf-fab` rules in input.css): 40px, safe-area aware, lifted above the
    // MiniPlayer, and hidden near the top of the page.
    html! {
        <div class={classes!("sf-fab", "sf-fab--top", (*show).then_some("is-visible"))}>
            <Tooltip text={t::TOOLTIP.to_string()} position={TooltipPosition::Left}>
                <button
                    type="button"
                    {onclick}
                    aria-label={t::TOOLTIP}
                    tabindex={if *show { "0" } else { "-1" }}
                    class="sf-fab__btn"
                >
                    <Icon name={IconName::ArrowUp} size={18} />
                </button>
            </Tooltip>
        </div>
    }
}
