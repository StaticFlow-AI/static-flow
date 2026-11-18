//! Shared chrome for every local admin surface (`/admin*`).
//!
//! `AdminShell` owns the operator workbench frame: a sticky grouped sidebar
//! (collapsible to an icon rail on desktop, an opaque full-height drawer on
//! narrow viewports), a breadcrumb top bar with site health / theme / exit
//! controls, and a width-capped main column. It also injects the admin-only
//! stylesheet (`admin.css` + `tools.css`) into `<head>` once and flips
//! `html.sf-admin-mode` while mounted so the public header/footer step aside
//! without touching the router.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::{
    prelude::{use_route, Link},
    Routable,
};

use crate::{
    api::API_BASE, components::theme_toggle::ThemeToggle,
    pages::admin_llm_access_handoff::llm_access_frontend_url, router::Route,
};

const ADMIN_STYLE_ID: &str = "sf-admin-style";
const ADMIN_MODE_CLASS: &str = "sf-admin-mode";
const SIDEBAR_COLLAPSED_KEY: &str = "sf-admin-sidebar-collapsed";
const ADMIN_CSS: &str = concat!(include_str!("admin.css"), "\n", include_str!("tools.css"));

/// Every navigable admin destination. The first nine live inside `/admin`
/// and are addressed by URL hash; the rest are standalone routes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdminNav {
    Overview,
    CommentTasks,
    PublishedComments,
    AuditLogs,
    MusicWishes,
    ArticleRequests,
    ApiBehavior,
    RuntimeMemory,
    SiteConfig,
    LocalMedia,
}

impl AdminNav {
    /// All `/admin` hash sections, in sidebar order.
    pub const HASH_SECTIONS: [AdminNav; 9] = [
        AdminNav::Overview,
        AdminNav::CommentTasks,
        AdminNav::PublishedComments,
        AdminNav::AuditLogs,
        AdminNav::MusicWishes,
        AdminNav::ArticleRequests,
        AdminNav::ApiBehavior,
        AdminNav::RuntimeMemory,
        AdminNav::SiteConfig,
    ];

    /// URL hash (without `#`) for sections hosted by `/admin`.
    pub fn hash(self) -> Option<&'static str> {
        match self {
            AdminNav::Overview => Some("overview"),
            AdminNav::CommentTasks => Some("tasks"),
            AdminNav::PublishedComments => Some("published"),
            AdminNav::AuditLogs => Some("audit"),
            AdminNav::MusicWishes => Some("wishes"),
            AdminNav::ArticleRequests => Some("articles"),
            AdminNav::ApiBehavior => Some("behavior"),
            AdminNav::RuntimeMemory => Some("memory"),
            AdminNav::SiteConfig => Some("config"),
            AdminNav::LocalMedia => None,
        }
    }

    /// Parses a location hash (with or without the leading `#`).
    pub fn from_hash(hash: &str) -> Option<AdminNav> {
        let raw = hash.trim_start_matches('#');
        Self::HASH_SECTIONS
            .into_iter()
            .find(|nav| nav.hash() == Some(raw))
    }

    pub fn label(self) -> &'static str {
        match self {
            AdminNav::Overview => "概览",
            AdminNav::CommentTasks => "评论任务",
            AdminNav::PublishedComments => "已发布评论",
            AdminNav::AuditLogs => "审计日志",
            AdminNav::MusicWishes => "点歌心愿",
            AdminNav::ArticleRequests => "文章入库",
            AdminNav::ApiBehavior => "API 行为",
            AdminNav::RuntimeMemory => "运行时内存",
            AdminNav::SiteConfig => "全局配置",
            AdminNav::LocalMedia => "本地媒体",
        }
    }

    pub fn group(self) -> &'static str {
        match self {
            AdminNav::Overview => "工作台",
            AdminNav::CommentTasks | AdminNav::PublishedComments | AdminNav::AuditLogs => {
                "内容审核"
            },
            AdminNav::MusicWishes | AdminNav::ArticleRequests => "共建队列",
            AdminNav::ApiBehavior | AdminNav::RuntimeMemory => "运行洞察",
            AdminNav::SiteConfig => "站点配置",
            AdminNav::LocalMedia => "工具",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            AdminNav::Overview => "fa-house",
            AdminNav::CommentTasks => "fa-inbox",
            AdminNav::PublishedComments => "fa-comments",
            AdminNav::AuditLogs => "fa-clock-rotate-left",
            AdminNav::MusicWishes => "fa-music",
            AdminNav::ArticleRequests => "fa-file-import",
            AdminNav::ApiBehavior => "fa-chart-line",
            AdminNav::RuntimeMemory => "fa-memory",
            AdminNav::SiteConfig => "fa-sliders",
            AdminNav::LocalMedia => "fa-film",
        }
    }

    /// SPA href for this destination.
    pub fn href(self) -> String {
        match self.hash() {
            Some(hash) => format!("{}#{hash}", Route::Admin.to_path()),
            None => match self {
                #[cfg(feature = "local-media")]
                AdminNav::LocalMedia => Route::AdminLocalMedia.to_path(),
                _ => Route::Admin.to_path(),
            },
        }
    }
}

