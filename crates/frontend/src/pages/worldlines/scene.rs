use gloo_events::EventListener;
use gloo_render::{request_animation_frame, AnimationFrame};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlCanvasElement, IntersectionObserver, IntersectionObserverEntry, ResizeObserver};
use yew::{html::Scope, prelude::*};

use super::{
    puzzle::{divergence, is_aligned, STATIONS},
    renderer::{Camera, Renderer},
};

const INITIAL_VIEW: [f32; 2] = [-0.68, 0.28];

#[derive(Properties, PartialEq)]
pub(super) struct Props {
    pub gates: u8,
    #[prop_or_default]
    pub on_send: Option<Callback<usize>>,
    #[prop_or_default]
    pub last_mail: Option<usize>,
    #[prop_or_default]
    pub finished: bool,
}

pub(super) enum Msg {
    InView(bool),
    TabVisible(bool),
    ReducedMotion(bool),
    Resize,
    Frame(f64),
    ContextLost(bool),
    Grab(PointerEvent),
    Move(PointerEvent),
    Release(i32),
    Key(KeyboardEvent),
    Reset,
}

/// Observer callbacks and browser subscriptions share the canvas lifetime.
struct Observers {
    intersection: Option<IntersectionObserver>,
    resize: Option<ResizeObserver>,
    _intersection_callback: Closure<dyn Fn(js_sys::Array)>,
    _resize_callback: Closure<dyn Fn()>,
    _events: Vec<EventListener>,
}

impl Observers {
    fn attach(canvas: &HtmlCanvasElement, link: &Scope<FieldCanvas>) -> Self {
        let visibility = link.callback(Msg::InView);
        let on_intersection = Closure::new(move |entries: js_sys::Array| {
            if let Ok(entry) = entries.get(0).dyn_into::<IntersectionObserverEntry>() {
                visibility.emit(entry.is_intersecting());
            }
        });
        let intersection = IntersectionObserver::new(on_intersection.as_ref().unchecked_ref()).ok();
        if let Some(observer) = &intersection {
            observer.observe(canvas);
        } else {
            link.send_message(Msg::InView(true));
        }
        let resized = link.callback(|()| Msg::Resize);
        let on_resize = Closure::new(move || resized.emit(()));
        let resize = ResizeObserver::new(on_resize.as_ref().unchecked_ref()).ok();
        if let Some(observer) = &resize {
            observer.observe(canvas);
        }
        let window = gloo_utils::window();
        let doc = gloo_utils::document();
        let tab = link.callback(Msg::TabVisible);
        let watch_document = doc.clone();
        let lost = link.callback(Msg::ContextLost);
        let restored = lost.clone();
        let resized = link.callback(|()| Msg::Resize);
        let mut events = vec![
            EventListener::new(&doc, "visibilitychange", move |_| {
                tab.emit(!watch_document.hidden())
            }),
            EventListener::new(&window, "resize", move |_| resized.emit(())),
            EventListener::new_with_options(
                canvas,
                "webglcontextlost",
                gloo_events::EventListenerOptions::enable_prevent_default(),
                move |event| {
                    event.prevent_default();
                    lost.emit(true);
                },
            ),
            EventListener::new(canvas, "webglcontextrestored", move |_| restored.emit(false)),
        ];
        if let Ok(Some(query)) = window.match_media("(prefers-reduced-motion: reduce)") {
            link.send_message(Msg::ReducedMotion(query.matches()));
            let motion = link.callback(Msg::ReducedMotion);
            let setting = query.clone();
            events.push(EventListener::new(&query, "change", move |_| {
                motion.emit(setting.matches())
            }));
        }
        Self {
            intersection,
            resize,
            _intersection_callback: on_intersection,
            _resize_callback: on_resize,
            _events: events,
        }
    }
}

impl Drop for Observers {
    fn drop(&mut self) {
        if let Some(observer) = &self.intersection {
            observer.disconnect();
        }
        if let Some(observer) = &self.resize {
            observer.disconnect();
        }
    }
}

pub(super) struct FieldCanvas {
    canvas: NodeRef,
    observers: Option<Observers>,
    renderer: Option<Renderer>,
    frame: Option<AnimationFrame>,
    in_view: bool,
    tab_visible: bool,
    reduced_motion: bool,
    unavailable: bool,
    density: f32,
    aspect: f32,
    last_frame: Option<f64>,
    clock: f32,
    mail_time: f32,
    gates: [f32; 4],
    orbit: [f32; 2],
    camera: Camera,
    drag: Option<(i32, i32, i32)>,
}

