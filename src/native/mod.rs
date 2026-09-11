pub(crate) mod chart;
pub(crate) mod input;

pub use chart::Chart;
pub use input::Input;

/// 星盘类型：本命 / 天象。输入页与结果页按此取用对应的输入数据。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChartMode {
    Native,
    Event,
}
