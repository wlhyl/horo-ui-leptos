//! 与后台 horo-api / horo-storage-api 严格对齐的响应数据契约。
//! 后台 serde 默认使用 snake_case，枚举序列化为枚举名字符串（英文）。
//! 仅声明前端需要的字段，其余字段由 serde 自动忽略。

use serde::Deserialize;

use crate::enums::{
    house::HouseName,
    planet::{PlanetName, PlanetSpeedState},
};

/// 登录成功响应（horo-storage-api）：JWT token。
#[derive(Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

/// 地名搜索结果（horo-storage-api location_search）。
#[derive(Clone, Deserialize)]
pub struct LocationResponse {
    pub name: String,
    /// 经度（字符串形式，nominatim 上游如此）
    pub longitude: String,
    /// 纬度（字符串形式）
    pub latitude: String,
}

#[derive(Clone, Copy, Deserialize)]
pub struct Planet {
    pub name: PlanetName,
    /// 黄经（度）
    pub long: f64,
    /// 黄纬（度）
    pub lat: f64,
    /// 每日移动速度（黄经），<0 表示逆行
    pub speed: f64,
    pub ra: f64,
    pub dec: f64,
    pub orb: u8,
    /// 速度状态：快、平均、慢
    pub speed_state: PlanetSpeedState,
}

#[derive(Clone, Copy, Deserialize)]
pub struct Aspect {
    /// 相位角度（0/30/60/90/120/150/180...）
    pub aspect_value: u8,
    /// 入相位 true / 出相位 false
    pub apply: bool,
    /// 容许度 / 差值（度）
    pub d: f64,
    pub p0: PlanetName,
    pub p1: PlanetName,
}

#[derive(Clone, Deserialize)]
pub struct FixedStar {
    pub fixed_star: String,
    pub long: f64,
    pub xiu: String,
    pub xiu_degree: f64,
    pub desc: String,
}

#[derive(Clone, Deserialize)]
pub struct Horoscope {
    pub house_name: HouseName,
    /// 12 宫头黄经
    pub cusps: Vec<f64>,
    pub asc: Planet,
    pub mc: Planet,
    pub dsc: Planet,
    pub ic: Planet,
    pub part_of_fortune: Planet,
    pub planets: Vec<Planet>,
    pub is_diurnal: bool,
    pub planetary_day: PlanetName,
    pub planetary_hours: PlanetName,
    pub aspects: Vec<Aspect>,
    pub antiscoins: Vec<Aspect>,
    pub contraantiscias: Vec<Aspect>,
    pub fixed_stars: Vec<FixedStar>,
}
