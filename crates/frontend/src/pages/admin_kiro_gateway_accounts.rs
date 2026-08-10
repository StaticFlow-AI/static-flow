//! Kiro account onboarding page (`/admin/kiro-gateway/accounts/manage`).
//!
//! Hosts the local-CLI-auth import and manual-account forms plus summary
//! counts, in the `.admin-shell` design system. Status browsing lives on the
//! dedicated paginated status page.

use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use super::admin_kiro_gateway::{
    kiro_account_status_abnormal_href, kiro_account_status_cta_text, kiro_account_status_route,
    kiro_pool_strategy_description, kiro_pool_strategy_options, normalized_str_option,
};
use crate::{
    api::{
        create_admin_kiro_manual_account, fetch_admin_kiro_accounts_page,
        import_admin_kiro_account, AdminAccountsSummaryView, CreateManualKiroAccountInput,
    },
    llm_gateway_contracts,
    router::Route,
};

/// Admin-shell text field bound to a string state handle.
fn text_field(
    label: &str,
    state: &UseStateHandle<String>,
    wide: bool,
    hint: Option<&'static str>,
) -> Html {
    let state_handle = state.clone();
    html! {
        <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]", wide.then_some("lg:col-span-2"))}>
            { label }
            <input
                class={classes!("mono")}
                value={(**state).clone()}
                oninput={Callback::from(move |event: InputEvent| {
                    let input: HtmlInputElement = event.target_unchecked_into();
                    state_handle.set(input.value());
                })}
            />
            if let Some(hint) = hint {
                <span class={classes!("text-[11px]", "text-[var(--faint)]")}>{ hint }</span>
            }
        </label>
    }
}

