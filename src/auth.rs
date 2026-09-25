//! 认证服务：登录态管理（对应原版 horo-ui 的 AuthService）。
//!
//! JWT token 以裸字符串存于 localStorage（键名 `token`，与原版一致，两版互通）；
//! 前端只解码 payload 读取用户信息，不校验签名（与原版 jwt-decode 行为一致，
//! 签名校验由后台中间件负责）。启动时校验过期时间，失效即清除。

use base64::Engine as _;
use leptos::prelude::*;
use serde::Deserialize;

use crate::storage::local_storage;

/// localStorage 键名（与原版 horo-ui 一致）。
const KEY_TOKEN: &str = "token";

/// JWT claims（horo-storage-api 签发的字段：id/name/role/exp）。
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AuthUser {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) role: String,
    /// 过期时间（秒级 Unix 时间戳）
    pub(crate) exp: i64,
}

/// 全局认证状态：内存信号 + localStorage 双写。
///
/// 在 App 根部 `AuthService::init()` 创建一次并 `provide_context` 共享；
/// 结构体为 Copy，各处克隆共享同一个信号（同 HoroStorage 模式）。
#[derive(Clone, Copy)]
pub(crate) struct AuthService {
    user: RwSignal<Option<AuthUser>>,
}

impl AuthService {
    /// 从 localStorage 恢复登录态：解析 token 并校验过期，
    /// 无 token / 解析失败 / 已过期均视为未登录（后两者清除残留 token）。
    pub(crate) fn init() -> Self {
        let user = match stored_token() {
            Some(token) => match parse_token(&token).filter(is_valid) {
                Some(user) => Some(user),
                None => {
                    remove_stored_token();
                    None
                }
            },
            None => None,
        };
        Self {
            user: RwSignal::new(user),
        }
    }

    /// 当前登录用户（None = 未登录）。
    pub(crate) fn user(&self) -> Option<AuthUser> {
        self.user.get()
    }

    /// 登录成功：存储 token 并解析用户信息。
    /// 后台刚签发的 token 解析失败几乎不可能发生，仍保留 token 以便下次启动重试。
    pub(crate) fn login(&self, token: &str) {
        store_token(token);
        match parse_token(token) {
            Some(user) => self.user.set(Some(user)),
            None => leptos::logging::warn!("登录成功但解析 token 失败，已保留 token"),
        }
    }

    /// 注销：清除 token 与登录态。
    pub(crate) fn logout(&self) {
        remove_stored_token();
        self.user.set(None);
    }
}

/// 未过期（exp 为秒，js_sys::Date::now 为毫秒）。
fn is_valid(user: &AuthUser) -> bool {
    (user.exp as f64) * 1000.0 > js_sys::Date::now()
}

/// 解码 JWT payload（base64url 无 padding）为用户信息，任一步失败返回 None。
fn parse_token(token: &str) -> Option<AuthUser> {
    let payload = token.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

// ---------------------------------------------------------------------------
// token 裸读写（非 JSON 封装，与原版格式互通）
// ---------------------------------------------------------------------------

fn stored_token() -> Option<String> {
    local_storage()?.get_item(KEY_TOKEN).ok().flatten()
}

fn store_token(token: &str) {
    if let Some(storage) = local_storage() {
        if let Err(e) = storage.set_item(KEY_TOKEN, token) {
            leptos::logging::warn!("写入 token 失败（配额满或权限限制）: {e:?}");
        }
    }
}

fn remove_stored_token() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(KEY_TOKEN);
    }
}
