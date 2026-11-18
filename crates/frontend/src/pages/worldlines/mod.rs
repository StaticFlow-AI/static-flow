//! A first-person exploration of the Future Gadget Laboratory.
mod story;
mod walker;

pub use walker::Walker;
use yew::prelude::*;
use yew_router::prelude::Link;

use self::story::{name, Line, Panel, Story, CHOICES, MEMBERS};
use crate::{components::spatial_surface::SpatialSurface, router::Route};

pub enum Message {
    Scene(String, String),
    Inspect(String),
    Phone,
    Close,
    Help,
    Journal,
    Send(usize),
    Leap,
    ConfirmRestart,
    Restart,
}

const SAVE_KEY: &str = "staticflow.lab.worldlines.v1";
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub struct WorldlinesGame {
    story: Story,
    focus: String,
    zone: String,
    ready: bool,
    failed: bool,
    dialog: NodeRef,
}
impl Component for WorldlinesGame {
    type Message = Message;
    type Properties = ();
    fn create(_: &Context<Self>) -> Self {
        Self {
            story: storage()
                .and_then(|store| store.get_item(SAVE_KEY).ok().flatten())
                .and_then(|json| serde_json::from_str::<Story>(&json).ok())
                .filter(|story| story.mail.iter().all(|i| *i < CHOICES.len()))
                .unwrap_or_default(),
            focus: String::new(),
            zone: "二楼 · 未来道具研究所".into(),
            ready: false,
            failed: false,
            dialog: NodeRef::default(),
        }
    }
    fn update(&mut self, _: &Context<Self>, msg: Message) -> bool {
        let checkpoint = matches!(&msg, Message::Scene(event,_) if event=="act")
            || matches!(
                msg,
                Message::Inspect(_) | Message::Send(_) | Message::Leap | Message::Restart
            );
        match msg {
            Message::Scene(event, value) => match event.as_str() {
                "ready" => {
                    self.ready = true;
                    self.failed = false;
                },
                "error" => self.failed = true,
                "zone" => self.zone = value,
                "focus" => self.focus = value,
                "act" if self.story.panel.is_none() => self.story.inspect(&value),
                "phone" if self.story.phone => self.story.panel = Some(Panel::Phone),
                "close" => self.story.panel = None,
                _ => return false,
            },
            Message::Inspect(id) => self.story.inspect(&id),
            Message::Phone if self.story.phone => self.story.panel = Some(Panel::Phone),
            Message::Phone => return false,
            Message::Close => self.story.panel = None,
            Message::Help => self.story.panel = Some(Panel::Help),
            Message::Journal => self.story.panel = Some(Panel::Journal),
            Message::Send(i) => {
                self.story.send(i);
                self.focus.clear();
            },
            Message::Leap => {
                self.story.leap();
                self.focus.clear();
            },
            Message::ConfirmRestart => self.story.panel = Some(Panel::Restart),
            Message::Restart => {
                let epoch = self.story.epoch + 1;
                self.story = Story {
                    epoch,
                    ..Story::default()
                };
                self.focus.clear();
            },
        }
        if checkpoint {
            if let (Some(store), Ok(json)) = (storage(), serde_json::to_string(&self.story)) {
                let _ = store.set_item(SAVE_KEY, &json);
            }
        }
        true
    }
    fn rendered(&mut self, _: &Context<Self>, _: bool) {
        if let Some(d) = self.dialog.cast::<web_sys::HtmlDialogElement>() {
            match (self.story.panel.is_some(), d.open()) {
                (true, false) => {
                    let _ = d.show_modal();
                },
                (false, true) => d.close(),
                _ => {},
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let story = &self.story;
        let link = ctx.link();
        let (objective, instruction) = story.objective();
        let snapshot = serde_json::json!({
            "line": story.line.id(), "number": story.line.number(), "phone": story.phone,
            "crt": story.crt, "upa": story.upa, "armed": story.armed, "epoch": story.epoch,
            "objective": objective, "modal": story.panel.is_some()
        })
        .to_string();
        let event = link.callback(|(topic, value)| Message::Scene(topic, value));
        let is_phone = matches!(story.panel, Some(Panel::Phone));
        let (signal, arrival) = match story.line {
            Line::Alpha => ("TIME LEAP", "记忆回到了 17:43"),
            Line::Echo => ("D-MAIL RECEIVED", "沙发上的人消失了"),
            Line::Beta => ("D-MAIL RECEIVED", "电脑前的椅子空了"),
            Line::Gate => ("STEINS GATE", "怀表又响了一声"),
        };
        let inspect = link.callback({
            let id = self.focus.clone();
            move |_| Message::Inspect(id.clone())
        });
        let cancel = link.callback(|e: Event| {
            e.prevent_default();
            Message::Close
        });
        html! {
            <main class={classes!("lab-world", self.failed.then_some("lab-text"))} data-line={story.line.id()}>
                <SpatialSurface kind="lab" state={snapshot} {event}/>
                <header class="lab-top">
                    <Link<Route> classes="lab-exit" to={Route::Home}><span aria-hidden="true">{"←"}</span>{" 星系"}</Link<Route>>
                    <div class="lab-location"><h1>{"未来道具研究所"}</h1><span>{&self.zone}</span></div>
                    <button type="button" class="lab-tool" onclick={link.callback(|_| Message::Help)} aria-label="操作说明">{"?"}</button>
                </header>
                <section class="lab-objective" aria-label="当前目标" aria-live="polite">
                    <span class="lab-clock">{format!("8月13日  {}", story.line.time())}</span><p>{instruction}</p>
                </section>
                if self.failed {
                    {self.action_directory(ctx)}
                } else if !self.ready {
                    <div class="lab-loading" role="status">{"正在打开研究所的门…"}</div>
                } else {
                    <div class="lab-cross" aria-hidden="true"></div>
                }
                <footer class="lab-hud">
                    <button class="divergence" type="button" aria-label="查看世界线记录" onclick={link.callback(|_| Message::Journal)}>
                        <small>{"世界线变动率"}</small><strong>{story.line.number()}</strong>
                    </button>
                    if !self.focus.is_empty() && !self.failed {
                        <button type="button" class="lab-interact" onclick={inspect}><kbd>{"E"}</kbd>{name(&self.focus)}</button>
                    }
                    if story.phone {
                        <button class="lab-pocket" type="button" onclick={link.callback(|_| Message::Phone)}>
                            <span aria-hidden="true">{"✉"}</span>{" 手机"}<kbd>{"M"}</kbd>
                        </button>
                    }
                </footer>
                if self.ready && !story.phone {
                    <p class="lab-controls">
                        <span class="lab-desktop-help">{"WASD 移动　·　拖动环顾　·　E 调查"}</span>
                        <span class="lab-touch-help">{"左手摇杆移动　·　右侧滑动环顾　·　点按调查"}</span>
                    </p>
                }
                if story.epoch > 0 {
                    <div key={story.epoch} class="lab-shift" aria-live="polite">
                        <small>{signal}</small><strong>{story.line.number()}</strong><p>{arrival}</p>
                    </div>
                }
                <dialog ref={self.dialog.clone()} class={classes!("lab-dialog", is_phone.then_some("flip-phone"))}
                    aria-label={if is_phone { "随身手机" } else { "研究所记录" }} oncancel={cancel}>
                    {self.panel(ctx)}
                </dialog>
            </main>
        }
    }
}
impl WorldlinesGame {
    fn action_directory(&self, ctx: &Context<Self>) -> Html {
        let story = &self.story;
        let targets = ["phone", "crt", "wave", "leap", "upa"]
            .into_iter()
            .chain(MEMBERS.iter().map(|(id, _)| *id))
            .filter(|id| match *id {
                "phone" => !story.phone,
                "upa" => !story.upa,
                "daru" => story.line != Line::Beta,
                "mayuri" => story.line != Line::Echo,
                _ => true,
            });
        html! {
            <section class="lab-access">
                <h2>{"研究所行动目录"}</h2>
                <p>{"当前设备无法绘制三维场景。选择地点和人物可以继续同一段剧情。"}</p>
                <div>{for targets.map(|id| html! {
                    <button type="button" onclick={ctx.link().callback(move |_| Message::Inspect(id.to_owned()))}>{name(id)}</button>
                })}</div>
            </section>
        }
    }

    fn panel(&self, ctx: &Context<Self>) -> Html {
        let story = &self.story;
        let Some(panel) = &story.panel else { return Html::default() };
        let link = ctx.link();
        let close = link.callback(|_| Message::Close);
        let dismiss = html! {
            <button class="lab-confirm" type="button" onclick={close.clone()}>{"放下，回到研究所"}</button>
        };
        let history = || {
            story.mail.iter().map(|&i| {
                let mail = &CHOICES[i];
                html! { <p class="lab-log-entry">{format!("{}　{}", mail.to, mail.body)}</p> }
            })
        };
        match panel {
            Panel::Phone => {
                let (from, message) = story.incoming();
                let choices = CHOICES
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| story.can_send(*i));
                html! {
                    <>
                        <div class="phone-speaker" aria-hidden="true"></div>
                        <div class="phone-screen">
                            <div class="phone-status"><span>{"▰▰▰  MADO"}</span><time>{story.line.time()}</time></div>
                            <h2>{"收件箱"}</h2><small>{from}</small><p>{message}</p>
                            if story.armed {
                                <h3>{"D-Mail · 投递至 8 月 12 日"}</h3>
                                {for choices.map(|(i,mail)| html! {
                                    <button type="button" class="mail-choice" onclick={link.callback(move |_| Message::Send(i))}>
                                        <small>{mail.to}</small>{mail.body}<span aria-hidden="true">{" ↗"}</span>
                                    </button>
                                })}
                            } else if story.line == Line::Alpha {
                                <p class="phone-signal">{"发信状态　等待电话微波炉接入"}</p>
                            }
                            if !story.mail.is_empty() {
                                <details><summary>{format!("已发送 · {}", story.mail.len())}</summary>{for history()}</details>
                            }
                        </div>
                        <div class="phone-hinge" aria-hidden="true"></div>
                        <button type="button" class="phone-end" onclick={close}>{"合上手机"}</button>
                    </>
                }
            },
            Panel::Talk(who, words) => html! { <><h2>{*who}</h2><p>{*words}</p>{dismiss}</> },
            Panel::Leap => html! {
                <>
                    <div class="lab-dialog-meta">{"TIME LEAP / 08.13"}</div><h2>{"把记忆送回 17:43？"}</h2>
                    <p>{"你会回到那封 D-Mail 发出前。显像管需要重新通电，亲眼见过的事会跟你一起回去。"}</p>
                    <button class="lab-confirm" type="button" onclick={link.callback(|_| Message::Leap)}>{"戴好耳机，时间跳跃"}</button>
                    <button class="lab-secondary" type="button" onclick={close}>{"再等一会"}</button>
                </>
            },
            Panel::Help => html! {
                <>
                    <h2>{"在研究所里行动"}</h2>
                    <dl class="lab-manual">
                        <dt>{"电脑"}</dt><dd>{"WASD 移动，Shift 快走。拖动环顾或点击画面锁定鼠标，方向键也能转动视角。Esc 释放鼠标。"}</dd>
                        <dt>{"触屏"}</dt><dd>{"左手摇杆移动，右侧拖动环顾。面向物件，点调查按钮。"}</dd>
                        <dt>{"设备"}</dt><dd>{"靠近并面向人物或物件，按 E 调查。拾取手机后按 M 看信。金色标记指向当前目标或通往目标的楼梯。"}</dd>
                    </dl>
                    {dismiss}
                </>
            },
            Panel::Restart => html! {
                <>
                    <h2>{"重新开始这个下午？"}</h2><p>{"这台设备上的世界线记录会回到第一次收到短信时。"}</p>
                    <button type="button" class="lab-confirm" onclick={link.callback(|_| Message::Restart)}>{"从 17:43 重新开始"}</button>
                    <button type="button" class="lab-secondary" onclick={close}>{"继续这条世界线"}</button>
                </>
            },
            Panel::Journal => html! {
                <>
                    <div class="lab-dialog-meta">{"DIVERGENCE METER"}</div><h2>{format!("{}%", story.line.number())}</h2>
                    <p>{format!("时间跳跃 {} 次　·　D-Mail {} 封", story.leaps, story.mail.len())}</p>
                    {for history()}{dismiss}
                    if story.phone {
                        <button class="lab-secondary" type="button" onclick={link.callback(|_| Message::ConfirmRestart)}>{"重新开始"}</button>
                    }
                </>
            },
        }
    }
}
