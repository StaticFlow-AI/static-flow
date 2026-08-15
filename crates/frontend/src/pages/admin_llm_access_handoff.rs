use yew::prelude::*;

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
        <main class={classes!("min-h-screen", "bg-[var(--bg)]", "px-4", "py-10", "lg:px-8")}>
            <section class={classes!(
                "mx-auto",
                "max-w-2xl",
                "rounded-[var(--radius)]",
                "border",
                "border-[var(--border)]",
                "bg-[var(--surface)]",
                "p-6",
                "shadow-[var(--shadow)]",
                "lg:p-8"
            )}>
                <div class={classes!("text-xs", "font-semibold", "uppercase", "tracking-[0.16em]", "text-[var(--muted)]")}>
                    { "LLM ACCESS FRONTEND" }
                </div>
                <h1 class={classes!("mb-0", "mt-3", "text-2xl", "font-semibold")}>
                    { format!("{} 已迁移", props.workspace) }
                </h1>
                <p class={classes!("mt-3", "text-sm", "leading-7", "text-[var(--muted)]")}>
                    { "llm-access 的管理配置、运行监控、用量明细、审核策略和 AI Reviewer 已由独立前端服务统一接管。StaticFlow Admin 保留这个入口用于平滑过渡，不再维护这里的旧管理页面。" }
                </p>
                <div class={classes!("mt-6", "flex", "flex-wrap", "items-center", "gap-3")}>
                    <a class={classes!("btn-fluent-primary")} href={target.clone()}>
                        <i class={classes!("fas", "fa-arrow-up-right-from-square", "mr-2")} aria-hidden="true"></i>
                        { "打开 llm-access frontend" }
                    </a>
                    <a class={classes!("btn-fluent-secondary")} href="/admin">
                        { "返回 StaticFlow Admin" }
                    </a>
                </div>
                <div class={classes!("mt-5", "rounded-lg", "border", "border-[var(--border)]", "bg-[var(--surface-alt)]", "px-3", "py-2", "font-mono", "text-xs", "text-[var(--muted)]", "break-all")}>
                    { target }
                </div>
            </section>
        </main>
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
