use js_sys::Reflect;
use wasm_bindgen::{JsCast, JsValue};
use yew::prelude::*;

use crate::{
    components::{
        icons::{Icon, IconName},
        tooltip::{Tooltip, TooltipPosition},
    },
    i18n::current::toc_button as t,
};

/// 目录移动端切换按钮组件（温暖细线无阴影风格）
#[function_component(TocButton)]
pub fn toc_button() -> Html {
    let onclick = Callback::from(|_: MouseEvent| {
        if let Some(window) = web_sys::window() {
            if let Some(existing) = window
                .document()
                .and_then(|d| d.query_selector(".article-toc").ok())
                .flatten()
            {
                let class_list = existing.class_list();
                if class_list.contains("mobile-open") {
                    let _ = class_list.remove_1("mobile-open");
                } else {
                    let _ = class_list.add_1("mobile-open");
                }
                return;
            }

            if let Ok(gen) = Reflect::get(&window, &JsValue::from_str("generateTOC")) {
                if let Ok(func) = gen.dyn_into::<js_sys::Function>() {
                    let _ = func.call0(&window);
                }
            }

            if let Some(toc) = window
                .document()
                .and_then(|d| d.query_selector(".article-toc").ok())
                .flatten()
            {
                let class_list = toc.class_list();
                if class_list.contains("mobile-open") {
                    let _ = class_list.remove_1("mobile-open");
                } else {
                    let _ = class_list.add_1("mobile-open");
                }
            }
        }
    });

    // Stacked above the scroll-to-top button in the bottom-right corner; the
    // position is owned by the `.sf-fab` rules in input.css.
    html! {
        <div class={classes!("mobile-toc-fab", "sf-fab", "sf-fab--toc")}>
            <Tooltip text={t::TOOLTIP.to_string()} position={TooltipPosition::Left}>
                <button
                    type="button"
                    {onclick}
                    aria-label={t::TOOLTIP}
                    class="sf-fab__btn"
                >
                    <Icon name={IconName::List} size={18} />
                </button>
            </Tooltip>
        </div>
    }
}
