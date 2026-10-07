//! 工作台窗口状态类型与管理服务（对应原版 window-state.ts + window.service.ts）。
//!
//! 窗口数量少（<20），整表 `RwSignal<Vec<…>>` 更新开销可忽略，不引入更细粒度结构。
use leptos::prelude::*;

use crate::enums::planet::PlanetName;
use crate::models::data::{HoroData, ProcessData};
use crate::models::datetime::DateTimeData;
use crate::render::glyphs::planet_glyph;

/// 默认窗口尺寸（与原版一致）。
const DEFAULT_WIDTH: f64 = 460.0;
const DEFAULT_HEIGHT: f64 = 560.0;
/// 窗口最小尺寸（拖拽缩放下限）。
pub const MIN_WIDTH: f64 = 320.0;
pub const MIN_HEIGHT: f64 = 360.0;
/// 多窗口级联摆放的偏移步长。
const CASCADE_OFFSET: f64 = 30.0;
/// 级联偏移的循环上限（8 个窗口一轮）。
const CASCADE_STEPS: u32 = 8;
/// 窗口与工作区边缘的最小留白。
const MARGIN: f64 = 10.0;

/// 窗口可打开的星盘类型。后续接入其他推运盘时在此扩展，
/// 并在 window_content 中补充对应的请求与渲染分支。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    /// 本命盘（快照取输入面板的出生数据）
    Native,
    /// 天象盘（快照取输入面板的天象数据）
    Event,
    /// 衍生盘（快照取出生数据 + 基准行星，行星斜升为中天）
    Derived,
    /// 主向推运（快照取出生数据 + 推运数据）
    Direction,
    /// 每日回归方向弧（快照取出生数据 + 推运数据）
    DailyDirection,
    /// 太阳弧（快照取出生数据 + 推运数据）
    SolarArc,
    /// 日返（快照取出生数据 + 推运数据）
    SolarReturn,
    /// 月返（快照取出生数据 + 推运数据）
    LunarReturn,
    /// 每日回归盘（快照取出生数据 + 推运数据）
    DailyReturn,
}

impl ChartType {
    /// 窗口主标题。
    pub fn title(self) -> &'static str {
        match self {
            ChartType::Native => "本命盘",
            ChartType::Event => "天象盘",
            ChartType::Derived => "衍生盘",
            ChartType::Direction => "主向推运",
            ChartType::DailyDirection => "每日回归方向弧",
            ChartType::SolarArc => "太阳弧",
            ChartType::SolarReturn => "日返",
            ChartType::LunarReturn => "月返",
            ChartType::DailyReturn => "每日回归",
        }
    }

    /// 是否为方向推运类窗口（DirectionView 渲染）。
    pub fn is_direction(self) -> bool {
        matches!(
            self,
            ChartType::Direction | ChartType::DailyDirection | ChartType::SolarArc
        )
    }

    /// 是否为返照盘类窗口（ReturnView 渲染）。
    pub fn is_return(self) -> bool {
        matches!(
            self,
            ChartType::SolarReturn | ChartType::LunarReturn | ChartType::DailyReturn
        )
    }

    /// 是否为携带推运数据快照的窗口（方向推运 + 返照盘）。
    pub fn is_process(self) -> bool {
        self.is_direction() || self.is_return()
    }
}

/// 窗口状态机（与原版 WindowState 对齐）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// 正常浮动窗口
    Normal,
    /// 最小化（仅保留在窗口列表中）
    Minimized,
    /// 最大化（铺满工作区，保留 prev_rect 供还原）
    Maximized,
    /// 隐藏（比最小化更低调，同样仅保留在列表中）
    Hidden,
}

impl WindowState {
    /// 是否占据工作区（Normal / Maximized）。
    pub fn is_visible(self) -> bool {
        matches!(self, WindowState::Normal | WindowState::Maximized)
    }

    /// 窗口列表中的状态标签。
    pub fn label(self) -> &'static str {
        match self {
            WindowState::Minimized => "最小化",
            WindowState::Maximized => "最大化",
            WindowState::Hidden => "隐藏",
            WindowState::Normal => "",
        }
    }
}

/// 窗口矩形（相对工作区左上角，单位 px）。
#[derive(Clone, Copy, PartialEq)]
pub struct WindowRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl WindowRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// 工作台中的一个窗口。
#[derive(Clone)]
pub struct WorkbenchWindow {
    pub id: u64,
    /// 标题（主标题 + 打开时刻的日期摘要，便于区分多窗口）
    pub title: String,
    pub chart_type: ChartType,
    pub state: WindowState,
    pub rect: WindowRect,
    pub z_index: u32,
    /// 最大化前的原矩形（还原用）
    pub prev_rect: Option<WindowRect>,
    /// 打开时刻的输入数据快照：面板后续编辑不影响已开窗口
    pub snapshot: HoroData,
    /// 衍生盘窗口的基准行星（打开时快照，窗口间相互独立；非衍生盘为 None，
    /// 对应原版 WorkbenchWindow.derivedPlanetName）
    pub derived_planet: Option<PlanetName>,
    /// 方向推运 / 返照盘类窗口的推运数据快照（推运时间 / 居住地 / 算法 /
    /// 日返月亮；打开时快照，窗口间相互独立；其余类型为 None）
    pub process: Option<ProcessData>,
}

