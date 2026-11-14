use std::collections::BTreeMap;

use gloo_timers::callback::Timeout;
use static_flow_shared::ArticleListItem;
use wasm_bindgen::JsCast;
use web_sys::{window, Event, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_router::prelude::{use_location, Link};

use crate::{
    api::{self, ArticleRequestItem},
    components::{
        article_card::ArticleCard, pagination::Pagination, raw_html::RawHtml,
        scroll_to_top_button::ScrollToTopButton, skeleton::SkeletonCard, toast::use_toast,
    },
    i18n::current::{article_request as ar_t, latest_articles_page as t},
    router::Route,
    utils::markdown_to_html,
};

const PAGE_SIZE: usize = 12;
const REQUEST_PAGE_SIZE: usize = 12;

fn article_request_has_article(req: &ArticleRequestItem) -> bool {
    req.ingested_article_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|id| !id.is_empty())
}

fn article_request_done_without_article(req: &ArticleRequestItem) -> bool {
    req.status == "done" && !article_request_has_article(req)
}

#[derive(Properties, PartialEq)]
struct RequestCardProps {
    pub req: ArticleRequestItem,
    pub on_follow_up: Callback<ArticleRequestItem>,
    #[prop_or_default]
    pub parent_summary: Option<AttrValue>,
}

