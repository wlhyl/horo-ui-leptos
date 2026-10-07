//! 星盘输入表单状态（本命 / 天象 / 工作台共用）。
//!
//! 保存在 Store 中：字段级细粒度响应，改一个字段不会惊扰其他字段的绑定。
//! 与缓存模型 `HoroData` 双向转换：进入页面时 From<HoroData> 恢复，
//! 提交或开窗口时 From<&FormState> 取快照。
use reactive_stores::Store;

use crate::enums::house::HouseName;
use crate::models::data::HoroData;
use crate::models::datetime::DateTimeData;
use crate::models::geo::GeoPosition;

/// 表单状态（保存在 Store 中：字段级细粒度响应，改一个字段不会惊扰其他字段的绑定）。
#[derive(Clone, Store)]
pub(crate) struct FormState {
    /// 档案 ID（随缓存数据往返，表单不编辑）
    pub id: u32,
    pub name: String,
    pub sex: bool,
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub tz: f64,
    pub st: bool,
    /// 地点名称（随缓存数据往返，由地点选择功能写入）
    pub geo_name: String,
    pub long: f64,
    pub lat: f64,
    pub house: HouseName,
}

impl From<HoroData> for FormState {
    fn from(r: HoroData) -> Self {
        FormState {
            id: r.id,
            name: r.name,
            sex: r.sex,
            year: r.date.year,
            month: r.date.month,
            day: r.date.day,
            hour: r.date.hour,
            minute: r.date.minute,
            second: r.date.second,
            tz: r.date.tz,
            st: r.date.st,
            geo_name: r.geo_name,
            long: r.geo.long,
            lat: r.geo.lat,
            house: r.house,
        }
    }
}

impl From<&FormState> for HoroData {
    fn from(s: &FormState) -> Self {
        HoroData {
            id: s.id,
            date: s.date(),
            geo_name: s.geo_name.clone(),
            geo: GeoPosition {
                long: s.long,
                lat: s.lat,
            },
            house: s.house,
            name: s.name.clone(),
            sex: s.sex,
        }
    }
}

impl FormState {
    /// 汇总日期时间字段（转换回 HoroData / ProcessData 用）。
    pub(crate) fn date(&self) -> DateTimeData {
        DateTimeData {
            year: self.year,
            month: self.month,
            day: self.day,
            hour: self.hour,
            minute: self.minute,
            second: self.second,
            tz: self.tz,
            st: self.st,
        }
    }
}
