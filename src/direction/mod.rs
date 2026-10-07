//! 方向推运（移植自 horo-ui 的 process.page + direction.component）。
//!
//! 三种模式共用一套结果视图：主向推运（Direction）/ 每日回归方向弧（DailyDirection）/
//! 太阳弧（SolarArc）；计算全部在后端 horo-api，前端只组装请求、过滤与渲染。
//! 输入页（/process）写回 HoroStorage 的推运参数，结果页按快照只读渲染。

pub(crate) mod input;
pub(crate) mod utils;
pub(crate) mod view;

pub(crate) use input::ProcessInput;
pub(crate) use view::{DirectionPage, DirectionView};
