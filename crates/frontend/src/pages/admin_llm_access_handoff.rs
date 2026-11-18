use yew::prelude::*;
use yew_router::prelude::Link;

use crate::{
    pages::admin::shell::{AdminNav, AdminPageHeader, AdminShell},
    router::Route,
};

const DEFAULT_LLM_ACCESS_FRONTEND_BASE: &str = "http://127.0.0.1:19191";

#[derive(Properties, PartialEq)]
pub struct AdminLlmAccessHandoffProps {
    pub destination: AttrValue,
    pub workspace: AttrValue,
}

pub fn llm_access_frontend_url(destination: &str) -> String {
    let base = option_env!("STATICFLOW_LLM_ACCESS_FRONTEND_BASE")
        .unwrap_or(DEFAULT_LLM_ACCESS_FRONTEND_BASE)
        .trim_end_matches('/');
    format!("{base}/{}", destination.trim_start_matches('/'))
}

#[function_component(AdminLlmAccessHandoffPage)]
pub fn admin_llm_access_handoff_page(props: &AdminLlmAccessHandoffProps) -> Html {
    let target = llm_access_frontend_url(props.destination.as_str());

    html! {
        <AdminShell active={AdminNav::Overview}>
            <AdminPageHeader
                eyebrow={Some(AttrValue::Static("外部控制台 · LLM ACCESS"))}
                title={AttrValue::from(format!("{} 已迁移至独立控制台", props.workspace))}
                description={Some(AttrValue::Static(
                    "多 Provider 密钥、账号池、Fallback 路由图、实时监控、用量分析与审核策略已由独立 llm-access 前端服务统一接管。"
                ))}
            >
                <Link<Route> to={Route::Admin} classes={classes!("adm-btn", "adm-btn--secondary")}>
                    { "返回后台概览" }
                </Link<Route>>
                <a
                    class="adm-btn adm-btn--primary"
                    href={target.clone()}
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    <span>{ "打开 llm-access 控制台" }</span>
                    <span aria-hidden="true">{ "↗" }</span>
                </a>
            </AdminPageHeader>

            <section class="adm-card" style="max-width: 760px;">
                <div class="adm-card__header">
                    <div>
                        <h2 class="adm-card__title">{ "跳转入口说明" }</h2>
                        <p class="adm-card__sub">
                            { "StaticFlow 本地后台保留此路由仅用于书签与历史链接的平滑过渡，不再维护旧版内置表单。" }
                        </p>
                    </div>
                    <span class="adm-pill adm-pill--accent">{ "独立服务" }</span>
                </div>
                <div class="adm-card__body adm-stack-sm">
                    <dl class="adm-kv">
                        <div>
                            <dt>{ "工作区" }</dt>
                            <dd>{ props.workspace.clone() }</dd>
                        </div>
                        <div>
                            <dt>{ "目标路径" }</dt>
                            <dd class="adm-mono">{ props.destination.clone() }</dd>
                        </div>
                    </dl>
                    <div class="adm-field">
                        <span class="adm-label">{ "目标控制台地址" }</span>
                        <pre class="adm-code">{ target.clone() }</pre>
                    </div>
                </div>
                <div class="adm-card__footer">
                    <span>{ "请确认本地 tmux 会话 sf-llm-access-frontend（127.0.0.1:19191）处于运行状态。" }</span>
                    <a class="adm-link adm-mono" href={target}>
                        { "直接前往 ↗" }
                    </a>
                </div>
            </section>
        </AdminShell>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_access_frontend_url_joins_console_destination() {
        let url = llm_access_frontend_url("/console/usage/codex");
        assert!(url.ends_with("/console/usage/codex"));
        assert!(!url.contains("//console"));
    }
}
