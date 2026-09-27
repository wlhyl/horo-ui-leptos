//! 黄道尊贵（dignity）主星表：庙 / 旺 / 三分 / 界 / 面 / 陷 / 弱。
//! 移植自 horo-ui 的 `utils/image/zodiac.ts` 与 `utils/reception/reception.ts`。

use crate::enums::planet::PlanetName;
use crate::enums::zodiac::Zodiac;

/// 尊贵种类。
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum DignityKind {
    Rulership,
    Exaltation,
    Triplicity,
    Term,
    Face,
}

impl DignityKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            DignityKind::Rulership => "庙",
            DignityKind::Exaltation => "旺",
            DignityKind::Triplicity => "三分",
            DignityKind::Term => "界",
            DignityKind::Face => "面",
        }
    }
}

/// 星座守护星（庙）。
pub(crate) fn rulership(z: Zodiac) -> PlanetName {
    match z {
        Zodiac::Aries | Zodiac::Scorpio => PlanetName::Mars,
        Zodiac::Taurus | Zodiac::Libra => PlanetName::Venus,
        Zodiac::Gemini | Zodiac::Virgo => PlanetName::Mercury,
        Zodiac::Cancer => PlanetName::Moon,
        Zodiac::Leo => PlanetName::Sun,
        Zodiac::Sagittarius | Zodiac::Pisces => PlanetName::Jupiter,
        Zodiac::Capricorn | Zodiac::Aquarius => PlanetName::Saturn,
    }
}

/// 星座擢升星（旺），无则为 None。
pub(crate) fn exaltation(z: Zodiac) -> Option<PlanetName> {
    match z {
        Zodiac::Aries => Some(PlanetName::Sun),
        Zodiac::Taurus => Some(PlanetName::Moon),
        Zodiac::Gemini => Some(PlanetName::NorthNode),
        Zodiac::Cancer => Some(PlanetName::Jupiter),
        Zodiac::Leo => None,
        Zodiac::Virgo => Some(PlanetName::Mercury),
        Zodiac::Libra => Some(PlanetName::Saturn),
        Zodiac::Scorpio => None,
        Zodiac::Sagittarius => Some(PlanetName::SouthNode),
        Zodiac::Capricorn => Some(PlanetName::Mars),
        Zodiac::Aquarius => None,
        Zodiac::Pisces => Some(PlanetName::Venus),
    }
}

/// 星座失势星（弱）。
pub(crate) fn detriment(z: Zodiac) -> PlanetName {
    match z {
        Zodiac::Aries | Zodiac::Scorpio => PlanetName::Venus,
        Zodiac::Taurus | Zodiac::Libra => PlanetName::Mars,
        Zodiac::Gemini | Zodiac::Virgo => PlanetName::Jupiter,
        Zodiac::Cancer | Zodiac::Leo => PlanetName::Saturn,
        Zodiac::Sagittarius | Zodiac::Pisces => PlanetName::Mercury,
        Zodiac::Capricorn => PlanetName::Moon,
        Zodiac::Aquarius => PlanetName::Sun,
    }
}

/// 星座陷落星（陷），无则为 None。
pub(crate) fn fall(z: Zodiac) -> Option<PlanetName> {
    match z {
        Zodiac::Aries => Some(PlanetName::Saturn),
        Zodiac::Taurus => None,
        Zodiac::Gemini => None,
        Zodiac::Cancer => Some(PlanetName::Mars),
        Zodiac::Leo => None,
        Zodiac::Virgo => Some(PlanetName::Venus),
        Zodiac::Libra => Some(PlanetName::Sun),
        Zodiac::Scorpio => Some(PlanetName::Moon),
        Zodiac::Sagittarius => None,
        Zodiac::Capricorn => Some(PlanetName::Jupiter),
        Zodiac::Aquarius => None,
        Zodiac::Pisces => Some(PlanetName::Mercury),
    }
}

/// Lily 版三分性主星（行星力量计算使用此版本）。
pub(crate) fn triplicity_of_lily(z: Zodiac) -> [PlanetName; 2] {
    match z {
        Zodiac::Aries | Zodiac::Leo | Zodiac::Sagittarius => [PlanetName::Sun, PlanetName::Jupiter],
        Zodiac::Capricorn | Zodiac::Taurus | Zodiac::Virgo => [PlanetName::Venus, PlanetName::Moon],
        Zodiac::Libra | Zodiac::Aquarius | Zodiac::Gemini => {
            [PlanetName::Saturn, PlanetName::Mercury]
        }
        Zodiac::Cancer | Zodiac::Scorpio | Zodiac::Pisces => [PlanetName::Mars, PlanetName::Mars],
    }
}

