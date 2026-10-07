//! 窗口框架：标题栏拖拽移动、8 方向缩放、最小化/隐藏/最大化/关闭按钮。
//!
//! 窗口的 rect / state / z_index 是动态字段：`For` 以 id 为 key 复用视图、
//! 不会因 item 内容变化重渲染，因此这些字段在本组件内用 Memo 从 WindowMgr
//! 读取；title / chart_type / snapshot / derived_planet 打开后不变，经 props 传入一次即可。
use leptos::prelude::*;

use crate::workbench::window::{
    MIN_HEIGHT, MIN_WIDTH, WindowMgr, WindowRect, WindowState, WorkbenchWindow, date_summary,
};
use crate::workbench::window_content::WindowContent;

stylance::import_crate_style!(style, "src/workbench/window_frame.module.css");

/// 拖拽会话状态（纯数据，存 RwSignal 供事件闭包共享）。
#[derive(Clone)]
struct DragState {
    /// 缩放方向（"n"/"s"/"e"/"w"/"nw"/"ne"/"sw"/"se"），None 表示移动
    resizing: Option<&'static str>,
    /// 鼠标起点（client 坐标）
    start_x: f64,
    start_y: f64,
    /// 窗口起始矩形
    start_rect: WindowRect,
    /// 工作区尺寸快照（mousemove 中不再读布局）
    bounds: WindowRect,
}

impl DragState {
    /// 由方向与位移计算新矩形（对齐原版 window-frame 的缩放算法）。
    fn apply(&self, dx: f64, dy: f64) -> WindowRect {
        let r = self.start_rect;
        let mut x = r.x;
        let mut y = r.y;
        let mut width = r.width;
        let mut height = r.height;
        let dir = self.resizing.unwrap_or_default();

        if dir.contains('e') {
            width = (r.width + dx).max(MIN_WIDTH);
        }
        if dir.contains('s') {
            height = (r.height + dy).max(MIN_HEIGHT);
        }
        if dir.contains('w') {
            let w = (r.width - dx).max(MIN_WIDTH);
            x = r.x + (r.width - w);
            width = w;
        }
        if dir.contains('n') {
            let h = (r.height - dy).max(MIN_HEIGHT);
            y = r.y + (r.height - h);
            height = h;
        }
        if dir.is_empty() {
            // 移动：保留至少 40px 在工作区内（标题栏不被拖出可视区）
            let max_x = (self.bounds.width - 40.0).max(0.0);
            let max_y = (self.bounds.height - 40.0).max(0.0);
            x = (r.x + dx).clamp(0.0, max_x);
            y = (r.y + dy).clamp(0.0, max_y);
        }
        WindowRect::new(x, y, width, height)
    }
}

