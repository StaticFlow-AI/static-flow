//! Global Kiro model aliases and built-in overrides.

use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    api::{
        create_admin_kiro_model, delete_admin_kiro_model, fetch_admin_kiro_model,
        fetch_admin_kiro_models, patch_admin_kiro_model, AdminKiroModelSummaryView,
        CreateAdminKiroModelInput, PatchAdminKiroModelInput,
    },
    pages::llm_access_shared::confirm_destructive,
    router::Route,
};

#[function_component(AdminKiroGatewayModelsPage)]
pub fn admin_kiro_gateway_models_page() -> Html {
    let models = use_state(Vec::<AdminKiroModelSummaryView>::new);
    let loading = use_state(|| true);
    let saving = use_state(|| false);
    let error = use_state(|| None::<String>);
    let notice = use_state(|| None::<String>);
    let refresh_tick = use_state(|| 0u64);
    let search = use_state(String::new);

    let selected_model = use_state(|| None::<String>);
    let form_builtin = use_state(|| false);
    let model_id = use_state(String::new);
    let display_name = use_state(String::new);
    let target_model_id = use_state(|| "claude-opus-5".to_string());
    let system_prompt = use_state(String::new);
    let enabled = use_state(|| true);

    {
        let models = models.clone();
        let loading = loading.clone();
        let error = error.clone();
        use_effect_with(*refresh_tick, move |_| {
            let models = models.clone();
            let loading = loading.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match fetch_admin_kiro_models().await {
                    Ok(response) => {
                        models.set(response.models);
                        error.set(None);
                    },
                    Err(err) => error.set(Some(err)),
                }
                loading.set(false);
            });
            || ()
        });
    }

    let reload = {
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_| refresh_tick.set((*refresh_tick).saturating_add(1)))
    };

    let on_new = {
        let selected_model = selected_model.clone();
        let form_builtin = form_builtin.clone();
        let model_id = model_id.clone();
        let display_name = display_name.clone();
        let target_model_id = target_model_id.clone();
        let system_prompt = system_prompt.clone();
        let enabled = enabled.clone();
        Callback::from(move |_: ()| {
            selected_model.set(None);
            form_builtin.set(false);
            model_id.set(String::new());
            display_name.set(String::new());
            target_model_id.set("claude-opus-5".to_string());
            system_prompt.set(String::new());
            enabled.set(true);
        })
    };

    let on_save = {
        let selected_model = selected_model.clone();
        let model_id = model_id.clone();
        let display_name = display_name.clone();
        let target_model_id = target_model_id.clone();
        let system_prompt = system_prompt.clone();
        let enabled = enabled.clone();
        let saving = saving.clone();
        let error = error.clone();
        let notice = notice.clone();
        let reload = reload.clone();
        Callback::from(move |_| {
            if *saving {
                return;
            }
            let selected = (*selected_model).clone();
            let model_value = (*model_id).trim().to_string();
            let display_value = (*display_name).trim().to_string();
            let target_value = (*target_model_id).trim().to_string();
            let prompt_value = (*system_prompt).clone();
            let enabled_value = *enabled;
            if model_value.is_empty() || display_value.is_empty() || target_value.is_empty() {
                error.set(Some(
                    "Model ID, display name, and target model are required.".to_string(),
                ));
                return;
            }
            saving.set(true);
            let saving = saving.clone();
            let error = error.clone();
            let notice = notice.clone();
            let reload = reload.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let prompt = (!prompt_value.is_empty()).then_some(prompt_value.as_str());
                let result = if let Some(selected) = selected {
                    patch_admin_kiro_model(&selected, PatchAdminKiroModelInput {
                        display_name: &display_value,
                        target_model_id: &target_value,
                        system_prompt: prompt,
                        clear_system_prompt: prompt.is_none(),
                        enabled: enabled_value,
                    })
                    .await
                } else {
                    create_admin_kiro_model(CreateAdminKiroModelInput {
                        model_id: &model_value,
                        display_name: &display_value,
                        target_model_id: &target_value,
                        system_prompt: prompt,
                        enabled: enabled_value,
                    })
                    .await
                };
                match result {
                    Ok(model) => {
                        notice.set(Some(format!("Saved `{}`.", model.summary.model_id)));
                        error.set(None);
                        reload.emit(());
                    },
                    Err(err) => {
                        error.set(Some(err));
                        notice.set(None);
                    },
                }
                saving.set(false);
            });
        })
    };

    let builtin_targets = models
        .iter()
        .filter(|model| model.builtin)
        .cloned()
        .collect::<Vec<_>>();
    let query = search.trim().to_lowercase();
    let visible_models = models
        .iter()
        .filter(|model| {
            query.is_empty()
                || model.model_id.to_lowercase().contains(&query)
                || model.display_name.to_lowercase().contains(&query)
                || model.target_model_id.to_lowercase().contains(&query)
        })
        .cloned()
        .collect::<Vec<_>>();

    html! {
        <main class={classes!("admin-shell", "min-h-screen", "px-4", "py-6", "lg:px-8")}>
            <div class={classes!("mx-auto", "max-w-7xl", "space-y-4")}>
                <header class={classes!("flex", "flex-wrap", "items-end", "justify-between", "gap-4")}>
                    <div>
                        <div class={classes!("eyebrow")}>{ "Kiro / Models" }</div>
                        <h1 class={classes!("m-0", "text-xl", "font-bold", "tracking-tight")}>{ "Model Registry" }</h1>
                        <p class={classes!("mt-1", "text-sm", "text-[var(--muted-foreground)]")}>{ "Global aliases override per-key model maps. Custom system prompts are appended last." }</p>
                    </div>
                    <div class={classes!("bar-actions")}>
                        <Link<Route> to={Route::AdminKiroGateway} classes={classes!("linkbtn")}>{ "Kiro Overview" }</Link<Route>>
                        <button type="button" class={classes!("linkbtn")} onclick={{
                            let on_new = on_new.clone();
                            Callback::from(move |_| on_new.emit(()))
                        }}>{ "New Model" }</button>
                        <button type="button" class={classes!("primary")} disabled={*loading} onclick={{
                            let reload = reload.clone();
                            Callback::from(move |_| reload.emit(()))
                        }}>{ if *loading { "Loading..." } else { "Refresh" } }</button>
                    </div>
                </header>

                if let Some(message) = (*notice).clone() {
                    <div class={classes!("okline", "text-sm")}>{ message }</div>
                }
                if let Some(message) = (*error).clone() {
                    <div class={classes!("errorline", "text-sm")}>{ message }</div>
                }

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head")}>
                        <h2>{ if selected_model.is_some() { "Edit Model" } else { "New Custom Model" } }</h2>
                        <span class={classes!("text-xs", "text-[var(--muted-foreground)]")}>{ format!("Prompt size: {} bytes / 1 MiB", system_prompt.len()) }</span>
                    </div>
                    <div class={classes!("panel-body", "grid", "gap-3", "lg:grid-cols-2")}>
                        <label class={classes!("grid", "gap-1", "text-xs")}>
                            { "Model ID" }
                            <input class={classes!("mono")} disabled={selected_model.is_some()} value={(*model_id).clone()} oninput={{
                                let model_id = model_id.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    model_id.set(input.value());
                                })
                            }} />
                        </label>
                        <label class={classes!("grid", "gap-1", "text-xs")}>
                            { "Display name" }
                            <input value={(*display_name).clone()} oninput={{
                                let display_name = display_name.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    display_name.set(input.value());
                                })
                            }} />
                        </label>
                        <label class={classes!("grid", "gap-1", "text-xs")}>
                            { "Target built-in model" }
                            <select value={(*target_model_id).clone()} onchange={{
                                let target_model_id = target_model_id.clone();
                                Callback::from(move |event: Event| {
                                    let input: HtmlSelectElement = event.target_unchecked_into();
                                    target_model_id.set(input.value());
                                })
                            }}>
                                { for builtin_targets.iter().map(|model| html! {
                                    <option value={model.model_id.clone()} selected={*target_model_id == model.model_id}>{ format!("{} · {}", model.model_id, model.display_name) }</option>
                                }) }
                            </select>
                        </label>
                        <label class={classes!("flex", "items-center", "gap-2", "self-end", "text-sm")}>
                            <input type="checkbox" checked={*enabled} disabled={*form_builtin} onchange={{
                                let enabled = enabled.clone();
                                Callback::from(move |event: Event| {
                                    let input: HtmlInputElement = event.target_unchecked_into();
                                    enabled.set(input.checked());
                                })
                            }} />
                            { if *form_builtin { "Built-in model (always enabled)" } else { "Enabled" } }
                        </label>
                        <label class={classes!("grid", "gap-1", "text-xs", "lg:col-span-2")}>
                            { "System prompt · trusted, appended after user and built-in instructions" }
                            <textarea class={classes!("mono", "min-h-80", "text-xs")} value={(*system_prompt).clone()} oninput={{
                                let system_prompt = system_prompt.clone();
                                Callback::from(move |event: InputEvent| {
                                    let input: HtmlTextAreaElement = event.target_unchecked_into();
                                    system_prompt.set(input.value());
                                })
                            }} />
                        </label>
                        <div class={classes!("flex", "flex-wrap", "gap-2", "lg:col-span-2")}>
                            <button type="button" class={classes!("primary")} disabled={*saving} onclick={on_save}>{ if *saving { "Saving..." } else { "Save Model" } }</button>
                            if selected_model.is_some() && !*form_builtin {
                                <button type="button" class={classes!("danger")} disabled={*saving} onclick={{
                                    let selected_model = selected_model.clone();
                                    let notice = notice.clone();
                                    let error = error.clone();
                                    let reload = reload.clone();
                                    let on_new = on_new.clone();
                                    Callback::from(move |_| {
                                        let Some(model_id) = (*selected_model).clone() else { return; };
                                        if !confirm_destructive(&format!("Delete custom model `{model_id}`?")) { return; }
                                        let notice = notice.clone();
                                        let error = error.clone();
                                        let reload = reload.clone();
                                        let on_new = on_new.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            match delete_admin_kiro_model(&model_id).await {
                                                Ok(()) => {
                                                    notice.set(Some(format!("Deleted `{model_id}`.")));
                                                    error.set(None);
                                                    on_new.emit(());
                                                    reload.emit(());
                                                },
                                                Err(err) => error.set(Some(err)),
                                            }
                                        });
                                    })
                                }}>{ "Delete" }</button>
                            }
                        </div>
                    </div>
                </section>

                <section class={classes!("panel")}>
                    <div class={classes!("panel-head", "flex", "flex-wrap", "items-center", "justify-between", "gap-3")}>
                        <h2>{ format!("Models · {}", models.len()) }</h2>
                        <input placeholder="Search model, display name, or target" value={(*search).clone()} oninput={{
                            let search = search.clone();
                            Callback::from(move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                search.set(input.value());
                            })
                        }} />
                    </div>
                    <div class={classes!("divide-y", "divide-[var(--border)]")}>
                        { for visible_models.into_iter().map(|model| {
                            let edit_id = model.model_id.clone();
                            let selected_model = selected_model.clone();
                            let form_builtin = form_builtin.clone();
                            let model_id = model_id.clone();
                            let display_name = display_name.clone();
                            let target_model_id = target_model_id.clone();
                            let system_prompt = system_prompt.clone();
                            let enabled = enabled.clone();
                            let error = error.clone();
                            let on_edit = Callback::from(move |_| {
                                let edit_id = edit_id.clone();
                                let selected_model = selected_model.clone();
                                let form_builtin = form_builtin.clone();
                                let model_id = model_id.clone();
                                let display_name = display_name.clone();
                                let target_model_id = target_model_id.clone();
                                let system_prompt = system_prompt.clone();
                                let enabled = enabled.clone();
                                let error = error.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    match fetch_admin_kiro_model(&edit_id).await {
                                        Ok(detail) => {
                                            selected_model.set(Some(detail.summary.model_id.clone()));
                                            form_builtin.set(detail.summary.builtin);
                                            model_id.set(detail.summary.model_id);
                                            display_name.set(detail.summary.display_name);
                                            target_model_id.set(detail.summary.target_model_id);
                                            system_prompt.set(detail.system_prompt.unwrap_or_default());
                                            enabled.set(detail.summary.enabled);
                                            error.set(None);
                                        },
                                        Err(err) => error.set(Some(err)),
                                    }
                                });
                            });
                            html! {
                                <div class={classes!("flex", "flex-wrap", "items-center", "justify-between", "gap-3", "px-4", "py-3")}>
                                    <div class={classes!("min-w-0")}>
                                        <div class={classes!("flex", "flex-wrap", "items-center", "gap-2")}>
                                            <strong class={classes!("mono", "text-sm")}>{ model.model_id.clone() }</strong>
                                            <span class={classes!("badge")}>{ if model.builtin { "built-in" } else { "custom" } }</span>
                                            if !model.enabled { <span class={classes!("badge", "warn")}>{ "disabled" }</span> }
                                            if model.overridden { <span class={classes!("badge", "ok")}>{ "override" }</span> }
                                        </div>
                                        <div class={classes!("mt-1", "text-sm")}>{ model.display_name.clone() }</div>
                                        <div class={classes!("mt-1", "mono", "text-xs", "text-[var(--muted-foreground)]")}>{ format!("→ {} · prompt {} bytes · sha256 {}", model.target_model_id, model.system_prompt_bytes, model.system_prompt_sha256.as_deref().unwrap_or("-")) }</div>
                                    </div>
                                    <button type="button" class={classes!("linkbtn")} onclick={on_edit}>{ "Edit" }</button>
                                </div>
                            }
                        }) }
                    </div>
                </section>
            </div>
        </main>
    }
}
