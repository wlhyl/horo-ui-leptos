mod api;
mod app;
mod astro;
mod auth;
mod components;
mod config;
mod enums;
mod home;
mod models;
mod native;
mod render;
mod routes;
mod storage;
mod user;

use leptos::prelude::mount_to_body;

fn main() {
    console_error_panic_hook::set_once();

    if let Err(error) = console_log::init_with_level(log::Level::Info) {
        let message = format!("日志初始化失败: {error}");
        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&message));
    }

    mount_to_body(app::App);
}
