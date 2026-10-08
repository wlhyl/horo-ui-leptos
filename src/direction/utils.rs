//! 方向推运工具函数（对应原版 direction-utils + promittor 工具）。
use leptos::prelude::*;

use crate::api::response::{Promittor, Significator};
use crate::components::{SelectGroup, SelectPopup};
use crate::astro::horo_math::{degree_to_dms, zodiac_long};
use crate::enums::{
    arc_to_date_method::ArcToDateMethod, daily_direction_method::DailyDirectionMethod,
    direction_method::DirectionMethod, planet::PlanetName, process_name::ProcessName,
    zodiac::Zodiac,
};
use crate::models::datetime::{DateTimeData, StepUnit};

stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);

// 分组表 → 弹层选项组（转发 components::grouped_items，供 method_select 组装）
pub(crate) use crate::components::grouped_items;

/// 主限算法候选项（下拉遍历用）。
pub(crate) const DIRECTION_METHODS: [DirectionMethod; 3] = [
    DirectionMethod::SemiArc,
    DirectionMethod::UnderPole,
    DirectionMethod::ZodiacalUnderPole,
];

/// 换算方式候选项。
pub(crate) const ARC_TO_DATE_METHODS: [ArcToDateMethod; 2] =
    [ArcToDateMethod::DayPerYear, ArcToDateMethod::DegreePerYear];

/// 方向弧算法候选项。
pub(crate) const DAILY_DIRECTION_METHODS: [DailyDirectionMethod; 2] = [
    DailyDirectionMethod::SolarArc,
    DailyDirectionMethod::SemiArcZodiacal,
];

/// 算法类下拉框（枚举以 Display 中文名作选项文案；选项为静态常量表）。
pub(crate) fn method_select<M>(
    label: &'static str,
    value: RwSignal<M>,
    options: &[M],
) -> impl IntoView
where
    M: Copy + PartialEq + std::fmt::Display + Send + Sync + 'static,
{
    let options = options.to_vec();
    view! {
        <div class=form::field>
            <label>{label}</label>
            <div class=form::control>
                <SelectPopup
                    current=move || Some(value.get())
                    on_pick=move |m| value.set(m)
                    groups=move || vec![SelectGroup { label: "", items: options.clone() }]
                    label=|m| m.to_string()
                />
            </div>
        </div>
    }
}

/// 结果页可作筛选与展示的全体显著星（对应原版 ALL_SIGNIFICATORS）。
pub(crate) const ALL_SIGNIFICATORS: [PlanetName; 14] = [
    PlanetName::ASC,
    PlanetName::MC,
    PlanetName::DSC,
    PlanetName::IC,
    PlanetName::Sun,
    PlanetName::Moon,
    PlanetName::Mercury,
    PlanetName::Venus,
    PlanetName::Mars,
    PlanetName::Jupiter,
    PlanetName::Saturn,
    PlanetName::NorthNode,
    PlanetName::SouthNode,
    PlanetName::PartOfFortune,
];

/// 推运类型下拉的全体可选项：方向推运 + 返照盘 + 比较盘（对应原版 processOptions 子集）。
pub(crate) const PROCESS_OPTIONS: [ProcessName; 14] = [
    ProcessName::Direction,
    ProcessName::DailyDirection,
    ProcessName::SolarArc,
    ProcessName::SolarReturn,
    ProcessName::LunarReturn,
    ProcessName::DailyReturn,
    ProcessName::Transit,
    ProcessName::SolarcomparNative,
    ProcessName::NativecomparSolar,
    ProcessName::LunarcomparNative,
    ProcessName::NativecomparLunar,
    ProcessName::DailycomparNative,
    ProcessName::NativecomparDaily,
    ProcessName::SecondaryProgressionComparNative,
];

