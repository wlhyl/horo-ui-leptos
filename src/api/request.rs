//! 与后台 horo-api 严格对齐的请求数据契约。
//! 后台 serde 默认使用 snake_case，枚举序列化为枚举名字符串（英文）。
//! 仅声明前端需要的字段，其余字段由 serde 自动忽略。

use serde::Serialize;

use crate::{
    enums::{house::HouseName, planet::PlanetName},
    models::datetime::DateTimeData,
    models::geo::GeoPosition,
};

/// 本命 / 天象盘共用请求体（name/sex 为界面展示用，后台不接收）。
#[derive(Clone, Copy, Serialize)]
pub struct HoroNativeRequest {
    pub date: DateTimeData,
    pub geo: GeoPosition,
    pub house: HouseName,
}

/// 衍生盘请求体：以指定行星的斜升（OA）为基准计算衍生盘。
#[derive(Clone, Copy, Serialize)]
pub struct DerivedHoroRequest {
    pub date: DateTimeData,
    pub geo: GeoPosition,
    pub house: HouseName,
    /// 衍生盘的基准行星
    pub planet_name: PlanetName,
}

/// 登录请求体（horo-storage-api：name/password 均要求非空）。
#[derive(Serialize)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
}
