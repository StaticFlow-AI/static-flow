pub mod shell;

use std::collections::{HashMap, HashSet};

use gloo_events::EventListener;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use self::shell::{
    admin_nav_href, navigate_to_admin_section, AdminNav, AdminPageHeader, AdminShell,
};
use crate::{
    api::{
        admin_approve_and_run_article_request, admin_approve_and_run_comment_task,
        admin_approve_and_run_music_wish, admin_approve_comment_task, admin_cleanup_api_behavior,
        admin_cleanup_comments, admin_delete_article_request, admin_delete_comment_task,
        admin_delete_music_wish, admin_reject_article_request, admin_reject_comment_task,
        admin_reject_music_wish, admin_reset_memory_profiler, admin_retry_article_request,
        admin_retry_comment_task, admin_retry_music_wish, admin_update_memory_profiler_config,
        delete_admin_published_comment, fetch_admin_api_behavior_config,
        fetch_admin_api_behavior_events, fetch_admin_api_behavior_overview,
        fetch_admin_article_requests, fetch_admin_comment_audit_logs,
        fetch_admin_comment_runtime_config, fetch_admin_comment_task,
        fetch_admin_comment_task_ai_output, fetch_admin_comment_tasks_grouped,
        fetch_admin_compaction_runtime_config, fetch_admin_memory_profiler_functions,
        fetch_admin_memory_profiler_modules, fetch_admin_memory_profiler_overview,
        fetch_admin_memory_profiler_stacks, fetch_admin_music_runtime_config,
        fetch_admin_music_wishes, fetch_admin_published_comments,
        fetch_admin_view_analytics_config, patch_admin_comment_task, patch_admin_published_comment,
        update_admin_api_behavior_config, update_admin_comment_runtime_config,
        update_admin_compaction_runtime_config, update_admin_music_runtime_config,
        update_admin_view_analytics_config, AdminApiBehaviorCleanupRequest, AdminApiBehaviorEvent,
        AdminApiBehaviorEventsQuery, AdminApiBehaviorOverviewResponse, AdminCleanupRequest,
        AdminCommentAuditLog, AdminCommentTask, AdminCommentTaskAiOutputResponse,
        AdminCommentTaskGroup, AdminPatchCommentTaskRequest, AdminPatchPublishedCommentRequest,
        AdminTaskActionRequest, ApiBehaviorBucket, ApiBehaviorConfig, ArticleComment,
        ArticleRequestItem, ArticleViewPoint, CommentRuntimeConfig, CompactionRuntimeConfig,
        MemoryFunctionEntry, MemoryFunctionReport, MemoryModuleEntry, MemoryModuleReport,
        MemoryProfilerConfigSnapshot, MemoryProfilerConfigUpdate, MemoryProfilerOverview,
        MemoryStackEntry, MemoryStackReport, MusicRuntimeConfig, MusicWishItem,
        ViewAnalyticsConfig,
    },
    components::{
        copy_button::CopyButton,
        drawer::Drawer,
        empty_state::EmptyState,
        loading_spinner::{LoadingSpinner, SpinnerSize},
        modal::ConfirmModal,
        pagination::Pagination,
        search_box::SearchBox,
        status_badge::StatusBadge,
        toast::use_toast,
        view_trend_chart::ViewTrendChart,
    },
    pages::{admin_llm_access_handoff::llm_access_frontend_url, llm_access_shared::format_ms},
    router::Route,
};

/// Data-fetch buckets backing the nine `/admin` sections.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum LoadBucket {
    Core,
    Audit,
    Behavior,
    RuntimeMemory,
    MusicWishes,
    ArticleRequests,
}

fn buckets_for_section(nav: AdminNav) -> &'static [LoadBucket] {
    match nav {
        AdminNav::Overview => {
            &[LoadBucket::Core, LoadBucket::MusicWishes, LoadBucket::ArticleRequests]
        },
        AdminNav::CommentTasks | AdminNav::PublishedComments | AdminNav::SiteConfig => {
            &[LoadBucket::Core]
        },
        AdminNav::AuditLogs => &[LoadBucket::Audit],
        AdminNav::MusicWishes => &[LoadBucket::MusicWishes],
        AdminNav::ArticleRequests => &[LoadBucket::ArticleRequests],
        AdminNav::ApiBehavior => &[LoadBucket::Behavior],
        AdminNav::RuntimeMemory => &[LoadBucket::RuntimeMemory],
        AdminNav::LocalMedia => &[],
    }
}

fn read_current_section() -> AdminNav {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .and_then(|h| AdminNav::from_hash(&h))
        .unwrap_or(AdminNav::Overview)
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;
    let bytes_f = bytes as f64;

    if bytes_f >= TB {
        format!("{:.2} TB", bytes_f / TB)
    } else if bytes_f >= GB {
        format!("{:.2} GB", bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{:.2} MB", bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{:.2} KB", bytes_f / KB)
    } else {
        format!("{bytes} B")
    }
}

fn format_ratio_percent(ratio: f64) -> String {
    format!("{:.2}%", ratio * 100.0)
}

fn article_request_has_article(req: &ArticleRequestItem) -> bool {
    req.ingested_article_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|id| !id.is_empty())
}

fn http_status_pill(code: i32) -> Html {
    let tone = if (200..300).contains(&code) {
        "adm-pill adm-pill--ok adm-mono"
    } else if (300..400).contains(&code) {
        "adm-pill adm-pill--info adm-mono"
    } else if (400..500).contains(&code) {
        "adm-pill adm-pill--warn adm-mono"
    } else {
        "adm-pill adm-pill--err adm-mono"
    };
    html! { <span class={tone}>{ code }</span> }
}

fn table_skeleton(rows: usize) -> Html {
    html! {
        <div class="adm-card__body adm-stack-sm" aria-hidden="true">
            { for (0..rows).map(|_| html! { <span class="adm-skel adm-skel--row"></span> }) }
        </div>
    }
}

fn kpi_skeleton(count: usize) -> Html {
    html! {
        <div class="adm-kpis" aria-hidden="true">
            { for (0..count).map(|_| html! {
                <div class="adm-kpi">
                    <span class="adm-skel" style="width: 72px;"></span>
                    <span class="adm-skel adm-skel--lg" style="width: 108px; margin-top: 10px;"></span>
                    <span class="adm-skel" style="width: 132px; margin-top: 8px;"></span>
                </div>
            }) }
        </div>
    }
}

/// Destructive-ish actions fired from a Music Wish row.
#[derive(Clone, Copy, PartialEq, Eq)]
enum WishAction {
    Approve,
    Reject,
    Retry,
    Delete,
}

/// Parallel action dispatcher enum for Article Request rows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ArticleRequestAction {
    Approve,
    Reject,
    Retry,
    Delete,
}

#[derive(Properties, PartialEq)]
struct MusicWishRowProps {
    wish: MusicWishItem,
    inflight: bool,
    on_action: Callback<(String, WishAction)>,
}

#[function_component(MusicWishRow)]
fn music_wish_row(props: &MusicWishRowProps) -> Html {
    let wish = &props.wish;
    let wid = wish.wish_id.clone();
    let status = wish.status.clone();
    let inflight = props.inflight;

    let dispatch = |action: WishAction| {
        let on_action = props.on_action.clone();
        let wid = wid.clone();
        Callback::from(move |_| on_action.emit((wid.clone(), action)))
    };

    let artist = wish
        .artist_hint
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("—");

    html! {
        <tr>
            <td style="max-width: 240px;">
                <span class="adm-trunc" title={wish.song_name.clone()}>{ &wish.song_name }</span>
                if !wish.wish_message.trim().is_empty() {
                    <span class="adm-trunc adm-faint" style="font-size: 12px;" title={wish.wish_message.clone()}>
                        { &wish.wish_message }
                    </span>
                }
            </td>
            <td style="max-width: 160px;">
                <span class="adm-trunc" title={artist.to_string()}>{ artist }</span>
            </td>
            <td>{ &wish.nickname }</td>
            <td><StatusBadge status={status.clone()} /></td>
            <td class="adm-muted">{ &wish.ip_region }</td>
            <td class="num">{ format_ms(wish.created_at) }</td>
            <td class="actions">
                <div class="adm-row-actions">
                    if status == "pending" {
                        <button type="button" class="adm-btn adm-btn--primary adm-btn--sm"
                            disabled={inflight} onclick={dispatch(WishAction::Approve)}>
                            { "批准并执行" }
                        </button>
                        <button type="button" class="adm-btn adm-btn--ghost adm-btn--sm"
                            disabled={inflight} onclick={dispatch(WishAction::Reject)}>
                            { "驳回" }
                        </button>
                    }
                    if status == "failed" {
                        <button type="button" class="adm-btn adm-btn--primary adm-btn--sm"
                            disabled={inflight} onclick={dispatch(WishAction::Retry)}>
                            { "重试" }
                        </button>
                    }
                    if status == "done" || status == "running" || status == "failed" {
                        <Link<Route>
                            to={Route::AdminMusicWishRuns { wish_id: wid.clone() }}
                            classes={classes!("adm-btn", "adm-btn--ghost", "adm-btn--sm")}
                        >
                            { "AI 日志 ↗" }
                        </Link<Route>>
                    }
                    <button type="button" class="adm-btn adm-btn--danger adm-btn--sm"
                        disabled={inflight} onclick={dispatch(WishAction::Delete)}>
                        { "删除" }
                    </button>
                </div>
            </td>
        </tr>
    }
}

#[derive(Properties, PartialEq)]
struct ArticleRequestRowProps {
    request: ArticleRequestItem,
    inflight: bool,
    on_action: Callback<(String, ArticleRequestAction)>,
}

#[function_component(ArticleRequestRow)]
fn article_request_row(props: &ArticleRequestRowProps) -> Html {
    let req = &props.request;
    let rid = req.request_id.clone();
    let status = req.status.clone();
    let inflight = props.inflight;

    let dispatch = |action: ArticleRequestAction| {
        let on_action = props.on_action.clone();
        let rid = rid.clone();
        Callback::from(move |_| on_action.emit((rid.clone(), action)))
    };

    let url_display: String = if req.article_url.chars().count() > 52 {
        format!("{}…", req.article_url.chars().take(49).collect::<String>())
    } else {
        req.article_url.clone()
    };

    let title_hint = req
        .title_hint
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("—");

    html! {
        <tr>
            <td style="max-width: 260px;" title={req.article_url.clone()}>
                <a href={req.article_url.clone()} target="_blank" rel="noopener noreferrer"
                    class="adm-link adm-mono adm-trunc">
                    { url_display }
                </a>
                if !req.request_message.trim().is_empty() {
                    <span class="adm-trunc adm-faint" style="font-size: 12px;" title={req.request_message.clone()}>
                        { &req.request_message }
                    </span>
                }
            </td>
            <td style="max-width: 180px;">
                <span class="adm-trunc" title={title_hint.to_string()}>{ title_hint }</span>
            </td>
            <td>{ &req.nickname }</td>
            <td>
                if req.status == "done" && !article_request_has_article(req) {
                    <StatusBadge status="pending" label={Some(AttrValue::from("done/no-article"))} />
                } else {
                    <StatusBadge status={req.status.clone()} />
                }
            </td>
            <td class="adm-muted">{ &req.ip_region }</td>
            <td class="num">{ format_ms(req.created_at) }</td>
            <td class="actions">
                <div class="adm-row-actions">
                    if status == "pending" {
                        <button type="button" class="adm-btn adm-btn--primary adm-btn--sm"
                            disabled={inflight} onclick={dispatch(ArticleRequestAction::Approve)}>
                            { "批准并执行" }
                        </button>
                        <button type="button" class="adm-btn adm-btn--ghost adm-btn--sm"
                            disabled={inflight} onclick={dispatch(ArticleRequestAction::Reject)}>
                            { "驳回" }
                        </button>
                    }
                    if status == "failed" {
                        <button type="button" class="adm-btn adm-btn--primary adm-btn--sm"
                            disabled={inflight} onclick={dispatch(ArticleRequestAction::Retry)}>
                            { "重试" }
                        </button>
                    }
                    if let Some(article_id) = req
                        .ingested_article_id
                        .as_ref()
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                    {
                        <Link<Route>
                            to={Route::ArticleDetail { id: article_id.to_string() }}
                            classes={classes!("adm-btn", "adm-btn--ghost", "adm-btn--sm")}
                        >
                            { "查看文章 ↗" }
                        </Link<Route>>
                    }
                    if status == "done" || status == "running" || status == "failed" {
                        <Link<Route>
                            to={Route::AdminArticleRequestRuns { request_id: rid.clone() }}
                            classes={classes!("adm-btn", "adm-btn--ghost", "adm-btn--sm")}
                        >
                            { "AI 日志 ↗" }
                        </Link<Route>>
                    }
                    <button type="button" class="adm-btn adm-btn--danger adm-btn--sm"
                        disabled={inflight} onclick={dispatch(ArticleRequestAction::Delete)}>
                        { "删除" }
                    </button>
                </div>
            </td>
        </tr>
    }
}

fn to_view_points(buckets: &[ApiBehaviorBucket]) -> Vec<ArticleViewPoint> {
    buckets
        .iter()
        .map(|item| ArticleViewPoint {
            key: item.key.clone(),
            views: item.count,
        })
        .collect()
}

fn behavior_distribution_card(title: &str, items: &[ApiBehaviorBucket], path_like: bool) -> Html {
    let max_count = items.iter().map(|item| item.count).max().unwrap_or(1);
    let key_class = if path_like {
        classes!("admin-dist-item__key", "admin-dist-item__key--path")
    } else {
        classes!("admin-dist-item__key")
    };

    html! {
        <article class="admin-dist-card">
            <header class="admin-dist-card__header">
                <span>{ title }</span>
                <span class="admin-dist-card__count">{ format!("{} 项", items.len()) }</span>
            </header>
            if items.is_empty() {
                <div class="adm-card__body adm-faint" style="font-size: 12.5px;">{ "暂无统计数据" }</div>
            } else {
                <ul class="admin-dist-list">
                    { for items.iter().map(|item| {
                        let fill = if max_count == 0 {
                            0.0
                        } else {
                            item.count as f64 / max_count as f64
                        };
                        let fill_style = format!("--fill: {:.4};", fill);
                        html! {
                            <li class="dist-bar admin-dist-item" style={fill_style}>
                                <span class={key_class.clone()} title={item.key.clone()}>{ &item.key }</span>
                                <span class="admin-dist-item__value">{ item.count }</span>
                            </li>
                        }
                    }) }
                </ul>
            }
        </article>
    }
}

fn memory_function_list(entries: &[MemoryFunctionEntry]) -> Html {
    let max_bytes = entries
        .iter()
        .map(|e| e.live_bytes_estimate)
        .max()
        .unwrap_or(1)
        .max(1);
    html! {
        <ul class="admin-dist-list">
            { for entries.iter().map(|entry| {
                let fill = entry.live_bytes_estimate as f64 / max_bytes as f64;
                let style = format!("--fill:{:.3}", fill);
                html! {
                    <li class="dist-bar admin-dist-item" {style}>
                        <div class="admin-mem-name" style="flex:1;min-width:0">
                            <details>
                                <summary title={entry.function.clone()}>{ &entry.function }</summary>
                                <div class="admin-mem-full">{ &entry.function }</div>
                            </details>
                            <div class="admin-mem-sub" title={entry.module.clone()}>{ &entry.module }</div>
                        </div>
                        <span class="admin-dist-item__value">
                            { format!("{} ({})", format_bytes(entry.live_bytes_estimate), format_ratio_percent(entry.live_ratio_heap)) }
                        </span>
                    </li>
                }
            }) }
        </ul>
    }
}

fn memory_module_list(entries: &[MemoryModuleEntry]) -> Html {
    let max_bytes = entries
        .iter()
        .map(|e| e.live_bytes_estimate)
        .max()
        .unwrap_or(1)
        .max(1);
    html! {
        <ul class="admin-dist-list">
            { for entries.iter().map(|entry| {
                let fill = entry.live_bytes_estimate as f64 / max_bytes as f64;
                let style = format!("--fill:{:.3}", fill);
                html! {
                    <li class="dist-bar admin-dist-item" {style}>
                        <div class="admin-mem-name" style="flex:1;min-width:0">
                            <details>
                                <summary title={entry.module.clone()}>{ &entry.module }</summary>
                                <div class="admin-mem-full">{ &entry.module }</div>
                            </details>
                            <div class="admin-mem-sub">
                                { format!("fns={} stacks={}", entry.function_count, entry.stack_count) }
                            </div>
                        </div>
                        <span class="admin-dist-item__value">
                            { format!("{} ({})", format_bytes(entry.live_bytes_estimate), format_ratio_percent(entry.live_ratio_heap)) }
                        </span>
                    </li>
                }
            }) }
        </ul>
    }
}

fn memory_stack_list(entries: &[MemoryStackEntry]) -> Html {
    let max_bytes = entries
        .iter()
        .map(|e| e.live_bytes_estimate)
        .max()
        .unwrap_or(1)
        .max(1);
    html! {
        <ul class="admin-dist-list">
            { for entries.iter().map(|entry| {
                let fill = entry.live_bytes_estimate as f64 / max_bytes as f64;
                let style = format!("--fill:{:.3}", fill);
                let top_frame = entry.frames.first().cloned().unwrap_or_default();
                html! {
                    <li class="dist-bar admin-dist-item" {style}>
                        <div class="admin-mem-name" style="flex:1;min-width:0">
                            <details>
                                <summary title={top_frame.clone()}>{ &top_frame }</summary>
                                <div class="admin-mem-full">
                                    { for entry.frames.iter().map(|f| html! {
                                        <div>{ f }</div>
                                    }) }
                                </div>
                            </details>
                            <div class="admin-mem-sub">
                                { format!("alloc={} free={}", entry.alloc_count, entry.free_count) }
                            </div>
                        </div>
                        <span class="admin-dist-item__value">
                            { format!("{} ({})", format_bytes(entry.live_bytes_estimate), format_ratio_percent(entry.live_ratio_heap)) }
                        </span>
                    </li>
                }
            }) }
        </ul>
    }
}

