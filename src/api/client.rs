//! 后台 HTTP 调用封装（gloo-net，wasm fetch）。

use crate::api::request::{HoroNativeRequest, LoginRequest};
use crate::api::response::{Horoscope, LocationResponse, TokenResponse};
use crate::config::{ADMIN_API_BASE_URL, API_BASE_URL};
use gloo_net::http::Request;
use serde::Deserialize;

/// 错误响应体（horo-storage-api）：`{"error": "..."}`。
#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

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

/// 调用 POST /api/horo-admin/login 登录，成功返回 JWT token。
/// 错误消息按状态码映射（404 用户不存在 / 403 密码错误）。
pub async fn post_login(req: &LoginRequest) -> Result<String, String> {
    let url = format!("{ADMIN_API_BASE_URL}/api/horo-admin/login");
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(req)
        .map_err(|e| format!("序列化失败：{e}"))?
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?;
    match res.status() {
        200 => res
            .json::<TokenResponse>()
            .await
            .map(|t| t.token)
            .map_err(|e| format!("解析响应失败：{e}")),
        404 => Err("用户不存在".into()),
        403 => Err("密码错误".into()),
        status => Err(format!("登录失败（HTTP {status}）")),
    }
}

/// 调用 GET /api/horo-admin/location_search 根据地名搜索经纬度。
/// 404（未找到地名）/ 403（token 失效）映射为后台返回的 error 消息。
pub async fn get_location_search(q: &str, token: &str) -> Result<Vec<LocationResponse>, String> {
    // 中文地名必须 URL 编码
    let q = js_sys::encode_uri_component(q).as_string().unwrap_or_default();
    let url = format!("{ADMIN_API_BASE_URL}/api/horo-admin/location_search?q={q}");
    let res = Request::get(&url)
        .header("token", token)
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?;
    match res.status() {
        200 => res
            .json::<Vec<LocationResponse>>()
            .await
            .map_err(|e| format!("解析响应失败：{e}")),
        404 | 403 => {
            let msg = res
                .json::<ErrorResponse>()
                .await
                .map(|r| r.error)
                .unwrap_or_else(|_| "查询失败".into());
            Err(msg)
        }
        status => Err(format!("查询失败（HTTP {status}）")),
    }
}