#[function_component(RequestCard)]
fn request_card(props: &RequestCardProps) -> Html {
    let req = &props.req;
    let show_modal = use_state(|| false);

    // Status chips reuse the theme-aware `wish-status--*` palettes from
    // input.css so the label stays legible on its tint in light and dark mode.
    let (status_class, status_dot) = if article_request_done_without_article(req) {
        ("wish-status--pending", "bg-amber-500")
    } else {
        match req.status.as_str() {
            "done" => ("wish-status--done", "bg-emerald-500"),
            "running" => {
                ("wish-status--running", "bg-sky-500 animate-pulse motion-reduce:animate-none")
            },
            "failed" => ("wish-status--failed", "bg-red-500"),
            "rejected" => ("wish-status--default", "bg-[var(--muted)]"),
            _ => ("wish-status--pending", "bg-amber-500"),
        }
    };
    let status_chip_class = format!(
        "wish-status {status_class} inline-flex items-center gap-1.5 rounded-full border px-2.5 \
         py-1 text-xs font-medium leading-none"
    );
    let status_dot_class = format!("h-1.5 w-1.5 shrink-0 rounded-full {status_dot}");
    let follow_up_badge_class = "inline-flex items-center rounded-full border \
                                 border-[var(--primary)]/25 bg-[var(--primary)]/10 px-2 py-0.5 \
                                 text-[11px] font-medium leading-tight text-[var(--primary)]";
    let detail_label_class =
        "mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-[var(--muted)]";
    let card_action_class = "inline-flex items-center gap-1 rounded-md px-2 py-1 text-xs \
                             font-medium text-[var(--primary)] transition-colors \
                             duration-[var(--motion-fast)] ease-[var(--ease-snap)] \
                             hover:bg-[var(--primary)]/10";
    let status_text = if article_request_done_without_article(req) {
        ar_t::STATUS_DONE_NO_ARTICLE
    } else {
        match req.status.as_str() {
            "pending" => ar_t::STATUS_PENDING,
            "approved" => ar_t::STATUS_APPROVED,
            "running" => ar_t::STATUS_RUNNING,
            "done" => ar_t::STATUS_DONE,
            "failed" => ar_t::STATUS_FAILED,
            _ => &req.status,
        }
    };

    let open_modal = {
        let show_modal = show_modal.clone();
        Callback::from(move |_: MouseEvent| show_modal.set(true))
    };
    let close_modal = {
        let show_modal = show_modal.clone();
        Callback::from(move |_: MouseEvent| show_modal.set(false))
    };
    let stop_bubble = Callback::from(|e: MouseEvent| e.stop_propagation());

    let on_follow_up_click = {
        let cb = props.on_follow_up.clone();
        let req = req.clone();
        Callback::from(move |_: MouseEvent| cb.emit(req.clone()))
    };

    let has_ai_reply = req.status == "done" && req.ai_reply.is_some();

    // Render the detail modal
    let modal_html = if *show_modal {
        let ai_rendered = req.ai_reply.as_deref().map(markdown_to_html);
        html! {
            <div class="fixed inset-0 z-[200] flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
                 onclick={close_modal.clone()}>
                <div class="relative flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden \
                            rounded-[var(--radius-lg)] border border-[var(--border)] \
                            bg-[var(--surface)] shadow-[var(--shadow-16)]"
                     onclick={stop_bubble.clone()}>
                    // Header
                    <div class="flex shrink-0 items-center justify-between gap-4 \
                                border-b border-[var(--border)] px-5 py-4">
                        <h3 class="text-base font-semibold tracking-[-0.01em] text-[var(--text)]">
                            {ar_t::DETAIL_MODAL_TITLE}
                        </h3>
                        <button onclick={close_modal.clone()}
                            class="inline-flex h-8 w-8 items-center justify-center rounded-[var(--radius-sm)] \
                                   text-[var(--muted)] transition-colors duration-[var(--motion-fast)] \
                                   ease-[var(--ease-snap)] hover:bg-[var(--surface-alt)] hover:text-[var(--text)]">
                            <svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none"
                                 viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                    // Body
                    <div class="flex-1 space-y-5 overflow-y-auto px-5 py-5">
                        // Status + nickname row
                        <div class="flex flex-wrap items-center gap-2">
                            <span class={status_chip_class.clone()}>
                                <span class={status_dot_class.clone()} />
                                {status_text}
                            </span>
                            if req.parent_request_id.is_some() {
                                <span class={follow_up_badge_class}>
                                    {ar_t::FOLLOW_UP_BADGE}
                                </span>
                            }
                            <span class="ml-auto text-xs text-[var(--muted)]">{&req.nickname}</span>
                        </div>
                        if let Some(ref summary) = props.parent_summary {
                            <div class="flex items-center gap-2 rounded-[var(--radius)] border \
                                        border-[var(--primary)]/20 bg-[var(--primary)]/10 px-3 py-2 \
                                        text-xs text-[var(--primary)]">
                                <svg class="h-3.5 w-3.5 shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none"
                                     viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                          d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3" />
                                </svg>
                                <span>{summary}</span>
                            </div>
                        }
                        if article_request_done_without_article(req) {
                            <div class="rounded-[var(--radius)] border border-amber-500/30 bg-amber-500/10 \
                                        px-3 py-2 text-sm text-amber-800 dark:text-amber-200">
                                {ar_t::NO_ARTICLE_NOTICE}
                            </div>
                        }
                        // URL
                        <div>
                            <p class={detail_label_class}>
                                {ar_t::LABEL_URL}
                            </p>
                            <a href={req.article_url.clone()} target="_blank" rel="noopener noreferrer"
                                class="break-all text-sm text-[var(--link)] underline-offset-2 hover:underline">
                                {&req.article_url}
                            </a>
                        </div>
                        // Title hint
                        if let Some(ref hint) = req.title_hint {
                            <div>
                                <p class={detail_label_class}>
                                    {ar_t::LABEL_TITLE}
                                </p>
                                <p class="text-sm text-[var(--text)]">{hint}</p>
                            </div>
                        }
                        // Request message
                        <div>
                            <p class={detail_label_class}>
                                {ar_t::LABEL_MESSAGE}
                            </p>
                            <p class="whitespace-pre-wrap text-sm leading-relaxed text-[var(--text)]">{&req.request_message}</p>
                        </div>
                        // AI reply
                        if let Some(ref rendered) = ai_rendered {
                            <div>
                                <p class={detail_label_class}>
                                    {ar_t::LABEL_AI_REPLY}
                                </p>
                                <div class="prose prose-sm max-w-none rounded-[var(--radius)] border \
                                            border-[var(--border)] bg-[var(--surface-alt)] p-4 text-sm \
                                            text-[var(--text)] dark:prose-invert">
                                    <RawHtml html={AttrValue::from(rendered.clone())} />
                                </div>
                            </div>
                        }
                        // Ingested article link
                        if req.status == "done" {
                            if let Some(ref aid) = req.ingested_article_id {
                                if !aid.trim().is_empty() {
                                    <div>
                                    <Link<Route> to={Route::ArticleDetail { id: aid.clone() }}
                                        classes="inline-flex items-center gap-1.5 text-sm font-medium \
                                                 text-[var(--primary)] underline-offset-2 hover:underline">
                                        {ar_t::VIEW_ARTICLE}
                                        <i class="fas fa-arrow-right text-xs" aria-hidden="true"></i>
                                    </Link<Route>>
                                    </div>
                                }
                            }
                        }
                        // Region
                        <p class="border-t border-[var(--border)] pt-4 font-mono text-[11px] text-[var(--muted)]">
                            {ar_t::LABEL_REGION}{": "}{&req.ip_region}
                        </p>
                    </div>
                </div>
            </div>
        }
    } else {
        Html::default()
    };

    // Card body — compact summary
    html! {
        <>
        <div class="sf-panel flex flex-col gap-3 p-4 transition-[transform,box-shadow,border-color] \
                    duration-[var(--motion-base)] ease-[var(--ease-spring)] \
                    hover:-translate-y-0.5 hover:border-[var(--primary)]/40 hover:shadow-[var(--shadow-4)] \
                    motion-reduce:transition-none motion-reduce:hover:translate-y-0">
            // Row 1: status + badges + nickname
            <div class="flex items-center justify-between gap-2">
                <div class="flex min-w-0 flex-wrap items-center gap-1.5">
                    <span class={status_chip_class}>
                        <span class={status_dot_class} />
                        {status_text}
                    </span>
                    if req.parent_request_id.is_some() {
                        <span class={follow_up_badge_class}>
                            {ar_t::FOLLOW_UP_BADGE}
                        </span>
                    }
                </div>
                <span class="shrink-0 truncate text-xs text-[var(--muted)]">{&req.nickname}</span>
            </div>
            // Parent reference
            if let Some(ref summary) = props.parent_summary {
                <div class="flex items-center gap-1.5 rounded-[var(--radius-sm)] bg-[var(--primary)]/10 \
                            px-2 py-1 text-[11px] text-[var(--primary)]">
                    <svg class="h-3 w-3 shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none"
                         viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round"
                              d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3" />
                    </svg>
                    <span class="truncate">{summary}</span>
                </div>
            }
            // URL — full, no truncation
            <a href={req.article_url.clone()} target="_blank" rel="noopener noreferrer"
                class="break-all text-sm font-medium leading-snug text-[var(--link)] underline-offset-2 hover:underline">
                {&req.article_url}
            </a>
            // Title hint — full
            if let Some(ref hint) = req.title_hint {
                <p class="text-xs leading-relaxed text-[var(--muted)]">{hint}</p>
            }
            // Request message — full
            <p class="whitespace-pre-wrap text-sm leading-relaxed text-[var(--text)]">{&req.request_message}</p>
            // Ingested article link
            if req.status == "done" {
                if let Some(ref aid) = req.ingested_article_id {
                    if !aid.trim().is_empty() {
                        <Link<Route> to={Route::ArticleDetail { id: aid.clone() }}
                            classes="inline-flex items-center gap-1 self-start text-xs font-medium \
                                     text-[var(--primary)] underline-offset-2 hover:underline">
                            {ar_t::VIEW_ARTICLE}
                            <i class="fas fa-arrow-right text-[10px]" aria-hidden="true"></i>
                        </Link<Route>>
                    }
                }
            }
            // Footer: detail button + follow-up + region
            <div class="mt-auto flex items-center justify-between gap-2 border-t border-[var(--border)] pt-3">
                <div class="-ml-2 flex items-center gap-1">
                    if has_ai_reply {
                        <button onclick={open_modal} class={card_action_class}>
                            <svg class="h-3.5 w-3.5" xmlns="http://www.w3.org/2000/svg" fill="none"
                                 viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                      d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 \
                                         13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 \
                                         2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 \
                                         1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                            </svg>
                            {ar_t::VIEW_DETAIL_BTN}
                        </button>
                    }
                    if req.status == "done" {
                        <button onclick={on_follow_up_click} class={card_action_class}>
                            {ar_t::FOLLOW_UP_BTN}
                        </button>
                    }
                </div>
                // Region
                <span class="truncate font-mono text-[10px] text-[var(--muted)]">{&req.ip_region}</span>
            </div>
        </div>
        {modal_html}
        </>
    }
}

