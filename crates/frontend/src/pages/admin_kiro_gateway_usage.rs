//! Standalone Kiro gateway usage page (`/admin/kiro-gateway/usage`).
//!
//! Replaces the old 5-row preview tab with a real, server-paginated event
//! list in the `.admin-shell` design system, plus the full event detail
//! viewer.

use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_kiro_gateway::{
    anthropic_routing_badge, anthropic_routing_summary, format_optional_stream_bytes,
    format_usage_stream_summary, usage_stream_state_label,
};
use crate::{
    api::{
        fetch_admin_kiro_usage_event_detail, fetch_admin_kiro_usage_events,
        AdminLlmGatewayUsageEventDetailView, AdminLlmGatewayUsageEventView,
        AdminLlmGatewayUsageEventsQuery, AdminUsageTotalsView,
    },
    pages::llm_access_shared::{
        first_token_latency_color, format_latency_ms, format_ms, format_number_u64,
        total_latency_color, usage_error_summary,
    },
    router::Route,
};

const USAGE_PAGE_SIZE: usize = 20;

fn status_badge_classes(status_code: i32) -> Classes {
    if (200..300).contains(&status_code) {
        classes!("badge", "ok")
    } else if status_code >= 500 {
        classes!("badge", "failed")
    } else {
        classes!("badge", "warn")
    }
}

fn detail_kv(label: &str, value: String) -> Html {
    html! {
        <div class={classes!("rounded-[var(--r-field)]", "border", "border-[var(--border)]", "bg-[var(--card-2)]", "px-3", "py-2", "mono")}>
            <div class={classes!("uppercase", "tracking-[0.08em]", "text-[var(--muted-foreground)]")}>{ label }</div>
            <div class={classes!("mt-1", "truncate")} title={value.clone()}>{ value }</div>
        </div>
    }
}

fn detail_pre(label: &str, value: String) -> Html {
    html! {
        <div class={classes!("rounded-[var(--r-field)]", "border", "border-[var(--border)]", "bg-[var(--card-2)]", "px-3", "py-2")}>
            <div class={classes!("mb-2", "mono", "uppercase", "tracking-[0.08em]", "text-[var(--muted-foreground)]")}>{ label }</div>
            <pre class={classes!("m-0", "max-h-64", "overflow-auto", "mono", "leading-5")}>{ value }</pre>
        </div>
    }
}