/// 推运类型的分组显示：方向推运 / 返照盘 / 比较盘三组（与工作台「添加星盘」
/// 按钮组的分隔线分组对应，原版 input-panel 的 推运 / 返照 分组）。
pub(crate) const PROCESS_GROUPS: [(&str, &[ProcessName]); 3] = [
    (
        "方向推运",
        &[
            ProcessName::Direction,
            ProcessName::DailyDirection,
            ProcessName::SolarArc,
        ],
    ),
    (
        "返照盘",
        &[
            ProcessName::SolarReturn,
            ProcessName::LunarReturn,
            ProcessName::DailyReturn,
        ],
    ),
    (
        "比较盘",
        &[
            ProcessName::Transit,
            ProcessName::SolarcomparNative,
            ProcessName::NativecomparSolar,
            ProcessName::LunarcomparNative,
            ProcessName::NativecomparLunar,
            ProcessName::DailycomparNative,
            ProcessName::NativecomparDaily,
            ProcessName::SecondaryProgressionComparNative,
        ],
    ),
];

/// 推运模式的标题（对应原版 titleForMode / ProcessName.name）。
pub(crate) fn process_title(mode: ProcessName) -> &'static str {
    match mode {
        ProcessName::Direction => "主向推运",
        ProcessName::DailyDirection => "每日回归方向弧",
        ProcessName::SolarArc => "太阳弧",
        ProcessName::SolarReturn => "日返",
        ProcessName::LunarReturn => "月返",
        ProcessName::DailyReturn => "每日回归",
        ProcessName::Transit => "行运",
        ProcessName::SolarcomparNative => "日返比本命",
        ProcessName::NativecomparSolar => "本命比日返",
        ProcessName::LunarcomparNative => "月返比本命",
        ProcessName::NativecomparLunar => "本命比月返",
        ProcessName::DailycomparNative => "每日回归比本命",
        ProcessName::NativecomparDaily => "本命比每日回归",
        ProcessName::SecondaryProgressionComparNative => "次限比本命",
        _ => "推运",
    }
}

/// 承诺星形态（对应原版 PromittorType）：相位类型筛选的候选项，
/// 同时驱动 promittor 单元格的中文说明。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PromittorType {
    Conjunction,
    SinisterTrine,
    DexterTrine,
    SinisterSextile,
    DexterSextile,
    SinisterSquare,
    DexterSquare,
    Opposition,
    Term,
    Antiscoins,
    Contraantiscias,
    Cusp,
    Sign,
}

pub(crate) const ALL_PROMITTOR_TYPES: [PromittorType; 13] = [
    PromittorType::Conjunction,
    PromittorType::SinisterTrine,
    PromittorType::DexterTrine,
    PromittorType::SinisterSextile,
    PromittorType::DexterSextile,
    PromittorType::SinisterSquare,
    PromittorType::DexterSquare,
    PromittorType::Opposition,
    PromittorType::Term,
    PromittorType::Antiscoins,
    PromittorType::Contraantiscias,
    PromittorType::Cusp,
    PromittorType::Sign,
];

/// 用户面向的展示名（筛选 chips），对齐原版 PromittorType.nameMap。
impl std::fmt::Display for PromittorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PromittorType::Conjunction => "合相",
            PromittorType::SinisterTrine => "左三合",
            PromittorType::DexterTrine => "右三合",
            PromittorType::SinisterSextile => "左六合",
            PromittorType::DexterSextile => "右六合",
            PromittorType::SinisterSquare => "左刑",
            PromittorType::DexterSquare => "右刑",
            PromittorType::Opposition => "对冲",
            PromittorType::Term => "界",
            PromittorType::Antiscoins => "映点",
            PromittorType::Contraantiscias => "反映点",
            PromittorType::Cusp => "宫头",
            PromittorType::Sign => "星座",
        };
        f.write_str(s)
    }
}

