//! 行星力量（先天尊贵 + 后天状态）计算。
//! 移植自 horo-ui 的 `utils/planet-power/planet-power.ts`。

use std::collections::HashMap;

use crate::api::response::{Horoscope, Planet};
use crate::astro::dignity::{detriment, fall, get_dignity_lords_at, DignityKind};
use crate::astro::horo_math::{angular_distance, deg_norm, get_planet_house, zodiac_long};
use crate::astro::reception::calculate_mutual_receptions;
use crate::enums::planet::{PlanetName, PlanetSpeedState};

/// 先天尊贵分值。
pub(crate) const DIGNITY_SCORES: DignityScores = DignityScores {
    rulership: 5,
    exaltation: 4,
    triplicity: 3,
    term: 2,
    face: 1,
    fall: -4,
    detriment: -5,
    peregrine: -5,
    mutual_reception_rulership: 5,
    mutual_reception_exaltation: 4,
};

pub(crate) struct DignityScores {
    pub rulership: i8,
    pub exaltation: i8,
    pub triplicity: i8,
    pub term: i8,
    pub face: i8,
    pub fall: i8,
    pub detriment: i8,
    pub peregrine: i8,
    pub mutual_reception_rulership: i8,
    pub mutual_reception_exaltation: i8,
}

/// 后天力量分值。
pub(crate) const ACCIDENTAL_SCORES: AccidentalScores = AccidentalScores {
    house1or10: 5,
    house7or4or11: 4,
    house2or5: 3,
    house9: 2,
    house3: 1,
    house12: -5,
    house6or8: -2,
    direct: 4,
    retrograde: -5,
    fast: 2,
    slow: -2,
    oriental_superior: 2,
    occidental_inferior: 2,
    occidental_superior: -2,
    oriental_inferior: -2,
    moon_waxing: 2,
    moon_waning: -2,
    free_from_sun: 5,
    cazimi: 5,
    combust: -5,
    under_sunbeams: -4,
    conjunct_benefic: 5,
    conjunct_north_node: 4,
    trine_benefic: 4,
    sextile_benefic: 3,
    conjunct_malefic: -5,
    conjunct_south_node: -4,
    besieged: -5,
    opposition_malefic: -4,
    square_malefic: -3,
};

pub(crate) struct AccidentalScores {
    pub house1or10: i8,
    pub house7or4or11: i8,
    pub house2or5: i8,
    pub house9: i8,
    pub house3: i8,
    pub house12: i8,
    pub house6or8: i8,
    pub direct: i8,
    pub retrograde: i8,
    pub fast: i8,
    pub slow: i8,
    pub oriental_superior: i8,
    pub occidental_inferior: i8,
    pub occidental_superior: i8,
    pub oriental_inferior: i8,
    pub moon_waxing: i8,
    pub moon_waning: i8,
    pub free_from_sun: i8,
    pub cazimi: i8,
    pub combust: i8,
    pub under_sunbeams: i8,
    pub conjunct_benefic: i8,
    pub conjunct_north_node: i8,
    pub trine_benefic: i8,
    pub sextile_benefic: i8,
    pub conjunct_malefic: i8,
    pub conjunct_south_node: i8,
    pub besieged: i8,
    pub opposition_malefic: i8,
    pub square_malefic: i8,
}

/// 日核阈值：与太阳角距 < 17'。
const CAZIMI_THRESHOLD: f64 = 17.0;
/// 燃烧阈值：与太阳角距 < 8°30'（510'）。
const COMBUST_THRESHOLD: f64 = 510.0;
/// 太阳光束下阈值：与太阳角距 < 17°（1020'）。
const SUNBEAMS_THRESHOLD: f64 = 1020.0;

/// 行星先天黄道力量：描述某行星在其所处位置的尊贵状态。
#[derive(Clone)]
pub(crate) struct PlanetDignity {
    pub planet: Planet,
    pub rulership: bool,
    pub exaltation: bool,
    pub triplicity: bool,
    pub term: bool,
    pub face: bool,
    pub fall: bool,
    pub detriment: bool,
    pub peregrine: bool,
    pub mutual_reception_rulership: bool,
    pub mutual_reception_exaltation: bool,
    pub score: i8,
}

