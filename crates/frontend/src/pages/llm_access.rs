use gloo_timers::callback::Timeout;
use serde::Deserialize;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlInputElement, HtmlTextAreaElement, KeyboardEvent};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    api::{
        fetch_llm_gateway_access, fetch_llm_gateway_public_page,
        submit_gpt2api_account_contribution_request,
        submit_llm_gateway_account_contribution_request, submit_llm_gateway_sponsor_request,
        submit_llm_gateway_token_request, LlmGatewayAccessResponse, LlmGatewayPublicKeyView,
        LlmGatewaySupportConfigView, PublicLlmGatewayAccountContributionView,
        PublicLlmGatewaySponsorView, SubmitGpt2ApiAccountContributionInput,
        SubmitLlmGatewayAccountContributionInput, SubmitLlmGatewaySponsorInput, API_BASE,
    },
    pages::llm_access_shared::{
        format_ms, format_number_i64, format_number_u64, resolved_base_url, usage_ratio,
        MaskedSecretCode, REMOTE_COMPACT_ARTICLE_ID,
    },
    router::Route,
};

#[wasm_bindgen(inline_js = r#"
export function copy_text(text) {
    if (navigator.clipboard) {
        navigator.clipboard.writeText(text).catch(function(){});
    }
}
"#)]
extern "C" {
    fn copy_text(text: &str);
}

fn github_avatar_url(github_id: &str) -> String {
    format!("https://github.com/{}.png?size=96", github_id.trim())
}

fn github_profile_url(github_id: &str) -> String {
    format!("https://github.com/{}", github_id.trim())
}
// PLACEHOLDER_RESOLVE_SUPPORT_ASSET_URL

fn resolve_support_asset_url(path_or_url: &str) -> String {
    let normalized = path_or_url.trim();
    if normalized.starts_with("http://")
        || normalized.starts_with("https://")
        || normalized.starts_with("data:")
    {
        normalized.to_string()
    } else if normalized.starts_with("/api/") {
        format!("{}{}", API_BASE.trim_end_matches("/api"), normalized)
    } else {
        normalized.to_string()
    }
}


#[derive(Clone, PartialEq)]
enum ActiveModal {
    None,
    TokenWish,
    AccountContribution,
    GptAccountContribution,
    Sponsor,
}

