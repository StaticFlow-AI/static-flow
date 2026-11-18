use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{footer::Footer, header::Header},
    music_context::{MusicAction, MusicPlayerContext},
    pages,
};


#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[cfg(not(feature = "mock"))]
    #[at("/")]
    Home,
    #[cfg(feature = "mock")]
    #[at("/static_flow/")]
    Home,

    #[cfg(not(feature = "mock"))]
    #[at("/latest")]
    LatestArticles,
    #[cfg(feature = "mock")]
    #[at("/static_flow/latest")]
    LatestArticles,

    #[cfg(not(feature = "mock"))]
    #[at("/studio")]
    Studio,
    #[cfg(feature = "mock")]
    #[at("/static_flow/studio")]
    Studio,

    #[cfg(not(feature = "mock"))]
    #[at("/lab/worldlines")]
    Worldlines,
    #[cfg(feature = "mock")]
    #[at("/static_flow/lab/worldlines")]
    Worldlines,

    #[cfg(not(feature = "mock"))]
    #[at("/posts")]
    Posts,
    #[cfg(feature = "mock")]
    #[at("/static_flow/posts")]
    Posts,

    #[cfg(not(feature = "mock"))]
    #[at("/posts/:id")]
    ArticleDetail { id: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/posts/:id")]
    ArticleDetail { id: String },

    #[cfg(not(feature = "mock"))]
    #[at("/posts/:id/interactive")]
    ArticleInteractive { id: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/posts/:id/interactive")]
    ArticleInteractive { id: String },

    #[cfg(not(feature = "mock"))]
    #[at("/posts/:id/raw/:lang")]
    ArticleRaw { id: String, lang: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/posts/:id/raw/:lang")]
    ArticleRaw { id: String, lang: String },

    #[cfg(not(feature = "mock"))]
    #[at("/tags")]
    Tags,
    #[cfg(feature = "mock")]
    #[at("/static_flow/tags")]
    Tags,

    #[cfg(not(feature = "mock"))]
    #[at("/tags/:tag")]
    TagDetail { tag: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/tags/:tag")]
    TagDetail { tag: String },

    #[cfg(not(feature = "mock"))]
    #[at("/categories")]
    Categories,
    #[cfg(feature = "mock")]
    #[at("/static_flow/categories")]
    Categories,

    #[cfg(not(feature = "mock"))]
    #[at("/categories/:category")]
    CategoryDetail { category: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/categories/:category")]
    CategoryDetail { category: String },

    #[cfg(not(feature = "mock"))]
    #[at("/search")]
    Search,
    #[cfg(feature = "mock")]
    #[at("/static_flow/search")]
    Search,

    #[cfg(not(feature = "mock"))]
    #[at("/llm-access/help")]
    LlmAccessGuide,
    #[cfg(feature = "mock")]
    #[at("/static_flow/llm-access/help")]
    LlmAccessGuide,

    #[cfg(not(feature = "mock"))]
    #[at("/llm-access")]
    LlmAccess,
    #[cfg(feature = "mock")]
    #[at("/static_flow/llm-access")]
    LlmAccess,

    #[cfg(not(feature = "mock"))]
    #[at("/llm-access/usage")]
    LlmAccessUsage,
    #[cfg(feature = "mock")]
    #[at("/static_flow/llm-access/usage")]
    LlmAccessUsage,

    #[cfg(not(feature = "mock"))]
    #[at("/llm-access/quota-status")]
    LlmAccessQuotaStatus,
    #[cfg(feature = "mock")]
    #[at("/static_flow/llm-access/quota-status")]
    LlmAccessQuotaStatus,

    #[cfg(not(feature = "mock"))]
    #[at("/kiro-access")]
    KiroAccess,
    #[cfg(feature = "mock")]
    #[at("/static_flow/kiro-access")]
    KiroAccess,

    #[cfg(not(feature = "mock"))]
    #[at("/admin")]
    Admin,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin")]
    Admin,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway")]
    AdminLlmGateway,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway")]
    AdminLlmGateway,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/monitor")]
    AdminLlmGatewayMonitor,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/monitor")]
    AdminLlmGatewayMonitor,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/moderation")]
    AdminLlmGatewayModeration,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/moderation")]
    AdminLlmGatewayModeration,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/keys")]
    AdminLlmGatewayKeys,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/keys")]
    AdminLlmGatewayKeys,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/groups")]
    AdminLlmGatewayGroups,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/groups")]
    AdminLlmGatewayGroups,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/accounts")]
    AdminLlmGatewayAccounts,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/accounts")]
    AdminLlmGatewayAccounts,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/usage")]
    AdminLlmGatewayUsage,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/usage")]
    AdminLlmGatewayUsage,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/journal")]
    AdminLlmGatewayJournal,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/journal")]
    AdminLlmGatewayJournal,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/requests")]
    AdminLlmGatewayRequests,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/requests")]
    AdminLlmGatewayRequests,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/llm-gateway/settings")]
    AdminLlmGatewaySettings,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/llm-gateway/settings")]
    AdminLlmGatewaySettings,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/kiro-gateway")]
    AdminKiroGateway,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/kiro-gateway")]
    AdminKiroGateway,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/kiro-gateway/upstream-channels")]
    AdminKiroAnthropicUpstreams,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/kiro-gateway/upstream-channels")]
    AdminKiroAnthropicUpstreams,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/kiro-gateway/accounts")]
    AdminKiroAccountStatus,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/kiro-gateway/accounts")]
    AdminKiroAccountStatus,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/kiro-gateway/accounts/manage")]
    AdminKiroGatewayAccountsManage,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/kiro-gateway/accounts/manage")]
    AdminKiroGatewayAccountsManage,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/kiro-gateway/keys")]
    AdminKiroGatewayKeys,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/kiro-gateway/keys")]
    AdminKiroGatewayKeys,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/kiro-gateway/models")]
    AdminKiroGatewayModels,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/kiro-gateway/models")]
    AdminKiroGatewayModels,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/kiro-gateway/groups")]
    AdminKiroGatewayGroups,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/kiro-gateway/groups")]
    AdminKiroGatewayGroups,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/kiro-gateway/usage")]
    AdminKiroGatewayUsage,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/kiro-gateway/usage")]
    AdminKiroGatewayUsage,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/gpt2api-rs")]
    AdminGpt2ApiRs,
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/gpt2api-rs")]
    AdminGpt2ApiRs,

    #[cfg(not(feature = "mock"))]
    #[at("/admin/comments/runs/:task_id")]
    AdminCommentRuns { task_id: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/comments/runs/:task_id")]
    AdminCommentRuns { task_id: String },

    #[cfg(not(feature = "mock"))]
    #[at("/admin/music-wishes/runs/:wish_id")]
    AdminMusicWishRuns { wish_id: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/music-wishes/runs/:wish_id")]
    AdminMusicWishRuns { wish_id: String },

    #[cfg(not(feature = "mock"))]
    #[at("/admin/article-requests/runs/:request_id")]
    AdminArticleRequestRuns { request_id: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/admin/article-requests/runs/:request_id")]
    AdminArticleRequestRuns { request_id: String },

    #[cfg(all(feature = "local-media", not(feature = "mock")))]
    #[at("/admin/local-media")]
    AdminLocalMedia,
    #[cfg(all(feature = "local-media", feature = "mock"))]
    #[at("/static_flow/admin/local-media")]
    AdminLocalMedia,

    #[cfg(all(feature = "local-media", not(feature = "mock")))]
    #[at("/admin/local-media/player")]
    AdminLocalMediaPlayer,
    #[cfg(all(feature = "local-media", feature = "mock"))]
    #[at("/static_flow/admin/local-media/player")]
    AdminLocalMediaPlayer,

    #[cfg(not(feature = "mock"))]
    #[at("/media/video")]
    MediaVideo,
    #[cfg(feature = "mock")]
    #[at("/static_flow/media/video")]
    MediaVideo,

    #[cfg(not(feature = "mock"))]
    #[at("/media/audio")]
    MediaAudio,
    #[cfg(feature = "mock")]
    #[at("/static_flow/media/audio")]
    MediaAudio,

    #[cfg(not(feature = "mock"))]
    #[at("/media/image")]
    MediaImage,
    #[cfg(feature = "mock")]
    #[at("/static_flow/media/image")]
    MediaImage,

    #[cfg(not(feature = "mock"))]
    #[at("/media/audio/:id")]
    MusicPlayer { id: String },
    #[cfg(feature = "mock")]
    #[at("/static_flow/media/audio/:id")]
    MusicPlayer { id: String },

    #[not_found]
    #[cfg(not(feature = "mock"))]
    #[at("/404")]
    NotFound,
    #[not_found]
    #[cfg(feature = "mock")]
    #[at("/static_flow/404")]
    NotFound,
}

/// Default `/admin/{llm,kiro}-gateway*` routes are handoff pages into
/// `deps/llm-access/apps/llm-access-frontend`. The legacy Yew admin modules are
/// compiled and mounted only when built with `--features legacy-llm-admin` (or
/// `STATICFLOW_ENABLE_LEGACY_LLM_ADMIN=1` via
/// `scripts/build_frontend_selfhosted.sh`).
macro_rules! migrated_llm_admin_route {
    ($destination:expr, $workspace:expr, $legacy_component:path) => {{
        #[cfg(feature = "legacy-llm-admin")]
        {
            if option_env!("STATICFLOW_ENABLE_LEGACY_LLM_ADMIN") != Some("0") {
                return html! { <$legacy_component /> };
            }
        }
        html! {
            <pages::admin_llm_access_handoff::AdminLlmAccessHandoffPage
                destination={$destination}
                workspace={$workspace}
            />
        }
    }};
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <pages::home::HomePage /> },
        Route::LatestArticles => html! { <pages::latest_articles::LatestArticlesPage /> },
        Route::Studio => html! { <pages::studio::StudioPage /> },
        Route::Worldlines => html! { <pages::worldlines::WorldlinesGame /> },
        Route::Posts => html! { <pages::PostsPage /> },
        Route::ArticleDetail {
            id,
        } => {
            html! { <pages::article_detail::ArticleDetailPage id={id} /> }
        },
        Route::ArticleInteractive {
            id,
        } => {
            html! { <pages::interactive_article::InteractiveArticlePage id={id} /> }
        },
        Route::ArticleRaw {
            id,
            lang,
        } => {
            html! { <pages::article_raw::ArticleRawPage id={id} lang={lang} /> }
        },
        Route::Tags => html! { <pages::tags::TagsPage /> },
        Route::TagDetail {
            tag,
        } => {
            html! { <pages::tag_detail::TagDetailPage tag={tag} /> }
        },
        Route::Categories => html! { <pages::categories::CategoriesPage /> },
        Route::CategoryDetail {
            category,
        } => {
            html! { <pages::category_detail::CategoryDetailPage category={category} /> }
        },
        Route::Search => html! { <pages::search::SearchPage /> },
        Route::LlmAccessGuide => html! { <pages::llm_access_guide::LlmAccessGuidePage /> },
        Route::LlmAccess => html! { <pages::llm_access::LlmAccessPage /> },
        Route::LlmAccessUsage => html! { <pages::llm_access_usage::LlmAccessUsagePage /> },
        Route::LlmAccessQuotaStatus => {
            html! { <pages::llm_access_quota_status::LlmAccessQuotaStatusPage /> }
        },
        Route::KiroAccess => html! { <pages::kiro_access::KiroAccessPage /> },
        Route::Admin => html! { <pages::admin::AdminPage /> },
        Route::AdminLlmGateway => migrated_llm_admin_route!(
            "/console",
            "LLM Gateway",
            pages::admin_llm_gateway::AdminLlmGatewayPage
        ),
        Route::AdminLlmGatewayKeys => migrated_llm_admin_route!(
            "/console/codex/keys",
            "Codex Keys",
            pages::admin_llm_gateway_keys::AdminLlmGatewayKeysPage
        ),
        Route::AdminLlmGatewayGroups => migrated_llm_admin_route!(
            "/console/codex/groups",
            "Codex 账号组",
            pages::admin_llm_gateway_groups::AdminLlmGatewayGroupsPage
        ),
        Route::AdminLlmGatewayAccounts => migrated_llm_admin_route!(
            "/console/codex/accounts",
            "Codex 账号池",
            pages::admin_llm_gateway_accounts::AdminLlmGatewayAccountsPage
        ),
        Route::AdminLlmGatewayUsage => migrated_llm_admin_route!(
            "/console/usage/codex",
            "Codex 用量",
            pages::admin_llm_gateway_usage::AdminLlmGatewayUsagePage
        ),
        Route::AdminLlmGatewayJournal => migrated_llm_admin_route!(
            "/console/journal",
            "Usage Journal",
            pages::admin_llm_gateway_journal::AdminLlmGatewayJournalPage
        ),
        Route::AdminLlmGatewayRequests => migrated_llm_admin_route!(
            "/console/codex/requests",
            "LLM 申请审核",
            pages::admin_llm_gateway_requests::AdminLlmGatewayRequestsPage
        ),
        Route::AdminLlmGatewaySettings => migrated_llm_admin_route!(
            "/console/system/runtime",
            "llm-access 运行时配置",
            pages::admin_llm_gateway_settings::AdminLlmGatewaySettingsPage
        ),
        Route::AdminLlmGatewayMonitor => migrated_llm_admin_route!(
            "/console/observability",
            "LLM 运行监控",
            pages::admin_llm_gateway_monitor::AdminLlmGatewayMonitorPage
        ),
        Route::AdminLlmGatewayModeration => migrated_llm_admin_route!(
            "/console/policy/moderation",
            "LLM 审核策略",
            pages::admin_moderation::AdminModerationPage
        ),
        Route::AdminKiroGateway => migrated_llm_admin_route!(
            "/console/kiro/runtime",
            "Kiro Gateway",
            pages::admin_kiro_gateway::AdminKiroGatewayPage
        ),
        Route::AdminKiroGatewayAccountsManage => migrated_llm_admin_route!(
            "/console/kiro/accounts",
            "Kiro 账号池",
            pages::admin_kiro_gateway_accounts::AdminKiroGatewayAccountsPage
        ),
        Route::AdminKiroGatewayKeys => migrated_llm_admin_route!(
            "/console/kiro/keys",
            "Kiro Keys",
            pages::admin_kiro_gateway_keys::AdminKiroGatewayKeysPage
        ),
        Route::AdminKiroGatewayModels => migrated_llm_admin_route!(
            "/console/kiro/models",
            "Kiro 模型目录",
            pages::admin_kiro_gateway_models::AdminKiroGatewayModelsPage
        ),
        Route::AdminKiroGatewayGroups => migrated_llm_admin_route!(
            "/console/kiro/groups",
            "Kiro 账号组",
            pages::admin_kiro_gateway_groups::AdminKiroGatewayGroupsPage
        ),
        Route::AdminKiroGatewayUsage => migrated_llm_admin_route!(
            "/console/usage/kiro",
            "Kiro 用量",
            pages::admin_kiro_gateway_usage::AdminKiroGatewayUsagePage
        ),
        Route::AdminKiroAnthropicUpstreams => migrated_llm_admin_route!(
            "/console/kiro/upstreams",
            "Anthropic 上游通道",
            pages::admin_kiro_anthropic_upstreams::AdminKiroAnthropicUpstreamsPage
        ),
        Route::AdminKiroAccountStatus => migrated_llm_admin_route!(
            "/console/kiro/health",
            "Kiro 账号健康",
            pages::admin_kiro_account_status::AdminKiroAccountStatusPage
        ),
        Route::AdminGpt2ApiRs => html! { <pages::admin_gpt2api_rs::AdminGpt2ApiRsPage /> },
        Route::AdminCommentRuns {
            task_id,
        } => {
            html! { <pages::admin_ai_stream::AdminCommentRunsPage task_id={task_id} /> }
        },
        Route::AdminMusicWishRuns {
            wish_id,
        } => {
            html! { <pages::admin_music_wish_stream::AdminMusicWishRunsPage wish_id={wish_id} /> }
        },
        Route::AdminArticleRequestRuns {
            request_id,
        } => {
            html! { <pages::admin_article_request_stream::AdminArticleRequestRunsPage request_id={request_id} /> }
        },
        #[cfg(feature = "local-media")]
        Route::AdminLocalMedia => html! { <pages::admin_local_media::AdminLocalMediaPage /> },
        #[cfg(feature = "local-media")]
        Route::AdminLocalMediaPlayer => {
            html! { <pages::admin_local_media_player::AdminLocalMediaPlayerPage /> }
        },
        Route::MediaVideo => html! { <pages::coming_soon::ComingSoonPage feature={"video"} /> },
        Route::MediaAudio => html! { <pages::music_library::MusicLibraryPage /> },
        Route::MediaImage => html! { <pages::image_library::ImageLibraryPage /> },
        Route::MusicPlayer {
            id,
        } => html! { <pages::music_player::MusicPlayerPage id={id} /> },
        Route::NotFound => html! { <pages::not_found::NotFoundPage /> },
    }
}

#[function_component(AppRouter)]
pub fn app_router() -> Html {
    html! {
        <BrowserRouter>
            <AppRouterInner />
        </BrowserRouter>
    }
}

#[function_component(AppRouterInner)]
fn app_router_inner() -> Html {
    let route = use_route::<Route>();

    // Auto-minimize player when navigating away from MusicPlayer page
    {
        let route = route.clone();
        let player_ctx = use_context::<MusicPlayerContext>();
        use_effect_with(route.clone(), move |route| {
            if let Some(ref ctx) = player_ctx {
                let is_player = matches!(route, Some(Route::MusicPlayer { .. }));
                if ctx.visible && !ctx.minimized && !is_player {
                    ctx.dispatch(MusicAction::Minimize);
                }
            }
            || ()
        });
    }

    {
        let route = route.clone();
        use_effect_with(route.clone(), move |active_route| {
            crate::seo::apply_route_seo(active_route.as_ref());
            || ()
        });
    }

    let immersive = matches!(route, Some(Route::Home | Route::Worldlines));
    html! {
        <div class="flex flex-col bg-[var(--bg)]" style="min-height: 100vh; min-height: 100svh;">
            if !immersive { <Header /> }
            <div class={classes!("flex-1", "min-w-0", (!immersive).then_some("pt-[var(--space-sm)]"))}>
                <Switch<Route> render={switch} />
            </div>
            if !immersive { <Footer /> }
            <crate::components::persistent_audio::PersistentAudio />
            <crate::components::mini_player::MiniPlayer />
        </div>
    }
}