/// 一条 promittor 的形态（一条记录只属于一种）。
pub(crate) fn promittor_type(p: &Promittor) -> PromittorType {
    match p {
        Promittor::Conjunction { .. } => PromittorType::Conjunction,
        Promittor::SinisterTrine { .. } => PromittorType::SinisterTrine,
        Promittor::DexterTrine { .. } => PromittorType::DexterTrine,
        Promittor::SinisterSextile { .. } => PromittorType::SinisterSextile,
        Promittor::DexterSextile { .. } => PromittorType::DexterSextile,
        Promittor::SinisterSquare { .. } => PromittorType::SinisterSquare,
        Promittor::DexterSquare { .. } => PromittorType::DexterSquare,
        Promittor::Opposition { .. } => PromittorType::Opposition,
        Promittor::Term { .. } => PromittorType::Term,
        Promittor::Antiscoins { .. } => PromittorType::Antiscoins,
        Promittor::Contraantiscias { .. } => PromittorType::Contraantiscias,
        Promittor::Cusp { .. } => PromittorType::Cusp,
        Promittor::Sign { .. } => PromittorType::Sign,
    }
}

/// 承诺星涉及的行星（界取界主星；宫头 / 星座形态无行星）。
pub(crate) fn promittor_planet(p: &Promittor) -> Option<PlanetName> {
    match p {
        Promittor::Conjunction { conjunction } => Some(*conjunction),
        Promittor::SinisterTrine { sinister_trine } => Some(*sinister_trine),
        Promittor::DexterTrine { dexter_trine } => Some(*dexter_trine),
        Promittor::SinisterSextile { sinister_sextile } => Some(*sinister_sextile),
        Promittor::DexterSextile { dexter_sextile } => Some(*dexter_sextile),
        Promittor::SinisterSquare { sinister_square } => Some(*sinister_square),
        Promittor::DexterSquare { dexter_square } => Some(*dexter_square),
        Promittor::Opposition { opposition } => Some(*opposition),
        Promittor::Term { term } => Some(term.0),
        Promittor::Antiscoins { antiscoins } => Some(*antiscoins),
        Promittor::Contraantiscias { contraantiscias } => Some(*contraantiscias),
        Promittor::Cusp { .. } | Promittor::Sign { .. } => None,
    }
}

/// 相位信息 (角度, 是否左相位)；映点 / 反映点按合相处理（对齐原版）。
pub(crate) fn promittor_aspect(p: &Promittor) -> Option<(u8, bool)> {
    let (deg, is_left) = match p {
        Promittor::Conjunction { .. } => (0, true),
        Promittor::SinisterTrine { .. } => (120, true),
        Promittor::DexterTrine { .. } => (120, false),
        Promittor::SinisterSextile { .. } => (60, true),
        Promittor::DexterSextile { .. } => (60, false),
        Promittor::SinisterSquare { .. } => (90, true),
        Promittor::DexterSquare { .. } => (90, false),
        Promittor::Opposition { .. } => (180, true),
        Promittor::Antiscoins { .. } | Promittor::Contraantiscias { .. } => (0, true),
        _ => return None,
    };
    Some((deg, is_left))
}

/// 映点 / 反映点标记（"Ant" / "C-Ant"）。
pub(crate) fn promittor_antiscia(p: &Promittor) -> Option<&'static str> {
    match p {
        Promittor::Antiscoins { .. } => Some("Ant"),
        Promittor::Contraantiscias { .. } => Some("C-Ant"),
        _ => None,
    }
}

/// 宫头形态的宫号（1-12）。
pub(crate) fn promittor_cusp(p: &Promittor) -> Option<u8> {
    match p {
        Promittor::Cusp { cusp } => Some(*cusp),
        _ => None,
    }
}

/// 星座形态的黄道序号（0-11）。
pub(crate) fn promittor_sign(p: &Promittor) -> Option<u8> {
    match p {
        Promittor::Sign { sign } => Some(*sign),
        _ => None,
    }
}

/// 界形态的展示信息：(星座, 界内度数 DMS 文案)。
pub(crate) fn term_info(p: &Promittor) -> Option<(Zodiac, String)> {
    let Promittor::Term { term } = p else {
        return None;
    };
    let pos = zodiac_long(term.1);
    let (d, m, s) = degree_to_dms(pos.degree);
    let dms = if m == 0 && s == 0 {
        format!("{d}°")
    } else {
        format!("{d}°{m:02}′{s:02}″")
    };
    Some((pos.zodiac, dms))
}

