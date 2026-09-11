//! 气质（temperament）计算：按传统体液学说收集影响气质的行星与星座。
//! 移植自 horo-ui 的 `utils/temperament/temperament.ts`。

use crate::api::response::Horoscope;
use crate::astro::dignity::rulership;
use crate::astro::horo_math::{deg_norm, get_planet_house, zodiac_long};
use crate::astro::planet_power::{calculate_all_planet_dignities, find_chart_almuten};
use crate::enums::planet::PlanetName;
use crate::enums::zodiac::Zodiac;

/// 四性质。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Quality {
    Hot,
    Cold,
    Dry,
    Wet,
}

/// 贡献者名称（替代 TS 的 PlanetName | Zodiac 联合类型）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContributorName {
    Planet(PlanetName),
    Sign(Zodiac),
}

impl ContributorName {
    /// 稳定 id，如 "planet:Sun" / "sign:Aries"。
    pub(crate) fn id(self) -> String {
        match self {
            ContributorName::Planet(p) => format!("planet:{p}"),
            ContributorName::Sign(s) => format!("sign:{s}"),
        }
    }
}

/// 一条气质贡献者：来源与四性质开关（可被用户交互修改）。
#[derive(Clone)]
pub(crate) struct TemperamentContributor {
    pub id: String,
    pub name: ContributorName,
    pub sources: Vec<String>,
    pub hot: bool,
    pub cold: bool,
    pub dry: bool,
    pub wet: bool,
}

/// 四体液汇总。
#[derive(Clone, PartialEq)]
pub(crate) struct TemperamentSummary {
    pub hot: u16,
    pub cold: u16,
    pub dry: u16,
    pub wet: u16,
    /// 多血质（热+湿）
    pub sanguine: u16,
    /// 粘液质（冷+湿）
    pub phlegmatic: u16,
    /// 胆汁质（热+干）
    pub choleric: u16,
    /// 抑郁质（冷+干）
    pub melancholic: u16,
    pub total: u16,
    pub percentages: TemperamentPercentages,
}

/// 四体液占比（对应 TS `TemperamentSummary.percentages`）。
#[derive(Clone, PartialEq)]
pub(crate) struct TemperamentPercentages {
    /// 多血质（热+湿）
    pub sanguine: f64,
    /// 粘液质（冷+湿）
    pub phlegmatic: f64,
    /// 胆汁质（热+干）
    pub choleric: f64,
    /// 抑郁质（冷+干）
    pub melancholic: f64,
}

/// 参与收集的行星：七颗行星 + 南北交。
pub(crate) const GATHERABLE_PLANETS: [PlanetName; 9] = [
    PlanetName::Sun,
    PlanetName::Moon,
    PlanetName::Mercury,
    PlanetName::Venus,
    PlanetName::Mars,
    PlanetName::Jupiter,
    PlanetName::Saturn,
    PlanetName::NorthNode,
    PlanetName::SouthNode,
];

/// 固定性质的行星（日/月按星盘实时计算，不在此表）。
fn planet_fixed_qualities(name: PlanetName) -> &'static [Quality] {
    match name {
        PlanetName::Saturn => &[Quality::Cold, Quality::Dry],
        PlanetName::Jupiter => &[Quality::Hot, Quality::Wet],
        PlanetName::Mars => &[Quality::Hot, Quality::Dry],
        PlanetName::Venus => &[Quality::Hot, Quality::Wet],
        PlanetName::Mercury => &[Quality::Hot, Quality::Wet],
        PlanetName::NorthNode => &[Quality::Hot, Quality::Wet],
        PlanetName::SouthNode => &[Quality::Hot, Quality::Dry, Quality::Cold],
        _ => &[],
    }
}

/// 太阳性质：按所在星座三等分（本位 / 固定 / 变动）。
fn sun_qualities(sun_sign: Zodiac) -> &'static [Quality] {
    match (sun_sign as u8) / 3 {
        0 => &[Quality::Hot, Quality::Wet],
        1 => &[Quality::Hot, Quality::Dry],
        2 => &[Quality::Cold, Quality::Dry],
        _ => &[Quality::Cold, Quality::Wet],
    }
}

/// 月亮性质：按月日距角四等分（月相）。
fn moon_qualities(elongation: f64) -> &'static [Quality] {
    match (elongation / 90.0).floor() as u8 {
        0 => &[Quality::Hot, Quality::Wet],
        1 => &[Quality::Hot, Quality::Dry],
        2 => &[Quality::Cold, Quality::Dry],
        _ => &[Quality::Cold, Quality::Wet],
    }
}

/// 星座性质：按元素（火 / 土 / 风 / 水）。
fn sign_qualities(sign: Zodiac) -> &'static [Quality] {
    match (sign as u8) % 4 {
        0 => &[Quality::Hot, Quality::Dry],
        1 => &[Quality::Cold, Quality::Dry],
        2 => &[Quality::Hot, Quality::Wet],
        _ => &[Quality::Cold, Quality::Wet],
    }
}

