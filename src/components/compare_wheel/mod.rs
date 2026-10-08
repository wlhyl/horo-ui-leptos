//! 比较盘双盘轮 SVG 组件（外环比较盘 + 内环原盘）。
use leptos::prelude::*;

use crate::api::response::HoroscopeComparison;
use crate::config::CHART_SVG_SIZE;
use crate::render::compare::build_compare_wheel_svg;

stylance::import_crate_style!(style, "src/components/compare_wheel/compare_wheel.module.css");

#[component]
pub fn CompareWheel(comparison: HoroscopeComparison) -> impl IntoView {
    let inner = build_compare_wheel_svg(&comparison, CHART_SVG_SIZE);
    view! { <svg viewBox=format!("0 0 {CHART_SVG_SIZE} {CHART_SVG_SIZE}") class=style::wheel inner_html=inner></svg> }
}
