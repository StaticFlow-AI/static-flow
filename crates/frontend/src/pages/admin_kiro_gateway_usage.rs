//! Standalone Kiro gateway usage page (`/admin/kiro-gateway/usage`).
//!
//! Server-paginated event table in the `.admin-shell` design system with a
//! date-range + source/status/key/model/account/endpoint filter bar, a page
//! credit rollup in the stat strip, and a full event detail viewer.

use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use super::{
    admin_kiro_gateway::{
        anthropic_routing_badge, anthropic_routing_summary, format_optional_stream_bytes,
        format_usage_stream_summary, usage_stream_state_label,
    },
    admin_llm_gateway::{
        effective_routing_wait_ms, format_datetime_local_input, format_latency_breakdown,
        parse_datetime_local_input_to_ms, pretty_headers_json, pretty_json_text, LatencyBreakdown,
        USAGE_SOURCE_ALL, USAGE_SOURCE_ARCHIVE, USAGE_SOURCE_HOT, USAGE_STATUS_KIND_ALL,
        USAGE_STATUS_KIND_NON_OK, USAGE_STATUS_KIND_OK,
    },
};
use crate::{
    api::{
        fetch_admin_kiro_keys_page, fetch_admin_kiro_usage_event_detail,
        fetch_admin_kiro_usage_events, AdminLlmGatewayKeyView, AdminLlmGatewayUsageEventDetailView,
        AdminLlmGatewayUsageEventView, AdminLlmGatewayUsageEventsQuery, AdminUsageTotalsView,
    },
    components::date_range_picker::DateRangePicker,
    pages::llm_access_shared::{
        first_token_latency_color, format_latency_ms, format_ms, format_number_u64,
        total_latency_color, usage_error_summary,
    },
    router::Route,
};

const USAGE_PAGE_SIZE: usize = 20;
/// How many kiro keys the filter dropdown loads (kiro key counts are small).
const KEY_OPTION_LIMIT: usize = 200;

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

/// The filter set actually applied to the query; the staged inputs above the
/// table only take effect when 查询 is pressed.
#[derive(Clone, PartialEq)]
struct AppliedFilters {
    start_ms: Option<i64>,
    end_ms: Option<i64>,
    source: String,
    status_kind: String,
    key_id: String,
    model: String,
    account: String,
    endpoint: String,
}

impl Default for AppliedFilters {
    fn default() -> Self {
        Self {
            start_ms: None,
            end_ms: None,
            source: USAGE_SOURCE_HOT.to_string(),
            status_kind: USAGE_STATUS_KIND_ALL.to_string(),
            key_id: String::new(),
            model: String::new(),
            account: String::new(),
            endpoint: String::new(),
        }
    }
}

