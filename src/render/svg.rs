//! SVG 绘制基元与共享主题色（平移自原 horo-ui 的 fabric 绘图逻辑）。
//!
//! 角度约定：标准数学角，0°=右，90°=上（SVG 的 y 轴向下，故 y = cy - r·sin）。
//! 颜色采用浅色主题，保证在白底上清晰可读。

/// 主描边色。
pub const STROKE: &str = "#5b6472";
/// 金色：外框、星座符号、出相相位。
pub const GOLD: &str = "#b8860b";
/// 靛蓝：内圆、入相相位。
pub const INDIGO: &str = "#4f5bd5";
/// 正文色。
pub const TEXT: &str = "#1f2430";
/// 次要文字色。
pub const MUTE: &str = "#5b6472";

/// 极坐标 -> 直角坐标。
pub fn to_xy(cx: f64, cy: f64, r: f64, angle_deg: f64) -> (f64, f64) {
    let a = angle_deg.to_radians();
    (cx + r * a.cos(), cy - r * a.sin())
}

pub fn svg_line(x1: f64, y1: f64, x2: f64, y2: f64, dash: bool, stroke: &str, w: f64) -> String {
    let d = if dash {
        " stroke-dasharray=\"3,2\""
    } else {
        ""
    };
    format!(
        "<line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" stroke=\"{stroke}\" stroke-width=\"{w:.2}\"{d}/>"
    )
}

pub fn svg_circle(cx: f64, cy: f64, r: f64, fill: &str, stroke: &str, w: f64) -> String {
    format!(
        "<circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{r:.2}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{w:.2}\"/>"
    )
}

pub fn svg_text(
    x: f64,
    y: f64,
    size: f64,
    content: &str,
    fill: &str,
    anchor: &str,
    weight: u32,
) -> String {
    format!(
        "<text x=\"{x:.2}\" y=\"{y:.2}\" font-size=\"{size:.2}\" fill=\"{fill}\" text-anchor=\"{anchor}\" font-weight=\"{weight}\" dominant-baseline=\"central\">{content}</text>"
    )
}

/// 牛顿迭代求指示线起点（从文字位置指向真实圆周点的交点）。
pub fn solve_t(xt: f64, yt: f64, xc: f64, yc: f64, cx: f64, cy: f64, r: f64) -> f64 {
    let mut t = 0.0;
    for _ in 0..14 {
        let fx = (xc - xt) * t + xt - cx;
        let fy = (yc - yt) * t + yt - cy;
        let f = fx * fx + fy * fy - r * r;
        let df = 2.0 * fx * (xc - xt) + 2.0 * fy * (yc - yt);
        if df.abs() < 1e-9 {
            break;
        }
        t -= f / df;
        if t < 0.0 {
            t = 0.0;
        }
    }
    t
}
