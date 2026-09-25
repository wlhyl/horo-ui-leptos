//! 用户页：登录 / 注销（对应原版 horo-ui 的 UserPage）。
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::client::post_login;
use crate::api::request::LoginRequest;
use crate::auth::AuthService;

// 作用域样式：src/user/user.module.css
stylance::import_crate_style!(style, "src/user/user.module.css");
// 显式共享的样式模块（内部并非每个类都被本文件用到，故关闭 dead_code 警告）
stylance::import_crate_style!(card, "src/shared/card.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

#[component]
pub fn User() -> impl IntoView {
    let auth = use_context::<AuthService>().expect("AuthService 未初始化");

    let name = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    // 登录失败提示（页面内文本，非弹窗）
    let error = RwSignal::new(String::new());
    // 请求进行中：禁用按钮防重复提交
    let submitting = RwSignal::new(false);

    let login = move |_| {
        let req = LoginRequest {
            name: name.get().trim().to_owned(),
            password: password.get(),
        };
        if req.name.is_empty() || req.password.is_empty() {
            error.set("请输入用户名和密码".into());
            return;
        }
        submitting.set(true);
        spawn_local(async move {
            match post_login(&req).await {
                Ok(token) => {
                    auth.login(&token);
                    password.set(String::new());
                    error.set(String::new());
                }
                // 失败保留密码，便于重试
                Err(msg) => error.set(msg),
            }
            submitting.set(false);
        });
    };

    let logout = move |_| auth.logout();

    view! {
        // 未登录：登录表单（互斥渲染用 Option + bool::then，遵循项目惯例）
        {move || {
            auth.user()
                .is_none()
                .then(move || {
                    view! {
                        <div class=card::card>
                            <h2>"用户登录"</h2>
                            <div class=style::form>
                                <div class=form::field>
                                    <label>"用户名"</label>
                                    <div class=form::control>
                                        <input
                                            type="text"
                                            placeholder="必填"
                                            value=move || name.get()
                                            on:input=move |ev| name.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                                <div class=form::field>
                                    <label>"密码"</label>
                                    <div class=form::control>
                                        <input
                                            type="password"
                                            placeholder="必填"
                                            value=move || password.get()
                                            on:input=move |ev| password.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                                {move || {
                                    (!error.get().is_empty())
                                        .then(|| view! { <div class=feedback::error>{error.get()}</div> })
                                }}
                                <button
                                    class=form::btn_primary
                                    disabled=move || submitting.get()
                                    on:click=login
                                >
                                    "登录"
                                </button>
                            </div>
                        </div>
                    }
                })
        }}

        // 已登录：用户信息与注销
        {move || {
            auth.user().map(move |user| {
                view! {
                    <div class=card::card>
                        <h2>"用户"</h2>
                        <div class=style::logged_in>
                            <span class=style::logged_in_label>"已登录："</span>
                            <span class=style::logged_in_name>{user.name}</span>
                        </div>
                        <button class=form::btn_primary on:click=logout>"注销"</button>
                    </div>
                }
            })
        }}
    }
}
