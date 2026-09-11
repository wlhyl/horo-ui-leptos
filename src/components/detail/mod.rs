//! 详情页：行星力量 / 接纳 / 互融 / 气质 / 映点 / 反映点 / 视力点 / 恒星。
//! 功能对齐 horo-ui 的 detail.component(.html)。
mod planet_power;
mod reception;
mod temperament;

use leptos::either::Either;
use leptos::prelude::*;

use crate::api::response::{Aspect, Horoscope, Planet};
use crate::astro::horo_math::{angular_distance, deg_norm, degree_to_dms, zodiac_long};
use crate::enums::planet::PlanetName;
use crate::render::glyphs::{planet_glyph, zodiac_glyph};

use planet_power::PlanetPower;
use reception::Reception;
use temperament::Temperament;

// 作用域样式：src/components/detail/detail.module.css（四个子组件共享，各取所需）
stylance::import_crate_style!(
    #[allow(dead_code)]
    style,
    "src/components/detail/detail.module.css"
);
stylance::import_crate_style!(
    #[allow(dead_code)]
    card,
    "src/shared/card.module.css"
);

/// 太阳视力点：ASC + 太阳黄经 - 火星黄经。
fn part_of_solar_vision(h: &Horoscope) -> Option<f64> {
    let sun = h.planets.iter().find(|p| p.name == PlanetName::Sun)?;
    let mars = h.planets.iter().find(|p| p.name == PlanetName::Mars)?;
    Some(deg_norm(h.asc.long + sun.long - mars.long))
}

/// 月亮视力点：ASC + 月亮黄经 - 土星黄经。
fn part_of_lunar_vision(h: &Horoscope) -> Option<f64> {
    let moon = h.planets.iter().find(|p| p.name == PlanetName::Moon)?;
    let saturn = h.planets.iter().find(|p| p.name == PlanetName::Saturn)?;
    Some(deg_norm(h.asc.long + moon.long - saturn.long))
}

/// 视力点显示行：d° 星座符号 m's。
fn vision_position(long: f64) -> impl IntoView {
    let position = zodiac_long(long);
    let (d, m, s) = degree_to_dms(position.degree);
    view! {
        <span class=style::vision_position>
            {format!("{}° {} {}'{}", d, zodiac_glyph(position.zodiac), m, s)}
        </span>
    }
}

/// 视力点视图：有值显示位置，缺数据显示错误。
fn vision_view(vision: Option<f64>) -> impl IntoView {
    match vision {
        Some(long) => Either::Left(vision_position(long)),
        None => Either::Right(
            view! { <span class=style::vision_error>"计算错误：缺少必要行星数据"</span> },
        ),
    }
}

/// 映点 / 反映点列表：「p0 与 p1，X度Y分Z秒」。
fn antiscia_items(items: &[Aspect]) -> impl IntoView + use<> {
    items
        .iter()
        .map(|item| {
            let (d, m, s) = degree_to_dms(item.d);
            view! {
                <li>
                    <span class=style::glyph>{planet_glyph(item.p0)}</span>
                    " 与 "
                    <span class=style::glyph>{planet_glyph(item.p1)}</span>
                    {format!("，{}度{}分{}秒", d, m, s)}
                </li>
            }
        })
        .collect::<Vec<_>>()
}

/// 恒星附近天体：四轴 + 行星 + 福点，角距 ≤ 5° 时列出。
fn nearby_body_items(star_long: f64, bodies: &[&Planet]) -> Vec<String> {
    bodies
        .iter()
        .filter_map(|p| {
            let diff = angular_distance(p.long, star_long);
            (diff <= 5.0).then(|| {
                let (d, m, s) = degree_to_dms(diff);
                format!("{} {}° {}' {}", planet_glyph(p.name), d, m, s)
            })
        })
        .collect()
}

#[component]
pub fn Detail(horoscope: Horoscope) -> impl IntoView {
    let h = horoscope;

    let solar_vision = part_of_solar_vision(&h);
    let lunar_vision = part_of_lunar_vision(&h);

    // 恒星附近天体候选：四轴 + 行星 + 福点
    let angular_bodies: Vec<&Planet> = [&h.asc, &h.mc, &h.dsc, &h.ic, &h.part_of_fortune]
        .into_iter()
        .chain(h.planets.iter())
        .collect();

    view! {
        <div class=style::detail_grid>
            <PlanetPower horoscope=h.clone()/>
            <Reception horoscope=h.clone()/>
            <Temperament horoscope=h.clone()/>

            <div class=card::card>
                <h3 class=style::section_title>"映点"</h3>
                <ul class=style::plain_list>{antiscia_items(&h.antiscoins)}</ul>
            </div>

            <div class=card::card>
                <h3 class=style::section_title>"反映点"</h3>
                <ul class=style::plain_list>{antiscia_items(&h.contraantiscias)}</ul>
            </div>

            <div class=card::card>
                <h3 class=style::section_title>"视力点"</h3>
                <p class=style::vision_line>
                    "视力点( "<span class=style::glyph>{planet_glyph(PlanetName::Sun)}</span>")："
                    {vision_view(solar_vision)}
                    <br/>
                    <small class=style::formula>"计算公式：ASC + 太阳黄道经度 - 火星黄道经度"</small>
                </p>
                <p class=style::vision_line>
                    "视力点( "<span class=style::glyph>{planet_glyph(PlanetName::Moon)}</span>")："
                    {vision_view(lunar_vision)}
                    <br/>
                    <small class=style::formula>"计算公式：ASC + 月亮黄道经度 - 土星黄道经度"</small>
                </p>
            </div>

            <div class=stylance::classes!(card::card, style::star_section)>
                <h3 class=style::section_title>"恒星"</h3>
                <div class=style::star_grid>
                    {h
                        .fixed_stars
                        .iter()
                        .map(|star| {
                            let (d, m, s) = degree_to_dms(star.xiu_degree);
                            let position = zodiac_long(star.long);
                            let (sd, sm, ss) = degree_to_dms(position.degree);
                            let nearby = nearby_body_items(star.long, &angular_bodies);
                            let nearby_text = (!nearby.is_empty())
                                .then(|| format!("附近天体：{}", nearby.join(" ")));
                            view! {
                                <div class=style::star_item>
                                    <h4 class=style::star_name>{star.fixed_star.clone()}</h4>
                                    <p>{star.desc.clone()}</p>
                                    <p>
                                        {format!("{}° ", sd)}
                                        <span class=style::star_glyph>{zodiac_glyph(position.zodiac)}</span>
                                        {format!(" {}'{}", sm, ss)}
                                    </p>
                                    <p>{format!("{}宿：{}°{}'{}", star.xiu, d, m, s)}</p>
                                    {nearby_text
                                        .map(|text| {
                                            view! { <p class=style::nearby>{text}</p> }
                                        })}
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}
