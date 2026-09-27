//! 行星力量表页面：列出 12 星座的先天尊贵主星（庙 / 旺 / 三分 / 界 / 面 / 陷 / 落）。
//! 移植自 horo-ui 的 `power.page.ts` / `power.page.html`，符号改用 Unicode glyph 渲染。
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::astro::dignity::{
    detriment, egyptian_terms, exaltation, fall, face_lords, ptolemy_terms, rulership, triplicity,
    triplicity_of_lily,
};
use crate::enums::planet::PlanetName;
use crate::enums::zodiac::Zodiac;
use crate::render::glyphs::{planet_color, planet_glyph, zodiac_glyph};
use crate::routes::AppRoute;

stylance::import_crate_style!(style, "src/power/power.module.css");

/// 力量表体系：本命所用（标准三分 + 埃及界） / Lily所用（Lily 三分 + 托勒密界）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum PowerMode {
    Natal,
    Horary,
}

/// 行星 glyph 单元格：按行星着色，强制文字呈现（避免被当彩色 emoji）。
fn planet_cell(p: PlanetName) -> impl IntoView {
    view! {
        <span class=style::glyph style=format!("color: {}", planet_color(p))>
            {planet_glyph(p)}
        </span>
    }
}

/// 可空行星 glyph 单元格（无旺 / 无落时留空占位，保持列对齐）。
fn planet_cell_opt(p: Option<PlanetName>) -> impl IntoView {
    view! {
        <span class=style::glyph>{p.map(planet_glyph).unwrap_or("")}</span>
    }
}

#[component]
pub(crate) fn Power() -> impl IntoView {
    let (mode, set_mode) = signal(PowerMode::Natal);
    let nav = use_navigate();

    let go_back = move |_| {
        nav(AppRoute::Home.path(), NavigateOptions::default());
    };

    view! {
        <div class=style::page>
            <header class=style::header>
                <button class=style::back_btn on:click=go_back aria-label="返回">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M15 18l-6-6 6-6"/>
                    </svg>
                </button>
                <h1 class=style::title>"行星力量表"</h1>
            </header>

            <div class=style::segment>
                <button
                    class=move || {
                        if mode.get() == PowerMode::Natal {
                            style::segment_btn_active
                        } else {
                            style::segment_btn
                        }
                    }
                    on:click=move |_| set_mode.set(PowerMode::Natal)>
                    "本命所用"
                </button>
                <button
                    class=move || {
                        if mode.get() == PowerMode::Horary {
                            style::segment_btn_active
                        } else {
                            style::segment_btn
                        }
                    }
                    on:click=move |_| set_mode.set(PowerMode::Horary)>
                    "Lily所用"
                </button>
            </div>

            <div class=style::table_card>
                <div class=style::table_scroll>
                    {move || {
                        let is_natal = mode.get() == PowerMode::Natal;
                        let triplicity_span = if is_natal { "3" } else { "2" };
                        view! {
                            <table class=style::table>
                                <thead>
                                    <tr>
                                        <th class=style::th_zodiac>""></th>
                                        <th>"庙"</th>
                                        <th>"旺"</th>
                                        <th colspan=triplicity_span>"三分主星"</th>
                                        <th colspan="5">"界"</th>
                                        <th colspan="3">"面"</th>
                                        <th>"陷"</th>
                                        <th>"落"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {Zodiac::ALL
                                        .iter()
                                        .copied()
                                        .map(|z| {
                                            let ruler = rulership(z);
                                            let exal = exaltation(z);
                                            let tri: Vec<PlanetName> = if is_natal {
                                                triplicity(z).to_vec()
                                            } else {
                                                triplicity_of_lily(z).to_vec()
                                            };
                                            let terms: Vec<(PlanetName, u8)> = if is_natal {
                                                egyptian_terms(z).to_vec()
                                            } else {
                                                ptolemy_terms(z).to_vec()
                                            };
                                            let faces = face_lords(z);
                                            let detr = detriment(z);
                                            let fal = fall(z);
                                            view! {
                                                <tr>
                                                    <td class=style::td_zodiac>
                                                        <span class=style::glyph style="color: var(--gold)">
                                                            {zodiac_glyph(z)}
                                                        </span>
                                                    </td>
                                                    <td>{planet_cell(ruler)}</td>
                                                    <td>{planet_cell_opt(exal)}</td>
                                                    {tri
                                                        .into_iter()
                                                        .map(|p| view! { <td>{planet_cell(p)}</td> })
                                                        .collect::<Vec<_>>()}
                                                    {terms
                                                        .into_iter()
                                                        .map(|(p, d)| {
                                                            view! {
                                                                <td>
                                                                    {planet_cell(p)}
                                                                    <span class=style::deg>{d.to_string()}</span>
                                                                </td>
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()}
                                                    {faces
                                                        .into_iter()
                                                        .enumerate()
                                                        .map(|(i, p)| {
                                                            view! {
                                                                <td>
                                                                    {planet_cell(p)}
                                                                    <span class=style::deg>
                                                                        {(i + 1) * 10}
                                                                    </span>
                                                                </td>
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()}
                                                    <td>{planet_cell(detr)}</td>
                                                    <td>{planet_cell_opt(fal)}</td>
                                                </tr>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
