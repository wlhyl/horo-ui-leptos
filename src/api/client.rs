//! 后台 HTTP 调用封装（gloo-net，wasm fetch）。

use crate::api::request::{HoroNativeRequest, LoginRequest};
use crate::api::response::{Horoscope, TokenResponse};
use crate::config::{ADMIN_API_BASE_URL, API_BASE_URL};
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
