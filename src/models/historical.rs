//! 古代星盘数据（对应原版 horo-admin/historical-horoscope.ts）。
use serde::{Deserialize, Serialize};

use crate::enums::planet::PlanetName;

/// 古代星盘宫头（后台对应结构体 HouseCusp）。
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct HistoricalHouseCusp {
    /// 1-12
    pub(crate) house_number: u8,
    /// 0-359
    pub(crate) longitude_degree: u16,
    /// 0-59
    pub(crate) longitude_minute: u8,
    /// 0-59
    pub(crate) longitude_second: u8,
}

/// 古代星盘行星位置（后台对应结构体 PlanetPosition）。
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct HistoricalPlanetPosition {
    pub(crate) planet_name: PlanetName,
    /// 黄经 0-359
    pub(crate) longitude_degree: u16,
    pub(crate) longitude_minute: u8,
    pub(crate) longitude_second: u8,
    /// 黄纬 0-90
    pub(crate) latitude_degree: u8,
    pub(crate) latitude_minute: u8,
    pub(crate) latitude_second: u8,
    pub(crate) latitude_north: bool,
    pub(crate) is_retrograde: bool,
}

/// 古代星盘暂存数据（对应原版 HistoricalStorageData）。
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalData {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) house_system: String,
    pub(crate) house_cusps: Vec<HistoricalHouseCusp>,
    pub(crate) planet_positions: Vec<HistoricalPlanetPosition>,
}
