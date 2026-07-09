//! LLM gateway requests page (`/admin/llm-gateway/requests`).
//!
//! Owns the three public request queues (token wishes, account
//! contributions, sponsor requests): server-paginated lists with status
//! filters, and the approve / validate / reject / delete admin actions.
//! Extracted from the mega llm gateway panel.

use std::collections::HashSet;

use gloo_timers::callback::Timeout;
use web_sys::HtmlSelectElement;
use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_llm_gateway::{copy_icon_button, preview_text};
use crate::{
    api::{
        admin_approve_and_issue_llm_gateway_account_contribution_request,
        admin_approve_and_issue_llm_gateway_token_request,
        admin_approve_llm_gateway_sponsor_request,
        admin_reject_llm_gateway_account_contribution_request,
        admin_reject_llm_gateway_token_request,
        admin_validate_llm_gateway_account_contribution_request,
        delete_admin_llm_gateway_sponsor_request,
        fetch_admin_llm_gateway_account_contribution_requests,
        fetch_admin_llm_gateway_sponsor_requests, fetch_admin_llm_gateway_token_requests,
        AdminLlmGatewayAccountContributionRequestView,
        AdminLlmGatewayAccountContributionRequestsQuery, AdminLlmGatewaySponsorRequestView,
        AdminLlmGatewaySponsorRequestsQuery, AdminLlmGatewayTokenRequestView,
        AdminLlmGatewayTokenRequestsQuery,
    },
    components::{
        copy_button::copy_to_clipboard, empty_state::EmptyState, pagination::Pagination,
        status_badge::StatusBadge,
    },
    pages::llm_access_shared::{confirm_destructive, format_ms, format_number_u64},
    router::Route,
};

const TOKEN_REQUEST_PAGE_SIZE: usize = 20;
const ACCOUNT_CONTRIBUTION_REQUEST_PAGE_SIZE: usize = 20;
const SPONSOR_REQUEST_PAGE_SIZE: usize = 20;

fn copyable_token_preview(label: &str, value: &str, on_copy: &Callback<(String, String)>) -> Html {
    html! {
        <div class={classes!("rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2")}>
            <div class={classes!("flex", "items-center", "justify-between", "gap-3")}>
                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>
                    { label }
                </div>
                { copy_icon_button(value, on_copy) }
            </div>
            <code class={classes!("mt-2", "block", "break-all", "text-xs", "text-[var(--text)]")}>
                { preview_text(value, 96) }
            </code>
        </div>
    }
}

