use serde::{Deserialize, Serialize};

/// 次限推运换算方式（对应原版 SecondaryProgressionMethod）。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SecondaryProgressionMethod {
    /// 1天=1年
    DayPerYear,
    /// 1度=1年
    DegreePerYear,
}