/// 显著星为行星 / 四轴福点时取出行星名。
pub(crate) fn significator_planet(sig: &Significator) -> Option<PlanetName> {
    match sig {
        Significator::Planet { planet } => Some(*planet),
        Significator::Cusp { .. } => None,
    }
}

/// 显著星为宫头时取出宫号（1-12）。
pub(crate) fn significator_cusp(sig: &Significator) -> Option<u8> {
    match sig {
        Significator::Cusp { cusp } => Some(*cusp),
        Significator::Planet { .. } => None,
    }
}

// ---------------------------------------------------------------------------
// 展示格式化
// ---------------------------------------------------------------------------

/// 日期时间 → "YYYY-MM-DD HH:mm:ss"。
pub(crate) fn format_date(d: &DateTimeData) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        d.year, d.month, d.day, d.hour, d.minute, d.second
    )
}

/// 方向弧 → "±D°MM′SS″"（正负号整体前置，对齐原版 formatArc）。
pub(crate) fn format_arc(arc: f64) -> String {
    let (d, m, s) = degree_to_dms(arc);
    let sign = if arc >= 0.0 { "" } else { "-" };
    format!("{sign}{}°{:02}′{:02}″", d.abs(), m.abs(), s.abs())
}

/// 出生时间加 N 年（重置筛选的结束日期用；闰日按进位归一化）。
pub(crate) fn add_years(date: DateTimeData, years: i32) -> DateTimeData {
    date.stepped(StepUnit::Year, years)
}

/// 当前本地时间减 N 年（开始日期默认值；时区 / 夏令时取出生时间，对齐原版）。
pub(crate) fn now_minus_years(years: i32, native: &DateTimeData) -> DateTimeData {
    let mut now = DateTimeData::now();
    now.tz = native.tz;
    now.stepped(StepUnit::Year, -years)
}

// ---------------------------------------------------------------------------
// 过滤谓词（对应原版 direction-utils 的 check* 系列）
// ---------------------------------------------------------------------------

/// 日期范围过滤：各时间字段逐元组比较（等价原版 dateToNumber 的本地时间戳比较，
/// 同样忽略时区字段）。
pub(crate) fn check_date_range(
    date: &DateTimeData,
    start: &DateTimeData,
    end: &DateTimeData,
) -> bool {
    let key = |d: &DateTimeData| (d.year, d.month, d.day, d.hour, d.minute, d.second);
    key(date) >= key(start) && key(date) <= key(end)
}

/// 显著星过滤：两者全空表示不过滤；只勾行星时宫头型显著星被滤除（对齐原版）。
pub(crate) fn check_significator(sig: &Significator, planets: &[PlanetName]) -> bool {
    if planets.is_empty() {
        return true;
    }
    match sig {
        Significator::Planet { planet } => planets.contains(planet),
        Significator::Cusp { .. } => false,
    }
}

/// 相位类型过滤：空列表表示全部。
pub(crate) fn check_promittor_type(p: &Promittor, filter: &[PromittorType]) -> bool {
    filter.is_empty() || filter.contains(&promittor_type(p))
}

/// 承诺星行星过滤：空列表表示全部；无行星形态的记录在有筛选时被滤除。
pub(crate) fn check_promittor_planet(p: &Promittor, selected: &[PlanetName]) -> bool {
    if selected.is_empty() {
        return true;
    }
    promittor_planet(p).is_some_and(|planet| selected.contains(&planet))
}

/// 弧方向过滤（对应原版 checkArcDirection）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArcDirectionFilter {
    All,
    /// 正向弧（arc >= 0）
    Direct,
    /// 反向弧（arc < 0）
    Converse,
}

impl ArcDirectionFilter {
    pub(crate) fn check(self, arc: f64) -> bool {
        match self {
            ArcDirectionFilter::All => true,
            ArcDirectionFilter::Direct => arc >= 0.0,
            ArcDirectionFilter::Converse => arc < 0.0,
        }
    }
}
