pub(crate) mod chart;
pub(crate) mod input;

pub use chart::Chart;
pub use input::Input;

/// 星盘类型：本命 / 天象 / 衍生。输入页与结果页按此取用对应的输入数据与请求分支。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChartMode {
    Native,
    Event,
    /// 衍生盘：以出生数据为基准，基准行星的斜升（OA）为中天。
    Derived,
}