/// Plain `<a href>` target for an admin destination (`/admin#tasks`,
/// `/admin/local-media`, ...).
pub fn admin_nav_href(nav: AdminNav) -> String {
    nav.href()
}

/// Sidebar groups, in order. `LocalMedia` is only listed when the
/// `local-media` feature (and therefore its route) is compiled in.
fn nav_groups() -> Vec<(&'static str, Vec<AdminNav>)> {
    let mut groups = vec![
        ("工作台", vec![AdminNav::Overview]),
        ("内容审核", vec![
            AdminNav::CommentTasks,
            AdminNav::PublishedComments,
            AdminNav::AuditLogs,
        ]),
        ("共建队列", vec![AdminNav::MusicWishes, AdminNav::ArticleRequests]),
        ("运行洞察", vec![AdminNav::ApiBehavior, AdminNav::RuntimeMemory]),
        ("站点配置", vec![AdminNav::SiteConfig]),
    ];
    if cfg!(feature = "local-media") {
        groups.push(("工具", vec![AdminNav::LocalMedia]));
    }
    groups
}

/// Inserts the admin stylesheet once. Idempotent across remounts; every rule
/// in it is scoped to `.adm-shell` / `html.sf-admin-mode`, so leaving it in
/// place after navigating back to public pages is harmless.
fn ensure_admin_style() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    if let Some(existing) = document.get_element_by_id(ADMIN_STYLE_ID) {
        if existing.text_content().as_deref() != Some(ADMIN_CSS) {
            existing.set_text_content(Some(ADMIN_CSS));
        }
        return;
    }
    let Ok(style) = document.create_element("style") else {
        return;
    };
    style.set_id(ADMIN_STYLE_ID);
    style.set_text_content(Some(ADMIN_CSS));
    if let Some(head) = document.head() {
        let _ = head.append_child(&style);
    }
}

fn set_html_class(class: &str, on: bool) {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let list = root.class_list();
        let _ = if on { list.add_1(class) } else { list.remove_1(class) };
    }
}

fn set_body_scroll_locked(locked: bool) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    if let Some(root_el) = doc
        .document_element()
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let style = root_el.style();
        let _ = if locked {
            style.set_property("overflow", "hidden")
        } else {
            style.remove_property("overflow").map(|_| ())
        };
    }
    if let Some(body) = doc.body() {
        let style = body.style();
        let _ = if locked {
            style.set_property("overflow", "hidden")
        } else {
            style.remove_property("overflow").map(|_| ())
        };
    }
}

fn load_collapsed() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(SIDEBAR_COLLAPSED_KEY).ok().flatten())
        .is_some_and(|v| v == "1")
}

