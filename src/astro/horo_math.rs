//! 星盘数学计算（参考原 horo-ui 的 `utils/horo-math`）。
//! 仅包含与视图无关的黄经/宫位纯计算，供绘制层与组件明细列表共用。

use crate::api::response::{Horoscope, Planet};
use crate::enums::zodiac::Zodiac;

/// 归一化到 [0, 360)。
pub fn deg_norm(mut x: f64) -> f64 {
    x %= 360.0;
    if x < 0.0 {
        x += 360.0;
    }
    x
}

/// 星座与黄经位置（星座 + 星座内角度）。
pub struct ZodiacPosition {
    pub zodiac: Zodiac,
    pub degree: f64,
}

/// 黄经 -> 星座位置。
pub fn zodiac_long(long: f64) -> ZodiacPosition {
    let long = deg_norm(long);
    let sign = (long / 30.0).floor() as u8;
    ZodiacPosition {
        zodiac: Zodiac::from_index(sign),
        degree: long - f64::from(sign) * 30.0,
    }
}

/// 角度 -> (度, 分, 秒)，截断秒并保留各分量的正负号，不归一化。
/// 用于常见黄经范围，整数度数的绝对值须不超过 i16::MAX。
pub fn degree_to_dms(degree: f64) -> (i16, i8, i8) {
    let long = degree.abs();
    let deg = long.floor() as i16;
    let min = (long * 60.0 - f64::from(deg) * 60.0).floor() as i8;
    let sec = (long * 3600.0 - f64::from(deg) * 3600.0 - f64::from(min) * 60.0).floor() as i8;
    if degree >= 0.0 {
        (deg, min, sec)
    } else {
        (-deg, -min, -sec)
    }
}

/// 所有需要绘制的星体（七行星 + 南北交 + 四轴 + 福点），均为借用。
pub fn bodies(h: &Horoscope) -> Vec<&Planet> {
    let mut v: Vec<&Planet> = h.planets.iter().collect();
    v.extend([&h.asc, &h.mc, &h.dsc, &h.ic, &h.part_of_fortune]);
    v
}

