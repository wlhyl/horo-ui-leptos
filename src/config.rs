//! 全局常量与符号映射配置。

use crate::enums::planet::PlanetName;

/// 后台 API 基址。
///
/// - 未设置 `API_BASE_URL` 时默认 `http://localhost:8080`（本地开发）。
/// - Docker 镜像构建时设为空字符串，前端改用同源相对路径 `/api/...`，
///   由 ingress 路由到后台服务（容器内 nginx 仅托管静态资源）。
pub const API_BASE_URL: &str = match option_env!("API_BASE_URL") {
    Some(s) => s,
    None => "http://localhost:8080",
};

/// 登录 / 数据管理后台（horo-storage-api）的基址，规则同 [`API_BASE_URL`]。
pub const ADMIN_API_BASE_URL: &str = match option_env!("ADMIN_API_BASE_URL") {
    Some(s) => s,
    None => "http://localhost:8081",
};

/// 星盘 / 相位 SVG 的视图尺寸（同时用于 `viewBox` 与几何计算）。
pub const CHART_SVG_SIZE: f64 = 700.0;

/// 相位网格中参与计算的星体（与 `horo-config.service.ts` 中的 `horoPlanets` 一致）。
pub const GRID_PLANETS: [PlanetName; 14] = [
    PlanetName::Sun,
    PlanetName::Moon,
    PlanetName::Mercury,
    PlanetName::Venus,
    PlanetName::Mars,
    PlanetName::Jupiter,
    PlanetName::Saturn,
    PlanetName::NorthNode,
    PlanetName::SouthNode,
    PlanetName::ASC,
    PlanetName::MC,
    PlanetName::DSC,
    PlanetName::IC,
    PlanetName::PartOfFortune,
];
