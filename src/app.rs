//! 应用外壳：响应式布局、顶部应用栏与路由。
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::{use_location, use_navigate};

use crate::auth::AuthService;
use crate::clean;
use crate::compare;
use crate::direction;
use crate::enums::process_name::ProcessName;
use crate::home::Home;
use crate::native;
use crate::power;
use crate::return_chart;
use crate::routes::AppRoute;
use crate::storage::HoroStorage;
use crate::user;
use crate::workbench;

// 作用域样式：src/app.module.css -> 类名形如 `app-shell-a1b2c3d`
stylance::import_crate_style!(style, "src/app.module.css");
// 404 兜底文案复用共享的反馈样式
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

#[component]
pub fn App() -> impl IntoView {
    // 本地存储服务：启动时从 localStorage 恢复（对应原版 HoroStorageService）
    let storage = HoroStorage::init();
    provide_context(storage);
    // 认证服务：启动时从 localStorage 的 token 恢复登录态（对应原版 AuthService）
    let auth = AuthService::init();
    provide_context(auth);

    view! {
        <Router>
            <div class=style::app_shell>
                <header class=style::app_bar>
                    <div class=style::logo></div>
                    <TitleHome/>
                    <div class=style::spacer></div>
                    <UserEntry/>
                </header>

                <MainShell>
                    <Routes fallback=move || view! { <div class=feedback::error>"页面不存在"</div> }>
                        <Route path=AppRoute::Home view=move || view! { <Home/> }/>
                        <Route path=AppRoute::Native view=move || view! { <native::Input mode=native::ChartMode::Native/> }/>
                        <Route path=AppRoute::Event view=move || view! { <native::Input mode=native::ChartMode::Event/> }/>
                        <Route path=AppRoute::Derived view=move || view! { <native::Input mode=native::ChartMode::Derived/> }/>
                        <Route path=AppRoute::NativeChart view=move || view! { <native::Chart mode=native::ChartMode::Native/> }/>
                        <Route path=AppRoute::EventChart view=move || view! { <native::Chart mode=native::ChartMode::Event/> }/>
                        <Route path=AppRoute::DerivedChart view=move || view! { <native::Chart mode=native::ChartMode::Derived/> }/>
                        <Route path=AppRoute::Process view=move || view! { <direction::ProcessInput/> }/>
                        <Route path=AppRoute::Direction view=move || view! { <direction::DirectionPage mode=ProcessName::Direction/> }/>
                        <Route path=AppRoute::DailyDirection view=move || view! { <direction::DirectionPage mode=ProcessName::DailyDirection/> }/>
                        <Route path=AppRoute::SolarArc view=move || view! { <direction::DirectionPage mode=ProcessName::SolarArc/> }/>
                        <Route path=AppRoute::ReturnSolar view=move || view! { <return_chart::ReturnPage mode=ProcessName::SolarReturn/> }/>
                        <Route path=AppRoute::ReturnLunar view=move || view! { <return_chart::ReturnPage mode=ProcessName::LunarReturn/> }/>
                        <Route path=AppRoute::ReturnDaily view=move || view! { <return_chart::ReturnPage mode=ProcessName::DailyReturn/> }/>
                        <Route path=AppRoute::CompareTransit view=move || view! { <compare::ComparePage mode=ProcessName::Transit/> }/>
                        <Route path=AppRoute::CompareSolarNative view=move || view! { <compare::ComparePage mode=ProcessName::SolarcomparNative/> }/>
                        <Route path=AppRoute::CompareNativeSolar view=move || view! { <compare::ComparePage mode=ProcessName::NativecomparSolar/> }/>
                        <Route path=AppRoute::CompareLunarNative view=move || view! { <compare::ComparePage mode=ProcessName::LunarcomparNative/> }/>
                        <Route path=AppRoute::CompareNativeLunar view=move || view! { <compare::ComparePage mode=ProcessName::NativecomparLunar/> }/>
                        <Route path=AppRoute::CompareDailyNative view=move || view! { <compare::ComparePage mode=ProcessName::DailycomparNative/> }/>
                        <Route path=AppRoute::CompareNativeDaily view=move || view! { <compare::ComparePage mode=ProcessName::NativecomparDaily/> }/>
                        <Route path=AppRoute::CompareSecondaryProgression view=move || view! { <compare::ComparePage mode=ProcessName::SecondaryProgressionComparNative/> }/>
                        <Route path=AppRoute::User view=move || view! { <user::User/> }/>
                        <Route path=AppRoute::Clean view=move || view! { <clean::Clean/> }/>
                        <Route path=AppRoute::Power view=move || view! { <power::Power/> }/>
                        <Route path=AppRoute::Workbench view=move || view! { <workbench::Workbench/> }/>
                    </Routes>
                </MainShell>
            </div>
        </Router>
    }
}

/// 页面主容器：工作台等全幅页面切换为不限宽布局，其余页面保持限宽居中。
/// 必须是 Router 内的子组件：`use_location` 需在渲染期的 Router 上下文中调用。
#[component]
fn MainShell(children: Children) -> impl IntoView {
    let location = use_location();
    let is_wide = Memo::new(move |_| location.pathname.get() == AppRoute::Workbench.path());

    view! {
        <main class=move || if is_wide.get() { style::app_main_wide } else { style::app_main }>
            {children()}
        </main>
    }
}

/// 顶栏标题「星盘 · Horo」：点击回到首页。
/// 必须是 Router 内的子组件：`use_navigate` 需在渲染期的 Router 上下文中调用。
#[component]
fn TitleHome() -> impl IntoView {
    let nav = use_navigate();
    let go_home = move |_| {
        nav(AppRoute::Home.path(), NavigateOptions::default());
    };

    view! {
        <h1 class=style::title_home on:click=go_home>"星盘 · Horo"</h1>
    }
}

/// 顶栏右侧的用户入口：未登录显示「登录」，已登录显示用户名，点击进入用户页。
/// 必须是 Router 内的子组件：`use_navigate` 需在渲染期的 Router 上下文中调用。
#[component]
fn UserEntry() -> impl IntoView {
    let nav = use_navigate();
    let auth = use_context::<AuthService>().expect("AuthService 未初始化");

    let go_user = move |_| {
        nav(AppRoute::User.path(), NavigateOptions::default());
    };

    view! {
        <button class=style::user_entry on:click=go_user>
            // 用户图标（stroke 风格同首页图标）
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
            </svg>
            {move || match auth.user() {
                Some(user) => user.name,
                None => "登录".to_string(),
            }}
        </button>
    }
}
