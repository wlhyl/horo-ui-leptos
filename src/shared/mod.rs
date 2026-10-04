//! 跨页面共用的小工具（除 *.module.css 样式外的 Rust 部分）。

/// 基于 Promise 的毫秒级休眠（wasm 无标准线程，借浏览器 setTimeout 实现）。
/// 直接把 Promise 的 resolve 作为定时回调，无需构造额外闭包。
pub(crate) async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve: js_sys::Function, _| {
        let _ = web_sys::window()
            .map(|w| w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms));
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}
