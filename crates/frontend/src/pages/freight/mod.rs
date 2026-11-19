//! First-person team match on a container ship.
mod map;
mod matchplay;

use matchplay::FreightMatch;
use wasm_bindgen::{closure::Closure, prelude::*};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::router::Route;

#[wasm_bindgen(module = "/spatial/bridge.js")]
extern "C" {
    type ArenaView;
    #[wasm_bindgen(js_name=startFreight)]
    fn start_freight(
        host: &web_sys::HtmlElement,
        base: &str,
        sim: JsValue,
        notify: &Closure<dyn Fn(String)>,
    ) -> ArenaView;
    #[wasm_bindgen(method)]
    fn dispose(this: &ArenaView);
}
#[function_component(FreightGame)]
pub fn freight_game() -> Html {
    let host = use_node_ref();
    let status = use_state(|| "loading".to_owned());
    {
        let host = host.clone();
        let status = status.clone();
        use_effect_with((), move |()| {
            let emit = Closure::<dyn Fn(String)>::new(move |state| status.set(state));
            let view = host.cast::<web_sys::HtmlElement>().map(|el| {
                let seed = js_sys::Date::now() as u64 as u32;
                start_freight(
                    &el,
                    &crate::config::asset_path("static/"),
                    JsValue::from(FreightMatch::new(seed)),
                    &emit,
                )
            });
            move || {
                if let Some(v) = view {
                    v.dispose()
                }
                drop(emit);
            }
        });
    }
    html! {<main class="freight-shell" aria-label="运输船 · 甲板交锋">
        <link rel="stylesheet" href={crate::config::asset_path("static/freight/arena.css")}/>
        <div class="freight-stage" ref={host}/>
        <Link<Route> classes="freight-back" to={Route::Home}>{"‹ 博客"}</Link<Route>>
        {match status.as_str(){
            "loading"=>html!{<div class="freight-load" role="status">{"正在载入运输船…"}</div>},
            "error"=>html!{<div class="freight-load" role="alert"><strong>{"甲板场景加载失败"}</strong><p>{"请确认 WebGL 2 可用，然后重新载入游戏资源。"}</p><button onclick={Callback::from(|_|{if let Some(w)=web_sys::window(){let _=w.location().reload();}})}>{"重新载入"}</button></div>},
            _=>Html::default()
        }}
    </main>}
}