/// 计算行星性质：日需太阳星座，月需月日距角，其余查固定表。
fn planet_qualities(
    name: PlanetName,
    sun_sign: Option<Zodiac>,
    elongation: Option<f64>,
) -> Result<&'static [Quality], String> {
    if name == PlanetName::Sun {
        let Some(sun_sign) = sun_sign else {
            return Err("计算太阳性质需要提供太阳所在星座".to_string());
        };
        return Ok(sun_qualities(sun_sign));
    }
    if name == PlanetName::Moon {
        let Some(elongation) = elongation else {
            return Err("计算月亮性质需要提供月日距角".to_string());
        };
        return Ok(moon_qualities(elongation));
    }
    Ok(planet_fixed_qualities(name))
}

fn to_contributor(
    name: ContributorName,
    sources: Vec<String>,
    qualities: &[Quality],
) -> TemperamentContributor {
    TemperamentContributor {
        id: name.id(),
        name,
        sources,
        hot: qualities.contains(&Quality::Hot),
        cold: qualities.contains(&Quality::Cold),
        dry: qualities.contains(&Quality::Dry),
        wet: qualities.contains(&Quality::Wet),
    }
}

/// 保序来源表：键按首次插入顺序排列，来源去重追加（对应 TS 的 Map 插入序）。
struct OrderedSources<K: Copy + PartialEq> {
    entries: Vec<(K, Vec<String>)>,
}

impl<K: Copy + PartialEq> OrderedSources<K> {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add(&mut self, key: K, source: &str) {
        if let Some((_, sources)) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            if !sources.iter().any(|s| s == source) {
                sources.push(source.to_string());
            }
        } else {
            self.entries.push((key, vec![source.to_string()]));
        }
    }

    fn into_entries(self) -> Vec<(K, Vec<String>)> {
        self.entries
    }
}

/// 计算气质贡献者列表。
pub(crate) fn calculate_temperament_contributors(
    horo: &Horoscope,
) -> Result<Vec<TemperamentContributor>, String> {
    let sun = horo
        .planets
        .iter()
        .find(|p| p.name == PlanetName::Sun)
        .ok_or_else(|| "星盘中缺少太阳或月亮，无法计算气质".to_string())?;
    let moon = horo
        .planets
        .iter()
        .find(|p| p.name == PlanetName::Moon)
        .ok_or_else(|| "星盘中缺少太阳或月亮，无法计算气质".to_string())?;

    let elongation = deg_norm(moon.long - sun.long);
    let sun_sign = zodiac_long(sun.long).zodiac;
    let asc_sign = zodiac_long(horo.asc.long).zodiac;

    let mut planet_sources: OrderedSources<PlanetName> = OrderedSources::new();

    planet_sources.add(rulership(asc_sign), "1宫主星");

    for p in &horo.planets {
        if !GATHERABLE_PLANETS.contains(&p.name) {
            continue;
        }
        let house = get_planet_house(p.long, &horo.cusps)?;
        if house == 1 {
            planet_sources.add(p.name, "1宫内行星");
        }
    }

    for a in &horo.aspects {
        let other = if a.p0 == PlanetName::ASC {
            Some(a.p1)
        } else if a.p1 == PlanetName::ASC {
            Some(a.p0)
        } else {
            None
        };
        if let Some(other) = other {
            if GATHERABLE_PLANETS.contains(&other) {
                planet_sources.add(other, "与ASC相位");
            }
        }
    }

    planet_sources.add(PlanetName::Moon, "月亮");

    for a in &horo.aspects {
        let other = if a.p0 == PlanetName::Moon {
            Some(a.p1)
        } else if a.p1 == PlanetName::Moon {
            Some(a.p0)
        } else {
            None
        };
        if let Some(other) = other {
            if GATHERABLE_PLANETS.contains(&other) {
                planet_sources.add(other, "与月亮相位");
            }
        }
    }

    planet_sources.add(PlanetName::Sun, "太阳");

    if let Ok(dignities) = calculate_all_planet_dignities(&horo.planets) {
        if let Some(almuten) = find_chart_almuten(&dignities) {
            planet_sources.add(almuten.planet.name, "星盘主星");
        }
    }

    let mut sign_sources: OrderedSources<Zodiac> = OrderedSources::new();
    sign_sources.add(asc_sign, "1宫头星座");

    let mut contributors = Vec::new();
    for (name, sources) in planet_sources.into_entries() {
        let planet = horo.planets.iter().find(|p| p.name == name);
        if let Some(planet) = planet {
            sign_sources.add(zodiac_long(planet.long).zodiac, "行星所落星座");
        }
        let qualities = planet_qualities(name, Some(sun_sign), Some(elongation))?;
        contributors.push(to_contributor(
            ContributorName::Planet(name),
            sources,
            qualities,
        ));
    }

    for (sign, sources) in sign_sources.into_entries() {
        contributors.push(to_contributor(
            ContributorName::Sign(sign),
            sources,
            sign_qualities(sign),
        ));
    }

    Ok(contributors)
}

