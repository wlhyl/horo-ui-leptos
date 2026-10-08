//! 比较盘相位矩阵 SVG 组件（列：比较盘 p0 横看；行：原盘 p1 竖看）。
use leptos::prelude::*;

use crate::api::response::HoroscopeComparison;
use crate::config::CHART_SVG_SIZE;
use crate::render::compare::build_compare_aspect_svg;

stylance::import_crate_style!(
    style,
    "src/components/compare_aspect_grid/compare_aspect_grid.module.css"
);

#[component]
pub fn CompareAspectGrid(comparison: HoroscopeComparison) -> impl IntoView {
    let inner = build_compare_aspect_svg(&comparison, CHART_SVG_SIZE);
    view! { <svg viewBox=format!("0 0 {CHART_SVG_SIZE} {CHART_SVG_SIZE}") class=style::aspect_svg inner_html=inner></svg> }
}
