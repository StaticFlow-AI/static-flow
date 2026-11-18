use gloo_events::EventListener;
use yew::prelude::*;
use yew_router::prelude::{use_navigator, Link};

use crate::{
    api::{self, SongListItem},
    components::{spatial_surface::SpatialSurface, theme_toggle::ThemeToggle},
    config::{asset_path, route_path},
    models::ArticleListItem,
    music_context::{MusicAction, MusicPlayerContext},
    router::Route,
};
const PORTS: [(&str, &str); 7] = [
    ("lab", "未来道具研究所"),
    ("archive", "知识库"),
    ("music", "曲库"),
    ("gallery", "图库"),
    ("studio", "共建台"),
    ("tools", "工具站"),
    ("search", "检索"),
];

fn destinations(links: impl IntoIterator<Item = (&'static str, Route)>) -> Html {
    html! {
        <div class="port-links">
            { for links.into_iter().map(|(label, to)| html! {
                <Link<Route> {to}>{label}<span aria-hidden="true">{"↗"}</span></Link<Route>>
            }) }
        </div>
    }
}

fn external(label: &str, href: String) -> Html {
    html! {
        <a {href} target="_blank" rel="noopener noreferrer">
            {label}<span aria-hidden="true">{"↗"}</span>
        </a>
    }
}

#[function_component(Archive)]
fn archive() -> Html {
    let articles = use_state(Vec::<ArticleListItem>::new);
    {
        let articles = articles.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(page) = api::fetch_articles(None, None, Some(5), Some(0)).await {
                    articles.set(page.articles);
                }
            });
        });
    }
    html! {
        <>
            {destinations([("全部文章", Route::Posts), ("最新时间线", Route::LatestArticles),
                ("标签", Route::Tags), ("分类", Route::Categories)])}
            if !articles.is_empty() {
                <h3>{"最近发表"}</h3>
                <div class="port-list">
                    { for articles.iter().map(|a| html! {
                        <Link<Route> to={Route::ArticleDetail { id: a.id.clone() }}>{&a.title}</Link<Route>>
                    }) }
                </div>
            }
        </>
    }
}

#[function_component(Jukebox)]
fn jukebox() -> Html {
    let songs = use_state(Vec::<SongListItem>::new);
    let loading = use_state(|| false);
    let player = use_context::<MusicPlayerContext>();
    {
        let songs = songs.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let list = match api::fetch_random_recommended_songs(Some(4), &[]).await {
                    Ok(songs) if !songs.is_empty() => songs,
                    _ => api::fetch_songs(Some(4), Some(0), None, None, None)
                        .await
                        .map(|p| p.songs)
                        .unwrap_or_default(),
                };
                songs.set(list);
            });
        });
    }
    let play = {
        let loading = loading.clone();
        let player = player.clone();
        let playlist: Vec<_> = songs.iter().map(|song| song.id.clone()).collect();
        Callback::from(move |id: String| {
            let href = route_path(&format!("/media/audio/{}", urlencoding::encode(&id)));
            let Some(player) = player.clone() else {
                let _ = crate::navigation_context::navigate_spa_to(&href);
                return;
            };
            if player.song_id.as_ref() == Some(&id) {
                player.dispatch(MusicAction::TogglePlay);
                return;
            }
            loading.set(true);
            let loading = loading.clone();
            let ids = playlist.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(Some(song)) = api::fetch_song_detail(&id).await {
                    player.dispatch(MusicAction::SetPlaylist {
                        source: "home:galaxy".into(),
                        ids,
                    });
                    player.dispatch(MusicAction::PlaySong {
                        song,
                        id: id.clone(),
                    });
                    let _ = api::track_song_play(&id).await;
                } else {
                    let _ = crate::navigation_context::navigate_spa_to(&href);
                }
                loading.set(false);
            });
        })
    };
    let track = |song: &SongListItem| {
        let active = player
            .as_ref()
            .is_some_and(|p| p.song_id.as_ref() == Some(&song.id) && p.playing);
        let id = song.id.clone();
        let onclick = play.reform(move |_| id.clone());
        html! {
            <div class="port-song">
                <Link<Route> to={Route::MusicPlayer { id: song.id.clone() }}>
                    {&song.title}<small>{&song.artist}</small>
                </Link<Route>>
                <button type="button" {onclick} disabled={*loading}
                    aria-label={format!("{} {}", if active { "暂停" } else { "播放" }, song.title)}>
                    {if active { "Ⅱ" } else { "▶" }}
                </button>
            </div>
        }
    };
    html! {
        <>
            {destinations([("进入曲库", Route::MediaAudio)])}
            if !songs.is_empty() {
                <h3>{"在星图上听"}</h3><div class="port-list">{for songs.iter().map(track)}</div>
            }
        </>
    }
}

