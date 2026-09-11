//! 日期时间数据（对应后台 horo_date_time 的 HoroDateTime，多 st 字段）。
use serde::{Deserialize, Serialize};

/// 本地存储 / 请求共用的日期时间。
///
/// 比后台核心库的 `HoroDateTime` 多一个夏令时标志 `st`（后台 API 契约
/// `DateRequest` 需要）；后台另有的 `ms` / `jd_*` 儒略日字段为内部计算用，
/// 不参与序列化，故此处不设。
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct DateTimeData {
    pub(crate) year: i32,
    pub(crate) month: u8,
    pub(crate) day: u8,
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
    /// 时区，东为正、西为负
    pub(crate) tz: f64,
    /// 夏令时
    pub(crate) st: bool,
}

/// 中国夏令时实行区间（1986–1991）。
/// 起始日 02:00（含）进入夏令时；结束日 02:00（不含）恢复标准时。
/// 元素为 (年, (起始月, 起始日), (结束月, 结束日))，边界时间固定 02:00。
const CHINESE_DST_RANGES: [(i32, (u8, u8), (u8, u8)); 6] = [
    (1986, (5, 4), (9, 14)),
    (1987, (4, 12), (9, 13)),
    (1988, (4, 10), (9, 11)),
    (1989, (4, 16), (9, 17)),
    (1990, (4, 15), (9, 16)),
    (1991, (4, 14), (9, 15)),
];

impl DateTimeData {
    /// 浏览器当前本地时间（对应原版 nowDate）。
    pub(crate) fn now() -> Self {
        let d = js_sys::Date::new_0();
        Self {
            year: d.get_full_year() as i32,
            month: d.get_month() as u8 + 1,
            day: d.get_date() as u8,
            hour: d.get_hours() as u8,
            minute: d.get_minutes() as u8,
            second: d.get_seconds() as u8,
            // JS 返回 UTC−本地（分钟），取负再除 60 得「东正西负」的时区
            tz: -d.get_timezone_offset() / 60.0,
            st: false,
        }
    }

    /// 是否处于中国夏令时期间（1986–1991），移植自原版 `utils/dst/dst.ts`。
    ///
    /// 按（年, 月, 日, 时, 分）元组字典序比较即天然单调，无需像 TS 版那样
    /// 折算时间戳（原版依赖「所有日期同一函数构造、时区偏移相互抵消」的约定）。
    pub(crate) fn is_in_chinese_dst(&self) -> bool {
        let t = (self.year, self.month, self.day, self.hour, self.minute);
        CHINESE_DST_RANGES
            .iter()
            .any(|&(y, (sm, sd), (em, ed))| (y, sm, sd, 2, 0) <= t && t < (y, em, ed, 2, 0))
    }
}
