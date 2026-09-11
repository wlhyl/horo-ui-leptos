#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
#[repr(u8)]
pub enum Zodiac {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl Zodiac {
    pub fn from_index(index: u8) -> Self {
        match index {
            0 => Self::Aries,
            1 => Self::Taurus,
            2 => Self::Gemini,
            3 => Self::Cancer,
            4 => Self::Leo,
            5 => Self::Virgo,
            6 => Self::Libra,
            7 => Self::Scorpio,
            8 => Self::Sagittarius,
            9 => Self::Capricorn,
            10 => Self::Aquarius,
            11 => Self::Pisces,
            _ => unreachable!("星座索引必须在 0..12 内"),
        }
    }
}

/// 用户面向的展示名，实现后自动获得 `to_string()`。
impl std::fmt::Display for Zodiac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Zodiac::Aries => "Aries",
            Zodiac::Taurus => "Taurus",
            Zodiac::Gemini => "Gemini",
            Zodiac::Cancer => "Cancer",
            Zodiac::Leo => "Leo",
            Zodiac::Virgo => "Virgo",
            Zodiac::Libra => "Libra",
            Zodiac::Scorpio => "Scorpio",
            Zodiac::Sagittarius => "Sagittarius",
            Zodiac::Capricorn => "Capricorn",
            Zodiac::Aquarius => "Aquarius",
            Zodiac::Pisces => "Pisces",
        };
        f.write_str(s)
    }
}