/// 标准（Dorothean）三分性主星：每元素三主星，火/土/风/水各一组。
/// 区别于 [`triplicity_of_lily`]（仅两主星），此为「本命所用」力量表使用的版本。
pub(crate) fn triplicity(z: Zodiac) -> [PlanetName; 3] {
    match z {
        Zodiac::Aries | Zodiac::Leo | Zodiac::Sagittarius => {
            [PlanetName::Sun, PlanetName::Jupiter, PlanetName::Saturn]
        }
        Zodiac::Capricorn | Zodiac::Taurus | Zodiac::Virgo => {
            [PlanetName::Venus, PlanetName::Moon, PlanetName::Mars]
        }
        Zodiac::Libra | Zodiac::Aquarius | Zodiac::Gemini => {
            [PlanetName::Saturn, PlanetName::Mercury, PlanetName::Jupiter]
        }
        Zodiac::Cancer | Zodiac::Scorpio | Zodiac::Pisces => {
            [PlanetName::Venus, PlanetName::Mars, PlanetName::Moon]
        }
    }
}

/// 星座的三个十度面主星（按 0-10°/10-20°/20-30°）。
pub(crate) fn face_lords(z: Zodiac) -> [PlanetName; 3] {
    match z {
        Zodiac::Aries => [PlanetName::Mars, PlanetName::Sun, PlanetName::Venus],
        Zodiac::Taurus => [PlanetName::Mercury, PlanetName::Moon, PlanetName::Saturn],
        Zodiac::Gemini => [PlanetName::Jupiter, PlanetName::Mars, PlanetName::Sun],
        Zodiac::Cancer => [PlanetName::Venus, PlanetName::Mercury, PlanetName::Moon],
        Zodiac::Leo => [PlanetName::Saturn, PlanetName::Jupiter, PlanetName::Mars],
        Zodiac::Virgo => [PlanetName::Sun, PlanetName::Venus, PlanetName::Mercury],
        Zodiac::Libra => [PlanetName::Moon, PlanetName::Saturn, PlanetName::Jupiter],
        Zodiac::Scorpio => [PlanetName::Mars, PlanetName::Sun, PlanetName::Venus],
        Zodiac::Sagittarius => [PlanetName::Mercury, PlanetName::Moon, PlanetName::Saturn],
        Zodiac::Capricorn => [PlanetName::Jupiter, PlanetName::Mars, PlanetName::Sun],
        Zodiac::Aquarius => [PlanetName::Venus, PlanetName::Mercury, PlanetName::Moon],
        Zodiac::Pisces => [PlanetName::Saturn, PlanetName::Jupiter, PlanetName::Mars],
    }
}