/// 日期摘要："2000-01-01 12:00"，拼进窗口标题。
pub(crate) fn date_summary(date: &DateTimeData) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        date.year, date.month, date.day, date.hour, date.minute
    )
}

/// 窗口管理服务：Workbench 页面构造后 `provide_context`，
/// 供输入面板（open）、窗口框架（focus/rect…）、窗口列表（restore/close）调用。
///
/// 结构体为 Copy，各处克隆共享同一批信号（与 HoroStorage 同一模式）。
#[derive(Clone, Copy)]
pub struct WindowMgr {
    windows: RwSignal<Vec<WorkbenchWindow>>,
    id_counter: RwSignal<u64>,
    z_counter: RwSignal<u32>,
    /// 级联摆放计数：`open()` 取它对 CASCADE_STEPS 取模乘 CASCADE_OFFSET，
    /// 得到新窗口的左上偏移，使连开的窗口逐层错开、不全遮挡。
    /// 只增不减（关窗不回收）；满 CASCADE_STEPS 个后取模回绕，位置从头复用。
    cascade_index: RwSignal<u32>,
    /// 当前最顶层可见窗口 id（无可见窗口时为 None）
    top_window_id: Memo<Option<u64>>,
}

impl WindowMgr {
    pub fn new() -> Self {
        let windows = RwSignal::new(Vec::<WorkbenchWindow>::new());
        let top_window_id = Memo::new(move |_| {
            windows
                .get()
                .iter()
                .filter(|w| w.state.is_visible())
                .max_by_key(|w| w.z_index)
                .map(|w| w.id)
        });
        Self {
            windows,
            id_counter: RwSignal::new(0),
            z_counter: RwSignal::new(10),
            cascade_index: RwSignal::new(0),
            top_window_id,
        }
    }

    pub fn windows(&self) -> Vec<WorkbenchWindow> {
        self.windows.get()
    }

    pub fn is_top(&self, id: u64) -> bool {
        self.top_window_id.get() == Some(id)
    }

    /// 读取窗口当前矩形（`For` 的 key 复用不重渲染 children，
    /// 窗口框架组件内部用 Memo 调此方法取动态字段）。
    pub fn rect_of(&self, id: u64) -> Option<WindowRect> {
        self.windows
            .with(|wins| wins.iter().find(|w| w.id == id).map(|w| w.rect))
    }

    /// 读取窗口当前状态。
    pub fn state_of(&self, id: u64) -> Option<WindowState> {
        self.windows
            .with(|wins| wins.iter().find(|w| w.id == id).map(|w| w.state))
    }

    /// 读取窗口当前层级。
    pub fn z_of(&self, id: u64) -> Option<u32> {
        self.windows
            .with(|wins| wins.iter().find(|w| w.id == id).map(|w| w.z_index))
    }

    /// 读取窗口当前标题（方向推运窗口选中显著星后标题变化，
    /// `For` 的 key 复用不重渲染 children，窗口框架组件内部用 Memo 调此方法）。
    pub fn title_of(&self, id: u64) -> Option<String> {
        self.windows
            .with(|wins| wins.iter().find(|w| w.id == id).map(|w| w.title.clone()))
    }

    /// 打开新窗口：级联摆放并夹取到工作区内，打开时快照输入数据。
    /// `work_area` 为工作区尺寸（宽高，原点 0,0）。
    /// 衍生盘窗口经 `derived_planet` 携带基准行星快照（其余类型传 None）；
    /// 方向推运 / 返照盘类窗口经 `process` 携带推运数据快照（其余类型传 None）。
    pub fn open(
        &self,
        chart_type: ChartType,
        snapshot: HoroData,
        work_area: WindowRect,
        derived_planet: Option<PlanetName>,
        process: Option<ProcessData>,
    ) {
        let offset = (self.cascade_index.get_untracked() % CASCADE_STEPS) as f64 * CASCADE_OFFSET;
        // 默认尺寸按工作区收紧（宽高都裁进工作区，而非只贴回 x/y）
        let fitted = WindowRect::new(
            MARGIN.max(offset),
            MARGIN.max(offset),
            DEFAULT_WIDTH.min((work_area.width - 2.0 * MARGIN).max(MIN_WIDTH)),
            DEFAULT_HEIGHT.min((work_area.height - 2.0 * MARGIN).max(MIN_HEIGHT)),
        );
        // 窄工作区（手机竖屏 / 侧栏挤压）放不下浮动窗口时直接最大化打开：
        // 否则默认 460px 宽会把标题栏右侧按钮挤出屏幕，窗口无法关闭。
        // prev_rect 存放下限尺寸的「还原矩形」，还原 / 恢复时回到可用大小。
        let area_too_narrow = work_area.width < DEFAULT_WIDTH + 2.0 * MARGIN;

        let id = self.next_id();
        let z = self.next_z();
        // 衍生盘标题带基准行星符号（如「衍生盘·♄」）
        let title = match chart_type {
            ChartType::Derived => format!(
                "{}·{} · {}",
                chart_type.title(),
                planet_glyph(derived_planet.unwrap_or(PlanetName::Sun)),
                date_summary(&snapshot.date)
            ),
            _ => format!("{} · {}", chart_type.title(), date_summary(&snapshot.date)),
        };
        self.windows.update(|wins| {
            wins.push(WorkbenchWindow {
                id,
                title,
                chart_type,
                state: if area_too_narrow {
                    WindowState::Maximized
                } else {
                    WindowState::Normal
                },
                rect: if area_too_narrow {
                    WindowRect::new(0.0, 0.0, work_area.width, work_area.height)
                } else {
                    fitted
                },
                z_index: z,
                prev_rect: area_too_narrow.then_some(fitted),
                snapshot,
                derived_planet: (chart_type == ChartType::Derived)
                    .then_some(derived_planet.unwrap_or(PlanetName::Sun)),
                process: chart_type.is_process().then_some(process).flatten(),
            })
        });
        self.cascade_index.update(|i| *i += 1);
    }