#[derive(Debug, Deserialize, Default)]
struct ImportedCodexAuthTokens {
    #[serde(default, alias = "idToken")]
    id_token: Option<String>,
    #[serde(default, alias = "accessToken")]
    access_token: Option<String>,
    #[serde(default, alias = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(default, alias = "accountId")]
    account_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ImportedCodexAuthFile {
    #[serde(default)]
    tokens: Option<ImportedCodexAuthTokens>,
    #[serde(default, alias = "idToken")]
    id_token: Option<String>,
    #[serde(default, alias = "accessToken")]
    access_token: Option<String>,
    #[serde(default, alias = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(default, alias = "accountId")]
    account_id: Option<String>,
}

struct ParsedImportedAuthJson {
    id_token: String,
    access_token: String,
    refresh_token: String,
    account_id: Option<String>,
}

fn parse_imported_auth_json(raw: &str) -> Result<ParsedImportedAuthJson, String> {
    let parsed: ImportedCodexAuthFile =
        serde_json::from_str(raw).map_err(|_| "auth.json 不是合法 JSON".to_string())?;
    let tokens = parsed.tokens.unwrap_or_default();
    let id_token = tokens
        .id_token
        .or(parsed.id_token)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    let access_token = tokens
        .access_token
        .or(parsed.access_token)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    let refresh_token = tokens
        .refresh_token
        .or(parsed.refresh_token)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    if id_token.is_empty() && access_token.is_empty() && refresh_token.is_empty() {
        return Err("auth.json 没有识别到可用 token 字段".to_string());
    }
    let account_id = tokens
        .account_id
        .or(parsed.account_id)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    Ok(ParsedImportedAuthJson {
        id_token,
        access_token,
        refresh_token,
        account_id,
    })
}

// --- Sub-components ---

#[derive(Properties, PartialEq)]
struct PublicKeyCardProps {
    key_item: LlmGatewayPublicKeyView,
    on_copy: Callback<(String, String)>,
    on_refresh: Callback<(String, String)>,
    refreshing: bool,
}

// PLACEHOLDER_PUBLIC_KEY_CARD

#[function_component(PublicKeyCard)]
fn public_key_card(props: &PublicKeyCardProps) -> Html {
    let key_item = props.key_item.clone();
    let usage_percent = (usage_ratio(&key_item) * 100.0).round() as i32;
    html! {
        <article class="sf-card overflow-hidden rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-5 transition-colors hover:border-[var(--border-strong,var(--primary))]">
            <div class="flex items-center justify-between gap-3">
                <h3 class="m-0 font-serif text-lg font-normal text-[var(--text)]">
                    { key_item.name.clone() }
                </h3>
                <button
                    type="button"
                    class="sf-btn sf-btn--secondary inline-flex h-7 w-7 items-center justify-center rounded-[6px] border border-[var(--border)] bg-transparent text-xs text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)] disabled:cursor-not-allowed disabled:opacity-50"
                    title="刷新额度"
                    aria-label="刷新额度"
                    onclick={{
                        let on_refresh = props.on_refresh.clone();
                        let key_id = key_item.id.clone();
                        let key_name = key_item.name.clone();
                        Callback::from(move |_| on_refresh.emit((key_id.clone(), key_name.clone())))
                    }}
                    disabled={props.refreshing}
                >
                    <i class={classes!("fas", if props.refreshing { "fa-spinner animate-spin" } else { "fa-rotate-right" })} />
                </button>
            </div>
            <div class="mt-3 rounded-[8px] border border-[var(--border)] bg-[var(--surface-sunk,var(--surface-alt))] px-3 py-2.5 text-[var(--text)]">
                <MaskedSecretCode
                    value={key_item.secret.clone()}
                    copy_label={"Key"}
                    on_copy={props.on_copy.clone()}
                    code_class={classes!("text-[var(--text)]")}
                />
            </div>
            <div class="mt-4 grid grid-cols-2 gap-3">
                <div>
                    <div class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">{ "剩余" }</div>
                    <div class="mt-1 font-mono text-2xl font-medium tabular-nums text-[var(--text)]">
                        { format_number_i64(key_item.remaining_billable) }
                    </div>
                </div>
                <div>
                    <div class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">{ "总额度" }</div>
                    <div class="mt-1 font-mono text-2xl font-medium tabular-nums text-[var(--text)]">
                        { format_number_u64(key_item.quota_billable_limit) }
                    </div>
                </div>
            </div>
            <div class="mt-4">
                <div class="flex items-center justify-between font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">
                    <span>{ "用量" }</span>
                    <span class="tabular-nums">{ format!("{usage_percent}%") }</span>
                </div>
                <div class="mt-1.5 h-1.5 overflow-hidden rounded-full bg-[var(--surface-alt)]">
                    <div
                        class="h-full rounded-full bg-[var(--primary)] transition-[width] duration-300"
                        style={format!("width: {}%;", usage_percent.clamp(0, 100))}
                    />
                </div>
                <div class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-[11px] tabular-nums text-[var(--muted)]">
                    <span>{ format!("in {}", format_number_u64(key_item.usage_input_uncached_tokens)) }</span>
                    <span>{ format!("cache {}", format_number_u64(key_item.usage_input_cached_tokens)) }</span>
                    <span>{ format!("out {}", format_number_u64(key_item.usage_output_tokens)) }</span>
                    if let Some(ts) = key_item.last_used_at {
                        <span class="ml-auto">{ format_ms(ts) }</span>
                    }
                </div>
            </div>
        </article>
    }
}

// PLACEHOLDER_MAIN_COMPONENT

#[function_component(LlmAccessPage)]
pub fn llm_access_page() -> Html {
    let access = use_state(|| None::<LlmGatewayAccessResponse>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let support_config = use_state(|| None::<LlmGatewaySupportConfigView>);
    let support_error = use_state(|| None::<String>);
    let toast = use_state(|| None::<(String, bool)>);
    let toast_timeout = use_mut_ref(|| None::<Timeout>);
    let refreshing_key = use_state(|| None::<String>);
    let active_modal = use_state(|| ActiveModal::None);
    let qr_zoomed = use_state(|| false);
    // Token wish form
    let wish_quota = use_state(String::new);
    let wish_reason = use_state(String::new);
    let wish_email = use_state(String::new);
    let wish_submitting = use_state(|| false);
    let wish_feedback = use_state(|| None::<(String, bool)>);
    // Contributions
    let contributions = use_state(Vec::<PublicLlmGatewayAccountContributionView>::new);
    let contribution_error = use_state(|| None::<String>);
    let contribution_account_name = use_state(String::new);
    let contribution_raw_auth_json = use_state(String::new);
    let contribution_raw_auth_feedback = use_state(|| None::<(String, bool)>);
    let contribution_account_id = use_state(String::new);
    let contribution_id_token = use_state(String::new);
    let contribution_access_token = use_state(String::new);
    let contribution_refresh_token = use_state(String::new);
    let contribution_email = use_state(String::new);
    let contribution_message = use_state(String::new);
    let contribution_github_id = use_state(String::new);
    let contribution_submitting = use_state(|| false);
    let contribution_feedback = use_state(|| None::<(String, bool)>);
    // GPT image account contribution form
    let gpt_contribution_account_name = use_state(String::new);
    let gpt_contribution_access_token = use_state(String::new);
    let gpt_contribution_session_json = use_state(String::new);
    let gpt_contribution_email = use_state(String::new);
    let gpt_contribution_message = use_state(String::new);
    let gpt_contribution_github_id = use_state(String::new);
    let gpt_contribution_submitting = use_state(|| false);
    let gpt_contribution_feedback = use_state(|| None::<(String, bool)>);
    // Sponsors
    let sponsors = use_state(Vec::<PublicLlmGatewaySponsorView>::new);
    let sponsor_error = use_state(|| None::<String>);
    let sponsor_email = use_state(String::new);
    let sponsor_display_name = use_state(String::new);
    let sponsor_github_id = use_state(String::new);
    let sponsor_message = use_state(String::new);
    let sponsor_submitting = use_state(|| false);
    let sponsor_feedback = use_state(|| None::<(String, bool)>);

    // --- Data fetching effects ---
    {
        let access = access.clone();
        let loading = loading.clone();
        let error = error.clone();
        let contributions = contributions.clone();
        let contribution_error = contribution_error.clone();
        let support_config = support_config.clone();
        let support_error = support_error.clone();
        let sponsors = sponsors.clone();
        let sponsor_error = sponsor_error.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_llm_gateway_public_page().await {
                    Ok(data) => {
                        access.set(Some(data.access));
                        contributions.set(data.account_contributions.contributions);
                        support_config.set(Some(data.support_config));
                        sponsors.set(data.sponsors.sponsors);
                        error.set(None);
                        contribution_error.set(None);
                        support_error.set(None);
                        sponsor_error.set(None);
                    },
                    Err(err) => {
                        access.set(None);
                        contributions.set(vec![]);
                        support_config.set(None);
                        sponsors.set(vec![]);
                        error.set(Some(err.clone()));
                        contribution_error.set(Some(err.clone()));
                        support_error.set(Some(err.clone()));
                        sponsor_error.set(Some(err));
                    },
                }
                loading.set(false);
            });
            || ()
        });
    }
    // Unified Escape key handler
    {
        let qr_zoomed = qr_zoomed.clone();
        let active_modal = active_modal.clone();
        let has_overlay = *qr_zoomed || *active_modal != ActiveModal::None;
        use_effect_with(has_overlay, move |is_open| {
            let listener_opt = if *is_open {
                let qr_h = qr_zoomed.clone();
                let modal_h = active_modal.clone();
                let listener =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |event: KeyboardEvent| {
                        if event.key() == "Escape" {
                            if *qr_h {
                                qr_h.set(false);
                            } else if *modal_h != ActiveModal::None {
                                modal_h.set(ActiveModal::None);
                            }
                        }
                    })
                        as Box<dyn FnMut(_)>);
                if let Some(win) = web_sys::window() {
                    let _ = win.add_event_listener_with_callback(
                        "keydown",
                        listener.as_ref().unchecked_ref(),
                    );
                }
                Some(listener)
            } else {
                None
            };
            move || {
                if let Some(listener) = listener_opt {
                    if let Some(win) = web_sys::window() {
                        let _ = win.remove_event_listener_with_callback(
                            "keydown",
                            listener.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        });
    }
    // PLACEHOLDER_CALLBACKS

    let show_toast = {
        let toast = toast.clone();
        let toast_timeout = toast_timeout.clone();
        move |msg: String, is_error: bool, duration_ms: u32| {
            toast.set(Some((msg, is_error)));
            toast_timeout.borrow_mut().take();
            let toast = toast.clone();
            let clear_handle = toast_timeout.clone();
            let timeout = Timeout::new(duration_ms, move || {
                toast.set(None);
                clear_handle.borrow_mut().take();
            });
            *toast_timeout.borrow_mut() = Some(timeout);
        }
    };

    let on_copy = {
        let show_toast = show_toast.clone();
        Callback::from(move |(label, value): (String, String)| {
            copy_text(&value);
            show_toast(format!("已复制{}", label), false, 1800);
        })
    };

    let on_refresh_key = {
        let access = access.clone();
        let refreshing_key = refreshing_key.clone();
        let show_toast = show_toast.clone();
        Callback::from(move |(key_id, key_name): (String, String)| {
            refreshing_key.set(Some(key_id));
            let access = access.clone();
            let refreshing_key = refreshing_key.clone();
            let show_toast = show_toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_llm_gateway_access().await {
                    Ok(data) => {
                        access.set(Some(data));
                        show_toast(format!("已刷新 {}", key_name), false, 2200);
                    },
                    Err(err) => {
                        show_toast(format!("刷新失败：{}", err), true, 2200);
                    },
                }
                refreshing_key.set(None);
            });
        })
    };

    // PLACEHOLDER_FORM_CALLBACKS

    let on_submit_token_wish = {
        let wish_quota = wish_quota.clone();
        let wish_reason = wish_reason.clone();
        let wish_email = wish_email.clone();
        let wish_submitting = wish_submitting.clone();
        let wish_feedback = wish_feedback.clone();
        let active_modal = active_modal.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let quota_raw = (*wish_quota).trim().to_string();
            let reason = (*wish_reason).trim().to_string();
            let email = (*wish_email).trim().to_string();
            let Ok(quota) = quota_raw.parse::<u64>() else {
                wish_feedback.set(Some(("token 量必须是正整数".to_string(), true)));
                return;
            };
            if quota == 0 || reason.is_empty() || email.is_empty() {
                wish_feedback.set(Some(("token 量、缘由和邮箱都必须填写".to_string(), true)));
                return;
            }
            let frontend_page_url = web_sys::window().and_then(|w| w.location().href().ok());
            let wish_quota = wish_quota.clone();
            let wish_reason = wish_reason.clone();
            let wish_email = wish_email.clone();
            let wish_submitting = wish_submitting.clone();
            let wish_feedback = wish_feedback.clone();
            let active_modal = active_modal.clone();
            wish_submitting.set(true);
            wish_feedback.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match submit_llm_gateway_token_request(
                    quota,
                    &reason,
                    &email,
                    frontend_page_url.as_deref(),
                )
                .await
                {
                    Ok(_) => {
                        wish_quota.set(String::new());
                        wish_reason.set(String::new());
                        wish_email.set(String::new());
                        wish_feedback.set(Some((
                            "许愿已提交，审核通过后会创建 token 并发送到你的邮箱。".to_string(),
                            false,
                        )));
                        active_modal.set(ActiveModal::None);
                    },
                    Err(err) => {
                        wish_feedback.set(Some((err, true)));
                    },
                }
                wish_submitting.set(false);
            });
        })
    };

    let on_submit_account_contribution = {
        let contribution_account_name = contribution_account_name.clone();
        let contribution_raw_auth_json = contribution_raw_auth_json.clone();
        let contribution_raw_auth_feedback = contribution_raw_auth_feedback.clone();
        let contribution_account_id = contribution_account_id.clone();
        let contribution_id_token = contribution_id_token.clone();
        let contribution_access_token = contribution_access_token.clone();
        let contribution_refresh_token = contribution_refresh_token.clone();
        let contribution_email = contribution_email.clone();
        let contribution_message = contribution_message.clone();
        let contribution_github_id = contribution_github_id.clone();
        let contribution_submitting = contribution_submitting.clone();
        let contribution_feedback = contribution_feedback.clone();
        let active_modal = active_modal.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let account_name = (*contribution_account_name).trim().to_string();
            let account_id = (*contribution_account_id).trim().to_string();
            let id_token = (*contribution_id_token).trim().to_string();
            let access_token_val = (*contribution_access_token).trim().to_string();
            let refresh_token_val = (*contribution_refresh_token).trim().to_string();
            let email = (*contribution_email).trim().to_string();
            let message = (*contribution_message).trim().to_string();
            let github_id = (*contribution_github_id).trim().to_string();
            if account_name.is_empty() || refresh_token_val.is_empty() || message.is_empty() {
                contribution_feedback
                    .set(Some(("账号名、refresh_token 和留言必须填写".to_string(), true)));
                return;
            }
            let frontend_page_url = web_sys::window().and_then(|w| w.location().href().ok());
            let contribution_account_name = contribution_account_name.clone();
            let contribution_raw_auth_json = contribution_raw_auth_json.clone();
            let contribution_raw_auth_feedback = contribution_raw_auth_feedback.clone();
            let contribution_account_id = contribution_account_id.clone();
            let contribution_id_token = contribution_id_token.clone();
            let contribution_access_token = contribution_access_token.clone();
            let contribution_refresh_token = contribution_refresh_token.clone();
            let contribution_email = contribution_email.clone();
            let contribution_message = contribution_message.clone();
            let contribution_github_id = contribution_github_id.clone();
            let contribution_submitting = contribution_submitting.clone();
            let contribution_feedback = contribution_feedback.clone();
            let active_modal = active_modal.clone();
            contribution_submitting.set(true);
            contribution_feedback.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let input = SubmitLlmGatewayAccountContributionInput {
                    account_name,
                    account_id: (!account_id.is_empty()).then_some(account_id),
                    id_token,
                    access_token: access_token_val,
                    refresh_token: refresh_token_val,
                    requester_email: (!email.is_empty()).then_some(email),
                    contributor_message: message,
                    github_id: (!github_id.is_empty()).then_some(github_id),
                    frontend_page_url,
                };
                match submit_llm_gateway_account_contribution_request(&input).await {
                    Ok(_) => {
                        contribution_account_name.set(String::new());
                        contribution_raw_auth_json.set(String::new());
                        contribution_raw_auth_feedback.set(None);
                        contribution_account_id.set(String::new());
                        contribution_id_token.set(String::new());
                        contribution_access_token.set(String::new());
                        contribution_refresh_token.set(String::new());
                        contribution_email.set(String::new());
                        contribution_message.set(String::new());
                        contribution_github_id.set(String::new());
                        contribution_feedback.set(Some((
                            "账号贡献已提交，审核验证通过后会导入账号并生成绑定 token。"
                                .to_string(),
                            false,
                        )));
                        active_modal.set(ActiveModal::None);
                    },
                    Err(err) => contribution_feedback.set(Some((err, true))),
                }
                contribution_submitting.set(false);
            });
        })
    };

    let on_submit_gpt_account_contribution = {
        let gpt_contribution_account_name = gpt_contribution_account_name.clone();
        let gpt_contribution_access_token = gpt_contribution_access_token.clone();
        let gpt_contribution_session_json = gpt_contribution_session_json.clone();
        let gpt_contribution_email = gpt_contribution_email.clone();
        let gpt_contribution_message = gpt_contribution_message.clone();
        let gpt_contribution_github_id = gpt_contribution_github_id.clone();
        let gpt_contribution_submitting = gpt_contribution_submitting.clone();
        let gpt_contribution_feedback = gpt_contribution_feedback.clone();
        let active_modal = active_modal.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let account_name = (*gpt_contribution_account_name).trim().to_string();
            let access_token = (*gpt_contribution_access_token).trim().to_string();
            let session_json = (*gpt_contribution_session_json).trim().to_string();
            let email = (*gpt_contribution_email).trim().to_string();
            let message = (*gpt_contribution_message).trim().to_string();
            let github_id = (*gpt_contribution_github_id).trim().to_string();
            if account_name.is_empty()
                || (access_token.is_empty() && session_json.is_empty())
                || email.is_empty()
                || message.is_empty()
            {
                gpt_contribution_feedback.set(Some((
                    "显示名、access token 或 session JSON、邮箱和留言都必须填写".to_string(),
                    true,
                )));
                return;
            }
            let frontend_page_url = web_sys::window().and_then(|w| w.location().href().ok());
            let gpt_contribution_account_name = gpt_contribution_account_name.clone();
            let gpt_contribution_access_token = gpt_contribution_access_token.clone();
            let gpt_contribution_session_json = gpt_contribution_session_json.clone();
            let gpt_contribution_email = gpt_contribution_email.clone();
            let gpt_contribution_message = gpt_contribution_message.clone();
            let gpt_contribution_github_id = gpt_contribution_github_id.clone();
            let gpt_contribution_submitting = gpt_contribution_submitting.clone();
            let gpt_contribution_feedback = gpt_contribution_feedback.clone();
            let active_modal = active_modal.clone();
            gpt_contribution_submitting.set(true);
            gpt_contribution_feedback.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let input = SubmitGpt2ApiAccountContributionInput {
                    account_name,
                    access_token: (!access_token.is_empty()).then_some(access_token),
                    session_json: (!session_json.is_empty()).then_some(session_json),
                    requester_email: email,
                    contributor_message: message,
                    github_id: (!github_id.is_empty()).then_some(github_id),
                    frontend_page_url,
                };
                match submit_gpt2api_account_contribution_request(&input).await {
                    Ok(_) => {
                        gpt_contribution_account_name.set(String::new());
                        gpt_contribution_access_token.set(String::new());
                        gpt_contribution_session_json.set(String::new());
                        gpt_contribution_email.set(String::new());
                        gpt_contribution_message.set(String::new());
                        gpt_contribution_github_id.set(String::new());
                        gpt_contribution_feedback.set(Some((
                            "GPT 账号贡献已提交，审核通过后会把 /gpt2api/login 可用的 key \
                             发到你的邮箱。"
                                .to_string(),
                            false,
                        )));
                        active_modal.set(ActiveModal::None);
                    },
                    Err(err) => gpt_contribution_feedback.set(Some((err, true))),
                }
                gpt_contribution_submitting.set(false);
            });
        })
    };

    let on_submit_sponsor = {
        let sponsor_email = sponsor_email.clone();
        let sponsor_display_name = sponsor_display_name.clone();
        let sponsor_github_id = sponsor_github_id.clone();
        let sponsor_message = sponsor_message.clone();
        let sponsor_submitting = sponsor_submitting.clone();
        let sponsor_feedback = sponsor_feedback.clone();
        let active_modal = active_modal.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let email = (*sponsor_email).trim().to_string();
            let display_name = (*sponsor_display_name).trim().to_string();
            let github_id = (*sponsor_github_id).trim().to_string();
            let message = (*sponsor_message).trim().to_string();
            if email.is_empty() || message.is_empty() {
                sponsor_feedback.set(Some(("邮箱和留言都必须填写".to_string(), true)));
                return;
            }
            let frontend_page_url = web_sys::window().and_then(|w| w.location().href().ok());
            let sponsor_email = sponsor_email.clone();
            let sponsor_display_name = sponsor_display_name.clone();
            let sponsor_github_id = sponsor_github_id.clone();
            let sponsor_message = sponsor_message.clone();
            let sponsor_submitting = sponsor_submitting.clone();
            let sponsor_feedback = sponsor_feedback.clone();
            let active_modal = active_modal.clone();
            sponsor_submitting.set(true);
            sponsor_feedback.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let input = SubmitLlmGatewaySponsorInput {
                    requester_email: email,
                    sponsor_message: message,
                    display_name: (!display_name.is_empty()).then_some(display_name),
                    github_id: (!github_id.is_empty()).then_some(github_id),
                    frontend_page_url,
                };
                match submit_llm_gateway_sponsor_request(&input).await {
                    Ok(response) => {
                        sponsor_email.set(String::new());
                        sponsor_display_name.set(String::new());
                        sponsor_github_id.set(String::new());
                        sponsor_message.set(String::new());
                        let feedback = if response.payment_email_sent {
                            "收款码和付款说明已发到你的邮箱，付款后请直接回复那封邮件。".to_string()
                        } else {
                            format!(
                                "赞助请求已记录（状态 \
                                 {}），付款说明邮件暂未发出，可通过群或邮箱联系。",
                                response.status
                            )
                        };
                        sponsor_feedback.set(Some((feedback, false)));
                        active_modal.set(ActiveModal::None);
                    },
                    Err(err) => sponsor_feedback.set(Some((err, true))),
                }
                sponsor_submitting.set(false);
            });
        })
    };
    // PLACEHOLDER_CONTENT_RENDER

    // Pre-compute QR URL for lightbox (accessible outside content closure)
    let group_qr_url_for_lightbox = (*support_config)
        .as_ref()
        .and_then(|c| c.qq_group_qr_url.as_deref())
        .map(resolve_support_asset_url);

    let secondary_btn_class = "sf-btn sf-btn--secondary inline-flex items-center gap-1.5 \
                               rounded-[8px] border border-[var(--border-strong,var(--border))] \
                               bg-transparent px-3 py-1.5 font-mono text-xs text-[var(--text)] \
                               no-underline transition-colors hover:bg-[var(--surface-alt)]";
    let primary_btn_class = "sf-btn sf-btn--primary inline-flex items-center gap-1.5 \
                             rounded-[8px] bg-[var(--primary)] px-3.5 py-1.5 font-mono text-xs \
                             font-medium text-[var(--on-primary,#FAF9F5)] no-underline \
                             transition-colors hover:bg-[var(--primary-hover,#B85735)] \
                             disabled:cursor-not-allowed disabled:opacity-50";

    let render_feedback_banner = |message: String, is_error: bool| -> Html {
        html! {
            <div class={classes!(
                "mt-4",
                "rounded-[10px]",
                "border",
                "px-4",
                "py-3",
                "font-mono",
                "text-xs",
                "leading-relaxed",
                if is_error {
                    "border-[var(--err,#B8443A)]/35 bg-[var(--err-soft,rgba(184,68,58,0.10))] text-[var(--err,#B8443A)]"
                } else {
                    "border-[var(--ok,#4E7A51)]/35 bg-[var(--ok-soft,rgba(78,122,81,0.10))] text-[var(--ok,#4E7A51)]"
                }
            )}>
                { message }
            </div>
        }
    };

    let content = if *loading {
        html! {
            <div class="mt-10 rounded-[12px] border border-[var(--border)] bg-[var(--surface)] px-5 py-12 text-center font-mono text-sm text-[var(--muted)]">
                { "正在加载公开 Key 列表..." }
            </div>
        }
    } else if let Some(err) = (*error).clone() {
        html! {
            <div class="mt-10 rounded-[12px] border border-[var(--err,#B8443A)]/35 bg-[var(--err-soft,rgba(184,68,58,0.10))] px-5 py-5 font-mono text-sm text-[var(--err,#B8443A)]">
                { err }
            </div>
        }
    } else if let Some(access) = (*access).clone() {
        let base_url = resolved_base_url(&access);
        let support_config_value = (*support_config).clone();
        let support_error_value = (*support_error).clone();
        let group_name = support_config_value
            .as_ref()
            .map(|c| c.group_name.clone())
            .unwrap_or_else(|| "美区词元魔盗团".to_string());
        let group_number = support_config_value
            .as_ref()
            .map(|c| c.qq_group_number.clone())
            .unwrap_or_default();
        let group_qr_url = support_config_value
            .as_ref()
            .and_then(|c| c.qq_group_qr_url.as_deref())
            .map(resolve_support_asset_url);

        let status_view = html! {
            <Link<Route>
                to={Route::LlmAccessQuotaStatus}
                classes="sf-card group flex cursor-pointer items-center justify-between gap-3 rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-5 no-underline transition-colors hover:border-[var(--border-strong,var(--primary))]"
            >
                <div class="flex items-center gap-3.5">
                    <div class="inline-flex h-9 w-9 items-center justify-center rounded-[8px] border border-[var(--border)] bg-[var(--surface-alt)] text-[var(--primary)] transition-colors group-hover:border-[var(--primary)]/40 group-hover:bg-[var(--primary-soft,rgba(201,100,66,0.12))]">
                        <i class="fas fa-chart-bar text-sm" />
                    </div>
                    <div>
                        <h2 class="m-0 font-serif text-lg font-normal text-[var(--text)]">
                            { "限额状态" }
                        </h2>
                        <p class="m-0 mt-0.5 font-mono text-[11px] text-[var(--muted)]">
                            { "查看所有账号池的实时额度与健康详情" }
                        </p>
                    </div>
                </div>
                <span class="font-mono text-xs text-[var(--muted)] transition-colors group-hover:text-[var(--primary)]">
                    { "查看 →" }
                </span>
            </Link<Route>>
        };

        html! {
            <>
                // Editorial Page Header
                <section class="sf-card rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-6 sm:p-7">
                    <div class="flex flex-wrap items-start justify-between gap-4">
                        <div>
                            <div class="mb-2 flex flex-wrap items-center gap-2.5">
                                <span class="sf-chip sf-chip--active inline-flex items-center rounded-full border border-[var(--primary)]/35 bg-[var(--primary-soft,rgba(201,100,66,0.12))] px-2.5 py-0.5 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--primary)]">
                                    { "CODEX / OPENAI GATEWAY" }
                                </span>
                                <span class="rounded-full border border-[var(--border)] bg-[var(--surface-alt)] px-2.5 py-0.5 font-mono text-[11px] tabular-nums text-[var(--muted)]">
                                    { format!("{} keys", access.keys.len()) }
                                </span>
                            </div>
                            <h1 class="m-0 font-serif text-[clamp(1.75rem,3vw,2.375rem)] font-normal leading-[1.15] tracking-[-0.02em] text-[var(--text)]">
                                { "LLM Gateway · 共享密钥大厅" }
                            </h1>
                        </div>

                        <div class="flex flex-wrap items-center gap-2">
                            <button
                                type="button"
                                class={primary_btn_class}
                                onclick={{
                                    let on_copy = on_copy.clone();
                                    let base_url = base_url.clone();
                                    Callback::from(move |_| on_copy.emit(("Base URL".to_string(), base_url.clone())))
                                }}
                            >
                                <i class="fas fa-copy" />
                                <span>{ "复制 Base URL" }</span>
                            </button>
                            <Link<Route> to={Route::LlmAccessGuide} classes={secondary_btn_class}>
                                <i class="fas fa-book" />
                                <span>{ "接入帮助" }</span>
                            </Link<Route>>
                            <Link<Route> to={Route::LlmAccessUsage} classes={secondary_btn_class}>
                                <i class="fas fa-chart-line" />
                                <span>{ "Key 查询" }</span>
                            </Link<Route>>
                            <Link<Route> to={Route::KiroAccess} classes={secondary_btn_class}>
                                <i class="fas fa-bolt" />
                                <span>{ "Kiro" }</span>
                            </Link<Route>>
                            <Link<Route> to={Route::AdminLlmGateway} classes={secondary_btn_class}>
                                <i class="fas fa-sliders" />
                            </Link<Route>>
                        </div>
                    </div>

                    <div class="mt-4 rounded-[8px] border border-[var(--border)] bg-[var(--surface-sunk,var(--surface-alt))] px-3.5 py-2.5">
                        <div class="font-mono text-[10px] uppercase tracking-[0.12em] text-[var(--muted)]">
                            { "ENDPOINT BASE URL" }
                        </div>
                        <code class="mt-1 block break-all font-mono text-sm text-[var(--text)]">
                            { base_url.clone() }
                        </code>
                    </div>
                </section>

                // Warm terracotta callouts
                <div class="mt-4 rounded-[10px] border border-[var(--border)] border-l-2 border-l-[var(--primary)] bg-[var(--surface-alt)] px-4 py-3 font-mono text-xs leading-relaxed text-[var(--text-2,var(--text))]">
                    { "remote compact 必须保留 — " }
                    <Link<Route> to={Route::LlmAccessGuide} classes="text-[var(--primary)] underline underline-offset-4">
                        { "接入帮助" }
                    </Link<Route>>
                    { " · " }
                    <Link<Route>
                        to={Route::ArticleDetail { id: REMOTE_COMPACT_ARTICLE_ID.to_string() }}
                        classes="text-[var(--primary)] underline underline-offset-4"
                    >
                        { "深潜文章" }
                    </Link<Route>>
                </div>

                <div class="mt-3 rounded-[10px] border border-[var(--border)] border-l-2 border-l-[var(--warn,#A66B2B)] bg-[var(--surface-alt)] px-4 py-3 font-mono text-xs leading-relaxed text-[var(--text-2,var(--text))]">
                    { "🦀 你正在用的这个站就是纯 Rust 全栈开源项目哦 (ノ°▽°)ノ 几乎 100% vibe coded — " }
                    <a
                        href="https://github.com/acking-you/static_flow"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-[var(--primary)] underline underline-offset-4"
                    >
                        { "来 GitHub 看看这个站的源码？" }
                    </a>
                </div>

                // Public Keys Section
                <section class="mt-8">
                    <div class="mb-3 flex items-baseline justify-between">
                        <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">
                            { "公开 Key" }
                        </h2>
                        <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">
                            { "PUBLIC KEYS" }
                        </span>
                    </div>
                    if access.keys.is_empty() {
                        <div class="rounded-[12px] border border-[var(--border)] bg-[var(--surface)] px-5 py-10 text-center font-mono text-sm text-[var(--muted)]">
                            { "当前没有公开放出的 Key" }
                        </div>
                    } else {
                        <div class="grid gap-4 lg:grid-cols-2">
                            { for access.keys.iter().map(|key_item| html! {
                                <PublicKeyCard
                                    key={key_item.id.clone()}
                                    key_item={key_item.clone()}
                                    on_copy={on_copy.clone()}
                                    on_refresh={on_refresh_key.clone()}
                                    refreshing={(*refreshing_key).as_deref() == Some(key_item.id.as_str())}
                                />
                            }) }
                        </div>
                    }
                </section>

                // Community Action Buttons
                <section class="mt-6 flex flex-wrap items-center gap-2.5">
                    <button
                        type="button"
                        class={primary_btn_class}
                        onclick={{
                            let active_modal = active_modal.clone();
                            Callback::from(move |_| active_modal.set(ActiveModal::TokenWish))
                        }}
                    >
                        <i class="fas fa-wand-magic-sparkles" />
                        <span>{ "许愿 Token" }</span>
                    </button>
                    <button
                        type="button"
                        class={secondary_btn_class}
                        onclick={{
                            let active_modal = active_modal.clone();
                            Callback::from(move |_| active_modal.set(ActiveModal::AccountContribution))
                        }}
                    >
                        <i class="fas fa-user-plus" />
                        <span>{ "贡献账号" }</span>
                    </button>
                    <button
                        type="button"
                        class={secondary_btn_class}
                        onclick={{
                            let active_modal = active_modal.clone();
                            Callback::from(move |_| active_modal.set(ActiveModal::GptAccountContribution))
                        }}
                    >
                        <i class="fas fa-image" />
                        <span>{ "贡献 GPT" }</span>
                    </button>
                    <button
                        type="button"
                        class={secondary_btn_class}
                        onclick={{
                            let active_modal = active_modal.clone();
                            Callback::from(move |_| active_modal.set(ActiveModal::Sponsor))
                        }}
                    >
                        <i class="fas fa-mug-hot" />
                        <span>{ "赞助站点" }</span>
                    </button>
                </section>

                // Quota Status Link Card
                <section class="mt-6">
                    { status_view }
                </section>

                // QQ Group Card
                if !group_number.is_empty() {
                    <section class="sf-card mt-6 rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-5">
                        <div class="flex flex-wrap items-start gap-4">
                            if let Some(group_qr_url) = group_qr_url.clone() {
                                <div
                                    class="shrink-0 cursor-pointer overflow-hidden rounded-[8px] border border-[var(--border)] bg-white transition-colors hover:border-[var(--primary)]"
                                    onclick={{
                                        let qr_zoomed = qr_zoomed.clone();
                                        Callback::from(move |_: MouseEvent| qr_zoomed.set(true))
                                    }}
                                    role="button"
                                    tabindex="0"
                                    aria-label="放大查看 QR 码"
                                >
                                    <img
                                        src={group_qr_url}
                                        alt="QQ group QR"
                                        class="block h-20 w-20 object-cover"
                                        loading="lazy"
                                    />
                                </div>
                            }
                            <div class="min-w-0 flex-1">
                                <div class="flex flex-wrap items-center gap-2.5">
                                    <h2 class="m-0 font-serif text-lg font-normal text-[var(--text)]">
                                        { group_name.clone() }
                                    </h2>
                                    <span class="font-mono text-xs tabular-nums text-[var(--muted)]">
                                        { group_number.clone() }
                                    </span>
                                    <button
                                        type="button"
                                        class="rounded-[6px] border border-[var(--border)] px-2 py-0.5 font-mono text-[11px] text-[var(--muted)] transition-colors hover:border-[var(--primary)] hover:text-[var(--primary)]"
                                        onclick={{
                                            let on_copy = on_copy.clone();
                                            let group_number = group_number.clone();
                                            Callback::from(move |_| on_copy.emit(("群号".to_string(), group_number.clone())))
                                        }}
                                    >
                                        <i class="fas fa-copy mr-1" />
                                        { "复制" }
                                    </button>
                                </div>
                                <p class="m-0 mt-2 text-sm leading-relaxed text-[var(--muted)]">
                                    { "欢迎加 QQ 群一起交流模型、分享 prompt，遇到接入问题也能快速解决。扫码或搜群号直接加入 🎉" }
                                </p>
                            </div>
                        </div>
                    </section>
                }

                // Inline feedback banners after modal submission
                if let Some((message, is_error)) = (*wish_feedback).clone() {
                    { render_feedback_banner(message, is_error) }
                }
                if let Some((message, is_error)) = (*contribution_feedback).clone() {
                    { render_feedback_banner(message, is_error) }
                }
                if let Some((message, is_error)) = (*gpt_contribution_feedback).clone() {
                    { render_feedback_banner(message, is_error) }
                }
                if let Some((message, is_error)) = (*sponsor_feedback).clone() {
                    { render_feedback_banner(message, is_error) }
                }

                if let Some(err) = support_error_value.clone() {
                    <div class="mt-4 rounded-[10px] border border-[var(--border)] bg-[var(--surface-alt)] px-4 py-3 font-mono text-xs text-[var(--muted)]">
                        { format!("社区配置暂不可用：{}", err) }
                    </div>
                }

                // Contribution Thank-You Wall
                if !contributions.is_empty() || (*contribution_error).is_some() {
                    <section class="mt-8">
                        <div class="mb-3 flex items-baseline justify-between">
                            <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">
                                { "贡献感谢墙" }
                            </h2>
                            <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">
                                { "CONTRIBUTORS" }
                            </span>
                        </div>
                        if let Some(err) = (*contribution_error).clone() {
                            { render_feedback_banner(err, true) }
                        } else {
                            <div class="grid gap-3 lg:grid-cols-2">
                                { for contributions.iter().map(|item| {
                                    let github_id = item.github_id.clone();
                                    let avatar_url = github_id.as_deref().map(github_avatar_url).unwrap_or_default();
                                    let profile_url = github_id.as_deref().map(github_profile_url).unwrap_or_default();
                                    html! {
                                        <article class="sf-card rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-4 transition-colors hover:border-[var(--border-strong,var(--primary))]">
                                            <div class="flex items-start gap-3">
                                                if let Some(gid) = github_id.clone() {
                                                    <a href={profile_url.clone()} target="_blank" rel="noreferrer noopener" class="shrink-0">
                                                        <img
                                                            src={avatar_url}
                                                            alt={gid}
                                                            class="h-10 w-10 rounded-full border border-[var(--border)] object-cover"
                                                            loading="lazy"
                                                        />
                                                    </a>
                                                } else {
                                                    <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full border border-[var(--primary)]/25 bg-[var(--primary-soft,rgba(201,100,66,0.10))] text-[var(--primary)]">
                                                        <i class="fas fa-user-astronaut text-sm" />
                                                    </div>
                                                }
                                                <div class="min-w-0 flex-1">
                                                    <div class="flex flex-wrap items-center gap-2">
                                                        <span class="rounded bg-[var(--primary-soft,rgba(201,100,66,0.10))] px-2 py-0.5 font-mono text-xs font-medium text-[var(--primary)]">
                                                            { item.account_name.clone() }
                                                        </span>
                                                        if let Some(gid) = item.github_id.clone() {
                                                            <a
                                                                href={profile_url}
                                                                target="_blank"
                                                                rel="noreferrer noopener"
                                                                class="font-mono text-[11px] text-[var(--muted)] no-underline transition-colors hover:text-[var(--primary)]"
                                                            >
                                                                { format!("@{gid}") }
                                                            </a>
                                                        }
                                                        if let Some(ts) = item.processed_at {
                                                            <span class="font-mono text-[11px] tabular-nums text-[var(--muted)]">
                                                                { format_ms(ts) }
                                                            </span>
                                                        }
                                                    </div>
                                                    <p class="m-0 mt-2 whitespace-pre-wrap break-words text-sm leading-relaxed text-[var(--text-2,var(--text))]">
                                                        { item.contributor_message.clone() }
                                                    </p>
                                                </div>
                                            </div>
                                        </article>
                                    }
                                }) }
                            </div>
                        }
                    </section>
                }

                // Sponsor Thank-You Wall
                if !sponsors.is_empty() || (*sponsor_error).is_some() {
                    <section class="mt-8">
                        <div class="mb-3 flex items-baseline justify-between">
                            <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">
                                { "Sponsor 感谢墙" }
                            </h2>
                            <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">
                                { "SPONSORS" }
                            </span>
                        </div>
                        if let Some(err) = (*sponsor_error).clone() {
                            { render_feedback_banner(err, true) }
                        } else {
                            <div class="grid gap-3 lg:grid-cols-2">
                                { for sponsors.iter().map(|item| {
                                    let github_id = item.github_id.clone();
                                    let avatar_url = github_id.as_deref().map(github_avatar_url).unwrap_or_default();
                                    let profile_url = github_id.as_deref().map(github_profile_url).unwrap_or_default();
                                    let display_name = item
                                        .display_name
                                        .clone()
                                        .filter(|v| !v.trim().is_empty())
                                        .unwrap_or_else(|| {
                                            github_id
                                                .as_ref()
                                                .map(|id| format!("@{id}"))
                                                .unwrap_or_else(|| "匿名".to_string())
                                        });
                                    html! {
                                        <article class="sf-card rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-4 transition-colors hover:border-[var(--border-strong,var(--primary))]">
                                            <div class="flex items-start gap-3">
                                                if let Some(gid) = github_id {
                                                    <a href={profile_url} target="_blank" rel="noreferrer noopener" class="shrink-0">
                                                        <img
                                                            src={avatar_url}
                                                            alt={gid}
                                                            class="h-10 w-10 rounded-full border border-[var(--border)] object-cover"
                                                            loading="lazy"
                                                        />
                                                    </a>
                                                } else {
                                                    <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full border border-[var(--warn,#A66B2B)]/30 bg-[var(--warn-soft,rgba(166,107,43,0.10))] text-[var(--warn,#A66B2B)]">
                                                        <i class="fas fa-mug-hot text-sm" />
                                                    </div>
                                                }
                                                <div class="min-w-0 flex-1">
                                                    <div class="flex flex-wrap items-center gap-2">
                                                        <span class="font-serif text-base font-normal text-[var(--text)]">
                                                            { display_name }
                                                        </span>
                                                        if let Some(ts) = item.processed_at {
                                                            <span class="font-mono text-[11px] tabular-nums text-[var(--muted)]">
                                                                { format_ms(ts) }
                                                            </span>
                                                        }
                                                    </div>
                                                    <p class="m-0 mt-2 whitespace-pre-wrap break-words text-sm leading-relaxed text-[var(--text-2,var(--text))]">
                                                        { item.sponsor_message.clone() }
                                                    </p>
                                                </div>
                                            </div>
                                        </article>
                                    }
                                }) }
                            </div>
                        }
                    </section>
                }
            </>
        }
    } else {
        Html::default()
    };

    let ic = "sf-input mt-1 w-full rounded-[8px] border border-[var(--border)] bg-[var(--bg)] \
              px-3 py-2 font-mono text-sm text-[var(--text)] placeholder-[var(--muted)] \
              transition-colors focus:border-[var(--primary)] focus:outline-none";
    let ic_mono_xs = "sf-textarea mt-1 w-full resize-y rounded-[8px] border \
                      border-[var(--border)] bg-[var(--bg)] px-3 py-2 font-mono text-xs \
                      text-[var(--text)] placeholder-[var(--muted)] transition-colors \
                      focus:border-[var(--primary)] focus:outline-none";
    let modal_overlay_class = "fixed inset-0 z-[100] flex items-center justify-center \
                               bg-[rgba(20,20,19,0.52)] p-4 backdrop-blur-[2px] \
                               dark:bg-[rgba(0,0,0,0.72)]";
    let modal_card_class = "w-full max-w-lg rounded-[12px] border \
                            border-[var(--border-strong,var(--border))] bg-[var(--surface)] p-6 \
                            shadow-[var(--shadow-lg)]";
    let modal_error_class = "rounded-[8px] border border-[var(--err,#B8443A)]/35 \
                             bg-[var(--err-soft,rgba(184,68,58,0.10))] px-3 py-2 font-mono \
                             text-xs text-[var(--err,#B8443A)]";

    html! {
        <main class="relative min-h-screen bg-[var(--bg)]">
            <div class="relative mx-auto max-w-[960px] px-4 pb-20 pt-10 sm:px-6 md:pt-14">
                { content }
            </div>

            // Warm QR Lightbox Overlay
            if *qr_zoomed {
                if let Some(ref lightbox_url) = group_qr_url_for_lightbox {
                    <div
                        class="fixed inset-0 z-[100] flex cursor-pointer items-center justify-center bg-[rgba(250,249,245,0.96)] p-4 backdrop-blur-[2px] dark:bg-[rgba(20,20,19,0.96)]"
                        role="dialog"
                        aria-modal="true"
                        onclick={{ let qr_zoomed = qr_zoomed.clone(); Callback::from(move |_: MouseEvent| qr_zoomed.set(false)) }}
                    >
                        <img
                            src={lightbox_url.clone()}
                            alt="QQ group QR code"
                            class="max-h-[80vh] max-w-[90vw] rounded-[12px] border border-[var(--border)] bg-white shadow-[var(--shadow-lg)]"
                            onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                        />
                    </div>
                }
            }

            // Modal: Token Wish
            if *active_modal == ActiveModal::TokenWish {
                <div
                    class={modal_overlay_class}
                    role="dialog"
                    aria-modal="true"
                    onclick={{ let m = active_modal.clone(); Callback::from(move |_: MouseEvent| m.set(ActiveModal::None)) }}
                >
                    <div
                        class={modal_card_class}
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <div class="flex items-center justify-between gap-3">
                            <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">{ "许愿 Token" }</h2>
                            <button
                                type="button"
                                class={secondary_btn_class}
                                onclick={{ let m = active_modal.clone(); Callback::from(move |_| m.set(ActiveModal::None)) }}
                            >
                                <i class="fas fa-xmark" />
                            </button>
                        </div>
                        <p class="m-0 mt-2 font-mono text-xs text-[var(--muted)]">
                            { "提交额度申请，审核通过后 token 会发到你的邮箱。" }
                        </p>
                        <form class="mt-4 grid gap-3" onsubmit={on_submit_token_wish}>
                            <div class="grid gap-3 sm:grid-cols-2">
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "token 量" }</span>
                                    <input
                                        type="number"
                                        min="1"
                                        step="1"
                                        placeholder="500000"
                                        class={ic}
                                        value={(*wish_quota).clone()}
                                        required=true
                                        oninput={{ let s = wish_quota.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "邮箱" }</span>
                                    <input
                                        type="email"
                                        placeholder="you@example.com"
                                        class={ic}
                                        value={(*wish_email).clone()}
                                        required=true
                                        oninput={{ let s = wish_email.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                            </div>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "缘由" }</span>
                                <textarea
                                    rows="3"
                                    placeholder="用途和需求量说明"
                                    class={ic_mono_xs}
                                    value={(*wish_reason).clone()}
                                    required=true
                                    oninput={{ let s = wish_reason.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <div class="flex justify-end pt-1">
                                <button type="submit" class={primary_btn_class} disabled={*wish_submitting}>
                                    <i class={classes!("fas", if *wish_submitting { "fa-spinner animate-spin" } else { "fa-paper-plane" })} />
                                    <span>{ if *wish_submitting { "提交中..." } else { "提交" } }</span>
                                </button>
                            </div>
                            if let Some((msg, is_err)) = (*wish_feedback).clone() {
                                if is_err {
                                    <div class={modal_error_class}>{ msg }</div>
                                }
                            }
                        </form>
                    </div>
                </div>
            }

            // Modal: Account Contribution
            if *active_modal == ActiveModal::AccountContribution {
                <div
                    class={classes!(modal_overlay_class, "overflow-y-auto")}
                    role="dialog"
                    aria-modal="true"
                    onclick={{ let m = active_modal.clone(); Callback::from(move |_: MouseEvent| m.set(ActiveModal::None)) }}
                >
                    <div
                        class={classes!(modal_card_class, "my-8")}
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <div class="flex items-center justify-between gap-3">
                            <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">{ "贡献账号" }</h2>
                            <button
                                type="button"
                                class={secondary_btn_class}
                                onclick={{ let m = active_modal.clone(); Callback::from(move |_| m.set(ActiveModal::None)) }}
                            >
                                <i class="fas fa-xmark" />
                            </button>
                        </div>
                        <p class="m-0 mt-2 font-mono text-xs text-[var(--muted)]">
                            { "贡献 Codex 账号到池里，审核验证通过后会生成一把绑定该账号的 token。" }
                        </p>
                        <form class="mt-4 grid gap-3" onsubmit={on_submit_account_contribution}>
                            <div class="grid gap-3 sm:grid-cols-2">
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "账号名" }</span>
                                    <input
                                        type="text"
                                        placeholder="my-pro-account"
                                        class={ic}
                                        value={(*contribution_account_name).clone()}
                                        required=true
                                        oninput={{ let s = contribution_account_name.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "邮箱（可选）" }</span>
                                    <input
                                        type="email"
                                        placeholder="you@example.com"
                                        class={ic}
                                        value={(*contribution_email).clone()}
                                        oninput={{ let s = contribution_email.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                            </div>
                            <label class="text-sm">
                                <div class="flex items-center justify-between gap-2">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "auth.json（可选，粘贴后自动回填）" }</span>
                                </div>
                                <textarea
                                    rows="4"
                                    placeholder="{\"tokens\":{...}}"
                                    class={ic_mono_xs}
                                    value={(*contribution_raw_auth_json).clone()}
                                    oninput={{
                                        let raw_s = contribution_raw_auth_json.clone();
                                        let fb = contribution_raw_auth_feedback.clone();
                                        let aid = contribution_account_id.clone();
                                        let idt = contribution_id_token.clone();
                                        let act = contribution_access_token.clone();
                                        let rft = contribution_refresh_token.clone();
                                        Callback::from(move |e: InputEvent| {
                                            if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() {
                                                let raw = t.value();
                                                let trimmed = raw.trim().to_string();
                                                raw_s.set(raw);
                                                if trimmed.is_empty() {
                                                    fb.set(None);
                                                    return;
                                                }
                                                match parse_imported_auth_json(&trimmed) {
                                                    Ok(p) => {
                                                        aid.set(p.account_id.unwrap_or_default());
                                                        idt.set(p.id_token);
                                                        act.set(p.access_token);
                                                        rft.set(p.refresh_token);
                                                        fb.set(Some(("已自动回填可识别 token 字段".to_string(), false)));
                                                    },
                                                    Err(err) => {
                                                        if trimmed.ends_with('}') || trimmed.contains('\n') {
                                                            fb.set(Some((err, true)));
                                                        } else {
                                                            fb.set(None);
                                                        }
                                                    },
                                                }
                                            }
                                        })
                                    }}
                                />
                                if let Some((msg, is_err)) = (*contribution_raw_auth_feedback).clone() {
                                    <div class={classes!(
                                        "mt-1",
                                        "font-mono",
                                        "text-[11px]",
                                        if is_err { "text-[var(--err,#B8443A)]" } else { "text-[var(--ok,#4E7A51)]" }
                                    )}>
                                        { msg }
                                    </div>
                                }
                            </label>
                            <div class="grid gap-3 sm:grid-cols-2">
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "GitHub ID（可选）" }</span>
                                    <input
                                        type="text"
                                        placeholder="ackingliu"
                                        class={ic}
                                        value={(*contribution_github_id).clone()}
                                        oninput={{ let s = contribution_github_id.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "account_id（可选）" }</span>
                                    <input
                                        type="text"
                                        class={ic}
                                        value={(*contribution_account_id).clone()}
                                        oninput={{ let s = contribution_account_id.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                            </div>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "access_token（可选）" }</span>
                                <textarea
                                    rows="2"
                                    class={ic_mono_xs}
                                    value={(*contribution_access_token).clone()}
                                    oninput={{ let s = contribution_access_token.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <div class="grid gap-3 sm:grid-cols-2">
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "id_token（可选）" }</span>
                                    <textarea
                                        rows="2"
                                        class={ic_mono_xs}
                                        value={(*contribution_id_token).clone()}
                                        oninput={{ let s = contribution_id_token.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "refresh_token" }</span>
                                    <textarea
                                        rows="2"
                                        class={ic_mono_xs}
                                        value={(*contribution_refresh_token).clone()}
                                        required=true
                                        oninput={{ let s = contribution_refresh_token.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                            </div>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "留言" }</span>
                                <textarea
                                    rows="3"
                                    placeholder="为什么愿意贡献这个账号"
                                    class={ic_mono_xs}
                                    value={(*contribution_message).clone()}
                                    required=true
                                    oninput={{ let s = contribution_message.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <div class="flex justify-end pt-1">
                                <button type="submit" class={primary_btn_class} disabled={*contribution_submitting}>
                                    <i class={classes!("fas", if *contribution_submitting { "fa-spinner animate-spin" } else { "fa-heart-circle-plus" })} />
                                    <span>{ if *contribution_submitting { "提交中..." } else { "提交" } }</span>
                                </button>
                            </div>
                            if let Some((msg, is_err)) = (*contribution_feedback).clone() {
                                if is_err {
                                    <div class={modal_error_class}>{ msg }</div>
                                }
                            }
                        </form>
                    </div>
                </div>
            }

            // Modal: GPT Account Contribution
            if *active_modal == ActiveModal::GptAccountContribution {
                <div
                    class={classes!(modal_overlay_class, "overflow-y-auto")}
                    role="dialog"
                    aria-modal="true"
                    onclick={{ let m = active_modal.clone(); Callback::from(move |_: MouseEvent| m.set(ActiveModal::None)) }}
                >
                    <div
                        class={classes!(modal_card_class, "my-8")}
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <div class="flex items-center justify-between gap-3">
                            <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">{ "贡献 GPT" }</h2>
                            <button
                                type="button"
                                class={secondary_btn_class}
                                onclick={{ let m = active_modal.clone(); Callback::from(move |_| m.set(ActiveModal::None)) }}
                            >
                                <i class="fas fa-xmark" />
                            </button>
                        </div>
                        <p class="m-0 mt-2 font-mono text-xs text-[var(--muted)]">
                            { "贡献 GPT 生图账号到 gpt2api-rs 池里；服务离线时提交会直接失败，审核通过后会发一把绑定账号和邮箱的 key。" }
                        </p>
                        <form class="mt-4 grid gap-3" onsubmit={on_submit_gpt_account_contribution}>
                            <div class="grid gap-3 sm:grid-cols-2">
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "显示名" }</span>
                                    <input
                                        type="text"
                                        placeholder="my-gpt-image-account"
                                        class={ic}
                                        value={(*gpt_contribution_account_name).clone()}
                                        required=true
                                        oninput={{ let s = gpt_contribution_account_name.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "邮箱" }</span>
                                    <input
                                        type="email"
                                        placeholder="you@example.com"
                                        class={ic}
                                        value={(*gpt_contribution_email).clone()}
                                        required=true
                                        oninput={{ let s = gpt_contribution_email.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                            </div>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "access_token" }</span>
                                <textarea
                                    rows="3"
                                    class={ic_mono_xs}
                                    value={(*gpt_contribution_access_token).clone()}
                                    oninput={{ let s = gpt_contribution_access_token.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "session JSON（可选，和 access_token 二选一）" }</span>
                                <textarea
                                    rows="4"
                                    placeholder="{\"accessToken\":\"...\",\"sessionToken\":\"...\"}"
                                    class={ic_mono_xs}
                                    value={(*gpt_contribution_session_json).clone()}
                                    oninput={{ let s = gpt_contribution_session_json.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "GitHub ID（可选）" }</span>
                                <input
                                    type="text"
                                    placeholder="ackingliu"
                                    class={ic}
                                    value={(*gpt_contribution_github_id).clone()}
                                    oninput={{ let s = gpt_contribution_github_id.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "留言" }</span>
                                <textarea
                                    rows="3"
                                    placeholder="为什么愿意贡献这个 GPT 生图账号"
                                    class={ic_mono_xs}
                                    value={(*gpt_contribution_message).clone()}
                                    required=true
                                    oninput={{ let s = gpt_contribution_message.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <div class="flex justify-end pt-1">
                                <button type="submit" class={primary_btn_class} disabled={*gpt_contribution_submitting}>
                                    <i class={classes!("fas", if *gpt_contribution_submitting { "fa-spinner animate-spin" } else { "fa-image" })} />
                                    <span>{ if *gpt_contribution_submitting { "提交中..." } else { "提交" } }</span>
                                </button>
                            </div>
                            if let Some((msg, is_err)) = (*gpt_contribution_feedback).clone() {
                                if is_err {
                                    <div class={modal_error_class}>{ msg }</div>
                                }
                            }
                        </form>
                    </div>
                </div>
            }

            // Modal: Sponsor
            if *active_modal == ActiveModal::Sponsor {
                <div
                    class={modal_overlay_class}
                    role="dialog"
                    aria-modal="true"
                    onclick={{ let m = active_modal.clone(); Callback::from(move |_: MouseEvent| m.set(ActiveModal::None)) }}
                >
                    <div
                        class={modal_card_class}
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <div class="flex items-center justify-between gap-3">
                            <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">{ "请站长喝杯咖啡" }</h2>
                            <button
                                type="button"
                                class={secondary_btn_class}
                                onclick={{ let m = active_modal.clone(); Callback::from(move |_| m.set(ActiveModal::None)) }}
                            >
                                <i class="fas fa-xmark" />
                            </button>
                        </div>
                        <p class="m-0 mt-2 text-sm leading-relaxed text-[var(--muted)]">
                            { "这是一个公益站点，服务器和带宽都需要持续投入。一杯咖啡的支持就能帮站长分担运维成本，让大家继续免费用下去。" }
                        </p>
                        <p class="m-0 mt-2 font-mono text-[11px] text-[var(--muted)]">
                            { "填写邮箱 → 收到付款说明 → 付款后回复邮件 → 上墙" }
                        </p>
                        <form class="mt-4 grid gap-3" onsubmit={on_submit_sponsor}>
                            <div class="grid gap-3 sm:grid-cols-2">
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "邮箱" }</span>
                                    <input
                                        type="email"
                                        placeholder="you@example.com"
                                        class={ic}
                                        value={(*sponsor_email).clone()}
                                        required=true
                                        oninput={{ let s = sponsor_email.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                                <label class="text-sm">
                                    <span class="font-mono text-xs text-[var(--muted)]">{ "名字（可选）" }</span>
                                    <input
                                        type="text"
                                        placeholder="显示名字"
                                        class={ic}
                                        value={(*sponsor_display_name).clone()}
                                        oninput={{ let s = sponsor_display_name.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                    />
                                </label>
                            </div>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "GitHub ID（可选）" }</span>
                                <input
                                    type="text"
                                    placeholder="ackingliu"
                                    class={ic}
                                    value={(*sponsor_github_id).clone()}
                                    oninput={{ let s = sponsor_github_id.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlInputElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <label class="text-sm">
                                <span class="font-mono text-xs text-[var(--muted)]">{ "留言" }</span>
                                <textarea
                                    rows="3"
                                    placeholder="想对站点说的话"
                                    class={ic_mono_xs}
                                    value={(*sponsor_message).clone()}
                                    required=true
                                    oninput={{ let s = sponsor_message.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<HtmlTextAreaElement>() { s.set(t.value()); } }) }}
                                />
                            </label>
                            <div class="flex items-center justify-between gap-3 pt-1">
                                <span class="font-mono text-[11px] text-[var(--muted)]">{ "邮箱不会公开" }</span>
                                <button type="submit" class={primary_btn_class} disabled={*sponsor_submitting}>
                                    <i class={classes!("fas", if *sponsor_submitting { "fa-spinner animate-spin" } else { "fa-mug-hot" })} />
                                    <span>{ if *sponsor_submitting { "提交中..." } else { "提交" } }</span>
                                </button>
                            </div>
                            if let Some((msg, is_err)) = (*sponsor_feedback).clone() {
                                if is_err {
                                    <div class={modal_error_class}>{ msg }</div>
                                }
                            }
                        </form>
                    </div>
                </div>
            }

            // Warm Floating Toast
            if let Some((message, is_error)) = (*toast).clone() {
                <div class={classes!(
                    "fixed",
                    "bottom-5",
                    "right-5",
                    "z-[90]",
                    "inline-flex",
                    "items-center",
                    "gap-2",
                    "rounded-[10px]",
                    "border",
                    "bg-[var(--surface)]",
                    "px-4",
                    "py-2.5",
                    "font-mono",
                    "text-xs",
                    "shadow-[var(--shadow-lg)]",
                    if is_error {
                        "border-[var(--err,#B8443A)]/40 text-[var(--err,#B8443A)]"
                    } else {
                        "border-[var(--border-strong,var(--border))] text-[var(--text)]"
                    }
                )}>
                    <span class={classes!(
                        "h-2",
                        "w-2",
                        "rounded-full",
                        if is_error { "bg-[var(--err,#B8443A)]" } else { "bg-[var(--ok,#4E7A51)]" }
                    )} />
                    <span>{ message }</span>
                </div>
            }
        </main>
    }
}
