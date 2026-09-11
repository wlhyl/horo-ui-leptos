//! 占星计算层：无状态、无 IO、无渲染产物的纯计算。
//!
//! 准入规则：只放与视图无关的领域算法（黄经换算、宫位、推运、星曜力量等）。
//! 需要产出 SVG 的绘制逻辑一律放 `crate::render`。

pub(crate) mod dignity;
pub(crate) mod horo_math;
pub(crate) mod planet_power;
pub(crate) mod reception;
pub(crate) mod temperament;
