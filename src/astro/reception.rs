//! 接纳（reception）与互容（mutual reception）判定。
//! 移植自 horo-ui 的 `utils/reception/reception.ts`。

use std::collections::HashMap;

use crate::api::response::{Aspect, Horoscope};
use crate::astro::dignity::{get_dignities_of, DignityKind};
use crate::astro::horo_math::zodiac_long;
use crate::enums::planet::{PlanetName, TRADITIONAL_PLANETS};

/// 可被接纳的星体：七传统行星 + 福点 + 四轴（接纳关系中被接纳方仅限此列表）。
fn is_receivable_body(planet: PlanetName) -> bool {
    TRADITIONAL_PLANETS.contains(&planet)
        || matches!(
            planet,
            PlanetName::PartOfFortune
                | PlanetName::ASC
                | PlanetName::DSC
                | PlanetName::MC
                | PlanetName::IC
        )
}

/// 一条接纳关系：receiver 在 received 所落位置拥有尊贵。
pub(crate) struct Reception {
    pub receiver: PlanetName,
    pub received: PlanetName,
    pub dignities: Vec<DignityKind>,
    pub aspect: Aspect,
}

/// 一条互容关系：a 在 b 的位置有尊贵，且 b 在 a 的位置也有尊贵。
pub(crate) struct MutualReception {
    pub a: PlanetName,
    pub b: PlanetName,
    pub a_dignities: Vec<DignityKind>,
    pub b_dignities: Vec<DignityKind>,
}

/// 星体黄经表：行星 + 四轴 + 福点。
fn build_body_long_map(h: &Horoscope) -> HashMap<PlanetName, f64> {
    let mut map: HashMap<PlanetName, f64> = h.planets.iter().map(|p| (p.name, p.long)).collect();
    map.insert(h.asc.name, h.asc.long);
    map.insert(h.mc.name, h.mc.long);
    map.insert(h.dsc.name, h.dsc.long);
    map.insert(h.ic.name, h.ic.long);
    map.insert(h.part_of_fortune.name, h.part_of_fortune.long);
    map
}

/// 判定单个相位某个方向上是否存在接纳。
fn consider_reception(
    aspect: &Aspect,
    long_map: &HashMap<PlanetName, f64>,
    reverse: bool,
) -> Option<Reception> {
    let receiver = if reverse { aspect.p1 } else { aspect.p0 };
    let received = if reverse { aspect.p0 } else { aspect.p1 };

    if receiver == received {
        return None;
    }
    if !TRADITIONAL_PLANETS.contains(&receiver) {
        return None;
    }
    if !is_receivable_body(received) {
        return None;
    }

    let long = *long_map.get(&received)?;

    let position = zodiac_long(long);
    let dignities = get_dignities_of(receiver, position.zodiac, position.degree);
    if dignities.is_empty() {
        return None;
    }

    Some(Reception {
        receiver,
        received,
        dignities,
        aspect: *aspect,
    })
}

/// 计算星盘的全部接纳关系：遍历相位，正反两个方向各判定一次。
pub(crate) fn calculate_receptions(h: &Horoscope) -> Vec<Reception> {
    let long_map = build_body_long_map(h);
    let mut result = Vec::new();
    for aspect in &h.aspects {
        if let Some(r) = consider_reception(aspect, &long_map, false) {
            result.push(r);
        }
        if let Some(r) = consider_reception(aspect, &long_map, true) {
            result.push(r);
        }
    }
    result
}