#[function_component(AdminLlmGatewayRequestsPage)]
pub fn admin_llm_gateway_requests_page() -> Html {
    let token_requests = use_state(Vec::<AdminLlmGatewayTokenRequestView>::new);
    let token_request_total = use_state(|| 0_usize);
    let token_request_page = use_state(|| 1_usize);
    let token_request_loading = use_state(|| false);
    let token_request_status_filter = use_state(String::new);
    let token_request_action_inflight = use_state(HashSet::<String>::new);
    let account_contribution_requests =
        use_state(Vec::<AdminLlmGatewayAccountContributionRequestView>::new);
    let account_contribution_request_total = use_state(|| 0_usize);
    let account_contribution_request_page = use_state(|| 1_usize);
    let account_contribution_request_loading = use_state(|| false);
    let account_contribution_request_status_filter = use_state(String::new);
    let account_contribution_request_action_inflight = use_state(HashSet::<String>::new);
    let sponsor_requests = use_state(Vec::<AdminLlmGatewaySponsorRequestView>::new);
    let sponsor_request_total = use_state(|| 0_usize);
    let sponsor_request_page = use_state(|| 1_usize);
    let sponsor_request_loading = use_state(|| false);
    let sponsor_request_status_filter = use_state(String::new);
    let sponsor_request_action_inflight = use_state(HashSet::<String>::new);
    let load_error = use_state(|| None::<String>);
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

    let on_copy = {
        let flash = flash.clone();
        Callback::from(move |(label, value): (String, String)| {
            copy_to_clipboard(&value);
            flash.emit((format!("已复制{}", label), false));
        })
    };

    let reload_token_requests = {
        let token_requests = token_requests.clone();
        let token_request_total = token_request_total.clone();
        let token_request_page = token_request_page.clone();
        let token_request_loading = token_request_loading.clone();
        let token_request_status_filter = token_request_status_filter.clone();
        let load_error = load_error.clone();
        Callback::from(move |(requested_page, override_status): (Option<usize>, Option<String>)| {
            let token_requests = token_requests.clone();
            let token_request_total = token_request_total.clone();
            let token_request_page = token_request_page.clone();
            let token_request_loading = token_request_loading.clone();
            let token_request_status_filter = token_request_status_filter.clone();
            let load_error = load_error.clone();
            let page = requested_page.unwrap_or(*token_request_page).max(1);
            let selected_status =
                override_status.unwrap_or_else(|| (*token_request_status_filter).clone());
            token_request_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let query = AdminLlmGatewayTokenRequestsQuery {
                    status: (!selected_status.is_empty()).then_some(selected_status),
                    limit: Some(TOKEN_REQUEST_PAGE_SIZE),
                    offset: Some((page - 1) * TOKEN_REQUEST_PAGE_SIZE),
                };
                match fetch_admin_llm_gateway_token_requests(&query).await {
                    Ok(resp) => {
                        token_request_total.set(resp.total);
                        token_requests.set(resp.requests);
                        token_request_page.set(page);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                token_request_loading.set(false);
            });
        })
    };

    let reload_account_contribution_requests = {
        let account_contribution_requests = account_contribution_requests.clone();
        let account_contribution_request_total = account_contribution_request_total.clone();
        let account_contribution_request_page = account_contribution_request_page.clone();
        let account_contribution_request_loading = account_contribution_request_loading.clone();
        let account_contribution_request_status_filter =
            account_contribution_request_status_filter.clone();
        let load_error = load_error.clone();
        Callback::from(move |(requested_page, override_status): (Option<usize>, Option<String>)| {
            let account_contribution_requests = account_contribution_requests.clone();
            let account_contribution_request_total = account_contribution_request_total.clone();
            let account_contribution_request_page = account_contribution_request_page.clone();
            let account_contribution_request_loading = account_contribution_request_loading.clone();
            let account_contribution_request_status_filter =
                account_contribution_request_status_filter.clone();
            let load_error = load_error.clone();
            let page = requested_page
                .unwrap_or(*account_contribution_request_page)
                .max(1);
            let selected_status = override_status
                .unwrap_or_else(|| (*account_contribution_request_status_filter).clone());
            account_contribution_request_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let query = AdminLlmGatewayAccountContributionRequestsQuery {
                    status: (!selected_status.is_empty()).then_some(selected_status),
                    limit: Some(ACCOUNT_CONTRIBUTION_REQUEST_PAGE_SIZE),
                    offset: Some((page - 1) * ACCOUNT_CONTRIBUTION_REQUEST_PAGE_SIZE),
                };
                match fetch_admin_llm_gateway_account_contribution_requests(&query).await {
                    Ok(resp) => {
                        account_contribution_request_total.set(resp.total);
                        account_contribution_requests.set(resp.requests);
                        account_contribution_request_page.set(page);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                account_contribution_request_loading.set(false);
            });
        })
    };

    let reload_sponsor_requests = {
        let sponsor_requests = sponsor_requests.clone();
        let sponsor_request_total = sponsor_request_total.clone();
        let sponsor_request_page = sponsor_request_page.clone();
        let sponsor_request_loading = sponsor_request_loading.clone();
        let sponsor_request_status_filter = sponsor_request_status_filter.clone();
        let load_error = load_error.clone();
        Callback::from(move |(requested_page, override_status): (Option<usize>, Option<String>)| {
            let sponsor_requests = sponsor_requests.clone();
            let sponsor_request_total = sponsor_request_total.clone();
            let sponsor_request_page = sponsor_request_page.clone();
            let sponsor_request_loading = sponsor_request_loading.clone();
            let sponsor_request_status_filter = sponsor_request_status_filter.clone();
            let load_error = load_error.clone();
            let page = requested_page.unwrap_or(*sponsor_request_page).max(1);
            let selected_status =
                override_status.unwrap_or_else(|| (*sponsor_request_status_filter).clone());
            sponsor_request_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let query = AdminLlmGatewaySponsorRequestsQuery {
                    status: (!selected_status.is_empty()).then_some(selected_status),
                    limit: Some(SPONSOR_REQUEST_PAGE_SIZE),
                    offset: Some((page - 1) * SPONSOR_REQUEST_PAGE_SIZE),
                };
                match fetch_admin_llm_gateway_sponsor_requests(&query).await {
                    Ok(resp) => {
                        sponsor_request_total.set(resp.total);
                        sponsor_requests.set(resp.requests);
                        sponsor_request_page.set(page);
                        load_error.set(None);
                    },
                    Err(err) => load_error.set(Some(err)),
                }
                sponsor_request_loading.set(false);
            });
        })
    };

    let on_reload = {
        let reload_token_requests = reload_token_requests.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        Callback::from(move |_: ()| {
            reload_token_requests.emit((None, None));
            reload_account_contribution_requests.emit((None, None));
            reload_sponsor_requests.emit((None, None));
        })
    };

    {
        let reload_token_requests = reload_token_requests.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        use_effect_with((), move |_| {
            reload_token_requests.emit((Some(1), Some(String::new())));
            reload_account_contribution_requests.emit((Some(1), Some(String::new())));
            reload_sponsor_requests.emit((Some(1), Some(String::new())));
            || ()
        });
    }

    let token_request_total_pages = (*token_request_total)
        .max(1)
        .div_ceil(TOKEN_REQUEST_PAGE_SIZE);
    let account_contribution_request_total_pages = (*account_contribution_request_total)
        .max(1)
        .div_ceil(ACCOUNT_CONTRIBUTION_REQUEST_PAGE_SIZE);
    let sponsor_request_total_pages = (*sponsor_request_total)
        .max(1)
        .div_ceil(SPONSOR_REQUEST_PAGE_SIZE);

    let on_token_request_status_filter_change = {
        let token_request_status_filter = token_request_status_filter.clone();
        let token_request_page = token_request_page.clone();
        let reload_token_requests = reload_token_requests.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                let status = target.value();
                token_request_status_filter.set(status.clone());
                token_request_page.set(1);
                reload_token_requests.emit((Some(1), Some(status)));
            }
        })
    };

    let on_token_request_page_change = {
        let token_request_page = token_request_page.clone();
        let reload_token_requests = reload_token_requests.clone();
        Callback::from(move |page: usize| {
            token_request_page.set(page);
            reload_token_requests.emit((Some(page), None));
        })
    };

    let on_approve_token_request = {
        let token_request_action_inflight = token_request_action_inflight.clone();
        let token_requests = token_requests.clone();
        let reload_token_requests = reload_token_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let token_request_action_inflight = token_request_action_inflight.clone();
            let token_requests = token_requests.clone();
            let reload_token_requests = reload_token_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*token_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                token_request_action_inflight.set(inflight);

                match admin_approve_and_issue_llm_gateway_token_request(&request_id, None).await {
                    Ok(updated) => {
                        let mut list = (*token_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        token_requests.set(list);
                        load_error.set(None);
                        reload_token_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*token_request_action_inflight).clone();
                inflight.remove(&request_id);
                token_request_action_inflight.set(inflight);
            });
        })
    };

    let on_reject_token_request = {
        let token_request_action_inflight = token_request_action_inflight.clone();
        let token_requests = token_requests.clone();
        let reload_token_requests = reload_token_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let token_request_action_inflight = token_request_action_inflight.clone();
            let token_requests = token_requests.clone();
            let reload_token_requests = reload_token_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*token_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                token_request_action_inflight.set(inflight);

                match admin_reject_llm_gateway_token_request(&request_id, None).await {
                    Ok(updated) => {
                        let mut list = (*token_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        token_requests.set(list);
                        load_error.set(None);
                        reload_token_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*token_request_action_inflight).clone();
                inflight.remove(&request_id);
                token_request_action_inflight.set(inflight);
            });
        })
    };

    let on_account_contribution_status_filter_change = {
        let account_contribution_request_status_filter =
            account_contribution_request_status_filter.clone();
        let account_contribution_request_page = account_contribution_request_page.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                let status = target.value();
                account_contribution_request_status_filter.set(status.clone());
                account_contribution_request_page.set(1);
                reload_account_contribution_requests.emit((Some(1), Some(status)));
            }
        })
    };

    let on_account_contribution_page_change = {
        let account_contribution_request_page = account_contribution_request_page.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        Callback::from(move |page: usize| {
            account_contribution_request_page.set(page);
            reload_account_contribution_requests.emit((Some(page), None));
        })
    };

    let on_validate_account_contribution_request = {
        let account_contribution_request_action_inflight =
            account_contribution_request_action_inflight.clone();
        let account_contribution_requests = account_contribution_requests.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let account_contribution_request_action_inflight =
                account_contribution_request_action_inflight.clone();
            let account_contribution_requests = account_contribution_requests.clone();
            let reload_account_contribution_requests = reload_account_contribution_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                account_contribution_request_action_inflight.set(inflight);

                match admin_validate_llm_gateway_account_contribution_request(&request_id, None)
                    .await
                {
                    Ok(updated) => {
                        let mut list = (*account_contribution_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        account_contribution_requests.set(list);
                        load_error.set(None);
                        reload_account_contribution_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.remove(&request_id);
                account_contribution_request_action_inflight.set(inflight);
            });
        })
    };

    let on_approve_account_contribution_request = {
        let account_contribution_request_action_inflight =
            account_contribution_request_action_inflight.clone();
        let account_contribution_requests = account_contribution_requests.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let account_contribution_request_action_inflight =
                account_contribution_request_action_inflight.clone();
            let account_contribution_requests = account_contribution_requests.clone();
            let reload_account_contribution_requests = reload_account_contribution_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                account_contribution_request_action_inflight.set(inflight);

                match admin_approve_and_issue_llm_gateway_account_contribution_request(
                    &request_id,
                    None,
                )
                .await
                {
                    Ok(updated) => {
                        let mut list = (*account_contribution_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        account_contribution_requests.set(list);
                        load_error.set(None);
                        reload_account_contribution_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.remove(&request_id);
                account_contribution_request_action_inflight.set(inflight);
            });
        })
    };

    let on_reject_account_contribution_request = {
        let account_contribution_request_action_inflight =
            account_contribution_request_action_inflight.clone();
        let account_contribution_requests = account_contribution_requests.clone();
        let reload_account_contribution_requests = reload_account_contribution_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let account_contribution_request_action_inflight =
                account_contribution_request_action_inflight.clone();
            let account_contribution_requests = account_contribution_requests.clone();
            let reload_account_contribution_requests = reload_account_contribution_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                account_contribution_request_action_inflight.set(inflight);

                match admin_reject_llm_gateway_account_contribution_request(&request_id, None).await
                {
                    Ok(updated) => {
                        let mut list = (*account_contribution_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        account_contribution_requests.set(list);
                        load_error.set(None);
                        reload_account_contribution_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*account_contribution_request_action_inflight).clone();
                inflight.remove(&request_id);
                account_contribution_request_action_inflight.set(inflight);
            });
        })
    };

    let on_sponsor_request_status_filter_change = {
        let sponsor_request_status_filter = sponsor_request_status_filter.clone();
        let sponsor_request_page = sponsor_request_page.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        Callback::from(move |event: Event| {
            if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                let status = target.value();
                sponsor_request_status_filter.set(status.clone());
                sponsor_request_page.set(1);
                reload_sponsor_requests.emit((Some(1), Some(status)));
            }
        })
    };

    let on_sponsor_request_page_change = {
        let sponsor_request_page = sponsor_request_page.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        Callback::from(move |page: usize| {
            sponsor_request_page.set(page);
            reload_sponsor_requests.emit((Some(page), None));
        })
    };

    let on_approve_sponsor_request = {
        let sponsor_request_action_inflight = sponsor_request_action_inflight.clone();
        let sponsor_requests = sponsor_requests.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            let sponsor_request_action_inflight = sponsor_request_action_inflight.clone();
            let sponsor_requests = sponsor_requests.clone();
            let reload_sponsor_requests = reload_sponsor_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*sponsor_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                sponsor_request_action_inflight.set(inflight);

                match admin_approve_llm_gateway_sponsor_request(&request_id, None).await {
                    Ok(updated) => {
                        let mut list = (*sponsor_requests).clone();
                        if let Some(item) = list
                            .iter_mut()
                            .find(|item| item.request_id == updated.request_id)
                        {
                            *item = updated;
                        }
                        sponsor_requests.set(list);
                        load_error.set(None);
                        reload_sponsor_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*sponsor_request_action_inflight).clone();
                inflight.remove(&request_id);
                sponsor_request_action_inflight.set(inflight);
            });
        })
    };

    let on_delete_sponsor_request = {
        let sponsor_request_action_inflight = sponsor_request_action_inflight.clone();
        let sponsor_requests = sponsor_requests.clone();
        let sponsor_request_total = sponsor_request_total.clone();
        let reload_sponsor_requests = reload_sponsor_requests.clone();
        let load_error = load_error.clone();
        Callback::from(move |request_id: String| {
            if !confirm_destructive("确认删除这条 Sponsor 请求？") {
                return;
            }

            let sponsor_request_action_inflight = sponsor_request_action_inflight.clone();
            let sponsor_requests = sponsor_requests.clone();
            let sponsor_request_total = sponsor_request_total.clone();
            let reload_sponsor_requests = reload_sponsor_requests.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut inflight = (*sponsor_request_action_inflight).clone();
                inflight.insert(request_id.clone());
                sponsor_request_action_inflight.set(inflight);

                match delete_admin_llm_gateway_sponsor_request(&request_id).await {
                    Ok(_) => {
                        let filtered = (*sponsor_requests)
                            .iter()
                            .filter(|item| item.request_id != request_id)
                            .cloned()
                            .collect::<Vec<_>>();
                        sponsor_requests.set(filtered);
                        sponsor_request_total.set((*sponsor_request_total).saturating_sub(1));
                        load_error.set(None);
                        reload_sponsor_requests.emit((None, None));
                    },
                    Err(err) => load_error.set(Some(err)),
                }

                let mut inflight = (*sponsor_request_action_inflight).clone();
                inflight.remove(&request_id);
                sponsor_request_action_inflight.set(inflight);
            });
        })
    };

    let requests_loading =
        *token_request_loading || *account_contribution_request_loading || *sponsor_request_loading;

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "LLM Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Requests" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminLlmGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                        <button type="button" class={classes!("primary")} disabled={requests_loading} onclick={{
                            let on_reload = on_reload.clone();
                            Callback::from(move |_| on_reload.emit(()))
                        }}>
                            { if requests_loading { "刷新中..." } else { "刷新" } }
                        </button>
                    </div>
                </header>

                if let Some(err) = (*load_error).clone() {
                    <div class={classes!("errorline", "text-sm")}>{ err }</div>
                }

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Token Wishes" }</h2>
                            <p class={classes!("mt-1", "m-0", "text-xs", "text-[var(--muted)]")}>
                                { "只有在这里审核通过后，系统才会真正创建 key 并通过邮件发给申请人。" }
                            </p>
                        </div>
                        <button
                            class={classes!("ghost")}
                            onclick={{
                                let reload_token_requests = reload_token_requests.clone();
                                Callback::from(move |_| reload_token_requests.emit((None, None)))
                            }}
                            disabled={*token_request_loading}
                        >
                            <i class={classes!("fas", if *token_request_loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                        </button>
                    </div>

                    <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-[minmax(0,16rem)_auto]")}>
                        <label class={classes!("text-sm")}>
                            <span class={classes!("text-[var(--muted)]")}>{ "状态" }</span>
                            <select
                                key={format!("token-request-filter-{}", (*token_request_status_filter).clone())}
                                class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                onchange={on_token_request_status_filter_change}
                            >
                                <option value="" selected={(*token_request_status_filter).is_empty()}>{ "全部" }</option>
                                <option value="pending" selected={*token_request_status_filter == "pending"}>{ "pending" }</option>
                                <option value="failed" selected={*token_request_status_filter == "failed"}>{ "failed" }</option>
                                <option value="issued" selected={*token_request_status_filter == "issued"}>{ "issued" }</option>
                                <option value="rejected" selected={*token_request_status_filter == "rejected"}>{ "rejected" }</option>
                            </select>
                        </label>
                    </div>

                    if token_requests.is_empty() && !*token_request_loading {
                        <div class={classes!("mt-4")}>
                            <EmptyState icon="fa-inbox" title="当前筛选下还没有 token 许愿。" />
                        </div>
                    } else {
                        <div class={classes!("mt-4", "space-y-3")}>
                            { for token_requests.iter().map(|item| {
                                let request_id = item.request_id.clone();
                                let approve_request_id = item.request_id.clone();
                                let reject_request_id = item.request_id.clone();
                                let approve_cb = on_approve_token_request.clone();
                                let reject_cb = on_reject_token_request.clone();
                                let action_busy = token_request_action_inflight.contains(&request_id);
                                html! {
                                    <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
                                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("min-w-0", "space-y-1")}>
                                                <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                                    <StatusBadge status={item.status.clone()} />
                                                    <span class={classes!("font-semibold")}>{ item.requester_email.clone() }</span>
                                                    <span class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>{ item.request_id.clone() }</span>
                                                </div>
                                                <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                    { format!("{} / {} · created {}", item.client_ip, item.ip_region, format_ms(item.created_at)) }
                                                </div>
                                            </div>
                                            <div class={classes!("text-right")}>
                                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "申请 token" }</div>
                                                <div class={classes!("mt-1", "font-mono", "text-2xl", "font-black")}>{ format_number_u64(item.requested_quota_billable_limit) }</div>
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]")}>
                                            <div>
                                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "缘由" }</div>
                                                <div class={classes!("mt-2", "whitespace-pre-wrap", "break-words", "text-sm", "leading-6", "text-[var(--text)]")}>
                                                    { item.request_reason.clone() }
                                                </div>
                                            </div>
                                            <div class={classes!("space-y-2", "text-sm")}>
                                                if let Some(frontend_page_url) = item.frontend_page_url.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "页面" }</div>
                                                        <div class={classes!("mt-1", "break-all", "text-[var(--text)]")}>{ frontend_page_url }</div>
                                                    </div>
                                                }
                                                if let Some(issued_key_name) = item.issued_key_name.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "已发放 Key" }</div>
                                                        <div class={classes!("mt-1", "text-[var(--text)]")}>
                                                            { format!("{} ({})", issued_key_name, item.issued_key_id.clone().unwrap_or_default()) }
                                                        </div>
                                                    </div>
                                                }
                                                if let Some(admin_note) = item.admin_note.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Admin Note" }</div>
                                                        <div class={classes!("mt-1", "whitespace-pre-wrap", "break-words", "text-[var(--text)]")}>{ admin_note }</div>
                                                    </div>
                                                }
                                                if let Some(failure_reason) = item.failure_reason.clone() {
                                                    <div class={classes!("rounded-lg", "border", "border-red-400/25", "bg-red-500/8", "px-3", "py-2", "text-red-700", "dark:text-red-200")}>
                                                        { failure_reason }
                                                    </div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                { item.processed_at.map(format_ms).map(|value| format!("processed {}", value)).unwrap_or_else(|| "尚未处理".to_string()) }
                                            </div>
                                            <div class={classes!("flex", "items-center", "gap-2")}>
                                                if item.status == "pending" || item.status == "failed" {
                                                    <button
                                                        class={classes!("primary")}
                                                        onclick={Callback::from(move |_| approve_cb.emit(approve_request_id.clone()))}
                                                        disabled={action_busy}
                                                    >
                                                        { if action_busy { "处理中..." } else { "批准并发放" } }
                                                    </button>
                                                }
                                                if item.status == "pending" || item.status == "failed" {
                                                    <button
                                                        class={classes!("ghost", "btn-terminal-danger")}
                                                        onclick={Callback::from(move |_| reject_cb.emit(reject_request_id.clone()))}
                                                        disabled={action_busy}
                                                    >
                                                        { "拒绝" }
                                                    </button>
                                                }
                                            </div>
                                        </div>
                                    </article>
                                }
                            }) }
                        </div>
                    }

                    <div class={classes!("mt-5")}>
                        <Pagination current_page={*token_request_page} total_pages={token_request_total_pages} on_page_change={on_token_request_page_change} />
                    </div>
                </section>

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Account Contributions" }</h2>
                                <p class={classes!("mt-1", "m-0", "text-xs", "text-[var(--muted)]")}>
                                    { "公开页提交的 Codex 账号贡献申请会先进入这里；先验证 auth refresh，validated 后才能入库并发放绑定该账号路由的 token。" }
                                </p>
                        </div>
                        <button
                            class={classes!("ghost")}
                            onclick={{
                                let reload_account_contribution_requests = reload_account_contribution_requests.clone();
                                Callback::from(move |_| reload_account_contribution_requests.emit((None, None)))
                            }}
                            disabled={*account_contribution_request_loading}
                        >
                            <i class={classes!("fas", if *account_contribution_request_loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                        </button>
                    </div>

                    <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-[minmax(0,16rem)_auto]")}>
                        <label class={classes!("text-sm")}>
                            <span class={classes!("text-[var(--muted)]")}>{ "状态" }</span>
                            <select
                                key={format!("account-contribution-filter-{}", (*account_contribution_request_status_filter).clone())}
                                class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                onchange={on_account_contribution_status_filter_change}
                            >
                                <option value="" selected={(*account_contribution_request_status_filter).is_empty()}>{ "全部" }</option>
                                    <option value="pending" selected={*account_contribution_request_status_filter == "pending"}>{ "pending" }</option>
                                    <option value="validated" selected={*account_contribution_request_status_filter == "validated"}>{ "validated" }</option>
                                    <option value="failed" selected={*account_contribution_request_status_filter == "failed"}>{ "failed" }</option>
                                <option value="issued" selected={*account_contribution_request_status_filter == "issued"}>{ "issued" }</option>
                                <option value="rejected" selected={*account_contribution_request_status_filter == "rejected"}>{ "rejected" }</option>
                            </select>
                        </label>
                    </div>

                    if account_contribution_requests.is_empty() && !*account_contribution_request_loading {
                        <div class={classes!("mt-4")}>
                            <EmptyState icon="fa-inbox" title="当前筛选下还没有账号贡献申请。" />
                        </div>
                    } else {
                        <div class={classes!("mt-4", "space-y-3")}>
                            { for account_contribution_requests.iter().map(|item| {
                                    let request_id = item.request_id.clone();
                                    let validate_request_id = item.request_id.clone();
                                    let approve_request_id = item.request_id.clone();
                                    let reject_request_id = item.request_id.clone();
                                    let validate_cb = on_validate_account_contribution_request.clone();
                                    let approve_cb = on_approve_account_contribution_request.clone();
                                let reject_cb = on_reject_account_contribution_request.clone();
                                let on_copy = on_copy.clone();
                                let action_busy =
                                    account_contribution_request_action_inflight.contains(&request_id);
                                html! {
                                    <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
                                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("min-w-0", "space-y-1")}>
                                                <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                                    <StatusBadge status={item.status.clone()} />
                                                    <span class={classes!("font-semibold")}>{ item.account_name.clone() }</span>
                                                        if !item.requester_email.trim().is_empty() {
                                                            <span class={classes!("text-xs", "text-[var(--muted)]")}>{ item.requester_email.clone() }</span>
                                                        }
                                                    <span class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>{ item.request_id.clone() }</span>
                                                </div>
                                                <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                    { format!("{} / {} · created {}", item.client_ip, item.ip_region, format_ms(item.created_at)) }
                                                </div>
                                            </div>
                                            <div class={classes!("text-right", "space-y-1")}>
                                                if let Some(github_id) = item.github_id.clone() {
                                                    <div class={classes!("text-sm", "font-semibold")}>{ format!("@{}", github_id) }</div>
                                                }
                                                if let Some(account_id) = item.account_id.clone() {
                                                    <div class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>{ account_id }</div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]")}>
                                            <div>
                                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "留言" }</div>
                                                <div class={classes!("mt-2", "whitespace-pre-wrap", "break-words", "text-sm", "leading-6", "text-[var(--text)]")}>
                                                    { item.contributor_message.clone() }
                                                </div>
                                            </div>
                                            <div class={classes!("space-y-2", "text-sm")}>
                                                if let Some(frontend_page_url) = item.frontend_page_url.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "页面" }</div>
                                                        <div class={classes!("mt-1", "break-all", "text-[var(--text)]")}>{ frontend_page_url }</div>
                                                    </div>
                                                }
                                                if let Some(imported_account_name) = item.imported_account_name.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "已导入账号" }</div>
                                                        <div class={classes!("mt-1", "text-[var(--text)]")}>{ imported_account_name }</div>
                                                    </div>
                                                }
                                                if let Some(issued_key_name) = item.issued_key_name.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "已发放 Key" }</div>
                                                        <div class={classes!("mt-1", "text-[var(--text)]")}>
                                                            { format!("{} ({})", issued_key_name, item.issued_key_id.clone().unwrap_or_default()) }
                                                        </div>
                                                    </div>
                                                }
                                                if let Some(admin_note) = item.admin_note.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Admin Note" }</div>
                                                        <div class={classes!("mt-1", "whitespace-pre-wrap", "break-words", "text-[var(--text)]")}>{ admin_note }</div>
                                                    </div>
                                                }
                                                if let Some(failure_reason) = item.failure_reason.clone() {
                                                    <div class={classes!("rounded-lg", "border", "border-red-400/25", "bg-red-500/8", "px-3", "py-2", "text-red-700", "dark:text-red-200")}>
                                                        { failure_reason }
                                                    </div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-3")}>
                                            { copyable_token_preview("access_token", &item.access_token, &on_copy) }
                                            { copyable_token_preview("id_token", &item.id_token, &on_copy) }
                                            { copyable_token_preview("refresh_token", &item.refresh_token, &on_copy) }
                                        </div>

                                        <div class={classes!("mt-4", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                { item.processed_at.map(format_ms).map(|value| format!("processed {}", value)).unwrap_or_else(|| "尚未处理".to_string()) }
                                            </div>
                                            <div class={classes!("flex", "items-center", "gap-2")}>
                                                    if item.status == "pending" || item.status == "failed" {
                                                        <button
                                                            class={classes!("primary")}
                                                            onclick={Callback::from(move |_| validate_cb.emit(validate_request_id.clone()))}
                                                            disabled={action_busy}
                                                        >
                                                            { if action_busy { "验证中..." } else { "验证" } }
                                                        </button>
                                                    }
                                                    if item.status == "validated" {
                                                        <button
                                                            class={classes!("primary")}
                                                            onclick={Callback::from(move |_| approve_cb.emit(approve_request_id.clone()))}
                                                            disabled={action_busy}
                                                        >
                                                            { if action_busy { "入库中..." } else { "入库并发放" } }
                                                        </button>
                                                    }
                                                if item.status == "pending" || item.status == "failed" {
                                                    <button
                                                        class={classes!("ghost", "btn-terminal-danger")}
                                                        onclick={Callback::from(move |_| reject_cb.emit(reject_request_id.clone()))}
                                                        disabled={action_busy}
                                                    >
                                                        { "拒绝" }
                                                    </button>
                                                }
                                            </div>
                                        </div>
                                    </article>
                                }
                            }) }
                        </div>
                    }

                    <div class={classes!("mt-5")}>
                        <Pagination
                            current_page={*account_contribution_request_page}
                            total_pages={account_contribution_request_total_pages}
                            on_page_change={on_account_contribution_page_change}
                        />
                    </div>
                </section>

                <section class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-5")}>
                    <div class={classes!("flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                        <div>
                            <h2 class={classes!("m-0", "font-mono", "text-base", "font-bold", "text-[var(--text)]")}>{ "Sponsors" }</h2>
                            <p class={classes!("mt-1", "m-0", "text-xs", "text-[var(--muted)]")}>
                                { "这批请求是「先填邮箱，再发付款说明邮件」的人工确认流。你确认对方已经按邮件说明完成赞助后，再在这里标记通过。" }
                            </p>
                        </div>
                        <button
                            class={classes!("ghost")}
                            onclick={{
                                let reload_sponsor_requests = reload_sponsor_requests.clone();
                                Callback::from(move |_| reload_sponsor_requests.emit((None, None)))
                            }}
                            disabled={*sponsor_request_loading}
                        >
                            <i class={classes!("fas", if *sponsor_request_loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })}></i>
                        </button>
                    </div>

                    <div class={classes!("mt-3", "grid", "gap-3", "md:grid-cols-[minmax(0,16rem)_auto]")}>
                        <label class={classes!("text-sm")}>
                            <span class={classes!("text-[var(--muted)]")}>{ "状态" }</span>
                            <select
                                key={format!("sponsor-filter-{}", (*sponsor_request_status_filter).clone())}
                                class={classes!("mt-1", "w-full", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface)]", "px-3", "py-2")}
                                onchange={on_sponsor_request_status_filter_change}
                            >
                                <option value="" selected={(*sponsor_request_status_filter).is_empty()}>{ "全部" }</option>
                                <option value="submitted" selected={*sponsor_request_status_filter == "submitted"}>{ "submitted" }</option>
                                <option value="payment_email_sent" selected={*sponsor_request_status_filter == "payment_email_sent"}>{ "payment_email_sent" }</option>
                                <option value="approved" selected={*sponsor_request_status_filter == "approved"}>{ "approved" }</option>
                            </select>
                        </label>
                    </div>

                    if sponsor_requests.is_empty() && !*sponsor_request_loading {
                        <div class={classes!("mt-4")}>
                            <EmptyState icon="fa-inbox" title="当前筛选下还没有 Sponsor 请求。" />
                        </div>
                    } else {
                        <div class={classes!("mt-4", "space-y-3")}>
                            { for sponsor_requests.iter().map(|item| {
                                let request_id = item.request_id.clone();
                                let approve_request_id = item.request_id.clone();
                                let delete_request_id = item.request_id.clone();
                                let approve_cb = on_approve_sponsor_request.clone();
                                let delete_cb = on_delete_sponsor_request.clone();
                                let action_busy = sponsor_request_action_inflight.contains(&request_id);
                                html! {
                                    <article class={classes!("rounded-xl", "border", "border-[var(--border)]", "bg-[var(--surface)]", "p-4")}>
                                        <div class={classes!("flex", "items-start", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("min-w-0", "space-y-1")}>
                                                <div class={classes!("flex", "items-center", "gap-2", "flex-wrap")}>
                                                    <StatusBadge status={item.status.clone()} />
                                                    <span class={classes!("font-semibold")}>{ item.requester_email.clone() }</span>
                                                    <span class={classes!("text-xs", "font-mono", "text-[var(--muted)]")}>{ item.request_id.clone() }</span>
                                                </div>
                                                <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                    { format!("{} / {} · created {}", item.client_ip, item.ip_region, format_ms(item.created_at)) }
                                                </div>
                                            </div>
                                            <div class={classes!("text-right", "space-y-1")}>
                                                if let Some(display_name) = item.display_name.clone() {
                                                    <div class={classes!("text-sm", "font-semibold")}>{ display_name }</div>
                                                }
                                                if let Some(github_id) = item.github_id.clone() {
                                                    <div class={classes!("text-xs", "font-semibold", "text-[var(--muted)]")}>{ format!("@{}", github_id) }</div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "grid", "gap-3", "xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]")}>
                                            <div>
                                                <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "留言" }</div>
                                                <div class={classes!("mt-2", "whitespace-pre-wrap", "break-words", "text-sm", "leading-6", "text-[var(--text)]")}>
                                                    { item.sponsor_message.clone() }
                                                </div>
                                            </div>
                                            <div class={classes!("space-y-2", "text-sm")}>
                                                if let Some(frontend_page_url) = item.frontend_page_url.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "页面" }</div>
                                                        <div class={classes!("mt-1", "break-all", "text-[var(--text)]")}>{ frontend_page_url }</div>
                                                    </div>
                                                }
                                                if let Some(payment_email_sent_at) = item.payment_email_sent_at {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "付款说明邮件" }</div>
                                                        <div class={classes!("mt-1", "text-[var(--text)]")}>{ format_ms(payment_email_sent_at) }</div>
                                                    </div>
                                                }
                                                if let Some(admin_note) = item.admin_note.clone() {
                                                    <div>
                                                        <div class={classes!("text-xs", "uppercase", "tracking-widest", "text-[var(--muted)]")}>{ "Admin Note" }</div>
                                                        <div class={classes!("mt-1", "whitespace-pre-wrap", "break-words", "text-[var(--text)]")}>{ admin_note }</div>
                                                    </div>
                                                }
                                                if let Some(failure_reason) = item.failure_reason.clone() {
                                                    <div class={classes!("rounded-lg", "border", "border-red-400/25", "bg-red-500/8", "px-3", "py-2", "text-red-700", "dark:text-red-200")}>
                                                        { failure_reason }
                                                    </div>
                                                }
                                            </div>
                                        </div>

                                        <div class={classes!("mt-4", "flex", "items-center", "justify-between", "gap-3", "flex-wrap")}>
                                            <div class={classes!("text-xs", "text-[var(--muted)]")}>
                                                { item.processed_at.map(format_ms).map(|value| format!("processed {}", value)).unwrap_or_else(|| "尚未确认".to_string()) }
                                            </div>
                                            <div class={classes!("flex", "items-center", "gap-2")}>
                                                if item.status != "approved" {
                                                    <button
                                                        class={classes!("primary")}
                                                        onclick={Callback::from(move |_| approve_cb.emit(approve_request_id.clone()))}
                                                        disabled={action_busy}
                                                    >
                                                        { if action_busy { "处理中..." } else { "标记已确认" } }
                                                    </button>
                                                }
                                                <button
                                                    class={classes!("ghost", "btn-terminal-danger")}
                                                    onclick={Callback::from(move |_| delete_cb.emit(delete_request_id.clone()))}
                                                    disabled={action_busy}
                                                >
                                                    { "删除" }
                                                </button>
                                            </div>
                                        </div>
                                    </article>
                                }
                            }) }
                        </div>
                    }

                    <div class={classes!("mt-5")}>
                        <Pagination
                            current_page={*sponsor_request_page}
                            total_pages={sponsor_request_total_pages}
                            on_page_change={on_sponsor_request_page_change}
                        />
                    </div>
                </section>
            </div>
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
