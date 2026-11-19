//! Personal split-screen hero for the home page.
//!
//! Desktop: one viewport, cut diagonally into an ink pane (world-line
//! divergence meter + a live `sf-cli` terminal that types out real site data)
//! and a parchment pane (a "what lives here" timeline: knowledge base,
//! jukebox, reader-driven ingestion, lab). The avatar floats on the seam.
//! Mobile: the same identity block stacks above the timeline; the terminal is
//! dropped and the meter shrinks into a badge.

use std::{cell::RefCell, rc::Rc};

use gloo_timers::callback::{Interval, Timeout};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    api::{self, SongListItem},
    models::ArticleListItem,
    router::Route,
};

/// Famous world lines (Steins;Gate). The first one is where the meter rests.
const WORLD_LINES: &[(&str, &str)] = &[
    ("1.048596", "Steins;Gate 世界线"),
    ("0.571024", "α 世界线 · 起点"),
    ("0.337187", "α 世界线 · SERN"),
    ("1.130205", "β 世界线"),
    ("0.523299", "α 世界线 · 分歧"),
    ("1.382733", "β 世界线 · 第三次世界大战"),
];

const MOTTO_LINES: &[&str] = &["El Psy Kongroo", "Rustacean", "痴迷底层黑魔法的 Database 练习生"];

const TERMINAL_TICK_MS: u32 = 32;
const METER_TICK_MS: u32 = 55;
const METER_FLICKER_TICKS: u32 = 18;

