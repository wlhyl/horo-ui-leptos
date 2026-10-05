//! 星盘轮 SVG 生成（对应原 horo-ui 的 `utils/image/horo.ts`）。

use crate::api::response::Horoscope;
use crate::astro::horo_math::{bodies, deg_norm, degree_to_dms, zodiac_long};
use crate::render::glyphs::{planet_color, planet_glyph, visible_len, zodiac_glyph};
use crate::render::svg::{
    solve_t, svg_circle, svg_line, svg_text, to_xy, GOLD, INDIGO, MUTE, STROKE, TEXT,
};

/// 生成星盘 SVG 的内部节点字符串（viewBox 为 0 0 size size）。
pub fn build_wheel_svg(h: &Horoscope, size: f64) -> String {
    let mut s = String::new();
    let cx = size / 2.0; // 圆心 x
    let cy = size / 2.0; // 圆心 y
    let r0 = size / 2.0; // 外圆半径（金色外框）
    let ring = size * 0.09; // 黄道环带宽度（外圆与内圆之间，用于绘制星座符号和宫头度数）
    let r1 = r0 - ring; // 内圆半径（靛蓝圆，环带内边界，宫位线和星体刻度都从它出发）

    s.push_str(&svg_circle(cx, cy, r0, "none", GOLD, 2.0));
    s.push_str(&svg_circle(cx, cy, r1, "none", INDIGO, 1.5));

    let first = h.cusps[0];
    for (i, &cusp) in h.cusps.iter().enumerate() {
        let cup = deg_norm(cusp + 180.0 - first);
        let (x, y) = to_xy(cx, cy, r1, cup);
        s.push_str(&svg_line(cx, cy, x, y, false, STROKE, 1.0));

        // 宫位号（靠近圆心）
        let d = deg_norm(h.cusps[(i + 1) % h.cusps.len()] - cusp);
        let (hx, hy) = to_xy(cx, cy, r1 / 8.0, cup + d / 2.0);
        s.push_str(&svg_text(
            hx,
            hy,
            size * 0.028,
            &format!("{}", i + 1),
            MUTE,
            "middle",
            400,
        ));

        // 宫头所在星座符号（环带中部）
        // 字号：环带宽度的 0.55，与两侧的度数/分数字（ring*0.42）视觉协调，
        // 之前用 0.95 会让星座符号明显大于数字、且几乎占满整个环带。
        let (zx, zy) = to_xy(cx, cy, r1 + ring / 2.0, cup);
        let position = zodiac_long(cusp);
        s.push_str(&svg_text(
            zx,
            zy,
            ring * 0.55,
            zodiac_glyph(position.zodiac),
            GOLD,
            "middle",
            600,
        ));

        // 宫头度数 / 分（分列两侧）
        let (cd, cm, _) = degree_to_dms(position.degree);
        let tr = r1 + ring / 2.0;
        let off = 5.0;
        let (dx, dy) = to_xy(cx, cy, tr, if i < 7 { cup - off } else { cup + off });
        s.push_str(&svg_text(
            dx,
            dy,
            ring * 0.42,
            &format!("{}°", cd),
            MUTE,
            "middle",
            400,
        ));
        let (mx, my) = to_xy(cx, cy, tr, if i < 7 { cup + off } else { cup - off });
        s.push_str(&svg_text(
            mx,
            my,
            ring * 0.42,
            &format!("{}'", cm),
            MUTE,
            "middle",
            400,
        ));
    }

    // 星体
    let mut bs = bodies(h);
    bs.sort_by(|a, b| deg_norm(a.long).total_cmp(&deg_norm(b.long)));

    let n = bs.len();
    let mut p: Vec<f64> = bs
        .iter()
        .map(|b| deg_norm(b.long + 180.0 - first))
        .collect();
    let w = 6.0;
    for i in 0..n {
        let mut nn = n;
        for j in 1..n {
            if deg_norm(p[(i + j) % n] - p[i]) >= w * j as f64 {
                nn = j;
                break;
            }
        }
        for j in 1..nn {
            p[(i + j) % n] = deg_norm(p[i] + w * j as f64);
        }
    }
    for (k, &b) in bs.iter().enumerate() {
        let angle = p[k];
        let original_angle = b.long + 180.0 - first;
        let (xt, yt) = to_xy(cx, cy, r1 * 8.0 / 9.0, angle);
        let (xc2, yc2) = to_xy(cx, cy, r1, original_angle);
        let t = solve_t(xt, yt, xc2, yc2, cx, cy, r1 * 8.4 / 9.0);
        let (x0, y0) = (xt + (xc2 - xt) * t, yt + (yc2 - yt) * t);
        s.push_str(&svg_line(x0, y0, xc2, yc2, true, STROKE, 1.0));

        let g = planet_glyph(b.name);
        let size_g = if visible_len(g) > 1 {
            size * 0.026
        } else {
            size * 0.05
        };
        s.push_str(&svg_text(
            xt,
            yt,
            size_g,
            g,
            planet_color(b.name),
            "middle",
            600,
        ));

        let position = zodiac_long(b.long);
        let (pd, pm, _) = degree_to_dms(position.degree);
        let (px, py) = to_xy(cx, cy, r1 * 7.0 / 9.0, angle);
        s.push_str(&svg_text(
            px,
            py,
            size * 0.024,
            &format!("{}°", pd),
            MUTE,
            "middle",
            400,
        ));
        let (sx, sy) = to_xy(cx, cy, r1 * 6.5 / 9.0, angle);
        s.push_str(&svg_text(
            sx,
            sy,
            size * 0.024,
            zodiac_glyph(position.zodiac),
            GOLD,
            "middle",
            600,
        ));
        let (mmx, mmy) = to_xy(cx, cy, r1 * 6.0 / 9.0, angle);
        s.push_str(&svg_text(
            mmx,
            mmy,
            size * 0.024,
            &format!("{}'", pm),
            MUTE,
            "middle",
            400,
        ));
        if b.speed < 0.0 {
            let (rx, ry) = to_xy(cx, cy, r1 * 5.5 / 9.0, angle);
            s.push_str(&svg_text(
                rx,
                ry,
                size * 0.024,
                "℞\u{FE0E}",
                "#d63b3b",
                "middle",
                700,
            ));
        }
    }

    // 左上角说明
    // 说明行数随数据变化（衍生盘不含日主星 / 时主星，对应原版 calculateNotesElements
    // 的 undefined 跳过逻辑），故逐行推进 ly；有条件出现的行受最靠下一行与外圆
    // 左边缘间距的制约，整体比圆盘内文字略小：首行贴近左上角、行距收紧。
    let mut ly = size * 0.024;
    let fs = size * 0.03;
    s.push_str(&svg_text(
        8.0,
        ly,
        fs,
        h.house_name.as_str(),
        TEXT,
        "start",
        500,
    ));
    ly += fs * 1.3;
    s.push_str(&svg_text(
        8.0,
        ly,
        fs,
        if h.is_diurnal {
            "白天盘"
        } else {
            "夜间盘"
        },
        TEXT,
        "start",
        500,
    ));
    ly += fs * 1.3;
    if let Some(day) = h.planetary_day {
        s.push_str(&svg_text(8.0, ly, fs, "日主星:", MUTE, "start", 400));
        s.push_str(&svg_text(
            8.0 + fs * 3.6,
            ly,
            fs,
            planet_glyph(day),
            planet_color(day),
            "start",
            600,
        ));
        ly += fs * 1.3;
    }
    if let Some(hours) = h.planetary_hours {
        s.push_str(&svg_text(8.0, ly, fs, "时主星:", MUTE, "start", 400));
        s.push_str(&svg_text(
            8.0 + fs * 3.6,
            ly,
            fs,
            planet_glyph(hours),
            planet_color(hours),
            "start",
            600,
        ));
    }

    s
}
