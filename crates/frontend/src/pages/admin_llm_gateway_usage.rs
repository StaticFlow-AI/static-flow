//! Usage-events page (`/admin/llm-gateway/usage`).
//!
//! Standalone `.admin-shell` page: server-paginated usage events with a
//! date-range + key/model/account/endpoint/status filter bar, a full event
//! detail modal, and a debounced server-side key-filter search. Extracted
//! from the mega llm gateway panel.

use gloo_timers::callback::Timeout;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_llm_gateway::{
    compute_other_latency_ms, copy_icon_button, effective_routing_wait_ms, format_credit4,
    format_datetime_local_input, format_latency_breakdown, format_optional_bytes,
    format_optional_latency_ms, format_optional_latency_ms_or_na, format_stream_summary,
    normalized_usage_filter_text, normalized_usage_status_kind, parse_datetime_local_input_to_ms,
    pretty_headers_json, pretty_json_text, routing_diagnostics_summary, usage_account_label,
    usage_retry_title, usage_source_label, usage_status_kind_label,
    usage_stream_state_badge_classes, usage_stream_state_label, usage_time_description,
    LatencyBreakdown, UsageReloadArgs, USAGE_KEY_OPTION_LIMIT, USAGE_PAGE_SIZE, USAGE_SOURCE_ALL,
    USAGE_SOURCE_ARCHIVE, USAGE_SOURCE_HOT, USAGE_STATUS_KIND_ALL, USAGE_STATUS_KIND_NON_OK,
    USAGE_STATUS_KIND_OK,
};
use crate::{
    api::{
        fetch_admin_llm_gateway_keys_page_with_query, fetch_admin_llm_gateway_usage_event_detail,
        fetch_admin_llm_gateway_usage_events, fetch_admin_llm_gateway_usage_filter_options,
        AdminLlmGatewayKeyPageQuery, AdminLlmGatewayKeyView, AdminLlmGatewayUsageEventDetailView,
        AdminLlmGatewayUsageEventView, AdminLlmGatewayUsageEventsQuery,
        AdminLlmGatewayUsageFilterOptionsResponse, AdminUsageTotalsView,
    },
    components::{
        copy_button::copy_to_clipboard, date_range_picker::DateRangePicker, pagination::Pagination,
        search_box::SearchBox,
    },
    pages::llm_access_shared::{
        credit_usage_missing_label, first_token_latency_color, format_latency_ms, format_ms,
        format_number_u64, token_usage_missing_label, total_latency_color, usage_error_summary,
    },
    router::Route,
};