fn prefers_reduced_motion() -> bool {
    let Some(w) = web_sys::window() else {
        return false;
    };
    let Ok(fn_val) = js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("matchMedia"))
    else {
        return false;
    };
    let Ok(func) = wasm_bindgen::JsCast::dyn_into::<js_sys::Function>(fn_val) else {
        return false;
    };
    let Ok(mql) =
        func.call1(&w, &wasm_bindgen::JsValue::from_str("(prefers-reduced-motion: reduce)"))
    else {
        return false;
    };
    js_sys::Reflect::get(&mql, &wasm_bindgen::JsValue::from_str("matches"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn random_digit() -> char {
    let d = (js_sys::Math::random() * 10.0).floor() as u32 % 10;
    char::from_digit(d, 10).unwrap_or('0')
}

/// Scramble every digit of a world-line value, keeping the decimal point.
fn scramble(value: &str) -> String {
    value
        .chars()
        .map(|c| if c == '.' { '.' } else { random_digit() })
        .collect()
}

fn truncate_chars(text: &str, max: usize) -> String {
    let count = text.chars().count();
    if count <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn short_date(date: &str) -> String {
    // "2026-11-14" -> "11-14"
    let trimmed = date.trim();
    if trimmed.len() >= 10 && trimmed.as_bytes().get(4) == Some(&b'-') {
        trimmed[5..10].to_string()
    } else {
        truncate_chars(trimmed, 5)
    }
}

fn format_count(value: Option<usize>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "—".to_string())
}

#[derive(Clone, PartialEq)]
enum TermKind {
    /// A typed command after the prompt.
    Cmd,
    /// Plain command output.
    Out,
    /// Output whose `accent` substring is highlighted.
    Accent,
}

#[derive(Clone, PartialEq)]
struct TermLine {
    kind: TermKind,
    text: String,
    accent: String,
}

impl TermLine {
    fn cmd(text: impl Into<String>) -> Self {
        Self {
            kind: TermKind::Cmd,
            text: text.into(),
            accent: String::new(),
        }
    }

    fn out(text: impl Into<String>) -> Self {
        Self {
            kind: TermKind::Out,
            text: text.into(),
            accent: String::new(),
        }
    }

    fn accent(text: impl Into<String>, accent: impl Into<String>) -> Self {
        Self {
            kind: TermKind::Accent,
            text: text.into(),
            accent: accent.into(),
        }
    }

    /// Ticks this line consumes: commands type character by character and
    /// then pause; output lines land whole after a short beat.
    fn cost(&self) -> usize {
        match self.kind {
            TermKind::Cmd => self.text.chars().count() + 10,
            TermKind::Out | TermKind::Accent => 5,
        }
    }
}

fn build_script(props: &PersonalHeroProps) -> Vec<TermLine> {
    let mut lines = vec![
        TermLine::cmd("sf-cli stats"),
        TermLine::accent(
            format!(
                "articles {}  songs {}  reader-requests {}",
                format_count(props.total_articles),
                format_count(props.total_music),
                format_count(props.reader_total)
            ),
            format_count(props.total_articles),
        ),
        TermLine::cmd("sf-cli latest --limit 3"),
    ];
    if props.latest.is_empty() {
        lines.push(TermLine::out("  (fetching from LanceDB…)"));
    } else {
        for article in props.latest.iter().take(3) {
            lines.push(TermLine::out(format!(
                "  {}  {}",
                short_date(&article.date),
                truncate_chars(&article.title, 22)
            )));
        }
    }
    lines.push(TermLine::cmd("sf-cli play --shuffle"));
    match props.featured_song.as_ref() {
        Some(song) => {
            let title = truncate_chars(&song.title, 18);
            lines.push(TermLine::accent(
                format!("  ♪ {} — {}", title, truncate_chars(&song.artist, 14)),
                title,
            ));
        },
        None => lines.push(TermLine::out("  ♪ (queue warming up…)")),
    }
    lines.push(TermLine::cmd("echo $MOTTO"));
    lines.push(TermLine::accent("  El Psy Kongroo.", "El Psy Kongroo."));
    lines
}

fn render_accent_line(text: &str, accent: &str) -> Html {
    if accent.is_empty() {
        return html! { { text.to_string() } };
    }
    match text.find(accent) {
        Some(idx) => {
            let (head, rest) = text.split_at(idx);
            let tail = &rest[accent.len()..];
            html! {
                <>
                    { head.to_string() }
                    <mark class="sf-term__hl">{ accent.to_string() }</mark>
                    { tail.to_string() }
                </>
            }
        },
        None => html! { { text.to_string() } },
    }
}

/// Props for [`PersonalHero`].
#[derive(Properties, PartialEq)]
pub struct PersonalHeroProps {
    /// Total published articles (None until fetched).
    pub total_articles: Option<usize>,
    /// Total songs in the jukebox.
    pub total_music: Option<usize>,
    /// Reader-driven requests (articles + music wishes).
    pub reader_total: Option<usize>,
    /// Most recent articles (only the first few are shown).
    pub latest: Vec<ArticleListItem>,
    /// Song highlighted in the "now playing" slot.
    pub featured_song: Option<SongListItem>,
    /// Whether the featured song is the one currently playing.
    pub featured_playing: bool,
    /// Whether the featured song is being fetched for playback.
    pub featured_loading: bool,
    /// Play / pause the featured song.
    pub on_play_featured: Callback<MouseEvent>,
}

fn spa_link(path: &'static str) -> (String, Callback<MouseEvent>) {
    let href = crate::config::route_path(path);
    let target = href.clone();
    (
        href,
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&target);
        }),
    )
}