/// 行星后天力量：描述行星在星盘中的位置与运动状态带来的力量增减。
pub(crate) struct PlanetAccidentalPower {
    pub house: u8,
    pub direct: bool,
    pub retrograde: bool,
    pub fast: bool,
    pub slow: bool,
    pub oriental: bool,
    pub occidental: bool,
    pub waxing: bool,
    pub waning: bool,
    pub free_from_sun: bool,
    pub cazimi: bool,
    pub combust: bool,
    pub under_sunbeams: bool,
    pub conjunct_benefic: bool,
    pub conjunct_north_node: bool,
    pub trine_benefic: bool,
    pub sextile_benefic: bool,
    pub conjunct_malefic: bool,
    pub conjunct_south_node: bool,
    pub besieged: bool,
    pub opposition_malefic: bool,
    pub square_malefic: bool,
    pub score: i16,
}

/// 行星总力量：先天力量 + 后天力量。
pub(crate) struct PlanetPower {
    pub planet: Planet,
    pub essential: PlanetDignity,
    pub accidental: PlanetAccidentalPower,
    pub total_score: i16,
}

pub(crate) fn calculate_planet_dignity(planet: &Planet) -> PlanetDignity {
    let position = zodiac_long(planet.long);
    let lords = get_dignity_lords_at(position.zodiac, position.degree);

    let is_rulership = lords.rulership == planet.name;
    let is_exaltation = lords.exaltation == Some(planet.name);
    let is_triplicity = lords.triplicity.contains(&planet.name);
    let is_term = lords.term == planet.name;
    let is_face = lords.face == planet.name;
    let is_fall = fall(position.zodiac) == Some(planet.name);
    let is_detriment = detriment(position.zodiac) == planet.name;
    let is_peregrine = planet.name.is_traditional()
        && !is_rulership
        && !is_exaltation
        && !is_triplicity
        && !is_term
        && !is_face
        && !is_fall
        && !is_detriment;

    let score = (is_rulership as i8) * DIGNITY_SCORES.rulership
        + (is_exaltation as i8) * DIGNITY_SCORES.exaltation
        + (is_triplicity as i8) * DIGNITY_SCORES.triplicity
        + (is_term as i8) * DIGNITY_SCORES.term
        + (is_face as i8) * DIGNITY_SCORES.face
        + (is_fall as i8) * DIGNITY_SCORES.fall
        + (is_detriment as i8) * DIGNITY_SCORES.detriment
        + (is_peregrine as i8) * DIGNITY_SCORES.peregrine;

    PlanetDignity {
        planet: *planet,
        rulership: is_rulership,
        exaltation: is_exaltation,
        triplicity: is_triplicity,
        term: is_term,
        face: is_face,
        fall: is_fall,
        detriment: is_detriment,
        peregrine: is_peregrine,
        mutual_reception_rulership: false,
        mutual_reception_exaltation: false,
        score,
    }
}

pub(crate) fn calculate_all_planet_dignities(
    planets: &[Planet],
) -> Result<Vec<PlanetDignity>, String> {
    if !planets.iter().any(|p| p.name == PlanetName::Sun) {
        return Err("星盘中缺少太阳，无法计算行星力量".to_string());
    }

    Ok(planets
        .iter()
        .filter(|p| p.name.is_traditional())
        .map(calculate_planet_dignity)
        .collect())
}

/// 星盘主星（almuten）：先天分最高者，平分取先出现者。
pub(crate) fn find_chart_almuten(dignities: &[PlanetDignity]) -> Option<&PlanetDignity> {
    dignities.iter().reduce(|max, current| {
        if current.score > max.score {
            current
        } else {
            max
        }
    })
}

fn house_score(house: u8) -> i8 {
    match house {
        1 | 10 => ACCIDENTAL_SCORES.house1or10,
        7 | 4 | 11 => ACCIDENTAL_SCORES.house7or4or11,
        2 | 5 => ACCIDENTAL_SCORES.house2or5,
        9 => ACCIDENTAL_SCORES.house9,
        3 => ACCIDENTAL_SCORES.house3,
        12 => ACCIDENTAL_SCORES.house12,
        6 | 8 => ACCIDENTAL_SCORES.house6or8,
        _ => 0,
    }
}