#[component]
pub fn WindowFrame(
    window: WorkbenchWindow,
    work_area: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    let mgr = use_context::<WindowMgr>().expect("WindowMgr 未初始化");
    let id = window.id;

    // 动态字段：Memo 从管理器读取（rect/state/z 变化时仅更新本窗口）
    let rect = Memo::new(move |_| mgr.rect_of(id));
    let state = Memo::new(move |_| mgr.state_of(id));
    let z_index = Memo::new(move |_| mgr.z_of(id));
    let is_top = Memo::new(move |_| mgr.is_top(id));
    // 标题也是动态字段：方向推运窗口选中显著星后经 update_title 变化
    let title = Memo::new(move |_| mgr.title_of(id).unwrap_or_else(|| window.title.clone()));
    let is_visible = Memo::new(move |_| state.get().is_some_and(|s| s.is_visible()));
    let is_maximized = Memo::new(move |_| state.get() == Some(WindowState::Maximized));

    // 拖拽会话与全局监听 handle（handle 存信号：drop 即解绑）
    let drag = RwSignal::new(None::<DragState>);
    let drag_handles = RwSignal::new(None::<Vec<WindowListenerHandle>>);

    // 读取工作区尺寸（拖拽开始时一次性快照，mousemove 不触发布局读取）。
    let bounds_now = move || {
        work_area
            .get()
            .map(|el| {
                WindowRect::new(
                    0.0,
                    0.0,
                    el.client_width() as f64,
                    el.client_height() as f64,
                )
            })
            .unwrap_or_else(|| WindowRect::new(0.0, 0.0, 800.0, 600.0))
    };

    // pointerdown 统一入口：记录起点并挂全局 pointermove/pointerup 监听。
    // 用 Pointer Events 一套代码同时覆盖鼠标与触屏（触屏有隐式指针捕获，
    // 手指移出元素后事件仍冒泡到 window）。
    let start_drag = move |ev: leptos::ev::PointerEvent, resizing: Option<&'static str>| {
        ev.prevent_default();
        mgr.focus(id);
        let Some(start_rect) = rect.get_untracked() else {
            return;
        };
        drag.set(Some(DragState {
            resizing,
            start_x: ev.client_x() as f64,
            start_y: ev.client_y() as f64,
            start_rect,
            bounds: bounds_now(),
        }));

        let on_move = {
            let mgr = mgr;
            move |ev: leptos::ev::PointerEvent| {
                if let Some(d) = drag.get_untracked() {
                    let new_rect = d.apply(
                        ev.client_x() as f64 - d.start_x,
                        ev.client_y() as f64 - d.start_y,
                    );
                    mgr.update_rect(id, new_rect);
                }
            }
        };
        let on_up = move |_ev: leptos::ev::PointerEvent| {
            drag.set(None);
            // drop handle → 解绑全局监听
            drag_handles.set(None);
        };
        let h_move = window_event_listener(leptos::ev::pointermove, on_move.clone());
        let h_up = window_event_listener(leptos::ev::pointerup, on_up.clone());
        // 触屏手势被系统 / 浏览器接管（来电、下拉刷新等）时结束拖拽
        let h_cancel = window_event_listener(leptos::ev::pointercancel, on_up);
        drag_handles.set(Some(vec![h_move, h_up, h_cancel]));
    };

    // 标题栏按下：按钮已 stop_propagation，到这里即开始移动
    let on_title_down = move |ev: leptos::ev::PointerEvent| {
        if is_maximized.get_untracked() {
            return;
        }
        start_drag(ev, None);
    };

    // resize handle 按下（8 方向）
    let on_resize = move |dir: &'static str| {
        move |ev: leptos::ev::PointerEvent| {
            if is_maximized.get_untracked() {
                return;
            }
            ev.stop_propagation();
            start_drag(ev, Some(dir));
        }
    };

    // 窗口任意处按下即聚焦
    let on_frame_down = move |_ev: leptos::ev::PointerEvent| {
        mgr.focus(id);
    };

    // 标题栏按钮
    let on_minimize = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        mgr.minimize(id);
    };
    let on_hide = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        mgr.hide(id);
    };
    let on_toggle_max = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        let area = bounds_now();
        mgr.toggle_maximize(id, area);
    };
    let on_close = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        mgr.close(id);
    };

    // 按钮按下不触发拖拽
    let swallow = move |ev: leptos::ev::PointerEvent| {
        ev.stop_propagation();
    };

    view! {
        <div
            class=move || {
                let mut cls = style::frame.to_string();
                if !is_visible.get() {
                    cls.push(' ');
                    cls.push_str(&style::hidden);
                }
                if is_maximized.get() {
                    cls.push(' ');
                    cls.push_str(&style::maximized);
                }
                if is_top.get() {
                    cls.push(' ');
                    cls.push_str(&style::active);
                }
                cls
            }
            style=move || {
                let Some(r) = rect.get() else { return String::new() };
                let z = z_index.get().unwrap_or(0);
                format!(
                    "left:{:.0}px;top:{:.0}px;width:{:.0}px;height:{:.0}px;z-index:{}",
                    r.x, r.y, r.width, r.height, z
                )
            }
            on:pointerdown=on_frame_down
        >
            <div class=style::title_bar on:pointerdown=on_title_down>
                <span class=style::title_text title=move || title.get()>
                    {move || title.get()}
                </span>
                <div class=style::title_buttons on:pointerdown=swallow>
                    <button class=style::win_btn on:click=on_minimize title="最小化">
                        // 减号
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="2" stroke-linecap="round">
                            <path d="M5 12h14"/>
                        </svg>
                    </button>
                    <button class=style::win_btn on:click=on_hide title="隐藏">
                        // 眼睛斜线（隐藏）
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94"/>
                            <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19"/>
                            <path d="M14.12 14.12a3 3 0 1 1-4.24-4.24"/>
                            <path d="M1 1l22 22"/>
                        </svg>
                    </button>
                    <button class=style::win_btn on:click=on_toggle_max title="最大化/还原">
                        // 最大化/还原图标（随状态切换）
                        {move || if is_maximized.get() {
                            view! {
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                    stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M9 3H5a2 2 0 0 0-2 2v4"/>
                                    <path d="M15 3h4a2 2 0 0 1 2 2v4"/>
                                    <path d="M9 21H5a2 2 0 0 1-2-2v-4"/>
                                    <path d="M15 21h4a2 2 0 0 0 2-2v-4"/>
                                </svg>
                            }.into_any()
                        } else {
                            view! {
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                    stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <rect x="5" y="5" width="14" height="14" rx="2"/>
                                </svg>
                            }.into_any()
                        }}
                    </button>
                    <button class=move || format!("{} {}", style::win_btn, style::close_btn)
                        on:click=on_close title="关闭">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="2" stroke-linecap="round">
                            <path d="M18 6L6 18M6 6l12 12"/>
                        </svg>
                    </button>
                </div>
            </div>

            <div class=style::content>
                <WindowContent
                    chart_type=window.chart_type
                    snapshot=window.snapshot.clone()
                    derived_planet=window.derived_planet
                    process=window.process
                    // 方向推运窗口：显著星筛选变化时按后缀重建标题（对齐原版 titleChange）
                    on_title=if window.chart_type.is_direction() {
                        let mgr = mgr;
                        let date = window.snapshot.date;
                        Some(Callback::new(move |suffix: String| {
                            mgr.update_title(
                                id,
                                format!(
                                    "{} · {}{}",
                                    window.chart_type.title(),
                                    date_summary(&date),
                                    suffix
                                ),
                            );
                        }))
                    } else {
                        None
                    }
                />
            </div>

            // 缩放手柄（最大化时隐藏）
            <Show when=move || !is_maximized.get()>
                {view! {
                    <div class=style::handle_n on:pointerdown=on_resize("n")></div>
                    <div class=style::handle_s on:pointerdown=on_resize("s")></div>
                    <div class=style::handle_w on:pointerdown=on_resize("w")></div>
                    <div class=style::handle_e on:pointerdown=on_resize("e")></div>
                    <div class=style::handle_nw on:pointerdown=on_resize("nw")></div>
                    <div class=style::handle_ne on:pointerdown=on_resize("ne")></div>
                    <div class=style::handle_sw on:pointerdown=on_resize("sw")></div>
                    <div class=style::handle_se on:pointerdown=on_resize("se")></div>
                }}
            </Show>
        </div>
    }
}