/// 星座的托勒密界：五段 (主星, 区间上界度数)。
pub(crate) fn ptolemy_terms(z: Zodiac) -> [(PlanetName, u8); 5] {
    match z {
        Zodiac::Aries => [
            (PlanetName::Jupiter, 6),
            (PlanetName::Venus, 14),
            (PlanetName::Mercury, 21),
            (PlanetName::Mars, 26),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Taurus => [
            (PlanetName::Venus, 8),
            (PlanetName::Mercury, 15),
            (PlanetName::Jupiter, 22),
            (PlanetName::Saturn, 26),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Gemini => [
            (PlanetName::Mercury, 7),
            (PlanetName::Jupiter, 14),
            (PlanetName::Venus, 21),
            (PlanetName::Saturn, 25),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Cancer => [
            (PlanetName::Mars, 6),
            (PlanetName::Jupiter, 13),
            (PlanetName::Mercury, 20),
            (PlanetName::Venus, 27),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Leo => [
            (PlanetName::Saturn, 6),
            (PlanetName::Mercury, 13),
            (PlanetName::Venus, 19),
            (PlanetName::Jupiter, 25),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Virgo => [
            (PlanetName::Mercury, 7),
            (PlanetName::Venus, 13),
            (PlanetName::Jupiter, 18),
            (PlanetName::Saturn, 24),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Libra => [
            (PlanetName::Saturn, 6),
            (PlanetName::Venus, 11),
            (PlanetName::Jupiter, 19),
            (PlanetName::Mercury, 24),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Scorpio => [
            (PlanetName::Mars, 6),
            (PlanetName::Jupiter, 14),
            (PlanetName::Venus, 21),
            (PlanetName::Mercury, 27),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Sagittarius => [
            (PlanetName::Jupiter, 8),
            (PlanetName::Venus, 14),
            (PlanetName::Mercury, 20),
            (PlanetName::Saturn, 25),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Capricorn => [
            (PlanetName::Venus, 6),
            (PlanetName::Mercury, 12),
            (PlanetName::Jupiter, 19),
            (PlanetName::Mars, 25),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Aquarius => [
            (PlanetName::Saturn, 6),
            (PlanetName::Mercury, 12),
            (PlanetName::Venus, 20),
            (PlanetName::Jupiter, 25),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Pisces => [
            (PlanetName::Venus, 8),
            (PlanetName::Jupiter, 14),
            (PlanetName::Mercury, 20),
            (PlanetName::Mars, 26),
            (PlanetName::Saturn, 30),
        ],
    }
}

/// 星座的埃及界：五段 (主星, 区间上界度数)。用于「本命所用」力量表。
pub(crate) fn egyptian_terms(z: Zodiac) -> [(PlanetName, u8); 5] {
    match z {
        Zodiac::Aries => [
            (PlanetName::Jupiter, 6),
            (PlanetName::Venus, 12),
            (PlanetName::Mercury, 20),
            (PlanetName::Mars, 25),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Taurus => [
            (PlanetName::Venus, 8),
            (PlanetName::Mercury, 14),
            (PlanetName::Jupiter, 22),
            (PlanetName::Saturn, 27),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Gemini => [
            (PlanetName::Mercury, 6),
            (PlanetName::Jupiter, 12),
            (PlanetName::Venus, 17),
            (PlanetName::Mars, 24),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Cancer => [
            (PlanetName::Mars, 7),
            (PlanetName::Venus, 13),
            (PlanetName::Mercury, 19),
            (PlanetName::Jupiter, 26),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Leo => [
            (PlanetName::Jupiter, 6),
            (PlanetName::Venus, 11),
            (PlanetName::Saturn, 18),
            (PlanetName::Mercury, 24),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Virgo => [
            (PlanetName::Mercury, 7),
            (PlanetName::Venus, 17),
            (PlanetName::Jupiter, 21),
            (PlanetName::Mars, 28),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Libra => [
            (PlanetName::Saturn, 6),
            (PlanetName::Mercury, 14),
            (PlanetName::Jupiter, 21),
            (PlanetName::Venus, 28),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Scorpio => [
            (PlanetName::Mars, 7),
            (PlanetName::Venus, 11),
            (PlanetName::Mercury, 19),
            (PlanetName::Jupiter, 24),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Sagittarius => [
            (PlanetName::Jupiter, 12),
            (PlanetName::Venus, 17),
            (PlanetName::Mercury, 21),
            (PlanetName::Saturn, 26),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Capricorn => [
            (PlanetName::Mercury, 7),
            (PlanetName::Jupiter, 14),
            (PlanetName::Venus, 22),
            (PlanetName::Saturn, 26),
            (PlanetName::Mars, 30),
        ],
        Zodiac::Aquarius => [
            (PlanetName::Mercury, 7),
            (PlanetName::Venus, 13),
            (PlanetName::Jupiter, 20),
            (PlanetName::Mars, 25),
            (PlanetName::Saturn, 30),
        ],
        Zodiac::Pisces => [
            (PlanetName::Venus, 12),
            (PlanetName::Jupiter, 16),
            (PlanetName::Mercury, 19),
            (PlanetName::Mars, 28),
            (PlanetName::Saturn, 30),
        ],
    }
}

/// 某黄道位置的全部尊贵主星。
pub(crate) struct DignityLords {
    pub rulership: PlanetName,
    pub exaltation: Option<PlanetName>,
    pub triplicity: [PlanetName; 2],
    pub term: PlanetName,
    pub face: PlanetName,
}

/// 给定星座内某度数的全部尊贵主星（界用托勒密界，三分用 Lily 版）。
pub(crate) fn get_dignity_lords_at(zodiac: Zodiac, degree: f64) -> DignityLords {
    let terms = ptolemy_terms(zodiac);
    let mut term = terms[terms.len() - 1].0;
    for i in 0..terms.len() {
        let start = if i == 0 {
            0.0
        } else {
            f64::from(terms[i - 1].1)
        };
        let end = f64::from(terms[i].1);
        if degree >= start && degree < end {
            term = terms[i].0;
            break;
        }
    }

    DignityLords {
        rulership: rulership(zodiac),
        exaltation: exaltation(zodiac),
        triplicity: triplicity_of_lily(zodiac),
        term,
        face: face_lords(zodiac)[(degree / 10.0).floor() as usize % 3],
    }
}

/// 查询指定星体在给定黄道位置上拥有的所有尊贵种类（接纳 / 互容判定的基础）。
pub(crate) fn get_dignities_of(
    planet: PlanetName,
    zodiac: Zodiac,
    degree: f64,
) -> Vec<DignityKind> {
    let lords = get_dignity_lords_at(zodiac, degree);
    let mut result = Vec::new();
    if lords.rulership == planet {
        result.push(DignityKind::Rulership);
    }
    if lords.exaltation == Some(planet) {
        result.push(DignityKind::Exaltation);
    }
    if lords.triplicity.contains(&planet) {
        result.push(DignityKind::Triplicity);
    }
    if lords.term == planet {
        result.push(DignityKind::Term);
    }
    if lords.face == planet {
        result.push(DignityKind::Face);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        egyptian_terms, exaltation, fall, get_dignities_of, get_dignity_lords_at, rulership,
        triplicity, DignityKind,
    };
    use crate::enums::planet::PlanetName;
    use crate::enums::zodiac::Zodiac;

    #[test]
    fn dignity_lords_at_known_positions() {
        // 白羊 0-6° 界主木星，0-10° 面主火星
        let lords = get_dignity_lords_at(Zodiac::Aries, 3.0);
        assert_eq!(lords.rulership, PlanetName::Mars);
        assert_eq!(lords.exaltation, Some(PlanetName::Sun));
        assert_eq!(lords.term, PlanetName::Jupiter);
        assert_eq!(lords.face, PlanetName::Mars);
        // 界边界：6° 起为金星界
        assert_eq!(
            get_dignity_lords_at(Zodiac::Aries, 6.0).term,
            PlanetName::Venus
        );
        // 面边界：10° 起为太阳面
        assert_eq!(
            get_dignity_lords_at(Zodiac::Aries, 10.0).face,
            PlanetName::Sun
        );
        // 末段兜底：29.9° 仍取最后一段土星界
        assert_eq!(
            get_dignity_lords_at(Zodiac::Aries, 29.9).term,
            PlanetName::Saturn
        );
    }

    #[test]
    fn dignity_table_spot_checks() {
        assert_eq!(rulership(Zodiac::Cancer), PlanetName::Moon);
        assert_eq!(rulership(Zodiac::Aquarius), PlanetName::Saturn);
        assert_eq!(exaltation(Zodiac::Capricorn), Some(PlanetName::Mars));
        assert_eq!(exaltation(Zodiac::Leo), None);
        assert_eq!(fall(Zodiac::Libra), Some(PlanetName::Sun));
        assert_eq!(fall(Zodiac::Taurus), None);
    }

    #[test]
    fn get_dignities_of_collects_kinds() {
        // 太阳在白羊 3°：旺 + 三分（太阳为白羊的 Lily 三分主星）
        let ds = get_dignities_of(PlanetName::Sun, Zodiac::Aries, 3.0);
        assert_eq!(ds, vec![DignityKind::Exaltation, DignityKind::Triplicity]);
        // 火星在白羊 3°：庙 + 面（火星为白羊 0-10° 面主星），界主为木星
        let ds = get_dignities_of(PlanetName::Mars, Zodiac::Aries, 3.0);
        assert_eq!(ds, vec![DignityKind::Rulership, DignityKind::Face]);
        // 金星在金牛 5°：庙 + 三分 + 界（金牛面主星为水/月/土，不含金星）
        let ds = get_dignities_of(PlanetName::Venus, Zodiac::Taurus, 5.0);
        assert_eq!(
            ds,
            vec![
                DignityKind::Rulership,
                DignityKind::Triplicity,
                DignityKind::Term
            ]
        );
    }

    #[test]
    fn triplicity_standard_three_lords() {
        // 火象：日/木/土（区别于 Lily 的两主星版）
        assert_eq!(
            triplicity(Zodiac::Aries),
            [PlanetName::Sun, PlanetName::Jupiter, PlanetName::Saturn]
        );
        // 水象：金/火/月
        assert_eq!(
            triplicity(Zodiac::Cancer),
            [PlanetName::Venus, PlanetName::Mars, PlanetName::Moon]
        );
    }

    #[test]
    fn egyptian_terms_bounds() {
        // 白羊埃及界：木(6) 金(12) 水(20) 火(25) 土(30)
        assert_eq!(
            egyptian_terms(Zodiac::Aries),
            [
                (PlanetName::Jupiter, 6),
                (PlanetName::Venus, 12),
                (PlanetName::Mercury, 20),
                (PlanetName::Mars, 25),
                (PlanetName::Saturn, 30),
            ]
        );
        // 每星座五段上界之和及末段均为 30
        for z in 0..12u8 {
            let terms = egyptian_terms(Zodiac::from_index(z));
            assert_eq!(terms.len(), 5);
            assert_eq!(terms[4].1, 30);
        }
    }
}
