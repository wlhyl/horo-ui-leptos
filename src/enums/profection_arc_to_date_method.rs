use serde::{Deserialize, Serialize};

/// 中世纪小限弧转日期换算方式（对应原版 ProfectionArcToDateMethod）。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ProfectionArcToDateMethod {
    /// 真太阳弧
    TrueSolarArc,
    /// 平太阳弧
    MeanSolarArc,
}