/// 判断行星黄经是否在 a、b 两点之间的较短弧上（不含端点），用于火土包围判定。
fn is_between_arcs(planet_long: f64, a: f64, b: f64) -> bool {
    let a_to_b = deg_norm(b - a);
    let a_to_planet = deg_norm(planet_long - a);
    if a_to_b <= 180.0 {
        a_to_planet > 0.0 && a_to_planet < a_to_b
    } else {
        a_to_planet > a_to_b
    }
}

fn calculate_planet_accidental_power(
    planet: &Planet,
    horoscope: &Horoscope,
) -> Result<PlanetAccidentalPower, String> {
    let is_valid = planet.name.is_traditional() || planet.name == PlanetName::PartOfFortune;
    if !is_valid {
        return Err(format!("{}不计算后天力量", planet.name));
    }

    let sun = horoscope
        .planets
        .iter()
        .find(|p| p.name == PlanetName::Sun)
        .ok_or_else(|| "缺少太阳".to_string())?;
    let sun_long = sun.long;

    let house = get_planet_house(planet.long, &horoscope.cusps)?;
    let h_score: i16 = house_score(house).into();

    let is_non_moving = planet.name == PlanetName::Sun
        || planet.name == PlanetName::Moon
        || planet.name == PlanetName::PartOfFortune;
    let is_direct = !is_non_moving && planet.speed >= 0.0;
    let is_retrograde = !is_non_moving && planet.speed < 0.0;

    let is_no_speed = planet.name == PlanetName::PartOfFortune;
    let is_fast = !is_no_speed && planet.speed_state == PlanetSpeedState::快;
    let is_slow = !is_no_speed && planet.speed_state == PlanetSpeedState::慢;

    // 日心黄经差：0°=合相，180°=对冲；东西方判定见注释
    let elongation = deg_norm(planet.long - sun_long);
    let is_oriental = !is_non_moving && elongation > 180.0;
    let is_occidental = !is_non_moving && elongation > 0.0 && elongation < 180.0;

    let oriental_occidental_score: i16 = if planet.name.is_superior() {
        if is_oriental {
            ACCIDENTAL_SCORES.oriental_superior.into()
        } else if is_occidental {
            ACCIDENTAL_SCORES.occidental_superior.into()
        } else {
            0
        }
    } else if planet.name.is_inferior() {
        if is_occidental {
            ACCIDENTAL_SCORES.occidental_inferior.into()
        } else if is_oriental {
            ACCIDENTAL_SCORES.oriental_inferior.into()
        } else {
            0
        }
    } else {
        0
    };

    let is_waxing = planet.name == PlanetName::Moon && elongation > 0.0 && elongation <= 180.0;
    let is_waning = planet.name == PlanetName::Moon && elongation > 180.0;

    let (is_cazimi, is_combust, is_under_sunbeams, is_free_from_sun, solar_score) =
        if planet.name != PlanetName::Sun {
            let dist = angular_distance(planet.long, sun_long) * 60.0; // 转为角分
            if dist < CAZIMI_THRESHOLD {
                (true, false, false, false, ACCIDENTAL_SCORES.cazimi.into())
            } else if dist < COMBUST_THRESHOLD {
                (false, true, false, false, ACCIDENTAL_SCORES.combust.into())
            } else if dist < SUNBEAMS_THRESHOLD {
                (
                    false,
                    false,
                    true,
                    false,
                    ACCIDENTAL_SCORES.under_sunbeams.into(),
                )
            } else {
                (
                    false,
                    false,
                    false,
                    true,
                    ACCIDENTAL_SCORES.free_from_sun.into(),
                )
            }
        } else {
            (false, false, false, false, 0)
        };

    let mut is_conjunct_benefic = false;
    let mut is_conjunct_north_node = false;
    let mut is_trine_benefic = false;
    let mut is_sextile_benefic = false;
    let mut is_conjunct_malefic = false;
    let mut is_conjunct_south_node = false;
    let mut is_opposition_malefic = false;
    let mut is_square_malefic = false;

    for a in &horoscope.aspects {
        let other = if a.p0 == planet.name {
            a.p1
        } else if a.p1 == planet.name {
            a.p0
        } else {
            continue;
        };

        match a.aspect_value {
            0 if other.is_benefic() => is_conjunct_benefic = true,
            0 if other.is_malefic() => is_conjunct_malefic = true,
            0 if other == PlanetName::NorthNode => is_conjunct_north_node = true,
            0 if other == PlanetName::SouthNode => is_conjunct_south_node = true,
            120 if other.is_benefic() => is_trine_benefic = true,
            60 if other.is_benefic() => is_sextile_benefic = true,
            180 if other.is_malefic() => is_opposition_malefic = true,
            90 if other.is_malefic() => is_square_malefic = true,
            _ => {}
        }
    }

    let mars_long = horoscope
        .planets
        .iter()
        .find(|p| p.name == PlanetName::Mars)
        .map(|p| p.long);
    let saturn_long = horoscope
        .planets
        .iter()
        .find(|p| p.name == PlanetName::Saturn)
        .map(|p| p.long);
    let is_besieged = mars_long.is_some()
        && saturn_long.is_some()
        && planet.name != PlanetName::Mars
        && planet.name != PlanetName::Saturn
        && is_between_arcs(planet.long, mars_long.unwrap(), saturn_long.unwrap())
        && horoscope.aspects.iter().any(|a| {
            a.aspect_value == 0
                && ((a.p0 == planet.name && a.p1 == PlanetName::Mars)
                    || (a.p1 == planet.name && a.p0 == PlanetName::Mars))
        })
        && horoscope.aspects.iter().any(|a| {
            a.aspect_value == 0
                && ((a.p0 == planet.name && a.p1 == PlanetName::Saturn)
                    || (a.p1 == planet.name && a.p0 == PlanetName::Saturn))
        });

    let score = h_score
        + (is_direct as i16) * ACCIDENTAL_SCORES.direct as i16
        + (is_retrograde as i16) * ACCIDENTAL_SCORES.retrograde as i16
        + (is_fast as i16) * ACCIDENTAL_SCORES.fast as i16
        + (is_slow as i16) * ACCIDENTAL_SCORES.slow as i16
        + oriental_occidental_score
        + (is_waxing as i16) * ACCIDENTAL_SCORES.moon_waxing as i16
        + (is_waning as i16) * ACCIDENTAL_SCORES.moon_waning as i16
        + solar_score
        + (is_conjunct_benefic as i16) * ACCIDENTAL_SCORES.conjunct_benefic as i16
        + (is_conjunct_north_node as i16) * ACCIDENTAL_SCORES.conjunct_north_node as i16
        + (is_trine_benefic as i16) * ACCIDENTAL_SCORES.trine_benefic as i16
        + (is_sextile_benefic as i16) * ACCIDENTAL_SCORES.sextile_benefic as i16
        + (is_conjunct_malefic as i16) * ACCIDENTAL_SCORES.conjunct_malefic as i16
        + (is_conjunct_south_node as i16) * ACCIDENTAL_SCORES.conjunct_south_node as i16
        + (is_besieged as i16) * ACCIDENTAL_SCORES.besieged as i16
        + (is_opposition_malefic as i16) * ACCIDENTAL_SCORES.opposition_malefic as i16
        + (is_square_malefic as i16) * ACCIDENTAL_SCORES.square_malefic as i16;

    Ok(PlanetAccidentalPower {
        house,
        direct: is_direct,
        retrograde: is_retrograde,
        fast: is_fast,
        slow: is_slow,
        oriental: is_oriental,
        occidental: is_occidental,
        waxing: is_waxing,
        waning: is_waning,
        free_from_sun: is_free_from_sun,
        cazimi: is_cazimi,
        combust: is_combust,
        under_sunbeams: is_under_sunbeams,
        conjunct_benefic: is_conjunct_benefic,
        conjunct_north_node: is_conjunct_north_node,
        trine_benefic: is_trine_benefic,
        sextile_benefic: is_sextile_benefic,
        conjunct_malefic: is_conjunct_malefic,
        conjunct_south_node: is_conjunct_south_node,
        besieged: is_besieged,
        opposition_malefic: is_opposition_malefic,
        square_malefic: is_square_malefic,
        score,
    })
}

