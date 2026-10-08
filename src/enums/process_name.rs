use serde::{Deserialize, Serialize};

/// 推运种类（对应原版 ProcessName）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub(crate) enum ProcessName {
    Profection,
    MedievalProfection,
    CustomMonthProfection,
    CustomDayProfection,
    CustomLunarDayProfection,
    Transit,
    Firdaria,
    SolarReturn,
    LunarReturn,
    DailyReturn,
    SolarcomparNative,
    NativecomparSolar,
    LunarcomparNative,
    NativecomparLunar,
    DailycomparNative,
    NativecomparDaily,
    Direction,
    DailyDirection,
    SolarArc,
    QuadrantProcess,
    SecondaryProgression,
    SecondaryProgressionComparNative,
}
