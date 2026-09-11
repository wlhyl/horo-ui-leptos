use serde::{Deserialize, Serialize};

/// 每日回归方向弧算法（对应原版 DailyDirectionMethod）。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DailyDirectionMethod {
    /// 太阳弧风格：弧 = 承诺星黄经 − 显著星黄经
    SolarArc,
    /// 黄道向运风格（主向推运 SemiArc 算法）
    SemiArcZodiacal,
}
