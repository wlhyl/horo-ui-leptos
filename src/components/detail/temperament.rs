//! 气质子组件：贡献者表格（可交互勾选 / 删除 / 添加）+ 四体液摘要。
//! 对齐 horo-ui 的 temperament.component(.html)。
use leptos::control_flow::Show;
use leptos::prelude::*;

use crate::api::response::Horoscope;
use crate::astro::temperament::{
    calculate_temperament_contributors, calculate_temperament_summary, create_contributor,
    get_contributor_qualities, ContributorName, GATHERABLE_PLANETS, Quality,
    TemperamentContributor,
};
use crate::components::{SelectGroup, SelectPopup};
use crate::enums::planet::PlanetName;
use crate::enums::zodiac::Zodiac;
use crate::render::glyphs::{planet_glyph, zodiac_glyph};

stylance::import_crate_style!(#[allow(dead_code)] style, "src/components/detail/detail.module.css");
stylance::import_crate_style!(#[allow(dead_code)] card, "src/shared/card.module.css");

/// 贡献者名称的占星符号。
fn contributor_glyph(name: ContributorName) -> &'static str {
    match name {
        ContributorName::Planet(p) => planet_glyph(p),
        ContributorName::Sign(s) => zodiac_glyph(s),
    }
}

/// 四种性质与对应字段的读写。
fn quality_get(c: &TemperamentContributor, q: Quality) -> bool {
    match q {
        Quality::Hot => c.hot,
        Quality::Cold => c.cold,
        Quality::Dry => c.dry,
        Quality::Wet => c.wet,
    }
}

fn quality_set(c: &mut TemperamentContributor, q: Quality, value: bool) {
    match q {
        Quality::Hot => c.hot = value,
        Quality::Cold => c.cold = value,
        Quality::Dry => c.dry = value,
        Quality::Wet => c.wet = value,
    }
}

const QUALITY_KEYS: [Quality; 4] = [Quality::Hot, Quality::Cold, Quality::Dry, Quality::Wet];

/// 全部 12 星座。
const ALL_SIGNS: [Zodiac; 12] = [
    Zodiac::Aries,
    Zodiac::Taurus,
    Zodiac::Gemini,
    Zodiac::Cancer,
    Zodiac::Leo,
    Zodiac::Virgo,
    Zodiac::Libra,
    Zodiac::Scorpio,
    Zodiac::Sagittarius,
    Zodiac::Capricorn,
    Zodiac::Aquarius,
    Zodiac::Pisces,
];

#[component]
pub fn Temperament(horoscope: Horoscope) -> impl IntoView {
    let initial = calculate_temperament_contributors(&horoscope);
    let (cs, err) = match initial {
        Ok(cs) => (cs, String::new()),
        Err(e) => (Vec::new(), e),
    };
    let error = RwSignal::new(err);
    let contributors = RwSignal::new(cs);
    let selected_planet = RwSignal::new(None::<PlanetName>);
    let selected_sign = RwSignal::new(None::<Zodiac>);
    // 信号句柄是 Copy 的：捕获它的处理器闭包因此也是 Copy，
    // 可在 <Show> 的 children（须为 Fn）中重复使用。
    let horo = RwSignal::new(horoscope);

    let summary = Memo::new(move |_| calculate_temperament_summary(&contributors.get()));

    let refresh = move || {
        match calculate_temperament_contributors(&horo.get()) {
            Ok(cs) => {
                error.set(String::new());
                contributors.set(cs);
            }
            Err(e) => {
                error.set(e);
                contributors.set(Vec::new());
            }
        }
        selected_planet.set(None);
        selected_sign.set(None);
    };

    // 可添加的行星 / 星座（排除已在列表中的）
    let available_planets = Memo::new(move |_| {
        let existing = contributors.get();
        GATHERABLE_PLANETS
            .iter()
            .copied()
            .filter(|p| {
                !existing
                    .iter()
                    .any(|c| c.name == ContributorName::Planet(*p))
            })
            .collect::<Vec<_>>()
    });
    let available_signs = Memo::new(move |_| {
        let existing = contributors.get();
        ALL_SIGNS
            .iter()
            .copied()
            .filter(|s| !existing.iter().any(|c| c.name == ContributorName::Sign(*s)))
            .collect::<Vec<_>>()
    });

    let on_add_planet = move |_| {
        let Some(name) = selected_planet.get() else { return };
        match get_contributor_qualities(ContributorName::Planet(name), &horo.get()) {
            Ok(qualities) => {
                error.set(String::new());
                contributors.update(|cs| {
                    cs.push(create_contributor(
                        ContributorName::Planet(name),
                        vec!["手动添加".to_string()],
                        qualities,
                    ))
                });
                selected_planet.set(None);
            }
            Err(e) => error.set(e),
        }
    };
    let on_add_sign = move |_| {
        let Some(name) = selected_sign.get() else { return };
        match get_contributor_qualities(ContributorName::Sign(name), &horo.get()) {
            Ok(qualities) => {
                error.set(String::new());
                contributors.update(|cs| {
                    cs.push(create_contributor(
                        ContributorName::Sign(name),
                        vec!["手动添加".to_string()],
                        qualities,
                    ))
                });
                selected_sign.set(None);
            }
            Err(e) => error.set(e),
        }
    };

    let on_reset = move |_| refresh();

    view! {
        <div class=card::card>
            <h3 class=style::section_title>"气质"</h3>
            <Show when=move || !error.get().is_empty() fallback=|| ()>
                <p class=style::temperament_error>{move || error.get()}</p>
            </Show>
            <Show when=move || !contributors.get().is_empty() fallback=|| ()>
                // 窄屏 7 列放不下：包一层横向滚动，避免列被挤压到行高爆炸
                <div class=style::temperament_scroll>
                    <table class=style::temperament_table>
                        <thead>
                            <tr>
                                <th>"项目"</th>
                                <th>"来源"</th>
                                <th>"热"</th>
                                <th>"冷"</th>
                                <th>"干"</th>
                                <th>"湿"</th>
                                <th></th>
                            </tr>
                        </thead>
                    <tbody>
                        <For each=move || contributors.get() key=|c| c.id.clone() let(c)>
                            <tr>
                                <td class=style::name_cell>
                                    <span class=style::glyph>{contributor_glyph(c.name)}</span>
                                </td>
                                <td class=style::sources_cell>{c.sources.join("、")}</td>
                                {QUALITY_KEYS
                                    .iter()
                                    .map(|q| {
                                        let id_checked = c.id.clone();
                                        let id_change = c.id.clone();
                                        let checked = move || {
                                            contributors
                                                .get()
                                                .iter()
                                                .find(|c| c.id == id_checked)
                                                .map(|c| quality_get(c, *q))
                                                .unwrap_or(false)
                                        };
                                        let on_change = move |ev| {
                                            let value = event_target_checked(&ev);
                                            contributors.update(|cs| {
                                                if let Some(c) = cs
                                                    .iter_mut()
                                                    .find(|c| c.id == id_change)
                                                {
                                                    quality_set(c, *q, value);
                                                }
                                            });
                                        };
                                        view! {
                                            <td class=style::quality_cell>
                                                // label 撑满单元格：整个格子都可点选
                                                <label>
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=checked
                                                        on:change=on_change
                                                    />
                                                </label>
                                            </td>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                                <td class=style::action_cell>
                                    <button
                                        class=style::remove_btn
                                        on:click=move |_| {
                                            let id = c.id.clone();
                                            contributors
                                                .update(|cs| cs.retain(|c| c.id != id));
                                        }
                                    >
                                        "×"
                                    </button>
                                </td>
                            </tr>
                        </For>
                        </tbody>
                    </table>
                </div>

                <div class=style::add_row>
                    <SelectPopup
                        compact=true
                        current=move || selected_planet.get()
                        on_pick=move |p| selected_planet.set(Some(p))
                        groups=move || {
                            vec![SelectGroup { label: "", items: available_planets.get() }]
                        }
                        label=|p| planet_glyph(p).to_string()
                        placeholder="添加行星"
                    />
                    <button
                        class=style::add_btn
                        disabled=move || selected_planet.get().is_none()
                        on:click=on_add_planet
                    >
                        "添加"
                    </button>

                    <SelectPopup
                        compact=true
                        current=move || selected_sign.get()
                        on_pick=move |s| selected_sign.set(Some(s))
                        groups=move || {
                            vec![SelectGroup { label: "", items: available_signs.get() }]
                        }
                        label=|s| zodiac_glyph(s).to_string()
                        placeholder="添加星座"
                    />
                    <button
                        class=style::add_btn
                        disabled=move || selected_sign.get().is_none()
                        on:click=on_add_sign
                    >
                        "添加"
                    </button>
                </div>

                {move || {
                    let s = summary.get();
                    let rows = [
                        (style::humor_sanguine, "多血质", "热情、活泼、乐观、善交际", s.sanguine, s.percentages.sanguine),
                        (style::humor_phlegmatic, "粘液质", "冷静、稳重、迟缓、忍耐", s.phlegmatic, s.percentages.phlegmatic),
                        (style::humor_choleric, "胆汁质", "急躁、果断、精力充沛", s.choleric, s.percentages.choleric),
                        (style::humor_melancholic, "抑郁质", "敏感、深刻、忧郁、细致", s.melancholic, s.percentages.melancholic),
                    ];
                    rows
                        .iter()
                        .map(|(class, label, desc, score, percent)| {
                            view! {
                                <div class=style::humor_row>
                                    <span class=style::humor_label>{*label}</span>
                                    <div class=style::humor_bar_track>
                                        <div
                                            class=stylance::classes!(style::humor_bar_fill, *class)
                                            style=format!("width: {:.1}%", percent)
                                        ></div>
                                    </div>
                                    <span class=style::humor_score>
                                        {format!("{}分 ({:.0}%)", score, percent)}
                                    </span>
                                    <small class=style::humor_desc>{*desc}</small>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}

                <button class=style::reset_btn on:click=on_reset>"重置"</button>
            </Show>
        </div>
    }
}