/// 计算全部行星（七颗行星 + 福点）的先天 / 后天 / 总力量。
pub(crate) fn calculate_all_planet_powers(
    horoscope: &Horoscope,
) -> Result<Vec<PlanetPower>, String> {
    let dignities = calculate_all_planet_dignities(&horoscope.planets)?;

    // 互容映射：庙互容 / 旺互容
    let mutual_receptions = calculate_mutual_receptions(horoscope);
    let mut mr_map: HashMap<PlanetName, (bool, bool)> = HashMap::new();
    for mr in &mutual_receptions {
        let a_has_rulership = mr.a_dignities.contains(&DignityKind::Rulership);
        let b_has_rulership = mr.b_dignities.contains(&DignityKind::Rulership);
        let a_has_exaltation = mr.a_dignities.contains(&DignityKind::Exaltation);
        let b_has_exaltation = mr.b_dignities.contains(&DignityKind::Exaltation);

        if a_has_rulership && b_has_rulership {
            mr_map.entry(mr.a).or_default().0 = true;
            mr_map.entry(mr.b).or_default().0 = true;
        }
        if a_has_exaltation && b_has_exaltation {
            mr_map.entry(mr.a).or_default().1 = true;
            mr_map.entry(mr.b).or_default().1 = true;
        }
    }

    let mut powers = Vec::new();
    for d in dignities {
        let (mr_rulership, mr_exaltation) = mr_map
            .get(&d.planet.name)
            .copied()
            .unwrap_or((false, false));

        let essential_score = d.score
            + (mr_rulership as i8) * DIGNITY_SCORES.mutual_reception_rulership
            + (mr_exaltation as i8) * DIGNITY_SCORES.mutual_reception_exaltation;

        let essential = PlanetDignity {
            mutual_reception_rulership: mr_rulership,
            mutual_reception_exaltation: mr_exaltation,
            score: essential_score,
            ..d.clone()
        };

        let accidental = calculate_planet_accidental_power(&d.planet, horoscope)?;

        powers.push(PlanetPower {
            planet: d.planet,
            total_score: essential_score as i16 + accidental.score,
            essential,
            accidental,
        });
    }

    // 福点：不参与互容，直接叠加先天 + 后天
    let pof_dignity = calculate_planet_dignity(&horoscope.part_of_fortune);
    let pof_accidental = calculate_planet_accidental_power(&horoscope.part_of_fortune, horoscope)?;
    powers.push(PlanetPower {
        total_score: pof_dignity.score as i16 + pof_accidental.score,
        planet: horoscope.part_of_fortune,
        essential: pof_dignity,
        accidental: pof_accidental,
    });

    Ok(powers)
}

