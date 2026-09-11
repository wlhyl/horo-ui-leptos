//! 本地存储服务：完整对齐 horo-ui 的 HoroStorageService。
//!
//! 七份数据（本命 / 推运 / 合盘 / 天象 / 衍生盘基准星 / 交点命名 / 古代星盘）
//! 同时保存在内存信号与 localStorage 中；键名与 JSON 结构和原版一致，两版数据可互通。
//! 数据结构见 models/ 与 enums/；当前已接入本命盘与天象盘输入页，其余待对应页面迁移后接入。
use leptos::prelude::*;

use crate::{
    enums::{
        arc_to_date_method::ArcToDateMethod,
        custom_lunar_day_profection_method::CustomLunarDayProfectionMethod,
        daily_direction_method::DailyDirectionMethod, direction_method::DirectionMethod,
        house::HouseName, planet::PlanetName, process_name::ProcessName,
        profection_arc_to_date_method::ProfectionArcToDateMethod,
        secondary_progression_method::SecondaryProgressionMethod,
    },
    models::{
        data::{HoroData, ProcessData},
        datetime::DateTimeData,
        geo::GeoPosition,
        historical::{HistoricalData, HistoricalHouseCusp, HistoricalPlanetPosition},
    },
};

// ---------------------------------------------------------------------------
// 存储服务（对应原版 HoroStorageService）
// ---------------------------------------------------------------------------

/// localStorage 键名（与原版一致，两版数据可互通）。
const KEY_HORO_DATA: &str = "horo_data";
const KEY_PROCESS_DATA: &str = "process_data";
const KEY_SYNASTRY_DATA: &str = "synastry_data";
const KEY_EVENT_DATA: &str = "event_data";
const KEY_DERIVED_PLANET_NAME: &str = "derived_planet_name";
const KEY_NODE_NAME_OPTION: &str = "node_name_option";
const KEY_HISTORICAL_DATA: &str = "historical_data";

/// 全局存储：内存信号 + localStorage 双写。
///
/// 在 App 根部 `HoroStorage::init()` 创建一次并 `provide_context` 共享；
/// 结构体为 Copy，各处克隆共享同一批信号。
#[derive(Clone, Copy)]
pub(crate) struct HoroStorage {
    /// 本命盘输入数据
    horo_data: RwSignal<HoroData>,
    /// 推运页输入数据
    process_data: RwSignal<ProcessData>,
    /// 合盘输入数据
    synastry_data: RwSignal<HoroData>,
    /// 天象盘输入数据
    event_data: RwSignal<HoroData>,
    /// 衍生盘基准行星
    derived_planet_name: RwSignal<PlanetName>,
    /// 交点命名是否用「南罗北计」（对应原版 isNanLuoBeiJi）
    is_nan_luo_bei_ji: RwSignal<bool>,
    /// 古代星盘数据
    historical_data: RwSignal<HistoricalData>,
}

// 推运 / 合盘 / 衍生盘 / 七政 / 古代星盘 / 清除缓存页面尚未迁移，部分读写方法暂无调用方
// #[allow(dead_code)]
impl HoroStorage {
    /// 从 localStorage 恢复各份数据；无有效缓存时使用与原版相同的默认值。
    pub(crate) fn init() -> Self {
        HoroStorage {
            horo_data: RwSignal::new(read_stored(KEY_HORO_DATA).unwrap_or_else(default_horo_data)),
            process_data: RwSignal::new(
                read_stored(KEY_PROCESS_DATA).unwrap_or_else(default_process_data),
            ),
            synastry_data: RwSignal::new(
                read_stored(KEY_SYNASTRY_DATA).unwrap_or_else(default_horo_data),
            ),
            event_data: RwSignal::new(
                read_stored(KEY_EVENT_DATA).unwrap_or_else(default_horo_data),
            ),
            derived_planet_name: RwSignal::new(
                read_stored(KEY_DERIVED_PLANET_NAME).unwrap_or(PlanetName::Sun),
            ),
            is_nan_luo_bei_ji: RwSignal::new(read_stored(KEY_NODE_NAME_OPTION).unwrap_or(true)),
            historical_data: RwSignal::new(
                read_stored(KEY_HISTORICAL_DATA).unwrap_or_else(default_historical_data),
            ),
        }
    }