fn port_content(id: &str) -> Html {
    match id {
        "archive" => html! { <Archive /> },
        "music" => html! { <Jukebox /> },
        "gallery" => destinations([("浏览图库", Route::MediaImage), ("视频", Route::MediaVideo)]),
        "search" => destinations([("全文与语义检索", Route::Search)]),
        "studio" => {
            let studio_link = |(title, path): (&str, &str)| {
                let href = route_path(path);
                let target = href.clone();
                let onclick = Callback::from(move |e: MouseEvent| {
                    e.prevent_default();
                    let _ = crate::navigation_context::navigate_spa_to(&target);
                });
                html! { <a {href} {onclick}>{title}<span aria-hidden="true">{"↗"}</span></a> }
            };
            html! {
                <div class="port-links">
                    <Link<Route> to={Route::Studio}>{"读者共建台"}<span>{"↗"}</span></Link<Route>>
                    { for [("提交文章", "/studio#article"), ("许个音乐愿望", "/studio#music")].map(studio_link) }
                </div>
            }
        },
        "tools" => html! {
            <>
                {destinations([("LLM Gateway", Route::LlmAccess), ("Kiro Gateway", Route::KiroAccess),
                    ("使用指南", Route::LlmAccessGuide), ("用量查询", Route::LlmAccessUsage),
                    ("配额状态", Route::LlmAccessQuotaStatus)])}
                <h3>{"关于 L_B__"}</h3>
                <div class="port-links">
                    {external("GitHub", "https://github.com/ACking-you".into())}
                    {external("Bilibili", "https://space.bilibili.com/24264499".into())}
                    {external("StaticFlow 源码", "https://github.com/acking-you/static_flow".into())}
                    { for [2025, 2024].map(|year| external(&format!("GitHub Wrapped {year}"),
                        asset_path(&format!("standalone/github-wrapped-{year}.html")))) }
                </div>
            </>
        },
        _ => Html::default(),
    }
}


#[function_component(HomePage)]
pub fn home_page() -> Html {
    let nav = use_navigator();
    let selected = use_state(String::new);
    let error = use_state(|| false);
    let ready = use_state(|| false);
    {
        let nav = nav.clone();
        use_effect_with((), move |_| {
            let follow = move || {
                let hash = web_sys::window()
                    .and_then(|w| w.location().hash().ok())
                    .unwrap_or_default();
                match hash.as_str() {
                    "#workshop" | "#workshop-music" => {
                        let _ = crate::navigation_context::navigate_spa_to(&route_path(
                            if hash == "#workshop" { "/studio#article" } else { "/studio#music" },
                        ));
                    },
                    "#worldlines" => {
                        if let Some(nav) = &nav {
                            nav.replace(&Route::Worldlines);
                        }
                    },
                    _ => {},
                }
            };
            follow();
            let listener = web_sys::window()
                .map(|win| EventListener::new(&win, "hashchange", move |_| follow()));
            move || drop(listener)
        });
    }
    let choose = {
        let selected = selected.clone();
        let nav = nav.clone();
        Callback::from(move |id: String| {
            if id == "lab" {
                if let Some(nav) = &nav {
                    nav.push(&Route::Worldlines);
                }
            } else {
                selected.set(id);
            }
        })
    };
    let event = {
        let choose = choose.clone();
        let ready = ready.clone();
        let error = error.clone();
        let selected = selected.clone();
        Callback::from(move |(e, v): (String, String)| match e.as_str() {
            "planet" => choose.emit(v),
            "ready" => {
                ready.set(true);
                error.set(false);
            },
            "error" => {
                error.set(true);
                selected.set("index".into());
            },
            _ => {},
        })
    };
    let labels = PORTS
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let snapshot = serde_json::json!({"labels":labels,"selected":*selected}).to_string();
    let close = {
        let selected = selected.clone();
        Callback::from(move |_| selected.set(String::new()))
    };
    let index = {
        let selected = selected.clone();
        Callback::from(move |_| {
            selected.set(if selected.as_str() == "index" { String::new() } else { "index".into() })
        })
    };
    let on_key = {
        let selected = selected.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                selected.set(String::new());
            }
        })
    };
    let help = if *error {
        "使用星图目录选择目的地"
    } else if *ready {
        "拖动旋转　·　点星球前往　·　滚轮或双指缩放"
    } else {
        "正在绘制星图…"
    };
    let port_button = |(id, title): (&'static str, &'static str)| {
        let onclick = choose.reform(move |_| id.to_owned());
        html! {
            <button type="button" {onclick}>
                <i class={format!("port-dot dot-{id}")}></i>{title}<span aria-hidden="true">{"↗"}</span>
            </button>
        }
    };
    html! {
        <main class="galaxy-home" onkeydown={on_key}>
            <SpatialSurface kind="galaxy" state={snapshot} {event}/>
            <header class="galaxy-header">
                <div class="galaxy-brand">
                    <img src={asset_path("static/avatar.webp")} alt=""/>
                    <span>{"StaticFlow"}</span><small>{"L_B__"}</small>
                </div>
                <nav aria-label="站点工具">
                    <Link<Route> classes="galaxy-search" to={Route::Search}>{"检索"}</Link<Route>>
                    <ThemeToggle/>
                    <button class="star-index" type="button" onclick={index} aria-expanded={(!selected.is_empty()).to_string()}>
                        {"星图目录"}<span aria-hidden="true">{" ☷"}</span>
                    </button>
                </nav>
            </header>
            <div class="galaxy-title"><div class="galaxy-kicker">{"L_B__ 的个人博客"}</div><h1>{"世界线星图"}</h1></div>
            <footer class="galaxy-foot">
                <div class="galaxy-launch"><small>{"第一人称 · 世界线冒险"}</small>
                    <Link<Route> classes="launch-lab" to={Route::Worldlines}>
                        {"进入未来道具研究所"}<span aria-hidden="true">{" ↗"}</span>
                    </Link<Route>>
                </div>
                <p>{help}</p>
            </footer>
            if !selected.is_empty() {
                <aside class="port-panel" aria-label="星图目的地">
                    <div class="port-head">
                        <h2>{labels.get(selected.as_str()).copied().unwrap_or("星图目录")}</h2>
                        <button type="button" onclick={close} aria-label="关闭目的地">{"×"}</button>
                    </div>
                    if selected.as_str() == "index" {
                        <div class="port-index">{for PORTS.map(port_button)}</div>
                    } else {
                        {port_content(selected.as_str())}
                    }
                </aside>
            }
        </main>
    }
}
