//! 与后台 horo-api / horo-storage-api 严格对齐的请求数据契约。
//! 后台 serde 默认使用 snake_case，枚举序列化为枚举名字符串（英文）。
//! 仅声明前端需要的字段，其余字段由 serde 自动忽略。

use serde::Serialize;

use crate::{
    api::response::ChartType,
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

/// 档案记录的出生地（度分秒 + 半球标志，对齐 storage-api 的 LocationRequest）。
#[derive(Clone, Serialize)]
pub struct RecordLocationRequest {
    pub name: String,
    /// 东经 true / 西经 false
    pub is_east: bool,
    pub longitude_degree: u16,
    pub longitude_minute: u8,
    pub longitude_second: u8,
    /// 北纬 true / 南纬 false
    pub is_north: bool,
    pub latitude_degree: u8,
    pub latitude_minute: u8,
    pub latitude_second: u8,
}

/// 新增档案记录请求体（对齐原版 HoroscopeRecordRequest / storage-api 的 HoroscopeRequest）。
#[derive(Clone, Serialize)]
pub struct HoroscopeRecordRequest {
    pub name: String,
    pub gender: bool,
    pub birth_year: i32,
    pub birth_month: u8,
    pub birth_day: u8,
    pub birth_hour: u8,
    pub birth_minute: u8,
    pub birth_second: u8,
    /// 出生地时区（东区为正，西区为负）
    pub time_zone_offset: f64,
    pub is_dst: bool,
    pub chart_type: ChartType,
    pub is_time_precise: bool,
    pub location: RecordLocationRequest,
    /// 无笔记功能，固定空串
    pub description: String,
    pub lock: bool,
}

/// 更新档案记录请求体（对齐 storage-api 的 UpdateHoroscopeRequest）：
/// 全 Option，None 序列化为 null 表示不修改该字段；表单未覆盖的
/// description / chart_type / is_time_precise / lock 传 null 由后台保持原值。
#[derive(Clone, Serialize)]
pub struct UpdateHoroscopeRecordRequest {
    pub name: Option<String>,
    pub gender: Option<bool>,
    pub birth_year: Option<i32>,
    pub birth_month: Option<u8>,
    pub birth_day: Option<u8>,
    pub birth_hour: Option<u8>,
    pub birth_minute: Option<u8>,
    pub birth_second: Option<u8>,
    pub time_zone_offset: Option<f64>,
    pub is_dst: Option<bool>,
    pub location: Option<RecordLocationRequest>,
    pub description: Option<String>,
    pub chart_type: Option<ChartType>,
    pub is_time_precise: Option<bool>,
    pub lock: Option<bool>,
}
