//! 应用外壳：响应式布局、顶部应用栏与路由。
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::auth::AuthService;
use crate::clean;
use crate::home::Home;
use crate::native;
use crate::routes::AppRoute;
use crate::storage::HoroStorage;
use crate::user;

// 作用域样式：src/app.module.css -> 类名形如 `app-shell-a1b2c3d`
stylance::import_crate_style!(style, "src/app.module.css");
// 404 兜底文案复用共享的反馈样式
stylance::import_crate_style!(#[allow(dead_code)] feedback, "src/shared/feedback.module.css");

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

                <main class=style::app_main>
                    <Routes fallback=move || view! { <div class=feedback::error>"页面不存在"</div> }>
                        <Route path=AppRoute::Home view=move || view! { <Home/> }/>
                        <Route path=AppRoute::Native view=move || view! { <native::Input mode=native::ChartMode::Native/> }/>
                        <Route path=AppRoute::Event view=move || view! { <native::Input mode=native::ChartMode::Event/> }/>
                        <Route path=AppRoute::NativeChart view=move || view! { <native::Chart mode=native::ChartMode::Native/> }/>
                        <Route path=AppRoute::EventChart view=move || view! { <native::Chart mode=native::ChartMode::Event/> }/>
                        <Route path=AppRoute::User view=move || view! { <user::User/> }/>
                        <Route path=AppRoute::Clean view=move || view! { <clean::Clean/> }/>
                    </Routes>
                </main>
            </div>
        </Router>
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
