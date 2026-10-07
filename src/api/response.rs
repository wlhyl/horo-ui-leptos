//! 与后台 horo-api / horo-storage-api 严格对齐的响应数据契约。
//! 后台 serde 默认使用 snake_case，枚举序列化为枚举名字符串（英文）。
//! 仅声明前端需要的字段，其余字段由 serde 自动忽略。

use serde::{Deserialize, Serialize};

use crate::astro::horo_math::deg_norm;
use crate::enums::{
    house::HouseName,
    planet::{PlanetName, PlanetSpeedState},
};
use crate::models::datetime::DateTimeData;

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

/// 档案记录的盘类型（horo-storage-api）：本命 natal / 卜卦 horary。
/// 请求 / 响应共用（存档时序列化为枚举名字符串）。
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Natal,
    Horary,
}

/// 档案记录中的出生地（度分秒 + 半球标志，对应 storage-api 的 Location）。
#[derive(Clone, Deserialize)]
pub struct RecordLocation {
    pub name: String,
    pub is_east: bool,
    pub longitude_degree: u16,
    pub longitude_minute: u8,
    pub longitude_second: u8,
    pub is_north: bool,
    pub latitude_degree: u8,
    pub latitude_minute: u8,
    pub latitude_second: u8,
}

/// 档案记录（horo-storage-api 的 horoscopes 接口），仅声明前端需要的字段。
#[derive(Clone, Deserialize)]
pub struct HoroscopeRecord {
    pub id: u32,
    pub name: String,
    pub gender: bool,
    pub birth_year: i32,
    pub birth_month: u8,
    pub birth_day: u8,
    pub birth_hour: u8,
    pub birth_minute: u8,
    pub birth_second: u8,
    pub time_zone_offset: f64,
    pub is_dst: bool,
    pub location: RecordLocation,
    pub chart_type: ChartType,
    pub is_time_precise: bool,
}

/// 分页响应（horo-storage-api）：`total` 为总页数。
#[derive(Deserialize)]
pub struct PageResponser<T> {
    pub data: Vec<T>,
    pub total: u64,
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
    /// 日主星 / 时主星：衍生盘响应不含这两个字段（行星数据复用本命盘），缺省为 None
    pub planetary_day: Option<PlanetName>,
    pub planetary_hours: Option<PlanetName>,
    pub aspects: Vec<Aspect>,
    pub antiscoins: Vec<Aspect>,
    pub contraantiscias: Vec<Aspect>,
    pub fixed_stars: Vec<FixedStar>,
}

/// 返照盘响应（horo-api /api/process/return/*），仅声明前端需要的字段
/// （后台 ReturnHoroscop 另有 native_date / process_date / geo，由 serde 自动忽略）。
/// 后台不返回恒星与日主星 / 时主星（转 Horoscope 时分别置空 / None）。
#[derive(Clone, Deserialize)]
pub struct ReturnHoroscope {
    /// 返照发生的准确时刻
    pub return_date: DateTimeData,
    pub house_name: HouseName,
    /// 12 宫头黄经
    pub cusps: Vec<f64>,
    pub asc: Planet,
    pub mc: Planet,
    pub dsc: Planet,
    pub ic: Planet,
    /// 七颗行星
    pub planets: Vec<Planet>,
    pub part_of_fortune: Planet,
    /// 行星相位，仅包含四轴、行星间的相位
    pub aspects: Vec<Aspect>,
    /// 映点
    pub antiscoins: Vec<Aspect>,
    /// 反映点
    pub contraantiscias: Vec<Aspect>,
}

impl From<ReturnHoroscope> for Horoscope {
    /// 返照盘响应转星盘视图模型，复用轮盘 / 相位 / 详情组件：
    /// 恒星后台不返回（置空），日主星 / 时主星同衍生盘置 None；
    /// is_diurnal 按太阳是否在地平线上计算——太阳位于第 7–12 宫（黄经自
    /// DSC cusps[6] 前行至 ASC cusps[0] 的半圆，恰经 MC）为白天盘。
    fn from(r: ReturnHoroscope) -> Self {
        let dsc_to_asc = deg_norm(r.cusps[0] - r.cusps[6]);
        let is_diurnal = r
            .planets
            .iter()
            .find(|p| p.name == PlanetName::Sun)
            .is_some_and(|sun| deg_norm(sun.long - r.cusps[6]) < dsc_to_asc);
        Horoscope {
            house_name: r.house_name,
            cusps: r.cusps,
            asc: r.asc,
            mc: r.mc,
            dsc: r.dsc,
            ic: r.ic,
            part_of_fortune: r.part_of_fortune,
            planets: r.planets,
            is_diurnal,
            planetary_day: None,
            planetary_hours: None,
            aspects: r.aspects,
            antiscoins: r.antiscoins,
            contraantiscias: r.contraantiscias,
            fixed_stars: Vec::new(),
        }
    }
}

/// 方向推运的显著星（被推运方）：行星 / 四轴福点，或宫头（1-12）。
/// 后端按「单键对象」序列化（如 {"planet":"Sun"} / {"cusp":3}），故用 untagged。
#[derive(Clone, Copy, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Significator {
    Planet { planet: PlanetName },
    Cusp { cusp: u8 },
}

/// 方向推运的承诺星（触发方）13 种形态（对应原版 Promittor）。
/// 后端按「单键对象」序列化，键为变体名（部分 camelCase），故用 untagged。
#[derive(Clone, Copy, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Promittor {
    Conjunction {
        conjunction: PlanetName,
    },
    SinisterTrine {
        #[serde(rename = "sinisterTrine")]
        sinister_trine: PlanetName,
    },
    DexterTrine {
        #[serde(rename = "dexterTrine")]
        dexter_trine: PlanetName,
    },
    SinisterSextile {
        #[serde(rename = "sinisterSextile")]
        sinister_sextile: PlanetName,
    },
    DexterSextile {
        #[serde(rename = "dexterSextile")]
        dexter_sextile: PlanetName,
    },
    SinisterSquare {
        #[serde(rename = "sinisterSquare")]
        sinister_square: PlanetName,
    },
    DexterSquare {
        #[serde(rename = "dexterSquare")]
        dexter_square: PlanetName,
    },
    Opposition {
        opposition: PlanetName,
    },
    /// 界：界主星 + 界终点黄经
    Term {
        term: (PlanetName, f64),
    },
    /// 映点
    Antiscoins {
        antiscoins: PlanetName,
    },
    /// 反映点
    Contraantiscias {
        contraantiscias: PlanetName,
    },
    /// 宫头（1-12）
    Cusp {
        cusp: u8,
    },
    /// 星座（0-11 黄道序号）
    Sign {
        sign: u8,
    },
}

/// 一条方向推运记录（对应原版 Direction）。
#[derive(Clone, Copy, PartialEq, Deserialize)]
pub struct Direction {
    /// 显著星（主向推运的主向星为 MC）
    pub significator: Significator,
    /// 承诺星
    pub promittor: Promittor,
    /// 方向弧度数，负值为反向
    pub arc: f64,
    /// 弧转日期后的推运时间（响应侧 HoroDateTime 无 st 字段，反序列化缺省 false）
    pub date: DateTimeData,
}