#[cfg(test)]
mod tests {
    use super::{house_score, is_between_arcs, DIGNITY_SCORES};

    #[test]
    fn house_scores_match_table() {
        assert_eq!(house_score(1), 5);
        assert_eq!(house_score(10), 5);
        assert_eq!(house_score(7), 4);
        assert_eq!(house_score(4), 4);
        assert_eq!(house_score(11), 4);
        assert_eq!(house_score(2), 3);
        assert_eq!(house_score(5), 3);
        assert_eq!(house_score(9), 2);
        assert_eq!(house_score(3), 1);
        assert_eq!(house_score(12), -5);
        assert_eq!(house_score(6), -2);
        assert_eq!(house_score(8), -2);
    }

    #[test]
    fn is_between_arcs_uses_shorter_arc() {
        // a=0, b=90：短弧 (0, 90)
        assert!(is_between_arcs(45.0, 0.0, 90.0));
        assert!(!is_between_arcs(0.0, 0.0, 90.0));
        assert!(!is_between_arcs(90.0, 0.0, 90.0));
        assert!(!is_between_arcs(180.0, 0.0, 90.0));
        // a=90, b=0：aToB=270>180 → 短弧为 (0, 90)，行星的黄经差须落在 (270, 360)
        assert!(is_between_arcs(45.0, 90.0, 0.0));
        assert!(!is_between_arcs(300.0, 90.0, 0.0));
    }

    #[test]
    fn dignity_scores_are_traditional() {
        assert_eq!(DIGNITY_SCORES.rulership, 5);
        assert_eq!(DIGNITY_SCORES.exaltation, 4);
        assert_eq!(DIGNITY_SCORES.triplicity, 3);
        assert_eq!(DIGNITY_SCORES.term, 2);
        assert_eq!(DIGNITY_SCORES.face, 1);
        assert_eq!(DIGNITY_SCORES.fall, -4);
        assert_eq!(DIGNITY_SCORES.detriment, -5);
        assert_eq!(DIGNITY_SCORES.peregrine, -5);
    }
}