fn weights(gates: u8) -> [f32; 4] {
    std::array::from_fn(|i| f32::from(u8::from(is_aligned(gates, i))))
}

impl FieldCanvas {
    fn request_frame(&mut self, link: &Scope<Self>) {
        if !self.in_view || !self.tab_visible || self.unavailable {
            self.frame = None;
            self.last_frame = None;
        } else if self.frame.is_none() {
            let link = link.clone();
            self.frame =
                Some(request_animation_frame(move |time| link.send_message(Msg::Frame(time))));
        }
    }

    fn resize(&mut self, canvas: &HtmlCanvasElement) {
        self.density = gloo_utils::window().device_pixel_ratio().clamp(1.0, 1.5) as f32;
        let width = (canvas.client_width().max(1) as f32 * self.density).round() as u32;
        let height = (canvas.client_height().max(1) as f32 * self.density).round() as u32;
        if canvas.width() != width {
            canvas.set_width(width);
        }
        if canvas.height() != height {
            canvas.set_height(height);
        }
        self.aspect = width as f32 / height as f32;
        self.camera = Camera::new(self.orbit, self.aspect);
    }

    fn draw(&mut self, ctx: &Context<Self>, time: f64) -> bool {
        self.frame = None;
        let first_frame = self.renderer.is_none();
        if first_frame {
            let Some(canvas) = self.canvas.cast::<HtmlCanvasElement>() else { return false };
            self.resize(&canvas);
            match Renderer::new(&canvas) {
                Ok(renderer) => self.renderer = Some(renderer),
                Err(reason) => {
                    web_sys::console::warn_2(&"Worldline field".into(), &reason);
                    self.unavailable = true;
                    return true;
                },
            }
        }
        let delta = ((time - self.last_frame.replace(time).unwrap_or(time)) / 1000.0)
            .clamp(0.0, 0.08) as f32;
        let mix = if self.reduced_motion { 1.0 } else { 1.0 - (-9.0 * delta).exp() };
        for (value, target) in self.gates.iter_mut().zip(weights(ctx.props().gates)) {
            *value += (target - *value) * mix;
        }
        if !self.reduced_motion {
            self.clock += delta;
        }
        if let Some(renderer) = &self.renderer {
            let mail = match ctx.props().last_mail {
                Some(i) if !self.reduced_motion => {
                    [(i + 1) as f32 / 5.0, self.clock - self.mail_time]
                },
                _ => [0.0, 2.0],
            };
            renderer.draw(&self.camera, &self.gates, self.clock, self.density, mail);
        }
        if !self.reduced_motion {
            self.request_frame(ctx.link());
        }
        first_frame
    }
}