    pub(crate) fn horo_data(&self) -> HoroData {
        self.horo_data.get()
    }

    pub(crate) fn set_horo_data(&self, data: HoroData) {
        write_stored(KEY_HORO_DATA, &data);
        self.horo_data.set(data);
    }

    pub(crate) fn process_data(&self) -> ProcessData {
        self.process_data.get()
    }

    pub(crate) fn set_process_data(&self, data: ProcessData) {
        write_stored(KEY_PROCESS_DATA, &data);
        self.process_data.set(data);
    }

    pub(crate) fn synastry_data(&self) -> HoroData {
        self.synastry_data.get()
    }

    pub(crate) fn set_synastry_data(&self, data: HoroData) {
        write_stored(KEY_SYNASTRY_DATA, &data);
        self.synastry_data.set(data);
    }

    pub(crate) fn event_data(&self) -> HoroData {
        self.event_data.get()
    }

    pub(crate) fn set_event_data(&self, data: HoroData) {
        write_stored(KEY_EVENT_DATA, &data);
        self.event_data.set(data);
    }

    pub(crate) fn derived_planet_name(&self) -> PlanetName {
        self.derived_planet_name.get()
    }

    pub(crate) fn set_derived_planet_name(&self, planet: PlanetName) {
        self.derived_planet_name.set(planet);
        write_stored(KEY_DERIVED_PLANET_NAME, &planet);
    }

    pub(crate) fn is_nan_luo_bei_ji(&self) -> bool {
        self.is_nan_luo_bei_ji.get()
    }

    pub(crate) fn set_is_nan_luo_bei_ji(&self, value: bool) {
        self.is_nan_luo_bei_ji.set(value);
        write_stored(KEY_NODE_NAME_OPTION, &value);
    }

    pub(crate) fn historical_data(&self) -> HistoricalData {
        self.historical_data.get()
    }

    pub(crate) fn set_historical_data(&self, data: HistoricalData) {
        write_stored(KEY_HISTORICAL_DATA, &data);
        self.historical_data.set(data);
    }

    /// 清空 localStorage 中的七份数据并恢复默认值。
    pub(crate) fn clean(&self) {
        for key in [
            KEY_HORO_DATA,
            KEY_PROCESS_DATA,
            KEY_SYNASTRY_DATA,
            KEY_EVENT_DATA,
            KEY_DERIVED_PLANET_NAME,
            KEY_NODE_NAME_OPTION,
            KEY_HISTORICAL_DATA,
        ] {
            remove_stored(key);
        }
        self.horo_data.set(default_horo_data());
        self.process_data.set(default_process_data());
        self.synastry_data.set(default_horo_data());
        self.event_data.set(default_horo_data());
        self.derived_planet_name.set(PlanetName::Sun);
        self.is_nan_luo_bei_ji.set(true);
        self.historical_data.set(default_historical_data());
    }
}

// ---------------------------------------------------------------------------
// 默认值（与原版 horostorage.service.ts 逐字段对齐）
// ---------------------------------------------------------------------------

/// 本命 / 天象 / 合盘默认值：当前时间、北京（东经 116°25′，北纬 39°54′）、芮氏。
fn default_horo_data() -> HoroData {
    HoroData {
        id: 0,
        date: DateTimeData::now(),
        geo_name: "北京".into(),
        geo: GeoPosition {
            long: 116.0 + 25.0 / 60.0,
            lat: 39.0 + 54.0 / 60.0,
        },
        house: HouseName::Regiomontanus,
        name: String::new(),
        sex: true,
    }
}

