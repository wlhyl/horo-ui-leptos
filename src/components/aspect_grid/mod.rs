//! 相位网格 SVG 组件。
use leptos::prelude::*;

use crate::api::response::Horoscope;
use crate::render::aspect::build_aspect_svg;
use crate::config::CHART_SVG_SIZE;

stylance::import_crate_style!(style, "src/components/aspect_grid/aspect_grid.module.css");

#[component]
pub fn AspectGrid(horoscope: Horoscope) -> impl IntoView {
    let inner = build_aspect_svg(&horoscope, CHART_SVG_SIZE);
    view! { <svg viewBox=format!("0 0 {CHART_SVG_SIZE} {CHART_SVG_SIZE}") class=style::aspect_svg inner_html=inner></svg> }
}
