//! 接纳 / 互融子组件。
//! 对齐 horo-ui 的 reception.component(.html)。
use leptos::prelude::*;

use crate::api::response::Horoscope;
use crate::astro::dignity::DignityKind;
use crate::astro::horo_math::degree_to_dms;
use crate::astro::reception::{calculate_mutual_receptions, calculate_receptions};
use crate::render::glyphs::{aspect_glyph, planet_glyph};

stylance::import_crate_style!(#[allow(dead_code)] style, "src/components/detail/detail.module.css");
stylance::import_crate_style!(#[allow(dead_code)] card, "src/shared/card.module.css");

/// 相位容许度格式：d°mm'ss"（分秒两位补零）。
fn format_orb(degree: f64) -> String {
    let (d, m, s) = degree_to_dms(degree);
    format!("{}°{:02}'{:02}\"", d, m, s)
}

fn dignity_tag_spans(dignities: &[DignityKind]) -> impl IntoView + use<> {
    dignities
        .iter()
        .map(|dk| view! { <span class=style::dignity_tag>{dk.label()}</span> })
        .collect::<Vec<_>>()
}

#[component]
pub fn Reception(horoscope: Horoscope) -> impl IntoView {
    let receptions = calculate_receptions(&horoscope);
    let mutuals = calculate_mutual_receptions(&horoscope);

    view! {
        <div class=card::card>
            <h3 class=style::section_title>"接纳"</h3>
            {receptions.is_empty().then(|| view! { <p class=style::empty_note>"无接纳关系"</p> })}
            {(!receptions.is_empty()).then(|| {
                view! {
                    <ul class=style::plain_list>
                        {receptions
                            .into_iter()
                            .map(|r| {
                                let orb = format_orb(r.aspect.d);
                                let tags = dignity_tag_spans(&r.dignities);
                                view! {
                                    <li>
                                        <span class=style::glyph>{planet_glyph(r.receiver)}</span>
                                        " 接纳 "
                                        <span class=style::glyph>{planet_glyph(r.received)}</span>
                                        " "
                                        <span class=style::glyph>{aspect_glyph(r.aspect.aspect_value)}</span>
                                        <span class=style::orb>{orb}</span>
                                        {tags}
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </ul>
                }
            })}
        </div>

        <div class=card::card>
            <h3 class=style::section_title>"互融"</h3>
            {mutuals.is_empty().then(|| view! { <p class=style::empty_note>"无互融关系"</p> })}
            {(!mutuals.is_empty()).then(|| {
                view! {
                    <ul class=style::plain_list>
                        {mutuals
                            .into_iter()
                            .map(|m| {
                                let a_tags = dignity_tag_spans(&m.a_dignities);
                                let b_tags = dignity_tag_spans(&m.b_dignities);
                                view! {
                                    <li>
                                        <span class=style::glyph>{planet_glyph(m.a)}</span>
                                        {a_tags}
                                        <span class=style::mutual_arrow>"↔"</span>
                                        <span class=style::glyph>{planet_glyph(m.b)}</span>
                                        {b_tags}
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </ul>
                }
            })}
        </div>
    }
}