/// 手动添加贡献者时计算性质：日/月按星盘实时算，其余查表。
pub(crate) fn get_contributor_qualities(
    name: ContributorName,
    horo: &Horoscope,
) -> Result<&'static [Quality], String> {
    let planet_name = match name {
        ContributorName::Sign(sign) => return Ok(sign_qualities(sign)),
        ContributorName::Planet(planet_name) => planet_name,
    };

    if planet_name != PlanetName::Sun && planet_name != PlanetName::Moon {
        return planet_qualities(planet_name, None, None);
    }
    let sun = horo
        .planets
        .iter()
        .find(|p| p.name == PlanetName::Sun)
        .ok_or_else(|| "星盘中缺少太阳，无法计算太阳性质".to_string())?;
    if planet_name == PlanetName::Sun {
        return planet_qualities(planet_name, Some(zodiac_long(sun.long).zodiac), None);
    }
    let moon = horo
        .planets
        .iter()
        .find(|p| p.name == PlanetName::Moon)
        .ok_or_else(|| "星盘中缺少月亮，无法计算月亮性质".to_string())?;
    planet_qualities(
        planet_name,
        Some(zodiac_long(sun.long).zodiac),
        Some(deg_norm(moon.long - sun.long)),
    )
}

/// 创建一条贡献者（手动添加用，来源固定为「手动添加」）。
pub(crate) fn create_contributor(
    name: ContributorName,
    sources: Vec<String>,
    qualities: &[Quality],
) -> TemperamentContributor {
    to_contributor(name, sources, qualities)
}

/// 汇总四体液得分与百分比。
pub(crate) fn calculate_temperament_summary(
    contributors: &[TemperamentContributor],
) -> TemperamentSummary {
    let mut hot = 0;
    let mut cold = 0;
    let mut dry = 0;
    let mut wet = 0;
    for c in contributors {
        if c.hot {
            hot += 1;
        }
        if c.cold {
            cold += 1;
        }
        if c.dry {
            dry += 1;
        }
        if c.wet {
            wet += 1;
        }
    }

    let sanguine = hot + wet;
    let phlegmatic = cold + wet;
    let choleric = hot + dry;
    let melancholic = cold + dry;
    let total = sanguine + phlegmatic + choleric + melancholic;
    let pct = |v: u16| {
        if total == 0 {
            0.0
        } else {
            f64::from(v) / f64::from(total) * 100.0
        }
    };

    TemperamentSummary {
        hot,
        cold,
        dry,
        wet,
        sanguine,
        phlegmatic,
        choleric,
        melancholic,
        total,
        percentages: TemperamentPercentages {
            sanguine: pct(sanguine),
            phlegmatic: pct(phlegmatic),
            choleric: pct(choleric),
            melancholic: pct(melancholic),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{calculate_temperament_summary, create_contributor, ContributorName, Quality};

    fn contributor(name: ContributorName, qualities: &[Quality]) -> super::TemperamentContributor {
        create_contributor(name, vec!["手动添加".to_string()], qualities)
    }

    #[test]
    fn summary_counts_humors() {
        let cs = vec![
            contributor(
                ContributorName::Planet(super::PlanetName::Saturn),
                &[Quality::Cold, Quality::Dry],
            ),
            contributor(
                ContributorName::Planet(super::PlanetName::Jupiter),
                &[Quality::Hot, Quality::Wet],
            ),
        ];
        let s = calculate_temperament_summary(&cs);
        assert_eq!(s.hot, 1);
        assert_eq!(s.cold, 1);
        assert_eq!(s.dry, 1);
        assert_eq!(s.wet, 1);
        assert_eq!(s.sanguine, 2); // 热+湿
        assert_eq!(s.phlegmatic, 2); // 冷+湿
        assert_eq!(s.choleric, 2); // 热+干
        assert_eq!(s.melancholic, 2); // 冷+干
        assert_eq!(s.total, 8);
        assert_eq!(s.percentages.sanguine, 25.0);
    }

    #[test]
    fn empty_contributors_yield_zero_percent() {
        let s = calculate_temperament_summary(&[]);
        assert_eq!(s.total, 0);
        assert_eq!(s.percentages.sanguine, 0.0);
    }

    #[test]
    fn contributor_ids_are_stable() {
        assert_eq!(
            ContributorName::Planet(super::PlanetName::Sun).id(),
            "planet:Sun"
        );
        assert_eq!(
            ContributorName::Sign(super::Zodiac::Aries).id(),
            "sign:Aries"
        );
    }
}