#[function_component(LatestArticlesPage)]
pub fn latest_articles_page() -> Html {
    let route_location = use_location();
    let articles = use_state(Vec::<ArticleListItem>::new);
    let loading = use_state(|| true);
    let current_page = use_state(|| 1_usize);
    let total = use_state(|| 0_usize);
    let fetch_seq = use_mut_ref(|| 0_u64);
    let load_error = use_state(|| None::<String>);
    let grid_refresh_tick = use_state(|| 0_u32);

    // Article request state
    let ar_requests = use_state(Vec::<ArticleRequestItem>::new);
    let ar_loading = use_state(|| false);
    let ar_page = use_state(|| 1_usize);
    let ar_total = use_state(|| 0_usize);
    let ar_list_error = use_state(|| None::<String>);
    let ar_form_url = use_state(String::new);
    let ar_form_title = use_state(String::new);
    let ar_form_message = use_state(String::new);
    let ar_form_nickname = use_state(String::new);
    let ar_form_email = use_state(String::new);
    let ar_submitting = use_state(|| false);
    let toast = use_toast();
    let ar_refresh_seq = use_state(|| 0_u32);
    let ar_parent_request = use_state(|| None::<ArticleRequestItem>);

    let total_pages = {
        let t = *total;
        if t == 0 {
            1
        } else {
            t.div_ceil(PAGE_SIZE)
        }
    };
    let current_page_num = *current_page;

    {
        let articles = articles.clone();
        let loading = loading.clone();
        let total = total.clone();
        let fetch_seq = fetch_seq.clone();
        let load_error = load_error.clone();
        let page = *current_page;
        let tick = *grid_refresh_tick;
        use_effect_with((page, tick), move |(page, _)| {
            let offset = (*page - 1) * PAGE_SIZE;
            let request_id = {
                let mut seq = fetch_seq.borrow_mut();
                *seq += 1;
                *seq
            };
            loading.set(true);
            load_error.set(None);
            let articles = articles.clone();
            let loading = loading.clone();
            let total = total.clone();
            let fetch_seq = fetch_seq.clone();
            let load_error = load_error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::fetch_articles(None, None, Some(PAGE_SIZE), Some(offset)).await {
                    Ok(data) => {
                        if *fetch_seq.borrow() != request_id {
                            return;
                        }
                        total.set(data.total);
                        articles.set(data.articles);
                        load_error.set(None);
                    },
                    Err(e) => {
                        if *fetch_seq.borrow() != request_id {
                            return;
                        }
                        web_sys::console::error_1(
                            &format!("Failed to fetch articles: {}", e).into(),
                        );
                        load_error.set(Some(e.to_string()));
                    },
                }
                if *fetch_seq.borrow() != request_id {
                    return;
                }
                loading.set(false);
            });
            || ()
        });
    }

    let save_scroll_position = {
        let location = route_location.clone();
        Callback::from(move |_| {
            if crate::navigation_context::is_return_armed() {
                return;
            }
            let mut state = BTreeMap::new();
            state.insert("page".to_string(), current_page_num.to_string());
            if let Some(loc) = location.as_ref() {
                state.insert("location".to_string(), loc.path().to_string());
            }
            crate::navigation_context::save_context_for_current_page(state);
        })
    };

    {
        let location = route_location.clone();
        use_effect_with((location, current_page_num), move |_| {
            let mut on_scroll_opt: Option<wasm_bindgen::closure::Closure<dyn FnMut(Event)>> = None;

            if !crate::navigation_context::is_return_armed() {
                let persist = move || {
                    let mut state = BTreeMap::new();
                    state.insert("page".to_string(), current_page_num.to_string());
                    crate::navigation_context::save_context_for_current_page(state);
                };

                persist();

                let on_scroll = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: Event| {
                    if crate::navigation_context::is_return_armed() {
                        return;
                    }
                    let mut state = BTreeMap::new();
                    state.insert("page".to_string(), current_page_num.to_string());
                    crate::navigation_context::save_context_for_current_page(state);
                })
                    as Box<dyn FnMut(_)>);

                if let Some(win) = window() {
                    let _ = win.add_event_listener_with_callback(
                        "scroll",
                        on_scroll.as_ref().unchecked_ref(),
                    );
                }

                on_scroll_opt = Some(on_scroll);
            }

            move || {
                if let Some(on_scroll) = on_scroll_opt {
                    if let Some(win) = window() {
                        let _ = win.remove_event_listener_with_callback(
                            "scroll",
                            on_scroll.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }

    {
        let current_page = current_page.clone();
        let location_dep = route_location.clone();
        let article_len = articles.len();
        use_effect_with((location_dep, article_len), move |_| {
            if article_len > 0 {
                if let Some(context) =
                    crate::navigation_context::pop_context_if_armed_for_current_page()
                {
                    if let Some(page_num) = context
                        .page_state
                        .get("page")
                        .and_then(|raw| raw.parse::<usize>().ok())
                    {
                        current_page.set(page_num);
                    }

                    let scroll_y = context.scroll_y.max(0.0);
                    Timeout::new(140, move || {
                        if let Some(win) = window() {
                            win.scroll_to_with_x_and_y(0.0, scroll_y);
                        }
                    })
                    .forget();
                }
            }
            || ()
        });
    }

    let go_to_page = {
        let current_page = current_page.clone();
        Callback::from(move |page: usize| {
            current_page.set(page);
        })
    };

    let pagination_controls = if total_pages > 1 {
        html! {
            <div class={classes!("mt-10", "flex", "justify-center")}>
                <Pagination
                    current_page={current_page_num}
                    total_pages={total_pages}
                    on_page_change={go_to_page.clone()}
                />
            </div>
        }
    } else {
        Html::default()
    };

    // Article request: fetch on page change
    let ar_total_pages = {
        let t = *ar_total;
        if t == 0 {
            1
        } else {
            t.div_ceil(REQUEST_PAGE_SIZE)
        }
    };
    {
        let ar_requests = ar_requests.clone();
        let ar_loading = ar_loading.clone();
        let ar_total = ar_total.clone();
        let ar_list_error = ar_list_error.clone();
        let page = *ar_page;
        let seq = *ar_refresh_seq;
        use_effect_with((page, seq), move |(page, _seq)| {
            let offset = (*page - 1) * REQUEST_PAGE_SIZE;
            ar_loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_article_requests(Some(REQUEST_PAGE_SIZE), Some(offset)).await {
                    Ok(data) => {
                        ar_total.set(data.total);
                        ar_requests.set(data.requests);
                        ar_list_error.set(None);
                    },
                    Err(e) => {
                        ar_list_error.set(Some(e));
                    },
                }
                ar_loading.set(false);
            });
            || ()
        });
    }

    let on_ar_page_change = {
        let ar_page = ar_page.clone();
        Callback::from(move |page: usize| ar_page.set(page))
    };

    let on_ar_refresh = {
        let ar_refresh_seq = ar_refresh_seq.clone();
        Callback::from(move |_: MouseEvent| {
            ar_refresh_seq.set(*ar_refresh_seq + 1);
        })
    };

    let on_follow_up = {
        let ar_parent_request = ar_parent_request.clone();
        let ar_form_url = ar_form_url.clone();
        Callback::from(move |req: ArticleRequestItem| {
            ar_form_url.set(req.article_url.clone());
            ar_parent_request.set(Some(req));
            // Scroll to form
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("article-request-section") {
                    el.scroll_into_view();
                }
            }
        })
    };

    let on_cancel_follow_up = {
        let ar_parent_request = ar_parent_request.clone();
        let ar_form_url = ar_form_url.clone();
        Callback::from(move |_: MouseEvent| {
            ar_parent_request.set(None);
            ar_form_url.set(String::new());
        })
    };

    let has_parent = ar_parent_request.is_some();

    let on_ar_submit = {
        let ar_form_url = ar_form_url.clone();
        let ar_form_title = ar_form_title.clone();
        let ar_form_message = ar_form_message.clone();
        let ar_form_nickname = ar_form_nickname.clone();
        let ar_form_email = ar_form_email.clone();
        let ar_submitting = ar_submitting.clone();
        let toast = toast.clone();
        let ar_page = ar_page.clone();
        let ar_parent_request = ar_parent_request.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let url = (*ar_form_url).clone();
            let title = (*ar_form_title).clone();
            let message = (*ar_form_message).clone();
            let nickname = (*ar_form_nickname).clone();
            let email = (*ar_form_email).clone();
            let parent_id = (*ar_parent_request).as_ref().map(|r| r.request_id.clone());
            let ar_submitting = ar_submitting.clone();
            let toast = toast.clone();
            let ar_form_url = ar_form_url.clone();
            let ar_form_title = ar_form_title.clone();
            let ar_form_message = ar_form_message.clone();
            let ar_page = ar_page.clone();
            let ar_parent_request = ar_parent_request.clone();
            ar_submitting.set(true);
            let frontend_page_url = web_sys::window().and_then(|w| w.location().href().ok());
            wasm_bindgen_futures::spawn_local(async move {
                let title_opt = if title.trim().is_empty() { None } else { Some(title.trim()) };
                let nick_opt =
                    if nickname.trim().is_empty() { None } else { Some(nickname.trim()) };
                let email_opt = if email.trim().is_empty() { None } else { Some(email.trim()) };
                match api::submit_article_request(
                    &url,
                    title_opt,
                    &message,
                    nick_opt,
                    email_opt,
                    frontend_page_url.as_deref(),
                    parent_id.as_deref(),
                )
                .await
                {
                    Ok(_) => {
                        toast.success(ar_t::SUBMIT_SUCCESS);
                        ar_form_url.set(String::new());
                        ar_form_title.set(String::new());
                        ar_form_message.set(String::new());
                        ar_page.set(1);
                        ar_parent_request.set(None);
                    },
                    Err(err) => {
                        toast.error(&err);
                    },
                }
                ar_submitting.set(false);
            });
        })
    };

    let scroll_to_request_section = Callback::from(|_: MouseEvent| {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(el) = doc.get_element_by_id("article-request-section") {
                el.scroll_into_view();
            }
        }
    });

    html! {
        <main class={classes!("pb-20")}>
            <div class={classes!("container")}>
                <header class="sf-page-hero">
                    <div class="sf-eyebrow">{ t::HERO_INDEX }</div>
                    <div class="flex flex-wrap items-end justify-between gap-4">
                        <div>
                            <h1 class="sf-page-title">{ t::HERO_TITLE }</h1>
                            <p class="sf-page-sub">{ t::HERO_DESC }</p>
                        </div>
                        <button
                            type="button"
                            onclick={scroll_to_request_section.clone()}
                            class="btn-terminal shrink-0"
                        >
                            <i class="fas fa-pen-nib text-xs text-[var(--primary)]" aria-hidden="true"></i>
                            { ar_t::SECTION_TITLE }
                        </button>
                    </div>
                </header>

                // Article Grid with Editorial Style
                {
                    if *loading {
                        html! {
                            <div class={classes!(
                                "grid", "grid-cols-1", "md:grid-cols-2", "lg:grid-cols-3",
                                "gap-6", "mb-12"
                            )}>
                                { for (0..6).map(|_| html! { <SkeletonCard /> }) }
                            </div>
                        }
                    } else if let Some(err) = (*load_error).clone() {
                        let on_retry = {
                            let grid_refresh_tick = grid_refresh_tick.clone();
                            Callback::from(move |_| grid_refresh_tick.set(*grid_refresh_tick + 1))
                        };
                        html! {
                            <div class="sf-state sf-state--error mb-12">
                                <span class="sf-state__icon">
                                    <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                                </span>
                                <p class="sf-state__title">{ t::LOAD_ERROR_TITLE }</p>
                                <p class="sf-state__body">{ err }</p>
                                <div class="sf-state__action">
                                    <button
                                        type="button"
                                        class="btn-fluent-secondary"
                                        onclick={on_retry}
                                    >
                                        <i class="fas fa-rotate-right text-xs" aria-hidden="true"></i>
                                        { t::RETRY }
                                    </button>
                                </div>
                            </div>
                        }
                    } else if articles.is_empty() {
                        html! {
                            <div class="sf-state mb-12">
                                <span class="sf-state__icon">
                                    <i class="fas fa-inbox" aria-hidden="true"></i>
                                </span>
                                <p class="sf-state__title">{ t::EMPTY }</p>
                            </div>
                        }
                    } else {
                        html! {
                            <>
                                <div class={classes!(
                                    "articles-grid",
                                    "grid",
                                    "grid-cols-1",
                                    "md:grid-cols-2",
                                    "lg:grid-cols-3",
                                    "gap-6",
                                    "mb-12"
                                )}>
                                    { for articles.iter().map(|article| {
                                        html! {
                                            <ArticleCard
                                                key={article.id.clone()}
                                                article={article.clone()}
                                                on_before_navigate={Some(save_scroll_position.clone())}
                                            />
                                        }
                                    }) }
                                </div>
                                { pagination_controls }
                            </>
                        }
                    }
                }
            </div>

            // Article request section
            <div class="container scroll-mt-24" id="article-request-section">
                <section class="mt-16 border-t border-[var(--border)] pt-10">
                    <div class="mb-6">
                        <h2 class="text-xl font-bold tracking-[-0.02em] text-[var(--text)] md:text-2xl">
                            {ar_t::SECTION_TITLE}
                        </h2>
                        <p class="mt-1 max-w-2xl text-sm leading-relaxed text-[var(--muted)]">{ar_t::SECTION_SUBTITLE}</p>
                    </div>

                    <form onsubmit={on_ar_submit}
                        class="sf-panel mb-8 grid grid-cols-1 gap-4 p-5 sm:grid-cols-2 sm:gap-5 sm:p-6">
                        if has_parent {
                            <div class="flex items-center justify-between gap-3 rounded-[var(--radius)] border \
                                        border-[var(--primary)]/20 bg-[var(--primary)]/10 px-3 py-2 sm:col-span-2">
                                <span class="inline-flex items-center gap-2 text-xs font-medium text-[var(--primary)]">
                                    <i class="fas fa-reply text-[10px]" aria-hidden="true"></i>
                                    {ar_t::FOLLOW_UP_INDICATOR}
                                </span>
                                <button type="button" onclick={on_cancel_follow_up.clone()}
                                    class="rounded-md px-2 py-1 text-xs text-[var(--muted)] transition-colors \
                                           duration-[var(--motion-fast)] ease-[var(--ease-snap)] \
                                           hover:bg-[var(--surface)] hover:text-[var(--text)]">
                                    {ar_t::CANCEL_FOLLOW_UP}
                                </button>
                            </div>
                        }
                        <div class="flex flex-col gap-1.5 sm:col-span-2">
                            <label for="ar-url" class="form-label">{ar_t::URL_LABEL}</label>
                            <input id="ar-url" type="url" placeholder={ar_t::URL_PLACEHOLDER}
                                value={(*ar_form_url).clone()}
                                oninput={let s = ar_form_url.clone(); Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    s.set(input.value());
                                })}
                                class="form-input"
                                required={!has_parent} />
                        </div>
                        <div class="flex flex-col gap-1.5">
                            <label for="ar-title" class="form-label">{ar_t::TITLE_HINT_LABEL}</label>
                            <input id="ar-title" type="text" placeholder={ar_t::TITLE_HINT_PLACEHOLDER}
                                value={(*ar_form_title).clone()}
                                oninput={let s = ar_form_title.clone(); Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    s.set(input.value());
                                })}
                                class="form-input" />
                        </div>
                        <div class="flex flex-col gap-1.5">
                            <label for="ar-nickname" class="form-label">{ar_t::NICKNAME_LABEL}</label>
                            <input id="ar-nickname" type="text" placeholder={ar_t::NICKNAME_PLACEHOLDER}
                                value={(*ar_form_nickname).clone()}
                                oninput={let s = ar_form_nickname.clone(); Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    s.set(input.value());
                                })}
                                class="form-input" />
                        </div>
                        <div class="flex flex-col gap-1.5 sm:col-span-2">
                            <label for="ar-message" class="form-label">{ar_t::MESSAGE_LABEL}</label>
                            <textarea id="ar-message" placeholder={ar_t::MESSAGE_PLACEHOLDER}
                                value={(*ar_form_message).clone()}
                                oninput={let s = ar_form_message.clone(); Callback::from(move |e: InputEvent| {
                                    let input: HtmlTextAreaElement = e.target_unchecked_into();
                                    s.set(input.value());
                                })}
                                rows="3"
                                class="form-input min-h-24 resize-y"
                                required=true />
                        </div>
                        <div class="flex flex-col gap-1.5">
                            <label for="ar-email" class="form-label">{ar_t::EMAIL_LABEL}</label>
                            <input id="ar-email" type="email" placeholder={ar_t::EMAIL_PLACEHOLDER}
                                value={(*ar_form_email).clone()}
                                oninput={let s = ar_form_email.clone(); Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    s.set(input.value());
                                })}
                                class="form-input" />
                            <p class="text-[11px] leading-relaxed text-[var(--muted)]">{ar_t::EMAIL_HELP_TEXT}</p>
                        </div>
                        <div class="flex items-end sm:justify-end">
                            <button type="submit" disabled={*ar_submitting}
                                class="btn-fluent-primary w-full disabled:cursor-not-allowed disabled:opacity-50 sm:w-auto">
                                if *ar_submitting {
                                    <span class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-white/40 \
                                                 border-t-white motion-reduce:animate-none" aria-hidden="true" />
                                } else {
                                    <i class="fas fa-paper-plane text-xs" aria-hidden="true"></i>
                                }
                                {if *ar_submitting { ar_t::SUBMITTING } else { ar_t::SUBMIT_BTN }}
                            </button>
                        </div>
                    </form>

                    // Refresh button
                    <div class="mb-4 flex justify-end">
                        <button
                            onclick={on_ar_refresh}
                            disabled={*ar_loading}
                            class="btn-fluent-secondary disabled:cursor-not-allowed disabled:opacity-50"
                        >
                            <svg class={if *ar_loading { "h-4 w-4 animate-spin motion-reduce:animate-none" } else { "h-4 w-4" }}
                                 xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
                                 stroke-width="2" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round"
                                      d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 \
                                         3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 \
                                         13.803-3.7l3.181 3.182" />
                            </svg>
                            {if *ar_loading { ar_t::REFRESHING } else { ar_t::REFRESH_BTN }}
                        </button>
                    </div>

                    if *ar_loading && ar_requests.is_empty() {
                        <div class="flex justify-center py-10">
                            <div class="h-6 w-6 animate-spin rounded-full border-2 border-[var(--border)] \
                                        border-t-[var(--primary)] motion-reduce:animate-none" />
                        </div>
                    } else if let Some(err) = (*ar_list_error).clone() {
                        <div class="sf-state sf-state--error">
                            <span class="sf-state__icon">
                                <i class="fas fa-triangle-exclamation" aria-hidden="true"></i>
                            </span>
                            <p class="sf-state__body">{format!("Failed to load requests: {err}")}</p>
                        </div>
                    } else if ar_requests.is_empty() {
                        <div class="sf-state">
                            <span class="sf-state__icon">
                                <i class="fas fa-inbox" aria-hidden="true"></i>
                            </span>
                            <p class="sf-state__body">{ar_t::EMPTY_LIST}</p>
                        </div>
                    } else {
                        <>
                            if *ar_loading {
                                <div class="mb-3 inline-flex items-center gap-2 text-xs text-[var(--muted)]">
                                    <div class="h-4 w-4 animate-spin rounded-full border-2 border-[var(--border)] \
                                                border-t-[var(--primary)] motion-reduce:animate-none" />
                                    <span>{"Loading..."}</span>
                                </div>
                            }
                            <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                                { for ar_requests.iter().map(|req| {
                                    let key = req.request_id.clone();
                                    let parent_summary = req.parent_request_id.as_ref().and_then(|pid| {
                                        ar_requests.iter().find(|r| r.request_id == *pid).map(|parent| {
                                            let label = parent.title_hint.as_deref()
                                                .filter(|s| !s.is_empty())
                                                .unwrap_or(&parent.article_url);
                                            let display: String = if label.chars().count() > 40 {
                                                format!("{}…", label.chars().take(39).collect::<String>())
                                            } else {
                                                label.to_string()
                                            };
                                            AttrValue::from(format!("{} {}", ar_t::FOLLOW_UP_REF_PREFIX, display))
                                        })
                                    });
                                    html! {
                                        <RequestCard
                                            key={key}
                                            req={req.clone()}
                                            on_follow_up={on_follow_up.clone()}
                                            parent_summary={parent_summary}
                                        />
                                    }
                                }) }
                            </div>
                        </>
                    }
                    if ar_total_pages > 1 {
                        <div class="mt-8 flex justify-center">
                            <Pagination
                                current_page={*ar_page}
                                total_pages={ar_total_pages}
                                on_page_change={on_ar_page_change.clone()}
                            />
                        </div>
                    }
                </section>
            </div>

            // Fixed nav button (left bottom) — icon with tooltip
            <button
                onclick={scroll_to_request_section}
                title={ar_t::NAV_BTN}
                class="group fixed bottom-20 left-4 z-50 flex h-10 w-10 items-center justify-center \
                       rounded-full bg-[var(--primary)] text-white shadow-[var(--shadow-8)] \
                       transition-transform duration-[var(--motion-fast)] ease-[var(--ease-snap)] \
                       hover:scale-105 active:scale-95 dark:bg-[#2563eb] \
                       motion-reduce:transition-none motion-reduce:hover:scale-100 motion-reduce:active:scale-100"
            >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none"
                     viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round"
                          d="M12 4.5v15m7.5-7.5h-15" />
                </svg>
                <span class="pointer-events-none absolute left-full ml-2 whitespace-nowrap \
                             rounded-[var(--radius-sm)] border border-[var(--border)] bg-[var(--surface)] \
                             px-2 py-1 text-xs text-[var(--text)] shadow-[var(--shadow-4)] opacity-0 \
                             transition-opacity duration-[var(--motion-fast)] ease-[var(--ease-snap)] \
                             group-hover:opacity-100 group-focus-visible:opacity-100">
                    {ar_t::NAV_BTN}
                </span>
            </button>

            <ScrollToTopButton />
        </main>
    }
}