impl Component for FieldCanvas {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            canvas: NodeRef::default(),
            observers: None,
            renderer: None,
            frame: None,
            in_view: false,
            tab_visible: !gloo_utils::document().hidden(),
            reduced_motion: false,
            unavailable: false,
            density: 1.0,
            aspect: 1.0,
            last_frame: None,
            clock: 0.0,
            mail_time: 0.0,
            gates: weights(ctx.props().gates),
            orbit: INITIAL_VIEW,
            camera: Camera::new(INITIAL_VIEW, 1.0),
            drag: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first: bool) {
        if first {
            if let Some(canvas) = self.canvas.cast::<HtmlCanvasElement>() {
                self.observers = Some(Observers::attach(&canvas, ctx.link()));
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, previous: &Props) -> bool {
        if ctx.props().gates != previous.gates && ctx.props().last_mail.is_some() {
            self.mail_time = self.clock;
        }
        self.request_frame(ctx.link());
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let orbit_changed = matches!(msg, Msg::Move(_) | Msg::Key(_) | Msg::Reset);
        let layout_changed = matches!(msg, Msg::Resize);
        match msg {
            Msg::Frame(time) => return self.draw(ctx, time),
            Msg::InView(visible) => self.in_view = visible,
            Msg::TabVisible(visible) => self.tab_visible = visible,
            Msg::ReducedMotion(reduce) => self.reduced_motion = reduce,
            Msg::Resize => {
                if self.renderer.is_some() {
                    if let Some(canvas) = self.canvas.cast::<HtmlCanvasElement>() {
                        self.resize(&canvas);
                    }
                }
            },
            Msg::ContextLost(lost) => {
                self.renderer = None;
                self.unavailable = lost;
                self.request_frame(ctx.link());
                return true;
            },
            Msg::Grab(event) => {
                if event.is_primary() && event.button() == 0 {
                    if let Some(canvas) = self.canvas.cast::<HtmlCanvasElement>() {
                        let _ = canvas.set_pointer_capture(event.pointer_id());
                    }
                    self.drag = Some((event.pointer_id(), event.client_x(), event.client_y()));
                }
                return false;
            },
            Msg::Move(event) => {
                let Some((id, x, y)) = self.drag else { return false };
                if id != event.pointer_id() {
                    return false;
                }
                self.orbit[0] += (event.client_x() - x) as f32 * 0.008;
                self.orbit[1] += (event.client_y() - y) as f32 * 0.008;
                self.drag = Some((id, event.client_x(), event.client_y()));
            },
            Msg::Release(id) => {
                if self.drag.is_some_and(|drag| drag.0 == id) {
                    self.drag = None;
                }
                return false;
            },
            Msg::Key(event) => {
                match event.key().as_str() {
                    "ArrowLeft" => self.orbit[0] -= 0.15,
                    "ArrowRight" => self.orbit[0] += 0.15,
                    "ArrowUp" => self.orbit[1] -= 0.12,
                    "ArrowDown" => self.orbit[1] += 0.12,
                    "Home" => self.orbit = INITIAL_VIEW,
                    _ => return false,
                }
                event.prevent_default();
            },
            Msg::Reset => self.orbit = INITIAL_VIEW,
        }
        if orbit_changed {
            self.orbit[1] = self.orbit[1].clamp(-1.05, 1.05);
            self.camera = Camera::new(self.orbit, self.aspect);
        }
        self.request_frame(ctx.link());
        (orbit_changed || layout_changed) && ctx.props().on_send.is_some()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let gates = ctx.props().gates;
        let link = ctx.link();
        let release = link.callback(|e: PointerEvent| Msg::Release(e.pointer_id()));
        html! {
            <div class="wl-field">
                <canvas ref={self.canvas.clone()} tabindex={if self.unavailable { "-1" } else { "0" }}
                    aria-label="三维世界线观测仪，拖动或使用方向键旋转，Home 键复位"
                    onpointerdown={link.callback(Msg::Grab)} onpointermove={link.callback(Msg::Move)}
                    onpointerup={release.clone()} onpointercancel={release.clone()}
                    onlostpointercapture={release} onkeydown={link.callback(Msg::Key)} />
                <div class="wl-readout">
                    <span>{ "世界线变动率" }</span>
                    <strong>{ divergence(gates) }<small>{ "%" }</small></strong>
                </div>
                if self.unavailable {
                    <div class="wl-flat">
                        <p>{ "三维视图不可用 · 当前时间线" }</p>
                        <ol>{ for STATIONS.iter().enumerate().map(|(i, (time, _))| html! {
                            <li class={is_aligned(gates, i).then_some("is-aligned")}>
                                { *time }<small>{ if is_aligned(gates, i) { "收束" } else { "分歧" } }</small>
                            </li>
                        }) }</ol>
                    </div>
                } else {
                    if let Some(send) = ctx.props().on_send.as_ref().filter(|_| self.renderer.is_some()) {
                        { for STATIONS.iter().enumerate().map(|(i, (time, place))| {
                            let (x, y) = self.camera.label_position(i);
                            html! {
                                <button type="button" class={classes!("wl-clock", is_aligned(gates, i).then_some("is-aligned"))}
                                    style={format!("left:{x:.2}%;top:{y:.2}%")}
                                    aria-label={format!("从光环向 {time} {place} 投递")}
                                    disabled={ctx.props().finished} onclick={send.reform(move |_| i)}>{ *time }</button>
                            }
                        }) }
                    }
                    <div class="wl-view-tools">
                        <span>{ if ctx.props().on_send.is_some() { "拖动旋转 · 点时刻投递" } else { "拖动或按方向键旋转" } }</span>
                        <button type="button" onclick={link.callback(|_| Msg::Reset)}>{ "复位视角" }</button>
                    </div>
                }
            </div>
        }
    }
}