/// Split-screen personal hero.
#[function_component(PersonalHero)]
pub fn personal_hero(props: &PersonalHeroProps) -> Html {
    let reduced_motion = use_memo((), |_| prefers_reduced_motion());
    let reduced_motion = *reduced_motion;

    // ---- Divergence meter -------------------------------------------------
    let world_idx = use_state(|| 0usize);
    let meter_text = use_state(|| {
        if reduced_motion {
            WORLD_LINES[0].0.to_string()
        } else {
            scramble(WORLD_LINES[0].0)
        }
    });
    // Bumped to re-run the flicker for the current target world line.
    let meter_epoch = use_state(|| 0u32);
    {
        let meter_text = meter_text.clone();
        let target = WORLD_LINES[*world_idx % WORLD_LINES.len()].0;
        use_effect_with((*meter_epoch, *world_idx), move |_| {
            let interval = if reduced_motion {
                meter_text.set(target.to_string());
                None
            } else {
                let mut ticks = 0u32;
                let meter_text = meter_text.clone();
                Some(Interval::new(METER_TICK_MS, move || {
                    if ticks > METER_FLICKER_TICKS {
                        return;
                    }
                    ticks += 1;
                    if ticks >= METER_FLICKER_TICKS {
                        meter_text.set(target.to_string());
                    } else {
                        // Digits lock in left to right as the flicker winds down.
                        let locked = (ticks as usize * target.len()) / METER_FLICKER_TICKS as usize;
                        let next: String = target
                            .chars()
                            .enumerate()
                            .map(|(i, c)| if c == '.' || i < locked { c } else { random_digit() })
                            .collect();
                        meter_text.set(next);
                    }
                }))
            };
            move || drop(interval)
        });
    }
    let shift_world_line = {
        let world_idx = world_idx.clone();
        let meter_epoch = meter_epoch.clone();
        Callback::from(move |_: MouseEvent| {
            world_idx.set((*world_idx + 1) % WORLD_LINES.len());
            meter_epoch.set(meter_epoch.wrapping_add(1));
        })
    };
    let world_label = WORLD_LINES[*world_idx % WORLD_LINES.len()].1;

    // ---- Typewriter terminal ---------------------------------------------
    let script = build_script(props);
    let total_cost: usize = script.iter().map(TermLine::cost).sum();
    let progress = use_state(|| if reduced_motion { usize::MAX } else { 0 });
    // Latest script length, read by the ticking interval so it stops
    // re-rendering once everything has been typed (data may arrive late and
    // grow the script, in which case typing simply resumes).
    let total_ref = use_mut_ref(|| 0usize);
    *total_ref.borrow_mut() = total_cost;
    {
        let progress = progress.clone();
        let total_ref = total_ref.clone();
        use_effect_with((), move |_| {
            let slot: Rc<RefCell<Option<Interval>>> = Rc::new(RefCell::new(None));
            let start = if reduced_motion {
                None
            } else {
                let slot = slot.clone();
                // Let the hero settle before the first keystroke.
                Some(Timeout::new(650, move || {
                    let mut value = 0usize;
                    let interval = Interval::new(TERMINAL_TICK_MS, move || {
                        if value <= *total_ref.borrow() {
                            value += 1;
                            progress.set(value);
                        }
                    });
                    *slot.borrow_mut() = Some(interval);
                }))
            };
            move || {
                drop(start);
                slot.borrow_mut().take();
            }
        });
    }
    let typed = (*progress).min(total_cost);
    let finished = typed >= total_cost;

    let mut term_rows: Vec<Html> = Vec::new();
    let mut cursor = 0usize;
    let mut caret_placed = false;
    for (idx, line) in script.iter().enumerate() {
        let start = cursor;
        cursor += line.cost();
        if typed < start {
            break;
        }
        let elapsed = typed - start;
        match line.kind {
            TermKind::Cmd => {
                let len = line.text.chars().count();
                let shown: String = line.text.chars().take(elapsed.min(len)).collect();
                let typing_here = typed < cursor;
                term_rows.push(html! {
                    <div key={idx} class="sf-term__row">
                        <span class="sf-term__prompt">{ "❯ " }</span>
                        <span class="sf-term__cmd">{ shown }</span>
                        if typing_here {
                            <span class="sf-term__caret" aria-hidden="true" />
                        }
                    </div>
                });
                if typing_here {
                    caret_placed = true;
                }
            },
            TermKind::Out => {
                if elapsed >= 2 {
                    term_rows.push(html! {
                        <div key={idx} class="sf-term__row sf-term__out">{ line.text.clone() }</div>
                    });
                }
            },
            TermKind::Accent => {
                if elapsed >= 2 {
                    term_rows.push(html! {
                        <div key={idx} class="sf-term__row sf-term__out">
                            { render_accent_line(&line.text, &line.accent) }
                        </div>
                    });
                }
            },
        }
    }
    if finished || !caret_placed {
        term_rows.push(html! {
            <div key="idle" class="sf-term__row">
                <span class="sf-term__prompt">{ "❯ " }</span>
                <span class="sf-term__caret sf-term__caret--idle" aria-hidden="true" />
            </div>
        });
    }

    // ---- Avatar speech bubble & world-line shift --------------------------
    let bubble = use_state(|| false);
    let bubble_timer = use_mut_ref(|| None::<Timeout>);
    let on_avatar_click = {
        let bubble = bubble.clone();
        let bubble_timer = bubble_timer.clone();
        let shift = shift_world_line.clone();
        Callback::from(move |e: MouseEvent| {
            shift.emit(e);
            bubble.set(true);
            let bubble_reset = bubble.clone();
            *bubble_timer.borrow_mut() = Some(Timeout::new(2200, move || {
                bubble_reset.set(false);
            }));
        })
    };

    // ---- Right pane data ----------------------------------------------------
    let latest_two: Vec<ArticleListItem> = props.latest.iter().take(2).cloned().collect();
    let search_href = crate::config::route_path("/search");
    let on_search = {
        let href = search_href.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let _ = crate::navigation_context::navigate_spa_to(&href);
        })
    };

    let meter_digits = (*meter_text).clone();
    let (studio_href, go_studio) = spa_link("/studio");
    let (studio_article_href, go_studio_article) = spa_link("/studio#article");
    let (studio_music_href, go_studio_music) = spa_link("/studio#music");

    html! {
        <section class="sf-hero" aria-label="StaticFlow · L_B__ 的个人知识库与曲库">
            <div class="sf-hero__ink" aria-hidden="true" />
            <div class="sf-hero__grain" aria-hidden="true" />

            <div class="sf-hero__inner">
                // ── Left: world line + terminal (desktop only terminal) ──
                <div class="sf-hero__left">
                    <button
                        type="button"
                        class="sf-meter"
                        onclick={shift_world_line.clone()}
                        title="点击跳跃世界线"
                        aria-label={format!("世界线变动率 {meter_digits}，点击跳跃")}
                    >
                        <span class="sf-meter__label">{ "DIVERGENCE · 世界线变动率" }</span>
                        <span class="sf-meter__tubes" aria-hidden="true">
                            { for meter_digits.chars().enumerate().map(|(i, c)| html! {
                                <span key={i} class={classes!("sf-meter__tube", (c == '.').then_some("sf-meter__tube--dot"))}>
                                    { c.to_string() }
                                </span>
                            }) }
                        </span>
                        <span class="sf-meter__caption">{ world_label }</span>
                    </button>

                    <div class="sf-term" aria-label="sf-cli 终端演示" role="img">
                        <div class="sf-term__bar">
                            <span>{ "ackingliu@staticflow" }</span>
                            <span class="sf-term__live">
                                <span class="sf-term__dot" aria-hidden="true" />
                                { "LANCEDB · LIVE" }
                            </span>
                        </div>
                        <div class="sf-term__body">
                            { for term_rows.into_iter() }
                        </div>
                    </div>
                </div>

                // ── Center: identity on the seam ──
                <div class="sf-hero__center">
                    <div class={classes!("sf-avatar", (*bubble).then_some("is-shifting"))}>
                        <span class="sf-avatar__orbit" aria-hidden="true">
                            <span class="sf-avatar__node" />
                        </span>
                        <button
                            type="button"
                            class="sf-avatar__btn"
                            onclick={on_avatar_click}
                            title="点击跳跃世界线"
                            aria-label="L_B__ 的头像（点我跳跃世界线）"
                        >
                            <span class="sf-avatar__lens">
                                <picture class="sf-avatar__pic">
                                    <source
                                        srcset={crate::config::asset_path("static/avatar.webp?v=kurisu1")}
                                        type="image/webp"
                                    />
                                    <img
                                        src={crate::config::asset_path("static/avatar.jpg?v=kurisu1")}
                                        alt="L_B__ · ackingliu"
                                        width="160"
                                        height="160"
                                        loading="eager"
                                        fetchpriority="high"
                                        decoding="async"
                                        class="sf-avatar__img"
                                    />
                                </picture>
                            </span>
                        </button>
                        <span class={classes!("sf-avatar__bubble", (*bubble).then_some("is-on"))} aria-hidden={(!*bubble).to_string()}>
                            <span class="sf-avatar__bubble-dot" aria-hidden="true" />
                            { "El Psy Kongroo." }
                        </span>
                    </div>

                    <p class="sf-hero__hello">{ "Hi, I'm L_B__ ·  ackingliu" }</p>
                    <h1 class="sf-hero__motto">
                        { for MOTTO_LINES.iter().map(|line| html! { <span>{ *line }</span> }) }
                    </h1>
                    <p class="sf-hero__blurb">
                        { "这是我的个人知识库与曲库：写下的源码剖析、读过的好文、写代码时循环的歌，都住在这里。" }
                    </p>

                    <div class="sf-hero__ctas">
                        <Link<Route> to={Route::Posts} classes="sf-hero__cta sf-hero__cta--solid">
                            { "翻翻知识库 →" }
                        </Link<Route>>
                        <Link<Route> to={Route::MediaAudio} classes="sf-hero__cta sf-hero__cta--ghost">
                            { "听听曲库" }
                        </Link<Route>>
                    </div>

                    <div class="sf-hero__social">
                        <a href="https://github.com/ACking-you" target="_blank" rel="noopener noreferrer" aria-label="GitHub">
                            <i class="fa-brands fa-github" aria-hidden="true"></i>
                        </a>
                        <a href="https://space.bilibili.com/24264499" target="_blank" rel="noopener noreferrer" aria-label="Bilibili">
                            <i class="fa-brands fa-bilibili" aria-hidden="true"></i>
                        </a>
                        <a href="https://github.com/acking-you/static_flow" target="_blank" rel="noopener noreferrer" aria-label="本站源码">
                            <i class="fas fa-code-branch" aria-hidden="true"></i>
                        </a>
                    </div>

                    <button type="button" class="sf-meter-badge" onclick={shift_world_line} aria-hidden="true" tabindex="-1">
                        <span class="sf-meter-badge__dot" />
                        { format!("WORLD LINE {meter_digits}") }
                    </button>
                </div>

                // ── Right: what lives here ──
                <div class="sf-hero__right">
                    <a href={search_href} onclick={on_search} class="sf-hero__search">
                        <span>
                            <span class="sf-hero__search-title">{ "先搜一搜：全文 · 语义 · 混合检索" }</span>
                            <span class="sf-hero__search-sub">{ "文章、曲目、图片统一存在 LanceDB 里 · 按 / 唤起" }</span>
                        </span>
                        <span class="sf-hero__search-arrow" aria-hidden="true">{ "→" }</span>
                    </a>

                    <div class="sf-hero__eyebrow">{ "这里有什么 / WHAT LIVES HERE" }</div>

                    <ol class="sf-tl">
                        <li class="sf-tl__item">
                            <span class="sf-tl__node" aria-hidden="true" />
                            <div class="sf-tl__kicker">{ "01 · KNOWLEDGE" }</div>
                            <Link<Route> to={Route::Posts} classes="sf-tl__title">
                                { "知识库" }
                                <span class="sf-tl__note">{ format!("{} 篇 · 源码剖析与工程手记", format_count(props.total_articles)) }</span>
                            </Link<Route>>
                            <ul class="sf-tl__list">
                                if latest_two.is_empty() {
                                    <li><span class="skeleton inline-block h-3.5 w-48 rounded" /></li>
                                    <li><span class="skeleton inline-block h-3.5 w-40 rounded" /></li>
                                } else {
                                    { for latest_two.iter().map(|a| html! {
                                        <li key={a.id.clone()}>
                                            <Link<Route> to={Route::ArticleDetail { id: a.id.clone() }} classes="sf-tl__link">
                                                <span class="sf-tl__date">{ short_date(&a.date) }</span>
                                                <span class="sf-tl__text">{ a.title.clone() }</span>
                                            </Link<Route>>
                                        </li>
                                    }) }
                                }
                            </ul>
                        </li>

                        <li class="sf-tl__item">
                            <span class={classes!("sf-tl__node", props.featured_playing.then_some("is-live"))} aria-hidden="true" />
                            <div class="sf-tl__kicker">{ "02 · JUKEBOX" }</div>
                            <Link<Route> to={Route::MediaAudio} classes="sf-tl__title">
                                { "曲库" }
                                <span class="sf-tl__note">{ format!("{} 首 · 写代码时循环的歌", format_count(props.total_music)) }</span>
                            </Link<Route>>
                            {
                                match props.featured_song.as_ref() {
                                    Some(song) => {
                                        let cover = song
                                            .cover_image
                                            .as_ref()
                                            .filter(|c| !c.is_empty())
                                            .map(|_| api::song_cover_url(song.cover_image.as_deref()));
                                        html! {
                                            <div class="sf-np">
                                                <Link<Route> to={Route::MusicPlayer { id: song.id.clone() }} classes="sf-np__cover">
                                                    if let Some(src) = cover {
                                                        <img src={src} alt="" loading="lazy" decoding="async" />
                                                    } else {
                                                        <span>{ "♪" }</span>
                                                    }
                                                </Link<Route>>
                                                <div class="sf-np__meta">
                                                    <Link<Route> to={Route::MusicPlayer { id: song.id.clone() }} classes="sf-np__title">
                                                        { song.title.clone() }
                                                    </Link<Route>>
                                                    <span class="sf-np__artist">{ song.artist.clone() }</span>
                                                </div>
                                                if props.featured_playing {
                                                    <span class="sf-eq" aria-label="正在播放">
                                                        <i /><i /><i /><i />
                                                    </span>
                                                }
                                                <button
                                                    type="button"
                                                    class={classes!("sf-np__play", props.featured_playing.then_some("is-on"))}
                                                    onclick={props.on_play_featured.clone()}
                                                    aria-label={if props.featured_playing { "暂停" } else { "播放" }}
                                                >
                                                    if props.featured_loading {
                                                        <span class="sf-np__spin" aria-hidden="true" />
                                                    } else if props.featured_playing {
                                                        <i class="fas fa-pause" aria-hidden="true"></i>
                                                    } else {
                                                        <i class="fas fa-play" aria-hidden="true"></i>
                                                    }
                                                </button>
                                            </div>
                                        }
                                    },
                                    None => html! {
                                        <div class="sf-np"><span class="skeleton inline-block h-10 w-full rounded" /></div>
                                    },
                                }
                            }
                        </li>

                        <li class="sf-tl__item">
                            <span class="sf-tl__node" aria-hidden="true" />
                            <div class="sf-tl__kicker">{ "03 · STUDIO" }</div>
                            <a href={studio_href} onclick={go_studio} class="sf-tl__title">
                                { "共建台" }
                                <span class="sf-tl__note">{ format!("{} 次读者入库", format_count(props.reader_total)) }</span>
                            </a>
                            <p class="sf-tl__desc">{ "想读的文章、想听的歌，丢给我——AI 会去调研、翻译、抓取，然后入库。" }</p>
                            <div class="sf-tl__actions">
                                <a href={studio_article_href} onclick={go_studio_article}>{ "投一篇文章 →" }</a>
                                <a href={studio_music_href} onclick={go_studio_music}>{ "点一首歌 →" }</a>
                            </div>
                        </li>

                        <li class="sf-tl__item">
                            <span class="sf-tl__node" aria-hidden="true" />
                            <div class="sf-tl__kicker">{ "04 · LAB" }</div>
                            <span class="sf-tl__title sf-tl__title--static">
                                { "实验室" }
                                <span class="sf-tl__note">{ "Rust · Yew · WebAssembly · LanceDB" }</span>
                            </span>
                            <div class="sf-tl__actions">
                                <Link<Route> to={Route::LlmAccess}>{ "免费 LLM Key 🦞" }</Link<Route>>
                                <a href={crate::config::asset_path("standalone/github-wrapped-2025.html")} target="_blank" rel="noopener noreferrer">{ "2025 GitHub Wrapped" }</a>
                                <a href="https://github.com/acking-you/static_flow" target="_blank" rel="noopener noreferrer">{ "本站源码" }</a>
                            </div>
                        </li>
                    </ol>
                </div>
            </div>
        </section>
    }
}
