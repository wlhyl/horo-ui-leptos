//! 占星符号映射（Unicode，无需自定义字体）。
//!
//! 符号后统一追加 U+FE0E（VARIATION SELECTOR-15，文本呈现），强制按**文字字形**
//! 渲染。否则 macOS / 部分浏览器会把这些码位当彩色 emoji 处理，出现红色底框并放大。

use crate::enums::planet::PlanetName;
use crate::enums::zodiac::Zodiac;

/// 行星 / 四轴 / 福点的图形符号。
pub fn planet_glyph(name: PlanetName) -> &'static str {
    match name {
        PlanetName::Sun => "☉\u{FE0E}",
        PlanetName::Moon => "☽\u{FE0E}",
        PlanetName::Mercury => "☿\u{FE0E}",
        PlanetName::Venus => "♀\u{FE0E}",
        PlanetName::Mars => "♂\u{FE0E}",
        PlanetName::Jupiter => "♃\u{FE0E}",
        PlanetName::Saturn => "♄\u{FE0E}",
        PlanetName::NorthNode => "☊\u{FE0E}",
        PlanetName::SouthNode => "☋\u{FE0E}",
        PlanetName::ASC => "ASC",
        PlanetName::MC => "MC",
        PlanetName::DSC => "DSC",
        PlanetName::IC => "IC",
        PlanetName::PartOfFortune => "⊕\u{FE0E}",
    }
}

/// 行星着色（浅色主题，保证在白底上清晰可读）。
pub fn planet_color(name: PlanetName) -> &'static str {
    match name {
        PlanetName::Sun => "#b8860b",
        PlanetName::Moon => "#55606f",
        PlanetName::Mercury => "#4f5bd5",
        PlanetName::Venus => "#6a5acd",
        PlanetName::Mars => "#d63b3b",
        PlanetName::Jupiter => "#1a9e57",
        PlanetName::Saturn => "#b8860b",
        PlanetName::NorthNode => "#9b59b6",
        PlanetName::SouthNode => "#6b7280",
        PlanetName::ASC => "#1f2430",
        PlanetName::MC => "#1f2430",
        PlanetName::DSC => "#6b7280",
        PlanetName::IC => "#6b7280",
        PlanetName::PartOfFortune => "#11998e",
    }
}

pub const ZODIAC_GLYPH: [&str; 12] = [
    "♈\u{FE0E}",
    "♉\u{FE0E}",
    "♊\u{FE0E}",
    "♋\u{FE0E}",
    "♌\u{FE0E}",
    "♍\u{FE0E}",
    "♎\u{FE0E}",
    "♏\u{FE0E}",
    "♐\u{FE0E}",
    "♑\u{FE0E}",
    "♒\u{FE0E}",
    "♓\u{FE0E}",
];

pub fn zodiac_glyph(zodiac: Zodiac) -> &'static str {
    ZODIAC_GLYPH[zodiac as usize]
}

/// 可见字符数：忽略用于强制文字呈现的 U+FE0E 变体选择符。
///
/// 直接用 `chars().count()` 会把单个符号也算成 2（符号 + VS15），
/// 导致按字符数区分「单符号 / 多字符文本」的分支永远走不到单符号一侧。
pub fn visible_len(s: &str) -> usize {
    s.chars().filter(|&c| c != '\u{FE0E}').count()
}

/// 相位角度 -> 符号。
pub fn aspect_glyph(v: u8) -> &'static str {
    match v {
        0 => "☌\u{FE0E}",
        // 30 => "⚺\u{FE0E}",
        // 45 => "∠\u{FE0E}",
        60 => "⚹\u{FE0E}",
        // 72 => "⚸\u{FE0E}",
        90 => "□\u{FE0E}",
        120 => "△\u{FE0E}",
        // 135 => "⚼\u{FE0E}",
        // 144 => "⚿\u{FE0E}",
        // 150 => "⚻\u{FE0E}",
        180 => "☍\u{FE0E}",
        _ => "·",
    }
}

#[cfg(test)]
mod tests {
    use super::{planet_glyph, visible_len, PlanetName};

    #[test]
    fn visible_len_ignores_variation_selector() {
        for name in [
            PlanetName::Sun,
            PlanetName::Saturn,
            PlanetName::NorthNode,
            PlanetName::PartOfFortune,
        ] {
            assert_eq!(visible_len(planet_glyph(name)), 1);
        }
        assert_eq!(visible_len(planet_glyph(PlanetName::ASC)), 3);
        assert_eq!(visible_len("ASC"), 3);
        assert_eq!(visible_len(""), 0);
    }
}
