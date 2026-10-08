//! 比较盘 SVG 生成（对应原 horo-ui 的 `utils/image/compare.ts`）。
//!
//! 双盘轮：外层黄道环带与宫位线取原盘（houses_cusps），比较盘行星画在
//! 外侧轨道（r1），原盘行星画在内圆（r2 = r1×1.6/3）之内；两组行星各自
//! 按黄经排序并做避让散开（外环 8°、内环 12°，对齐原版 w 参数）。
//! 相位矩阵：列（底部标签）为比较盘行星 p0，行（左侧标签）为原盘行星 p1，
//! 对齐原版「比较盘行星横看、原盘行星竖看」的读图约定。

use crate::api::response::{HoroscopeComparison, Planet};
use crate::astro::horo_math::{deg_norm, degree_to_dms, zodiac_long};
use crate::config::GRID_PLANETS;
use crate::render::glyphs::{
    aspect_glyph, planet_color, planet_glyph, visible_len, zodiac_glyph,
};
use crate::render::svg::{
    solve_t, svg_circle, svg_line, svg_text, to_xy, GOLD, INDIGO, MUTE, STROKE, TEXT,
};

/// 生成比较盘双盘轮 SVG 的内部节点字符串（viewBox 为 0 0 size size）。
pub fn build_compare_wheel_svg(h: &HoroscopeComparison, size: f64) -> String {
    let mut s = String::new();
    let cx = size / 2.0;
    let cy = size / 2.0;
    let r0 = size / 2.0; // 外圆半径（金色外框）
    let ring = size * 0.09; // 黄道环带宽度
    let r1 = r0 - ring; // 内圆半径（比较盘行星轨道）
    let r2 = r1 * 1.6 / 3.0; // 内圆（原盘行星轨道，对齐原版 r1*1.6/3）

    s.push_str(&svg_circle(cx, cy, r0, "none", GOLD, 2.0));
    s.push_str(&svg_circle(cx, cy, r1, "none", INDIGO, 1.5));
    s.push_str(&svg_circle(cx, cy, r2, "none", INDIGO, 1.5));

    // 宫位线 + 宫号 + 环带刻度：全部取原盘宫头（对齐原版 houses_cusps）
    let cusps = &h.houses_cusps;
    let first = cusps[0];
    for (i, &cusp) in cusps.iter().enumerate() {
        let cup = deg_norm(cusp + 180.0 - first);
        let (x, y) = to_xy(cx, cy, r1, cup);
        s.push_str(&svg_line(cx, cy, x, y, false, STROKE, 1.0));

        // 宫位号（靠近圆心，落在内圆之内）
        let d = deg_norm(cusps[(i + 1) % cusps.len()] - cusp);
        let (hx, hy) = to_xy(cx, cy, r2 * 0.45, cup + d / 2.0);
        s.push_str(&svg_text(
            hx,
            hy,
            size * 0.022,
            &format!("{}", i + 1),
            MUTE,
            "middle",
            400,
        ));

        // 宫头所在星座符号（环带中部）
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

    // 比较盘行星（外环 r1，避让 8°；字号与单盘一致）
    let comparison: Vec<&Planet> = h
        .comparison_planets
        .iter()
        .chain([
            &h.comparison_asc,
            &h.comparison_mc,
            &h.comparison_dsc,
            &h.comparison_ic,
            &h.comparison_part_of_fortune,
        ])
        .collect();
    draw_ring(
        &mut s,
        &comparison,
        first,
        cx,
        cy,
        r1,
        8.0,
        size * 0.05,
        size * 0.026,
        size * 0.024,
        size * 0.024,
    );

    // 原盘行星（内环 r2，避让 12°；字号按内环半径等比缩小）
    let original: Vec<&Planet> = h
        .original_planets
        .iter()
        .chain([
            &h.original_asc,
            &h.original_mc,
            &h.original_dsc,
            &h.original_ic,
            &h.original_part_of_fortune,
        ])
        .collect();
    draw_ring(
        &mut s,
        &original,
        first,
        cx,
        cy,
        r2,
        12.0,
        r2 * 0.19,
        r2 * 0.11,
        r2 * 0.095,
        r2 * 0.095,
    );

    // 左上角说明
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
    s.push_str(&svg_text(8.0, ly, fs, "外环：比较盘", TEXT, "start", 500));
    ly += fs * 1.3;
    s.push_str(&svg_text(8.0, ly, fs, "内环：原盘", TEXT, "start", 500));

    s
}

/// 在半径 `r` 的轨道上绘制一组行星（符号 + 度数 / 星座 / 分 + 逆行标记）。
///
/// `spread` 为避让散开的字符间宽度（度）；行星真实位置在轨道上画指示线，
/// 文字沿轨道散开（与单盘轮 `build_wheel_svg` 同一算法）。
fn draw_ring(
    s: &mut String,
    planets: &[&Planet],
    first_cusp: f64,
    cx: f64,
    cy: f64,
    r: f64,
    spread: f64,
    glyph_size_single: f64,
    glyph_size_multi: f64,
    detail_size: f64,
    retro_size: f64,
) {
    let mut ps = planets.to_vec();
    ps.sort_by(|a, b| deg_norm(a.long).total_cmp(&deg_norm(b.long)));

    let n = ps.len();
    let mut p: Vec<f64> = ps
        .iter()
        .map(|b| deg_norm(b.long + 180.0 - first_cusp))
        .collect();
    for i in 0..n {
        let mut nn = n;
        for j in 1..n {
            if deg_norm(p[(i + j) % n] - p[i]) >= spread * j as f64 {
                nn = j;
                break;
            }
        }
        for j in 1..nn {
            p[(i + j) % n] = deg_norm(p[i] + spread * j as f64);
        }
    }

    for (k, &b) in ps.iter().enumerate() {
        let angle = p[k];
        let original_angle = b.long + 180.0 - first_cusp;
        let (xt, yt) = to_xy(cx, cy, r * 8.0 / 9.0, angle);
        let (xc, yc) = to_xy(cx, cy, r, original_angle);
        let t = solve_t(xt, yt, xc, yc, cx, cy, r * 8.4 / 9.0);
        let (x0, y0) = (xt + (xc - xt) * t, yt + (yc - yt) * t);
        s.push_str(&svg_line(x0, y0, xc, yc, true, STROKE, 1.0));

        let g = planet_glyph(b.name);
        let size_g = if visible_len(g) > 1 {
            glyph_size_multi
        } else {
            glyph_size_single
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
        let (px, py) = to_xy(cx, cy, r * 7.0 / 9.0, angle);
        s.push_str(&svg_text(
            px,
            py,
            detail_size,
            &format!("{}°", pd),
            MUTE,
            "middle",
            400,
        ));
        let (sx, sy) = to_xy(cx, cy, r * 6.5 / 9.0, angle);
        s.push_str(&svg_text(
            sx,
            sy,
            detail_size,
            zodiac_glyph(position.zodiac),
            GOLD,
            "middle",
            600,
        ));
        let (mx, my) = to_xy(cx, cy, r * 6.0 / 9.0, angle);
        s.push_str(&svg_text(
            mx,
            my,
            detail_size,
            &format!("{}'", pm),
            MUTE,
            "middle",
            400,
        ));
        if b.speed < 0.0 {
            let (rx, ry) = to_xy(cx, cy, r * 5.5 / 9.0, angle);
            s.push_str(&svg_text(
                rx,
                ry,
                retro_size,
                "℞\u{FE0E}",
                "#d63b3b",
                "middle",
                700,
            ));
        }
    }
}

/// 生成比较盘相位矩阵 SVG 的内部节点字符串。
///
/// 行（左侧标签）为原盘行星 p1，列（底部标签）为比较盘行星 p0；
/// 不做单盘的下三角归一化——两盘星体分属不同星盘，必须保持 p0/p1 方向。
pub fn build_compare_aspect_svg(h: &HoroscopeComparison, size: f64) -> String {
    let mut s = String::new();
    let planets = GRID_PLANETS;
    let col = planets.len() + 1;
    let w = size;
    let ht = size;
    let fs = size / col as f64; // 单元格边长，同时作为符号/文字的字号基准

    // 网格横线（含上下边）：从第一列右侧画到最右
    for i in 0..col {
        let y = i as f64 * ht / col as f64;
        s.push_str(&svg_line(w / col as f64, y, w, y, false, STROKE, 1.0));
    }
    // 网格竖线（含左右边）：不进入底部标签行
    for i in 0..col {
        let x = (i + 1) as f64 * w / col as f64;
        s.push_str(&svg_line(
            x,
            0.0,
            x,
            ht - ht / col as f64,
            false,
            STROKE,
            1.0,
        ));
    }

    // 行星名：左列（原盘 p1）+ 底行（比较盘 p0）
    for (i, &pl) in planets.iter().enumerate() {
        let g = planet_glyph(pl);
        let mut gfs = fs;
        let glen = visible_len(g);
        if glen > 1 {
            gfs /= glen as f64;
        } else {
            gfs *= 0.8;
        }

        // 左列：cx = w/col/2，cy = ht/row*(i + 0.5)
        let cx0 = w / col as f64 / 2.0;
        let cy0 = (ht / col as f64) * (i as f64 + 0.5);
        s.push_str(&svg_text(cx0, cy0, gfs, g, planet_color(pl), "middle", 600));

        // 底行：cx = w/col*(1.5 + i)，cy = ht - ht/row/2
        let cx1 = (w / col as f64) * (1.5 + i as f64);
        let cy1 = ht - (ht / col as f64) * 0.5;
        s.push_str(&svg_text(cx1, cy1, gfs, g, planet_color(pl), "middle", 600));
    }

    // 相位符号与度数：p0 定列（横看，比较盘）、p1 定行（竖看，原盘）
    for a in &h.aspects {
        let (Some(ic), Some(ir)) = (
            planets.iter().position(|&p| p == a.p0),
            planets.iter().position(|&p| p == a.p1),
        ) else {
            continue;
        };
        let cx = (w / col as f64) * (1.5 + ic as f64);
        let cy = (ht / col as f64) * (0.5 + ir as f64);
        s.push_str(&svg_text(
            cx,
            cy,
            fs * 0.8,
            aspect_glyph(a.aspect_value),
            if a.apply { INDIGO } else { GOLD },
            "middle",
            600,
        ));

        // Aspect value：度与分之间标注 A（应用）/ S（分离），与单盘一致
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