fn normalized(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
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
    // Staged filter inputs (take effect on 查询).
    let start_input = use_state(String::new);
    let end_input = use_state(String::new);
    let source_input = use_state(|| USAGE_SOURCE_HOT.to_string());
    let status_kind_input = use_state(|| USAGE_STATUS_KIND_ALL.to_string());
    let key_filter_input = use_state(String::new);
    let model_filter_input = use_state(String::new);
    let account_filter_input = use_state(String::new);
    let endpoint_filter_input = use_state(String::new);
    let applied = use_state(AppliedFilters::default);
    let key_options = use_state(Vec::<AdminLlmGatewayKeyView>::new);

    // Key dropdown options load once per refresh; paging never re-fetches them.
    {
        let key_options = key_options.clone();
        let error = error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let key_options = key_options.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_kiro_keys_page(KEY_OPTION_LIMIT, 0).await {
                    Ok(response) => key_options.set(response.keys),
                    Err(err) => error.set(Some(err)),
                }
            });
            || ()
        });
    }

    {
        let events = events.clone();
        let totals = totals.clone();
        let total = total.clone();
        let retention_days = retention_days.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with((*page, *refresh_tick, (*applied).clone()), move |(page, _, applied)| {
            let events = events.clone();
            let totals = totals.clone();
            let total = total.clone();
            let retention_days = retention_days.clone();
            let loading = loading.clone();
            let error = error.clone();
            let offset = page.saturating_sub(1) * USAGE_PAGE_SIZE;
            let query = AdminLlmGatewayUsageEventsQuery {
                key_id: normalized(&applied.key_id),
                start_ms: applied.start_ms,
                end_ms: applied.end_ms,
                source: Some(applied.source.clone()),
                model: normalized(&applied.model),
                account_name: normalized(&applied.account),
                endpoint: normalized(&applied.endpoint),
                status_code: None,
                status_kind: (applied.status_kind.as_str() != USAGE_STATUS_KIND_ALL)
                    .then(|| applied.status_kind.clone()),
                limit: Some(USAGE_PAGE_SIZE),
                offset: Some(offset),
            };
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match fetch_admin_kiro_usage_events(&query).await {
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

    let on_apply_filters = {
        let start_input = start_input.clone();
        let end_input = end_input.clone();
        let source_input = source_input.clone();
        let status_kind_input = status_kind_input.clone();
        let key_filter_input = key_filter_input.clone();
        let model_filter_input = model_filter_input.clone();
        let account_filter_input = account_filter_input.clone();
        let endpoint_filter_input = endpoint_filter_input.clone();
        let applied = applied.clone();
        let page = page.clone();
        Callback::from(move |_| {
            applied.set(AppliedFilters {
                start_ms: parse_datetime_local_input_to_ms(&start_input),
                end_ms: parse_datetime_local_input_to_ms(&end_input),
                source: (*source_input).clone(),
                status_kind: (*status_kind_input).clone(),
                key_id: (*key_filter_input).clone(),
                model: (*model_filter_input).clone(),
                account: (*account_filter_input).clone(),
                endpoint: (*endpoint_filter_input).clone(),
            });
            page.set(1);
        })
    };

    let on_reset_filters = {
        let start_input = start_input.clone();
        let end_input = end_input.clone();
        let source_input = source_input.clone();
        let status_kind_input = status_kind_input.clone();
        let key_filter_input = key_filter_input.clone();
        let model_filter_input = model_filter_input.clone();
        let account_filter_input = account_filter_input.clone();
        let endpoint_filter_input = endpoint_filter_input.clone();
        let applied = applied.clone();
        let page = page.clone();
        Callback::from(move |_| {
            start_input.set(String::new());
            end_input.set(String::new());
            source_input.set(USAGE_SOURCE_HOT.to_string());
            status_kind_input.set(USAGE_STATUS_KIND_ALL.to_string());
            key_filter_input.set(String::new());
            model_filter_input.set(String::new());
            account_filter_input.set(String::new());
            endpoint_filter_input.set(String::new());
            applied.set(AppliedFilters::default());
            page.set(1);
        })
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

    // Server-aggregated credit over the full matched result set.
    let matched_credit_text = if totals.credit_missing_events > 0 {
        format!("{:.4} · 未计入 {} 条", totals.credit_total, totals.credit_missing_events)
    } else {
        format!("{:.4}", totals.credit_total)
    };

    let pager = |on_prev: Callback<MouseEvent>, on_next: Callback<MouseEvent>| {
        html! {
            <div class={classes!("pager", "px-4", "py-2")}>
                <button type="button" disabled={current_page <= 1 || *loading} onclick={on_prev}>{ "上一页" }</button>
                <span>
                    {
                        format!(
                            "{}-{} / {} · 第 {}/{} 页",
                            (current_page - 1) * USAGE_PAGE_SIZE + 1,
                            ((current_page - 1) * USAGE_PAGE_SIZE + (*events).len()).min(*total),
                            *total,
                            current_page,
                            total_pages,
                        )
                    }
                </span>
                <button type="button" disabled={current_page >= total_pages || *loading} onclick={on_next}>{ "下一页" }</button>
            </div>
        }
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
                        <Link<Route> to={Route::AdminLlmGatewayUsage} classes={classes!("linkbtn")}>{ "完整记录 (LLM 面板)" }</Link<Route>>
                        <button type="button" class={classes!("primary")} disabled={*loading} onclick={on_refresh}>
                            { if *loading { "Loading..." } else { "Refresh" } }
                        </button>
                    </div>
                </header>

                if let Some(err) = (*error).clone() {
                    <div class={classes!("errorline", "text-sm")}>{ err }</div>
                }

                <section class={classes!("panel", "popover-host")}>
                    <div class={classes!("panel-body", "space-y-2")}>
                        <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                            <DateRangePicker
                                start_ms={parse_datetime_local_input_to_ms(&start_input)}
                                end_ms={parse_datetime_local_input_to_ms(&end_input)}
                                on_change={{
                                    let start_input = start_input.clone();
                                    let end_input = end_input.clone();
                                    Callback::from(move |(start, end): (Option<i64>, Option<i64>)| {
                                        start_input.set(start.map(format_datetime_local_input).unwrap_or_default());
                                        end_input.set(end.map(format_datetime_local_input).unwrap_or_default());
                                    })
                                }}
                            />
                        </div>
                        <div class={classes!("flex", "items-end", "gap-2", "flex-wrap")}>
                            <select
                                key={format!("kiro-usage-key-{}", (*key_filter_input).clone())}
                                class={classes!("w-auto", "max-w-[18rem]", "text-xs")}
                                onchange={{
                                    let key_filter_input = key_filter_input.clone();
                                    Callback::from(move |event: Event| {
                                        let input: HtmlSelectElement = event.target_unchecked_into();
                                        key_filter_input.set(input.value());
                                    })
                                }}
                            >
                                <option value="" selected={(*key_filter_input).is_empty()}>{ "全部 Key" }</option>
                                { for (*key_options).iter().map(|key_item| html! {
                                    <option
                                        value={key_item.id.clone()}
                                        selected={(*key_filter_input).as_str() == key_item.id.as_str()}
                                    >
                                        { format!("{} · {}", key_item.name, key_item.id) }
                                    </option>
                                }) }
                            </select>
                            <select
                                key={format!("kiro-usage-source-{}", (*source_input).clone())}
                                class={classes!("w-auto", "text-xs")}
                                onchange={{
                                    let source_input = source_input.clone();
                                    Callback::from(move |event: Event| {
                                        let input: HtmlSelectElement = event.target_unchecked_into();
                                        source_input.set(input.value());
                                    })
                                }}
                            >
                                <option value={USAGE_SOURCE_HOT} selected={*source_input == USAGE_SOURCE_HOT}>{ "在线" }</option>
                                <option value={USAGE_SOURCE_ARCHIVE} selected={*source_input == USAGE_SOURCE_ARCHIVE}>{ "归档" }</option>
                                <option value={USAGE_SOURCE_ALL} selected={*source_input == USAGE_SOURCE_ALL}>{ "全部" }</option>
                            </select>
                            <select
                                key={format!("kiro-usage-status-{}", (*status_kind_input).clone())}
                                class={classes!("w-auto", "text-xs")}
                                onchange={{
                                    let status_kind_input = status_kind_input.clone();
                                    Callback::from(move |event: Event| {
                                        let input: HtmlSelectElement = event.target_unchecked_into();
                                        status_kind_input.set(input.value());
                                    })
                                }}
                            >
                                <option value={USAGE_STATUS_KIND_ALL} selected={*status_kind_input == USAGE_STATUS_KIND_ALL}>{ "全部状态" }</option>
                                <option value={USAGE_STATUS_KIND_OK} selected={*status_kind_input == USAGE_STATUS_KIND_OK}>{ "200" }</option>
                                <option value={USAGE_STATUS_KIND_NON_OK} selected={*status_kind_input == USAGE_STATUS_KIND_NON_OK}>{ "非200" }</option>
                            </select>
                            <input
                                type="text"
                                class={classes!("w-28", "mono", "text-xs")}
                                placeholder="model"
                                value={(*model_filter_input).clone()}
                                oninput={{
                                    let model_filter_input = model_filter_input.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        model_filter_input.set(input.value());
                                    })
                                }}
                            />
                            <input
                                type="text"
                                class={classes!("w-28", "mono", "text-xs")}
                                placeholder="account"
                                value={(*account_filter_input).clone()}
                                oninput={{
                                    let account_filter_input = account_filter_input.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        account_filter_input.set(input.value());
                                    })
                                }}
                            />
                            <input
                                type="text"
                                class={classes!("w-36", "mono", "text-xs")}
                                placeholder="endpoint"
                                value={(*endpoint_filter_input).clone()}
                                oninput={{
                                    let endpoint_filter_input = endpoint_filter_input.clone();
                                    Callback::from(move |event: InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        endpoint_filter_input.set(input.value());
                                    })
                                }}
                            />
                            <button type="button" class={classes!("primary")} onclick={on_apply_filters} disabled={*loading}>
                                { "查询" }
                            </button>
                            <button type="button" class={classes!("ghost")} onclick={on_reset_filters} disabled={*loading}>
                                { "重置" }
                            </button>
                        </div>
                    </div>
                </section>

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
                            <span>{ "Credits (匹配)" }</span>
                            <b>{ matched_credit_text.clone() }</b>
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
                            <span>{ "当前筛选下暂无 usage 记录" }</span>
                        </div>
                    } else {
                        { pager(on_prev.clone(), on_next.clone()) }
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
                                                <td class={classes!("py-2", "pl-4", "pr-3", "whitespace-nowrap")}>
                                                    <div class={classes!("text-xs")}>{ format_ms(event.created_at) }</div>
                                                    <div class={classes!("mt-0.5", "max-w-[9rem]", "truncate", "mono", "text-[10px]", "text-[var(--faint)]")} title={event.id.clone()}>
                                                        { event.id.clone() }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2", "pr-3")}>
                                                    <div class={classes!("max-w-[10rem]", "truncate", "text-xs", "font-semibold")} title={event.key_name.clone()}>
                                                        { event.key_name.clone() }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2", "pr-3")}>
                                                    <div class={classes!("max-w-[10rem]", "truncate", "mono", "text-xs", "text-[var(--muted-foreground)]")} title={event.model.clone().unwrap_or_default()}>
                                                        { event.model.clone().unwrap_or_else(|| "-".to_string()) }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2", "pr-3", "min-w-[12rem]", "max-w-[22rem]")}>
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
                                                <td class={classes!("py-2", "pr-3", "whitespace-nowrap")}>
                                                    <div class={classes!("flex", "items-center", "gap-1")}>
                                                        <span class={classes!("inline-flex", "rounded-full", "border", "px-2", "py-0.5", "text-[11px]", "font-semibold", latency_color.0, latency_color.1, latency_color.2, latency_color.3)}>
                                                            { format_latency_ms(event.latency_ms) }
                                                        </span>
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
                                                <td class={classes!("py-2", "pr-3", "whitespace-nowrap", "mono", "text-[11px]")}>
                                                    <div>
                                                        { format!("{} / {}", format_number_u64(event.input_uncached_tokens + event.input_cached_tokens), format_number_u64(event.output_tokens)) }
                                                    </div>
                                                    <div class={classes!("text-[var(--faint)]")}>
                                                        { format!("缓存↓ {}", format_number_u64(event.input_cached_tokens)) }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2", "pr-3", "whitespace-nowrap", "mono", "text-xs")}>
                                                    { credit_text }
                                                </td>
                                                <td class={classes!("py-2", "pr-4")}>
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
                        { pager(on_prev.clone(), on_next.clone()) }
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
                    let detail_error_summary = usage_error_summary(
                        detail.status_code,
                        detail.error_message.as_deref(),
                        detail.error_class.as_deref(),
                        detail.session_blocked,
                    );
                    let detail_routing_wait_ms = effective_routing_wait_ms(
                        detail.routing_wait_ms,
                        detail.routing_diagnostics_json.as_deref(),
                    );
                    let latency_breakdown = format_latency_breakdown(LatencyBreakdown {
                        latency_ms: detail.latency_ms,
                        routing_wait_ms: detail_routing_wait_ms,
                        upstream_headers_ms: detail.upstream_headers_ms,
                        post_headers_body_ms: detail.post_headers_body_ms,
                        request_body_bytes: detail.request_body_bytes,
                        request_body_read_ms: detail.request_body_read_ms,
                        request_json_parse_ms: detail.request_json_parse_ms,
                        pre_handler_ms: detail.pre_handler_ms,
                        first_sse_write_ms: detail.first_sse_write_ms,
                        stream_finish_ms: detail.stream_finish_ms,
                        other_latency_ms: detail.other_latency_ms,
                        quota_failover_count: detail.quota_failover_count,
                    });
                    let credit_value = detail
                        .credit_usage
                        .map(|credit| format!("{credit:.4}"))
                        .unwrap_or_else(|| {
                            if detail.credit_usage_missing {
                                "缺失".to_string()
                            } else {
                                "-".to_string()
                            }
                        });
                    let retry_value = if detail.same_account_retry_count > 0 {
                        format!(
                            "×{} · {} ms · {}",
                            detail.same_account_retry_count,
                            detail.same_account_retry_delay_ms,
                            detail.same_account_retry_reasons.join(", "),
                        )
                    } else {
                        "-".to_string()
                    };
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
                                    if let Some(summary) = detail_error_summary {
                                        <div class={classes!("errorline", "text-sm")}>{ summary }</div>
                                    }
                                    <div class={classes!("grid", "gap-3", "sm:grid-cols-2", "lg:grid-cols-4")}>
                                        { detail_kv("created", format_ms(detail.created_at)) }
                                        { detail_kv("key", detail.key_name.clone()) }
                                        { detail_kv("account", detail.account_name.clone().unwrap_or_else(|| "-".to_string())) }
                                        { detail_kv("model", detail.model.clone().unwrap_or_else(|| "-".to_string())) }
                                        { detail_kv("status", detail.status_code.to_string()) }
                                        { detail_kv("latency", format!("{} ms", detail.latency_ms.max(0))) }
                                        { detail_kv("first token", detail.first_sse_write_ms.map(|value| format!("{} ms", value.max(0))).unwrap_or_else(|| "-".to_string())) }
                                        { detail_kv("credit", credit_value) }
                                        { detail_kv("stream", usage_stream_state_label(detail.stream_completed_cleanly, detail.downstream_disconnect).to_string()) }
                                        { detail_kv("final event", detail.final_event_type.clone().unwrap_or_else(|| "-".to_string())) }
                                        { detail_kv("stream bytes", format_optional_stream_bytes(detail.bytes_streamed)) }
                                        { detail_kv("retries", retry_value) }
                                        { detail_kv("input", format_number_u64(detail.input_uncached_tokens)) }
                                        { detail_kv("cached", format_number_u64(detail.input_cached_tokens)) }
                                        { detail_kv("output", format_number_u64(detail.output_tokens)) }
                                        { detail_kv("billable", format_number_u64(detail.billable_tokens)) }
                                        { detail_kv("endpoint", detail.endpoint.clone()) }
                                        { detail_kv("request", format!("{} {}", detail.request_method, detail.request_url)) }
                                        { detail_kv("client", format!("{} · {}", detail.client_ip, detail.ip_region)) }
                                        {
                                            if let Some(summary) = anthropic_summary {
                                                detail_kv("routing", summary)
                                            } else {
                                                Html::default()
                                            }
                                        }
                                    </div>
                                    { detail_pre("latency breakdown", latency_breakdown) }
                                    if let Some(last_message) = detail.last_message_content.clone() {
                                        { detail_pre("last message", last_message) }
                                    }
                                    { detail_pre("routing diagnostics", detail.routing_diagnostics_json.as_deref().map(pretty_json_text).unwrap_or_else(|| "-".to_string())) }
                                    { detail_pre("request headers", pretty_headers_json(&detail.request_headers_json)) }
                                    { detail_pre("client request", detail.client_request_body_json.as_deref().map(pretty_json_text).unwrap_or_else(|| "-".to_string())) }
                                    { detail_pre("upstream request", detail.upstream_request_body_json.as_deref().map(pretty_json_text).unwrap_or_else(|| "-".to_string())) }
                                    { detail_pre("full request", detail.full_request_json.as_deref().map(pretty_json_text).unwrap_or_else(|| "-".to_string())) }
                                    if let Some(error_body) = detail.error_body.clone() {
                                        { detail_pre("error body", error_body) }
                                    }
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
