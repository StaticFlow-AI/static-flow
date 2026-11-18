use wasm_bindgen::{closure::Closure, prelude::*};
use yew::prelude::*;

use crate::pages::worldlines::Walker;

#[wasm_bindgen(module = "/spatial/bridge.js")]
extern "C" {
    type SpaceView;
    #[wasm_bindgen(js_name = startSpace)]
    fn start_space(
        host: &web_sys::HtmlElement,
        base: &str,
        kind: &str,
        emit: &Closure<dyn Fn(String, String)>,
        walker: JsValue,
        initial: &str,
    ) -> SpaceView;
    #[wasm_bindgen(method)]
    fn sync(this: &SpaceView, state: &str);
    #[wasm_bindgen(method)]
    fn dispose(this: &SpaceView);
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub kind: &'static str,
    pub state: String,
    pub event: Callback<(String, String)>,
}

pub enum Message {
    Event(String, String),
}
type SceneCallback = Closure<dyn Fn(String, String)>;

pub struct SpatialSurface {
    host: NodeRef,
    mounted: Option<(SpaceView, SceneCallback)>,
}
impl Component for SpatialSurface {
    type Message = Message;
    type Properties = Props;
    fn create(_: &Context<Self>) -> Self {
        Self {
            host: NodeRef::default(),
            mounted: None,
        }
    }
    fn view(&self, _: &Context<Self>) -> Html {
        html! { <div class="space-canvas" ref={self.host.clone()} /> }
    }
    fn rendered(&mut self, ctx: &Context<Self>, first: bool) {
        if !first {
            return;
        }
        let Some(host) = self.host.cast::<web_sys::HtmlElement>() else { return };
        let link = ctx.link().clone();
        let emit =
            Closure::new(move |topic, value| link.send_message(Message::Event(topic, value)));
        let p = ctx.props();
        let walker = if p.kind == "lab" { JsValue::from(Walker::default()) } else { JsValue::NULL };
        let view = start_space(
            &host,
            &crate::config::asset_path("static/sf-space/"),
            p.kind,
            &emit,
            walker,
            &p.state,
        );
        self.mounted = Some((view, emit));
    }
    fn changed(&mut self, ctx: &Context<Self>, old: &Props) -> bool {
        if old.state != ctx.props().state {
            if let Some((view, _)) = &self.mounted {
                view.sync(&ctx.props().state);
            }
        }
        false
    }
    fn update(&mut self, ctx: &Context<Self>, Message::Event(topic, value): Message) -> bool {
        ctx.props().event.emit((topic, value));
        false
    }
    fn destroy(&mut self, _: &Context<Self>) {
        if let Some((view, _emit)) = self.mounted.take() {
            view.dispose();
        }
    }
}