/// 推运默认值。
fn default_process_data() -> ProcessData {
    ProcessData {
        date: DateTimeData::now(),
        geo_name: "北京".into(),
        geo: GeoPosition {
            long: 116.0 + 25.0 / 60.0,
            lat: 39.0 + 54.0 / 60.0,
        },
        process_name: ProcessName::Direction,
        is_solar_return: true,
        direction_method: DirectionMethod::SemiArc,
        arc_to_date_method: ArcToDateMethod::DegreePerYear,
        profection_arc_to_date_method: ProfectionArcToDateMethod::TrueSolarArc,
        daily_direction_method: DailyDirectionMethod::SolarArc,
        secondary_progression_method: SecondaryProgressionMethod::DegreePerYear,
        lunar_day_profection_method: CustomLunarDayProfectionMethod::Moon,
    }
}

/// 古代星盘默认值：整宫制宫头（白羊 0° 起每 30° 一宫）+ 七颗古典行星。
fn default_historical_data() -> HistoricalData {
    let house_cusps = (0u8..12)
        .map(|i| HistoricalHouseCusp {
            house_number: i + 1,
            longitude_degree: u16::from(i) * 30,
            longitude_minute: 0,
            longitude_second: 0,
        })
        .collect();

    const CLASSICAL: [PlanetName; 7] = [
        PlanetName::Sun,
        PlanetName::Moon,
        PlanetName::Mercury,
        PlanetName::Venus,
        PlanetName::Mars,
        PlanetName::Jupiter,
        PlanetName::Saturn,
    ];
    let planet_positions = CLASSICAL
        .into_iter()
        .enumerate()
        .map(|(i, planet_name)| HistoricalPlanetPosition {
            planet_name,
            longitude_degree: (i as u16) * 30,
            longitude_minute: 0,
            longitude_second: 0,
            latitude_degree: 0,
            latitude_minute: 0,
            latitude_second: 0,
            latitude_north: true,
            is_retrograde: false,
        })
        .collect();

    HistoricalData {
        name: String::new(),
        description: String::new(),
        house_system: "未知".into(),
        house_cusps,
        planet_positions,
    }
}

// ---------------------------------------------------------------------------
// localStorage 读写
// ---------------------------------------------------------------------------

/// 获取 localStorage（非浏览器环境或访问被拒时返回 None）。
/// 访问抛错（如沙箱 iframe 的 SecurityError）只警告一次，避免每次读写刷屏。
fn local_storage() -> Option<web_sys::Storage> {
    let window = web_sys::window()?;
    match window.local_storage() {
        Ok(storage) => storage,
        Err(e) => {
            static WARNED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                leptos::logging::warn!("localStorage 访问失败，数据仅保留在内存中: {e:?}");
            }
            None
        }
    }
}

/// 读取并反序列化；键不存在或 JSON 损坏时返回 None，由调用方走默认值
/// （原版 JSON.parse 遇损坏数据会抛异常，这里更宽容）。
fn read_stored<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
    let json = local_storage()?.get_item(key).ok().flatten()?;
    serde_json::from_str(&json)
        .inspect_err(|e| leptos::logging::warn!("缓存 {key} 损坏，已回退默认值: {e}"))
        .ok()
}

/// 序列化并写入。
fn write_stored<T: serde::Serialize>(key: &str, value: &T) {
    let Some(storage) = local_storage() else {
        return;
    };
    let json = match serde_json::to_string(value) {
        Ok(json) => json,
        Err(e) => {
            leptos::logging::warn!("序列化 {key} 失败，未写入缓存: {e}");
            return;
        }
    };
    if let Err(e) = storage.set_item(key, &json) {
        leptos::logging::warn!("写入缓存 {key} 失败（配额满或权限限制）: {e:?}");
    }
}

/// 删除单个键。
fn remove_stored(key: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(key);
    }
}
