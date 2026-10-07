use serde::{Deserialize, Serialize};

/// 弧转日期换算方式（对应原版 ArcToDateMethod）。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ArcToDateMethod {
    /// 1天=1年
    DayPerYear,
    /// 1度=1年
    DegreePerYear,
}

/// 用户面向的展示名（下拉框选项），对齐原版 nameMap。
impl std::fmt::Display for ArcToDateMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ArcToDateMethod::DayPerYear => "1天=1年",
            ArcToDateMethod::DegreePerYear => "1度=1年",
        };
        f.write_str(s)
    }
}
