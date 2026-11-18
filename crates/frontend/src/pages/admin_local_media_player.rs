use std::path::Path;

use gloo_timers::callback::Interval;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew_router::prelude::*;

use super::admin_local_media::AdminLocalMediaPlayerQuery;
use crate::{
    api::{
        build_admin_local_media_raw_playback, fetch_admin_local_media_job_status,
        open_admin_local_media_playback, LocalMediaPlaybackMode, LocalMediaPlaybackOpenResponse,
        LocalMediaPlaybackStatus,
    },
    pages::admin::shell::{AdminNav, AdminPageHeader, AdminShell},
    router::Route,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = sfLocalMediaPlayerMountLazy)]
    fn sf_local_media_player_mount(
        element: HtmlElement,
        url: &str,
        mode: &str,
        title: &str,
        storage_key: &str,
    );

    #[wasm_bindgen(js_namespace = window, js_name = sfLocalMediaPlayerUnmountLazy)]
    fn sf_local_media_player_unmount(element: HtmlElement);
}

#[function_component(AdminLocalMediaPlayerPage)]
pub fn admin_local_media_player_page() -> Html {
    let navigator = use_navigator();
    let location = use_location();
    let file = location
        .as_ref()
        .and_then(|loc| loc.query::<AdminLocalMediaPlayerQuery>().ok())
        .and_then(|query| query.file)
        .unwrap_or_default();

    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let playback = use_state(|| None::<LocalMediaPlaybackOpenResponse>);
    let selected_mode = use_state(|| PlaybackOpenMode::Raw);
    let player_host = use_node_ref();

    {
        let loading = loading.clone();
        let error = error.clone();
        let playback = playback.clone();
        let selected_mode = selected_mode.clone();
        let file = file.clone();
        use_effect_with((file.clone(), *selected_mode), move |(file, selected_mode)| {
            let has_file = !file.trim().is_empty();
            if !has_file {
                loading.set(false);
                error.set(Some("Missing file query".to_string()));
            } else {
                loading.set(true);
                error.set(None);
                let file = file.clone();
                match selected_mode {
                    PlaybackOpenMode::Raw => {
                        playback.set(Some(build_admin_local_media_raw_playback(&file)));
                        loading.set(false);
                    },
                    PlaybackOpenMode::Compatible => {
                        spawn_local(async move {
                            match open_admin_local_media_playback(&file).await {
                                Ok(response) => playback.set(Some(response)),
                                Err(err) => error.set(Some(err)),
                            }
                            loading.set(false);
                        });
                    },
                }
            }
            || ()
        });
    }

    {
        let playback = playback.clone();
        let error = error.clone();
        use_effect_with((*playback).clone(), move |playback_state| {
            let interval = playback_state.clone().and_then(|playback_state| {
                if playback_state.status != LocalMediaPlaybackStatus::Preparing {
                    return None;
                }

                playback_state.job_id.clone().map(|job_id| {
                    Interval::new(1500, move || {
                        let playback = playback.clone();
                        let error = error.clone();
                        let job_id = job_id.clone();
                        spawn_local(async move {
                            match fetch_admin_local_media_job_status(&job_id).await {
                                Ok(job) => {
                                    let next = LocalMediaPlaybackOpenResponse {
                                        status: job.status,
                                        mode: job.mode,
                                        job_id: Some(job.job_id),
                                        player_url: job.player_url,
                                        title: playback
                                            .as_ref()
                                            .as_ref()
                                            .map(|value| value.title.clone())
                                            .unwrap_or_else(|| "Preparing".to_string()),
                                        duration_seconds: job.duration_seconds,
                                        detail: job.detail,
                                        error: job.error,
                                    };
                                    playback.set(Some(next));
                                },
                                Err(err) => error.set(Some(err)),
                            }
                        });
                    })
                })
            });
            move || drop(interval)
        });
    }

    {
        let player_host = player_host.clone();
        let file = file.clone();
        use_effect_with((*playback).clone(), move |playback_state| {
            let mounted = if let Some(playback_state) = playback_state.clone() {
                if playback_state.status == LocalMediaPlaybackStatus::Ready {
                    if let (Some(player_url), Some(mode), Some(element)) = (
                        playback_state.player_url.clone(),
                        playback_state.mode,
                        player_host.cast::<HtmlElement>(),
                    ) {
                        let storage_key = format!("sf-local-media-progress:{file}");
                        let mode_name = match mode {
                            LocalMediaPlaybackMode::Raw => "raw",
                            LocalMediaPlaybackMode::Hls => "hls",
                        };
                        sf_local_media_player_mount(
                            element.clone(),
                            &player_url,
                            mode_name,
                            &playback_state.title,
                            &storage_key,
                        );
                        Some(element)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            move || {
                if let Some(element) = mounted {
                    sf_local_media_player_unmount(element);
                }
            }
        });
    }

    let back_to_browser = {
        let navigator = navigator.clone();
        let parent_dir = parent_dir(&file);
        Callback::from(move |_| {
            if let Some(nav) = navigator.clone() {
                let _ = nav.push_with_query(
                    &Route::AdminLocalMedia,
                    &super::admin_local_media::AdminLocalMediaQuery {
                        dir: parent_dir.clone(),
                    },
                );
            }
        })
    };

    let select_raw = {
        let selected_mode = selected_mode.clone();
        Callback::from(move |_| selected_mode.set(PlaybackOpenMode::Raw))
    };

    let select_compatible = {
        let selected_mode = selected_mode.clone();
        Callback::from(move |_| selected_mode.set(PlaybackOpenMode::Compatible))
    };

    let file_name = Path::new(&file)
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToString::to_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "播放器".to_string());
    let page_title = (*playback)
        .as_ref()
        .map(|value| value.title.clone())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| file_name.clone());
    let folder_label = parent_dir(&file)
        .map(|dir| format!("/{dir}"))
        .unwrap_or_else(|| "根目录".to_string());

    let (status_label, status_tone) = if *loading {
        ("正在打开", "neutral")
    } else if (*error).is_some() {
        ("出错", "err")
    } else {
        match (*playback).as_ref().map(|value| &value.status) {
            Some(LocalMediaPlaybackStatus::Ready) => ("就绪", "ok"),
            Some(LocalMediaPlaybackStatus::Preparing) => ("转码准备中", "warn"),
            Some(LocalMediaPlaybackStatus::Failed) => ("准备失败", "err"),
            None => ("空闲", "neutral"),
        }
    };
    let stream_label = (*playback)
        .as_ref()
        .and_then(|value| value.mode)
        .map(|mode| match mode {
            LocalMediaPlaybackMode::Raw => "原始流 · raw",
            LocalMediaPlaybackMode::Hls => "HLS 转码 · hls",
        });
    let duration_label = (*playback)
        .as_ref()
        .and_then(|value| value.duration_seconds)
        .map(format_duration);
    let job_label = (*playback).as_ref().and_then(|value| value.job_id.clone());

    let stage = if *loading {
        html! {
            <div class="media-stage__state" aria-busy="true" aria-live="polite">
                <span class="media-stage__spinner" aria-hidden="true"></span>
                <p class="media-stage__title">{ "正在打开播放器…" }</p>
            </div>
        }
    } else if let Some(err) = (*error).clone() {
        html! {
            <div class="media-stage__state media-stage__state--err" role="alert">
                <i class="fas fa-triangle-exclamation media-stage__icon" aria-hidden="true"></i>
                <p class="media-stage__title">{ "无法打开这个文件" }</p>
                <p class="media-stage__detail">{ err }</p>
            </div>
        }
    } else if let Some(playback) = (*playback).clone() {
        match playback.status {
            LocalMediaPlaybackStatus::Preparing => html! {
                <div class="media-stage__state" aria-live="polite">
                    <p class="media-stage__eyebrow">{ "兼容模式 · 准备中" }</p>
                    <p class="media-stage__title">{ playback.title }</p>
                    <p class="media-stage__detail">
                        { playback.detail.unwrap_or_else(|| "后端正在准备适合移动端的兼容播放流。".to_string()) }
                    </p>
                    <span class="media-stage__progress" aria-hidden="true"><span></span></span>
                </div>
            },
            LocalMediaPlaybackStatus::Failed => html! {
                <div class="media-stage__state media-stage__state--err" role="alert">
                    <i class="fas fa-circle-exclamation media-stage__icon" aria-hidden="true"></i>
                    <p class="media-stage__title">{ "播放准备失败" }</p>
                    <p class="media-stage__detail">
                        { playback.error.unwrap_or_else(|| "Playback preparation failed".to_string()) }
                    </p>
                </div>
            },
            LocalMediaPlaybackStatus::Ready => html! {
                <div ref={player_host} class="media-stage__player"></div>
            },
        }
    } else {
        Html::default()
    };

    let is_ready = !*loading
        && (*error).is_none()
        && (*playback)
            .as_ref()
            .is_some_and(|value| value.status == LocalMediaPlaybackStatus::Ready);

    html! {
        <AdminShell active={AdminNav::LocalMedia}>
            <div class="media-page media-page--player">
                <AdminPageHeader
                    eyebrow="工具"
                    title={AttrValue::from(page_title)}
                    description="默认使用浏览器原始播放；只有浏览器无法正常播放时，再切换到兼容模式。"
                >
                    <div class="media-seg" role="group" aria-label="播放模式">
                        <button
                            type="button"
                            class={classes!("media-seg__btn", (*selected_mode == PlaybackOpenMode::Raw).then_some("is-active"))}
                            aria-pressed={(*selected_mode == PlaybackOpenMode::Raw).to_string()}
                            onclick={select_raw}
                        >
                            { "原始" }
                        </button>
                        <button
                            type="button"
                            class={classes!("media-seg__btn", (*selected_mode == PlaybackOpenMode::Compatible).then_some("is-active"))}
                            aria-pressed={(*selected_mode == PlaybackOpenMode::Compatible).to_string()}
                            onclick={select_compatible}
                        >
                            { "兼容" }
                        </button>
                    </div>
                    <button type="button" class="adm-btn adm-btn--secondary" onclick={back_to_browser.clone()}>
                        <i class="fas fa-arrow-left" aria-hidden="true"></i>
                        <span>{ "返回文件夹" }</span>
                    </button>
                </AdminPageHeader>
                <nav class="media-crumbs media-crumbs--player" aria-label="文件路径">
                    <button type="button" class="media-crumbs__item" onclick={back_to_browser}>
                        <i class="fas fa-folder" aria-hidden="true"></i>
                        <span class="media-crumbs__text">{ folder_label }</span>
                    </button>
                    <span class="media-crumbs__sep" aria-hidden="true">{ "/" }</span>
                    <span class="media-crumbs__item is-current" aria-current="page" title={file.clone()}>
                        <span class="media-crumbs__text">{ file_name }</span>
                    </span>
                </nav>
                <section class="media-theater" aria-label="播放器">
                    <div class={classes!("media-stage", is_ready.then_some("is-ready"))}>
                        { stage }
                    </div>
                    <div class="media-theater__info">
                        <dl class="adm-kv media-kv">
                            <div>
                                <dt>{ "状态" }</dt>
                                <dd>
                                    <span class={classes!(
                                        "adm-pill",
                                        format!("adm-pill--{status_tone}"),
                                        (status_tone == "warn").then_some("adm-pill--live"),
                                    )}>
                                        { status_label }
                                    </span>
                                </dd>
                            </div>
                            if let Some(stream_label) = stream_label {
                                <div>
                                    <dt>{ "播放流" }</dt>
                                    <dd class="media-mono">{ stream_label }</dd>
                                </div>
                            }
                            if let Some(duration_label) = duration_label {
                                <div>
                                    <dt>{ "时长" }</dt>
                                    <dd class="media-mono">{ duration_label }</dd>
                                </div>
                            }
                            if let Some(job_label) = job_label {
                                <div>
                                    <dt>{ "转码任务" }</dt>
                                    <dd class="media-mono media-kv__wrap" title={job_label.clone()}>{ job_label }</dd>
                                </div>
                            }
                            <div class="media-kv__wide">
                                <dt>{ "相对路径" }</dt>
                                <dd class="media-mono media-kv__wrap" title={file.clone()}>{ file.clone() }</dd>
                            </div>
                        </dl>
                        if is_ready {
                            <p class="media-theater__tip">
                                <i class="fas fa-hand-pointer" aria-hidden="true"></i>
                                <span>{ "移动端支持：长按 2× 倍速、左右滑动快进/快退、双击播放/暂停。" }</span>
                            </p>
                        }
                    </div>
                </section>
            </div>
        </AdminShell>
    }
}

fn format_duration(duration_seconds: f64) -> String {
    let total_seconds = duration_seconds.max(0.0).round() as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlaybackOpenMode {
    Raw,
    Compatible,
}

fn parent_dir(file: &str) -> Option<String> {
    let path = Path::new(file);
    let parent = path.parent()?;
    let parts = parent
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}
