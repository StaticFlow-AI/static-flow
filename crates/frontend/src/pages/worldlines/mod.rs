//! First-person voxel sandbox at the Future Gadget Laboratory.
mod catalog;
mod lighting;
mod mobs;
mod player;
mod sandbox;
mod story;
mod survey;
mod terrain;
mod water;
mod weather;

use wasm_bindgen::{closure::Closure, prelude::*};
use yew::prelude::*;
use yew_router::prelude::Link;

use self::sandbox::Sandbox;
use crate::router::Route;

#[wasm_bindgen]
extern "C" {
    // wasm-bindgen emits this accessor in the entry module, including Trunk's
    // public base and hash.
    #[wasm_bindgen(thread_local_v2, js_namespace = ["import", "meta"], js_name = url)]
    static BINDINGS_URL: String;
}

#[wasm_bindgen(module = "/spatial/bridge.js")]
extern "C" {
    type GameView;
    #[wasm_bindgen(js_name=startGame)]
    fn start_game(
        host: &web_sys::HtmlElement,
        base: &str,
        game: JsValue,
        module: JsValue,
        bindings_url: &str,
        notify: &Closure<dyn Fn(String)>,
    ) -> GameView;
    #[wasm_bindgen(method)]
    fn dispose(this: &GameView);
}

#[function_component(WorldlinesGame)]
pub fn worldlines_game() -> Html {
    let host = use_node_ref();
    let status = use_state(|| "loading".to_owned());
    {
        let host = host.clone();
        let status = status.clone();
        use_effect_with((), move |()| {
            let emit = Closure::<dyn Fn(String)>::new(move |state| status.set(state));
            let view = host.cast::<web_sys::HtmlElement>().map(|element| {
                let seed = js_sys::Date::now() as u64 as u32;
                start_game(
                    &element,
                    &crate::config::asset_path("static/sf-space/"),
                    JsValue::from(Sandbox::new(seed)),
                    wasm_bindgen::module(),
                    &BINDINGS_URL.with(Clone::clone),
                    &emit,
                )
            });
            move || {
                if let Some(view) = view {
                    view.dispose();
                }
                drop(emit);
            }
        });
    }
    html! {<main class="block-lab" aria-label="方块研究所">
        <div class="block-viewport" ref={host}/>
        <Link<Route> classes="block-back" to={Route::Home}>{"← 博客"}</Link<Route>>
        {match status.as_str(){
            "loading"=>html!{<div class="block-load" role="status">{"正在搭建方块世界…"}</div>},
            "error"=>html!{<div class="block-load" role="alert"><strong>{"无法绘制方块世界"}</strong><p>{"请开启浏览器的 WebGL 2 和硬件加速后重新载入。"}</p><button onclick={Callback::from(|_|{if let Some(w)=web_sys::window(){let _=w.location().reload();}})}>{"重新载入"}</button></div>},
            _=>Html::default(),
        }}
    </main>}
}
