//! Public Kiro access page shown to end users.

use gloo_timers::callback::Timeout;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    api::{fetch_kiro_access, KiroAccessResponse},
    router::Route,
};

const CLAUDE_CODE_ENV_HINTS: [(&str, &str); 4] = [
    ("DISABLE_TELEMETRY=1", "禁用 Datadog + 1P 事件 + 反馈调查"),
    (
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1",
        "禁用所有非必要网络（遥测 + 更新 + GrowthBook）",
    ),
    ("CLAUDE_CODE_USE_BEDROCK=1", "使用 AWS Bedrock（自动禁用所有分析）"),
    ("CLAUDE_CODE_USE_VERTEX=1", "使用 GCP Vertex（自动禁用所有分析）"),
];

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

fn resolve_base_url(access: &KiroAccessResponse) -> String {
    if access.base_url.starts_with("http://") || access.base_url.starts_with("https://") {
        return access.base_url.clone();
    }
    let origin = web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_default();
    if origin.is_empty() {
        access.base_url.clone()
    } else {
        format!("{origin}{}", access.gateway_path)
    }
}

#[function_component(KiroAccessPage)]
/// Render the public Kiro access page, including connection examples and base
/// URL info.
pub fn kiro_access_page() -> Html {
    let access = use_state(|| None::<KiroAccessResponse>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let copied = use_state(|| None::<String>);
    let copy_timeout = use_mut_ref(|| None::<Timeout>);
    // 0 = Claude Code env, 1 = curl
    let active_tab = use_state(|| 0u8);

    let reload_access = {
        let access = access.clone();
        let loading = loading.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let access = access.clone();
            let loading = loading.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);
                match fetch_kiro_access().await {
                    Ok(response) => access.set(Some(response)),
                    Err(err) => error.set(Some(err)),
                }
                loading.set(false);
            });
        })
    };

    {
        let reload_access = reload_access.clone();
        use_effect_with((), move |_| {
            reload_access.emit(());
            || ()
        });
    }

    let on_copy = {
        let copied = copied.clone();
        let copy_timeout = copy_timeout.clone();
        Callback::from(move |value: String| {
            copy_text(&value);
            copied.set(Some("已复制到剪贴板".to_string()));
            let copied = copied.clone();
            *copy_timeout.borrow_mut() = Some(Timeout::new(2_000, move || {
                copied.set(None);
            }));
        })
    };

    let access_value = (*access).clone();
    let resolved_base = access_value
        .as_ref()
        .map(resolve_base_url)
        .unwrap_or_else(|| "<loading>".to_string());
    let example_secret = "<your-kiro-key>".to_string();
    let claude_env_example = format!(
        "export ANTHROPIC_BASE_URL=\"{resolved_base}\"\nexport \
         ANTHROPIC_AUTH_TOKEN=\"{example_secret}\"\nclaude"
    );
    let curl_example = format!(
        "curl {resolved_base}/v1/messages \\\n  -H 'x-api-key: {example_secret}' \\\n  -H \
         'anthropic-version: 2023-06-01' \\\n  -H 'content-type: application/json' \\\n  -d \
         '{{\n    \"model\": \"claude-sonnet-4-6\",\n    \"max_tokens\": 128,\n    \"messages\": \
         [\n      {{\"role\": \"user\", \"content\": \"Reply exactly OK.\"}}\n    ]\n  }}'"
    );

    let secondary_btn_class = "sf-btn sf-btn--secondary inline-flex items-center gap-1.5 \
                               rounded-[8px] border border-[var(--border-strong,var(--border))] \
                               bg-transparent px-3 py-1.5 font-mono text-xs text-[var(--text)] \
                               no-underline transition-colors hover:bg-[var(--surface-alt)] \
                               disabled:cursor-not-allowed disabled:opacity-50";

    html! {
        <main class="mx-auto max-w-[960px] space-y-6 px-4 pb-20 pt-10 sm:px-6 md:pt-14">
            // Editorial Header Card
            <section class="sf-card rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-6 sm:p-7">
                <div class="flex flex-wrap items-start justify-between gap-4">
                    <div>
                        <div class="mb-2 flex items-center gap-2.5">
                            <span class="sf-chip sf-chip--active inline-flex items-center rounded-full border border-[var(--primary)]/35 bg-[var(--primary-soft,rgba(201,100,66,0.12))] px-2.5 py-0.5 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--primary)]">
                                { "KIRO GATEWAY" }
                            </span>
                            <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-[var(--muted)]">
                                { "ANTHROPIC MESSAGES COMPATIBLE" }
                            </span>
                        </div>
                        <h1 class="m-0 font-serif text-[clamp(1.75rem,3vw,2.375rem)] font-normal leading-[1.15] tracking-[-0.02em] text-[var(--text)]">
                            { "Kiro Access · 网关接入" }
                        </h1>
                    </div>

                    <div class="flex flex-wrap items-center gap-2">
                        <button
                            type="button"
                            class={secondary_btn_class}
                            onclick={{
                                let reload_access = reload_access.clone();
                                Callback::from(move |_| reload_access.emit(()))
                            }}
                            disabled={*loading}
                            title="刷新接入信息"
                            aria-label="刷新接入信息"
                        >
                            <i class={classes!("fas", if *loading { "fa-spinner animate-spin" } else { "fa-rotate-right" })} />
                            <span>{ "刷新" }</span>
                        </button>
                        <button
                            type="button"
                            class="sf-btn sf-btn--primary inline-flex items-center gap-1.5 rounded-[8px] bg-[var(--primary)] px-3.5 py-1.5 font-mono text-xs font-medium text-[var(--on-primary,#FAF9F5)] transition-colors hover:bg-[var(--primary-hover,#B85735)]"
                            onclick={{
                                let on_copy = on_copy.clone();
                                let resolved_base = resolved_base.clone();
                                Callback::from(move |_| on_copy.emit(resolved_base.clone()))
                            }}
                        >
                            <i class="fas fa-copy" />
                            <span>{ "复制 Base URL" }</span>
                        </button>
                        <Link<Route> to={Route::LlmAccess} classes={secondary_btn_class}>
                            <span>{ "Codex Gateway" }</span>
                        </Link<Route>>
                        <Link<Route>
                            to={Route::AdminKiroGateway}
                            classes={secondary_btn_class}
                        >
                            <i class="fas fa-sliders" />
                            <span>{ "控制台" }</span>
                        </Link<Route>>
                    </div>
                </div>

                <div class="mt-4 rounded-[8px] border border-[var(--border)] bg-[var(--surface-sunk,var(--surface-alt))] px-3.5 py-2.5">
                    <div class="font-mono text-[10px] uppercase tracking-[0.12em] text-[var(--muted)]">
                        { "ENDPOINT BASE URL" }
                    </div>
                    <code class="mt-1 block break-all font-mono text-sm text-[var(--text)]">
                        { resolved_base.clone() }
                    </code>
                </div>

                if *loading {
                    <div class="mt-4 font-mono text-xs text-[var(--muted)]">{ "正在加载接入配置..." }</div>
                } else if let Some(err) = (*error).clone() {
                    <div class="mt-4 rounded-[8px] border border-[var(--err,#B8443A)]/35 bg-[var(--err-soft,rgba(184,68,58,0.10))] px-3.5 py-2.5 font-mono text-xs text-[var(--err,#B8443A)]">
                        { err }
                    </div>
                }
            </section>

            // Code Examples (Tabbed)
            <section class="sf-card rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-6">
                <div class="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border)] pb-3">
                    <div class="flex items-center gap-4">
                        <button
                            type="button"
                            class={classes!(
                                "border-b-[1.5px]",
                                "pb-2",
                                "-mb-[13px]",
                                "font-mono",
                                "text-xs",
                                "transition-colors",
                                if *active_tab == 0 {
                                    "border-[var(--primary)] font-medium text-[var(--text)]"
                                } else {
                                    "border-transparent text-[var(--muted)] hover:text-[var(--text)]"
                                }
                            )}
                            onclick={{ let active_tab = active_tab.clone(); Callback::from(move |_| active_tab.set(0)) }}
                        >
                            { "Claude Code" }
                        </button>
                        <button
                            type="button"
                            class={classes!(
                                "border-b-[1.5px]",
                                "pb-2",
                                "-mb-[13px]",
                                "font-mono",
                                "text-xs",
                                "transition-colors",
                                if *active_tab == 1 {
                                    "border-[var(--primary)] font-medium text-[var(--text)]"
                                } else {
                                    "border-transparent text-[var(--muted)] hover:text-[var(--text)]"
                                }
                            )}
                            onclick={{ let active_tab = active_tab.clone(); Callback::from(move |_| active_tab.set(1)) }}
                        >
                            { "curl" }
                        </button>
                    </div>

                    <button
                        type="button"
                        class={secondary_btn_class}
                        onclick={{
                            let on_copy = on_copy.clone();
                            let claude_env_example = claude_env_example.clone();
                            let curl_example = curl_example.clone();
                            let active_tab = active_tab.clone();
                            Callback::from(move |_| {
                                let text = if *active_tab == 0 {
                                    claude_env_example.clone()
                                } else {
                                    curl_example.clone()
                                };
                                on_copy.emit(text);
                            })
                        }}
                    >
                        <i class="fas fa-copy" />
                        <span>{ "复制代码" }</span>
                    </button>
                </div>

                <pre class="mt-4 overflow-x-auto rounded-[8px] border border-[var(--border)] bg-[var(--surface-sunk,var(--surface-alt))] p-4 font-mono text-xs leading-relaxed text-[var(--text)]">
                    <code>{ if *active_tab == 0 { claude_env_example } else { curl_example } }</code>
                </pre>

                <div class="mt-3 flex flex-wrap items-center gap-2 font-mono text-[11px] text-[var(--muted)]">
                    <span class="rounded border border-[var(--border)] bg-[var(--surface-alt)] px-2 py-0.5">{ "/v1/models" }</span>
                    <span class="rounded border border-[var(--border)] bg-[var(--surface-alt)] px-2 py-0.5">{ "/v1/messages" }</span>
                    <span class="rounded border border-[var(--border)] bg-[var(--surface-alt)] px-2 py-0.5">{ "/v1/messages/count_tokens" }</span>
                    <span class="rounded border border-[var(--border)] bg-[var(--surface-alt)] px-2 py-0.5">{ "/cc/v1/messages" }</span>
                </div>
            </section>

            // Recommended Settings
            <section class="sf-card rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-6">
                <div class="sf-eyebrow mb-1 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--muted)]">
                    { "RECOMMENDED CONFIGURATION" }
                </div>
                <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">
                    { "推荐配置 ~/.claude/settings.json" }
                </h2>
                <ul class="m-0 mt-4 list-none space-y-3 p-0">
                    <li class="rounded-[8px] border border-[var(--border)] bg-[var(--surface-alt)] p-3.5">
                        <code class="font-mono text-xs text-[var(--primary)]">
                            { r#""skipWebFetchPreflight": true"# }
                        </code>
                        <p class="mb-0 mt-1 text-xs leading-relaxed text-[var(--muted)]">
                            { "允许 Claude Code 正常使用 WebFetch 工具，跳过预检限制。" }
                        </p>
                    </li>
                    <li class="rounded-[8px] border border-[var(--border)] bg-[var(--surface-alt)] p-3.5">
                        <code class="font-mono text-xs text-[var(--primary)]">
                            { r#""showClearContextOnPlanAccept": true"# }
                        </code>
                        <p class="mb-0 mt-1 text-xs leading-relaxed text-[var(--muted)]">
                            { "Plan Mode 批准时保留旧版 context 感知能力，显示清除上下文选项。" }
                        </p>
                    </li>
                </ul>
            </section>

            // Optional Env Flags
            <section class="sf-card rounded-[12px] border border-[var(--border)] bg-[var(--surface)] p-6">
                <div class="sf-eyebrow mb-1 font-mono text-[11px] uppercase tracking-[0.14em] text-[var(--muted)]">
                    { "ENVIRONMENT FLAGS" }
                </div>
                <h2 class="m-0 font-serif text-xl font-normal text-[var(--text)]">
                    { "可选 Claude Code 环境变量" }
                </h2>
                <div class="mt-4 grid gap-3 sm:grid-cols-2">
                    { for CLAUDE_CODE_ENV_HINTS.iter().map(|(key, description)| html! {
                        <article class="rounded-[8px] border border-[var(--border)] bg-[var(--surface-alt)] p-3.5">
                            <code class="block break-all font-mono text-xs text-[var(--primary)]">
                                { *key }
                            </code>
                            <p class="mb-0 mt-1.5 text-xs leading-relaxed text-[var(--muted)]">
                                { *description }
                            </p>
                        </article>
                    }) }
                </div>
            </section>

            // Warm floating copy toast
            if let Some(message) = (*copied).clone() {
                <div class="fixed bottom-6 right-6 z-[80] inline-flex items-center gap-2 rounded-[10px] border border-[var(--border-strong,var(--border))] bg-[var(--surface)] px-4 py-2.5 font-mono text-xs text-[var(--text)] shadow-[var(--shadow-lg)]">
                    <span class="h-2 w-2 rounded-full bg-[var(--ok,#4E7A51)]" />
                    <span>{ message }</span>
                </div>
            }
        </main>
    }
}