#[function_component(AdminLlmGatewayUsagePage)]
pub fn admin_llm_gateway_usage_page() -> Html {
    let usage_events = use_state(Vec::<AdminLlmGatewayUsageEventView>::new);
    let usage_total = use_state(|| 0_usize);
    let usage_totals = use_state(AdminUsageTotalsView::default);
    let usage_page = use_state(|| 1_usize);
    let usage_current_rpm = use_state(|| 0_u32);
    let usage_current_in_flight = use_state(|| 0_u32);
    let usage_retention_days = use_state(|| 7_u64);
    let usage_loading = use_state(|| false);
    let usage_error = use_state(|| None::<String>);
    let usage_key_filter = use_state(String::new);
    let usage_key_search = use_state(String::new);
    let usage_key_search_debounced = use_state(String::new);
    let usage_key_options = use_state(Vec::<AdminLlmGatewayKeyView>::new);
    let usage_key_options_total = use_state(|| 0usize);
    let usage_key_filter_label = use_state(|| None::<String>);
    let usage_start_input = use_state(String::new);
    let usage_end_input = use_state(String::new);
    let usage_source = use_state(|| USAGE_SOURCE_HOT.to_string());
    let usage_model_filter = use_state(String::new);
    let usage_account_filter = use_state(String::new);
    let usage_endpoint_filter = use_state(String::new);
    let usage_filter_options = use_state(AdminLlmGatewayUsageFilterOptionsResponse::default);
    let usage_status_kind = use_state(|| USAGE_STATUS_KIND_ALL.to_string());
    let selected_usage_event = use_state(|| None::<AdminLlmGatewayUsageEventDetailView>);
    let usage_detail_loading = use_state(|| false);
    let toast = use_state(|| None::<(String, bool)>);
    let toast_timeout = use_mut_ref(|| None::<Timeout>);
    let flash = {
        let toast = toast.clone();
        let toast_timeout = toast_timeout.clone();
        Callback::from(move |(message, is_error): (String, bool)| {
            toast.set(Some((message, is_error)));
            toast_timeout.borrow_mut().take();
            let toast = toast.clone();
            let clear_handle = toast_timeout.clone();
            let timeout = Timeout::new(2600, move || {
                toast.set(None);
                clear_handle.borrow_mut().take();
            });
            *toast_timeout.borrow_mut() = Some(timeout);
        })
    };

    let open_usage_detail = {
        let selected_usage_event = selected_usage_event.clone();
        let usage_detail_loading = usage_detail_loading.clone();
        let flash = flash.clone();
        Callback::from(move |event_id: String| {
            let selected_usage_event = selected_usage_event.clone();
            let usage_detail_loading = usage_detail_loading.clone();
            let flash = flash.clone();
            selected_usage_event.set(None);
            usage_detail_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_admin_llm_gateway_usage_event_detail(&event_id).await {
                    Ok(detail) => selected_usage_event.set(Some(detail)),
                    Err(err) => flash.emit((err, true)),
                }
                usage_detail_loading.set(false);
            });
        })
    };

    let reload_usage = {
        let usage_events = usage_events.clone();
        let usage_total = usage_total.clone();
        let usage_totals = usage_totals.clone();
        let usage_filter_options = usage_filter_options.clone();
        let usage_page = usage_page.clone();
        let usage_current_rpm = usage_current_rpm.clone();
        let usage_current_in_flight = usage_current_in_flight.clone();
        let usage_retention_days = usage_retention_days.clone();
        let usage_loading = usage_loading.clone();
        let usage_error = usage_error.clone();
        let usage_key_filter = usage_key_filter.clone();
        let usage_start_input = usage_start_input.clone();
        let usage_end_input = usage_end_input.clone();
        let usage_source = usage_source.clone();
        let usage_model_filter = usage_model_filter.clone();
        let usage_account_filter = usage_account_filter.clone();
        let usage_endpoint_filter = usage_endpoint_filter.clone();
        let usage_status_kind = usage_status_kind.clone();
        Callback::from(move |args: UsageReloadArgs| {
            let usage_events = usage_events.clone();
            let usage_total = usage_total.clone();
            let usage_totals = usage_totals.clone();
            let usage_filter_options = usage_filter_options.clone();
            let usage_page = usage_page.clone();
            let usage_current_rpm = usage_current_rpm.clone();
            let usage_current_in_flight = usage_current_in_flight.clone();
            let usage_retention_days = usage_retention_days.clone();
            let usage_loading = usage_loading.clone();
            let usage_error = usage_error.clone();
            let usage_key_filter = usage_key_filter.clone();
            let usage_start_input = usage_start_input.clone();
            let usage_end_input = usage_end_input.clone();
            let usage_source = usage_source.clone();
            let usage_model_filter = usage_model_filter.clone();
            let usage_account_filter = usage_account_filter.clone();
            let usage_endpoint_filter = usage_endpoint_filter.clone();
            let usage_status_kind = usage_status_kind.clone();
            let page = args.page.unwrap_or(*usage_page).max(1);
            let selected_key_id = args.key_id.unwrap_or_else(|| (*usage_key_filter).clone());
            let selected_start_input = args
                .start_input
                .unwrap_or_else(|| (*usage_start_input).clone());
            let selected_end_input = args.end_input.unwrap_or_else(|| (*usage_end_input).clone());
            let selected_source = args.source.unwrap_or_else(|| (*usage_source).clone());
            let selected_model = args.model.unwrap_or_else(|| (*usage_model_filter).clone());
            let selected_account = args
                .account_name
                .unwrap_or_else(|| (*usage_account_filter).clone());
            let selected_endpoint = args
                .endpoint
                .unwrap_or_else(|| (*usage_endpoint_filter).clone());
            let selected_status_kind = args
                .status_kind
                .unwrap_or_else(|| (*usage_status_kind).clone());
            let start_ms = parse_datetime_local_input_to_ms(&selected_start_input);
            let end_ms = parse_datetime_local_input_to_ms(&selected_end_input);
            usage_loading.set(true);
            usage_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let query = AdminLlmGatewayUsageEventsQuery {
                    key_id: (!selected_key_id.is_empty()).then_some(selected_key_id),
                    start_ms,
                    end_ms,
                    source: Some(selected_source),
                    model: normalized_usage_filter_text(&selected_model),
                    account_name: normalized_usage_filter_text(&selected_account),
                    endpoint: normalized_usage_filter_text(&selected_endpoint),
                    status_code: None,
                    status_kind: normalized_usage_status_kind(&selected_status_kind),
                    limit: Some(USAGE_PAGE_SIZE),
                    offset: Some((page - 1) * USAGE_PAGE_SIZE),
                };
                if args.refresh_filter_options {
                    let filter_options_query = AdminLlmGatewayUsageEventsQuery {
                        offset: None,
                        limit: None,
                        ..query.clone()
                    };
                    if let Ok(options) =
                        fetch_admin_llm_gateway_usage_filter_options(&filter_options_query).await
                    {
                        usage_filter_options.set(options);
                    }
                }
                match fetch_admin_llm_gateway_usage_events(&query).await {
                    Ok(resp) => {
                        usage_total.set(resp.total);
                        usage_totals.set(resp.totals);
                        usage_current_rpm.set(resp.current_rpm);
                        usage_current_in_flight.set(resp.current_in_flight);
                        usage_retention_days.set(resp.retention_days);
                        usage_events.set(resp.events);
                        let actual_page = (resp.offset / resp.limit.max(1)).saturating_add(1);
                        usage_page.set(actual_page.max(1));
                    },
                    Err(err) => {
                        usage_totals.set(AdminUsageTotalsView::default());
                        usage_current_rpm.set(0);
                        usage_current_in_flight.set(0);
                        usage_error.set(Some(err));
                    },
                }
                usage_loading.set(false);
            });
        })
    };

    let on_copy = {
        let flash = flash.clone();
        Callback::from(move |(label, value): (String, String)| {
            copy_to_clipboard(&value);
            flash.emit((format!("已复制{}", label), false));
        })
    };

    let on_usage_key_pick = {
        let usage_key_filter = usage_key_filter.clone();
        let usage_key_search = usage_key_search.clone();
        let usage_key_search_debounced = usage_key_search_debounced.clone();
        let usage_key_filter_label = usage_key_filter_label.clone();
        let usage_key_options = usage_key_options.clone();
        let usage_page = usage_page.clone();
        Callback::from(move |selected_key_id: String| {
            if selected_key_id.is_empty() {
                usage_key_search.set(String::new());
                usage_key_search_debounced.set(String::new());
                usage_key_filter_label.set(None);
            } else {
                // Remember the label so the selection stays readable even
                // after later searches page it out of the options list.
                usage_key_filter_label.set(
                    usage_key_options
                        .iter()
                        .find(|key_item| key_item.id == selected_key_id)
                        .map(|key_item| format!("{} · {}", key_item.name, key_item.id)),
                );
            }
            usage_key_filter.set(selected_key_id.clone());
            usage_page.set(1);
        })
    };

    let on_usage_key_filter_change = {
        let on_usage_key_pick = on_usage_key_pick.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                on_usage_key_pick.emit(target.value());
            }
        })
    };

    let on_usage_key_search_change = {
        let usage_key_search = usage_key_search.clone();
        Callback::from(move |value: String| usage_key_search.set(value))
    };

    let on_usage_key_search_debounced = {
        let usage_key_search_debounced = usage_key_search_debounced.clone();
        Callback::from(move |value: String| usage_key_search_debounced.set(value))
    };

    let on_usage_source_change = {
        let usage_source = usage_source.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                usage_source.set(target.value());
            }
        })
    };

    let on_usage_model_filter_input = {
        let usage_model_filter = usage_model_filter.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            usage_model_filter.set(value.clone());
        })
    };

    let on_usage_account_filter_input = {
        let usage_account_filter = usage_account_filter.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            usage_account_filter.set(value.clone());
        })
    };

    let on_usage_endpoint_filter_input = {
        let usage_endpoint_filter = usage_endpoint_filter.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            usage_endpoint_filter.set(value.clone());
        })
    };

    let on_usage_status_kind_change = {
        let usage_status_kind = usage_status_kind.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                usage_status_kind.set(target.value());
            }
        })
    };

    let on_apply_usage_filters = {
        let reload_usage = reload_usage.clone();
        Callback::from(move |_| {
            reload_usage.emit(UsageReloadArgs {
                page: Some(1),
                refresh_filter_options: true,
                ..UsageReloadArgs::default()
            });
        })
    };

    let on_clear_usage_filters = {
        let usage_key_filter = usage_key_filter.clone();
        let usage_key_search = usage_key_search.clone();
        let usage_start_input = usage_start_input.clone();
        let usage_end_input = usage_end_input.clone();
        let usage_source = usage_source.clone();
        let usage_model_filter = usage_model_filter.clone();
        let usage_account_filter = usage_account_filter.clone();
        let usage_endpoint_filter = usage_endpoint_filter.clone();
        let usage_status_kind = usage_status_kind.clone();
        let reload_usage = reload_usage.clone();
        Callback::from(move |_| {
            usage_key_filter.set(String::new());
            usage_key_search.set(String::new());
            usage_start_input.set(String::new());
            usage_end_input.set(String::new());
            usage_source.set(USAGE_SOURCE_HOT.to_string());
            usage_model_filter.set(String::new());
            usage_account_filter.set(String::new());
            usage_endpoint_filter.set(String::new());
            usage_status_kind.set(USAGE_STATUS_KIND_ALL.to_string());
            reload_usage.emit(UsageReloadArgs {
                page: Some(1),
                key_id: Some(String::new()),
                start_input: Some(String::new()),
                end_input: Some(String::new()),
                source: Some(USAGE_SOURCE_HOT.to_string()),
                model: Some(String::new()),
                account_name: Some(String::new()),
                endpoint: Some(String::new()),
                status_kind: Some(String::new()),
                refresh_filter_options: true,
            });
        })
    };

    let on_usage_page_change = {
        let usage_page = usage_page.clone();
        let reload_usage = reload_usage.clone();
        Callback::from(move |page: usize| {
            usage_page.set(page);
            reload_usage.emit(UsageReloadArgs {
                page: Some(page),
                refresh_filter_options: false,
                ..UsageReloadArgs::default()
            });
        })
    };

    {
        let reload_usage = reload_usage.clone();
        use_effect_with((), move |_| {
            reload_usage.emit(UsageReloadArgs::default());
            || ()
        });
    }

    {
        let usage_key_options = usage_key_options.clone();
        let usage_key_options_total = usage_key_options_total.clone();
        use_effect_with((*usage_key_search_debounced).clone(), move |query| {
            let usage_key_options = usage_key_options.clone();
            let usage_key_options_total = usage_key_options_total.clone();
            let page_query = AdminLlmGatewayKeyPageQuery {
                q: Some(query.clone()),
                active_only: false,
                sort: None,
            };
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = fetch_admin_llm_gateway_keys_page_with_query(
                    USAGE_KEY_OPTION_LIMIT,
                    0,
                    &page_query,
                )
                .await
                {
                    usage_key_options_total.set(response.total);
                    usage_key_options.set(response.keys);
                }
            });
            || ()
        });
    }

    let usage_total_pages = (*usage_total).max(1).div_ceil(USAGE_PAGE_SIZE);
    let usage_key_query_lower = (*usage_key_search_debounced).trim().to_lowercase();
    let filtered_usage_keys: Vec<AdminLlmGatewayKeyView> = { (*usage_key_options).clone() };

    let usage_detail_modal = if *usage_detail_loading {
        Some(html! {
            <div class={classes!(
                "fixed",
                "inset-0",
                "z-[90]",
                "flex",
                "items-center",
                "justify-center",
                "bg-slate-950/58",
                "backdrop-blur-sm",
                "px-4",
                "py-8"
            )}>
                <div class={classes!(
                    "rounded-xl",
                    "border",
                    "border-[var(--border)]",
                    "bg-[var(--surface)]",
                    "px-5",
                    "py-4",
                    "text-sm",
                    "text-[var(--muted)]",
                    "shadow-[0_16px_48px_rgba(0,0,0,0.2)]"
                )}>
                    { "正在加载请求详情..." }
                </div>
            </div>
        })
    } else {
        (*selected_usage_event).clone().map(|event| {
        let account_label =
            usage_account_label(&event.account_name, &event.request_url, &event.endpoint);
        let detail_routing_wait_ms = effective_routing_wait_ms(
            event.routing_wait_ms,
            event.routing_diagnostics_json.as_deref(),
        );
        let stream_summary = format_stream_summary(
            event.stream_completed_cleanly,
            event.downstream_disconnect,
            event.final_event_type.as_deref(),
            event.bytes_streamed,
        );
        let request_detail_summary = format!(
            "{} {} · {} / {} · key {} · account {} · status {} · model {} · route {} · latency {} · stream {}",
            event.request_method,
            event.request_url,
            event.client_ip,
            event.ip_region,
            event.key_name,
            account_label,
            event.status_code,
            event.model.clone().unwrap_or_else(|| "-".to_string()),
            event.endpoint,
            format_latency_breakdown(LatencyBreakdown {
                latency_ms: event.latency_ms,
                routing_wait_ms: detail_routing_wait_ms,
                upstream_headers_ms: event.upstream_headers_ms,
                post_headers_body_ms: event.post_headers_body_ms,
                request_body_bytes: event.request_body_bytes,
                request_body_read_ms: event.request_body_read_ms,
                request_json_parse_ms: event.request_json_parse_ms,
                pre_handler_ms: event.pre_handler_ms,
                first_sse_write_ms: event.first_sse_write_ms,
                stream_finish_ms: event.stream_finish_ms,
                other_latency_ms: event.other_latency_ms,
                quota_failover_count: event.quota_failover_count,
            }),
            stream_summary,
        );
        let last_message_for_copy = event
            .last_message_content
            .clone()
            .unwrap_or_else(|| "-".to_string());
        let headers_json_for_copy = pretty_headers_json(&event.request_headers_json);
        let routing_diagnostics_for_copy = event
            .routing_diagnostics_json
            .as_deref()
            .map(pretty_json_text);
        let routing_diagnostics_summary_rows = event
            .routing_diagnostics_json
            .as_deref()
            .map(routing_diagnostics_summary)
            .unwrap_or_default();
        let detail_other_latency_ms = event.other_latency_ms.or_else(|| {
            compute_other_latency_ms(
                event.latency_ms,
                detail_routing_wait_ms,
                event.upstream_headers_ms,
                event.post_headers_body_ms,
            )
        });
        let detail_sse_applicable = event.first_sse_write_ms.is_some();
        let detail_first_sse_label =
            format_optional_latency_ms_or_na(event.first_sse_write_ms, detail_sse_applicable);
        let client_request_json_for_copy = event
            .client_request_body_json
            .as_deref()
            .map(pretty_json_text);
        let full_request_json_for_copy = event
            .full_request_json
            .as_deref()
            .map(pretty_json_text);
        let upstream_request_json_for_copy = event
            .upstream_request_body_json
            .as_deref()
            .map(pretty_json_text);
        let response_body_for_copy = event.response_body.as_deref().map(pretty_json_text);
        html! {
            <div
                class={classes!(
                    "fixed",
                    "inset-0",
                    "z-[90]",
                    "flex",
                    "items-start",
                    "sm:items-center",
                    "justify-center",
                    "overflow-y-auto",
                    "bg-slate-950/58",
                    "backdrop-blur-sm",
                    "px-4",
                    "py-8"
                )}
                onclick={{
                    let selected_usage_event = selected_usage_event.clone();
                    Callback::from(move |_| selected_usage_event.set(None))
                }}
            >
                <div
                    class={classes!(
                        "w-full",
                        "mx-auto",
                        "flex",
                        "max-h-[92vh]",
                        "max-w-4xl",
                        "flex-col",
                        "overflow-y-auto",
                        "rounded-xl",
                        "border",
                        "border-[var(--border)]",
                        "bg-[var(--surface)]",
                        "p-5",
                        "shadow-[0_16px_48px_rgba(0,0,0,0.2)]"
                    )}
                    onclick={Callback::from(|event: MouseEvent| event.stop_propagation())}
                >
                    <div class={classes!("flex", "items-start", "justify-between", "gap-4", "flex-wrap", "shrink-0")}>
                        <div class={classes!("max-w-3xl")}>
                            <p class={classes!("m-0", "text-xs", "uppercase", "tracking-[0.18em]", "text-[var(--muted)]")}>{ "Request Detail" }</p>
                            <h2 class={classes!("mt-3", "text-2xl", "font-black", "tracking-[-0.03em]")}>{ event.key_name.clone() }</h2>
                            <p class={classes!("mt-2", "m-0", "break-all", "text-sm", "leading-7", "text-[var(--muted)]")}>
                                { format!("{} {} · {} / {}", event.request_method, event.request_url, event.client_ip, event.ip_region) }
                            </p>
                        </div>
                        <div class={classes!("flex", "gap-2", "flex-wrap")}>
                            <button
                                class={classes!("ghost")}
                                onclick={{
                                    let on_copy = on_copy.clone();
                                    let request_detail_summary = request_detail_summary.clone();
                                    Callback::from(move |_| on_copy.emit(("Request Summary".to_string(), request_detail_summary.clone())))
                                }}
                            >
                                { "复制摘要" }
                            </button>
                            <button
                                class={classes!("ghost")}
                                onclick={{
                                    let on_copy = on_copy.clone();
                                    let headers_json_for_copy = headers_json_for_copy.clone();
                                    Callback::from(move |_| on_copy.emit(("Headers".to_string(), headers_json_for_copy.clone())))
                                }}
                            >
                                { "复制 Headers" }
                            </button>
                            <button
                                class={classes!("primary")}
                                onclick={{
                                    let selected_usage_event = selected_usage_event.clone();
                                    Callback::from(move |_| selected_usage_event.set(None))
                                }}
                            >
                                { "关闭" }
                            </button>
                        </div>
                    </div>

                    <div class={classes!("mt-4", "grid", "gap-3", "lg:grid-cols-6")}>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Key ID" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-xs", "break-all")}>{ event.key_id.clone() }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Account" }</div>
                            <div class={classes!("mt-1", "text-sm")}>{ usage_account_label(&event.account_name, &event.request_url, &event.endpoint) }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Status / Model" }</div>
                            <div class={classes!("mt-1", "text-sm")}>{ format!("{} · {}", event.status_code, event.model.clone().unwrap_or_else(|| "-".to_string())) }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Route" }</div>
                            <div class={classes!("mt-1", "font-mono", "text-xs", "break-all")}>{ event.endpoint.clone() }</div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Latency" }</div>
                            <div class={classes!("mt-1", "text-sm", "font-semibold")}>{ format_latency_ms(event.latency_ms) }</div>
                            <div class={classes!("mt-2", "grid", "gap-1", "font-mono", "text-[11px]", "text-[var(--muted)]")}>
                                <span>{ format!("route {}", format_optional_latency_ms(detail_routing_wait_ms)) }</span>
                                <span>{ format!("upstream headers {}", format_optional_latency_ms(event.upstream_headers_ms)) }</span>
                                <span>{ format!("post-headers body {}", format_optional_latency_ms(event.post_headers_body_ms)) }</span>
                                <span>{ format!("request body {}", format_optional_bytes(event.request_body_bytes)) }</span>
                                <span>{ format!("body read {}", format_optional_latency_ms(event.request_body_read_ms)) }</span>
                                <span>{ format!("json parse {}", format_optional_latency_ms(event.request_json_parse_ms)) }</span>
                                <span>{ format!("pre-handler {}", format_optional_latency_ms(event.pre_handler_ms)) }</span>
                                <span>{ format!("first SSE {}", detail_first_sse_label.clone()) }</span>
                                <span>{ format!("stream finish {}", format_optional_latency_ms(event.stream_finish_ms)) }</span>
                                <span>{ format!("other {}", format_optional_latency_ms(detail_other_latency_ms)) }</span>
                                <span>{ format!("quota failover {}", event.quota_failover_count) }</span>
                            </div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Stream" }</div>
                            <div class={classes!("mt-1", "flex", "items-center", "gap-2", "flex-wrap")}>
                                <span class={usage_stream_state_badge_classes(event.stream_completed_cleanly, event.downstream_disconnect)}>
                                    { usage_stream_state_label(event.stream_completed_cleanly, event.downstream_disconnect) }
                                </span>
                                <span class={classes!("font-mono", "text-xs", "text-[var(--muted)]")}>
                                    { format!("final {}", event.final_event_type.clone().unwrap_or_else(|| "-".to_string())) }
                                </span>
                            </div>
                            <div class={classes!("mt-2", "grid", "gap-1", "font-mono", "text-[11px]", "text-[var(--muted)]")}>
                                <span>{ format!("bytes {}", format_optional_bytes(event.bytes_streamed)) }</span>
                                <span>{ format!("disconnect {}", event.downstream_disconnect.map(|value| if value { "yes" } else { "no" }).unwrap_or("-")) }</span>
                            </div>
                        </div>
                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "px-3", "py-3")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Credit" }</div>
                            <div class={classes!("mt-1", "text-sm", "font-semibold")}>
                                { event.credit_usage.map(format_credit4).unwrap_or_else(|| "-".to_string()) }
                            </div>
                            if event.credit_usage_missing {
                                <div class={classes!("mt-1", "text-xs", "text-amber-700", "dark:text-amber-200")}>{ credit_usage_missing_label() }</div>
                            }
                        </div>
                    </div>

                    if let Some(routing_diagnostics_for_copy) = routing_diagnostics_for_copy {
                        <div class={classes!("mt-4")}>
                            <div class={classes!("mb-2", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Routing Diagnostics" }</div>
                                <button
                                    class={classes!("ghost")}
                                    onclick={{
                                        let on_copy = on_copy.clone();
                                        let routing_diagnostics_for_copy = routing_diagnostics_for_copy.clone();
                                        Callback::from(move |_| on_copy.emit(("Routing Diagnostics".to_string(), routing_diagnostics_for_copy.clone())))
                                    }}
                                >
                                    { "复制 Routing Diagnostics" }
                                </button>
                            </div>
                            if !routing_diagnostics_summary_rows.is_empty() {
                                <div class={classes!("mb-3", "grid", "gap-2", "sm:grid-cols-2", "lg:grid-cols-4")}>
                                    { for routing_diagnostics_summary_rows.iter().map(|(label, value)| html! {
                                        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2")}>
                                            <div class={classes!("text-[11px]", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>{ label.clone() }</div>
                                            <div class={classes!("mt-1", "font-mono", "text-xs", "text-[var(--text)]", "break-all")}>{ value.clone() }</div>
                                        </div>
                                    }) }
                                </div>
                            }
                            <pre class={classes!(
                                "max-h-[42vh]",
                                "overflow-x-auto",
                                "overflow-y-auto",
                                "rounded-lg",
                                "bg-slate-950",
                                "p-3",
                                "text-xs",
                                "leading-6",
                                "text-lime-100",
                                "whitespace-pre-wrap",
                                "break-words"
                            )}>
                                { routing_diagnostics_for_copy }
                            </pre>
                        </div>
                    }

                    <div class={classes!("mt-4")}>
                        <div class={classes!("mb-2", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Last Message" }</div>
                            <button
                                class={classes!("ghost")}
                                onclick={{
                                    let on_copy = on_copy.clone();
                                    let last_message_for_copy = last_message_for_copy.clone();
                                    Callback::from(move |_| on_copy.emit(("Last Message".to_string(), last_message_for_copy.clone())))
                                }}
                            >
                                { "复制 Last Message" }
                            </button>
                        </div>
                        <pre class={classes!(
                            "max-h-[40vh]",
                            "overflow-x-auto",
                            "overflow-y-auto",
                            "rounded-lg",
                            "bg-slate-950",
                            "p-3",
                            "text-xs",
                            "leading-6",
                            "text-amber-100",
                            "whitespace-pre-wrap",
                            "break-words"
                        )}>
                            { last_message_for_copy }
                        </pre>
                    </div>

                    <div class={classes!("mt-4")}>
                        <div class={classes!("mb-2", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                            <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Headers" }</div>
                            <button
                                class={classes!("ghost")}
                                onclick={{
                                    let on_copy = on_copy.clone();
                                    let headers_json_for_copy = headers_json_for_copy.clone();
                                    Callback::from(move |_| on_copy.emit(("Headers".to_string(), headers_json_for_copy.clone())))
                                }}
                            >
                                { "复制 Headers" }
                            </button>
                        </div>
                        <pre class={classes!(
                            "max-h-[42vh]",
                            "overflow-x-auto",
                            "overflow-y-auto",
                            "rounded-lg",
                            "bg-slate-950",
                            "p-3",
                            "text-xs",
                            "leading-6",
                            "text-emerald-200",
                            "whitespace-pre-wrap",
                            "break-words"
                        )}>
                            { headers_json_for_copy }
                        </pre>
                    </div>

                    if let Some(client_request_json_for_copy) = client_request_json_for_copy {
                        <div class={classes!("mt-4")}>
                            <div class={classes!("mb-2", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Client Request" }</div>
                                <button
                                    class={classes!("ghost")}
                                    onclick={{
                                        let on_copy = on_copy.clone();
                                        let client_request_json_for_copy = client_request_json_for_copy.clone();
                                        Callback::from(move |_| on_copy.emit(("Client Request".to_string(), client_request_json_for_copy.clone())))
                                    }}
                                >
                                    { "复制 Client Request" }
                                </button>
                            </div>
                            <pre class={classes!(
                                "max-h-[42vh]",
                                "overflow-x-auto",
                                "overflow-y-auto",
                                "rounded-lg",
                                "bg-slate-950",
                                "p-3",
                                "text-xs",
                                "leading-6",
                                "text-sky-100",
                                "whitespace-pre-wrap",
                                "break-words"
                            )}>
                                { client_request_json_for_copy }
                            </pre>
                        </div>
                    }

                    if let Some(full_request_json_for_copy) = full_request_json_for_copy {
                        <div class={classes!("mt-4")}>
                            <div class={classes!("mb-2", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Full Request" }</div>
                                <button
                                    class={classes!("ghost")}
                                    onclick={{
                                        let on_copy = on_copy.clone();
                                        let full_request_json_for_copy = full_request_json_for_copy.clone();
                                        Callback::from(move |_| on_copy.emit(("Full Request".to_string(), full_request_json_for_copy.clone())))
                                    }}
                                >
                                    { "复制 Full Request" }
                                </button>
                            </div>
                            <pre class={classes!(
                                "max-h-[42vh]",
                                "overflow-x-auto",
                                "overflow-y-auto",
                                "rounded-lg",
                                "bg-slate-950",
                                "p-3",
                                "text-xs",
                                "leading-6",
                                "text-cyan-100",
                                "whitespace-pre-wrap",
                                "break-words"
                            )}>
                                { full_request_json_for_copy }
                            </pre>
                        </div>
                    }

                    if let Some(upstream_request_json_for_copy) = upstream_request_json_for_copy {
                        <div class={classes!("mt-4")}>
                            <div class={classes!("mb-2", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Upstream Request" }</div>
                                <button
                                    class={classes!("ghost")}
                                    onclick={{
                                        let on_copy = on_copy.clone();
                                        let upstream_request_json_for_copy = upstream_request_json_for_copy.clone();
                                        Callback::from(move |_| on_copy.emit(("Upstream Request".to_string(), upstream_request_json_for_copy.clone())))
                                    }}
                                >
                                    { "复制 Upstream Request" }
                                </button>
                            </div>
                            <pre class={classes!(
                                "max-h-[42vh]",
                                "overflow-x-auto",
                                "overflow-y-auto",
                                "rounded-lg",
                                "bg-slate-950",
                                "p-3",
                                "text-xs",
                                "leading-6",
                                "text-fuchsia-100",
                                "whitespace-pre-wrap",
                                "break-words"
                            )}>
                                { upstream_request_json_for_copy }
                            </pre>
                        </div>
                    }

                    if let Some(response_body_for_copy) = response_body_for_copy {
                        <div class={classes!("mt-4")}>
                            <div class={classes!("mb-2", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Response Body" }</div>
                                <button
                                    class={classes!("ghost")}
                                    onclick={{
                                        let on_copy = on_copy.clone();
                                        let response_body_for_copy = response_body_for_copy.clone();
                                        Callback::from(move |_| on_copy.emit(("Response Body".to_string(), response_body_for_copy.clone())))
                                    }}
                                >
                                    { "复制 Response Body" }
                                </button>
                            </div>
                            <pre class={classes!(
                                "max-h-[42vh]",
                                "overflow-x-auto",
                                "overflow-y-auto",
                                "rounded-lg",
                                "bg-slate-950",
                                "p-3",
                                "text-xs",
                                "leading-6",
                                "text-violet-100",
                                "whitespace-pre-wrap",
                                "break-words"
                            )}>
                                { response_body_for_copy }
                            </pre>
                        </div>
                    }
                </div>
            </div>
        }
        })
    };

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "LLM Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Usage" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminLlmGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                    </div>
                </header>

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Usage Events" }</h2>
                            <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted)]")}>
                                { format!("仅展示最近 {} 天的 usage events", *usage_retention_days) }
                            </p>
                        </div>
                        <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                            <span class={classes!("text-xs", "text-[var(--muted)]")}>
                                { format!("{} · {} · {} · {} 条 · p{}", usage_source_label(&usage_source), usage_status_kind_label(&usage_status_kind), usage_time_description(&usage_start_input, &usage_end_input), *usage_total, *usage_page) }
                            </span>
                            <span class={classes!("rounded-full", "border", "border-[var(--border)]", "px-2.5", "py-0.5", "text-xs", "font-semibold", "text-[var(--muted)]")}>
                                { format!("RPM {}", *usage_current_rpm) }
                            </span>
                            <span class={classes!("rounded-full", "border", "border-[var(--border)]", "px-2.5", "py-0.5", "text-xs", "font-semibold", "text-[var(--muted)]")}>
                                { format!("In Flight {}", *usage_current_in_flight) }
                            </span>
                            <button
                                class={classes!("ghost")}
                                title="刷新事件"
                                aria-label="刷新事件"
                                onclick={{
                                    let reload_usage = reload_usage.clone();
                                    Callback::from(move |_| {
                                        reload_usage.emit(UsageReloadArgs::default())
                                    })
                                }}
                                disabled={*usage_loading}
                            >
                                <i class={classes!("fas", if *usage_loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                            </button>
                        </div>
                    </div>

                    <div class={classes!("mt-3", "flex", "flex-col", "gap-2")}>
                        // Row 1: date range picker
                        <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                            <DateRangePicker
                                start_ms={parse_datetime_local_input_to_ms(&usage_start_input)}
                                end_ms={parse_datetime_local_input_to_ms(&usage_end_input)}
                                on_change={{
                                    let usage_start_input = usage_start_input.clone();
                                    let usage_end_input = usage_end_input.clone();
                                    Callback::from(move |(start, end): (Option<i64>, Option<i64>)| {
                                        usage_start_input.set(start.map(format_datetime_local_input).unwrap_or_default());
                                        usage_end_input.set(end.map(format_datetime_local_input).unwrap_or_default());
                                    })
                                }}
                            />
                        </div>
                        // Row 2: key search + key filter dropdown
                        <div class={classes!("grid", "grid-cols-1", "sm:grid-cols-2", "gap-2", "items-end")}>
                            <div class={classes!("text-xs")}>
                                <SearchBox
                                    value={(*usage_key_search).clone()}
                                    on_change={on_usage_key_search_change.clone()}
                                    on_debounced_change={on_usage_key_search_debounced.clone()}
                                    placeholder={AttrValue::Static("搜索 key 名称 / id / provider")}
                                />
                            </div>
                            <select
                                key={format!("usage-filter-{}-{}", (*usage_key_filter).clone(), usage_key_query_lower)}
                                class={classes!("w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "text-xs")}
                                onchange={on_usage_key_filter_change}
                            >
                                <option value="" selected={(*usage_key_filter).is_empty()}>{ "全部 Key" }</option>
                                if !(*usage_key_filter).is_empty()
                                    && !filtered_usage_keys
                                        .iter()
                                        .any(|key_item| key_item.id.as_str() == (*usage_key_filter).as_str())
                                {
                                    <option
                                        value={(*usage_key_filter).clone()}
                                        selected=true
                                    >
                                        {
                                            format!(
                                                "{} (当前)",
                                                (*usage_key_filter_label)
                                                    .clone()
                                                    .unwrap_or_else(|| (*usage_key_filter).clone()),
                                            )
                                        }
                                    </option>
                                }
                                { for filtered_usage_keys.iter().map(|key_item| html! {
                                    <option
                                        value={key_item.id.clone()}
                                        selected={(*usage_key_filter).as_str() == key_item.id.as_str()}
                                    >
                                        { format!("{} · {}", key_item.name, key_item.id) }
                                    </option>
                                }) }
                            </select>
                        </div>
                        // Row 3: source, status, model, account, endpoint + action buttons
                        <div class={classes!("flex", "items-end", "gap-2", "flex-wrap")}>
                            <select
                                key={format!("usage-source-{}", (*usage_source).clone())}
                                class={classes!("w-auto", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "text-xs")}
                                onchange={on_usage_source_change}
                            >
                                <option value={USAGE_SOURCE_HOT} selected={*usage_source == USAGE_SOURCE_HOT}>{ "在线" }</option>
                                <option value={USAGE_SOURCE_ARCHIVE} selected={*usage_source == USAGE_SOURCE_ARCHIVE}>{ "归档" }</option>
                                <option value={USAGE_SOURCE_ALL} selected={*usage_source == USAGE_SOURCE_ALL}>{ "全部" }</option>
                            </select>
                            <select
                                key={format!("usage-status-kind-{}", (*usage_status_kind).clone())}
                                class={classes!("w-auto", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "text-xs")}
                                onchange={on_usage_status_kind_change}
                            >
                                <option value={USAGE_STATUS_KIND_ALL} selected={*usage_status_kind == USAGE_STATUS_KIND_ALL}>{ "全部状态" }</option>
                                <option value={USAGE_STATUS_KIND_OK} selected={*usage_status_kind == USAGE_STATUS_KIND_OK}>{ "200" }</option>
                                <option value={USAGE_STATUS_KIND_NON_OK} selected={*usage_status_kind == USAGE_STATUS_KIND_NON_OK}>{ "非200" }</option>
                            </select>
                            <input
                                type="text"
                                list="usage-filter-models"
                                class={classes!("w-28", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "font-mono", "text-xs")}
                                placeholder="model"
                                value={(*usage_model_filter).clone()}
                                oninput={on_usage_model_filter_input}
                            />
                            <datalist id="usage-filter-models">
                                { for usage_filter_options.models.iter().map(|m| html! { <option value={m.clone()} /> }) }
                            </datalist>
                            <input
                                type="text"
                                list="usage-filter-accounts"
                                class={classes!("w-28", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "font-mono", "text-xs")}
                                placeholder="account"
                                value={(*usage_account_filter).clone()}
                                oninput={on_usage_account_filter_input}
                            />
                            <datalist id="usage-filter-accounts">
                                { for usage_filter_options.accounts.iter().map(|a| html! { <option value={a.clone()} /> }) }
                            </datalist>
                            <input
                                type="text"
                                list="usage-filter-endpoints"
                                class={classes!("w-36", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-2.5", "py-1.5", "font-mono", "text-xs")}
                                placeholder="endpoint"
                                value={(*usage_endpoint_filter).clone()}
                                oninput={on_usage_endpoint_filter_input}
                            />
                            <datalist id="usage-filter-endpoints">
                                { for usage_filter_options.endpoints.iter().map(|ep| html! { <option value={ep.clone()} /> }) }
                            </datalist>
                            <button
                                type="button"
                                class={classes!("btn-terminal", "!py-1", "!px-2.5", "!text-xs")}
                                onclick={on_apply_usage_filters}
                                disabled={*usage_loading}
                            >
                                { "查询" }
                            </button>
                            <button
                                type="button"
                                class={classes!("btn-terminal", "!py-1", "!px-2.5", "!text-xs")}
                                onclick={on_clear_usage_filters}
                                disabled={*usage_loading}
                            >
                                { "重置" }
                            </button>
                        </div>
                    </div>

                    <div class={classes!("mt-3", "flex", "items-center", "gap-x-4", "gap-y-1", "flex-wrap", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-4", "py-2", "font-mono", "text-xs")}>
                        <span><span class={classes!("text-[var(--muted)]")}>{ "匹配 " }</span><span class={classes!("font-semibold")}>{ format_number_u64(usage_totals.event_count as u64) }</span></span>
                        <span class={classes!("text-[var(--border)]")}>{ "·" }</span>
                        <span><span class={classes!("text-[var(--muted)]")}>{ "In " }</span><span class={classes!("font-semibold")}>{ format_number_u64(usage_totals.input_uncached_tokens) }</span></span>
                        <span class={classes!("text-[var(--border)]")}>{ "·" }</span>
                        <span><span class={classes!("text-[var(--muted)]")}>{ "Cached " }</span><span class={classes!("font-semibold")}>{ format_number_u64(usage_totals.input_cached_tokens) }</span></span>
                        <span class={classes!("text-[var(--border)]")}>{ "·" }</span>
                        <span><span class={classes!("text-[var(--muted)]")}>{ "Out " }</span><span class={classes!("font-semibold")}>{ format_number_u64(usage_totals.output_tokens) }</span></span>
                        <span class={classes!("text-[var(--border)]")}>{ "·" }</span>
                        <span><span class={classes!("text-[var(--muted)]")}>{ "Billable " }</span><span class={classes!("font-semibold")}>{ format_number_u64(usage_totals.billable_tokens) }</span></span>
                    </div>

                    if !usage_key_query_lower.is_empty() {
                        <div class={classes!("mt-2", "flex", "items-center", "gap-2", "flex-wrap", "text-xs", "font-mono", "text-[var(--muted)]")}>
                            <span>{ format!("匹配 {}/{}", filtered_usage_keys.len(), *usage_key_options_total) }</span>
                            if filtered_usage_keys.is_empty() {
                                <span>{ "没有匹配的 key" }</span>
                            } else {
                                { for filtered_usage_keys.iter().take(8).map(|key_item| {
                                    let key_id = key_item.id.clone();
                                    let active = (*usage_key_filter).as_str() == key_item.id.as_str();
                                    let on_usage_key_pick = on_usage_key_pick.clone();
                                    html! {
                                        <button
                                            type="button"
                                            class={classes!(
                                                "rounded-full",
                                                "border",
                                                "px-2.5",
                                                "py-1",
                                                "text-xs",
                                                "font-semibold",
                                                if active { "border-emerald-500/50" } else { "border-[var(--border)]" },
                                                if active { "bg-emerald-500/12" } else { "bg-[var(--surface-alt)]" },
                                                if active { "text-emerald-700" } else { "text-[var(--text)]" },
                                                if active { "dark:text-emerald-200" } else { "dark:text-[var(--text)]" },
                                            )}
                                            onclick={Callback::from(move |_| on_usage_key_pick.emit(key_id.clone()))}
                                        >
                                            { format!("{} · {}", key_item.name, key_item.id) }
                                        </button>
                                    }
                                }) }
                                if *usage_key_options_total > 8 {
                                    <span>{ format!("另有 {} 个匹配项", *usage_key_options_total - 8) }</span>
                                }
                            }
                        </div>
                    }

                    if *usage_loading {
                        <div class={classes!("mt-3", "inline-flex", "items-center", "gap-2", "text-xs", "text-[var(--muted)]")}>
                            <i class={classes!("fas", "fa-spinner", "animate-spin")} />
                            <span>{ "加载中" }</span>
                        </div>
                    }
                    if let Some(err) = (*usage_error).clone() {
                        <div class={classes!("mt-3", "rounded-lg", "border", "border-red-400/35", "bg-red-500/8", "px-4", "py-3", "text-sm", "text-red-700", "dark:text-red-200")}>
                            <div class={classes!("font-semibold")}>{ "查询失败" }</div>
                            <pre class={classes!("mt-2", "m-0", "whitespace-pre-wrap", "break-all", "font-mono", "text-xs")}>{ err }</pre>
                        </div>
                    }

                    <div class={classes!("mt-3", "flex", "justify-end")}>
                        <Pagination current_page={*usage_page} total_pages={usage_total_pages} on_page_change={on_usage_page_change.clone()} />
                    </div>

                    <div class={classes!("mt-3", "overflow-x-auto", "rounded-xl", "border", "border-[var(--border)]")}>
                        <table class={classes!("min-w-[64rem]", "w-full", "text-sm")}>
                            <thead>
                                <tr class={classes!("text-left", "text-[var(--muted)]")}>
                                    <th class={classes!("py-2", "pl-3", "pr-3")}>{ "时间" }</th>
                                    <th class={classes!("py-2", "pr-3")}>{ "Key" }</th>
                                    <th class={classes!("py-2", "pr-3")}>{ "号池" }</th>
                                    <th class={classes!("py-2", "pr-3")}>{ "Model" }</th>
                                    <th class={classes!("py-2", "pr-3")}>{ "Status" }</th>
                                    <th class={classes!("py-2", "pr-3")}>{ "Latency" }</th>
                                    <th class={classes!("py-2", "pr-3")}>{ "Tokens" }</th>
                                    <th class={classes!("py-2", "pr-3")}>{ "" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                if usage_events.is_empty() && !*usage_loading && (*usage_error).is_none() {
                                    <tr class={classes!("border-t", "border-[var(--border)]")}>
                                        <td colspan="8" class={classes!("py-8", "text-center", "text-[var(--muted)]")}>{ "当前筛选下还没有 usage 事件" }</td>
                                    </tr>
                                } else {
                                    { for usage_events.iter().map(|event| {
                                        let event_id_for_detail = event.id.clone();
                                        let account_label = usage_account_label(
                                            &event.account_name,
                                            &event.request_url,
                                            &event.endpoint,
                                        );
                                        let latency_color = total_latency_color(event.latency_ms);
                                        let first_token = event.first_sse_write_ms.map(|first_ms| {
                                            let first_ms = first_ms.max(0);
                                            (first_ms, first_token_latency_color(first_ms))
                                        });
                                        let status_ok = (200..300).contains(&event.status_code);
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
                                        let same_account_retry_tooltip = (event.same_account_retry_count > 0)
                                            .then(|| {
                                                usage_retry_title(
                                                    event.same_account_retry_count,
                                                    event.same_account_retry_delay_ms,
                                                    &event.same_account_retry_reasons,
                                                )
                                            });
                                        let downstream_disconnect = event.downstream_disconnect == Some(true);
                                        let stream_incomplete =
                                            event.stream_completed_cleanly == Some(false) && !downstream_disconnect;
                                        html! {
                                            <tr class={classes!("border-t", "border-[var(--border)]", "align-top")}>
                                                <td class={classes!("py-2", "pl-3", "pr-3", "whitespace-nowrap")}>
                                                    <div class={classes!("text-xs")}>{ format_ms(event.created_at) }</div>
                                                    <div class={classes!("mt-0.5", "flex", "items-center", "gap-1")}>
                                                        <span class={classes!("max-w-[7rem]", "truncate", "font-mono", "text-[10px]", "text-[var(--muted)]")} title={event.id.clone()}>
                                                            { event.id.clone() }
                                                        </span>
                                                        { copy_icon_button(&event.id, &on_copy) }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2", "pr-3")}>
                                                    <div class={classes!("text-xs", "font-semibold", "text-[var(--text)]", "truncate", "max-w-[10rem]")} title={event.key_name.clone()}>{ event.key_name.clone() }</div>
                                                    <div class={classes!("max-w-[10rem]", "truncate", "font-mono", "text-[10px]", "text-[var(--muted)]")} title={event.key_id.clone()}>{ event.key_id.clone() }</div>
                                                </td>
                                                <td class={classes!("py-2", "pr-3")}>
                                                    <span class={classes!("inline-flex", "max-w-[11rem]", "rounded-full", "border", "border-emerald-500/20", "bg-emerald-500/10", "px-2", "py-0.5", "text-[11px]", "font-semibold", "text-emerald-700", "dark:text-emerald-200")} title={account_label.clone()}>
                                                        <span class={classes!("truncate")}>{ account_label.clone() }</span>
                                                    </span>
                                                </td>
                                                <td class={classes!("py-2", "pr-3")}>
                                                    <div class={classes!("text-xs", "truncate", "max-w-[10rem]")} title={event.model.clone().unwrap_or_default()}>
                                                        { event.model.clone().unwrap_or_else(|| "-".to_string()) }
                                                    </div>
                                                    if event.usage_missing {
                                                        <span class={classes!("inline-flex", "rounded-full", "border", "border-amber-500/20", "bg-amber-500/10", "px-1.5", "py-0.5", "text-[10px]", "font-semibold", "text-amber-700", "dark:text-amber-200")}>
                                                            { token_usage_missing_label() }
                                                        </span>
                                                    }
                                                </td>
                                                <td class={classes!("py-2", "pr-3", "min-w-[14rem]", "max-w-[24rem]")}>
                                                    <div class={classes!("flex", "flex-wrap", "items-center", "gap-1.5")}>
                                                        <span class={classes!(
                                                            "inline-flex", "items-center", "rounded-full", "border", "px-2", "py-0.5", "font-mono", "text-[11px]", "font-semibold",
                                                            if status_ok { "border-emerald-500/20" } else if event.status_code >= 500 { "border-red-500/20" } else { "border-amber-500/20" },
                                                            if status_ok { "bg-emerald-500/10" } else if event.status_code >= 500 { "bg-red-500/10" } else { "bg-amber-500/10" },
                                                            if status_ok { "text-emerald-700" } else if event.status_code >= 500 { "text-red-700" } else { "text-amber-700" },
                                                            if status_ok { "dark:text-emerald-200" } else if event.status_code >= 500 { "dark:text-red-200" } else { "dark:text-amber-200" },
                                                        )}>
                                                            { format!("status {}", event.status_code) }
                                                        </span>
                                                        if let Some(class_label) = error_class_label.clone() {
                                                            <span class={classes!("inline-flex", "items-center", "rounded-full", "border", "border-red-500/20", "bg-red-500/10", "px-2", "py-0.5", "font-mono", "text-[11px]", "font-semibold", "text-red-700", "dark:text-red-200")}>
                                                                { class_label }
                                                            </span>
                                                        }
                                                        if event.session_blocked {
                                                            <span class={classes!("inline-flex", "items-center", "rounded-full", "border", "border-red-600/25", "bg-red-600/10", "px-2", "py-0.5", "font-mono", "text-[11px]", "font-semibold", "text-red-800", "dark:text-red-200")}>
                                                                { "session blocked" }
                                                            </span>
                                                        }
                                                        if let Some(title) = same_account_retry_tooltip {
                                                            <span title={title} class={classes!("inline-flex", "items-center", "rounded-full", "border", "border-indigo-500/20", "bg-indigo-500/10", "px-2", "py-0.5", "font-mono", "text-[11px]", "font-semibold", "text-indigo-700", "dark:text-indigo-200")}>
                                                                { format!("retry ×{}", event.same_account_retry_count) }
                                                            </span>
                                                        }
                                                        if event.quota_failover_count > 0 {
                                                            <span class={classes!("inline-flex", "items-center", "rounded-full", "border", "border-amber-500/20", "bg-amber-500/10", "px-2", "py-0.5", "font-mono", "text-[11px]", "font-semibold", "text-amber-700", "dark:text-amber-200")}>
                                                                { format!("switch ×{}", event.quota_failover_count) }
                                                            </span>
                                                        }
                                                        if downstream_disconnect {
                                                            <span class={classes!("inline-flex", "items-center", "rounded-full", "border", "border-red-500/20", "bg-red-500/10", "px-2", "py-0.5", "font-mono", "text-[11px]", "font-semibold", "text-red-700", "dark:text-red-200")}>
                                                                { "disconnect" }
                                                            </span>
                                                        } else if stream_incomplete {
                                                            <span class={classes!("inline-flex", "items-center", "rounded-full", "border", "border-orange-500/20", "bg-orange-500/10", "px-2", "py-0.5", "font-mono", "text-[11px]", "font-semibold", "text-orange-700", "dark:text-orange-200")}>
                                                                { "incomplete" }
                                                            </span>
                                                        }
                                                    </div>
                                                    if let Some(summary) = status_error_summary.clone() {
                                                        <div class={classes!("mt-1", "max-w-[24rem]", "truncate", "font-mono", "text-[11px]", "text-red-700", "dark:text-red-300")} title={summary.clone()}>
                                                            { summary }
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
                                                            <span class={classes!("text-[10px]", "text-[var(--muted)]")}>{ "首字 -" }</span>
                                                        }
                                                    </div>
                                                </td>
                                                <td class={classes!("py-2", "pr-3", "whitespace-nowrap", "font-mono", "text-[11px]")}>
                                                    <span class={classes!("text-[var(--muted)]")}>
                                                        { format!("{}/{}/{}", format_number_u64(event.input_uncached_tokens), format_number_u64(event.input_cached_tokens), format_number_u64(event.output_tokens)) }
                                                    </span>
                                                </td>
                                                <td class={classes!("py-2", "pr-3")}>
                                                    <button
                                                        type="button"
                                                        class={classes!(
                                                            "inline-flex",
                                                            "h-7",
                                                            "w-7",
                                                            "items-center",
                                                            "justify-center",
                                                            "rounded-lg",
                                                            "border",
                                                            "border-[var(--border)]",
                                                            "bg-[var(--surface)]",
                                                            "text-[var(--muted)]",
                                                            "transition-colors",
                                                            "hover:text-[var(--primary)]",
                                                            "hover:bg-[var(--surface-alt)]"
                                                        )}
                                                        title="查看请求详情"
                                                        aria-label="查看请求详情"
                                                        onclick={{
                                                            let open_usage_detail = open_usage_detail.clone();
                                                            Callback::from(move |_| open_usage_detail.emit(event_id_for_detail.clone()))
                                                        }}
                                                    >
                                                        <i class={classes!("fas", "fa-bars-staggered", "text-xs")}></i>
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }) }
                                }
                            </tbody>
                        </table>
                    </div>

                    <div class={classes!("mt-5")}>
                        <Pagination current_page={*usage_page} total_pages={usage_total_pages} on_page_change={on_usage_page_change} />
                    </div>
                </section>

            </div>
            { usage_detail_modal.unwrap_or_default() }
            if let Some((message, is_error)) = (*toast).clone() {
                <div class={classes!("toasts")}>
                    <div class={classes!("toast", if is_error { "error" } else { "ok" })}>
                        { message }
                    </div>
                </div>
            }
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::admin_llm_gateway::preview_text;

    fn usage_last_message_preview(event: &AdminLlmGatewayUsageEventView) -> String {
        event
            .last_message_content
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "-".to_string())
    }

    fn usage_last_message_table_preview(event: &AdminLlmGatewayUsageEventView) -> String {
        let preview = usage_last_message_preview(event);
        if preview == "-" {
            return preview;
        }
        let single_line = preview.split_whitespace().collect::<Vec<_>>().join(" ");
        preview_text(&single_line, 120)
    }

    #[test]
    fn usage_last_message_preview_prefers_summary_content() {
        let event = AdminLlmGatewayUsageEventView {
            last_message_content: Some("hello".to_string()),
            ..AdminLlmGatewayUsageEventView::default()
        };

        assert_eq!(usage_last_message_preview(&event), "hello");
    }

    #[test]
    fn usage_last_message_preview_falls_back_for_blank_content() {
        let event = AdminLlmGatewayUsageEventView {
            last_message_content: Some("   ".to_string()),
            ..AdminLlmGatewayUsageEventView::default()
        };

        assert_eq!(usage_last_message_preview(&event), "-");
    }

    #[test]
    fn usage_last_message_table_preview_collapses_whitespace_and_truncates() {
        let event = AdminLlmGatewayUsageEventView {
            last_message_content: Some(
                "first line\n\nsecond   line with   extra spaces and a very long suffix that \
                 should be truncated in the table preview because it keeps going with more and \
                 more text until the shortened variant must end with ellipsis"
                    .to_string(),
            ),
            ..AdminLlmGatewayUsageEventView::default()
        };

        let preview = usage_last_message_table_preview(&event);

        assert!(!preview.contains('\n'));
        assert!(preview.contains("first line second line with extra spaces"));
        assert!(preview.ends_with("..."));
        assert!(preview.chars().count() <= 123);
    }

    #[test]
    fn usage_last_message_table_preview_keeps_short_single_line_text() {
        let event = AdminLlmGatewayUsageEventView {
            last_message_content: Some("short text".to_string()),
            ..AdminLlmGatewayUsageEventView::default()
        };

        assert_eq!(usage_last_message_table_preview(&event), "short text");
    }
}
