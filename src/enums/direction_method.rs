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
