use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api::{
        fetch_admin_local_media_list, LocalMediaEntry, LocalMediaEntryKind, LocalMediaListResponse,
    },
    components::empty_state::EmptyState,
    pages::admin::shell::{AdminNav, AdminPageHeader, AdminShell},
    router::Route,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AdminLocalMediaQuery {
    #[serde(default)]
    pub dir: Option<String>,
}

#[function_component(AdminLocalMediaPage)]
pub fn admin_local_media_page() -> Html {
    let navigator = use_navigator();
    let location = use_location();
    let current_dir = location
        .as_ref()
        .and_then(|loc| loc.query::<AdminLocalMediaQuery>().ok())
        .and_then(|query| query.dir)
        .unwrap_or_default();

    let data = use_state(|| None::<LocalMediaListResponse>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let refresh_nonce = use_state(|| 0u64);

    {
        let data = data.clone();
        let loading = loading.clone();
        let error = error.clone();
        let current_dir = current_dir.clone();
        let refresh_key = *refresh_nonce;
        use_effect_with((current_dir.clone(), refresh_key), move |(dir, _)| {
            loading.set(true);
            error.set(None);
            let dir = dir.clone();
            spawn_local(async move {
                match fetch_admin_local_media_list(
                    if dir.is_empty() { None } else { Some(dir.as_str()) },
                    None,
                    None,
                )
                .await
                {
                    Ok(response) => data.set(Some(response)),
                    Err(err) => error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let open_dir = {
        let navigator = navigator.clone();
        Callback::from(move |dir: String| {
            if let Some(nav) = navigator.clone() {
                let _ = nav.push_with_query(&Route::AdminLocalMedia, &AdminLocalMediaQuery {
                    dir: if dir.is_empty() { None } else { Some(dir) },
                });
            }
        })
    };

    let open_player = {
        let navigator = navigator.clone();
        Callback::from(move |file: String| {
            if let Some(nav) = navigator.clone() {
                let _ = nav.push_with_query(
                    &Route::AdminLocalMediaPlayer,
                    &AdminLocalMediaPlayerQuery {
                        file: Some(file),
                    },
                );
            }
        })
    };

    let refresh_dir = {
        let refresh_nonce = refresh_nonce.clone();
        Callback::from(move |_| {
            refresh_nonce.set(*refresh_nonce + 1);
        })
    };

    let summary = (*data)
        .as_ref()
        .filter(|response| response.configured && !*loading && (*error).is_none())
        .map(|response| {
            let dirs = response
                .entries
                .iter()
                .filter(|entry| entry.kind == LocalMediaEntryKind::Directory)
                .count();
            let videos = response.entries.len() - dirs;
            let more = if response.has_more {
                format!(" · 共 {}", response.total)
            } else {
                String::new()
            };
            format!("{dirs} 个文件夹 · {videos} 个视频{more}")
        });

    let content = if *loading {
        html! {
            <div class="media-section" aria-busy="true" aria-live="polite">
                <span class="sr-only">{ "正在加载本地媒体…" }</span>
                <div class="media-grid media-grid--video">
                    { for (0..6).map(|_| html! {
                        <div class="media-card media-card--skel" aria-hidden="true">
                            <div class="media-card__thumb adm-skel"></div>
                            <div class="media-card__body">
                                <div class="adm-skel media-skel-line"></div>
                                <div class="adm-skel media-skel-line media-skel-line--short"></div>
                            </div>
                        </div>
                    }) }
                </div>
            </div>
        }
    } else if let Some(err) = (*error).clone() {
        html! {
            <div class="adm-banner adm-banner--err media-banner" role="alert">
                <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                <div class="adm-banner__body media-banner__body">
                    <strong>{ "目录加载失败" }</strong>
                    <span class="media-banner__detail">{ err }</span>
                </div>
                <button type="button" class="adm-btn adm-btn--secondary adm-btn--sm" onclick={refresh_dir.reform(|_| ())}>
                    { "重试" }
                </button>
            </div>
        }
    } else if let Some(response) = (*data).clone() {
        if !response.configured {
            html! {
                <div class="adm-banner adm-banner--warn media-banner" role="status">
                    <i class="fas fa-circle-info" aria-hidden="true"></i>
                    <div class="adm-banner__body media-banner__body">
                        <strong>{ "本地媒体根目录未配置" }</strong>
                        <span class="media-banner__detail">
                            { "当前构建已启用本地媒体，但后端没有配置 " }
                            <code class="media-mono">{ "STATICFLOW_LOCAL_MEDIA_ROOT" }</code>
                            { "。" }
                        </span>
                    </div>
                </div>
            }
        } else if response.entries.is_empty() {
            html! {
                <div class="adm-card media-empty-card">
                    <EmptyState icon="fa-folder-open" title="这个目录还是空的" hint="在上方上传视频，或返回上一级打开其它文件夹。" />
                </div>
            }
        } else {
            let (dirs, videos): (Vec<_>, Vec<_>) = response
                .entries
                .iter()
                .partition(|entry| entry.kind == LocalMediaEntryKind::Directory);
            html! {
                <>
                    if !dirs.is_empty() {
                        <section class="media-section" aria-label="文件夹">
                            <div class="media-section__head">
                                <h2 class="media-section__title">{ "文件夹" }</h2>
                                <span class="media-section__count">{ dirs.len().to_string() }</span>
                            </div>
                            <div class="media-grid media-grid--dir">
                                { for dirs.iter().map(|entry| html! {
                                    <LocalMediaCard
                                        entry={(*entry).clone()}
                                        on_open_dir={open_dir.clone()}
                                        on_open_player={open_player.clone()}
                                    />
                                }) }
                            </div>
                        </section>
                    }
                    if !videos.is_empty() {
                        <section class="media-section" aria-label="视频">
                            <div class="media-section__head">
                                <h2 class="media-section__title">{ "视频" }</h2>
                                <span class="media-section__count">{ videos.len().to_string() }</span>
                            </div>
                            <div class="media-grid media-grid--video">
                                { for videos.iter().map(|entry| html! {
                                    <LocalMediaCard
                                        entry={(*entry).clone()}
                                        on_open_dir={open_dir.clone()}
                                        on_open_player={open_player.clone()}
                                    />
                                }) }
                            </div>
                        </section>
                    }
                </>
            }
        }
    } else {
        Html::default()
    };

    let breadcrumb = render_breadcrumbs(&current_dir, open_dir.clone());

    html! {
        <AdminShell active={AdminNav::LocalMedia}>
            <div class="media-page">
                <AdminPageHeader
                    eyebrow="工具"
                    title="本地媒体"
                    description="浏览已配置的本地媒体根目录：文件夹轻量浏览，视频直接进入独立播放页。"
                >
                    <button
                        type="button"
                        class="adm-btn adm-btn--secondary"
                        onclick={refresh_dir.reform(|_| ())}
                    >
                        <i class={classes!("fas", "fa-rotate-right", (*loading).then_some("fa-spin"))} aria-hidden="true"></i>
                        <span>{ "刷新" }</span>
                    </button>
                </AdminPageHeader>
                <div class="media-pathbar">
                    { breadcrumb }
                    if let Some(summary) = summary {
                        <span class="media-pathbar__summary">{ summary }</span>
                    }
                </div>
                <crate::components::admin_local_media_uploads::AdminLocalMediaUploads
                    current_dir={current_dir.clone()}
                    on_refresh_dir={refresh_dir}
                />
                { content }
            </div>
        </AdminShell>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AdminLocalMediaPlayerQuery {
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Properties, PartialEq, Clone)]
struct LocalMediaCardProps {
    entry: LocalMediaEntry,
    on_open_dir: Callback<String>,
    on_open_player: Callback<String>,
}

#[function_component(LocalMediaCard)]
fn local_media_card(props: &LocalMediaCardProps) -> Html {
    let poster_failed = use_state(|| false);
    let entry = props.entry.clone();
    let relative_path = entry.relative_path.clone();
    let click = match entry.kind {
        LocalMediaEntryKind::Directory => {
            let open_dir = props.on_open_dir.clone();
            Callback::from(move |_| open_dir.emit(relative_path.clone()))
        },
        LocalMediaEntryKind::Video => {
            let open_player = props.on_open_player.clone();
            Callback::from(move |_| open_player.emit(relative_path.clone()))
        },
    };
    let is_dir = entry.kind == LocalMediaEntryKind::Directory;
    let action_label = match entry.kind {
        LocalMediaEntryKind::Directory => "打开文件夹",
        LocalMediaEntryKind::Video => "播放",
    };
    let icon_class = match entry.kind {
        LocalMediaEntryKind::Directory => "fa-folder",
        LocalMediaEntryKind::Video => "fa-film",
    };
    let show_poster =
        entry.kind == LocalMediaEntryKind::Video && entry.poster_url.is_some() && !*poster_failed;
    let poster_error = {
        let poster_failed = poster_failed.clone();
        Callback::from(move |_| poster_failed.set(true))
    };
    let aria_label = format!("{action_label}：{}", entry.name);

    if is_dir {
        return html! {
            <button
                type="button"
                onclick={click}
                class="media-card media-card--dir"
                title={entry.relative_path.clone()}
                aria-label={aria_label}
            >
                <span class="media-card__glyph" aria-hidden="true">
                    <i class={classes!("fas", icon_class)}></i>
                </span>
                <span class="media-card__body">
                    <span class="media-card__name" title={entry.name.clone()}>{ entry.name.clone() }</span>
                    <span class="media-card__meta">
                        <span>{ format_modified(entry.modified_at_ms) }</span>
                    </span>
                </span>
                <i class="fas fa-chevron-right media-card__chev" aria-hidden="true"></i>
            </button>
        };
    }

    html! {
        <button
            type="button"
            onclick={click}
            class="media-card media-card--video"
            aria-label={aria_label}
        >
            <span class="media-card__thumb">
                if show_poster {
                    <img
                        src={entry.poster_url.clone().unwrap_or_default()}
                        alt=""
                        loading="lazy"
                        class="media-card__poster"
                        onerror={poster_error}
                    />
                } else {
                    <span class="media-card__placeholder" aria-hidden="true">
                        <i class={classes!("fas", icon_class)}></i>
                    </span>
                }
                <span class="media-card__play" aria-hidden="true">
                    <i class="fas fa-play"></i>
                </span>
            </span>
            <span class="media-card__body">
                <span class="media-card__name" title={entry.name.clone()}>{ entry.name.clone() }</span>
                <span class="media-card__path" title={entry.relative_path.clone()}>{ entry.relative_path.clone() }</span>
                <span class="media-card__meta">
                    <span>{ format_entry_meta(entry.size_bytes, entry.extension.as_deref()) }</span>
                    <span>{ format_modified(entry.modified_at_ms) }</span>
                </span>
            </span>
        </button>
    }
}

fn render_breadcrumbs(current_dir: &str, open_dir: Callback<String>) -> Html {
    let segments = current_dir
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let root_click = {
        let open_dir = open_dir.clone();
        Callback::from(move |_| open_dir.emit(String::new()))
    };
    let last_index = segments.len().checked_sub(1);

    let mut built = Vec::new();
    html! {
        <nav class="media-crumbs" aria-label="目录路径">
            <button
                type="button"
                class={classes!("media-crumbs__item", segments.is_empty().then_some("is-current"))}
                aria-current={segments.is_empty().then_some("page")}
                onclick={root_click}
            >
                <i class="fas fa-hard-drive" aria-hidden="true"></i>
                <span>{ "根目录" }</span>
            </button>
            { for segments.iter().enumerate().map(|(index, segment)| {
                built.push((*segment).to_string());
                let next_dir = built.join("/");
                let open_dir = open_dir.clone();
                let is_current = Some(index) == last_index;
                html! {
                    <>
                        <span class="media-crumbs__sep" aria-hidden="true">{ "/" }</span>
                        <button
                            type="button"
                            class={classes!("media-crumbs__item", is_current.then_some("is-current"))}
                            aria-current={is_current.then_some("page")}
                            title={next_dir.clone()}
                            onclick={Callback::from(move |_| open_dir.emit(next_dir.clone()))}
                        >
                            <span class="media-crumbs__text">{ (*segment).to_string() }</span>
                        </button>
                    </>
                }
            }) }
        </nav>
    }
}

fn format_entry_meta(size_bytes: Option<u64>, extension: Option<&str>) -> String {
    let size = size_bytes
        .map(format_bytes)
        .unwrap_or_else(|| "文件夹".to_string());
    match extension {
        Some(extension) => format!("{} · {}", extension.to_ascii_uppercase(), size),
        None => size,
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{} B", bytes as u64)
    }
}

fn format_modified(timestamp_ms: Option<i64>) -> String {
    timestamp_ms
        .map(|value| js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(value as f64)))
        .map(|date| {
            format!(
                "{:04}-{:02}-{:02}",
                date.get_full_year(),
                date.get_month() + 1,
                date.get_date()
            )
        })
        .unwrap_or_else(|| "—".to_string())
}
