//! 绘制层：把排盘结果渲染成 SVG 字符串。
//!
//! 准入规则：只做「数据 -> SVG」，不掺业务判断；纯计算放 `crate::astro`。
//! 后续新增体系（七政四余 / 推运 / 合盘）在此目录下按体系增文件即可。

pub mod aspect;
pub mod glyphs;
pub mod svg;
pub mod wheel;
