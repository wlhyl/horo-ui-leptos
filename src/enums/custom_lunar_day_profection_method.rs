use serde::{Deserialize, Serialize};

/// 自定义月返日小限行星推运度数算法（对应原版 CustomLunarDayProfectionMethod）。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CustomLunarDayProfectionMethod {
    /// 太阳基准：日小限推进速率均匀平滑
    Sun,
    /// 月亮基准：推进速率随月亮真实速度波动
    Moon,
}
