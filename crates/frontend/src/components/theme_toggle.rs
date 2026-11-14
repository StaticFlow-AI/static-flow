use gloo_events::EventListener;
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;

use crate::i18n::current::theme_toggle as t;

/// Window event dispatched by `window.__toggleTheme` (see `index.html`) right
/// after `data-theme` is mutated. With View Transitions the mutation happens
/// asynchronously inside the transition callback, so every mounted toggle
/// (desktop + mobile header) re-reads the theme from this event instead of
/// guessing at click time.
const THEME_CHANGE_EVENT: &str = "sf-theme-change";

fn is_dark_theme() -> bool {
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.document_element())
        .and_then(|el| el.get_attribute("data-theme"))
        .map(|theme| theme.eq_ignore_ascii_case("dark"))
        .unwrap_or(false)
}

#[derive(Properties, PartialEq)]
pub struct ThemeToggleProps {
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(ThemeToggle)]
pub fn theme_toggle(props: &ThemeToggleProps) -> Html {
    let ThemeToggleProps {
        class,
    } = props;
    let theme_state = use_state(is_dark_theme);
    let button_ref = use_node_ref();

    {
        let theme_state = theme_state.clone();
        use_effect_with((), move |_| {
            // Re-sync once on mount in case the theme flipped between the
            // initial state read and the effect running.
            theme_state.set(is_dark_theme());
            let listener = web_sys::window().map(|win| {
                EventListener::new(&win, THEME_CHANGE_EVENT, move |_| {
                    theme_state.set(is_dark_theme());
                })
            });
            move || drop(listener)
        });
    }

    let onclick = {
        let button_ref = button_ref.clone();
        Callback::from(move |event: MouseEvent| {
            let Some(win) = web_sys::window() else {
                return;
            };
            let Some(toggle) = js_sys::Reflect::get(&win, &JsValue::from_str("__toggleTheme"))
                .ok()
                .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            else {
                return;
            };
            // Yew delegates events at the app root, so `event.currentTarget`
            // is not this button by the time JS sees it. Hand the button over
            // explicitly so the reveal circle is anchored on its center.
            let origin_el = button_ref
                .cast::<web_sys::Element>()
                .map(JsValue::from)
                .unwrap_or(JsValue::NULL);
            let _ = toggle.call2(&JsValue::NULL, event.as_ref(), &origin_el);
        })
    };

    let label = if *theme_state { t::SWITCH_TO_LIGHT } else { t::SWITCH_TO_DARK };

    // Dark active -> reveal the sun (clicking goes to light); light active ->
    // reveal the moon. The hidden glyph rotates + scales out while the visible
    // one rotates + scales in, so the theme swap cross-fades on-token instead of
    // hard-cutting one FontAwesome character to another.
    let (sun_anim, moon_anim) = if *theme_state {
        ("opacity-100 rotate-0 scale-100", "opacity-0 -rotate-90 scale-50")
    } else {
        ("opacity-0 rotate-90 scale-50", "opacity-100 rotate-0 scale-100")
    };

    let button_class = classes!(
        "group",
        "sf-theme-toggle",
        "btn-fluent-icon",
        "border",
        "border-[var(--border)]",
        "bg-transparent",
        "hover:bg-[var(--surface-alt)]",
        "transition-all",
        "duration-[var(--motion-fast)]",
        "ease-[var(--ease-snap)]",
        class.clone()
    );

    html! {
        <button
            ref={button_ref}
            type="button"
            class={button_class}
            {onclick}
            aria-label={label}
            title={label}
            aria-pressed={(*theme_state).to_string()}
        >
            <span class="relative inline-flex h-[1.25em] w-[1.25em] items-center justify-center">
                <i
                    class={classes!(
                        "fas", "fa-sun", "fa-lg", "absolute",
                        "transition-all", "duration-[var(--motion-base)]", "ease-[var(--ease-spring)]",
                        "text-[var(--text)]", "group-hover:text-[var(--primary)]",
                        sun_anim
                    )}
                    aria-hidden="true"
                ></i>
                <i
                    class={classes!(
                        "fas", "fa-moon", "fa-lg", "absolute",
                        "transition-all", "duration-[var(--motion-base)]", "ease-[var(--ease-spring)]",
                        "text-[var(--text)]", "group-hover:text-[var(--primary)]",
                        moon_anim
                    )}
                    aria-hidden="true"
                ></i>
            </span>
            <span class="sr-only">{ label }</span>
        </button>
    }
}
