use serde::{Deserialize, Serialize};

/// 主限法算法（对应原版 DirectionMethod）。
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DirectionMethod {
    /// 半弧法
    SemiArc,
    /// 极下法（世俗向运）
    UnderPole,
    /// 极下法（黄道向运）
    ZodiacalUnderPole,
}

/// 用户面向的展示名（下拉框选项），对齐原版 nameMap。
impl std::fmt::Display for DirectionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DirectionMethod::SemiArc => "半弧法",
            DirectionMethod::UnderPole => "极下法（世俗向运）",
            DirectionMethod::ZodiacalUnderPole => "极下法（黄道向运）",
        };
        f.write_str(s)
    }
}
