use serde::{Deserialize, Serialize};

/// 宫位系统。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum HouseName {
    Alcabitus,
    Placidus,
    Regiomontanus,
    WholeSign,
}

impl HouseName {
    pub(crate) const ALL: [HouseName; 4] = [
        HouseName::Alcabitus,
        HouseName::Placidus,
        HouseName::Regiomontanus,
        HouseName::WholeSign,
    ];

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            HouseName::Alcabitus => "Alcabitus",
            HouseName::Placidus => "Placidus",
            HouseName::Regiomontanus => "Regiomontanus",
            HouseName::WholeSign => "WholeSign",
        }
    }

    /// 由 as_str 的逆映射解析；未匹配返回 None。
    pub(crate) fn from_str(s: &str) -> Option<HouseName> {
        Self::ALL.iter().find(|h| h.as_str() == s).copied()
    }
}