    /// 更新窗口标题（方向推运窗口选中恰一个显著星时附加该星）。
    pub fn update_title(&self, id: u64, title: String) {
        self.windows.update(|wins| {
            if let Some(w) = wins.iter_mut().find(|w| w.id == id) {
                w.title = title;
            }
        });
    }

    /// 关闭窗口（从列表移除，释放内容状态）。
    pub fn close(&self, id: u64) {
        self.windows.update(|wins| wins.retain(|w| w.id != id));
    }

    /// 窗口置顶（点击 / 拖拽标题栏 / 恢复时调用）。
    pub fn focus(&self, id: u64) {
        let z = self.next_z();
        self.windows.update(|wins| {
            if let Some(w) = wins.iter_mut().find(|w| w.id == id) {
                w.z_index = z;
            }
        });
    }

    pub fn minimize(&self, id: u64) {
        self.set_state(id, WindowState::Minimized);
    }

    pub fn hide(&self, id: u64) {
        self.set_state(id, WindowState::Hidden);
    }

    /// 最大化（快照当前矩形到 prev_rect）或还原，二态切换。
    pub fn toggle_maximize(&self, id: u64, work_area: WindowRect) {
        let z = self.next_z();
        self.windows.update(|wins| {
            if let Some(w) = wins.iter_mut().find(|w| w.id == id) {
                match w.state {
                    WindowState::Maximized => {
                        if let Some(prev) = w.prev_rect {
                            w.rect = prev;
                        }
                        w.prev_rect = None;
                        w.state = WindowState::Normal;
                        w.z_index = z;
                    }
                    _ => {
                        w.prev_rect = Some(w.rect);
                        w.rect = WindowRect::new(0.0, 0.0, work_area.width, work_area.height);
                        w.state = WindowState::Maximized;
                        w.z_index = z;
                    }
                }
            }
        });
    }

    /// 从最小化 / 隐藏恢复：之前是最大化则恢复为最大化（保留 prev_rect），
    /// 否则恢复为 Normal（对齐原版 restoreWindow 语义）。
    pub fn restore(&self, id: u64) {
        let z = self.next_z();
        self.windows.update(|wins| {
            if let Some(w) = wins.iter_mut().find(|w| w.id == id) {
                match w.state {
                    WindowState::Minimized | WindowState::Hidden => {
                        w.state = if w.prev_rect.is_some() {
                            WindowState::Maximized
                        } else {
                            WindowState::Normal
                        };
                        w.z_index = z;
                    }
                    _ => {}
                }
            }
        });
    }

    /// 窗口列表点击切换：不可见则恢复，可见则聚焦。
    pub fn activate(&self, id: u64) {
        let need_restore = self.windows.with_untracked(|wins| {
            wins.iter()
                .find(|w| w.id == id)
                .is_some_and(|w| !w.state.is_visible())
        });
        if need_restore {
            self.restore(id);
        } else {
            self.focus(id);
        }
    }

    /// 拖拽 / 缩放时更新矩形（最小尺寸钳制 + 取整）。
    pub fn update_rect(&self, id: u64, rect: WindowRect) {
        self.windows.update(|wins| {
            if let Some(w) = wins.iter_mut().find(|w| w.id == id) {
                w.rect = WindowRect::new(
                    rect.x.round(),
                    rect.y.round(),
                    rect.width.round().max(MIN_WIDTH),
                    rect.height.round().max(MIN_HEIGHT),
                );
            }
        });
    }

    fn set_state(&self, id: u64, state: WindowState) {
        self.windows.update(|wins| {
            if let Some(w) = wins.iter_mut().find(|w| w.id == id) {
                w.state = state;
            }
        });
    }

    fn next_id(&self) -> u64 {
        self.id_counter.update(|i| *i += 1);
        self.id_counter.get_untracked()
    }

    fn next_z(&self) -> u32 {
        self.z_counter.update(|z| *z += 1);
        self.z_counter.get_untracked()
    }
}
