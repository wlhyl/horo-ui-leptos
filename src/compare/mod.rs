//! 比较盘（移植自 horo-ui 的 compare.component）。
//!
//! 八种模式共用一套结果视图：行运 / 日返·月返·每日回归比本命（及反向）/
//! 次限比本命；计算全部在后端 horo-api，前端只组装请求与渲染。
//! 输入页（/process）写回 HoroStorage 的推运参数，结果页按快照只读渲染。

pub(crate) mod view;

pub(crate) use view::{ComparePage, CompareView};