/// 两黄经的角距（0°=重合，180°=相对）。
pub fn angular_distance(a: f64, b: f64) -> f64 {
    let diff = deg_norm(a - b);
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// 黄经是否落在环形区间 [start, end) 内（start > end 时跨 0° 判定）。
pub fn in_circular_range(long: f64, start: f64, end: f64) -> bool {
    if start <= end {
        long >= start && long < end
    } else {
        long >= start || long < end
    }
}

/// 行星力量宫位判定的 5 度偏移（第 n 宫区间为 [cusp_n - 5, cusp_{n+1} - 5)）。
const HOUSE_OFFSET: f64 = 5.0;

/// 调整宫位边界：若 cusp - offset 跨越星座边界，则取 cusp 所在星座的 0 度。
fn adjust_cusp_boundary(cusp: f64, offset: f64) -> f64 {
    let raw = deg_norm(cusp - offset);
    let cusp_sign = zodiac_long(cusp).zodiac;
    let raw_sign = zodiac_long(raw).zodiac;
    if raw_sign == cusp_sign {
        raw
    } else {
        f64::from(cusp_sign as u8) * 30.0
    }
}

/// 计算行星所在宫位（1-12），使用 5 度规则（参考 horo-ui 的 house-placement）。
pub fn get_planet_house(planet_long: f64, cusps: &[f64]) -> Result<u8, String> {
    if cusps.len() != 12 {
        return Err(format!(
            "数据错误：宫头数量必须为 12，当前为 {}，无法判定宫位",
            cusps.len()
        ));
    }
    let normalized_long = deg_norm(planet_long);
    // 每个宫头只计算一次调整后的边界并缓存：第 n 次迭代的 b 即下次迭代的 a，
    // 原循环会把同一宫头重复计算两次。
    let adjusted: [f64; 12] = core::array::from_fn(|i| adjust_cusp_boundary(cusps[i], HOUSE_OFFSET));
    for n in 0..12 {
        let a0 = adjusted[n];
        let b0 = adjusted[(n + 1) % 12];
        if in_circular_range(normalized_long, a0, b0) {
            return Ok((n + 1) as u8);
        }
    }
    Err(format!(
        "数据错误：12 个宫头未覆盖完整黄道，行星经度 {normalized_long} 无法判定宫位"
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        angular_distance, degree_to_dms, get_planet_house, in_circular_range, zodiac_long, Zodiac,
    };

    #[test]
    fn zodiac_long_normalizes_longitude() {
        for (long, zodiac, degree) in [
            (390.0, Zodiac::Taurus, 0.0),
            (-30.0, Zodiac::Pisces, 0.0),
            (360.0, Zodiac::Aries, 0.0),
            (29.5, Zodiac::Aries, 29.5),
            (30.0, Zodiac::Taurus, 0.0),
        ] {
            let position = zodiac_long(long);
            assert_eq!(position.zodiac, zodiac);
            assert_eq!(position.degree, degree);
        }
    }

    #[test]
    fn degree_to_dms_truncates_minutes_and_seconds() {
        assert_eq!(degree_to_dms(1.2), (1, 12, 0));
        assert_eq!(degree_to_dms(1.9999), (1, 59, 59));
    }

    #[test]
    fn degree_to_dms_preserves_sign_and_full_degrees() {
        assert_eq!(degree_to_dms(359.5), (359, 30, 0));
        assert_eq!(degree_to_dms(390.0), (390, 0, 0));
        assert_eq!(degree_to_dms(-1.9999), (-1, -59, -59));
        assert_eq!(degree_to_dms(-0.5), (0, -30, 0));
    }

    #[test]
    fn angular_distance_takes_shorter_arc() {
        assert_eq!(angular_distance(10.0, 10.0), 0.0);
        assert_eq!(angular_distance(10.0, 190.0), 180.0);
        assert_eq!(angular_distance(0.0, 350.0), 10.0);
        assert_eq!(angular_distance(350.0, 0.0), 10.0);
    }

    #[test]
    fn in_circular_range_handles_wrap() {
        assert!(in_circular_range(15.0, 10.0, 20.0));
        assert!(!in_circular_range(25.0, 10.0, 20.0));
        // start 为闭端点
        assert!(in_circular_range(10.0, 10.0, 20.0));
        assert!(in_circular_range(5.0, 350.0, 10.0));
        assert!(in_circular_range(355.0, 350.0, 10.0));
        assert!(!in_circular_range(20.0, 350.0, 10.0));
    }

    #[test]
    fn get_planet_house_uses_five_degree_rule() {
        // 宫头落在星座中部（10°）：cusp-5 不跨星座，5 度规则正常生效
        let cusps: Vec<f64> = (0..12u8).map(|i| 10.0 + f64::from(i) * 30.0).collect();
        // 第 1 宫区间 [10-5, 40-5) = [5, 35)
        assert_eq!(get_planet_house(5.0, &cusps), Ok(1));
        assert_eq!(get_planet_house(34.9, &cusps), Ok(1));
        // 35° 起归第 2 宫
        assert_eq!(get_planet_house(35.0, &cusps), Ok(2));
        // 3° 在第 1 宫起点前 → 落第 12 宫区间 [335, 5)
        assert_eq!(get_planet_house(3.0, &cusps), Ok(12));
        // 宫头恰在星座 0°（0°=白羊 0°）：回拉跨星座 → 边界取星座 0°=宫头本身，
        // 5 度回拉失效，第 1 宫区间 [0, 30)，30° 起归第 2 宫
        let whole: Vec<f64> = (0..12u8).map(|i| f64::from(i) * 30.0).collect();
        assert_eq!(get_planet_house(0.0, &whole), Ok(1));
        assert_eq!(get_planet_house(25.0, &whole), Ok(1));
        assert_eq!(get_planet_house(30.0, &whole), Ok(2));
        // 359° 落第 12 宫区间 [330, 360)
        assert_eq!(get_planet_house(359.0, &whole), Ok(12));
        // 跨 0 环形区间：cusp1=350, cusp2=20 → 区间 [345, 15)
        let cusps2 = [
            350.0, 20.0, 50.0, 80.0, 110.0, 140.0, 170.0, 200.0, 230.0, 260.0, 290.0, 320.0,
        ];
        assert_eq!(get_planet_house(355.0, &cusps2), Ok(1));
        assert_eq!(get_planet_house(5.0, &cusps2), Ok(1));
        assert_eq!(get_planet_house(25.0, &cusps2), Ok(2));
    }
}
