use serde::{Deserialize, Serialize};

/// 每日回归方向弧算法（对应原版 DailyDirectionMethod）。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DailyDirectionMethod {
    /// 太阳弧风格：弧 = 承诺星黄经 − 显著星黄经
    SolarArc,
    /// 黄道向运风格（主向推运 SemiArc 算法）
    SemiArcZodiacal,
}

/// 用户面向的展示名（下拉框选项），对齐原版 nameMap。
impl std::fmt::Display for DailyDirectionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DailyDirectionMethod::SolarArc => "太阳弧风格",
            DailyDirectionMethod::SemiArcZodiacal => "黄道向运风格",
        };
        f.write_str(s)
    }
}
