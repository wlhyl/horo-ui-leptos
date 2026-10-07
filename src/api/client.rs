//! 后台 HTTP 调用封装（gloo-net，wasm fetch）。

use crate::api::request::{
    DailyDirectionRequest, DerivedHoroRequest, DirectionRequest, HoroNativeRequest,
    HoroscopeRecordRequest, LoginRequest, ReturnRequest, SolarArcRequest,
    UpdateHoroscopeRecordRequest,
};
use crate::api::response::{
    Direction, Horoscope, HoroscopeRecord, LocationResponse, PageResponser, ReturnHoroscope,
    TokenResponse,
};
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

/// 调用 POST /api/horo/derived 计算衍生盘（基准行星的斜升为中天，
/// 行星、恒星数据复用本命盘，响应不含日主星 / 时主星）。
pub async fn post_derived(req: &DerivedHoroRequest) -> Result<Horoscope, String> {
    let url = format!("{API_BASE_URL}/api/horo/derived");
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

// ---------------------------------------------------------------------------
// 方向推运（horo-api /api/process/*，与原版 horo-ui 共用同一批接口）
// ---------------------------------------------------------------------------

/// 调用 POST /api/process/directions 计算主向推运表。
pub async fn post_direction(req: &DirectionRequest) -> Result<Vec<Direction>, String> {
    let url = format!("{API_BASE_URL}/api/process/directions");
    Request::post(&url)
        .header("Content-Type", "application/json")
        .json(req)
        .map_err(|e| format!("序列化失败：{e}"))?
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?
        .json::<Vec<Direction>>()
        .await
        .map_err(|e| format!("解析响应失败：{e}"))
}

/// 调用 POST /api/process/solar_arc 计算太阳弧推运表。
pub async fn post_solar_arc(req: &SolarArcRequest) -> Result<Vec<Direction>, String> {
    let url = format!("{API_BASE_URL}/api/process/solar_arc");
    Request::post(&url)
        .header("Content-Type", "application/json")
        .json(req)
        .map_err(|e| format!("序列化失败：{e}"))?
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?
        .json::<Vec<Direction>>()
        .await
        .map_err(|e| format!("解析响应失败：{e}"))
}

/// 调用 POST /api/process/daily_directions 计算每日回归方向弧表
/// （native_date 传每日返照时刻、st 固定 false）。
pub async fn post_daily_direction(req: &DailyDirectionRequest) -> Result<Vec<Direction>, String> {
    let url = format!("{API_BASE_URL}/api/process/daily_directions");
    Request::post(&url)
        .header("Content-Type", "application/json")
        .json(req)
        .map_err(|e| format!("序列化失败：{e}"))?
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?
        .json::<Vec<Direction>>()
        .await
        .map_err(|e| format!("解析响应失败：{e}"))
}

/// 调用 POST /api/process/return/solar 计算太阳返照盘（取返照时刻）。
pub async fn post_solar_return(req: &ReturnRequest) -> Result<ReturnHoroscope, String> {
    post_return("solar", req).await
}

/// 调用 POST /api/process/return/lunar 计算月亮返照盘（取返照时刻）。
pub async fn post_lunar_return(req: &ReturnRequest) -> Result<ReturnHoroscope, String> {
    post_return("lunar", req).await
}

/// 调用 POST /api/process/return/daily 计算每日回归盘（取返照时刻）。
pub async fn post_daily_return(req: &ReturnRequest) -> Result<ReturnHoroscope, String> {
    post_return("daily", req).await
}

/// 返照接口公共请求逻辑：URL 仅最后一段不同。
async fn post_return(kind: &str, req: &ReturnRequest) -> Result<ReturnHoroscope, String> {
    let url = format!("{API_BASE_URL}/api/process/return/{kind}");
    Request::post(&url)
        .header("Content-Type", "application/json")
        .json(req)
        .map_err(|e| format!("序列化失败：{e}"))?
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?
        .json::<ReturnHoroscope>()
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
    let q = js_sys::encode_uri_component(q)
        .as_string()
        .unwrap_or_default();
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

/// 调用 GET /api/horo-admin/horoscopes 分页列出档案记录。
pub async fn get_horoscopes(
    page: u64,
    size: u64,
    token: &str,
) -> Result<PageResponser<HoroscopeRecord>, String> {
    let url = format!("{ADMIN_API_BASE_URL}/api/horo-admin/horoscopes?page={page}&size={size}");
    fetch_records(&url, token).await
}

/// 调用 GET /api/horo-admin/horoscopes/search 按姓名模糊搜索档案记录。
pub async fn search_horoscopes(
    page: u64,
    size: u64,
    name: &str,
    token: &str,
) -> Result<PageResponser<HoroscopeRecord>, String> {
    // 姓名可能含中文，必须 URL 编码
    let name = js_sys::encode_uri_component(name)
        .as_string()
        .unwrap_or_default();
    let url = format!(
        "{ADMIN_API_BASE_URL}/api/horo-admin/horoscopes/search?page={page}&size={size}&name={name}"
    );
    fetch_records(&url, token).await
}

/// 档案记录接口公共请求逻辑：token 头 + 状态码错误映射
/// （400 校验失败 / 401 未认证 / 403 无权限 / 404 不存在，均返回 `{"error": "..."}`）。
async fn fetch_records(url: &str, token: &str) -> Result<PageResponser<HoroscopeRecord>, String> {
    let res = Request::get(url)
        .header("token", token)
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?;
    match res.status() {
        200 => res
            .json::<PageResponser<HoroscopeRecord>>()
            .await
            .map_err(|e| format!("解析响应失败：{e}")),
        400 | 401 | 403 | 404 => {
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

/// 调用 POST /api/horo-admin/horoscopes 新增档案记录，成功返回含新 id 的完整记录。
/// 后台校验失败（如姓名为空、年份越界）返回 400 与 `{"error": "..."}`。
pub async fn add_horoscope(
    req: &HoroscopeRecordRequest,
    token: &str,
) -> Result<HoroscopeRecord, String> {
    let url = format!("{ADMIN_API_BASE_URL}/api/horo-admin/horoscopes");
    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("token", token)
        .json(req)
        .map_err(|e| format!("序列化失败：{e}"))?
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?;
    match res.status() {
        200 => res
            .json::<HoroscopeRecord>()
            .await
            .map_err(|e| format!("解析响应失败：{e}")),
        400 | 401 | 403 | 404 => {
            let msg = res
                .json::<ErrorResponse>()
                .await
                .map(|r| r.error)
                .unwrap_or_else(|_| "存档失败".into());
            Err(msg)
        }
        status => Err(format!("存档失败（HTTP {status}）")),
    }
}

/// 调用 PUT /api/horo-admin/horoscopes/{id} 更新档案记录（200 空 body）。
/// 请求体为 diff 语义：null 字段后台不修改；锁定记录更新非描述字段返回 400。
pub async fn update_horoscope(
    id: u32,
    req: &UpdateHoroscopeRecordRequest,
    token: &str,
) -> Result<(), String> {
    let url = format!("{ADMIN_API_BASE_URL}/api/horo-admin/horoscopes/{id}");
    let res = Request::put(&url)
        .header("Content-Type", "application/json")
        .header("token", token)
        .json(req)
        .map_err(|e| format!("序列化失败：{e}"))?
        .send()
        .await
        .map_err(|e| format!("网络错误：{e}"))?;
    match res.status() {
        200 => Ok(()),
        400 | 401 | 403 | 404 => {
            let msg = res
                .json::<ErrorResponse>()
                .await
                .map(|r| r.error)
                .unwrap_or_else(|_| "存档失败".into());
            Err(msg)
        }
        status => Err(format!("存档失败（HTTP {status}）")),
    }
}
