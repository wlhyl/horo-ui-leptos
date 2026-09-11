//! 应用外壳：响应式布局、顶部应用栏与路由。
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};

use crate::home::Home;
use crate::native;
use crate::routes::AppRoute;
use crate::storage::HoroStorage;

// 作用域样式：src/app.module.css -> 类名形如 `app-shell-a1b2c3d`
stylance::import_crate_style!(style, "src/app.module.css");
// 404 兜底文案复用共享的反馈样式
stylance::import_crate_style!(#[allow(dead_code)] feedback, "src/shared/feedback.module.css");

#[component]
pub fn App() -> impl IntoView {
    // 本地存储服务：启动时从 localStorage 恢复（对应原版 HoroStorageService）
    let storage = HoroStorage::init();
    provide_context(storage);

    view! {
        <Router>
            <div class=style::app_shell>
                <header class=style::app_bar>
                    <div class=style::logo></div>
                    <h1>"星盘 · Horo"</h1>
                    <div class=style::spacer></div>
                </header>

                <main class=style::app_main>
                    <Routes fallback=move || view! { <div class=feedback::error>"页面不存在"</div> }>
                        <Route path=AppRoute::Home view=move || view! { <Home/> }/>
                        <Route path=AppRoute::Native view=move || view! { <native::Input mode=native::ChartMode::Native/> }/>
                        <Route path=AppRoute::Event view=move || view! { <native::Input mode=native::ChartMode::Event/> }/>
                        <Route path=AppRoute::NativeChart view=move || view! { <native::Chart mode=native::ChartMode::Native/> }/>
                        <Route path=AppRoute::EventChart view=move || view! { <native::Chart mode=native::ChartMode::Event/> }/>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
