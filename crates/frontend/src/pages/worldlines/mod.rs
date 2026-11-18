//! Interactive world-line exhibit and its D-Mail puzzle.

mod puzzle;
mod renderer;
mod scene;

use std::rc::Rc;

use yew::prelude::*;
use yew_router::prelude::Link;

use self::{
    puzzle::{is_aligned, Outcome, Run, CHAPTERS, CONVERGED, STATIONS},
    scene::FieldCanvas,
};
use crate::router::Route;

#[function_component(WorldlinesExhibit)]
pub fn worldlines_exhibit() -> Html {
    let gates = use_state(|| 0u8);
    html! {
        <section class="wl-exhibit wl-wrap" id="worldlines" aria-labelledby="wl-exhibit-title">
            <div class="wl-exhibit-copy">
                <h2 id="wl-exhibit-title">{ "世界线观测室" }</h2>
                <p>{ "同样的四个时刻，可以通往不同的未来。切换观测线，看看分开的光束怎样重新收束。" }</p>
                <div class="wl-selector" role="group" aria-label="选择观测线">
                    { for [(0u8, "α 世界线"), (CONVERGED, "目标世界线")].into_iter().map(|(mask, name)| {
                        let gates = gates.clone();
                        html! {
                            <button type="button" aria-pressed={(*gates == mask).to_string()}
                                onclick={Callback::from(move |_| gates.set(mask))}>{ name }</button>
                        }
                    }) }
                </div>
                <Link<Route> classes="wl-enter" to={Route::Worldlines}>
                    { "玩《逆时投递》" }<span aria-hidden="true">{ "→" }</span>
                </Link<Route>>
            </div>
            <FieldCanvas gates={*gates} />
        </section>
    }
}

enum Action {
    Send(usize),
    Retry,
    Advance,
}

impl Reducible for Run {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Action) -> Rc<Self> {
        let mut run = *self;
        match action {
            Action::Send(station) => run.send(station),
            Action::Retry => run = Run::start(run.chapter),
            Action::Advance if run.outcome() == Outcome::Delivered => {
                run = Run::start((run.chapter + 1) % CHAPTERS.len());
            },
            Action::Advance => {},
        }
        Rc::new(run)
    }
}

#[function_component(WorldlinesGame)]
pub fn worldlines_game() -> Html {
    crate::hooks::use_scroll_to_top();
    let run = use_reducer_eq(|| Run::start(0));
    let chapter = &CHAPTERS[run.chapter];
    let outcome = run.outcome();
    let send = {
        let run = run.clone();
        Callback::from(move |i| run.dispatch(Action::Send(i)))
    };
    let retry = {
        let run = run.clone();
        Callback::from(move |_| run.dispatch(Action::Retry))
    };
    let advance = {
        let run = run.clone();
        Callback::from(move |_| run.dispatch(Action::Advance))
    };
    html! {
        <main class="wl-game wl-wrap">
            <Link<Route> classes="wl-back" to={Route::Home}>{ "← 返回首页" }</Link<Route>>
            <header class="wl-game-head">
                <div>
                    <h1>{ "逆时投递" }</h1>
                    <p>{ "一封 D-Mail，能改变几个时刻的命运？" }</p>
                </div>
                <div class="wl-target">{ "目标变动率" }<strong>{ "1.048596%" }</strong></div>
            </header>
            <div class="wl-instrument">
                <FieldCanvas gates={run.gates} on_send={send.clone()} last_mail={run.last_mail}
                    finished={outcome != Outcome::Playing} />
                <section class="wl-console" aria-label="D-Mail 发信台">
                    <div class="wl-chapter">
                        <span class="wl-chapter-count">{ format!("实验 {}/{}", run.chapter + 1, CHAPTERS.len()) }</span>
                        <h2>{ chapter.title }</h2>
                        <p>{ chapter.incident }</p>
                    </div>
                    <div class="wl-controls">
                        <p class="wl-rule">{ "选一个时刻寄信，它和紧邻的时刻都会翻转。让四个时刻同时收束。" }</p>
                        <div class="wl-stations" role="group" aria-label="选择 D-Mail 送达时刻">
                            { for STATIONS.iter().enumerate().map(|(i, (time, place))| {
                                let aligned = is_aligned(run.gates, i);
                                let state = if aligned { "已收束" } else { "分歧中" };
                                html! {
                                    <button type="button" class={classes!("wl-station", aligned.then_some("is-aligned"))}
                                        disabled={outcome != Outcome::Playing}
                                        aria-label={format!("向 {time} {place} 发送 D-Mail，{state}")}
                                        onclick={send.reform(move |_| i)}>
                                        <span class="wl-time">{ *time }</span>
                                        <span class="wl-place">{ *place }</span>
                                        <span class="wl-gate-state">{ state }</span>
                                    </button>
                                }
                            }) }
                        </div>
                        <div class="wl-result">
                            <div role="status" aria-atomic="true">
                                <strong>{ format!("已收束 {}/4 · 剩余 {} 封 D-Mail", run.gates.count_ones(), run.remaining) }</strong>
                                if outcome == Outcome::Delivered {
                                    <p>{ chapter.ending }</p>
                                } else if outcome == Outcome::Exhausted {
                                    <p>{ "信箱空了，时间线仍有分歧。回到发信前，再试一次。" }</p>
                                }
                            </div>
                            if outcome == Outcome::Delivered {
                                <button class="wl-action" type="button" onclick={advance}>
                                    { if run.chapter + 1 == CHAPTERS.len() { "从头再玩 ↻" } else { "下一场实验 →" } }
                                </button>
                            } else {
                                <button class="wl-action" type="button" onclick={retry}
                                    disabled={*run == Run::start(run.chapter)}>{ "重试本关 ↻" }</button>
                            }
                        </div>
                    </div>
                </section>
            </div>
        </main>
    }
}