fn store_collapsed(collapsed: bool) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(SIDEBAR_COLLAPSED_KEY, if collapsed { "1" } else { "0" });
    }
}

fn is_on_admin_root(route: Option<&Route>) -> bool {
    if let Some(current) = route {
        return matches!(current, Route::Admin);
    }
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .is_some_and(|path| {
            let trimmed = path.trim_end_matches('/');
            trimmed == Route::Admin.to_path().trim_end_matches('/')
        })
}

/// Navigates to an `/admin` hash section. On `/admin` itself this only
/// rewrites the hash (the page listens to `hashchange`); from another admin
/// route it performs an SPA push and the page reads the hash on mount.
pub fn navigate_to_admin_section(nav: AdminNav, already_on_admin: bool) {
    let Some(hash) = nav.hash() else {
        return;
    };
    if already_on_admin {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_hash(hash);
        }
    } else {
        let target = nav.href();
        if !crate::navigation_context::navigate_spa_to(&target) {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&target);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Health {
    Checking,
    Ok,
    Down,
}

#[derive(Properties, PartialEq)]
pub struct AdminShellProps {
    pub active: AdminNav,
    #[prop_or_default]
    pub children: Html,
    /// Optional per-destination counters (e.g. pending items) shown as small
    /// mono badges next to sidebar entries.
    #[prop_or_default]
    pub badges: Vec<(AdminNav, usize)>,
}

#[function_component(AdminShell)]
pub fn admin_shell(props: &AdminShellProps) -> Html {
    let collapsed = use_state(load_collapsed);
    let drawer_open = use_state(|| false);
    let health = use_state(|| Health::Checking);
    let health_tick = use_state(|| 0_u32);

    use_effect_with((), |_| {
        ensure_admin_style();
        set_html_class(ADMIN_MODE_CLASS, true);
        move || {
            set_html_class(ADMIN_MODE_CLASS, false);
            set_body_scroll_locked(false);
        }
    });

    {
        let health = health.clone();
        use_effect_with(*health_tick, move |_| {
            health.set(Health::Checking);
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("{}/healthz", API_BASE.trim_end_matches('/'));
                let ok = matches!(
                    gloo_net::http::Request::get(&url).send().await,
                    Ok(resp) if resp.ok()
                );
                health.set(if ok { Health::Ok } else { Health::Down });
            });
            || ()
        });
    }

    {
        let open = *drawer_open;
        let drawer_open = drawer_open.clone();
        use_effect_with(open, move |open| {
            set_body_scroll_locked(*open);
            let listener = open.then(|| {
                let drawer_open = drawer_open.clone();
                web_sys::window().map(|window| {
                    EventListener::new(&window, "keydown", move |event| {
                        if let Some(key) = event.dyn_ref::<web_sys::KeyboardEvent>() {
                            if key.key() == "Escape" {
                                drawer_open.set(false);
                            }
                        }
                    })
                })
            });
            move || drop(listener)
        });
    }

    let current_route = use_route::<Route>();
    let active = props.active;
    let on_admin_root = is_on_admin_root(current_route.as_ref());

    let toggle_collapsed = {
        let collapsed = collapsed.clone();
        Callback::from(move |_: MouseEvent| {
            let next = !*collapsed;
            store_collapsed(next);
            collapsed.set(next);
        })
    };
    let open_drawer = {
        let drawer_open = drawer_open.clone();
        Callback::from(move |_: MouseEvent| drawer_open.set(true))
    };
    let close_drawer = {
        let drawer_open = drawer_open.clone();
        Callback::from(move |_: MouseEvent| drawer_open.set(false))
    };
    let recheck_health = {
        let health_tick = health_tick.clone();
        Callback::from(move |_: MouseEvent| health_tick.set(health_tick.wrapping_add(1)))
    };

    let badge_for = |nav: AdminNav| -> Option<usize> {
        props
            .badges
            .iter()
            .find(|(n, _)| *n == nav)
            .map(|(_, count)| *count)
            .filter(|count| *count > 0)
    };

    let render_item = |nav: AdminNav| -> Html {
        let is_active = nav == active;
        let badge = badge_for(nav).map(|count| {
            html! { <span class="adm-nav-badge">{ if count > 99 { "99+".to_string() } else { count.to_string() } }</span> }
        });
        let inner = html! {
            <>
                <i class={classes!("fas", nav.icon(), "adm-nav-icon")} aria-hidden="true"></i>
                <span class="adm-nav-text">{ nav.label() }</span>
                { badge.unwrap_or_default() }
            </>
        };
        let class = classes!("adm-nav-item", is_active.then_some("is-active"));
        let aria_current = is_active.then_some(AttrValue::Static("page"));
        if nav.hash().is_some() {
            let drawer_open = drawer_open.clone();
            let onclick = Callback::from(move |event: MouseEvent| {
                if event.ctrl_key() || event.meta_key() || event.shift_key() || event.button() != 0
                {
                    return;
                }
                event.prevent_default();
                drawer_open.set(false);
                navigate_to_admin_section(nav, on_admin_root);
            });
            html! {
                <a {class} href={nav.href()} {onclick} title={nav.label()} aria-current={aria_current}>
                    { inner }
                </a>
            }
        } else {
            let route = match nav {
                #[cfg(feature = "local-media")]
                AdminNav::LocalMedia => Route::AdminLocalMedia,
                _ => Route::Admin,
            };
            let drawer_open = drawer_open.clone();
            let onclick = Callback::from(move |_: MouseEvent| drawer_open.set(false));
            html! {
                <span {onclick} class="adm-nav-link-wrap">
                    <Link<Route> to={route} classes={class}>
                        { inner }
                    </Link<Route>>
                </span>
            }
        }
    };

    let groups = nav_groups();
    let llm_console = llm_access_frontend_url("/console");

    let health_view = {
        let (tone, label) = match *health {
            Health::Checking => ("is-checking", "检测中"),
            Health::Ok => ("is-ok", "站点正常"),
            Health::Down => ("is-down", "站点不可达"),
        };
        html! {
            <button type="button" class={classes!("adm-health", tone)} onclick={recheck_health}
                title="重新检测 /api/healthz">
                <span class="adm-health__dot" aria-hidden="true"></span>
                <span class="adm-health__label">{ label }</span>
            </button>
        }
    };

    html! {
        <div class={classes!(
            "adm-shell",
            (*collapsed).then_some("is-collapsed"),
            (*drawer_open).then_some("is-drawer-open"),
        )}>
            <div class="adm-scrim" onclick={close_drawer.clone()} aria-hidden="true"></div>
            <aside class="adm-sidebar" aria-label="后台导航">
                <div class="adm-brand">
                    <a class="adm-brand__link" href={Route::Admin.to_path()} title="StaticFlow Admin">
                        <span class="adm-brand__mark" aria-hidden="true"></span>
                        <span class="adm-brand__text">
                            <span class="adm-brand__name">{ "StaticFlow" }</span>
                            <span class="adm-brand__sub">{ "Admin · loopback" }</span>
                        </span>
                    </a>
                    <button type="button" class="adm-btn adm-btn--ghost adm-btn--icon adm-drawer-close"
                        onclick={close_drawer} aria-label="关闭导航">
                        <i class="fas fa-xmark" aria-hidden="true"></i>
                    </button>
                </div>
                <nav class="adm-nav">
                    { for groups.into_iter().map(|(group, items)| html! {
                        <div class="adm-nav-group">
                            <div class="adm-nav-label">{ group }</div>
                            { for items.into_iter().map(render_item) }
                        </div>
                    }) }
                    <div class="adm-nav-group">
                        <div class="adm-nav-label">{ "外部" }</div>
                        <a class="adm-nav-item" href={llm_console} target="_blank" rel="noopener noreferrer"
                            title="llm-access 控制台">
                            <i class="fas fa-arrow-up-right-from-square adm-nav-icon" aria-hidden="true"></i>
                            <span class="adm-nav-text">{ "llm-access 控制台" }</span>
                            <span class="adm-nav-ext" aria-hidden="true">{ "↗" }</span>
                        </a>
                    </div>
                </nav>
                <div class="adm-sidebar__foot">
                    <button type="button" class="adm-nav-item adm-collapse-btn" onclick={toggle_collapsed}
                        title={if *collapsed { "展开侧栏" } else { "收起侧栏" }}
                        aria-pressed={if *collapsed { "true" } else { "false" }}>
                        <i class={classes!("fas", if *collapsed { "fa-angles-right" } else { "fa-angles-left" }, "adm-nav-icon")}
                            aria-hidden="true"></i>
                        <span class="adm-nav-text">{ "收起侧栏" }</span>
                    </button>
                </div>
            </aside>
            <div class="adm-frame">
                <header class="adm-topbar">
                    <button type="button" class="adm-btn adm-btn--ghost adm-btn--icon adm-menu-btn"
                        onclick={open_drawer} aria-label="打开导航">
                        <i class="fas fa-bars" aria-hidden="true"></i>
                    </button>
                    <nav class="adm-crumbs" aria-label="面包屑">
                        <span class="adm-crumbs__group">{ active.group() }</span>
                        <span class="adm-crumbs__sep" aria-hidden="true">{ "/" }</span>
                        <span class="adm-crumbs__page">{ active.label() }</span>
                    </nav>
                    <div class="adm-topbar__right">
                        { health_view }
                        <ThemeToggle class={classes!("adm-theme-toggle")} />
                        <Link<Route> to={Route::Home} classes={classes!("adm-btn", "adm-btn--ghost", "adm-btn--sm", "adm-exit")}>
                            <span class="adm-exit__text">{ "返回站点" }</span>
                            <span aria-hidden="true">{ "↗" }</span>
                        </Link<Route>>
                    </div>
                </header>
                <main class="adm-main">
                    { props.children.clone() }
                </main>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AdminPageHeaderProps {
    pub title: AttrValue,
    #[prop_or_default]
    pub eyebrow: Option<AttrValue>,
    #[prop_or_default]
    pub description: Option<AttrValue>,
    /// Right-hand action area.
    #[prop_or_default]
    pub children: Html,
}

#[function_component(AdminPageHeader)]
pub fn admin_page_header(props: &AdminPageHeaderProps) -> Html {
    let has_actions = props.children != Html::default();
    html! {
        <header class="adm-page-header">
            <div class="adm-page-header__text">
                if let Some(eyebrow) = props.eyebrow.clone() {
                    <div class="adm-eyebrow">{ eyebrow }</div>
                }
                <h1 class="adm-title">{ props.title.clone() }</h1>
                if let Some(description) = props.description.clone() {
                    <p class="adm-desc">{ description }</p>
                }
            </div>
            if has_actions {
                <div class="adm-actions">{ props.children.clone() }</div>
            }
        </header>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_round_trips_for_every_admin_section() {
        for nav in AdminNav::HASH_SECTIONS {
            let hash = nav.hash().expect("section hash");
            assert_eq!(AdminNav::from_hash(hash), Some(nav));
            assert_eq!(AdminNav::from_hash(&format!("#{hash}")), Some(nav));
        }
        assert_eq!(AdminNav::from_hash("#nope"), None);
        assert_eq!(AdminNav::LocalMedia.hash(), None);
    }

    #[test]
    fn nav_groups_cover_each_destination_once() {
        let mut seen = Vec::new();
        for (group, items) in nav_groups() {
            for nav in items {
                assert_eq!(nav.group(), group);
                assert!(!seen.contains(&nav));
                seen.push(nav);
            }
        }
        for nav in AdminNav::HASH_SECTIONS {
            assert!(seen.contains(&nav));
        }
    }
}
