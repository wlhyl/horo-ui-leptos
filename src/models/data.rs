//! 页面输入数据（对应原版 request-data.ts）。
use serde::{Deserialize, Serialize};

use crate::{
    enums::{
        arc_to_date_method::ArcToDateMethod,
        custom_lunar_day_profection_method::CustomLunarDayProfectionMethod,
        daily_direction_method::DailyDirectionMethod, direction_method::DirectionMethod,
        house::HouseName, process_name::ProcessName,
        profection_arc_to_date_method::ProfectionArcToDateMethod,
        secondary_progression_method::SecondaryProgressionMethod,
    },
    models::{datetime::DateTimeData, geo::GeoPosition},
};

/// 本命 / 天象 / 合盘输入数据（对应原版 HoroRequest）。
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct HoroData {
    /// 档案 ID（未从档案库载入时为 0）
    pub(crate) id: u32,
    pub(crate) date: DateTimeData,
    pub(crate) geo_name: String,
    pub(crate) geo: GeoPosition,
    pub(crate) house: HouseName,
    pub(crate) name: String,
    pub(crate) sex: bool,
}

/// 推运页输入数据（对应原版 ProcessRequest）。
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ProcessData {
    pub(crate) date: DateTimeData,
    pub(crate) geo_name: String,
    pub(crate) geo: GeoPosition,
    pub(crate) process_name: ProcessName,
    pub(crate) is_solar_return: bool,
    pub(crate) direction_method: DirectionMethod,
    pub(crate) arc_to_date_method: ArcToDateMethod,
    pub(crate) profection_arc_to_date_method: ProfectionArcToDateMethod,
    pub(crate) daily_direction_method: DailyDirectionMethod,
    pub(crate) secondary_progression_method: SecondaryProgressionMethod,
    pub(crate) lunar_day_profection_method: CustomLunarDayProfectionMethod,
}
