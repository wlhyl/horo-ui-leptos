//! 相位网格 SVG 生成（对应原 horo-ui 的相位网格绘制逻辑）。

use crate::api::response::Horoscope;
use crate::astro::horo_math::degree_to_dms;
use crate::config::GRID_PLANETS;
use crate::render::glyphs::{aspect_glyph, planet_color, planet_glyph, visible_len};
use crate::render::svg::{svg_line, svg_text, GOLD, INDIGO, MUTE, STROKE};

/// 生成相位网格 SVG 的内部节点字符串。
pub fn build_aspect_svg(h: &Horoscope, size: f64) -> String {
    let mut s = String::new();
    let planets = GRID_PLANETS;
    let col = planets.len() + 1;
    let row = col;
    let w = size; // 画布宽（viewBox 为 size×size 正方形）
    let ht = size; // 画布高
    let fs = size / col as f64; // 单元格边长，同时作为符号/文字的字号基准

    // 网格横线
    for i in 1..col {
        let x0 = w / row as f64;
        let y0 = i as f64 * ht / col as f64;
        let x1 = w / row as f64 * (i + 1) as f64;
        s.push_str(&svg_line(x0, y0, x1, y0, false, STROKE, 1.0));
        if i == col - 1 {
            let y1 = y0 + ht / col as f64;
            s.push_str(&svg_line(x0, y1, x1, y1, false, STROKE, 1.0));
        }
    }
    // 网格竖线
    for i in 1..row {
        let x0 = (i + 1) as f64 * w / row as f64;
        let y0 = i as f64 * ht / row as f64;
        s.push_str(&svg_line(x0, y0, x0, ht, false, STROKE, 1.0));
        if i == 1 {
            let x1 = w / row as f64;
            s.push_str(&svg_line(x1, y0, x1, ht, false, STROKE, 1.0));
        }
    }

    // 行星名（首列 + 对角线）
    for (i, &pl) in planets.iter().enumerate() {
        let g = planet_glyph(pl);
        let mut gfs = fs;
        let glen = visible_len(g);
        if glen > 1 {
            gfs /= glen as f64;
        } else {
            gfs *= 0.8;
        }

        // 画第一列
        // 第一个字符的中心坐标
        // cx=width / col / 2
        // cy = height / row * 1.5
        // 第n个字符的中心坐标
        // cx = width / col
        // cy = height / row * 1.5 + i * (height / row)
        let cx0 = w / col as f64 / 2.0;
        let cy0 = (ht / row as f64) * (i as f64 + 1.5);
        s.push_str(&svg_text(cx0, cy0, gfs, g, planet_color(pl), "middle", 600));

        // 斜列
        // 第一个字符的中心坐标
        // cx=width / col * 1.5
        // cy = height / row / * 1.5
        // 第n个字符的中心坐标
        // cx = width / col * 1.5 + i * width / col
        // cy = height / row * .5 + i * (height / row)
        let cx1 = (w / col as f64) * (1.5 + i as f64);
        let cy1 = (ht / row as f64) * (i as f64 + 0.5);
        s.push_str(&svg_text(cx1, cy1, gfs, g, planet_color(pl), "middle", 600));
    }

    // 相位符号与度数
    for a in &h.aspects {
        let i0 = planets.iter().position(|&p| p == a.p0);
        let i1 = planets.iter().position(|&p| p == a.p1);

        // 跳过不在网格行星列表中的相位
        let (i0, i1) = match (i0, i1) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };

        // 确保相位符号位于对角线下方（左下三角区域），
        // 无论后端返回的 p0/p1 顺序如何
        let cx = (w / col as f64) * (1.5 + (i0.min(i1)) as f64);
        let cy = (ht / row as f64) * (1.5 + (i0.max(i1)) as f64);
        s.push_str(&svg_text(
            cx,
            cy,
            fs * 0.8,
            aspect_glyph(a.aspect_value),
            if a.apply { INDIGO } else { GOLD },
            "middle",
            600,
        ));

        // Aspect value：度与分之间标注 A（应用）/ S（分离），与原 horo.ts 一致
        let (dd, dm, _) = degree_to_dms(a.d);
        let mark = if a.apply { "A" } else { "S" };
        let val = format!("{}°{}{}'", dd, mark, dm);
        s.push_str(&svg_text(
            cx,
            cy + fs / 2.0 * 3.0 / 4.0,
            fs * 0.28,
            &val,
            MUTE,
            "middle",
            400,
        ));
    }

    s
}
