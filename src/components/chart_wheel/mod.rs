//! 星盘 SVG 组件。
use leptos::prelude::*;

use crate::api::response::Horoscope;
use crate::render::wheel::build_wheel_svg;
use crate::config::CHART_SVG_SIZE;

stylance::import_crate_style!(style, "src/components/chart_wheel/chart_wheel.module.css");

#[component]
pub fn ChartWheel(horoscope: Horoscope) -> impl IntoView {
    let inner = build_wheel_svg(&horoscope, CHART_SVG_SIZE);
    view! { <svg viewBox=format!("0 0 {CHART_SVG_SIZE} {CHART_SVG_SIZE}") class=style::wheel inner_html=inner></svg> }
}