#[function_component(AdminPage)]
pub fn admin_page() -> Html {
    let active_section = use_state(read_current_section);
    let load_error = use_state(|| None::<String>);
    let pending_delete_published = use_state(|| None::<String>);
    let pending_delete_task = use_state(|| None::<String>);
    let pending_delete_wish = use_state(|| None::<String>);
    let pending_delete_request = use_state(|| None::<String>);
    let pending_cleanup_failed = use_state(|| false);
    let pending_cleanup_logs = use_state(|| false);
    let toast = use_toast();

    let view_config = use_state(|| None::<ViewAnalyticsConfig>);
    let comment_config = use_state(|| None::<CommentRuntimeConfig>);
    let music_config = use_state(|| None::<MusicRuntimeConfig>);
    let behavior_config = use_state(|| None::<ApiBehaviorConfig>);
    let compaction_config = use_state(|| None::<CompactionRuntimeConfig>);

    let behavior_overview = use_state(|| None::<AdminApiBehaviorOverviewResponse>);
    let behavior_events = use_state(Vec::<AdminApiBehaviorEvent>::new);
    let behavior_days = use_state(|| "30".to_string());
    let behavior_date = use_state(String::new);
    let behavior_has_more = use_state(|| false);
    let behavior_total = use_state(|| 0_usize);
    let behavior_offset = use_state(|| 0_usize);
    let behavior_path_filter = use_state(String::new);
    let behavior_page_filter = use_state(String::new);
    let behavior_device_filter = use_state(String::new);
    let behavior_status_filter = use_state(String::new);

    let memory_overview = use_state(|| None::<MemoryProfilerOverview>);
    let memory_functions = use_state(|| None::<MemoryFunctionReport>);
    let memory_modules = use_state(|| None::<MemoryModuleReport>);
    let memory_stacks = use_state(|| None::<MemoryStackReport>);
    let memory_config = use_state(|| None::<MemoryProfilerConfigSnapshot>);
    let memory_top = use_state(|| "20".to_string());
    let memory_action_loading = use_state(|| false);

    let task_groups = use_state(Vec::<AdminCommentTaskGroup>::new);
    let grouped_status_counts = use_state(HashMap::<String, usize>::new);
    let status_filter = use_state(String::new);
    let selected_task_id = use_state(|| None::<String>);
    let selected_task = use_state(|| None::<AdminCommentTask>);
    let selected_task_ai_output = use_state(|| None::<AdminCommentTaskAiOutputResponse>);
    let task_action_inflight = use_state(HashSet::<String>::new);

    let published_comments = use_state(Vec::<ArticleComment>::new);
    let selected_published_id = use_state(|| None::<String>);
    let selected_published = use_state(|| None::<ArticleComment>);

    let audit_logs = use_state(Vec::<AdminCommentAuditLog>::new);
    let audit_task_filter = use_state(String::new);
    let audit_action_filter = use_state(String::new);

    let cleanup_days = use_state(|| "30".to_string());
    let loading = use_state(|| false);

    let music_wishes = use_state(Vec::<MusicWishItem>::new);
    let music_wish_action_inflight = use_state(HashSet::<String>::new);
    let music_wish_search = use_state(String::new);
    let article_requests = use_state(Vec::<ArticleRequestItem>::new);
    let article_request_action_inflight = use_state(HashSet::<String>::new);
    let article_request_search = use_state(String::new);
    let saving = use_state(|| false);

    let loaded_buckets = use_state(HashSet::<LoadBucket>::new);
    let bucket_loading = use_state(HashSet::<LoadBucket>::new);

    let refresh_all_seq = use_mut_ref(|| 0_u64);
    let refresh_audit_seq = use_mut_ref(|| 0_u64);
    let refresh_behavior_seq = use_mut_ref(|| 0_u64);
    let refresh_memory_seq = use_mut_ref(|| 0_u64);
    let refresh_music_wishes_seq = use_mut_ref(|| 0_u64);
    let refresh_article_requests_seq = use_mut_ref(|| 0_u64);

    const PAGE_SIZE: usize = 50;
    let tasks_page = use_state(|| 1_usize);
    let published_page = use_state(|| 1_usize);
    let audit_page = use_state(|| 1_usize);
    let behavior_page = use_state(|| 1_usize);
    let music_wish_page = use_state(|| 1_usize);
    let article_request_page = use_state(|| 1_usize);

    let tasks_total = use_state(|| 0_usize);
    let published_total = use_state(|| 0_usize);
    let audit_total = use_state(|| 0_usize);
    let music_wish_total = use_state(|| 0_usize);
    let article_request_total = use_state(|| 0_usize);
    let tasks_has_more = use_state(|| false);
    let published_has_more = use_state(|| false);
    let audit_has_more = use_state(|| false);
    let music_wish_has_more = use_state(|| false);
    let article_request_has_more = use_state(|| false);

    let behavior_overview_loading = use_state(|| false);
    let behavior_events_loading = use_state(|| false);

    // Sync active section with URL hash (`#overview`, `#tasks`, ...).
    {
        let active_section = active_section.clone();
        use_effect_with((), move |_| {
            active_section.set(read_current_section());
            let listeners = web_sys::window().map(|window| {
                let on_hash = {
                    let active_section = active_section.clone();
                    EventListener::new(&window, "hashchange", move |_| {
                        active_section.set(read_current_section());
                    })
                };
                let on_pop = EventListener::new(&window, "popstate", move |_| {
                    active_section.set(read_current_section());
                });
                (on_hash, on_pop)
            });
            move || drop(listeners)
        });
    }

    let refresh_music_wishes = {
        let music_wishes = music_wishes.clone();
        let load_error = load_error.clone();
        let bucket_loading = bucket_loading.clone();
        let music_wish_page = music_wish_page.clone();
        let music_wish_total = music_wish_total.clone();
        let music_wish_has_more = music_wish_has_more.clone();
        let refresh_music_wishes_seq = refresh_music_wishes_seq.clone();
        Callback::from(move |requested_page: Option<usize>| {
            let music_wishes = music_wishes.clone();
            let load_error = load_error.clone();
            let bucket_loading = bucket_loading.clone();
            let page = requested_page.unwrap_or(*music_wish_page).max(1);
            let music_wish_total = music_wish_total.clone();
            let music_wish_has_more = music_wish_has_more.clone();
            let refresh_music_wishes_seq = refresh_music_wishes_seq.clone();
            let request_id = {
                let mut seq = refresh_music_wishes_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            {
                let mut s = (*bucket_loading).clone();
                s.insert(LoadBucket::MusicWishes);
                bucket_loading.set(s);
            }
            wasm_bindgen_futures::spawn_local(async move {
                let offset = (page - 1) * PAGE_SIZE;
                match fetch_admin_music_wishes(None, Some(PAGE_SIZE), Some(offset)).await {
                    Ok(resp) => {
                        if *refresh_music_wishes_seq.borrow() != request_id {
                            return;
                        }
                        music_wishes.set(resp.wishes);
                        music_wish_total.set(resp.total);
                        music_wish_has_more.set(resp.has_more);
                        load_error.set(None);
                    },
                    Err(err) => {
                        if *refresh_music_wishes_seq.borrow() != request_id {
                            return;
                        }
                        load_error.set(Some(format!("加载点歌心愿失败：{err}")));
                    },
                }
                if *refresh_music_wishes_seq.borrow() != request_id {
                    return;
                }
                let mut s = (*bucket_loading).clone();
                s.remove(&LoadBucket::MusicWishes);
                bucket_loading.set(s);
            });
        })
    };

    let refresh_article_requests = {
        let article_requests = article_requests.clone();
        let load_error = load_error.clone();
        let bucket_loading = bucket_loading.clone();
        let article_request_page = article_request_page.clone();
        let article_request_total = article_request_total.clone();
        let article_request_has_more = article_request_has_more.clone();
        let refresh_article_requests_seq = refresh_article_requests_seq.clone();
        Callback::from(move |requested_page: Option<usize>| {
            let article_requests = article_requests.clone();
            let load_error = load_error.clone();
            let bucket_loading = bucket_loading.clone();
            let page = requested_page.unwrap_or(*article_request_page).max(1);
            let article_request_total = article_request_total.clone();
            let article_request_has_more = article_request_has_more.clone();
            let refresh_article_requests_seq = refresh_article_requests_seq.clone();
            let request_id = {
                let mut seq = refresh_article_requests_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            {
                let mut s = (*bucket_loading).clone();
                s.insert(LoadBucket::ArticleRequests);
                bucket_loading.set(s);
            }
            wasm_bindgen_futures::spawn_local(async move {
                let offset = (page - 1) * PAGE_SIZE;
                match fetch_admin_article_requests(None, Some(PAGE_SIZE), Some(offset)).await {
                    Ok(resp) => {
                        if *refresh_article_requests_seq.borrow() != request_id {
                            return;
                        }
                        article_requests.set(resp.requests);
                        article_request_total.set(resp.total);
                        article_request_has_more.set(resp.has_more);
                        load_error.set(None);
                    },
                    Err(err) => {
                        if *refresh_article_requests_seq.borrow() != request_id {
                            return;
                        }
                        load_error.set(Some(format!("加载文章入库请求失败：{err}")));
                    },
                }
                if *refresh_article_requests_seq.borrow() != request_id {
                    return;
                }
                let mut s = (*bucket_loading).clone();
                s.remove(&LoadBucket::ArticleRequests);
                bucket_loading.set(s);
            });
        })
    };

    let refresh_audit = {
        let load_error = load_error.clone();
        let audit_logs = audit_logs.clone();
        let audit_task_filter = audit_task_filter.clone();
        let audit_action_filter = audit_action_filter.clone();
        let bucket_loading = bucket_loading.clone();
        let audit_page = audit_page.clone();
        let audit_total = audit_total.clone();
        let audit_has_more = audit_has_more.clone();
        let refresh_audit_seq = refresh_audit_seq.clone();
        Callback::from(move |requested_page: Option<usize>| {
            let load_error = load_error.clone();
            let audit_logs = audit_logs.clone();
            let task_filter = (*audit_task_filter).trim().to_string();
            let action_filter = (*audit_action_filter).trim().to_string();
            let bucket_loading = bucket_loading.clone();
            let page = requested_page.unwrap_or(*audit_page).max(1);
            let audit_total = audit_total.clone();
            let audit_has_more = audit_has_more.clone();
            let refresh_audit_seq = refresh_audit_seq.clone();
            let request_id = {
                let mut seq = refresh_audit_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            {
                let mut s = (*bucket_loading).clone();
                s.insert(LoadBucket::Audit);
                bucket_loading.set(s);
            }
            wasm_bindgen_futures::spawn_local(async move {
                let offset = (page - 1) * PAGE_SIZE;
                match fetch_admin_comment_audit_logs(
                    if task_filter.is_empty() { None } else { Some(task_filter.as_str()) },
                    if action_filter.is_empty() { None } else { Some(action_filter.as_str()) },
                    Some(PAGE_SIZE),
                    Some(offset),
                )
                .await
                {
                    Ok(resp) => {
                        if *refresh_audit_seq.borrow() != request_id {
                            return;
                        }
                        audit_logs.set(resp.logs);
                        audit_total.set(resp.total);
                        audit_has_more.set(resp.has_more);
                        load_error.set(None);
                    },
                    Err(err) => {
                        if *refresh_audit_seq.borrow() != request_id {
                            return;
                        }
                        load_error.set(Some(format!("加载审计日志失败：{err}")));
                    },
                }
                if *refresh_audit_seq.borrow() != request_id {
                    return;
                }
                let mut s = (*bucket_loading).clone();
                s.remove(&LoadBucket::Audit);
                bucket_loading.set(s);
            });
        })
    };

    let on_refresh_audit_click = {
        let audit_page = audit_page.clone();
        let refresh_audit = refresh_audit.clone();
        Callback::from(move |_| {
            audit_page.set(1);
            refresh_audit.emit(Some(1));
        })
    };

    let refresh_behavior = {
        let load_error = load_error.clone();
        let behavior_config = behavior_config.clone();
        let behavior_overview = behavior_overview.clone();
        let behavior_events = behavior_events.clone();
        let behavior_days = behavior_days.clone();
        let behavior_date = behavior_date.clone();
        let behavior_has_more = behavior_has_more.clone();
        let behavior_total = behavior_total.clone();
        let behavior_offset = behavior_offset.clone();
        let behavior_path_filter = behavior_path_filter.clone();
        let behavior_page_filter = behavior_page_filter.clone();
        let behavior_device_filter = behavior_device_filter.clone();
        let behavior_status_filter = behavior_status_filter.clone();
        let bucket_loading = bucket_loading.clone();
        let behavior_page = behavior_page.clone();
        let behavior_overview_loading = behavior_overview_loading.clone();
        let behavior_events_loading = behavior_events_loading.clone();
        let refresh_behavior_seq = refresh_behavior_seq.clone();

        Callback::from(move |requested_page: Option<usize>| {
            let load_error = load_error.clone();
            let behavior_config = behavior_config.clone();
            let behavior_overview = behavior_overview.clone();
            let behavior_events = behavior_events.clone();
            let behavior_has_more = behavior_has_more.clone();
            let behavior_total = behavior_total.clone();
            let behavior_offset = behavior_offset.clone();
            let date_val = (*behavior_date).trim().to_string();
            let days = (*behavior_days)
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|value| *value > 0);
            let path_filter = (*behavior_path_filter).trim().to_string();
            let page_filter = (*behavior_page_filter).trim().to_string();
            let device_filter = (*behavior_device_filter).trim().to_string();
            let status_filter = (*behavior_status_filter).trim().parse::<i32>().ok();
            let bucket_loading = bucket_loading.clone();
            let page = requested_page.unwrap_or(*behavior_page).max(1);
            let behavior_overview_loading = behavior_overview_loading.clone();
            let behavior_events_loading = behavior_events_loading.clone();
            let refresh_behavior_seq = refresh_behavior_seq.clone();
            let request_id = {
                let mut seq = refresh_behavior_seq.borrow_mut();
                *seq += 1;
                *seq
            };

            let (query_days, query_date) =
                if date_val.is_empty() { (days, None) } else { (None, Some(date_val)) };

            {
                let mut s = (*bucket_loading).clone();
                s.insert(LoadBucket::Behavior);
                bucket_loading.set(s);
            }

            {
                let behavior_config = behavior_config.clone();
                let behavior_overview = behavior_overview.clone();
                let load_error = load_error.clone();
                let behavior_overview_loading = behavior_overview_loading.clone();
                let refresh_behavior_seq = refresh_behavior_seq.clone();
                behavior_overview_loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let config_result = fetch_admin_api_behavior_config().await;
                    let overview_result =
                        fetch_admin_api_behavior_overview(query_days, Some(20)).await;
                    match (config_result, overview_result) {
                        (Ok(config), Ok(overview)) => {
                            if *refresh_behavior_seq.borrow() != request_id {
                                return;
                            }
                            behavior_config.set(Some(config));
                            behavior_overview.set(Some(overview));
                        },
                        (cfg_err, over_err) => {
                            if *refresh_behavior_seq.borrow() != request_id {
                                return;
                            }
                            load_error.set(Some(format!(
                                "API 行为概览不可用：config={:?}, overview={:?}",
                                cfg_err.err(),
                                over_err.err()
                            )));
                        },
                    }
                    if *refresh_behavior_seq.borrow() != request_id {
                        return;
                    }
                    behavior_overview_loading.set(false);
                });
            }

            {
                let behavior_events = behavior_events.clone();
                let behavior_has_more = behavior_has_more.clone();
                let behavior_total = behavior_total.clone();
                let behavior_offset = behavior_offset.clone();
                let load_error = load_error.clone();
                let bucket_loading = bucket_loading.clone();
                let behavior_events_loading = behavior_events_loading.clone();
                let refresh_behavior_seq = refresh_behavior_seq.clone();
                behavior_events_loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let offset = (page - 1) * PAGE_SIZE;
                    let events_result =
                        fetch_admin_api_behavior_events(&AdminApiBehaviorEventsQuery {
                            days: query_days,
                            limit: Some(PAGE_SIZE),
                            offset: Some(offset),
                            path_contains: if path_filter.is_empty() {
                                None
                            } else {
                                Some(path_filter)
                            },
                            page_contains: if page_filter.is_empty() {
                                None
                            } else {
                                Some(page_filter)
                            },
                            device_type: if device_filter.is_empty() {
                                None
                            } else {
                                Some(device_filter)
                            },
                            method: None,
                            status_code: status_filter,
                            ip: None,
                            date: query_date,
                        })
                        .await;

                    match events_result {
                        Ok(events) => {
                            if *refresh_behavior_seq.borrow() != request_id {
                                return;
                            }
                            behavior_has_more.set(events.has_more);
                            behavior_total.set(events.total);
                            behavior_offset.set(events.offset);
                            behavior_events.set(events.events);
                        },
                        Err(err) => {
                            if *refresh_behavior_seq.borrow() != request_id {
                                return;
                            }
                            load_error.set(Some(format!("API 行为事件加载失败：{err:?}")));
                        },
                    }
                    if *refresh_behavior_seq.borrow() != request_id {
                        return;
                    }
                    behavior_events_loading.set(false);
                    let mut s = (*bucket_loading).clone();
                    s.remove(&LoadBucket::Behavior);
                    bucket_loading.set(s);
                });
            }
        })
    };

    let refresh_memory = {
        let load_error = load_error.clone();
        let memory_overview = memory_overview.clone();
        let memory_functions = memory_functions.clone();
        let memory_modules = memory_modules.clone();
        let memory_stacks = memory_stacks.clone();
        let memory_config = memory_config.clone();
        let memory_top = memory_top.clone();
        let bucket_loading = bucket_loading.clone();
        let refresh_memory_seq = refresh_memory_seq.clone();
        Callback::from(move |_| {
            let load_error = load_error.clone();
            let memory_overview = memory_overview.clone();
            let memory_functions = memory_functions.clone();
            let memory_modules = memory_modules.clone();
            let memory_stacks = memory_stacks.clone();
            let memory_config = memory_config.clone();
            let bucket_loading = bucket_loading.clone();
            let refresh_memory_seq = refresh_memory_seq.clone();
            let top = (*memory_top)
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|value| *value > 0);
            let request_id = {
                let mut seq = refresh_memory_seq.borrow_mut();
                *seq += 1;
                *seq
            };

            {
                let mut s = (*bucket_loading).clone();
                s.insert(LoadBucket::RuntimeMemory);
                bucket_loading.set(s);
            }

            wasm_bindgen_futures::spawn_local(async move {
                let overview_result = fetch_admin_memory_profiler_overview().await;
                let functions_result = fetch_admin_memory_profiler_functions(top).await;
                let modules_result = fetch_admin_memory_profiler_modules(top).await;
                let stacks_result = fetch_admin_memory_profiler_stacks(top).await;

                if *refresh_memory_seq.borrow() != request_id {
                    return;
                }

                match (overview_result, functions_result, modules_result, stacks_result) {
                    (Ok(overview), Ok(functions), Ok(modules), Ok(stacks)) => {
                        memory_config.set(Some(overview.config.clone()));
                        memory_overview.set(Some(overview));
                        memory_functions.set(Some(functions));
                        memory_modules.set(Some(modules));
                        memory_stacks.set(Some(stacks));
                        load_error.set(None);
                    },
                    (overview_err, functions_err, modules_err, stacks_err) => {
                        load_error.set(Some(format!(
                            "内存剖析器不可用：overview={:?}, functions={:?}, modules={:?}, \
                             stacks={:?}",
                            overview_err.err(),
                            functions_err.err(),
                            modules_err.err(),
                            stacks_err.err()
                        )));
                    },
                }

                if *refresh_memory_seq.borrow() != request_id {
                    return;
                }

                let mut s = (*bucket_loading).clone();
                s.remove(&LoadBucket::RuntimeMemory);
                bucket_loading.set(s);
            });
        })
    };

    let refresh_all = {
        let load_error = load_error.clone();
        let view_config = view_config.clone();
        let comment_config = comment_config.clone();
        let music_config = music_config.clone();
        let behavior_config = behavior_config.clone();
        let compaction_config = compaction_config.clone();
        let task_groups = task_groups.clone();
        let grouped_status_counts = grouped_status_counts.clone();
        let published_comments = published_comments.clone();
        let selected_task_id = selected_task_id.clone();
        let selected_task = selected_task.clone();
        let selected_task_ai_output = selected_task_ai_output.clone();
        let selected_published_id = selected_published_id.clone();
        let selected_published = selected_published.clone();
        let loading = loading.clone();
        let status_filter = status_filter.clone();
        let bucket_loading = bucket_loading.clone();
        let tasks_page = tasks_page.clone();
        let published_page = published_page.clone();
        let tasks_total = tasks_total.clone();
        let published_total = published_total.clone();
        let tasks_has_more = tasks_has_more.clone();
        let published_has_more = published_has_more.clone();
        let refresh_all_seq = refresh_all_seq.clone();

        Callback::from(move |requested_pages: (Option<usize>, Option<usize>)| {
            let load_error = load_error.clone();
            let view_config = view_config.clone();
            let comment_config = comment_config.clone();
            let music_config = music_config.clone();
            let behavior_config = behavior_config.clone();
            let compaction_config = compaction_config.clone();
            let task_groups = task_groups.clone();
            let grouped_status_counts = grouped_status_counts.clone();
            let published_comments = published_comments.clone();
            let selected_task_id = selected_task_id.clone();
            let selected_task = selected_task.clone();
            let selected_task_ai_output = selected_task_ai_output.clone();
            let selected_published_id = selected_published_id.clone();
            let selected_published = selected_published.clone();
            let loading = loading.clone();
            let bucket_loading = bucket_loading.clone();
            let tasks_total = tasks_total.clone();
            let published_total = published_total.clone();
            let tasks_has_more = tasks_has_more.clone();
            let published_has_more = published_has_more.clone();
            let refresh_all_seq = refresh_all_seq.clone();
            let request_id = {
                let mut seq = refresh_all_seq.borrow_mut();
                *seq += 1;
                *seq
            };

            let status = (*status_filter).trim().to_string();
            let t_page = requested_pages.0.unwrap_or(*tasks_page).max(1);
            let p_page = requested_pages.1.unwrap_or(*published_page).max(1);
            loading.set(true);
            {
                let mut s = (*bucket_loading).clone();
                s.insert(LoadBucket::Core);
                bucket_loading.set(s);
            }
            wasm_bindgen_futures::spawn_local(async move {
                let t_offset = (t_page - 1) * PAGE_SIZE;
                let p_offset = (p_page - 1) * PAGE_SIZE;
                let view_result = fetch_admin_view_analytics_config().await;
                let comment_result = fetch_admin_comment_runtime_config().await;
                let music_result = fetch_admin_music_runtime_config().await;
                let behavior_result = fetch_admin_api_behavior_config().await;
                let compaction_result = fetch_admin_compaction_runtime_config().await;
                let grouped_result = fetch_admin_comment_tasks_grouped(
                    if status.is_empty() { None } else { Some(status.as_str()) },
                    Some(PAGE_SIZE),
                    Some(t_offset),
                )
                .await;
                let published_result =
                    fetch_admin_published_comments(None, None, Some(PAGE_SIZE), Some(p_offset))
                        .await;

                if *refresh_all_seq.borrow() != request_id {
                    return;
                }

                match (
                    view_result,
                    comment_result,
                    music_result,
                    behavior_result,
                    compaction_result,
                    grouped_result,
                    published_result,
                ) {
                    (
                        Ok(view),
                        Ok(comment),
                        Ok(music),
                        Ok(behavior),
                        Ok(compaction),
                        Ok(grouped),
                        Ok(published),
                    ) => {
                        if *refresh_all_seq.borrow() != request_id {
                            return;
                        }
                        view_config.set(Some(view));
                        comment_config.set(Some(comment));
                        music_config.set(Some(music));
                        behavior_config.set(Some(behavior));
                        compaction_config.set(Some(compaction));
                        grouped_status_counts.set(grouped.status_counts);
                        tasks_total.set(grouped.total_articles);
                        tasks_has_more.set(grouped.has_more);
                        task_groups.set(grouped.groups.clone());
                        published_total.set(published.total);
                        published_has_more.set(published.has_more);
                        published_comments.set(published.comments.clone());

                        if let Some(task_id) = (*selected_task_id).clone() {
                            let mut found = None;
                            for group in grouped.groups {
                                if let Some(task) =
                                    group.tasks.into_iter().find(|task| task.task_id == task_id)
                                {
                                    found = Some(task);
                                    break;
                                }
                            }
                            selected_task.set(found);
                            match fetch_admin_comment_task_ai_output(&task_id, None, Some(1200))
                                .await
                            {
                                Ok(output) => {
                                    if *refresh_all_seq.borrow() != request_id {
                                        return;
                                    }
                                    selected_task_ai_output.set(Some(output));
                                },
                                Err(err) => {
                                    if *refresh_all_seq.borrow() != request_id {
                                        return;
                                    }
                                    selected_task_ai_output.set(None);
                                    load_error.set(Some(format!("加载任务 AI 输出失败：{err}")));
                                },
                            }
                        } else {
                            if *refresh_all_seq.borrow() != request_id {
                                return;
                            }
                            selected_task_ai_output.set(None);
                        }

                        if let Some(comment_id) = (*selected_published_id).clone() {
                            if *refresh_all_seq.borrow() != request_id {
                                return;
                            }
                            let found = published
                                .comments
                                .into_iter()
                                .find(|comment| comment.comment_id == comment_id);
                            selected_published.set(found);
                        }

                        if *refresh_all_seq.borrow() != request_id {
                            return;
                        }
                        load_error.set(None);
                    },
                    (
                        view_err,
                        comment_err,
                        music_err,
                        behavior_err,
                        compaction_err,
                        grouped_err,
                        published_err,
                    ) => {
                        if *refresh_all_seq.borrow() != request_id {
                            return;
                        }
                        load_error.set(Some(format!(
                            "后台核心接口不可用：view={:?}, comment={:?}, music={:?}, \
                             behavior={:?}, compaction={:?}, grouped={:?}, published={:?}",
                            view_err.err(),
                            comment_err.err(),
                            music_err.err(),
                            behavior_err.err(),
                            compaction_err.err(),
                            grouped_err.err(),
                            published_err.err()
                        )));
                    },
                }
                if *refresh_all_seq.borrow() != request_id {
                    return;
                }
                loading.set(false);
                let mut s = (*bucket_loading).clone();
                s.remove(&LoadBucket::Core);
                bucket_loading.set(s);
            });
        })
    };

    // Trigger each bucket at most once on first visit to a section that needs it.
    {
        let loaded_buckets = loaded_buckets.clone();
        let refresh_all = refresh_all.clone();
        let refresh_audit = refresh_audit.clone();
        let refresh_behavior = refresh_behavior.clone();
        let refresh_memory = refresh_memory.clone();
        let refresh_music_wishes = refresh_music_wishes.clone();
        let refresh_article_requests = refresh_article_requests.clone();
        let load_error = load_error.clone();
        use_effect_with(*active_section, move |section| {
            load_error.set(None);
            let needed = buckets_for_section(*section);
            let mut next = (*loaded_buckets).clone();
            let mut changed = false;
            for bucket in needed {
                if next.insert(*bucket) {
                    changed = true;
                    match bucket {
                        LoadBucket::Core => refresh_all.emit((None, None)),
                        LoadBucket::Audit => refresh_audit.emit(None),
                        LoadBucket::Behavior => refresh_behavior.emit(None),
                        LoadBucket::RuntimeMemory => refresh_memory.emit(()),
                        LoadBucket::MusicWishes => refresh_music_wishes.emit(None),
                        LoadBucket::ArticleRequests => refresh_article_requests.emit(None),
                    }
                }
            }
            if changed {
                loaded_buckets.set(next);
            }
            || ()
        });
    }

    let on_reload_section = {
        let active_section = active_section.clone();
        let refresh_all = refresh_all.clone();
        let refresh_audit = refresh_audit.clone();
        let refresh_behavior = refresh_behavior.clone();
        let refresh_memory = refresh_memory.clone();
        let refresh_music_wishes = refresh_music_wishes.clone();
        let refresh_article_requests = refresh_article_requests.clone();
        Callback::from(move |_: MouseEvent| {
            for bucket in buckets_for_section(*active_section) {
                match bucket {
                    LoadBucket::Core => refresh_all.emit((None, None)),
                    LoadBucket::Audit => refresh_audit.emit(None),
                    LoadBucket::Behavior => refresh_behavior.emit(None),
                    LoadBucket::RuntimeMemory => refresh_memory.emit(()),
                    LoadBucket::MusicWishes => refresh_music_wishes.emit(None),
                    LoadBucket::ArticleRequests => refresh_article_requests.emit(None),
                }
            }
        })
    };

    let on_filter_change = {
        let status_filter = status_filter.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                status_filter.set(target.value());
            }
        })
    };

    let on_tasks_apply = {
        let tasks_page = tasks_page.clone();
        let refresh_all = refresh_all.clone();
        Callback::from(move |_| {
            tasks_page.set(1);
            refresh_all.emit((Some(1), None));
        })
    };

    let make_quick_status = |value: &'static str| {
        let status_filter = status_filter.clone();
        let tasks_page = tasks_page.clone();
        let refresh_all = refresh_all.clone();
        Callback::from(move |_: MouseEvent| {
            status_filter.set(value.to_string());
            tasks_page.set(1);
            refresh_all.emit((Some(1), None));
        })
    };

    let on_save_configs = {
        let view_config = view_config.clone();
        let comment_config = comment_config.clone();
        let music_config = music_config.clone();
        let behavior_config = behavior_config.clone();
        let compaction_config = compaction_config.clone();
        let load_error = load_error.clone();
        let saving = saving.clone();
        let refresh_all = refresh_all.clone();
        let refresh_behavior = refresh_behavior.clone();
        let loaded_buckets = loaded_buckets.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            let Some(view_config_value) = (*view_config).clone() else {
                return;
            };
            let Some(comment_config_value) = (*comment_config).clone() else {
                return;
            };
            let Some(music_config_value) = (*music_config).clone() else {
                return;
            };
            let Some(behavior_config_value) = (*behavior_config).clone() else {
                return;
            };
            let Some(compaction_config_value) = (*compaction_config).clone() else {
                return;
            };

            let load_error = load_error.clone();
            let saving = saving.clone();
            let refresh_all = refresh_all.clone();
            let refresh_behavior = refresh_behavior.clone();
            let behavior_loaded = loaded_buckets.contains(&LoadBucket::Behavior);
            let toast = toast.clone();
            saving.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let view_result = update_admin_view_analytics_config(&view_config_value).await;
                let comment_result =
                    update_admin_comment_runtime_config(&comment_config_value).await;
                let music_result = update_admin_music_runtime_config(&music_config_value).await;
                let behavior_result =
                    update_admin_api_behavior_config(&behavior_config_value).await;
                let compaction_result =
                    update_admin_compaction_runtime_config(&compaction_config_value).await;
                match (
                    view_result,
                    comment_result,
                    music_result,
                    behavior_result,
                    compaction_result,
                ) {
                    (Ok(_), Ok(_), Ok(_), Ok(_), Ok(_)) => {
                        load_error.set(None);
                        toast.success("运行时配置已保存");
                        refresh_all.emit((None, None));
                        if behavior_loaded {
                            refresh_behavior.emit(None);
                        }
                    },
                    (view_err, comment_err, music_err, behavior_err, compaction_err) => {
                        load_error.set(Some(format!(
                            "保存配置失败：view={:?}, comment={:?}, music={:?}, behavior={:?}, \
                             compaction={:?}",
                            view_err.err(),
                            comment_err.err(),
                            music_err.err(),
                            behavior_err.err(),
                            compaction_err.err()
                        )));
                    },
                }
                saving.set(false);
            });
        })
    };

    let on_select_task = {
        let selected_task_id = selected_task_id.clone();
        let selected_task = selected_task.clone();
        let selected_task_ai_output = selected_task_ai_output.clone();
        let load_error = load_error.clone();
        Callback::from(move |task_id: String| {
            selected_task_id.set(Some(task_id.clone()));
            let selected_task = selected_task.clone();
            let selected_task_ai_output = selected_task_ai_output.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let task_result = fetch_admin_comment_task(&task_id).await;
                let ai_result =
                    fetch_admin_comment_task_ai_output(&task_id, None, Some(1200)).await;
                match (task_result, ai_result) {
                    (Ok(task), Ok(ai_output)) => {
                        selected_task.set(Some(task));
                        selected_task_ai_output.set(Some(ai_output));
                    },
                    (Err(err), _) => {
                        selected_task.set(None);
                        selected_task_ai_output.set(None);
                        load_error.set(Some(format!("加载任务详情失败：{err}")));
                    },
                    (Ok(task), Err(err)) => {
                        selected_task.set(Some(task));
                        selected_task_ai_output.set(None);
                        load_error.set(Some(format!("加载任务 AI 输出失败：{err}")));
                    },
                }
            });
        })
    };

    let on_select_task_ai_run = {
        let selected_task_id = selected_task_id.clone();
        let selected_task_ai_output = selected_task_ai_output.clone();
        let load_error = load_error.clone();
        Callback::from(move |run_id: String| {
            let Some(task_id) = (*selected_task_id).clone() else {
                return;
            };
            let selected_task_ai_output = selected_task_ai_output.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_comment_task_ai_output(&task_id, Some(&run_id), Some(1200)).await
                {
                    Ok(output) => selected_task_ai_output.set(Some(output)),
                    Err(err) => {
                        load_error.set(Some(format!("加载任务 AI 输出失败：{err}")));
                    },
                }
            });
        })
    };

    let on_selected_task_comment_change = {
        let selected_task = selected_task.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlTextAreaElement>() {
                let mut next = (*selected_task).clone();
                if let Some(task) = next.as_mut() {
                    task.comment_text = target.value();
                }
                selected_task.set(next);
            }
        })
    };

    let on_selected_task_note_change = {
        let selected_task = selected_task.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlTextAreaElement>() {
                let mut next = (*selected_task).clone();
                if let Some(task) = next.as_mut() {
                    task.admin_note = Some(target.value());
                }
                selected_task.set(next);
            }
        })
    };

    let on_save_task = {
        let selected_task = selected_task.clone();
        let load_error = load_error.clone();
        let refresh_all = refresh_all.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            let Some(task) = (*selected_task).clone() else {
                return;
            };
            let request = AdminPatchCommentTaskRequest {
                comment_text: Some(task.comment_text.clone()),
                selected_text: task.selected_text.clone(),
                anchor_block_id: task.anchor_block_id.clone(),
                anchor_context_before: task.anchor_context_before.clone(),
                anchor_context_after: task.anchor_context_after.clone(),
                admin_note: task.admin_note.clone(),
                operator: Some("admin-ui".to_string()),
            };
            let load_error = load_error.clone();
            let refresh_all = refresh_all.clone();
            let selected_task = selected_task.clone();
            let toast = toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match patch_admin_comment_task(&task.task_id, &request).await {
                    Ok(updated) => {
                        toast.success("任务修改已保存");
                        selected_task.set(Some(updated));
                        refresh_all.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(format!("更新评论任务失败：{err}"))),
                }
            });
        })
    };

    let execute_task_action = {
        let load_error = load_error.clone();
        let refresh_all = refresh_all.clone();
        let selected_task = selected_task.clone();
        let selected_task_ai_output = selected_task_ai_output.clone();
        let task_action_inflight = task_action_inflight.clone();
        Callback::from(move |(task_id, action): (String, String)| {
            if task_action_inflight.contains(&task_id) {
                return;
            }
            {
                let mut next = (*task_action_inflight).clone();
                next.insert(task_id.clone());
                task_action_inflight.set(next);
            }
            let load_error = load_error.clone();
            let refresh_all = refresh_all.clone();
            let selected_task = selected_task.clone();
            let selected_task_ai_output = selected_task_ai_output.clone();
            let task_action_inflight = task_action_inflight.clone();
            let request = AdminTaskActionRequest {
                operator: Some("admin-ui".to_string()),
                admin_note: None,
            };
            wasm_bindgen_futures::spawn_local(async move {
                let result = match action.as_str() {
                    "approve" => admin_approve_comment_task(&task_id, &request)
                        .await
                        .map(|_| ()),
                    "approve_run" => admin_approve_and_run_comment_task(&task_id, &request)
                        .await
                        .map(|_| ()),
                    "retry" => admin_retry_comment_task(&task_id, &request)
                        .await
                        .map(|_| ()),
                    "reject" => admin_reject_comment_task(&task_id, &request)
                        .await
                        .map(|_| ()),
                    "delete" => admin_delete_comment_task(&task_id, &request)
                        .await
                        .map(|_| ()),
                    _ => Ok(()),
                };
                match result {
                    Ok(()) => {
                        if selected_task
                            .as_ref()
                            .as_ref()
                            .map(|item| item.task_id.as_str())
                            == Some(task_id.as_str())
                        {
                            selected_task.set(None);
                            selected_task_ai_output.set(None);
                        }
                        refresh_all.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(format!("执行任务操作失败：{err}"))),
                }
                let mut next = (*task_action_inflight).clone();
                next.remove(&task_id);
                task_action_inflight.set(next);
            });
        })
    };

    let run_task_action = {
        let execute_task_action = execute_task_action.clone();
        let pending_delete_task = pending_delete_task.clone();
        Callback::from(move |(task_id, action): (String, String)| {
            if action == "delete" {
                pending_delete_task.set(Some(task_id));
                return;
            }
            execute_task_action.emit((task_id, action));
        })
    };

    let on_select_published = {
        let selected_published_id = selected_published_id.clone();
        let selected_published = selected_published.clone();
        Callback::from(move |comment: ArticleComment| {
            selected_published_id.set(Some(comment.comment_id.clone()));
            selected_published.set(Some(comment));
        })
    };

    let on_selected_published_comment_change = {
        let selected_published = selected_published.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlTextAreaElement>() {
                let mut next = (*selected_published).clone();
                if let Some(comment) = next.as_mut() {
                    comment.comment_text = target.value();
                }
                selected_published.set(next);
            }
        })
    };

    let on_selected_published_ai_change = {
        let selected_published = selected_published.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlTextAreaElement>() {
                let mut next = (*selected_published).clone();
                if let Some(comment) = next.as_mut() {
                    comment.ai_reply_markdown = Some(target.value());
                }
                selected_published.set(next);
            }
        })
    };

    let on_save_published = {
        let selected_published = selected_published.clone();
        let load_error = load_error.clone();
        let refresh_all = refresh_all.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            let Some(comment) = (*selected_published).clone() else {
                return;
            };
            let request = AdminPatchPublishedCommentRequest {
                ai_reply_markdown: comment.ai_reply_markdown.clone(),
                comment_text: Some(comment.comment_text.clone()),
                operator: Some("admin-ui".to_string()),
            };
            let load_error = load_error.clone();
            let refresh_all = refresh_all.clone();
            let selected_published = selected_published.clone();
            let toast = toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match patch_admin_published_comment(&comment.comment_id, &request).await {
                    Ok(updated) => {
                        toast.success("已发布评论已更新");
                        selected_published.set(Some(updated));
                        refresh_all.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(format!("更新已发布评论失败：{err}"))),
                }
            });
        })
    };

    let delete_published_action = {
        let pending_delete_published = pending_delete_published.clone();
        Callback::from(move |comment_id: String| {
            pending_delete_published.set(Some(comment_id));
        })
    };

    let confirm_delete_published = {
        let load_error = load_error.clone();
        let refresh_all = refresh_all.clone();
        let selected_published = selected_published.clone();
        let pending_delete_published = pending_delete_published.clone();
        let toast = toast.clone();
        Callback::from(move |_: ()| {
            let Some(comment_id) = (*pending_delete_published).clone() else {
                return;
            };
            pending_delete_published.set(None);
            let load_error = load_error.clone();
            let refresh_all = refresh_all.clone();
            let selected_published = selected_published.clone();
            let toast = toast.clone();
            let request = AdminTaskActionRequest {
                operator: Some("admin-ui".to_string()),
                admin_note: None,
            };
            wasm_bindgen_futures::spawn_local(async move {
                match delete_admin_published_comment(&comment_id, &request).await {
                    Ok(_) => {
                        toast.success("已删除该评论");
                        if selected_published
                            .as_ref()
                            .as_ref()
                            .map(|item| item.comment_id.as_str())
                            == Some(comment_id.as_str())
                        {
                            selected_published.set(None);
                        }
                        refresh_all.emit((None, None));
                    },
                    Err(err) => {
                        load_error.set(Some(format!("删除已发布评论失败：{err}")));
                    },
                }
            });
        })
    };

    let on_cleanup = {
        let cleanup_days = cleanup_days.clone();
        let load_error = load_error.clone();
        let refresh_all = refresh_all.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            let days = cleanup_days.parse::<i64>().ok();
            let request = AdminCleanupRequest {
                status: Some("failed".to_string()),
                retention_days: days,
            };
            let load_error = load_error.clone();
            let refresh_all = refresh_all.clone();
            let toast = toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match admin_cleanup_comments(&request).await {
                    Ok(_) => {
                        toast.success("失败评论任务清理完成");
                        refresh_all.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(format!("清理失败评论任务出错：{err}"))),
                }
            });
        })
    };

    let on_cleanup_days_change = {
        let cleanup_days = cleanup_days.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                cleanup_days.set(target.value());
            }
        })
    };

    let on_audit_task_filter_change = {
        let audit_task_filter = audit_task_filter.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                audit_task_filter.set(target.value());
            }
        })
    };

    let on_audit_action_filter_change = {
        let audit_action_filter = audit_action_filter.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                audit_action_filter.set(target.value());
            }
        })
    };

    let on_behavior_days_change = {
        let behavior_days = behavior_days.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                behavior_days.set(target.value());
            }
        })
    };

    let on_behavior_date_change = {
        let behavior_date = behavior_date.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                behavior_date.set(target.value());
            }
        })
    };

    let on_behavior_path_filter_change = {
        let behavior_path_filter = behavior_path_filter.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                behavior_path_filter.set(target.value());
            }
        })
    };

    let on_behavior_page_filter_change = {
        let behavior_page_filter = behavior_page_filter.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                behavior_page_filter.set(target.value());
            }
        })
    };

    let on_behavior_device_filter_change = {
        let behavior_device_filter = behavior_device_filter.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                behavior_device_filter.set(target.value());
            }
        })
    };

    let on_behavior_status_filter_change = {
        let behavior_status_filter = behavior_status_filter.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                behavior_status_filter.set(target.value());
            }
        })
    };

    let on_behavior_apply = {
        let behavior_page = behavior_page.clone();
        let behavior_events_loading = behavior_events_loading.clone();
        let refresh_behavior = refresh_behavior.clone();
        Callback::from(move |_| {
            behavior_page.set(1);
            behavior_events_loading.set(true);
            refresh_behavior.emit(Some(1));
        })
    };

    let on_behavior_refresh_events = {
        let behavior_days = behavior_days.clone();
        let behavior_date = behavior_date.clone();
        let behavior_path_filter = behavior_path_filter.clone();
        let behavior_page_filter = behavior_page_filter.clone();
        let behavior_device_filter = behavior_device_filter.clone();
        let behavior_status_filter = behavior_status_filter.clone();
        let behavior_page = behavior_page.clone();
        let behavior_events = behavior_events.clone();
        let behavior_has_more = behavior_has_more.clone();
        let behavior_total = behavior_total.clone();
        let behavior_offset = behavior_offset.clone();
        let behavior_events_loading = behavior_events_loading.clone();
        let load_error = load_error.clone();
        let bucket_loading = bucket_loading.clone();
        let refresh_behavior_seq = refresh_behavior_seq.clone();
        Callback::from(move |_| {
            let date_val = (*behavior_date).trim().to_string();
            let days = (*behavior_days)
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|value| *value > 0);
            let path_filter = (*behavior_path_filter).trim().to_string();
            let page_filter = (*behavior_page_filter).trim().to_string();
            let device_filter = (*behavior_device_filter).trim().to_string();
            let status_filter = (*behavior_status_filter).trim().parse::<i32>().ok();
            let page = (*behavior_page).max(1);
            let (query_days, query_date) =
                if date_val.is_empty() { (days, None) } else { (None, Some(date_val)) };

            let request_id = {
                let mut seq = refresh_behavior_seq.borrow_mut();
                *seq += 1;
                *seq
            };

            behavior_events_loading.set(true);
            {
                let mut s = (*bucket_loading).clone();
                s.insert(LoadBucket::Behavior);
                bucket_loading.set(s);
            }

            let behavior_events = behavior_events.clone();
            let behavior_has_more = behavior_has_more.clone();
            let behavior_total = behavior_total.clone();
            let behavior_offset = behavior_offset.clone();
            let behavior_events_loading = behavior_events_loading.clone();
            let load_error = load_error.clone();
            let bucket_loading = bucket_loading.clone();
            let refresh_behavior_seq = refresh_behavior_seq.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let offset = (page - 1) * PAGE_SIZE;
                let events_result = fetch_admin_api_behavior_events(&AdminApiBehaviorEventsQuery {
                    days: query_days,
                    limit: Some(PAGE_SIZE),
                    offset: Some(offset),
                    path_contains: if path_filter.is_empty() { None } else { Some(path_filter) },
                    page_contains: if page_filter.is_empty() { None } else { Some(page_filter) },
                    device_type: if device_filter.is_empty() { None } else { Some(device_filter) },
                    method: None,
                    status_code: status_filter,
                    ip: None,
                    date: query_date,
                })
                .await;

                match events_result {
                    Ok(events) => {
                        if *refresh_behavior_seq.borrow() != request_id {
                            return;
                        }
                        behavior_has_more.set(events.has_more);
                        behavior_total.set(events.total);
                        behavior_offset.set(events.offset);
                        behavior_events.set(events.events);
                        load_error.set(None);
                    },
                    Err(err) => {
                        if *refresh_behavior_seq.borrow() != request_id {
                            return;
                        }
                        load_error.set(Some(format!("API 行为事件加载失败：{err:?}")));
                    },
                }

                if *refresh_behavior_seq.borrow() != request_id {
                    return;
                }
                behavior_events_loading.set(false);
                let mut s = (*bucket_loading).clone();
                s.remove(&LoadBucket::Behavior);
                bucket_loading.set(s);
            });
        })
    };

    let on_behavior_cleanup = {
        let behavior_config = behavior_config.clone();
        let behavior_page = behavior_page.clone();
        let behavior_events_loading = behavior_events_loading.clone();
        let refresh_behavior = refresh_behavior.clone();
        let loaded_buckets = loaded_buckets.clone();
        let load_error = load_error.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            let Some(config) = (*behavior_config).clone() else {
                return;
            };
            let request = AdminApiBehaviorCleanupRequest {
                retention_days: Some(config.retention_days),
            };
            let refresh_behavior = refresh_behavior.clone();
            let load_error = load_error.clone();
            let behavior_page = behavior_page.clone();
            let behavior_events_loading = behavior_events_loading.clone();
            let behavior_loaded = loaded_buckets.contains(&LoadBucket::Behavior);
            let toast = toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match admin_cleanup_api_behavior(&request).await {
                    Ok(_) => {
                        toast.success("过期 API 行为日志已清理");
                        if behavior_loaded {
                            behavior_page.set(1);
                            behavior_events_loading.set(true);
                            refresh_behavior.emit(Some(1));
                        }
                    },
                    Err(err) => {
                        load_error.set(Some(format!("清理 API 行为日志失败：{err}")));
                    },
                }
            });
        })
    };

    let on_memory_top_change = {
        let memory_top = memory_top.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                memory_top.set(target.value());
            }
        })
    };

    let on_memory_refresh = {
        let refresh_memory = refresh_memory.clone();
        Callback::from(move |_| refresh_memory.emit(()))
    };

    let on_memory_enabled_change = {
        let memory_config = memory_config.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                let mut next = (*memory_config).clone();
                if let Some(cfg) = next.as_mut() {
                    cfg.enabled = target.checked();
                }
                memory_config.set(next);
            }
        })
    };

    let on_memory_sample_rate_change = {
        let memory_config = memory_config.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = target.value().parse::<u64>() {
                    let mut next = (*memory_config).clone();
                    if let Some(cfg) = next.as_mut() {
                        cfg.sample_rate = value;
                    }
                    memory_config.set(next);
                }
            }
        })
    };

    let on_memory_min_alloc_change = {
        let memory_config = memory_config.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = target.value().parse::<usize>() {
                    let mut next = (*memory_config).clone();
                    if let Some(cfg) = next.as_mut() {
                        cfg.min_alloc_bytes = value;
                    }
                    memory_config.set(next);
                }
            }
        })
    };

    let on_memory_max_tracked_change = {
        let memory_config = memory_config.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = target.value().parse::<usize>() {
                    let mut next = (*memory_config).clone();
                    if let Some(cfg) = next.as_mut() {
                        cfg.max_tracked_allocations = value;
                    }
                    memory_config.set(next);
                }
            }
        })
    };

    let on_memory_reset = {
        let memory_action_loading = memory_action_loading.clone();
        let load_error = load_error.clone();
        let refresh_memory = refresh_memory.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            if *memory_action_loading {
                return;
            }
            memory_action_loading.set(true);
            let memory_action_loading = memory_action_loading.clone();
            let load_error = load_error.clone();
            let refresh_memory = refresh_memory.clone();
            let toast = toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match admin_reset_memory_profiler().await {
                    Ok(()) => {
                        load_error.set(None);
                        toast.success("内存采样计数已重置");
                        refresh_memory.emit(());
                    },
                    Err(err) => {
                        load_error.set(Some(format!("重置内存剖析器失败：{err}")));
                    },
                }
                memory_action_loading.set(false);
            });
        })
    };

    let on_memory_save_config = {
        let memory_config = memory_config.clone();
        let memory_action_loading = memory_action_loading.clone();
        let load_error = load_error.clone();
        let refresh_memory = refresh_memory.clone();
        let toast = toast.clone();
        Callback::from(move |_| {
            if *memory_action_loading {
                return;
            }
            let Some(config) = (*memory_config).clone() else {
                return;
            };

            memory_action_loading.set(true);
            let memory_action_loading = memory_action_loading.clone();
            let memory_config = memory_config.clone();
            let load_error = load_error.clone();
            let refresh_memory = refresh_memory.clone();
            let toast = toast.clone();
            let request = MemoryProfilerConfigUpdate {
                enabled: Some(config.enabled),
                sample_rate: Some(config.sample_rate),
                min_alloc_bytes: Some(config.min_alloc_bytes),
                max_tracked_allocations: Some(config.max_tracked_allocations),
            };
            wasm_bindgen_futures::spawn_local(async move {
                match admin_update_memory_profiler_config(&request).await {
                    Ok(updated) => {
                        memory_config.set(Some(updated));
                        load_error.set(None);
                        toast.success("采样器配置已保存");
                        refresh_memory.emit(());
                    },
                    Err(err) => {
                        load_error.set(Some(format!("更新内存采样配置失败：{err}")));
                    },
                }
                memory_action_loading.set(false);
            });
        })
    };

    let grouped_total_tasks: usize = task_groups.iter().map(|group| group.total).sum();

    let on_tasks_page_change = {
        let page = tasks_page.clone();
        let refresh = refresh_all.clone();
        Callback::from(move |p: usize| {
            page.set(p);
            refresh.emit((Some(p), None));
        })
    };
    let on_published_page_change = {
        let page = published_page.clone();
        let refresh = refresh_all.clone();
        Callback::from(move |p: usize| {
            page.set(p);
            refresh.emit((None, Some(p)));
        })
    };
    let on_audit_page_change = {
        let page = audit_page.clone();
        let refresh = refresh_audit.clone();
        Callback::from(move |p: usize| {
            page.set(p);
            refresh.emit(Some(p));
        })
    };
    let on_behavior_page_change = {
        let page = behavior_page.clone();
        let refresh = refresh_behavior.clone();
        let behavior_events_loading = behavior_events_loading.clone();
        Callback::from(move |p: usize| {
            page.set(p);
            behavior_events_loading.set(true);
            refresh.emit(Some(p));
        })
    };
    let on_music_wish_page_change = {
        let page = music_wish_page.clone();
        let refresh = refresh_music_wishes.clone();
        Callback::from(move |p: usize| {
            page.set(p);
            refresh.emit(Some(p));
        })
    };
    let on_article_request_page_change = {
        let page = article_request_page.clone();
        let refresh = refresh_article_requests.clone();
        Callback::from(move |p: usize| {
            page.set(p);
            refresh.emit(Some(p));
        })
    };

    let tasks_total_pages = (*tasks_total).max(1).div_ceil(PAGE_SIZE);
    let published_total_pages = (*published_total).max(1).div_ceil(PAGE_SIZE);
    let audit_total_pages = (*audit_total).max(1).div_ceil(PAGE_SIZE);
    let behavior_total_pages = (*behavior_total).max(1).div_ceil(PAGE_SIZE);
    let music_wish_total_pages = (*music_wish_total).max(1).div_ceil(PAGE_SIZE);
    let article_request_total_pages = (*article_request_total).max(1).div_ceil(PAGE_SIZE);

    let music_wish_query_lower = (*music_wish_search).trim().to_lowercase();
    let filtered_music_wishes: Vec<MusicWishItem> = {
        let q = music_wish_query_lower.clone();
        use_memo(((*music_wishes).clone(), q), move |(items, q)| {
            if q.is_empty() {
                items.clone()
            } else {
                items
                    .iter()
                    .filter(|w| {
                        let hay = [
                            w.song_name.to_lowercase(),
                            w.artist_hint.clone().unwrap_or_default().to_lowercase(),
                            w.nickname.to_lowercase(),
                            w.wish_message.to_lowercase(),
                            w.status.to_lowercase(),
                        ];
                        hay.iter().any(|v| v.contains(q))
                    })
                    .cloned()
                    .collect()
            }
        })
        .as_ref()
        .clone()
    };
    let article_request_query_lower = (*article_request_search).trim().to_lowercase();
    let filtered_article_requests: Vec<ArticleRequestItem> = {
        let q = article_request_query_lower.clone();
        use_memo(((*article_requests).clone(), q), move |(items, q)| {
            if q.is_empty() {
                items.clone()
            } else {
                items
                    .iter()
                    .filter(|r| {
                        let hay = [
                            r.article_url.to_lowercase(),
                            r.title_hint.clone().unwrap_or_default().to_lowercase(),
                            r.nickname.to_lowercase(),
                            r.request_message.to_lowercase(),
                            r.status.to_lowercase(),
                        ];
                        hay.iter().any(|v| v.contains(q))
                    })
                    .cloned()
                    .collect()
            }
        })
        .as_ref()
        .clone()
    };
    let music_wish_matched = filtered_music_wishes.len();
    let article_request_matched = filtered_article_requests.len();
    let on_music_wish_search_change = {
        let music_wish_search = music_wish_search.clone();
        Callback::from(move |v: String| music_wish_search.set(v))
    };
    let on_article_request_search_change = {
        let article_request_search = article_request_search.clone();
        Callback::from(move |v: String| article_request_search.set(v))
    };

    let execute_music_wish_action = {
        let music_wishes = music_wishes.clone();
        let music_wish_action_inflight = music_wish_action_inflight.clone();
        let load_error = load_error.clone();
        Callback::from(move |(wid, action): (String, WishAction)| {
            let music_wishes = music_wishes.clone();
            let inflight = music_wish_action_inflight.clone();
            let load_error = load_error.clone();
            let mut s = (*inflight).clone();
            s.insert(wid.clone());
            inflight.set(s);
            wasm_bindgen_futures::spawn_local(async move {
                match action {
                    WishAction::Approve => match admin_approve_and_run_music_wish(&wid, None).await
                    {
                        Ok(updated) => {
                            let mut list = (*music_wishes).clone();
                            if let Some(item) =
                                list.iter_mut().find(|w| w.wish_id == updated.wish_id)
                            {
                                *item = updated;
                            }
                            music_wishes.set(list);
                            load_error.set(None);
                        },
                        Err(err) => load_error.set(Some(format!("批准点歌失败：{err}"))),
                    },
                    WishAction::Reject => match admin_reject_music_wish(&wid, None).await {
                        Ok(updated) => {
                            let mut list = (*music_wishes).clone();
                            if let Some(item) =
                                list.iter_mut().find(|w| w.wish_id == updated.wish_id)
                            {
                                *item = updated;
                            }
                            music_wishes.set(list);
                            load_error.set(None);
                        },
                        Err(err) => load_error.set(Some(format!("驳回点歌失败：{err}"))),
                    },
                    WishAction::Retry => match admin_retry_music_wish(&wid).await {
                        Ok(updated) => {
                            let mut list = (*music_wishes).clone();
                            if let Some(item) =
                                list.iter_mut().find(|w| w.wish_id == updated.wish_id)
                            {
                                *item = updated;
                            }
                            music_wishes.set(list);
                            load_error.set(None);
                        },
                        Err(err) => load_error.set(Some(format!("重试点歌失败：{err}"))),
                    },
                    WishAction::Delete => match admin_delete_music_wish(&wid).await {
                        Ok(()) => {
                            let list: Vec<_> = (*music_wishes)
                                .iter()
                                .filter(|w| w.wish_id != wid)
                                .cloned()
                                .collect();
                            music_wishes.set(list);
                            load_error.set(None);
                        },
                        Err(err) => load_error.set(Some(format!("删除点歌失败：{err}"))),
                    },
                }
                let mut s = (*inflight).clone();
                s.remove(&wid);
                inflight.set(s);
            });
        })
    };

    let on_music_wish_action = {
        let execute_music_wish_action = execute_music_wish_action.clone();
        let pending_delete_wish = pending_delete_wish.clone();
        Callback::from(move |(wid, action): (String, WishAction)| {
            if action == WishAction::Delete {
                pending_delete_wish.set(Some(wid));
                return;
            }
            execute_music_wish_action.emit((wid, action));
        })
    };

    let execute_article_request_action = {
        let article_requests = article_requests.clone();
        let article_request_action_inflight = article_request_action_inflight.clone();
        let load_error = load_error.clone();
        Callback::from(move |(rid, action): (String, ArticleRequestAction)| {
            let article_requests = article_requests.clone();
            let inflight = article_request_action_inflight.clone();
            let load_error = load_error.clone();
            let mut s = (*inflight).clone();
            s.insert(rid.clone());
            inflight.set(s);
            wasm_bindgen_futures::spawn_local(async move {
                match action {
                    ArticleRequestAction::Approve => {
                        match admin_approve_and_run_article_request(&rid, None).await {
                            Ok(updated) => {
                                let mut list = (*article_requests).clone();
                                if let Some(item) =
                                    list.iter_mut().find(|r| r.request_id == updated.request_id)
                                {
                                    *item = updated;
                                }
                                article_requests.set(list);
                                load_error.set(None);
                            },
                            Err(err) => load_error.set(Some(format!("批准入库失败：{err}"))),
                        }
                    },
                    ArticleRequestAction::Reject => {
                        match admin_reject_article_request(&rid, None).await {
                            Ok(updated) => {
                                let mut list = (*article_requests).clone();
                                if let Some(item) =
                                    list.iter_mut().find(|r| r.request_id == updated.request_id)
                                {
                                    *item = updated;
                                }
                                article_requests.set(list);
                                load_error.set(None);
                            },
                            Err(err) => load_error.set(Some(format!("驳回入库失败：{err}"))),
                        }
                    },
                    ArticleRequestAction::Retry => match admin_retry_article_request(&rid).await {
                        Ok(updated) => {
                            let mut list = (*article_requests).clone();
                            if let Some(item) =
                                list.iter_mut().find(|r| r.request_id == updated.request_id)
                            {
                                *item = updated;
                            }
                            article_requests.set(list);
                            load_error.set(None);
                        },
                        Err(err) => load_error.set(Some(format!("重试入库失败：{err}"))),
                    },
                    ArticleRequestAction::Delete => {
                        match admin_delete_article_request(&rid).await {
                            Ok(()) => {
                                let list: Vec<_> = (*article_requests)
                                    .iter()
                                    .filter(|r| r.request_id != rid)
                                    .cloned()
                                    .collect();
                                article_requests.set(list);
                                load_error.set(None);
                            },
                            Err(err) => load_error.set(Some(format!("删除入库请求失败：{err}"))),
                        }
                    },
                }
                let mut s = (*inflight).clone();
                s.remove(&rid);
                inflight.set(s);
            });
        })
    };

    let on_article_request_action = {
        let execute_article_request_action = execute_article_request_action.clone();
        let pending_delete_request = pending_delete_request.clone();
        Callback::from(move |(rid, action): (String, ArticleRequestAction)| {
            if action == ArticleRequestAction::Delete {
                pending_delete_request.set(Some(rid));
                return;
            }
            execute_article_request_action.emit((rid, action));
        })
    };

    // Derived counters for Overview KPIs & sidebar badges (from already-fetched
    // data).
    let pending_tasks_count = grouped_status_counts.get("pending").copied().unwrap_or(0);
    let failed_tasks_count = grouped_status_counts.get("failed").copied().unwrap_or(0);
    let running_tasks_count = grouped_status_counts.get("running").copied().unwrap_or(0);
    let all_status_tasks_total: usize = grouped_status_counts.values().copied().sum();

    let pending_wishes_count = music_wishes
        .iter()
        .filter(|w| w.status == "pending")
        .count();
    let failed_wishes_count = music_wishes.iter().filter(|w| w.status == "failed").count();

    let pending_requests_count = article_requests
        .iter()
        .filter(|r| r.status == "pending")
        .count();
    let failed_requests_count = article_requests
        .iter()
        .filter(|r| r.status == "failed")
        .count();
    let orphan_done_requests_count = article_requests
        .iter()
        .filter(|r| r.status == "done" && !article_request_has_article(r))
        .count();

    let sidebar_badges = vec![
        (AdminNav::CommentTasks, pending_tasks_count + failed_tasks_count),
        (AdminNav::MusicWishes, pending_wishes_count + failed_wishes_count),
        (
            AdminNav::ArticleRequests,
            pending_requests_count + failed_requests_count + orphan_done_requests_count,
        ),
    ];

    let core_busy = bucket_loading.contains(&LoadBucket::Core) || *loading;
    let audit_busy = bucket_loading.contains(&LoadBucket::Audit);
    let wishes_busy = bucket_loading.contains(&LoadBucket::MusicWishes);
    let requests_busy = bucket_loading.contains(&LoadBucket::ArticleRequests);
    let behavior_busy = bucket_loading.contains(&LoadBucket::Behavior)
        || *behavior_overview_loading
        || *behavior_events_loading;
    let memory_busy = bucket_loading.contains(&LoadBucket::RuntimeMemory);

    let section_busy = match *active_section {
        AdminNav::Overview => core_busy || wishes_busy || requests_busy,
        AdminNav::CommentTasks | AdminNav::PublishedComments | AdminNav::SiteConfig => core_busy,
        AdminNav::AuditLogs => audit_busy,
        AdminNav::MusicWishes => wishes_busy,
        AdminNav::ArticleRequests => requests_busy,
        AdminNav::ApiBehavior => behavior_busy,
        AdminNav::RuntimeMemory => memory_busy,
        AdminNav::LocalMedia => false,
    };

    let go_to = |nav: AdminNav| {
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            navigate_to_admin_section(nav, true);
        })
    };

    let error_banner = (*load_error).clone().map(|err| {
        let retry = on_reload_section.clone();
        let dismiss = {
            let load_error = load_error.clone();
            Callback::from(move |_: MouseEvent| load_error.set(None))
        };
        html! {
            <div class="adm-banner adm-banner--err" role="alert" style="margin-bottom: 20px;">
                <i class="fas fa-circle-exclamation" aria-hidden="true"></i>
                <div class="adm-banner__body">{ err }</div>
                <div class="adm-actions">
                    <button type="button" class="adm-btn adm-btn--ghost adm-btn--sm" onclick={retry}>
                        { "重试" }
                    </button>
                    <button type="button" class="adm-btn adm-btn--ghost adm-btn--icon adm-btn--sm"
                        onclick={dismiss} aria-label="关闭错误提示">
                        <i class="fas fa-xmark" aria-hidden="true"></i>
                    </button>
                </div>
            </div>
        }
    });

    let refresh_btn = |busy: bool| {
        let onclick = on_reload_section.clone();
        html! {
            <button type="button" class="adm-btn adm-btn--secondary" {onclick} disabled={busy}>
                <i class={classes!("fas", "fa-rotate-right", busy.then_some("fa-spin"))} aria-hidden="true"></i>
                <span>{ if busy { "刷新中…" } else { "刷新" } }</span>
            </button>
        }
    };

    let section_view = match *active_section {
        AdminNav::Overview => {
            let initial_empty = core_busy
                && view_config.is_none()
                && task_groups.is_empty()
                && music_wishes.is_empty()
                && article_requests.is_empty();
            let llm_console = llm_access_frontend_url("/console");
            #[cfg(feature = "local-media")]
            let local_media_overview_link = html! {
                <Link<Route> to={Route::AdminLocalMedia}
                    classes={classes!("adm-btn", "adm-btn--secondary", "adm-btn--sm")}>
                    <i class="fas fa-film" aria-hidden="true"></i>
                    <span>{ "本地媒体" }</span>
                </Link<Route>>
            };
            #[cfg(not(feature = "local-media"))]
            let local_media_overview_link = Html::default();

            html! {
                <div class="adm-stack adm-enter">
                    <AdminPageHeader
                        eyebrow={Some(AttrValue::Static("工作台 · LOOPBACK ONLY"))}
                        title={AttrValue::Static("管理概览")}
                        description={Some(AttrValue::Static(
                            "汇总本地站点的评论审核、读者共建队列与后台存储运行状态。"
                        ))}
                    >
                        { refresh_btn(section_busy) }
                    </AdminPageHeader>
                    { error_banner.clone().unwrap_or_default() }

                    if initial_empty {
                        { kpi_skeleton(5) }
                    } else {
                        <section class="adm-kpis" aria-label="核心指标">
                            <a class="adm-kpi adm-kpi--link" href={admin_nav_href(AdminNav::CommentTasks)}
                                onclick={go_to(AdminNav::CommentTasks)}>
                                <div class="adm-kpi__label">{ "待审评论任务" }</div>
                                <div class={classes!(
                                    "adm-kpi__value",
                                    (pending_tasks_count > 0).then_some("adm-kpi__value--accent"),
                                )}>
                                    { pending_tasks_count }
                                </div>
                                <div class="adm-kpi__meta">
                                    { format!("共 {} 条 · 运行 {} · 失败 {}", all_status_tasks_total, running_tasks_count, failed_tasks_count) }
                                </div>
                            </a>

                            <a class="adm-kpi adm-kpi--link" href={admin_nav_href(AdminNav::MusicWishes)}
                                onclick={go_to(AdminNav::MusicWishes)}>
                                <div class="adm-kpi__label">{ "点歌心愿队列" }</div>
                                <div class={classes!(
                                    "adm-kpi__value",
                                    (pending_wishes_count > 0).then_some("adm-kpi__value--accent"),
                                )}>
                                    { *music_wish_total }
                                </div>
                                <div class="adm-kpi__meta">
                                    { format!("本页待审 {} · 失败 {}", pending_wishes_count, failed_wishes_count) }
                                </div>
                            </a>

                            <a class="adm-kpi adm-kpi--link" href={admin_nav_href(AdminNav::ArticleRequests)}
                                onclick={go_to(AdminNav::ArticleRequests)}>
                                <div class="adm-kpi__label">{ "文章入库请求" }</div>
                                <div class={classes!(
                                    "adm-kpi__value",
                                    (pending_requests_count > 0).then_some("adm-kpi__value--accent"),
                                )}>
                                    { *article_request_total }
                                </div>
                                <div class="adm-kpi__meta">
                                    { format!("本页待审 {} · 失败 {}", pending_requests_count, failed_requests_count) }
                                </div>
                            </a>

                            <a class="adm-kpi adm-kpi--link" href={admin_nav_href(AdminNav::PublishedComments)}
                                onclick={go_to(AdminNav::PublishedComments)}>
                                <div class="adm-kpi__label">{ "已发布评论" }</div>
                                <div class="adm-kpi__value">{ *published_total }</div>
                                <div class="adm-kpi__meta">
                                    { (*comment_config)
                                        .as_ref()
                                        .map(|c| format!("提交限流 {}s · 保留 {}d", c.submit_rate_limit_seconds, c.cleanup_retention_days))
                                        .unwrap_or_else(|| "审核通过并公开展示".to_string()) }
                                </div>
                            </a>

                            <a class="adm-kpi adm-kpi--link" href={admin_nav_href(AdminNav::SiteConfig)}
                                onclick={go_to(AdminNav::SiteConfig)}>
                                <div class="adm-kpi__label">{ "Lance 自动整理" }</div>
                                <div class="adm-kpi__value">
                                    { match (*compaction_config).as_ref() {
                                        Some(c) if c.enabled => "运行中",
                                        Some(_) => "已暂停",
                                        None => "—",
                                    } }
                                </div>
                                <div class="adm-kpi__meta">
                                    { (*compaction_config)
                                        .as_ref()
                                        .map(|c| format!("扫描 {}s · 阈值 {} 片", c.scan_interval_seconds, c.fragment_threshold))
                                        .unwrap_or_else(|| "点击查看全局配置".to_string()) }
                                </div>
                            </a>
                        </section>
                    }

                    <div class="adm-grid-2">
                        <section class="adm-card">
                            <div class="adm-card__header">
                                <div>
                                    <h2 class="adm-card__title">{ "待办与队列入口" }</h2>
                                    <p class="adm-card__sub">{ "直接进入需要人工确认或失败重试的工作流。" }</p>
                                </div>
                            </div>
                            <div class="adm-card__body adm-card__body--flush">
                                <a class="adm-todo" href={admin_nav_href(AdminNav::CommentTasks)}
                                    onclick={go_to(AdminNav::CommentTasks)}>
                                    <span class="adm-todo__icon">
                                        <i class="fas fa-inbox" aria-hidden="true"></i>
                                    </span>
                                    <div class="adm-todo__main">
                                        <div class="adm-todo__title">{ "评论任务审核" }</div>
                                        <div class="adm-todo__meta">
                                            { format!("涉及 {} 篇文章 · 待审 {} · 失败 {}", *tasks_total, pending_tasks_count, failed_tasks_count) }
                                        </div>
                                    </div>
                                    <span class={classes!(
                                        "adm-todo__count",
                                        (pending_tasks_count + failed_tasks_count > 0).then_some("is-hot"),
                                    )}>
                                        { pending_tasks_count + failed_tasks_count }
                                    </span>
                                </a>

                                <a class="adm-todo" href={admin_nav_href(AdminNav::MusicWishes)}
                                    onclick={go_to(AdminNav::MusicWishes)}>
                                    <span class="adm-todo__icon">
                                        <i class="fas fa-music" aria-hidden="true"></i>
                                    </span>
                                    <div class="adm-todo__main">
                                        <div class="adm-todo__title">{ "点歌心愿队列" }</div>
                                        <div class="adm-todo__meta">
                                            { format!("累计 {} 条 · 本页待审 {} · 失败 {}", *music_wish_total, pending_wishes_count, failed_wishes_count) }
                                        </div>
                                    </div>
                                    <span class={classes!(
                                        "adm-todo__count",
                                        (pending_wishes_count + failed_wishes_count > 0).then_some("is-hot"),
                                    )}>
                                        { pending_wishes_count + failed_wishes_count }
                                    </span>
                                </a>

                                <a class="adm-todo" href={admin_nav_href(AdminNav::ArticleRequests)}
                                    onclick={go_to(AdminNav::ArticleRequests)}>
                                    <span class="adm-todo__icon">
                                        <i class="fas fa-file-import" aria-hidden="true"></i>
                                    </span>
                                    <div class="adm-todo__main">
                                        <div class="adm-todo__title">{ "文章入库队列" }</div>
                                        <div class="adm-todo__meta">
                                            { format!(
                                                "累计 {} 条 · 本页待审 {} · 失败 {} · 待补文章 {}",
                                                *article_request_total,
                                                pending_requests_count,
                                                failed_requests_count,
                                                orphan_done_requests_count
                                            ) }
                                        </div>
                                    </div>
                                    <span class={classes!(
                                        "adm-todo__count",
                                        (pending_requests_count + failed_requests_count + orphan_done_requests_count > 0).then_some("is-hot"),
                                    )}>
                                        { pending_requests_count + failed_requests_count + orphan_done_requests_count }
                                    </span>
                                </a>

                                <a class="adm-todo" href={admin_nav_href(AdminNav::AuditLogs)}
                                    onclick={go_to(AdminNav::AuditLogs)}>
                                    <span class="adm-todo__icon">
                                        <i class="fas fa-clock-rotate-left" aria-hidden="true"></i>
                                    </span>
                                    <div class="adm-todo__main">
                                        <div class="adm-todo__title">{ "评论审计日志" }</div>
                                        <div class="adm-todo__meta">{ "追溯每一次审批、修改、重试与清理记录" }</div>
                                    </div>
                                    <span class="adm-faint" aria-hidden="true">{ "→" }</span>
                                </a>
                            </div>
                        </section>

                        <section class="adm-card">
                            <div class="adm-card__header">
                                <div>
                                    <h2 class="adm-card__title">{ "运行时摘要与运维工具" }</h2>
                                    <p class="adm-card__sub">{ "当前生效的站点参数快照与独立子系统入口。" }</p>
                                </div>
                                <a class="adm-btn adm-btn--ghost adm-btn--sm"
                                    href={admin_nav_href(AdminNav::SiteConfig)}
                                    onclick={go_to(AdminNav::SiteConfig)}>
                                    { "编辑配置 →" }
                                </a>
                            </div>
                            <div class="adm-card__body adm-stack-sm">
                                <dl class="adm-kv">
                                    <div>
                                        <dt>{ "浏览去重窗口" }</dt>
                                        <dd class="adm-mono">
                                            { (*view_config)
                                                .as_ref()
                                                .map(|c| format!("{}s（默认 {}d / 上限 {}d）", c.dedupe_window_seconds, c.trend_default_days, c.trend_max_days))
                                                .unwrap_or_else(|| "—".to_string()) }
                                        </dd>
                                    </div>
                                    <div>
                                        <dt>{ "音乐播放去重" }</dt>
                                        <dd class="adm-mono">
                                            { (*music_config)
                                                .as_ref()
                                                .map(|c| format!("{}s · 评论限流 {}s", c.play_dedupe_window_seconds, c.comment_rate_limit_seconds))
                                                .unwrap_or_else(|| "—".to_string()) }
                                        </dd>
                                    </div>
                                    <div>
                                        <dt>{ "API 行为保留" }</dt>
                                        <dd class="adm-mono">
                                            { (*behavior_config)
                                                .as_ref()
                                                .map(|c| format!("{}d · 批次 {} · {}s", c.retention_days, c.flush_batch_size, c.flush_interval_seconds))
                                                .unwrap_or_else(|| "—".to_string()) }
                                        </dd>
                                    </div>
                                    <div>
                                        <dt>{ "Compaction 工作线程" }</dt>
                                        <dd class="adm-mono">
                                            { (*compaction_config)
                                                .as_ref()
                                                .map(|c| format!("{} workers · 清理 >{}h", c.worker_count, c.prune_older_than_hours))
                                                .unwrap_or_else(|| "—".to_string()) }
                                        </dd>
                                    </div>
                                </dl>

                                <hr class="adm-divider" />

                                <div class="adm-toolbar">
                                    <a class="adm-btn adm-btn--secondary adm-btn--sm"
                                        href={admin_nav_href(AdminNav::ApiBehavior)}
                                        onclick={go_to(AdminNav::ApiBehavior)}>
                                        <i class="fas fa-chart-line" aria-hidden="true"></i>
                                        <span>{ "API 行为洞察" }</span>
                                    </a>
                                    <a class="adm-btn adm-btn--secondary adm-btn--sm"
                                        href={admin_nav_href(AdminNav::RuntimeMemory)}
                                        onclick={go_to(AdminNav::RuntimeMemory)}>
                                        <i class="fas fa-memory" aria-hidden="true"></i>
                                        <span>{ "运行时内存" }</span>
                                    </a>
                                    { local_media_overview_link }
                                    <a class="adm-btn adm-btn--primary adm-btn--sm"
                                        href={llm_console} target="_blank" rel="noopener noreferrer">
                                        <span>{ "llm-access 控制台" }</span>
                                        <span aria-hidden="true">{ "↗" }</span>
                                    </a>
                                </div>
                            </div>
                        </section>
                    </div>
                </div>
            }
        },

        AdminNav::CommentTasks => {
            let mut sorted_counts: Vec<(String, usize)> = grouped_status_counts
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            sorted_counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

            let quick_statuses: [(&str, &str); 6] = [
                ("", "全部"),
                ("pending", "待审核"),
                ("approved", "已通过"),
                ("running", "运行中"),
                ("done", "已完成"),
                ("failed", "失败"),
            ];

            html! {
                <div class="adm-stack adm-enter">
                    <AdminPageHeader
                        eyebrow={Some(AttrValue::Static("内容审核"))}
                        title={AttrValue::Static("评论任务")}
                        description={Some(AttrValue::Static(
                            "按文章聚合的读者评论与 AI 回复队列；点击任务 ID 可在右侧抽屉修订正文、添加备注或检视 AI 执行输出。"
                        ))}
                    >
                        { refresh_btn(core_busy) }
                    </AdminPageHeader>
                    { error_banner.clone().unwrap_or_default() }

                    <section class="adm-card">
                        <div class="adm-card__header">
                            <div class="adm-tabs" style="border-bottom: 0;" role="tablist" aria-label="任务状态快捷筛选">
                                { for quick_statuses.into_iter().map(|(val, label)| {
                                    let active = (*status_filter).trim() == val;
                                    let count = if val.is_empty() {
                                        Some(all_status_tasks_total)
                                    } else {
                                        grouped_status_counts.get(val).copied()
                                    };
                                    html! {
                                        <button type="button" role="tab"
                                            aria-selected={if active { "true" } else { "false" }}
                                            class={classes!("adm-tab", active.then_some("is-active"))}
                                            onclick={make_quick_status(val)}>
                                            <span>{ label }</span>
                                            if let Some(c) = count {
                                                <span class="adm-tab__count">{ c }</span>
                                            }
                                        </button>
                                    }
                                }) }
                            </div>
                            <div class="adm-toolbar">
                                <input
                                    type="text"
                                    value={(*status_filter).clone()}
                                    oninput={on_filter_change}
                                    placeholder="自定义状态过滤（如 pending）"
                                    aria-label="按任务状态过滤"
                                    class="adm-input adm-input--mono"
                                    style="width: 210px;"
                                />
                                <button type="button" class="adm-btn adm-btn--secondary" onclick={on_tasks_apply}>
                                    { "应用" }
                                </button>
                            </div>
                        </div>
                        if !sorted_counts.is_empty() {
                            <div class="adm-card__footer">
                                <div class="adm-toolbar">
                                    <span class="adm-faint">
                                        { format!("当前页 {} 篇文章 / {} 条任务", task_groups.len(), grouped_total_tasks) }
                                    </span>
                                    { for sorted_counts.into_iter().map(|(st, count)| html! {
                                        <StatusBadge status={st.clone()} label={Some(AttrValue::from(format!("{st}: {count}")))} />
                                    }) }
                                </div>
                                if core_busy {
                                    <span class="adm-toolbar adm-faint" style="font-size: 12px;">
                                        <LoadingSpinner size={SpinnerSize::Small} />
                                        <span>{ "同步中…" }</span>
                                    </span>
                                }
                            </div>
                        }
                    </section>

                    if core_busy && task_groups.is_empty() {
                        <section class="adm-card">{ table_skeleton(5) }</section>
                    } else if task_groups.is_empty() {
                        <section class="adm-card">
                            <EmptyState
                                icon="fa-inbox"
                                title="暂无匹配的评论任务"
                                hint="当读者提交新评论或调整上方状态筛选条件后，任务将在此处按文章分组展示。"
                            />
                        </section>
                    } else {
                        <div class="adm-stack">
                            { for (*task_groups).iter().map(|group| {
                                let mut g_counts: Vec<(String, usize)> = group
                                    .status_counts
                                    .iter()
                                    .map(|(k, v)| (k.clone(), *v))
                                    .collect();
                                g_counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

                                html! {
                                    <article class="adm-card">
                                        <header class="adm-card__header">
                                            <div class="adm-toolbar" style="min-width: 0;">
                                                <span class="adm-eyebrow" style="margin: 0;">{ "ARTICLE" }</span>
                                                <Link<Route>
                                                    to={Route::ArticleDetail { id: group.article_id.clone() }}
                                                    classes={classes!("adm-link", "adm-mono", "adm-trunc")}
                                                >
                                                    { format!("{} ↗", group.article_id) }
                                                </Link<Route>>
                                            </div>
                                            <div class="adm-toolbar">
                                                { for g_counts.into_iter().map(|(st, count)| html! {
                                                    <StatusBadge status={st.clone()} label={Some(AttrValue::from(format!("{st}: {count}")))} />
                                                }) }
                                                <span class="adm-tag">{ format!("{} 条任务", group.total) }</span>
                                            </div>
                                        </header>
                                        <div class="adm-card__body adm-card__body--flush">
                                            <div class="adm-table-wrap">
                                                <table class="adm-table">
                                                    <thead>
                                                        <tr>
                                                            <th>{ "任务 ID" }</th>
                                                            <th>{ "评论摘要" }</th>
                                                            <th>{ "状态" }</th>
                                                            <th class="num">{ "尝试" }</th>
                                                            <th class="num">{ "创建时间" }</th>
                                                            <th class="actions">{ "操作" }</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        { for group.tasks.iter().map(|task| {
                                                            let task_id = task.task_id.clone();
                                                            let status = task.status.clone();
                                                            let is_busy = task_action_inflight.contains(&task_id);
                                                            let is_selected = (*selected_task_id).as_deref() == Some(task_id.as_str());
                                                            let can_approve = !is_busy && (status == "pending" || status == "failed");
                                                            let can_approve_run = !is_busy && (status == "pending" || status == "approved" || status == "failed");
                                                            let can_retry = !is_busy && status == "failed";
                                                            let can_reject = !is_busy && (status == "pending" || status == "approved" || status == "failed");
                                                            let can_delete = !is_busy && status != "running";

                                                            let select_click = {
                                                                let on_select_task = on_select_task.clone();
                                                                let task_id = task_id.clone();
                                                                Callback::from(move |_| on_select_task.emit(task_id.clone()))
                                                            };
                                                            let approve_click = {
                                                                let run_task_action = run_task_action.clone();
                                                                let task_id = task_id.clone();
                                                                Callback::from(move |_| run_task_action.emit((task_id.clone(), "approve".to_string())))
                                                            };
                                                            let approve_run_click = {
                                                                let run_task_action = run_task_action.clone();
                                                                let task_id = task_id.clone();
                                                                Callback::from(move |_| run_task_action.emit((task_id.clone(), "approve_run".to_string())))
                                                            };
                                                            let retry_click = {
                                                                let run_task_action = run_task_action.clone();
                                                                let task_id = task_id.clone();
                                                                Callback::from(move |_| run_task_action.emit((task_id.clone(), "retry".to_string())))
                                                            };
                                                            let reject_click = {
                                                                let run_task_action = run_task_action.clone();
                                                                let task_id = task_id.clone();
                                                                Callback::from(move |_| run_task_action.emit((task_id.clone(), "reject".to_string())))
                                                            };
                                                            let delete_click = {
                                                                let run_task_action = run_task_action.clone();
                                                                let task_id = task_id.clone();
                                                                Callback::from(move |_| run_task_action.emit((task_id.clone(), "delete".to_string())))
                                                            };

                                                            html! {
                                                                <tr class={classes!(is_selected.then_some("is-selected"))}>
                                                                    <td class="mono" style="max-width: 190px;">
                                                                        <button type="button"
                                                                            class="adm-link adm-mono adm-trunc"
                                                                            style="border: 0; background: none; padding: 0; text-align: left;"
                                                                            title={task.task_id.clone()}
                                                                            onclick={select_click}>
                                                                            { &task.task_id }
                                                                        </button>
                                                                    </td>
                                                                    <td style="max-width: 280px;">
                                                                        <span class="adm-trunc" title={task.comment_text.clone()}>
                                                                            { &task.comment_text }
                                                                        </span>
                                                                        if let Some(reason) = task.failure_reason.as_deref().filter(|r| !r.is_empty()) {
                                                                            <span class="adm-trunc" style="font-size: 11.5px; color: var(--err);" title={reason.to_string()}>
                                                                                { reason }
                                                                            </span>
                                                                        }
                                                                    </td>
                                                                    <td><StatusBadge status={status} /></td>
                                                                    <td class="num">{ task.attempt_count }</td>
                                                                    <td class="num">{ format_ms(task.created_at) }</td>
                                                                    <td class="actions">
                                                                        <div class="adm-row-actions">
                                                                            <button type="button" class="adm-btn adm-btn--ghost adm-btn--sm"
                                                                                onclick={approve_click} disabled={!can_approve}>
                                                                                { "通过" }
                                                                            </button>
                                                                            <button type="button" class="adm-btn adm-btn--primary adm-btn--sm"
                                                                                onclick={approve_run_click} disabled={!can_approve_run}>
                                                                                { "通过并运行" }
                                                                            </button>
                                                                            if can_retry {
                                                                                <button type="button" class="adm-btn adm-btn--secondary adm-btn--sm"
                                                                                    onclick={retry_click}>
                                                                                    { "重试" }
                                                                                </button>
                                                                            }
                                                                            <Link<Route>
                                                                                to={Route::AdminCommentRuns { task_id: task.task_id.clone() }}
                                                                                classes={classes!("adm-btn", "adm-btn--ghost", "adm-btn--sm")}
                                                                            >
                                                                                { "AI 流 ↗" }
                                                                            </Link<Route>>
                                                                            <button type="button" class="adm-btn adm-btn--ghost adm-btn--sm"
                                                                                onclick={reject_click} disabled={!can_reject}>
                                                                                { "驳回" }
                                                                            </button>
                                                                            <button type="button" class="adm-btn adm-btn--danger adm-btn--sm"
                                                                                onclick={delete_click} disabled={!can_delete}>
                                                                                { "删除" }
                                                                            </button>
                                                                        </div>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }) }
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </article>
                                }
                            }) }
                        </div>
                    }

                    <div class="adm-pager">
                        <span class="adm-pager__info">
                            { format!(
                                "第 {} / {} 页 · 共 {} 篇文章{}",
                                *tasks_page,
                                tasks_total_pages,
                                *tasks_total,
                                if *tasks_has_more { " · 仍有更多" } else { "" }
                            ) }
                        </span>
                        <Pagination current_page={*tasks_page} total_pages={tasks_total_pages} on_page_change={on_tasks_page_change} />
                    </div>

                    <Drawer
                        open={selected_task.is_some()}
                        on_close={{
                            let selected_task_id = selected_task_id.clone();
                            let selected_task = selected_task.clone();
                            let selected_task_ai_output = selected_task_ai_output.clone();
                            Callback::from(move |_| {
                                selected_task_id.set(None);
                                selected_task.set(None);
                                selected_task_ai_output.set(None);
                            })
                        }}
                        title={(*selected_task).as_ref().map(|t| AttrValue::from(format!("任务详情 · {}", t.task_id))).unwrap_or_default()}
                    >
                        if let Some(task) = (*selected_task).clone() {
                            <div class="adm-stack">
                                <dl class="adm-kv">
                                    <div>
                                        <dt>{ "当前状态" }</dt>
                                        <dd><StatusBadge status={task.status.clone()} /></dd>
                                    </div>
                                    <div>
                                        <dt>{ "所属文章" }</dt>
                                        <dd class="adm-mono">{ &task.article_id }</dd>
                                    </div>
                                    <div>
                                        <dt>{ "来源归属" }</dt>
                                        <dd class="adm-mono">{ format!("{} · {}", task.client_ip, task.ip_region) }</dd>
                                    </div>
                                    <div>
                                        <dt>{ "尝试次数" }</dt>
                                        <dd class="adm-mono">{ task.attempt_count }</dd>
                                    </div>
                                    <div>
                                        <dt>{ "创建时间" }</dt>
                                        <dd class="adm-mono">{ format_ms(task.created_at) }</dd>
                                    </div>
                                    <div>
                                        <dt>{ "更新时间" }</dt>
                                        <dd class="adm-mono">{ format_ms(task.updated_at) }</dd>
                                    </div>
                                </dl>

                                if let Some(reason) = task.failure_reason.as_deref().filter(|s| !s.trim().is_empty()) {
                                    <div class="adm-banner adm-banner--err">
                                        <i class="fas fa-circle-exclamation" aria-hidden="true"></i>
                                        <div class="adm-banner__body">{ format!("失败原因：{reason}") }</div>
                                    </div>
                                }

                                if let Some(selected) = task.selected_text.as_deref().filter(|s| !s.trim().is_empty()) {
                                    <div class="adm-field">
                                        <span class="adm-label">{ "划线引用片段 (selected_text)" }</span>
                                        <pre class="adm-code">{ selected }</pre>
                                    </div>
                                }

                                <label class="adm-field">
                                    <span class="adm-label">{ "评论内容 (comment_text)" }</span>
                                    <textarea
                                        class="adm-textarea"
                                        style="min-height: 120px;"
                                        value={task.comment_text.clone()}
                                        oninput={on_selected_task_comment_change}
                                    />
                                </label>

                                <label class="adm-field">
                                    <span class="adm-label">{ "管理员备注 (admin_note)" }</span>
                                    <textarea
                                        class="adm-textarea"
                                        style="min-height: 84px;"
                                        value={task.admin_note.clone().unwrap_or_default()}
                                        oninput={on_selected_task_note_change}
                                    />
                                </label>

                                <div class="adm-actions">
                                    <button type="button" class="adm-btn adm-btn--primary" onclick={on_save_task}>
                                        { "保存任务修改" }
                                    </button>
                                </div>

                                if let Some(ai_output) = (*selected_task_ai_output).clone() {
                                    <section class="adm-card adm-card--flat">
                                        <div class="adm-card__header">
                                            <div>
                                                <h4 class="adm-card__title">
                                                    { format!("AI 运行记录 ({})", ai_output.runs.len()) }
                                                </h4>
                                                <p class="adm-card__sub">
                                                    { format!("已捕获流式片段：{} 条", ai_output.chunks.len()) }
                                                </p>
                                            </div>
                                            <Link<Route>
                                                to={Route::AdminCommentRuns { task_id: task.task_id.clone() }}
                                                classes={classes!("adm-btn", "adm-btn--secondary", "adm-btn--sm")}
                                            >
                                                { "打开完整流式页 ↗" }
                                            </Link<Route>>
                                        </div>
                                        <div class="adm-card__body adm-stack-sm">
                                            if ai_output.runs.is_empty() {
                                                <p class="adm-hint">{ "该任务尚无 AI 运行记录。" }</p>
                                            } else {
                                                <div class="adm-toolbar">
                                                    { for ai_output.runs.iter().map(|run| {
                                                        let run_id = run.run_id.clone();
                                                        let selected = ai_output.selected_run_id.as_deref() == Some(run_id.as_str());
                                                        let click = {
                                                            let on_select_task_ai_run = on_select_task_ai_run.clone();
                                                            let run_id = run_id.clone();
                                                            Callback::from(move |_| on_select_task_ai_run.emit(run_id.clone()))
                                                        };
                                                        html! {
                                                            <button type="button"
                                                                class={classes!(
                                                                    "adm-btn",
                                                                    "adm-btn--sm",
                                                                    "adm-mono",
                                                                    if selected { "adm-btn--primary" } else { "adm-btn--secondary" }
                                                                )}
                                                                onclick={click}>
                                                                { format!("{} · {}", run.status, run.run_id) }
                                                            </button>
                                                        }
                                                    }) }
                                                </div>
                                            }

                                            if !ai_output.chunks.is_empty() {
                                                <div class="adm-stack-sm">
                                                    { for ai_output.chunks.iter().rev().take(10).rev().map(|chunk| {
                                                        let is_err = chunk.stream == "stderr";
                                                        let pill_cls = if is_err {
                                                            "adm-pill adm-pill--err"
                                                        } else {
                                                            "adm-pill adm-pill--info"
                                                        };
                                                        html! {
                                                            <div class="adm-field">
                                                                <div class="adm-toolbar">
                                                                    <span class={pill_cls}>{ &chunk.stream }</span>
                                                                    <span class="adm-mono adm-faint">{ format!("batch={}", chunk.batch_index) }</span>
                                                                </div>
                                                                <pre class="adm-code">{ &chunk.content }</pre>
                                                            </div>
                                                        }
                                                    }) }
                                                </div>
                                            }
                                        </div>
                                    </section>
                                }
                            </div>
                        }
                    </Drawer>
                </div>
            }
        },

        AdminNav::PublishedComments => html! {
            <div class="adm-stack adm-enter">
                <AdminPageHeader
                    eyebrow={Some(AttrValue::Static("内容审核"))}
                    title={AttrValue::Static("已发布评论")}
                    description={Some(AttrValue::Static(
                        "已公开展示在文章底部的读者评论与 AI 回复，支持原位修订或删除撤回。"
                    ))}
                >
                    { refresh_btn(core_busy) }
                </AdminPageHeader>
                { error_banner.clone().unwrap_or_default() }

                <section class="adm-card">
                    <div class="adm-card__header">
                        <div>
                            <h2 class="adm-card__title">{ "已发布评论列表" }</h2>
                            <p class="adm-card__sub">
                                { format!("当前页 {} 条 · 累计 {} 条", published_comments.len(), *published_total) }
                            </p>
                        </div>
                        if core_busy && !published_comments.is_empty() {
                            <span class="adm-toolbar adm-faint" style="font-size: 12px;">
                                <LoadingSpinner size={SpinnerSize::Small} />
                                <span>{ "同步中…" }</span>
                            </span>
                        }
                    </div>

                    if core_busy && published_comments.is_empty() {
                        { table_skeleton(6) }
                    } else if published_comments.is_empty() {
                        <EmptyState
                            icon="fa-comments"
                            title="暂无已发布评论"
                            hint="审核通过的评论与 AI 回复会公开展示并汇总在此处。"
                        />
                    } else {
                        <div class="adm-card__body adm-card__body--flush">
                            <div class="adm-table-wrap">
                                <table class="adm-table">
                                    <thead>
                                        <tr>
                                            <th>{ "评论 ID" }</th>
                                            <th>{ "所属文章" }</th>
                                            <th>{ "读者 / 正文摘要" }</th>
                                            <th>{ "关联任务" }</th>
                                            <th class="num">{ "发布时间" }</th>
                                            <th class="actions">{ "操作" }</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        { for (*published_comments).iter().map(|comment| {
                                            let is_selected = (*selected_published_id).as_deref() == Some(comment.comment_id.as_str());
                                            let select_click = {
                                                let on_select_published = on_select_published.clone();
                                                let comment = comment.clone();
                                                Callback::from(move |_| on_select_published.emit(comment.clone()))
                                            };
                                            let delete_click = {
                                                let delete_published_action = delete_published_action.clone();
                                                let comment_id = comment.comment_id.clone();
                                                Callback::from(move |_| delete_published_action.emit(comment_id.clone()))
                                            };
                                            html! {
                                                <tr class={classes!(is_selected.then_some("is-selected"))}>
                                                    <td class="mono" style="max-width: 170px;">
                                                        <span class="adm-trunc" title={comment.comment_id.clone()}>{ &comment.comment_id }</span>
                                                    </td>
                                                    <td class="mono" style="max-width: 200px;">
                                                        <Link<Route>
                                                            to={Route::ArticleDetail { id: comment.article_id.clone() }}
                                                            classes={classes!("adm-link", "adm-mono", "adm-trunc")}
                                                        >
                                                            { format!("{} ↗", comment.article_id) }
                                                        </Link<Route>>
                                                    </td>
                                                    <td style="max-width: 280px;">
                                                        <span class="adm-trunc" title={comment.comment_text.clone()}>
                                                            <strong>{ &comment.author_name }</strong>
                                                            { format!(" · {}", comment.comment_text) }
                                                        </span>
                                                    </td>
                                                    <td class="mono" style="max-width: 160px;">
                                                        <span class="adm-trunc adm-faint" title={comment.task_id.clone()}>{ &comment.task_id }</span>
                                                    </td>
                                                    <td class="num">{ format_ms(comment.published_at) }</td>
                                                    <td class="actions">
                                                        <div class="adm-row-actions">
                                                            <button type="button" class="adm-btn adm-btn--ghost adm-btn--sm" onclick={select_click}>
                                                                { "编辑" }
                                                            </button>
                                                            <button type="button" class="adm-btn adm-btn--danger adm-btn--sm" onclick={delete_click}>
                                                                { "删除" }
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        }) }
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }

                    <div class="adm-card__footer">
                        <span class="adm-pager__info">
                            { format!(
                                "第 {} / {} 页 · 共 {} 条{}",
                                *published_page,
                                published_total_pages,
                                *published_total,
                                if *published_has_more { " · 仍有更多" } else { "" }
                            ) }
                        </span>
                        <Pagination current_page={*published_page} total_pages={published_total_pages} on_page_change={on_published_page_change} />
                    </div>
                </section>

                <Drawer
                    open={selected_published.is_some()}
                    on_close={{
                        let selected_published_id = selected_published_id.clone();
                        let selected_published = selected_published.clone();
                        Callback::from(move |_| {
                            selected_published_id.set(None);
                            selected_published.set(None);
                        })
                    }}
                    title={(*selected_published).as_ref().map(|c| AttrValue::from(format!("已发布详情 · {}", c.comment_id))).unwrap_or_default()}
                >
                    if let Some(comment) = (*selected_published).clone() {
                        <div class="adm-stack">
                            <dl class="adm-kv">
                                <div>
                                    <dt>{ "读者昵称" }</dt>
                                    <dd>{ &comment.author_name }</dd>
                                </div>
                                <div>
                                    <dt>{ "所属文章" }</dt>
                                    <dd class="adm-mono">{ &comment.article_id }</dd>
                                </div>
                                <div>
                                    <dt>{ "关联任务" }</dt>
                                    <dd class="adm-mono">{ &comment.task_id }</dd>
                                </div>
                                <div>
                                    <dt>{ "发布时间" }</dt>
                                    <dd class="adm-mono">{ format_ms(comment.published_at) }</dd>
                                </div>
                            </dl>

                            <label class="adm-field">
                                <span class="adm-label">{ "评论正文 (comment_text)" }</span>
                                <textarea
                                    class="adm-textarea"
                                    style="min-height: 110px;"
                                    value={comment.comment_text.clone()}
                                    oninput={on_selected_published_comment_change}
                                />
                            </label>

                            <label class="adm-field">
                                <span class="adm-label">{ "AI 回复 Markdown (ai_reply_markdown)" }</span>
                                <textarea
                                    class="adm-textarea adm-input--mono"
                                    style="min-height: 180px;"
                                    value={comment.ai_reply_markdown.clone().unwrap_or_default()}
                                    oninput={on_selected_published_ai_change}
                                />
                            </label>

                            <div class="adm-actions">
                                <button type="button" class="adm-btn adm-btn--primary" onclick={on_save_published}>
                                    { "保存已发布修订" }
                                </button>
                            </div>
                        </div>
                    }
                </Drawer>
            </div>
        },

        AdminNav::AuditLogs => html! {
            <div class="adm-stack adm-enter">
                <AdminPageHeader
                    eyebrow={Some(AttrValue::Static("内容审核"))}
                    title={AttrValue::Static("审计日志")}
                    description={Some(AttrValue::Static(
                        "按时间倒序记录后台对评论任务的审批、修改、重试与删除轨迹。"
                    ))}
                >
                    { refresh_btn(audit_busy) }
                </AdminPageHeader>
                { error_banner.clone().unwrap_or_default() }

                <section class="adm-card">
                    <div class="adm-card__header">
                        <div>
                            <h2 class="adm-card__title">{ "操作审计列表" }</h2>
                            <p class="adm-card__sub">
                                { format!("当前页 {} 条 · 累计 {} 条", audit_logs.len(), *audit_total) }
                            </p>
                        </div>
                        <div class="adm-toolbar">
                            <input
                                type="text"
                                value={(*audit_task_filter).clone()}
                                oninput={on_audit_task_filter_change}
                                placeholder="按 task_id 过滤"
                                aria-label="按 task_id 过滤审计日志"
                                class="adm-input adm-input--mono"
                                style="width: 190px;"
                            />
                            <input
                                type="text"
                                value={(*audit_action_filter).clone()}
                                oninput={on_audit_action_filter_change}
                                placeholder="按 action 过滤"
                                aria-label="按 action 过滤审计日志"
                                class="adm-input adm-input--mono"
                                style="width: 150px;"
                            />
                            <button type="button" class="adm-btn adm-btn--secondary" onclick={on_refresh_audit_click}>
                                { "筛选" }
                            </button>
                        </div>
                    </div>

                    if audit_busy && audit_logs.is_empty() {
                        { table_skeleton(6) }
                    } else if audit_logs.is_empty() {
                        <EmptyState
                            icon="fa-clock-rotate-left"
                            title="暂无审计记录"
                            hint="对评论任务的审核、修订或删除操作会自动记录在此处。"
                        />
                    } else {
                        <div class="adm-card__body adm-card__body--flush">
                            <div class="adm-table-wrap">
                                <table class="adm-table">
                                    <thead>
                                        <tr>
                                            <th>{ "日志 ID" }</th>
                                            <th>{ "任务 ID" }</th>
                                            <th>{ "动作" }</th>
                                            <th>{ "操作者" }</th>
                                            <th class="num">{ "时间" }</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        { for (*audit_logs).iter().map(|log| html! {
                                            <tr>
                                                <td class="mono" style="max-width: 190px;">
                                                    <span class="adm-trunc" title={log.log_id.clone()}>{ &log.log_id }</span>
                                                </td>
                                                <td class="mono" style="max-width: 190px;">
                                                    <span class="adm-trunc" title={log.task_id.clone()}>{ &log.task_id }</span>
                                                </td>
                                                <td><StatusBadge status={log.action.clone()} /></td>
                                                <td class="mono">{ &log.operator }</td>
                                                <td class="num">{ format_ms(log.created_at) }</td>
                                            </tr>
                                        }) }
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }

                    <div class="adm-card__footer">
                        <span class="adm-pager__info">
                            { format!(
                                "第 {} / {} 页 · 共 {} 条{}",
                                *audit_page,
                                audit_total_pages,
                                *audit_total,
                                if *audit_has_more { " · 仍有更多" } else { "" }
                            ) }
                        </span>
                        <Pagination current_page={*audit_page} total_pages={audit_total_pages} on_page_change={on_audit_page_change} />
                    </div>
                </section>
            </div>
        },

        AdminNav::MusicWishes => html! {
            <div class="adm-stack adm-enter">
                <AdminPageHeader
                    eyebrow={Some(AttrValue::Static("共建队列"))}
                    title={AttrValue::Static("点歌心愿")}
                    description={Some(AttrValue::Static(
                        "读者提交的曲目入库心愿；批准后自动触发检索、解密与 LanceDB 音乐库入库流程。"
                    ))}
                >
                    { refresh_btn(wishes_busy) }
                </AdminPageHeader>
                { error_banner.clone().unwrap_or_default() }

                <section class="adm-card">
                    <div class="adm-card__header">
                        <div style="flex: 1 1 280px; max-width: 420px;">
                            <SearchBox
                                value={(*music_wish_search).clone()}
                                on_change={on_music_wish_search_change}
                                placeholder={AttrValue::Static("搜索歌曲 / 歌手 / 昵称 / 留言 / 状态")}
                            />
                        </div>
                        <div class="adm-toolbar adm-mono adm-faint" style="font-size: 12px;">
                            if !music_wish_query_lower.is_empty() {
                                <span>{ format!("当前页匹配 {}/{}", music_wish_matched, music_wishes.len()) }</span>
                            } else {
                                <span>{ format!("当前页 {} 条 · 累计 {} 条", music_wishes.len(), *music_wish_total) }</span>
                            }
                            if wishes_busy && !music_wishes.is_empty() {
                                <LoadingSpinner size={SpinnerSize::Small} />
                            }
                        </div>
                    </div>

                    if wishes_busy && music_wishes.is_empty() {
                        { table_skeleton(6) }
                    } else if music_wishes.is_empty() {
                        <EmptyState
                            icon="fa-music"
                            title="暂无点歌心愿"
                            hint="还没有读者提交点歌心愿请求。"
                        />
                    } else if filtered_music_wishes.is_empty() {
                        <EmptyState
                            icon="fa-filter"
                            title="没有匹配的点歌心愿"
                            hint="当前搜索关键词在本页没有命中记录，请尝试清空或更换关键词。"
                        />
                    } else {
                        <div class="adm-card__body adm-card__body--flush">
                            <div class="adm-table-wrap">
                                <table class="adm-table">
                                    <thead>
                                        <tr>
                                            <th>{ "曲目 / 留言" }</th>
                                            <th>{ "歌手提示" }</th>
                                            <th>{ "昵称" }</th>
                                            <th>{ "状态" }</th>
                                            <th>{ "地区" }</th>
                                            <th class="num">{ "提交时间" }</th>
                                            <th class="actions">{ "操作" }</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        { for filtered_music_wishes.iter().map(|wish| {
                                            let wid = wish.wish_id.clone();
                                            let inflight = music_wish_action_inflight.contains(&wid);
                                            html! {
                                                <MusicWishRow
                                                    wish={wish.clone()}
                                                    inflight={inflight}
                                                    on_action={on_music_wish_action.clone()}
                                                />
                                            }
                                        }) }
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }

                    <div class="adm-card__footer">
                        <span class="adm-pager__info">
                            { format!(
                                "第 {} / {} 页 · 共 {} 条{}",
                                *music_wish_page,
                                music_wish_total_pages,
                                *music_wish_total,
                                if *music_wish_has_more { " · 仍有更多" } else { "" }
                            ) }
                        </span>
                        <Pagination current_page={*music_wish_page} total_pages={music_wish_total_pages} on_page_change={on_music_wish_page_change} />
                    </div>
                </section>
            </div>
        },

        AdminNav::ArticleRequests => html! {
            <div class="adm-stack adm-enter">
                <AdminPageHeader
                    eyebrow={Some(AttrValue::Static("共建队列"))}
                    title={AttrValue::Static("文章入库")}
                    description={Some(AttrValue::Static(
                        "读者推荐的外部文章转载与双语翻译入库队列；批准后由后台 AI 抓取归一化并写入文库。"
                    ))}
                >
                    { refresh_btn(requests_busy) }
                </AdminPageHeader>
                { error_banner.clone().unwrap_or_default() }

                <section class="adm-card">
                    <div class="adm-card__header">
                        <div style="flex: 1 1 280px; max-width: 420px;">
                            <SearchBox
                                value={(*article_request_search).clone()}
                                on_change={on_article_request_search_change}
                                placeholder={AttrValue::Static("搜索 URL / 标题 / 昵称 / 留言 / 状态")}
                            />
                        </div>
                        <div class="adm-toolbar adm-mono adm-faint" style="font-size: 12px;">
                            if !article_request_query_lower.is_empty() {
                                <span>{ format!("当前页匹配 {}/{}", article_request_matched, article_requests.len()) }</span>
                            } else {
                                <span>{ format!("当前页 {} 条 · 累计 {} 条", article_requests.len(), *article_request_total) }</span>
                            }
                            if requests_busy && !article_requests.is_empty() {
                                <LoadingSpinner size={SpinnerSize::Small} />
                            }
                        </div>
                    </div>

                    if requests_busy && article_requests.is_empty() {
                        { table_skeleton(6) }
                    } else if article_requests.is_empty() {
                        <EmptyState
                            icon="fa-file-import"
                            title="暂无文章入库请求"
                            hint="还没有读者提交外部文章入库请求。"
                        />
                    } else if filtered_article_requests.is_empty() {
                        <EmptyState
                            icon="fa-filter"
                            title="没有匹配的入库请求"
                            hint="当前搜索关键词在本页没有命中记录，请尝试清空或更换关键词。"
                        />
                    } else {
                        <div class="adm-card__body adm-card__body--flush">
                            <div class="adm-table-wrap">
                                <table class="adm-table">
                                    <thead>
                                        <tr>
                                            <th>{ "原文链接 / 留言" }</th>
                                            <th>{ "标题提示" }</th>
                                            <th>{ "昵称" }</th>
                                            <th>{ "状态" }</th>
                                            <th>{ "地区" }</th>
                                            <th class="num">{ "提交时间" }</th>
                                            <th class="actions">{ "操作" }</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        { for filtered_article_requests.iter().map(|req| {
                                            let rid = req.request_id.clone();
                                            let inflight = article_request_action_inflight.contains(&rid);
                                            html! {
                                                <ArticleRequestRow
                                                    request={req.clone()}
                                                    inflight={inflight}
                                                    on_action={on_article_request_action.clone()}
                                                />
                                            }
                                        }) }
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }

                    <div class="adm-card__footer">
                        <span class="adm-pager__info">
                            { format!(
                                "第 {} / {} 页 · 共 {} 条{}",
                                *article_request_page,
                                article_request_total_pages,
                                *article_request_total,
                                if *article_request_has_more { " · 仍有更多" } else { "" }
                            ) }
                        </span>
                        <Pagination current_page={*article_request_page} total_pages={article_request_total_pages} on_page_change={on_article_request_page_change} />
                    </div>
                </section>
            </div>
        },

        AdminNav::ApiBehavior => html! {
            <div class="adm-stack adm-enter">
                <AdminPageHeader
                    eyebrow={Some(AttrValue::Static("运行洞察"))}
                    title={AttrValue::Static("API 行为分析")}
                    description={Some(AttrValue::Static(
                        "按时间窗口、路径、页面、设备与状态码观察站点访问量、终端分布及接口响应耗时。"
                    ))}
                >
                    <button type="button" class="adm-btn adm-btn--danger"
                        onclick={{
                            let pending_cleanup_logs = pending_cleanup_logs.clone();
                            Callback::from(move |_| pending_cleanup_logs.set(true))
                        }}>
                        { "清理旧日志" }
                    </button>
                    { refresh_btn(behavior_busy) }
                </AdminPageHeader>
                { error_banner.clone().unwrap_or_default() }

                <section class="adm-card">
                    <div class="adm-card__body">
                        <div class="adm-form-grid">
                            <label class="adm-field">
                                <span class="adm-label">{ "最近天数 (days)" }</span>
                                <input
                                    type="number"
                                    value={(*behavior_days).clone()}
                                    oninput={on_behavior_days_change}
                                    placeholder="30"
                                    aria-label="按最近天数筛选行为日志"
                                    disabled={!(*behavior_date).is_empty()}
                                    class="adm-input adm-input--mono"
                                />
                            </label>
                            <label class="adm-field">
                                <span class="adm-label">{ "指定日期 (覆盖天数)" }</span>
                                <input
                                    type="date"
                                    value={(*behavior_date).clone()}
                                    oninput={on_behavior_date_change}
                                    aria-label="按具体日期筛选行为日志"
                                    class="adm-input adm-input--mono"
                                />
                            </label>
                            <label class="adm-field">
                                <span class="adm-label">{ "接口路径包含" }</span>
                                <input
                                    type="text"
                                    value={(*behavior_path_filter).clone()}
                                    oninput={on_behavior_path_filter_change}
                                    placeholder="/api/articles"
                                    aria-label="按请求路径包含内容筛选"
                                    class="adm-input adm-input--mono"
                                />
                            </label>
                            <label class="adm-field">
                                <span class="adm-label">{ "页面路径包含" }</span>
                                <input
                                    type="text"
                                    value={(*behavior_page_filter).clone()}
                                    oninput={on_behavior_page_filter_change}
                                    placeholder="/posts/"
                                    aria-label="按页面标识包含内容筛选"
                                    class="adm-input adm-input--mono"
                                />
                            </label>
                            <label class="adm-field">
                                <span class="adm-label">{ "设备类型" }</span>
                                <input
                                    type="text"
                                    value={(*behavior_device_filter).clone()}
                                    oninput={on_behavior_device_filter_change}
                                    placeholder="desktop / mobile"
                                    aria-label="按设备筛选"
                                    class="adm-input adm-input--mono"
                                />
                            </label>
                            <label class="adm-field">
                                <span class="adm-label">{ "HTTP 状态码" }</span>
                                <input
                                    type="number"
                                    value={(*behavior_status_filter).clone()}
                                    oninput={on_behavior_status_filter_change}
                                    placeholder="200 / 404 / 500"
                                    title="按 HTTP 状态码筛选，例如 200 / 404 / 500"
                                    aria-label="按 HTTP 状态码筛选"
                                    class="adm-input adm-input--mono"
                                />
                            </label>
                        </div>
                        <div class="adm-actions" style="margin-top: 14px;">
                            <button type="button" class="adm-btn adm-btn--primary" onclick={on_behavior_apply}>
                                { "应用筛选" }
                            </button>
                            if !(*behavior_date).is_empty() {
                                <button type="button" class="adm-btn adm-btn--ghost adm-btn--sm"
                                    onclick={{
                                        let behavior_date = behavior_date.clone();
                                        Callback::from(move |_| behavior_date.set(String::new()))
                                    }}>
                                    { "清除指定日期" }
                                </button>
                            }
                        </div>
                    </div>
                </section>

                if let Some(overview) = (*behavior_overview).clone() {
                    <>
                        <section class="adm-kpis" aria-label="API 行为核心指标">
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "事件总数" }</div>
                                <div class="adm-kpi__value">{ overview.total_events }</div>
                                <div class="adm-kpi__meta">{ "筛选时间窗口累计请求" }</div>
                            </div>
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "独立访客 IP" }</div>
                                <div class="adm-kpi__value">{ overview.unique_ips }</div>
                                <div class="adm-kpi__meta">{ "去重客户端地址数" }</div>
                            </div>
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "独立页面" }</div>
                                <div class="adm-kpi__value">{ overview.unique_pages }</div>
                                <div class="adm-kpi__meta">{ "被访问的前端路径数" }</div>
                            </div>
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "平均延迟" }</div>
                                <div class="adm-kpi__value">{ format!("{:.1} ms", overview.avg_latency_ms) }</div>
                                <div class="adm-kpi__meta">{ "服务端响应均值" }</div>
                            </div>
                        </section>

                        <section class="adm-card">
                            <div class="adm-card__header">
                                <h2 class="adm-card__title">{ "请求量趋势" }</h2>
                            </div>
                            <div class="adm-card__body">
                                <ViewTrendChart
                                    points={to_view_points(&overview.timeseries)}
                                    empty_text={"暂无行为趋势数据".to_string()}
                                />
                            </div>
                        </section>

                        <div class="adm-grid-3">
                            { behavior_distribution_card("热门接口 (Top Endpoints)", &overview.top_endpoints, true) }
                            { behavior_distribution_card("热门页面 (Top Pages)", &overview.top_pages, true) }
                            { behavior_distribution_card("设备分布", &overview.device_distribution, false) }
                            { behavior_distribution_card("浏览器分布", &overview.browser_distribution, false) }
                            { behavior_distribution_card("操作系统分布", &overview.os_distribution, false) }
                            { behavior_distribution_card("地域分布", &overview.region_distribution, false) }
                        </div>
                    </>
                } else if *behavior_overview_loading {
                    <>
                        { kpi_skeleton(4) }
                        <div class="adm-grid-3" aria-hidden="true">
                            { for (0..6).map(|_| html! {
                                <div class="adm-card">
                                    <div class="adm-card__body adm-stack-sm">
                                        <span class="adm-skel" style="width: 45%;"></span>
                                        <span class="adm-skel adm-skel--row"></span>
                                        <span class="adm-skel adm-skel--row"></span>
                                    </div>
                                </div>
                            }) }
                        </div>
                    </>
                }

                <section class="adm-card">
                    <div class="adm-card__header">
                        <div>
                            <h2 class="adm-card__title">
                                { if (*behavior_date).is_empty() {
                                    "最近请求明细".to_string()
                                } else {
                                    format!("{} 请求明细", *behavior_date)
                                } }
                            </h2>
                            <p class="adm-card__sub">
                                { format!(
                                    "显示第 {}–{} 条 / 共 {} 条",
                                    if behavior_events.is_empty() { 0 } else { *behavior_offset + 1 },
                                    *behavior_offset + behavior_events.len(),
                                    *behavior_total
                                ) }
                            </p>
                        </div>
                        <button
                            type="button"
                            class="adm-btn adm-btn--secondary adm-btn--sm"
                            onclick={on_behavior_refresh_events}
                            disabled={*behavior_events_loading}
                        >
                            <i class={classes!("fas", "fa-rotate-right", (*behavior_events_loading).then_some("fa-spin"))} aria-hidden="true"></i>
                            <span>{ if *behavior_events_loading { "刷新中…" } else { "仅刷新事件" } }</span>
                        </button>
                    </div>

                    if *behavior_events_loading && behavior_events.is_empty() {
                        { table_skeleton(6) }
                    } else if behavior_events.is_empty() {
                        <EmptyState
                            icon="fa-chart-line"
                            title="暂无匹配的行为事件"
                            hint="调整上方时间范围或路径/状态码条件后重新应用筛选。"
                        />
                    } else {
                        <div class="adm-card__body adm-card__body--flush">
                            <div class="adm-table-wrap">
                                <table class="adm-table">
                                    <thead>
                                        <tr>
                                            <th class="num">{ "时间" }</th>
                                            <th>{ "页面路径" }</th>
                                            <th>{ "请求接口" }</th>
                                            <th class="num">{ "状态" }</th>
                                            <th>{ "设备" }</th>
                                            <th>{ "浏览器 / 系统" }</th>
                                            <th>{ "IP / 归属地" }</th>
                                            <th class="num">{ "耗时" }</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        { for (*behavior_events).iter().map(|event| {
                                            let full_api = if event.query.is_empty() {
                                                format!("{} {}", event.method, event.path)
                                            } else {
                                                format!("{} {}?{}", event.method, event.path, event.query)
                                            };
                                            let ip_reg = format!("{} · {}", event.client_ip, event.ip_region);
                                            html! {
                                                <tr>
                                                    <td class="num">{ format_ms(event.occurred_at) }</td>
                                                    <td class="mono" style="max-width: 200px;">
                                                        <div style="display: flex; align-items: center; gap: 6px; min-width: 0;">
                                                            <span class="adm-trunc" title={event.page_path.clone()}>{ &event.page_path }</span>
                                                            <CopyButton text={event.page_path.clone()} />
                                                        </div>
                                                    </td>
                                                    <td class="mono" style="max-width: 260px;">
                                                        <div style="display: flex; align-items: center; gap: 6px; min-width: 0;">
                                                            <span class="adm-trunc" title={full_api.clone()}>
                                                                { format!("{} {}", event.method, event.path) }
                                                            </span>
                                                            <CopyButton text={full_api} />
                                                        </div>
                                                    </td>
                                                    <td class="num">{ http_status_pill(event.status_code) }</td>
                                                    <td>{ &event.device_type }</td>
                                                    <td class="adm-muted" style="white-space: nowrap;">
                                                        { format!("{} / {}", event.browser_family, event.os_family) }
                                                    </td>
                                                    <td class="mono" style="white-space: nowrap;">
                                                        <div style="display: inline-flex; align-items: center; gap: 6px;">
                                                            <span>{ &ip_reg }</span>
                                                            <CopyButton text={format!("{}/{}", event.client_ip, event.ip_region)} />
                                                        </div>
                                                    </td>
                                                    <td class="num">{ format!("{} ms", event.latency_ms) }</td>
                                                </tr>
                                            }
                                        }) }
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }

                    <div class="adm-card__footer">
                        <span class="adm-pager__info">
                            { format!(
                                "第 {} / {} 页 · 共 {} 条{}",
                                *behavior_page,
                                behavior_total_pages,
                                *behavior_total,
                                if *behavior_has_more { " · 仍有更多" } else { "" }
                            ) }
                        </span>
                        <Pagination current_page={*behavior_page} total_pages={behavior_total_pages} on_page_change={on_behavior_page_change} />
                    </div>
                </section>
            </div>
        },

        AdminNav::RuntimeMemory => html! {
            <div class="adm-stack adm-enter">
                <AdminPageHeader
                    eyebrow={Some(AttrValue::Static("运行洞察"))}
                    title={AttrValue::Static("运行时内存剖析")}
                    description={Some(
                        (*memory_overview)
                            .as_ref()
                            .map(|ov| AttrValue::from(format!(
                                "快照时间 {} · 进程运行 {}s · Top {}",
                                format_ms(ov.generated_at_ms),
                                ov.process_uptime_secs,
                                memory_top.trim()
                            )))
                            .unwrap_or_else(|| AttrValue::Static(
                                "后端进程堆分配采样、MiMalloc 分配器统计以及函数/模块/调用栈热点排行。"
                            ))
                    )}
                >
                    <input
                        type="number"
                        value={(*memory_top).clone()}
                        oninput={on_memory_top_change}
                        min="1"
                        placeholder="Top N"
                        aria-label="热点排行条数 Top N"
                        class="adm-input adm-input--mono"
                        style="width: 92px; height: 34px;"
                    />
                    <button type="button" class="adm-btn adm-btn--secondary" onclick={on_memory_refresh} disabled={memory_busy}>
                        <i class={classes!("fas", "fa-rotate-right", memory_busy.then_some("fa-spin"))} aria-hidden="true"></i>
                        <span>{ "刷新" }</span>
                    </button>
                    <button type="button" class="adm-btn adm-btn--secondary"
                        onclick={on_memory_reset} disabled={*memory_action_loading}>
                        { if *memory_action_loading { "执行中…" } else { "重置统计" } }
                    </button>
                    <button type="button" class="adm-btn adm-btn--primary"
                        onclick={on_memory_save_config}
                        disabled={*memory_action_loading || memory_config.is_none()}>
                        { if *memory_action_loading { "保存中…" } else { "保存采样配置" } }
                    </button>
                </AdminPageHeader>
                { error_banner.clone().unwrap_or_default() }

                if let Some(overview) = (*memory_overview).clone() {
                    <>
                        <section class="adm-kpis" aria-label="进程与分配器内存指标">
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "活跃堆估算 (Live Heap)" }</div>
                                <div class="adm-kpi__value">{ format_bytes(overview.total_live_bytes_estimate) }</div>
                                <div class="adm-kpi__meta">
                                    { format!("追踪分配 {} · 独立栈 {}", overview.tracked_allocations, overview.distinct_stacks) }
                                </div>
                            </div>
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "累计分配估算" }</div>
                                <div class="adm-kpi__value">{ format_bytes(overview.total_alloc_bytes_estimate) }</div>
                                <div class="adm-kpi__meta">
                                    { format!("alloc={} free={} drop={}", overview.sampled_alloc_events, overview.sampled_dealloc_events, overview.dropped_allocations) }
                                </div>
                            </div>
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "进程常驻内存 (RSS)" }</div>
                                <div class="adm-kpi__value">{ format_bytes(overview.process_rss_bytes) }</div>
                                <div class="adm-kpi__meta">
                                    { format!("虚拟内存 {}", format_bytes(overview.process_virtual_bytes)) }
                                </div>
                            </div>
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "MiMalloc RSS" }</div>
                                <div class="adm-kpi__value">{ format_bytes(overview.mimalloc.current_rss_bytes) }</div>
                                <div class="adm-kpi__meta">
                                    { format!("峰值 {} · 缺页 {}", format_bytes(overview.mimalloc.peak_rss_bytes), overview.mimalloc.page_faults) }
                                </div>
                            </div>
                            <div class="adm-kpi">
                                <div class="adm-kpi__label">{ "MiMalloc Commit" }</div>
                                <div class="adm-kpi__value">{ format_bytes(overview.mimalloc.current_commit_bytes) }</div>
                                <div class="adm-kpi__meta">
                                    { format!("峰值 {} · {} ms", format_bytes(overview.mimalloc.peak_commit_bytes), overview.mimalloc.elapsed_millis) }
                                </div>
                            </div>
                        </section>
                    </>
                } else if memory_busy {
                    { kpi_skeleton(5) }
                }

                if let Some(config) = (*memory_config).clone() {
                    <section class="adm-card">
                        <div class="adm-card__header">
                            <div>
                                <h2 class="adm-card__title">{ "采样器运行时配置 (Profiler Config)" }</h2>
                                <p class="adm-card__sub">
                                    { format!(
                                        "修改后点击右上角「保存采样配置」生效 · stack_skip={} · max_stack_depth={}",
                                        config.stack_skip, config.max_stack_depth
                                    ) }
                                </p>
                            </div>
                            <label class="adm-check">
                                <input
                                    type="checkbox"
                                    checked={config.enabled}
                                    onchange={on_memory_enabled_change}
                                />
                                <span>{ "启用分配采样 (enabled)" }</span>
                            </label>
                        </div>
                        <div class="adm-card__body">
                            <div class="adm-form-grid">
                                <label class="adm-field">
                                    <span class="adm-label">{ "采样步长 (sample_rate)" }</span>
                                    <input
                                        type="number"
                                        min="1"
                                        value={config.sample_rate.to_string()}
                                        oninput={on_memory_sample_rate_change}
                                        class="adm-input adm-input--mono"
                                    />
                                </label>
                                <label class="adm-field">
                                    <span class="adm-label">{ "最小采样字节 (min_alloc_bytes)" }</span>
                                    <input
                                        type="number"
                                        min="1"
                                        value={config.min_alloc_bytes.to_string()}
                                        oninput={on_memory_min_alloc_change}
                                        class="adm-input adm-input--mono"
                                    />
                                </label>
                                <label class="adm-field">
                                    <span class="adm-label">{ "最大追踪分配数 (max_tracked_allocations)" }</span>
                                    <input
                                        type="number"
                                        min="1"
                                        value={config.max_tracked_allocations.to_string()}
                                        oninput={on_memory_max_tracked_change}
                                        class="adm-input adm-input--mono"
                                    />
                                </label>
                            </div>
                        </div>
                    </section>
                }

                <div class="adm-grid-3">
                    <article class="admin-dist-card">
                        <header class="admin-dist-card__header">
                            <span>{ "热点函数 (Top Functions)" }</span>
                            if let Some(report) = (*memory_functions).as_ref() {
                                <span class="admin-dist-card__count">{ report.entries.len() }</span>
                            }
                        </header>
                        if let Some(report) = (*memory_functions).as_ref() {
                            if report.entries.is_empty() {
                                <div class="adm-card__body adm-faint" style="font-size: 12.5px;">{ "暂无函数采样数据" }</div>
                            } else {
                                { memory_function_list(&report.entries) }
                            }
                        } else if memory_busy {
                            { table_skeleton(4) }
                        } else {
                            <div class="adm-card__body adm-faint" style="font-size: 12.5px;">{ "数据不可用" }</div>
                        }
                    </article>

                    <article class="admin-dist-card">
                        <header class="admin-dist-card__header">
                            <span>{ "热点模块 (Top Modules)" }</span>
                            if let Some(report) = (*memory_modules).as_ref() {
                                <span class="admin-dist-card__count">{ report.entries.len() }</span>
                            }
                        </header>
                        if let Some(report) = (*memory_modules).as_ref() {
                            if report.entries.is_empty() {
                                <div class="adm-card__body adm-faint" style="font-size: 12.5px;">{ "暂无模块采样数据" }</div>
                            } else {
                                { memory_module_list(&report.entries) }
                            }
                        } else if memory_busy {
                            { table_skeleton(4) }
                        } else {
                            <div class="adm-card__body adm-faint" style="font-size: 12.5px;">{ "数据不可用" }</div>
                        }
                    </article>

                    <article class="admin-dist-card">
                        <header class="admin-dist-card__header">
                            <span>{ "热点调用栈 (Top Stacks)" }</span>
                            if let Some(report) = (*memory_stacks).as_ref() {
                                <span class="admin-dist-card__count">{ report.entries.len() }</span>
                            }
                        </header>
                        if let Some(report) = (*memory_stacks).as_ref() {
                            if report.entries.is_empty() {
                                <div class="adm-card__body adm-faint" style="font-size: 12.5px;">{ "暂无调用栈采样数据" }</div>
                            } else {
                                { memory_stack_list(&report.entries) }
                            }
                        } else if memory_busy {
                            { table_skeleton(4) }
                        } else {
                            <div class="adm-card__body adm-faint" style="font-size: 12.5px;">{ "数据不可用" }</div>
                        }
                    </article>
                </div>
            </div>
        },

        AdminNav::SiteConfig | AdminNav::LocalMedia => {
            let configs_ready = view_config.is_some()
                && comment_config.is_some()
                && music_config.is_some()
                && behavior_config.is_some()
                && compaction_config.is_some();

            html! {
                <div class="adm-stack adm-enter">
                    <AdminPageHeader
                        eyebrow={Some(AttrValue::Static("站点配置"))}
                        title={AttrValue::Static("全局运行时配置与维护")}
                        description={Some(AttrValue::Static(
                            "管理站点浏览统计、评论与音乐限流、行为日志落盘缓冲以及 LanceDB 后台自动压缩与数据清理。"
                        ))}
                    >
                        { refresh_btn(core_busy) }
                        <button type="button" class="adm-btn adm-btn--primary"
                            onclick={on_save_configs}
                            disabled={*saving || !configs_ready}>
                            <i class="fas fa-floppy-disk" aria-hidden="true"></i>
                            <span>{ if *saving { "保存中…" } else { "保存全部配置" } }</span>
                        </button>
                    </AdminPageHeader>
                    { error_banner.clone().unwrap_or_default() }

                    <div class="adm-grid-2">
                        <section class="adm-card">
                            <div class="adm-card__header">
                                <div>
                                    <h2 class="adm-card__title">{ "浏览统计 (View Analytics)" }</h2>
                                    <p class="adm-card__sub">{ "控制阅读计数去重时间窗与趋势图跨度上限。" }</p>
                                </div>
                            </div>
                            <div class="adm-card__body">
                                if let Some(cfg) = (*view_config).clone() {
                                    <div class="adm-stack-sm">
                                        <label class="adm-field">
                                            <span class="adm-label">{ "去重时间窗秒数 (dedupe_window_seconds)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.dedupe_window_seconds.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let view_config = view_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<u64>() {
                                                                let mut next = (*view_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.dedupe_window_seconds = v;
                                                                }
                                                                view_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "趋势默认天数 (trend_default_days)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.trend_default_days.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let view_config = view_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<usize>() {
                                                                let mut next = (*view_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.trend_default_days = v;
                                                                }
                                                                view_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "趋势最大天数 (trend_max_days)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.trend_max_days.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let view_config = view_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<usize>() {
                                                                let mut next = (*view_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.trend_max_days = v;
                                                                }
                                                                view_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                    </div>
                                } else if core_busy {
                                    { table_skeleton(3) }
                                } else {
                                    <p class="adm-hint">{ "配置暂不可用" }</p>
                                }
                            </div>
                        </section>

                        <section class="adm-card">
                            <div class="adm-card__header">
                                <div>
                                    <h2 class="adm-card__title">{ "评论运行时 (Comment Runtime)" }</h2>
                                    <p class="adm-card__sub">{ "控制评论提交限流、默认分页大小与清理保留天数。" }</p>
                                </div>
                            </div>
                            <div class="adm-card__body">
                                if let Some(cfg) = (*comment_config).clone() {
                                    <div class="adm-stack-sm">
                                        <label class="adm-field">
                                            <span class="adm-label">{ "提交限流秒数 (submit_rate_limit_seconds)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.submit_rate_limit_seconds.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let comment_config = comment_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<u64>() {
                                                                let mut next = (*comment_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.submit_rate_limit_seconds = v;
                                                                }
                                                                comment_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "列表默认条数 (list_default_limit)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.list_default_limit.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let comment_config = comment_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<usize>() {
                                                                let mut next = (*comment_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.list_default_limit = v;
                                                                }
                                                                comment_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "默认清理保留天数 (cleanup_retention_days)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.cleanup_retention_days.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let comment_config = comment_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<i64>() {
                                                                let mut next = (*comment_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.cleanup_retention_days = v;
                                                                }
                                                                comment_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                    </div>
                                } else if core_busy {
                                    { table_skeleton(3) }
                                } else {
                                    <p class="adm-hint">{ "配置暂不可用" }</p>
                                }
                            </div>
                        </section>

                        <section class="adm-card">
                            <div class="adm-card__header">
                                <div>
                                    <h2 class="adm-card__title">{ "音乐运行时 (Music Runtime)" }</h2>
                                    <p class="adm-card__sub">{ "控制曲目播放去重窗口、乐评提交频率与默认列表上限。" }</p>
                                </div>
                            </div>
                            <div class="adm-card__body">
                                if let Some(cfg) = (*music_config).clone() {
                                    <div class="adm-stack-sm">
                                        <label class="adm-field">
                                            <span class="adm-label">{ "播放去重秒数 (play_dedupe_window_seconds)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.play_dedupe_window_seconds.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let music_config = music_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<u64>() {
                                                                let mut next = (*music_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.play_dedupe_window_seconds = v;
                                                                }
                                                                music_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "乐评限流秒数 (comment_rate_limit_seconds)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.comment_rate_limit_seconds.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let music_config = music_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<u64>() {
                                                                let mut next = (*music_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.comment_rate_limit_seconds = v;
                                                                }
                                                                music_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "列表默认条数 (list_default_limit)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.list_default_limit.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let music_config = music_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<usize>() {
                                                                let mut next = (*music_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.list_default_limit = v;
                                                                }
                                                                music_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                    </div>
                                } else if core_busy {
                                    { table_skeleton(3) }
                                } else {
                                    <p class="adm-hint">{ "配置暂不可用" }</p>
                                }
                            </div>
                        </section>

                        <section class="adm-card">
                            <div class="adm-card__header">
                                <div>
                                    <h2 class="adm-card__title">{ "LanceDB 自动整理 (Storage Maintenance)" }</h2>
                                    <p class="adm-card__sub">{ "控制后台定时 compact / prune 调度器与并发工作线程。" }</p>
                                </div>
                                if let Some(cfg) = (*compaction_config).clone() {
                                    <label class="adm-check">
                                        <input
                                            type="checkbox"
                                            checked={cfg.enabled}
                                            oninput={{
                                                let compaction_config = compaction_config.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                        let mut next = (*compaction_config).clone();
                                                        if let Some(cfg) = next.as_mut() {
                                                            cfg.enabled = target.checked();
                                                        }
                                                        compaction_config.set(next);
                                                    }
                                                })
                                            }}
                                        />
                                        <span>{ "启用 (enabled)" }</span>
                                    </label>
                                }
                            </div>
                            <div class="adm-card__body">
                                if let Some(cfg) = (*compaction_config).clone() {
                                    <div class="adm-form-grid">
                                        <label class="adm-field">
                                            <span class="adm-label">{ "扫描间隔秒数 (scan_interval_seconds)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.scan_interval_seconds.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let compaction_config = compaction_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<u64>() {
                                                                let mut next = (*compaction_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.scan_interval_seconds = v;
                                                                }
                                                                compaction_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "碎片数阈值 (fragment_threshold)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.fragment_threshold.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let compaction_config = compaction_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<usize>() {
                                                                let mut next = (*compaction_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.fragment_threshold = v;
                                                                }
                                                                compaction_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "历史版本保留小时数 (prune_older_than_hours)" }</span>
                                            <input
                                                type="number"
                                                value={cfg.prune_older_than_hours.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let compaction_config = compaction_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<i64>() {
                                                                let mut next = (*compaction_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.prune_older_than_hours = v;
                                                                }
                                                                compaction_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                        <label class="adm-field">
                                            <span class="adm-label">{ "工作线程数 (worker_count)" }</span>
                                            <input
                                                type="number"
                                                min="1"
                                                value={cfg.worker_count.to_string()}
                                                class="adm-input adm-input--mono"
                                                oninput={{
                                                    let compaction_config = compaction_config.clone();
                                                    Callback::from(move |event: InputEvent| {
                                                        if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                            if let Ok(v) = target.value().parse::<usize>() {
                                                                let mut next = (*compaction_config).clone();
                                                                if let Some(cfg) = next.as_mut() {
                                                                    cfg.worker_count = v.max(1);
                                                                }
                                                                compaction_config.set(next);
                                                            }
                                                        }
                                                    })
                                                }}
                                            />
                                        </label>
                                    </div>
                                } else if core_busy {
                                    { table_skeleton(3) }
                                } else {
                                    <p class="adm-hint">{ "配置暂不可用" }</p>
                                }
                            </div>
                        </section>
                    </div>

                    <section class="adm-card">
                        <div class="adm-card__header">
                            <div>
                                <h2 class="adm-card__title">{ "API 行为采集与落盘缓冲 (API Behavior)" }</h2>
                                <p class="adm-card__sub">{ "控制行为事件保留周期、查询窗口上限以及内存缓冲批量刷盘参数。" }</p>
                            </div>
                        </div>
                        <div class="adm-card__body">
                            if let Some(cfg) = (*behavior_config).clone() {
                                <div class="adm-form-grid">
                                    <label class="adm-field">
                                        <span class="adm-label">{ "日志保留天数 (retention_days)" }</span>
                                        <input
                                            type="number"
                                            value={cfg.retention_days.to_string()}
                                            class="adm-input adm-input--mono"
                                            oninput={{
                                                let behavior_config = behavior_config.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                        if let Ok(v) = target.value().parse::<i64>() {
                                                            let mut next = (*behavior_config).clone();
                                                            if let Some(cfg) = next.as_mut() {
                                                                cfg.retention_days = v;
                                                            }
                                                            behavior_config.set(next);
                                                        }
                                                    }
                                                })
                                            }}
                                        />
                                    </label>
                                    <label class="adm-field">
                                        <span class="adm-label">{ "查询默认天数 (default_days)" }</span>
                                        <input
                                            type="number"
                                            value={cfg.default_days.to_string()}
                                            class="adm-input adm-input--mono"
                                            oninput={{
                                                let behavior_config = behavior_config.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                        if let Ok(v) = target.value().parse::<usize>() {
                                                            let mut next = (*behavior_config).clone();
                                                            if let Some(cfg) = next.as_mut() {
                                                                cfg.default_days = v;
                                                            }
                                                            behavior_config.set(next);
                                                        }
                                                    }
                                                })
                                            }}
                                        />
                                    </label>
                                    <label class="adm-field">
                                        <span class="adm-label">{ "查询最大天数 (max_days)" }</span>
                                        <input
                                            type="number"
                                            value={cfg.max_days.to_string()}
                                            class="adm-input adm-input--mono"
                                            oninput={{
                                                let behavior_config = behavior_config.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                        if let Ok(v) = target.value().parse::<usize>() {
                                                            let mut next = (*behavior_config).clone();
                                                            if let Some(cfg) = next.as_mut() {
                                                                cfg.max_days = v;
                                                            }
                                                            behavior_config.set(next);
                                                        }
                                                    }
                                                })
                                            }}
                                        />
                                    </label>
                                    <label class="adm-field">
                                        <span class="adm-label">{ "刷盘批次条数 (flush_batch_size)" }</span>
                                        <input
                                            type="number"
                                            min="1"
                                            value={cfg.flush_batch_size.to_string()}
                                            class="adm-input adm-input--mono"
                                            oninput={{
                                                let behavior_config = behavior_config.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                        if let Ok(v) = target.value().parse::<usize>() {
                                                            let mut next = (*behavior_config).clone();
                                                            if let Some(cfg) = next.as_mut() {
                                                                cfg.flush_batch_size = v;
                                                            }
                                                            behavior_config.set(next);
                                                        }
                                                    }
                                                })
                                            }}
                                        />
                                    </label>
                                    <label class="adm-field">
                                        <span class="adm-label">{ "刷盘间隔秒数 (flush_interval_seconds)" }</span>
                                        <input
                                            type="number"
                                            min="1"
                                            value={cfg.flush_interval_seconds.to_string()}
                                            class="adm-input adm-input--mono"
                                            oninput={{
                                                let behavior_config = behavior_config.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                        if let Ok(v) = target.value().parse::<u64>() {
                                                            let mut next = (*behavior_config).clone();
                                                            if let Some(cfg) = next.as_mut() {
                                                                cfg.flush_interval_seconds = v;
                                                            }
                                                            behavior_config.set(next);
                                                        }
                                                    }
                                                })
                                            }}
                                        />
                                    </label>
                                    <label class="adm-field">
                                        <span class="adm-label">{ "缓冲字节上限 (flush_max_buffer_bytes)" }</span>
                                        <input
                                            type="number"
                                            min="1024"
                                            value={cfg.flush_max_buffer_bytes.to_string()}
                                            class="adm-input adm-input--mono"
                                            oninput={{
                                                let behavior_config = behavior_config.clone();
                                                Callback::from(move |event: InputEvent| {
                                                    if let Some(target) = event.target_dyn_into::<HtmlInputElement>() {
                                                        if let Ok(v) = target.value().parse::<usize>() {
                                                            let mut next = (*behavior_config).clone();
                                                            if let Some(cfg) = next.as_mut() {
                                                                cfg.flush_max_buffer_bytes = v;
                                                            }
                                                            behavior_config.set(next);
                                                        }
                                                    }
                                                })
                                            }}
                                        />
                                    </label>
                                </div>
                            } else if core_busy {
                                { table_skeleton(2) }
                            } else {
                                <p class="adm-hint">{ "配置暂不可用" }</p>
                            }
                        </div>
                    </section>

                    <section class="adm-card">
                        <div class="adm-card__header">
                            <div>
                                <h2 class="adm-card__title">{ "数据清理维护 (Maintenance Cleanup)" }</h2>
                                <p class="adm-card__sub">{ "不可逆删除操作均需二次弹窗确认。" }</p>
                            </div>
                        </div>
                        <div class="adm-card__body">
                            <div class="adm-grid-2">
                                <div class="adm-field">
                                    <span class="adm-label">{ "清理失败评论任务（保留最近 N 天内失败项）" }</span>
                                    <div class="adm-toolbar">
                                        <input
                                            type="number"
                                            value={(*cleanup_days).clone()}
                                            oninput={on_cleanup_days_change}
                                            aria-label="失败评论任务保留天数"
                                            class="adm-input adm-input--mono"
                                            style="width: 140px;"
                                        />
                                        <button type="button" class="adm-btn adm-btn--danger"
                                            onclick={{
                                                let pending_cleanup_failed = pending_cleanup_failed.clone();
                                                Callback::from(move |_| pending_cleanup_failed.set(true))
                                            }}>
                                            { "清理失败任务" }
                                        </button>
                                    </div>
                                    <p class="adm-hint">{ "仅删除状态为 failed 且早于指定保留天数的评论任务。" }</p>
                                </div>

                                <div class="adm-field">
                                    <span class="adm-label">{ "清理过期 API 行为日志" }</span>
                                    <div class="adm-toolbar">
                                        <button type="button" class="adm-btn adm-btn--danger"
                                            disabled={behavior_config.is_none()}
                                            onclick={{
                                                let pending_cleanup_logs = pending_cleanup_logs.clone();
                                                Callback::from(move |_| pending_cleanup_logs.set(true))
                                            }}>
                                            { "按保留期清理行为日志" }
                                        </button>
                                    </div>
                                    <p class="adm-hint">
                                        { (*behavior_config)
                                            .as_ref()
                                            .map(|c| format!("按当前 retention_days = {} 天清理过期事件记录。", c.retention_days))
                                            .unwrap_or_else(|| "等待加载 API 行为配置…".to_string()) }
                                    </p>
                                </div>
                            </div>
                        </div>
                    </section>
                </div>
            }
        },
    };

    html! {
        <>
            <ConfirmModal
                open={pending_delete_published.is_some()}
                title="删除已发布评论"
                message="确认删除这条已发布评论？此操作不可撤销。"
                danger=true
                on_confirm={confirm_delete_published}
                on_cancel={{
                    let pending_delete_published = pending_delete_published.clone();
                    Callback::from(move |_| pending_delete_published.set(None))
                }}
            />
            <ConfirmModal
                open={pending_delete_task.is_some()}
                title="删除评论任务"
                message="确认删除这条评论任务记录？此操作不可撤销。"
                danger=true
                on_confirm={{
                    let pending_delete_task = pending_delete_task.clone();
                    let execute_task_action = execute_task_action.clone();
                    Callback::from(move |_: ()| {
                        if let Some(task_id) = (*pending_delete_task).clone() {
                            pending_delete_task.set(None);
                            execute_task_action.emit((task_id, "delete".to_string()));
                        }
                    })
                }}
                on_cancel={{
                    let pending_delete_task = pending_delete_task.clone();
                    Callback::from(move |_| pending_delete_task.set(None))
                }}
            />
            <ConfirmModal
                open={pending_delete_wish.is_some()}
                title="删除点歌心愿"
                message="确认删除这条点歌心愿？此操作不可撤销。"
                danger=true
                on_confirm={{
                    let pending_delete_wish = pending_delete_wish.clone();
                    let execute_music_wish_action = execute_music_wish_action.clone();
                    Callback::from(move |_: ()| {
                        if let Some(wid) = (*pending_delete_wish).clone() {
                            pending_delete_wish.set(None);
                            execute_music_wish_action.emit((wid, WishAction::Delete));
                        }
                    })
                }}
                on_cancel={{
                    let pending_delete_wish = pending_delete_wish.clone();
                    Callback::from(move |_| pending_delete_wish.set(None))
                }}
            />
            <ConfirmModal
                open={pending_delete_request.is_some()}
                title="删除文章入库请求"
                message="确认删除这条文章入库请求？此操作不可撤销。"
                danger=true
                on_confirm={{
                    let pending_delete_request = pending_delete_request.clone();
                    let execute_article_request_action = execute_article_request_action.clone();
                    Callback::from(move |_: ()| {
                        if let Some(rid) = (*pending_delete_request).clone() {
                            pending_delete_request.set(None);
                            execute_article_request_action.emit((rid, ArticleRequestAction::Delete));
                        }
                    })
                }}
                on_cancel={{
                    let pending_delete_request = pending_delete_request.clone();
                    Callback::from(move |_| pending_delete_request.set(None))
                }}
            />
            <ConfirmModal
                open={*pending_cleanup_failed}
                title="清理失败评论任务"
                message="确认按保留天数删除失败的评论任务？此操作不可撤销。"
                danger=true
                on_confirm={{
                    let pending_cleanup_failed = pending_cleanup_failed.clone();
                    let on_cleanup = on_cleanup.clone();
                    Callback::from(move |_: ()| {
                        pending_cleanup_failed.set(false);
                        on_cleanup.emit(());
                    })
                }}
                on_cancel={{
                    let pending_cleanup_failed = pending_cleanup_failed.clone();
                    Callback::from(move |_| pending_cleanup_failed.set(false))
                }}
            />
            <ConfirmModal
                open={*pending_cleanup_logs}
                title="清理过期行为日志"
                message="确认按保留天数删除旧的 API 行为日志？此操作不可撤销。"
                danger=true
                on_confirm={{
                    let pending_cleanup_logs = pending_cleanup_logs.clone();
                    let on_behavior_cleanup = on_behavior_cleanup.clone();
                    Callback::from(move |_: ()| {
                        pending_cleanup_logs.set(false);
                        on_behavior_cleanup.emit(());
                    })
                }}
                on_cancel={{
                    let pending_cleanup_logs = pending_cleanup_logs.clone();
                    Callback::from(move |_| pending_cleanup_logs.set(false))
                }}
            />

            <AdminShell active={*active_section} badges={sidebar_badges}>
                { section_view }
            </AdminShell>
        </>
    }
}