#[function_component(AdminKiroGatewayUsagePage)]
pub fn admin_kiro_gateway_usage_page() -> Html {
    let events = use_state(Vec::<AdminLlmGatewayUsageEventView>::new);
    let totals = use_state(AdminUsageTotalsView::default);
    let total = use_state(|| 0usize);
    let retention_days = use_state(|| 7u64);
    let page = use_state(|| 1usize);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0u32);
    let selected_detail = use_state(|| None::<AdminLlmGatewayUsageEventDetailView>);
    let detail_loading = use_state(|| false);

    {
        let events = events.clone();
        let totals = totals.clone();
        let total = total.clone();
        let retention_days = retention_days.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with((*page, *refresh_tick), move |(page, _)| {
            let events = events.clone();
            let totals = totals.clone();
            let total = total.clone();
            let retention_days = retention_days.clone();
            let loading = loading.clone();
            let error = error.clone();
            let offset = page.saturating_sub(1) * USAGE_PAGE_SIZE;
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match fetch_admin_kiro_usage_events(&AdminLlmGatewayUsageEventsQuery {
                    key_id: None,
                    start_ms: None,
                    end_ms: None,
                    source: Some("all".to_string()),
                    model: None,
                    account_name: None,
                    endpoint: None,
                    status_code: None,
                    status_kind: None,
                    limit: Some(USAGE_PAGE_SIZE),
                    offset: Some(offset),
                })
                .await
                {
                    Ok(response) => {
                        retention_days.set(response.retention_days);
                        totals.set(response.totals);
                        total.set(response.total);
                        events.set(response.events);
                        error.set(None);
                    },
                    Err(err) => error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let open_detail = {
        let selected_detail = selected_detail.clone();
        let detail_loading = detail_loading.clone();
        let error = error.clone();
        Callback::from(move |event_id: String| {
            let selected_detail = selected_detail.clone();
            let detail_loading = detail_loading.clone();
            let error = error.clone();
            selected_detail.set(None);
            detail_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_kiro_usage_event_detail(&event_id).await {
                    Ok(detail) => selected_detail.set(Some(detail)),
                    Err(err) => error.set(Some(err)),
                }
                detail_loading.set(false);
            });
        })
    };

    let close_detail = {
        let selected_detail = selected_detail.clone();
        Callback::from(move |_| selected_detail.set(None))
    };

    let on_refresh = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_| refresh_tick.set(refresh_tick.wrapping_add(1)))
    };

    let total_pages = (*total).div_ceil(USAGE_PAGE_SIZE).max(1);
    let current_page = (*page).clamp(1, total_pages);
    let on_prev = {
        let page = page.clone();
        Callback::from(move |_| page.set((*page).saturating_sub(1).max(1)))
    };
    let on_next = {
        let page = page.clone();
        Callback::from(move |_| page.set((*page).saturating_add(1)))
    };

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "Kiro Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Usage" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminKiroGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                        <Link<Route> to={Route::AdminLlmGateway} classes={classes!("linkbtn")}>{ "完整记录 (LLM 面板)" }</Link<Route>>
                        <button type="button" class={classes!("primary")} disabled={*loading} onclick={on_refresh}>
                            { if *loading { "Loading..." } else { "Refresh" } }
                        </button>
                    </div>
                </header>

                if let Some(err) = (*error).clone() {
                    <div class={classes!("errorline", "text-sm")}>{ err }</div>
                }

                <section class={classes!("panel")}>
                    <div class={classes!("stat-strip")}>
                        <div class={classes!("stat")}>
                            <span>{ "Requests" }</span>
                            <b>{ format_number_u64(*total as u64) }</b>
                        </div>
                        <div class={classes!("stat")}>
                            <span>{ "Billable" }</span>
                            <b>{ format_number_u64(totals.billable_tokens) }</b>
                        </div>
                        <div class={classes!("stat")}>
                            <span>{ "Output" }</span>
                            <b>{ format_number_u64(totals.output_tokens) }</b>
                        </div>
                        <div class={classes!("stat")}>
                            <span>{ "Cached In" }</span>
                            <b>{ format_number_u64(totals.input_cached_tokens) }</b>
                        </div>
                        <div class={classes!("stat")}>
                            <span>{ "Retention" }</span>
                            <b>{ format!("{}d", *retention_days) }</b>
                        </div>
                    </div>
                </section>

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <h2>{ "Events" }</h2>
                        <span class={classes!("badge")}>{ format!("{} / {} 页", current_page, total_pages) }</span>
                    </div>
                    if *loading && (*events).is_empty() {
                        <div class={classes!("skeleton", "px-4", "py-4")}>
                            <i></i><i></i><i></i><i></i><i></i><i></i>
                        </div>
                    } else if (*events).is_empty() {
                        <div class={classes!("empty")}>
                            <span>{ "暂无 usage 记录" }</span>
                        </div>
                    } else {
                        <div class={classes!("overflow-x-auto")}>
                            <table class={classes!("min-w-[56rem]", "w-full", "text-sm")}>
                                <thead>
                                    <tr class={classes!("text-left", "text-xs", "text-[var(--muted-foreground)]")}>
                                        <th class={classes!("py-2", "pl-4", "pr-3", "font-medium")}>{ "时间" }</th>
                                        <th class={classes!("py-2", "pr-3", "font-medium")}>{ "Key" }</th>
                                        <th class={classes!("py-2", "pr-3", "font-medium")}>{ "模型" }</th>
                                        <th class={classes!("py-2", "pr-3", "font-medium")}>{ "状态" }</th>
                                        <th class={classes!("py-2", "pr-3", "font-medium")}>{ "耗时" }</th>
                                        <th class={classes!("py-2", "pr-3", "font-medium")}>{ "Tokens" }</th>
                                        <th class={classes!("py-2", "pr-3", "font-medium")}>{ "Credit" }</th>
                                        <th class={classes!("py-2", "pr-4")}></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    { for (*events).iter().map(|event| {
                                        let credit_text = event.credit_usage
                                            .map(|credit| format!("{credit:.4}"))
                                            .unwrap_or_else(|| "-".to_string());
                                        let stream_summary = format_usage_stream_summary(
                                            event.stream_completed_cleanly,
                                            event.downstream_disconnect,
                                            event.final_event_type.as_deref(),
                                            event.bytes_streamed,
                                        );
                                        let error_class_label = event
                                            .error_class
                                            .as_deref()
                                            .map(str::trim)
                                            .filter(|value| !value.is_empty())
                                            .map(str::to_string);
                                        let status_error_summary = usage_error_summary(
                                            event.status_code,
                                            event.error_message.as_deref(),
                                            event.error_class.as_deref(),
                                            event.session_blocked,
                                        );
                                        let anthropic_badge =
                                            anthropic_routing_badge(event.routing_diagnostics_json.as_deref());
                                        let latency_color = total_latency_color(event.latency_ms);
                                        let first_token = event.first_sse_write_ms.map(|first_ms| {
                                            let first_ms = first_ms.max(0);
                                            (first_ms, first_token_latency_color(first_ms))
                                        });
                                        let tokens_per_second = (event.latency_ms > 0
                                            && event.output_tokens > 0)
                                            .then(|| {
                                                event.output_tokens as f64
                                                    / (event.latency_ms as f64 / 1000.0)
                                            });
                                        let event_id = event.id.clone();
                                        let on_detail = {
                                            let open_detail = open_detail.clone();
                                            let event_id = event_id.clone();
                                            Callback::from(move |_| open_detail.emit(event_id.clone()))
                                        };
                                        html! {
                                            <tr class={classes!("border-t", "border-[var(--border)]", "align-top")}>
                                                <td class={classes!("py-2.5", "pl-4", "pr-3", "whitespace-nowrap")}>
                                                    <div class={classes!("text-xs")}>{ format_ms(event.created_at) }</div>
                                                    <div class={classes!("mt-0.5", "max-w-[9rem]", "truncate", "mono", "text-[10px]", "text-[var(--faint)]")} title={event.id.clone()}>
                                                        { event.id.clone() }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2.5", "pr-3")}>
                                                    <div class={classes!("max-w-[10rem]", "truncate", "text-xs", "font-semibold")} title={event.key_name.clone()}>
                                                        { event.key_name.clone() }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2.5", "pr-3")}>
                                                    <div class={classes!("max-w-[10rem]", "truncate", "mono", "text-xs", "text-[var(--muted-foreground)]")} title={event.model.clone().unwrap_or_default()}>
                                                        { event.model.clone().unwrap_or_else(|| "-".to_string()) }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2.5", "pr-3", "min-w-[12rem]", "max-w-[22rem]")}>
                                                    <div class={classes!("flex", "flex-wrap", "items-center", "gap-1.5")}>
                                                        <span class={status_badge_classes(event.status_code)}>{ event.status_code }</span>
                                                        if let Some(class_label) = error_class_label {
                                                            <span class={classes!("badge", "failed")}>{ class_label }</span>
                                                        }
                                                        if let Some(badge) = anthropic_badge {
                                                            <span class={classes!("badge", "info")}>{ badge }</span>
                                                        }
                                                    </div>
                                                    if let Some(summary) = status_error_summary {
                                                        <div class={classes!("mt-1", "max-w-[22rem]", "truncate", "mono", "text-[11px]", "text-[var(--destructive)]")} title={summary.clone()}>
                                                            { summary }
                                                        </div>
                                                    } else {
                                                        <div class={classes!("mt-1", "max-w-[22rem]", "truncate", "mono", "text-[11px]", "text-[var(--faint)]")} title={stream_summary.clone()}>
                                                            { stream_summary }
                                                        </div>
                                                    }
                                                </td>
                                                <td class={classes!("py-2.5", "pr-3", "whitespace-nowrap")}>
                                                    <span class={classes!("inline-flex", "rounded-full", "border", "px-2", "py-0.5", "text-[11px]", "font-semibold", latency_color.0, latency_color.1, latency_color.2, latency_color.3)}>
                                                        { format_latency_ms(event.latency_ms) }
                                                    </span>
                                                    <div class={classes!("mt-0.5", "flex", "items-center", "gap-1")}>
                                                        if let Some((first_ms, first_color)) = first_token {
                                                            <span class={classes!("inline-flex", "rounded-full", "border", "px-1.5", "py-0.5", "text-[10px]", "font-semibold", first_color.0, first_color.1, first_color.2, first_color.3)}>
                                                                { format!("首字 {}", format_latency_ms(first_ms)) }
                                                            </span>
                                                        } else {
                                                            <span class={classes!("text-[10px]", "text-[var(--faint)]")}>{ "首字 -" }</span>
                                                        }
                                                        if let Some(tps) = tokens_per_second {
                                                            <span class={classes!("text-[10px]", "text-[var(--muted-foreground)]")}>
                                                                { format!("流 · {tps:.0} t/s") }
                                                            </span>
                                                        }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2.5", "pr-3", "whitespace-nowrap", "mono", "text-[11px]")}>
                                                    <div>
                                                        { format!("{} / {}", format_number_u64(event.input_uncached_tokens + event.input_cached_tokens), format_number_u64(event.output_tokens)) }
                                                    </div>
                                                    <div class={classes!("text-[var(--faint)]")}>
                                                        { format!("缓存↓ {}", format_number_u64(event.input_cached_tokens)) }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2.5", "pr-3", "whitespace-nowrap", "mono", "text-xs")}>
                                                    { credit_text }
                                                </td>
                                                <td class={classes!("py-2.5", "pr-4")}>
                                                    <button
                                                        type="button"
                                                        class={classes!("ghost", "!min-h-0", "!px-2", "!py-1", "text-xs")}
                                                        title="查看请求详情"
                                                        aria-label="查看请求详情"
                                                        onclick={on_detail}
                                                    >
                                                        <i class={classes!("fas", "fa-bars-staggered", "text-xs")}></i>
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }) }
                                </tbody>
                            </table>
                        </div>
                        <div class={classes!("pager", "px-4", "pb-3")}>
                            <button type="button" disabled={current_page <= 1 || *loading} onclick={on_prev}>{ "上一页" }</button>
                            <span>
                                {
                                    format!(
                                        "{}-{} / {}",
                                        (current_page - 1) * USAGE_PAGE_SIZE + 1,
                                        ((current_page - 1) * USAGE_PAGE_SIZE + (*events).len()).min(*total),
                                        *total,
                                    )
                                }
                            </span>
                            <button type="button" disabled={current_page >= total_pages || *loading} onclick={on_next}>{ "下一页" }</button>
                        </div>
                    }
                </section>
            </div>

            {
                if *detail_loading {
                    html! {
                        <>
                        <div class={classes!("scrim")}></div>
                        <div class={classes!("fixed", "inset-0", "z-[95]", "flex", "items-center", "justify-center", "px-4", "pointer-events-none")}>
                            <div class={classes!("panel", "px-5", "py-4", "mono")}>{ "加载 usage 详情…" }</div>
                        </div>
                        </>
                    }
                } else if let Some(detail) = (*selected_detail).clone() {
                    let close_detail = close_detail.clone();
                    let close_detail_scrim = close_detail.clone();
                    let anthropic_summary =
                        anthropic_routing_summary(detail.routing_diagnostics_json.as_deref());
                    html! {
                        <>
                        <div class={classes!("scrim")} onclick={close_detail_scrim}></div>
                        <div class={classes!("fixed", "inset-0", "z-[95]", "flex", "items-center", "justify-center", "px-4", "pointer-events-none")}>
                            <div class={classes!("panel", "pointer-events-auto", "max-h-[86vh]", "w-[min(64rem,100%)]")}>
                                <div class={classes!("panel-head")}>
                                    <div class={classes!("min-w-0")}>
                                        <div class={classes!("eyebrow")}>{ "Usage Detail" }</div>
                                        <div class={classes!("truncate", "mono", "font-semibold")} title={detail.id.clone()}>{ detail.id.clone() }</div>
                                    </div>
                                    <button type="button" class={classes!("ghost")} onclick={close_detail}>{ "关闭" }</button>
                                </div>
                                <div class={classes!("panel-body", "max-h-[72vh]", "space-y-3")}>
                                    <div class={classes!("grid", "gap-3", "sm:grid-cols-2", "lg:grid-cols-4")}>
                                        { detail_kv("created", format_ms(detail.created_at)) }
                                        { detail_kv("key", detail.key_name.clone()) }
                                        { detail_kv("account", detail.account_name.clone().unwrap_or_else(|| "-".to_string())) }
                                        { detail_kv("model", detail.model.clone().unwrap_or_else(|| "-".to_string())) }
                                        { detail_kv("status", detail.status_code.to_string()) }
                                        { detail_kv("latency", format!("{} ms", detail.latency_ms.max(0))) }
                                        { detail_kv("stream", usage_stream_state_label(detail.stream_completed_cleanly, detail.downstream_disconnect).to_string()) }
                                        { detail_kv("final event", detail.final_event_type.clone().unwrap_or_else(|| "-".to_string())) }
                                        { detail_kv("stream bytes", format_optional_stream_bytes(detail.bytes_streamed)) }
                                        { detail_kv("input", format_number_u64(detail.input_uncached_tokens)) }
                                        { detail_kv("cached", format_number_u64(detail.input_cached_tokens)) }
                                        { detail_kv("output", format_number_u64(detail.output_tokens)) }
                                        { detail_kv("billable", format_number_u64(detail.billable_tokens)) }
                                        {
                                            if let Some(summary) = anthropic_summary {
                                                detail_kv("routing", summary)
                                            } else {
                                                Html::default()
                                            }
                                        }
                                    </div>
                                    { detail_pre("routing diagnostics", detail.routing_diagnostics_json.clone().unwrap_or_else(|| "-".to_string())) }
                                    { detail_pre("request headers", detail.request_headers_json.clone()) }
                                    { detail_pre("client request", detail.client_request_body_json.clone().unwrap_or_else(|| "-".to_string())) }
                                    { detail_pre("upstream request", detail.upstream_request_body_json.clone().unwrap_or_else(|| "-".to_string())) }
                                    { detail_pre("full request", detail.full_request_json.clone().unwrap_or_else(|| "-".to_string())) }
                                    { detail_pre("response body", detail.response_body.clone().unwrap_or_else(|| "-".to_string())) }
                                </div>
                            </div>
                        </div>
                        </>
                    }
                } else {
                    Html::default()
                }
            }
        </main>
    }
}
