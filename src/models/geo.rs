//! 地理位置数据（字段与后台 geo_position 的 GeoPosition 一致）。
use serde::{Deserialize, Serialize};

/// 大地经纬度。
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct GeoPosition {
    pub(crate) long: f64,
    pub(crate) lat: f64,
}
