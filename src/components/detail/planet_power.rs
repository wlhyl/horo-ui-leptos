//! 行星力量子组件：星盘主星 + 各行星先天 / 后天力量标签。
//! 对齐 horo-ui 的 planet-power.component(.html)。
use leptos::prelude::*;

use crate::api::response::Horoscope;
use crate::astro::planet_power::{calculate_all_planet_powers, PlanetPower};
use crate::components::AlertDialog;
use crate::enums::planet::PlanetName;
use crate::render::glyphs::planet_glyph;

stylance::import_crate_style!(#[allow(dead_code)] style, "src/components/detail/detail.module.css");
stylance::import_crate_style!(#[allow(dead_code)] card, "src/shared/card.module.css");

/// 一个力量标签：css 类 + 显示文本。
struct Tag(&'static str, String);

fn tag(class: &'static str, label: &str, score: i8) -> Tag {
    Tag(class, format!("{label}{score}"))
}

/// 先天尊贵标签列表（顺序与 Angular 模板一致）。
fn essential_tags(p: &PlanetPower) -> Vec<Tag> {
    let mut tags = Vec::new();
    if p.essential.rulership {
        tags.push(tag(style::dignity_tag, "庙", 5));
    }
    if p.essential.exaltation {
        tags.push(tag(style::dignity_tag, "旺", 4));
    }
    if p.essential.triplicity {
        tags.push(tag(style::dignity_tag, "三分", 3));
    }
    if p.essential.term {
        tags.push(tag(style::dignity_tag, "界", 2));
    }
    if p.essential.face {
        tags.push(tag(style::dignity_tag, "面", 1));
    }
    if p.essential.mutual_reception_rulership {
        tags.push(tag(style::dignity_tag, "庙互容", 5));
    }
    if p.essential.mutual_reception_exaltation {
        tags.push(tag(style::dignity_tag, "旺互容", 4));
    }
    if p.essential.fall {
        tags.push(tag(style::detriment_tag, "陷", -4));
    }
    if p.essential.detriment {
        tags.push(tag(style::detriment_tag, "弱", -5));
    }
    if p.essential.peregrine {
        tags.push(tag(style::detriment_tag, "游离", -5));
    }
    tags
}

/// 后天力量标签列表（顺序与 Angular 模板一致）。
fn accidental_tags(p: &PlanetPower) -> Vec<Tag> {
    let a = &p.accidental;
    let mut tags = Vec::new();

    match a.house {
        1 | 10 => tags.push(tag(style::accidental_tag, &format!("{}宫", a.house), 5)),
        7 | 4 | 11 => tags.push(tag(style::accidental_tag, &format!("{}宫", a.house), 4)),
        2 | 5 => tags.push(tag(style::accidental_tag, &format!("{}宫", a.house), 3)),
        9 => tags.push(tag(style::accidental_tag, &format!("{}宫", a.house), 2)),
        3 => tags.push(tag(style::accidental_tag, &format!("{}宫", a.house), 1)),
        12 => tags.push(tag(style::detriment_tag, &format!("{}宫", a.house), -5)),
        6 | 8 => tags.push(tag(style::detriment_tag, &format!("{}宫", a.house), -2)),
        _ => {}
    }
    if a.direct {
        tags.push(tag(style::accidental_tag, "顺", 4));
    }
    if a.retrograde {
        tags.push(tag(style::detriment_tag, "逆", -5));
    }
    if a.fast {
        tags.push(tag(style::accidental_tag, "快", 2));
    }
    if a.slow {
        tags.push(tag(style::detriment_tag, "慢", -2));
    }
    // 东西分值因上级行星 / 下级行星而异
    if a.oriental && p.planet.name.is_superior() {
        tags.push(tag(style::accidental_tag, "东", 2));
    }
    if a.occidental && p.planet.name.is_superior() {
        tags.push(tag(style::detriment_tag, "西", -2));
    }
    if a.occidental && p.planet.name.is_inferior() {
        tags.push(tag(style::accidental_tag, "西", 2));
    }
    if a.oriental && p.planet.name.is_inferior() {
        tags.push(tag(style::detriment_tag, "东", -2));
    }
    if a.waxing {
        tags.push(tag(style::accidental_tag, "渐圆", 2));
    }
    if a.waning {
        tags.push(tag(style::detriment_tag, "渐亏", -2));
    }
    if a.cazimi {
        tags.push(tag(style::solar_tag, "日核", 5));
    }
    if a.combust {
        tags.push(tag(style::detriment_tag, "燃烧", -5));
    }
    if a.under_sunbeams {
        tags.push(tag(style::detriment_tag, "光束下", -4));
    }
    if a.free_from_sun {
        tags.push(tag(style::accidental_tag, "离日", 5));
    }
    if a.conjunct_benefic {
        tags.push(tag(style::accidental_tag, "吉星合", 5));
    }
    if a.conjunct_north_node {
        tags.push(tag(style::accidental_tag, "北交合", 4));
    }
    if a.trine_benefic {
        tags.push(tag(style::accidental_tag, "吉星三合", 4));
    }
    if a.sextile_benefic {
        tags.push(tag(style::accidental_tag, "吉星六合", 3));
    }
    if a.conjunct_malefic {
        tags.push(tag(style::detriment_tag, "凶星合", -5));
    }
    if a.conjunct_south_node {
        tags.push(tag(style::detriment_tag, "南交合", -4));
    }
    if a.besieged {
        tags.push(tag(style::detriment_tag, "包围", -5));
    }
    if a.opposition_malefic {
        tags.push(tag(style::detriment_tag, "凶星冲", -4));
    }
    if a.square_malefic {
        tags.push(tag(style::detriment_tag, "凶星刑", -3));
    }
    tags
}

fn tag_spans(tags: Vec<Tag>) -> impl IntoView {
    tags.into_iter()
        .map(|Tag(class, text)| view! { <span class=class>{text}</span> })
        .collect::<Vec<_>>()
}

#[component]
pub fn PlanetPower(horoscope: Horoscope) -> impl IntoView {
    let message = RwSignal::new(String::new());
    let powers = match calculate_all_planet_powers(&horoscope) {
        Ok(powers) => powers,
        Err(err) => {
            message.set(err);
            Vec::new()
        }
    };

    // 总分最高的行星（剔除福点，平分取先出现者），展示为「星盘主星」；
    // 与 find_chart_almuten 的先天分口径不同
    let lord_of_horo = powers
        .iter()
        .filter(|p| p.planet.name != PlanetName::PartOfFortune)
        .reduce(|max, cur| if cur.total_score > max.total_score { cur } else { max });

    view! {
        <div class=card::card>
            <h3 class=style::section_title>"行星力量"</h3>
            <AlertDialog header="错误" message=message/>
            {lord_of_horo.map(|a| {
                view! {
                    <p class=style::almuten_summary>
                        "星盘主星："
                        <span class=style::glyph>{planet_glyph(a.planet.name)}</span>
                        {format!("（{}分）", a.total_score)}
                    </p>
                }
            })}
            <ul class=style::power_list>
                {powers
                    .into_iter()
                    .map(|p| {
                        let essential = tag_spans(essential_tags(&p));
                        let accidental = tag_spans(accidental_tags(&p));
                        let glyph = planet_glyph(p.planet.name);
                        let total = p.total_score;
                        let essential_score = p.essential.score;
                        let accidental_score = p.accidental.score;
                        view! {
                            <li>
                                <span class=style::glyph>{glyph}</span>
                                {format!("：{total}分")}
                                <div class=style::score_section>
                                    <span class=style::section_label>
                                        {format!("先天 {essential_score}")}
                                    </span>
                                    {essential}
                                </div>
                                <div class=style::score_section>
                                    <span class=style::section_label>
                                        {format!("后天 {accidental_score}")}
                                    </span>
                                    {accidental}
                                </div>
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </div>
    }
}