/// 计算七传统行星两两之间的互容关系。
pub(crate) fn calculate_mutual_receptions(h: &Horoscope) -> Vec<MutualReception> {
    let long_map = build_body_long_map(h);
    let mut result = Vec::new();

    for i in 0..TRADITIONAL_PLANETS.len() {
        let a = TRADITIONAL_PLANETS[i];
        let Some(&a_long) = long_map.get(&a) else {
            continue;
        };

        for j in (i + 1)..TRADITIONAL_PLANETS.len() {
            let b = TRADITIONAL_PLANETS[j];
            let Some(&b_long) = long_map.get(&b) else {
                continue;
            };

            let a_pos = zodiac_long(a_long);
            let b_pos = zodiac_long(b_long);
            let a_dignities = get_dignities_of(a, b_pos.zodiac, b_pos.degree);
            let b_dignities = get_dignities_of(b, a_pos.zodiac, a_pos.degree);
            if !a_dignities.is_empty() && !b_dignities.is_empty() {
                result.push(MutualReception {
                    a,
                    b,
                    a_dignities,
                    b_dignities,
                });
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{calculate_mutual_receptions, calculate_receptions};
    use crate::astro::dignity::DignityKind;
    use crate::enums::planet::PlanetName;

    fn horoscope_with(longs: &[(PlanetName, f64)]) -> crate::api::response::Horoscope {
        crate::api::response::Horoscope {
            house_name: crate::enums::house::HouseName::Alcabitus,
            cusps: (0..12).map(|i| i as f64 * 30.0).collect(),
            asc: planet(PlanetName::ASC, 0.0),
            mc: planet(PlanetName::MC, 270.0),
            dsc: planet(PlanetName::DSC, 180.0),
            ic: planet(PlanetName::IC, 90.0),
            part_of_fortune: planet(PlanetName::PartOfFortune, 45.0),
            planets: longs
                .iter()
                .map(|&(name, long)| planet(name, long))
                .collect(),
            is_diurnal: true,
            planetary_day: Some(PlanetName::Sun),
            planetary_hours: Some(PlanetName::Sun),
            aspects: vec![],
            antiscoins: vec![],
            contraantiscias: vec![],
            fixed_stars: vec![],
        }
    }

    fn planet(name: PlanetName, long: f64) -> crate::api::response::Planet {
        crate::api::response::Planet {
            name,
            long,
            lat: 0.0,
            speed: 1.0,
            ra: 0.0,
            dec: 0.0,
            orb: 0,
            speed_state: crate::enums::planet::PlanetSpeedState::均,
        }
    }

    #[test]
    fn mutual_reception_between_rulers() {
        // 火星落摩羯（土星庙）、土星落白羊（火星庙）→ 庙互容
        let h = horoscope_with(&[(PlanetName::Mars, 280.0), (PlanetName::Saturn, 5.0)]);
        let mrs = calculate_mutual_receptions(&h);
        assert_eq!(mrs.len(), 1);
        assert_eq!(mrs[0].a, PlanetName::Mars);
        assert_eq!(mrs[0].b, PlanetName::Saturn);
        // 火星在白羊 5°：庙 + 面（火星为白羊 0-10° 面主星）
        assert_eq!(
            mrs[0].a_dignities,
            vec![DignityKind::Rulership, DignityKind::Face]
        );
        assert_eq!(mrs[0].b_dignities, vec![DignityKind::Rulership]);
    }

    #[test]
    fn receptions_require_aspect_and_dignity() {
        use crate::api::response::Aspect;
        // 火星合 ASC：ASC 落白羊 0°，火星为白羊主星 → 火星接纳 ASC
        let mut h = horoscope_with(&[(PlanetName::Mars, 1.0)]);
        h.aspects = vec![Aspect {
            aspect_value: 0,
            apply: true,
            d: 1.0,
            p0: PlanetName::Mars,
            p1: PlanetName::ASC,
        }];
        let rs = calculate_receptions(&h);
        assert_eq!(rs.len(), 1);
        assert_eq!(rs[0].receiver, PlanetName::Mars);
        assert_eq!(rs[0].received, PlanetName::ASC);
        // 火星在白羊 0°：庙 + 面
        assert_eq!(
            rs[0].dignities,
            vec![DignityKind::Rulership, DignityKind::Face]
        );

        // 无相位则无接纳
        h.aspects.clear();
        assert!(calculate_receptions(&h).is_empty());
    }
}
