//! 全项目路由路径的集中定义。
use leptos_router::{PartialPathMatch, PathSegment, PossibleRouteMatch, StaticSegment};

/// 全应用路由枚举：新增页面时在此添加变体及对应路径。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppRoute {
    /// 首页：选择星盘类型。
    Home,
    /// 本命星盘输入页。
    Native,
    /// 天象盘输入页。
    Event,
    /// 衍生盘输入页。
    Derived,
    /// 推运输入页（推运类型选择主向推运 / 每日回归方向弧 / 太阳弧）。
    Process,
    /// 本命星盘结果页。
    NativeChart,
    /// 天象盘结果页。
    EventChart,
    /// 衍生盘结果页。
    DerivedChart,
    /// 主向推运结果页。
    Direction,
    /// 每日回归方向弧结果页。
    DailyDirection,
    /// 太阳弧结果页。
    SolarArc,
    /// 用户登录页。
    User,
    /// 清除缓存页。
    Clean,
    /// 行星力量表页。
    Power,
    /// 多窗口星盘工作台页。
    Workbench,
}

impl AppRoute {
    /// 对应的 URL 路径（含前导 `/`）。
    pub fn path(self) -> &'static str {
        match self {
            AppRoute::Home => "/",
            AppRoute::Native => "/native",
            AppRoute::Event => "/event",
            AppRoute::Derived => "/derived",
            AppRoute::Process => "/process",
            AppRoute::NativeChart => "/native/chart",
            AppRoute::EventChart => "/event/chart",
            AppRoute::DerivedChart => "/derived/chart",
            AppRoute::Direction => "/direction",
            AppRoute::DailyDirection => "/daily_direction",
            AppRoute::SolarArc => "/solar_arc",
            AppRoute::User => "/user",
            AppRoute::Clean => "/clean",
            AppRoute::Power => "/power",
            AppRoute::Workbench => "/workbench",
        }
    }

    /// 路径中的各段（去掉前导 `/` 产生的空串）。
    fn segments(self) -> impl Iterator<Item = &'static str> {
        self.path().split('/').filter(|s| !s.is_empty())
    }
}

/// 供 `<Route path=...>` 使用的匹配器：由 `path()` 派生，路径字符串只需维护一份。
///
/// `StaticSegment` 只匹配单段（遇 `/` 即止），多段路径必须逐段匹配；
/// 这里按段遍历，因此任意层级的路径（如 `/native/chart/detail`）都能直接用枚举注册。
impl PossibleRouteMatch for AppRoute {
    fn optional(&self) -> bool {
        false
    }

    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let mut matched_len = 0;
        let mut remaining = path;
        for segment in self.segments() {
            let matched = StaticSegment(segment).test(remaining)?;
            matched_len += matched.matched().len();
            remaining = matched.remaining();
        }
        Some(PartialPathMatch::new(
            remaining,
            Vec::new(),
            &path[..matched_len],
        ))
    }

    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        for segment in self.segments() {
            path.push(PathSegment::Static(segment.into()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(route: AppRoute, path: &str) -> bool {
        route.test(path).is_some_and(|m| m.is_complete())
    }

    #[test]
    fn matches_own_path_only() {
        assert!(matches(AppRoute::Home, "/"));
        assert!(!matches(AppRoute::Home, "/native"));

        assert!(matches(AppRoute::Native, "/native"));
        assert!(!matches(AppRoute::Native, "/native/chart"));

        assert!(matches(AppRoute::NativeChart, "/native/chart"));
        assert!(!matches(AppRoute::NativeChart, "/native"));
        assert!(!matches(AppRoute::NativeChart, "/event/chart"));

        assert!(matches(AppRoute::EventChart, "/event/chart"));
        // 不做前缀匹配，避免 /native/chart/detail 之类的路径被上层路由吃掉
        assert!(!matches(AppRoute::NativeChart, "/native/chart/detail"));

        assert!(matches(AppRoute::Derived, "/derived"));
        assert!(matches(AppRoute::DerivedChart, "/derived/chart"));
        assert!(!matches(AppRoute::NativeChart, "/derived/chart"));
        assert!(!matches(AppRoute::DerivedChart, "/native/chart"));

        assert!(matches(AppRoute::Process, "/process"));
        assert!(!matches(AppRoute::Direction, "/process"));
        assert!(matches(AppRoute::Direction, "/direction"));
        assert!(matches(AppRoute::DailyDirection, "/daily_direction"));
        assert!(matches(AppRoute::SolarArc, "/solar_arc"));
        assert!(!matches(AppRoute::DailyDirection, "/direction"));
        assert!(!matches(AppRoute::Direction, "/daily_direction"));

        assert!(matches(AppRoute::User, "/user"));
        assert!(!matches(AppRoute::User, "/"));
        assert!(!matches(AppRoute::Home, "/user"));

        assert!(matches(AppRoute::Clean, "/clean"));
        assert!(!matches(AppRoute::Clean, "/native"));
        assert!(!matches(AppRoute::Home, "/clean"));

        assert!(matches(AppRoute::Workbench, "/workbench"));
        assert!(!matches(AppRoute::Workbench, "/native"));
        assert!(!matches(AppRoute::Home, "/workbench"));
    }

    #[test]
    fn generate_path_round_trips() {
        for route in [
            AppRoute::Home,
            AppRoute::Native,
            AppRoute::Event,
            AppRoute::Derived,
            AppRoute::Process,
            AppRoute::NativeChart,
            AppRoute::EventChart,
            AppRoute::DerivedChart,
            AppRoute::Direction,
            AppRoute::DailyDirection,
            AppRoute::SolarArc,
            AppRoute::User,
            AppRoute::Clean,
            AppRoute::Power,
            AppRoute::Workbench,
        ] {
            let mut segments = Vec::new();
            route.generate_path(&mut segments);
            let rebuilt: String = segments
                .iter()
                .map(|s| match s {
                    PathSegment::Static(s) => format!("/{s}"),
                    _ => unreachable!("全部为静态段"),
                })
                .collect();
            // 首页 path() 为 "/"，去掉末尾斜杠后与 generate_path 的空结果一致
            assert_eq!(rebuilt, route.path().trim_end_matches('/'));
        }
    }
}
