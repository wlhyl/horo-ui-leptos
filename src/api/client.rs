//! 后台 HTTP 调用封装（gloo-net，wasm fetch）。

use crate::api::request::HoroNativeRequest;
use crate::api::response::Horoscope;
use crate::config::API_BASE_URL;
use gloo_net::http::Request;

/// 调用 POST /api/horo/native 计算星盘。
pub async fn post_native(req: &HoroNativeRequest) -> Result<Horoscope, String> {
    let url = format!("{API_BASE_URL}/api/horo/native");
    Request::post(&url)
        .header("Content-Type", "application/json")
        .json(req)
        .map_err(|e| format!("序列化失败：{e}"))?
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?
        .json::<Horoscope>()
        .await
        .map_err(|e| format!("解析响应失败：{e}"))
}
