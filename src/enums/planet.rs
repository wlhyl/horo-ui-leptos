use serde::{Deserialize, Serialize};

/// 行星速度状态：快、平均、慢（后台按中文变体名序列化）。
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum PlanetSpeedState {
    快,
    均,
    慢,
}

/// 行星 / 四轴 / 福点名称。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum PlanetName {
    ASC,
    MC,
    DSC,
    IC,
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    NorthNode,
    SouthNode,
    PartOfFortune,
}

/// 七传统行星：日月水金火木土（先天力量、接纳等传统占星判定范围）。
pub(crate) const TRADITIONAL_PLANETS: [PlanetName; 7] = [
    PlanetName::Sun,
    PlanetName::Moon,
    PlanetName::Mercury,
    PlanetName::Venus,
    PlanetName::Mars,
    PlanetName::Jupiter,
    PlanetName::Saturn,
];

impl PlanetName {
    /// 是否为上级行星（轨道在地球之外，含火星）。
    pub(crate) fn is_superior(self) -> bool {
        matches!(
            self,
            PlanetName::Mars | PlanetName::Jupiter | PlanetName::Saturn
        )
    }

    /// 是否为下级行星（轨道在地球之内：金/水）。
    pub(crate) fn is_inferior(self) -> bool {
        matches!(self, PlanetName::Venus | PlanetName::Mercury)
    }

    /// 是否为吉星（金/木）。
    pub(crate) fn is_benefic(self) -> bool {
        matches!(self, PlanetName::Venus | PlanetName::Jupiter)
    }

    /// 是否为凶星（火/土）。
    pub(crate) fn is_malefic(self) -> bool {
        matches!(self, PlanetName::Mars | PlanetName::Saturn)
    }

    /// 是否属于七传统行星。
    pub(crate) fn is_traditional(self) -> bool {
        TRADITIONAL_PLANETS.contains(&self)
    }
}

/// 用户面向的展示名（供错误文案与界面使用），实现后自动获得 `to_string()`。
impl std::fmt::Display for PlanetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PlanetName::ASC => "ASC",
            PlanetName::MC => "MC",
            PlanetName::DSC => "DSC",
            PlanetName::IC => "IC",
            PlanetName::Sun => "Sun",
            PlanetName::Moon => "Moon",
            PlanetName::Mercury => "Mercury",
            PlanetName::Venus => "Venus",
            PlanetName::Mars => "Mars",
            PlanetName::Jupiter => "Jupiter",
            PlanetName::Saturn => "Saturn",
            PlanetName::NorthNode => "NorthNode",
            PlanetName::SouthNode => "SouthNode",
            PlanetName::PartOfFortune => "PartOfFortune",
        };
        f.write_str(s)
    }
}
