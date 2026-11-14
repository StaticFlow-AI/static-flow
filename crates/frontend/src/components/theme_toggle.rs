use std::sync::atomic::{AtomicUsize, Ordering};

use gloo_events::EventListener;
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;

use crate::i18n::current::theme_toggle as t;

/// Window event dispatched synchronously by `window.__toggleTheme` (see
/// `index.html`) on every click with `{ detail: { theme: desiredTheme } }`.
/// Reading `detail.theme` first keeps every mounted toggle icon in zero-latency
/// lockstep even across rapid multi-clicks during an active ViewTransition.
const THEME_CHANGE_EVENT: &str = "sf-theme-change";

static NEXT_MASK_ID: AtomicUsize = AtomicUsize::new(0);

const SUN_RAYS: [(&str, &str, &str, &str, &str); 8] = [
    ("12", "2.2", "12", "4.4", "--ray-idx: 0;"),
    ("17.37", "6.63", "18.93", "5.07", "--ray-idx: 1;"),
    ("19.6", "12", "21.8", "12", "--ray-idx: 2;"),
    ("17.37", "17.37", "18.93", "18.93", "--ray-idx: 3;"),
    ("12", "19.6", "12", "21.8", "--ray-idx: 4;"),
    ("6.63", "17.37", "5.07", "18.93", "--ray-idx: 5;"),
    ("2.2", "12", "4.4", "12", "--ray-idx: 6;"),
    ("6.63", "6.63", "5.07", "5.07", "--ray-idx: 7;"),
];

fn is_dark_theme() -> bool {
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.document_element())
        .and_then(|el| el.get_attribute("data-theme"))
        .map(|theme| theme.eq_ignore_ascii_case("dark"))
        .unwrap_or(false)
}

fn theme_from_event_or_dom(event: &web_sys::Event) -> bool {
    if let Ok(detail) = js_sys::Reflect::get(event.as_ref(), &JsValue::from_str("detail")) {
        if !detail.is_null() && !detail.is_undefined() {
            if let Ok(theme_val) = js_sys::Reflect::get(&detail, &JsValue::from_str("theme")) {
                if let Some(theme_str) = theme_val.as_string() {
                    return theme_str.eq_ignore_ascii_case("dark");
                }
            }
        }
    }
    is_dark_theme()
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
    let mask_id =
        use_state(|| format!("sf-theme-mask-{}", NEXT_MASK_ID.fetch_add(1, Ordering::Relaxed)));
    let button_ref = use_node_ref();

    {
        let theme_state = theme_state.clone();
        use_effect_with((), move |_| {
            theme_state.set(is_dark_theme());
            let listener = web_sys::window().map(|win| {
                EventListener::new(&win, THEME_CHANGE_EVENT, move |event| {
                    theme_state.set(theme_from_event_or_dom(event));
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
            let origin_el = button_ref
                .cast::<web_sys::Element>()
                .map(JsValue::from)
                .unwrap_or(JsValue::NULL);
            let _ = toggle.call2(&JsValue::NULL, event.as_ref(), &origin_el);
        })
    };

    let is_dark = *theme_state;
    let label = if is_dark { t::SWITCH_TO_LIGHT } else { t::SWITCH_TO_DARK };
    let core_r = if is_dark { "8" } else { "5" };
    let core_style = if is_dark { "r: 8px;" } else { "r: 5px;" };
    let cutout_style =
        if is_dark { "transform: translateX(-9px);" } else { "transform: translateX(0px);" };
    let mask_url = format!("url(#{})", *mask_id);

    let button_class =
        classes!("sf-theme-toggle", is_dark.then_some("sf-theme-toggle--dark"), class.clone());

    html! {
        <button
            ref={button_ref}
            type="button"
            class={button_class}
            {onclick}
            aria-label={label}
            title={label}
            aria-pressed={is_dark.to_string()}
        >
            <svg
                class="sf-theme-toggle__svg"
                viewBox="0 0 24 24"
                fill="none"
                aria-hidden="true"
                focusable="false"
            >
                <defs>
                    <mask id={(*mask_id).clone()}>
                        <rect x="0" y="0" width="24" height="24" fill="white" />
                        <circle
                            class="sf-theme-toggle__cutout"
                            cx="24"
                            cy="9"
                            r="7"
                            fill="black"
                            style={cutout_style}
                        />
                    </mask>
                </defs>
                <circle
                    class="sf-theme-toggle__core"
                    cx="12"
                    cy="12"
                    r={core_r}
                    style={core_style}
                    fill="currentColor"
                    mask={mask_url}
                />
                <g stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
                    { for SUN_RAYS.iter().map(|(x1, y1, x2, y2, style)| html! {
                        <line
                            class="sf-theme-toggle__ray"
                            x1={*x1}
                            y1={*y1}
                            x2={*x2}
                            y2={*y2}
                            style={*style}
                        />
                    }) }
                </g>
            </svg>
            <span class="sr-only">{ label }</span>
        </button>
    }
}