#[function_component(AdminKiroGatewayAccountsPage)]
pub fn admin_kiro_gateway_accounts_page() -> Html {
    let summary = use_state(AdminAccountsSummaryView::default);
    let loading = use_state(|| true);
    let flash = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0u32);

    let import_name = use_state(|| "default".to_string());
    let import_sqlite_path = use_state(String::new);
    let import_scheduler_max = use_state(|| "1".to_string());
    let import_scheduler_rpm = use_state(|| "5".to_string());
    let import_scheduler_min = use_state(|| "0".to_string());
    let importing_local = use_state(|| false);

    let manual_form_expanded = use_state(|| false);
    let manual_name = use_state(String::new);
    let manual_auth_method = use_state(|| "social".to_string());
    let manual_access_token = use_state(String::new);
    let manual_refresh_token = use_state(String::new);
    let manual_profile_arn = use_state(String::new);
    let manual_expires_at = use_state(String::new);
    let manual_client_id = use_state(String::new);
    let manual_client_secret = use_state(String::new);
    let manual_region = use_state(|| "us-east-1".to_string());
    let manual_auth_region = use_state(|| "us-east-1".to_string());
    let manual_api_region = use_state(|| "us-east-1".to_string());
    let manual_machine_id = use_state(String::new);
    let manual_provider = use_state(String::new);
    let manual_email = use_state(String::new);
    let manual_subscription_title = use_state(String::new);
    let manual_scheduler_max = use_state(|| "1".to_string());
    let manual_scheduler_rpm = use_state(|| "5".to_string());
    let manual_scheduler_min = use_state(|| "0".to_string());
    let manual_minimum_remaining_credits_before_block = use_state(|| "0".to_string());
    let manual_pool_strategy = use_state(llm_gateway_contracts::default_kiro_pool_strategy);
    let manual_disabled = use_state(|| false);
    let creating_manual = use_state(|| false);

    let notify = {
        let flash = flash.clone();
        let error = error.clone();
        Callback::from(move |(message, is_error): (String, bool)| {
            if is_error {
                error.set(Some(message));
                flash.set(None);
            } else {
                flash.set(Some(message));
                error.set(None);
            }
        })
    };

    let on_reload = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_| refresh_tick.set(refresh_tick.wrapping_add(1)))
    };

    {
        let summary = summary.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let summary = summary.clone();
            let loading = loading.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match fetch_admin_kiro_accounts_page(1, 0).await {
                    Ok(response) => summary.set(response.summary),
                    Err(err) => error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_import_local = {
        let import_name = import_name.clone();
        let import_sqlite_path = import_sqlite_path.clone();
        let import_scheduler_max = import_scheduler_max.clone();
        let import_scheduler_rpm = import_scheduler_rpm.clone();
        let import_scheduler_min = import_scheduler_min.clone();
        let notify = notify.clone();
        let on_reload = on_reload.clone();
        let importing_local = importing_local.clone();
        Callback::from(move |_| {
            if *importing_local {
                return;
            }
            let import_name = (*import_name).clone();
            let import_sqlite_path = (*import_sqlite_path).clone();
            let import_scheduler_max = (*import_scheduler_max).clone();
            let import_scheduler_rpm = (*import_scheduler_rpm).clone();
            let import_scheduler_min = (*import_scheduler_min).clone();
            let notify = notify.clone();
            let on_reload = on_reload.clone();
            let importing_local = importing_local.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(parsed_max) = import_scheduler_max.trim().parse::<u64>() else {
                    notify.emit((
                        "Import max concurrency must be a valid integer.".to_string(),
                        true,
                    ));
                    return;
                };
                let Ok(parsed_min) = import_scheduler_min.trim().parse::<u64>() else {
                    notify.emit((
                        "Import min start interval must be a valid integer.".to_string(),
                        true,
                    ));
                    return;
                };
                let Ok(parsed_rpm) = import_scheduler_rpm.trim().parse::<u64>() else {
                    notify.emit(("Import RPM must be a valid integer.".to_string(), true));
                    return;
                };
                if parsed_rpm == 0 {
                    notify.emit(("Import RPM must be greater than zero.".to_string(), true));
                    return;
                }
                importing_local.set(true);
                match import_admin_kiro_account(
                    Some(import_name.as_str()),
                    if import_sqlite_path.trim().is_empty() {
                        None
                    } else {
                        Some(import_sqlite_path.as_str())
                    },
                    Some(parsed_max),
                    Some(parsed_rpm),
                    Some(parsed_min),
                )
                .await
                {
                    Ok(account) => {
                        notify
                            .emit((format!("Imported local Kiro auth `{}`.", account.name), false));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        notify.emit((format!("Failed to import local Kiro auth.\n{err}"), true));
                    },
                }
                importing_local.set(false);
            });
        })
    };

    let on_create_manual = {
        let manual_name = manual_name.clone();
        let manual_auth_method = manual_auth_method.clone();
        let manual_access_token = manual_access_token.clone();
        let manual_refresh_token = manual_refresh_token.clone();
        let manual_profile_arn = manual_profile_arn.clone();
        let manual_expires_at = manual_expires_at.clone();
        let manual_client_id = manual_client_id.clone();
        let manual_client_secret = manual_client_secret.clone();
        let manual_region = manual_region.clone();
        let manual_auth_region = manual_auth_region.clone();
        let manual_api_region = manual_api_region.clone();
        let manual_machine_id = manual_machine_id.clone();
        let manual_provider = manual_provider.clone();
        let manual_email = manual_email.clone();
        let manual_subscription_title = manual_subscription_title.clone();
        let manual_scheduler_max = manual_scheduler_max.clone();
        let manual_scheduler_rpm = manual_scheduler_rpm.clone();
        let manual_scheduler_min = manual_scheduler_min.clone();
        let manual_minimum_remaining_credits_before_block =
            manual_minimum_remaining_credits_before_block.clone();
        let manual_pool_strategy = manual_pool_strategy.clone();
        let manual_disabled = manual_disabled.clone();
        let notify = notify.clone();
        let on_reload = on_reload.clone();
        let creating_manual = creating_manual.clone();
        Callback::from(move |_| {
            if *creating_manual {
                return;
            }
            let notify = notify.clone();
            let on_reload = on_reload.clone();
            let creating_manual = creating_manual.clone();
            let Ok(parsed_max) = (*manual_scheduler_max).trim().parse::<u64>() else {
                notify.emit((
                    "Manual account max concurrency must be a valid integer.".to_string(),
                    true,
                ));
                return;
            };
            let Ok(parsed_min) = (*manual_scheduler_min).trim().parse::<u64>() else {
                notify.emit((
                    "Manual account min start interval must be a valid integer.".to_string(),
                    true,
                ));
                return;
            };
            let Ok(parsed_rpm) = (*manual_scheduler_rpm).trim().parse::<u64>() else {
                notify.emit(("Manual account RPM must be a valid integer.".to_string(), true));
                return;
            };
            if parsed_rpm == 0 {
                notify.emit(("Manual account RPM must be greater than zero.".to_string(), true));
                return;
            }
            let parsed_minimum_remaining_credits_before_block =
                match (*manual_minimum_remaining_credits_before_block)
                    .trim()
                    .parse::<f64>()
                {
                    Ok(value) if value.is_finite() && value >= 0.0 => value,
                    _ => {
                        notify.emit((
                            "Manual account minimum remaining credits must be a non-negative \
                             number."
                                .to_string(),
                            true,
                        ));
                        return;
                    },
                };
            let input = CreateManualKiroAccountInput {
                name: (*manual_name).trim().to_string(),
                access_token: normalized_str_option(&manual_access_token),
                refresh_token: normalized_str_option(&manual_refresh_token),
                profile_arn: normalized_str_option(&manual_profile_arn),
                expires_at: normalized_str_option(&manual_expires_at),
                auth_method: normalized_str_option(&manual_auth_method),
                client_id: normalized_str_option(&manual_client_id),
                client_secret: normalized_str_option(&manual_client_secret),
                region: normalized_str_option(&manual_region),
                auth_region: normalized_str_option(&manual_auth_region),
                api_region: normalized_str_option(&manual_api_region),
                machine_id: normalized_str_option(&manual_machine_id),
                provider: normalized_str_option(&manual_provider),
                email: normalized_str_option(&manual_email),
                subscription_title: normalized_str_option(&manual_subscription_title),
                kiro_channel_max_concurrency: Some(parsed_max),
                kiro_channel_rpm_limit: Some(parsed_rpm),
                kiro_channel_min_start_interval_ms: Some(parsed_min),
                minimum_remaining_credits_before_block: Some(
                    parsed_minimum_remaining_credits_before_block,
                ),
                manual_usage_limit: None,
                pool_strategy: Some((*manual_pool_strategy).clone()),
                disabled: *manual_disabled,
            };
            wasm_bindgen_futures::spawn_local(async move {
                creating_manual.set(true);
                match create_admin_kiro_manual_account(&input).await {
                    Ok(account) => {
                        notify.emit((
                            format!("Saved manual Kiro account `{}`.", account.name),
                            false,
                        ));
                        on_reload.emit(());
                    },
                    Err(err) => {
                        notify.emit((format!("Failed to save manual Kiro account.\n{err}"), true));
                    },
                }
                creating_manual.set(false);
            });
        })
    };

    let disabled_count = summary.disabled_count;

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "Kiro Gateway" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Accounts" }</h1>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminKiroGateway} classes={classes!("linkbtn")}>{ "Overview" }</Link<Route>>
                        <a href={kiro_account_status_abnormal_href()} class={classes!("linkbtn")}>{ "Abnormal Accounts" }</a>
                        <Link<Route> to={kiro_account_status_route()} classes={classes!("linkbtn")}>{ kiro_account_status_cta_text() }</Link<Route>>
                        <button type="button" class={classes!("primary")} disabled={*loading} onclick={{
                            let on_reload = on_reload.clone();
                            Callback::from(move |_| on_reload.emit(()))
                        }}>
                            { if *loading { "Loading..." } else { "Refresh" } }
                        </button>
                    </div>
                </header>

                if let Some(message) = (*flash).clone() {
                    <div class={classes!("okline", "text-sm")}>{ message }</div>
                }
                if let Some(err) = (*error).clone() {
                    <div class={classes!("errorline", "text-sm")}>{ err }</div>
                }

                <section class={classes!("panel")}>
                    <div class={classes!("stat-strip")}>
                        <div class={classes!("stat")}>
                            <span>{ "Imported Accounts" }</span>
                            <b>{ summary.total }</b>
                        </div>
                        <div class={classes!("stat", (disabled_count > 0).then_some("warn"))}>
                            <span>{ "Disabled" }</span>
                            <b>{ disabled_count }</b>
                        </div>
                    </div>
                    if summary.total == 0 && !*loading {
                        <div class={classes!("empty")}>
                            <span>{ "当前还没有导入任何 Kiro 账号" }</span>
                            <span class={classes!("text-xs")}>{ "可以从下面的 SQLite 导入，或手动填写字段生成一个账号文件。" }</span>
                        </div>
                    }
                </section>

                <section class={classes!("grid", "gap-4", "xl:grid-cols-2", "items-start")}>
                    <div class={classes!("panel")}>
                        <div class={classes!("panel-head")}>
                            <h2>{ "Import Local Kiro CLI Auth" }</h2>
                        </div>
                        <div class={classes!("panel-body", "space-y-3")}>
                            { text_field("Account Name", &import_name, false, None) }
                            { text_field("SQLite Path Override", &import_sqlite_path, false, Some("默认 ~/.local/share/kiro-cli/data.sqlite3")) }
                            <div class={classes!("grid", "gap-3", "md:grid-cols-3")}>
                                { text_field("Max Concurrency", &import_scheduler_max, false, None) }
                                { text_field("RPM", &import_scheduler_rpm, false, None) }
                                { text_field("Min Start Interval Ms", &import_scheduler_min, false, None) }
                            </div>
                            <button type="button" class={classes!("primary")} onclick={on_import_local} disabled={*importing_local}>
                                { if *importing_local { "Importing..." } else { "Import Local Auth" } }
                            </button>
                        </div>
                    </div>

                    <div class={classes!("panel")}>
                        <div class={classes!("panel-head")}>
                            <div>
                                <h2>{ "Create Manual Kiro Account" }</h2>
                                <p class={classes!("m-0", "mt-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                    { "适合已有 refresh token / profileArn / IDC 凭据的场景。" }
                                </p>
                            </div>
                            <button
                                type="button"
                                class={classes!("ghost")}
                                onclick={{
                                    let manual_form_expanded = manual_form_expanded.clone();
                                    Callback::from(move |_| manual_form_expanded.set(!*manual_form_expanded))
                                }}
                            >
                                { if *manual_form_expanded { "收起" } else { "展开" } }
                            </button>
                        </div>
                        if *manual_form_expanded {
                            <div class={classes!("panel-body")}>
                                <div class={classes!("grid", "gap-3", "lg:grid-cols-2")}>
                                    { text_field("Name", &manual_name, false, None) }
                                    <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                        { "Auth Method" }
                                        <select value={(*manual_auth_method).clone()} onchange={{
                                            let manual_auth_method = manual_auth_method.clone();
                                            Callback::from(move |event: Event| {
                                                let input: HtmlSelectElement = event.target_unchecked_into();
                                                manual_auth_method.set(input.value());
                                            })
                                        }}>
                                            <option value="social" selected={(*manual_auth_method).as_str() == "social"}>{ "social" }</option>
                                            <option value="idc" selected={(*manual_auth_method).as_str() == "idc"}>{ "idc" }</option>
                                        </select>
                                    </label>
                                    { text_field("Refresh Token", &manual_refresh_token, true, None) }
                                    { text_field("Access Token", &manual_access_token, true, None) }
                                    { text_field("Profile ARN", &manual_profile_arn, true, None) }
                                    { text_field("Expires At (RFC3339)", &manual_expires_at, false, None) }
                                    { text_field("Provider", &manual_provider, false, None) }
                                    { text_field("Email", &manual_email, false, None) }
                                    { text_field("Subscription Title", &manual_subscription_title, false, None) }
                                    { text_field("Client ID", &manual_client_id, false, None) }
                                    { text_field("Client Secret", &manual_client_secret, false, None) }
                                    { text_field("Region", &manual_region, false, None) }
                                    { text_field("Auth Region", &manual_auth_region, false, None) }
                                    { text_field("API Region", &manual_api_region, false, None) }
                                    { text_field("Machine ID", &manual_machine_id, false, None) }
                                    { text_field("Max Concurrency", &manual_scheduler_max, false, None) }
                                    { text_field("RPM", &manual_scheduler_rpm, false, None) }
                                    { text_field("Min Start Interval Ms", &manual_scheduler_min, false, None) }
                                    { text_field(
                                        "Min Remaining Credits",
                                        &manual_minimum_remaining_credits_before_block,
                                        false,
                                        Some("0 keeps the historic zero-only behavior."),
                                    ) }
                                    <label class={classes!("grid", "gap-1", "text-xs", "text-[var(--muted-foreground)]")}>
                                        { "Pool Strategy" }
                                        <select value={(*manual_pool_strategy).clone()} onchange={{
                                            let manual_pool_strategy = manual_pool_strategy.clone();
                                            Callback::from(move |event: Event| {
                                                if let Some(target) = event.target_dyn_into::<HtmlSelectElement>() {
                                                    manual_pool_strategy.set(target.value());
                                                }
                                            })
                                        }}>
                                            { kiro_pool_strategy_options((*manual_pool_strategy).as_str()) }
                                        </select>
                                        <span class={classes!("text-[11px]", "text-[var(--faint)]")}>
                                            { kiro_pool_strategy_description((*manual_pool_strategy).as_str()) }
                                        </span>
                                    </label>
                                </div>
                                <div class={classes!("mt-4", "flex", "items-center", "gap-4", "flex-wrap")}>
                                    <label class={classes!("inline-flex", "items-center", "gap-2", "text-sm", "text-[var(--muted-foreground)]")}>
                                        <input
                                            type="checkbox"
                                            checked={*manual_disabled}
                                            onchange={{
                                                let manual_disabled = manual_disabled.clone();
                                                Callback::from(move |event: Event| {
                                                    let input: HtmlInputElement = event.target_unchecked_into();
                                                    manual_disabled.set(input.checked());
                                                })
                                            }}
                                        />
                                        { "disabled" }
                                    </label>
                                    <button type="button" class={classes!("primary")} onclick={on_create_manual} disabled={*creating_manual}>
                                        { if *creating_manual { "Saving..." } else { "Save Manual Account" } }
                                    </button>
                                </div>
                            </div>
                        }
                    </div>
                </section>
            </div>
        </main>
    }
}
