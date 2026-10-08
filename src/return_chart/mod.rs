//! 返照盘（移植自 horo-ui 的 return.component）。
//!
//! 三种模式共用一套结果视图：日返（SolarReturn）/ 月返（LunarReturn）/
//! 每日回归（DailyReturn）；计算全部在后端 horo-api，前端只组装请求与渲染。
//! 输入页（/process）写回 HoroStorage 的推运参数，结果页按快照只读渲染。

pub(crate) mod view;

pub(crate) use view::{ReturnPage, ReturnView};
/// 返照时刻请求链（日返→月返→每日回归），比较盘复用。
pub(crate) use view::fetch_return;
