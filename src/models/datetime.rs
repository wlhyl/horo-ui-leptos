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

/// 步进单位（星盘页时间步进器的六个档位，对应原版 date-time-change 的 stepUnit）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepUnit {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

/// 某年是否为公历闰年（4/100/400 规则）。
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// 某年某月的天数（month 取 1–12）。
fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => unreachable!("month 已归一化到 1–12"),
    }
}

/// 把可能溢出的 (年, 月, 日, 时, 分, 秒) 归一化为合法日期时间（月为 1 起始）。
///
/// 语义对齐 JS Date 的 MakeDay/MakeDate：先归一化年月（闰年判定跟随归一化后
/// 的年份），时分秒溢出折算成整天数并入日，日再按所在自然月天数进位/借位
/// （如 2020-01-31 +1月 → 2020-03-02、2020-02-29 +1年 → 2021-03-01）。
fn normalize(
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
) -> (i32, u8, u8, u8, u8, u8) {
    // 时分秒固定进制，溢出部分折算成整天数并入 day
    let minute = minute + second.div_euclid(60);
    let second = second.rem_euclid(60);
    let hour = hour + minute.div_euclid(60);
    let minute = minute.rem_euclid(60);
    let mut day = day + hour.div_euclid(24);
    let hour = hour.rem_euclid(24);

    // 年月先归一化（month 为 1 起始，可任意溢出）
    let mut year = year + (month - 1).div_euclid(12);
    let mut month = (month - 1).rem_euclid(12) + 1;

    // 日按所在月天数进位/借位，可能跨月连进（12月 → 次年1月）
    loop {
        let dim = days_in_month(year, month as u8) as i32;
        if day > dim {
            day -= dim;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        } else if day < 1 {
            month -= 1;
            if month < 1 {
                month = 12;
                year -= 1;
            }
            day += days_in_month(year, month as u8) as i32;
        } else {
            break;
        }
    }

    (
        year,
        month as u8,
        day as u8,
        hour as u8,
        minute as u8,
        second as u8,
    )
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

    /// 当月天数（含闰年），选择面板的日列候选项用。
    pub(crate) fn days_in_month_of(&self) -> u8 {
        days_in_month(self.year, self.month)
    }

    /// 把日钳制到当月最后一天（选择面板改年 / 月后日可能溢出当月时用）。
    pub(crate) fn clamp_day(&mut self) {
        let dim = days_in_month(self.year, self.month);
        if self.day > dim {
            self.day = dim;
        }
    }

    /// 按单位加减 amount（可为负）并归一化，返回新值；时区与夏令时不变。
    ///
    /// 对应原版 image 页 applyStepChange 的 JS Date 日历运算：溢出进位而非
    /// 钳制（1月31日 +1月 → 3月2日）。星盘页步进器一次只步进一个单位，
    /// 只改内存副本，不写 localStorage。
    pub(crate) fn stepped(&self, unit: StepUnit, amount: i32) -> Self {
        let (year, month, day, hour, minute, second) = normalize(
            self.year + if unit == StepUnit::Year { amount } else { 0 },
            self.month as i32 + if unit == StepUnit::Month { amount } else { 0 },
            self.day as i32 + if unit == StepUnit::Day { amount } else { 0 },
            self.hour as i32 + if unit == StepUnit::Hour { amount } else { 0 },
            self.minute as i32 + if unit == StepUnit::Minute { amount } else { 0 },
            self.second as i32 + if unit == StepUnit::Second { amount } else { 0 },
        );
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            tz: self.tz,
            st: self.st,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> DateTimeData {
        DateTimeData {
            year,
            month,
            day,
            hour,
            minute,
            second,
            tz: 8.0,
            st: false,
        }
    }

    fn step(d: &DateTimeData, unit: StepUnit, amount: i32) -> DateTimeData {
        d.stepped(unit, amount)
    }

    #[test]
    fn month_overflow_carries_like_js_date() {
        // 1月31日 +1月：2月无31日，进位到3月（闰年2月29天）
        let d = step(&dt(2020, 1, 31, 12, 0, 0), StepUnit::Month, 1);
        assert_eq!((d.year, d.month, d.day), (2020, 3, 2));
        // 3月31日 -1月：借位语义与 JS Date setMonth 一致
        let d = step(&dt(2020, 3, 31, 12, 0, 0), StepUnit::Month, -1);
        assert_eq!((d.year, d.month, d.day), (2020, 3, 2));
        // 平年2月28天
        let d = step(&dt(2021, 1, 31, 0, 0, 0), StepUnit::Month, 1);
        assert_eq!((d.year, d.month, d.day), (2021, 3, 3));
    }

    #[test]
    fn year_step_follows_leap_rules() {
        // 闰日 +1年 → 3月1日
        let d = step(&dt(2020, 2, 29, 0, 0, 0), StepUnit::Year, 1);
        assert_eq!((d.year, d.month, d.day), (2021, 3, 1));
        // 闰日 +4年 → 仍是闰日
        let d = step(&dt(2020, 2, 29, 8, 30, 0), StepUnit::Year, 4);
        assert_eq!((d.year, d.month, d.day), (2024, 2, 29));
        // 世纪年：1900 非闰年
        let d = step(&dt(1900, 2, 28, 0, 0, 0), StepUnit::Year, 0);
        assert_eq!(d.day, 28);
    }

    #[test]
    fn day_step_crosses_month_and_year() {
        // 12月31日 +1日 → 次年1月1日
        let d = step(&dt(2019, 12, 31, 23, 59, 59), StepUnit::Day, 1);
        assert_eq!((d.year, d.month, d.day), (2020, 1, 1));
        // 1月1日 -1日 → 上一年12月31日
        let d = step(&dt(2020, 1, 1, 0, 0, 0), StepUnit::Day, -1);
        assert_eq!((d.year, d.month, d.day), (2019, 12, 31));
        // 3月1日 -1日：闰年回到2月29，平年回到2月28
        let d = step(&dt(2020, 3, 1, 0, 0, 0), StepUnit::Day, -1);
        assert_eq!((d.year, d.month, d.day), (2020, 2, 29));
        let d = step(&dt(2021, 3, 1, 0, 0, 0), StepUnit::Day, -1);
        assert_eq!((d.year, d.month, d.day), (2021, 2, 28));
    }

    #[test]
    fn time_steps_carry_into_date() {
        // 23:59:59 +1秒 → 次日 00:00:00
        let d = step(&dt(2020, 6, 15, 23, 59, 59), StepUnit::Second, 1);
        assert_eq!(
            (d.year, d.month, d.day, d.hour, d.minute, d.second),
            (2020, 6, 16, 0, 0, 0)
        );
        // 00:00:00 -1分 → 前一日 23:59:00
        let d = step(&dt(2020, 6, 15, 0, 0, 0), StepUnit::Minute, -1);
        assert_eq!(
            (d.year, d.month, d.day, d.hour, d.minute, d.second),
            (2020, 6, 14, 23, 59, 0)
        );
        // 23:30 +1时 → 次日 00:30
        let d = step(&dt(2020, 1, 31, 23, 30, 0), StepUnit::Hour, 1);
        assert_eq!(
            (d.year, d.month, d.day, d.hour, d.minute),
            (2020, 2, 1, 0, 30)
        );
    }

    #[test]
    fn tz_and_st_are_preserved() {
        let mut base = dt(2000, 1, 1, 0, 0, 0);
        base.tz = -5.5;
        base.st = true;
        let d = step(&base, StepUnit::Day, 3);
        assert_eq!(d.tz, -5.5);
        assert!(d.st);
    }
}
