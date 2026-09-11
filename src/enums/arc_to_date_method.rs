use serde::{Deserialize, Serialize};

/// 弧转日期换算方式（对应原版 ArcToDateMethod）。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ArcToDateMethod {
    /// 1天=1年
    DayPerYear,
    /// 1度=1年
    DegreePerYear,
}
