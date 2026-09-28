//! 多窗口星盘工作台（对应原版 workbench.page）：左侧输入面板 + 右侧工作区。
//!
//! 顶栏工具条：返回首页 / 面板开关 / 窗口列表下拉；
//! 侧栏：可折叠、可拖拽调宽（280px ～ 50vw）；
//! 工作区：空态引导 + 浮动窗口（拖拽 / 缩放 / 最大化 / 最小化 / 隐藏 / 关闭）。
mod input_panel;
mod window;
mod window_content;
mod window_frame;

use std::cmp::Reverse;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use wasm_bindgen::JsCast;

pub use input_panel::InputPanel;
use window::WindowMgr;
use window_frame::WindowFrame;

use crate::routes::AppRoute;

stylance::import_crate_style!(style, "src/workbench/workbench.module.css");

/// 侧栏默认宽度。
const SIDEBAR_DEFAULT_WIDTH: f64 = 340.0;
/// 侧栏最小宽度。
const SIDEBAR_MIN_WIDTH: f64 = 280.0;

/// 侧栏拖宽会话（纯数据）。
#[derive(Clone)]
struct SidebarDrag {
    start_x: f64,
    start_width: f64,
}

#[component]
pub fn Workbench() -> impl IntoView {
    let nav = use_navigate();
    let mgr = WindowMgr::new();
    provide_context(mgr);

    // 侧栏折叠与宽度；窄视口（<760px）默认折叠
    let sidebar_collapsed = RwSignal::new(
        web_sys::window()
            .and_then(|w| w.inner_width().ok())
            .and_then(|w| w.as_f64())
            .is_some_and(|w| w < 760.0),
    );
    let sidebar_width = RwSignal::new(SIDEBAR_DEFAULT_WIDTH);

    // 工作区容器（窗口定位的基准 + 打开窗口时的级联边界）
    let work_area = NodeRef::<leptos::html::Div>::new();

    // 窗口列表下拉
    let list_open = RwSignal::new(false);
    let list_wrapper = NodeRef::<leptos::html::Div>::new();

    // 窗口集合：任何窗口变化（开/关/状态/层级）时重算；
    // 窗口内部 rect 等动态字段由各 WindowFrame 自行从 mgr 读取。
    // WorkbenchWindow 无 PartialEq，用 Signal::derive 而非 Memo。
    let windows = Signal::derive(move || mgr.windows());
    let window_count = Signal::derive(move || windows.get().len());
    // 列表按层级降序（顶层在前）
    let sorted_windows = Signal::derive(move || {
        let mut wins = windows.get();
        wins.sort_by_key(|w| Reverse(w.z_index));
        wins
    });

    let go_home = move |_| {
        nav(AppRoute::Home.path(), NavigateOptions::default());
    };
    let toggle_sidebar = move |_| sidebar_collapsed.update(|v| *v = !*v);

    // —— 侧栏拖宽（同窗口拖拽模式：mousedown 挂全局监听，mouseup 解绑）——
    let sidebar_drag = RwSignal::new(None::<SidebarDrag>);
    let sidebar_drag_handles =
        RwSignal::new(None::<(WindowListenerHandle, WindowListenerHandle)>);
    let on_sidebar_resize_start = move |ev: leptos::ev::MouseEvent| {
        if sidebar_collapsed.get_untracked() {
            return;
        }
        ev.prevent_default();
        sidebar_drag.set(Some(SidebarDrag {
            start_x: ev.client_x() as f64,
            start_width: sidebar_width.get_untracked(),
        }));
        let on_move = move |ev: leptos::ev::MouseEvent| {
            if let Some(d) = sidebar_drag.get_untracked() {
                // 上限为半屏；极窄视口（< 2 倍最小宽度）下半屏会小于 SIDEBAR_MIN_WIDTH，
                // 直接 clamp 会因 min > max panic，故下限兜底（此时侧栏固定为最小宽度）。
                let max = web_sys::window()
                    .map(|w| w.inner_width().ok().and_then(|w| w.as_f64()))
                    .flatten()
                    .map(|w| w / 2.0)
                    .unwrap_or(600.0)
                    .max(SIDEBAR_MIN_WIDTH);
                let w = (d.start_width + ev.client_x() as f64 - d.start_x)
                    .clamp(SIDEBAR_MIN_WIDTH, max);
                sidebar_width.set(w);
            }
        };
        let on_up = move |_ev: leptos::ev::MouseEvent| {
            sidebar_drag.set(None);
            sidebar_drag_handles.set(None);
        };
        let h_move = window_event_listener(leptos::ev::mousemove, on_move);
        let h_up = window_event_listener(leptos::ev::mouseup, on_up);
        sidebar_drag_handles.set(Some((h_move, h_up)));
    };

    // —— 窗口列表下拉：点击外部关闭（打开时挂全局 mousedown）——
    let list_click_handles = RwSignal::new(None::<WindowListenerHandle>);
    let toggle_list = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        let opening = !list_open.get_untracked();
        list_open.set(opening);
        if opening {
            let on_doc_down = move |ev: leptos::ev::MouseEvent| {
                let inside = list_wrapper
                    .get_untracked()
                    .zip(ev.target())
                    .is_some_and(|(wrapper, t)| {
                        wrapper.contains(Some(&t.unchecked_into::<web_sys::Node>()))
                    });
                if !inside {
                    list_open.set(false);
                    list_click_handles.set(None);
                }
            };
            let h = window_event_listener(leptos::ev::mousedown, on_doc_down);
            list_click_handles.set(Some(h));
        } else {
            list_click_handles.set(None);
        }
    };
    // 下拉内切换窗口后收起
    let close_list = move || {
        list_open.set(false);
        list_click_handles.set(None);
    };

    view! {
        <div class=style::workbench>
            // —— 工具条 ——
            <div class=style::toolbar>
                <button class=style::back_btn on:click=go_home title="返回首页">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M19 12H5"/>
                        <path d="M12 19l-7-7 7-7"/>
                    </svg>
                    "首页"
                </button>
                <h2 class=style::toolbar_title>"工作台"</h2>
                <div class=style::toolbar_spacer></div>
                <button
                    class=move || {
                        if sidebar_collapsed.get() {
                            style::panel_btn.to_string()
                        } else {
                            format!("{} {}", style::panel_btn, style::panel_btn_active)
                        }
                    }
                    on:click=toggle_sidebar
                    title="显示/隐藏输入面板"
                >
                    // 眼睛图标
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                        <circle cx="12" cy="12" r="3"/>
                    </svg>
                    "面板"
                </button>

                <div class=style::list_wrapper node_ref=list_wrapper>
                    <Show when=move || window_count.get() != 0>
                        <button class=style::list_btn on:click=toggle_list>
                            // 层叠图标
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M12 2l10 5-10 5L2 7z"/>
                                <path d="M2 12l10 5 10-5"/>
                                <path d="M2 17l10 5 10-5"/>
                            </svg>
                            {move || format!("{} 个窗口", window_count.get())}
                            <svg
                                class=move || if list_open.get() { style::chevron_up } else { style::chevron }
                                viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                            >
                                <path d="M6 9l6 6 6-6"/>
                            </svg>
                        </button>
                    </Show>
                    <Show when=move || list_open.get()>
                        <div class=style::list_dropdown>
                            <div class=style::list_header>
                                <span>"窗口列表"</span>
                                <button class=style::list_close on:click=move |_| close_list()>
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                        stroke-width="2" stroke-linecap="round">
                                        <path d="M18 6L6 18M6 6l12 12"/>
                                    </svg>
                                </button>
                            </div>
                            <div class=style::list_items>
                                <For each=move || sorted_windows.get() key=|w| w.id let(win)>
                                    {
                                        let id = win.id;
                                        let mgr = mgr;
                                        let is_top = move || mgr.is_top(id);
                                        let activate = {
                                            let mgr = mgr;
                                            move |_| {
                                                mgr.activate(id);
                                                close_list();
                                            }
                                        };
                                        let close_item = {
                                            let mgr = mgr;
                                            move |ev: leptos::ev::MouseEvent| {
                                                ev.stop_propagation();
                                                mgr.close(id);
                                            }
                                        };
                                        view! {
                                            <div
                                                class=move || if is_top() {
                                                    format!("{} {}", style::list_item, style::list_item_active)
                                                } else {
                                                    style::list_item.to_string()
                                                }
                                                title=win.title.clone()
                                                on:click=activate
                                            >
                                                <span class=style::item_state_label>
                                                    {win.state.label()}
                                                </span>
                                                <span class=style::item_title>{win.title.clone()}</span>
                                                <button class=style::item_close on:click=close_item>
                                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                                        stroke-width="2" stroke-linecap="round">
                                                        <path d="M18 6L6 18M6 6l12 12"/>
                                                    </svg>
                                                </button>
                                            </div>
                                        }
                                    }
                                </For>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>

            // —— 主体：侧栏 + 工作区 ——
            <div class=style::body>
                <Show when=move || !sidebar_collapsed.get()>
                    <aside
                        class=style::sidebar
                        style=move || format!("width:{:.0}px", sidebar_width.get())
                    >
                        <InputPanel work_area=work_area/>
                        <div class=style::sidebar_handle on:mousedown=on_sidebar_resize_start></div>
                    </aside>
                </Show>
                <button class=style::sidebar_toggle on:click=toggle_sidebar>
                    {move || if sidebar_collapsed.get() { "»" } else { "«" }}
                </button>

                <div class=style::work_area node_ref=work_area>
                    <Show when=move || window_count.get() == 0>
                        <div class=style::empty_state>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.2" stroke-linejoin="round">
                                <rect x="3" y="3" width="8" height="12" rx="1.5"/>
                                <rect x="13" y="7" width="8" height="12" rx="1.5"/>
                            </svg>
                            <p>"从左侧面板选择星盘类型添加到工作台"</p>
                        </div>
                    </Show>
                    <For each=move || windows.get() key=|w| w.id let(win)>
                        <WindowFrame window=win work_area=work_area/>
                    </For>
                </div>
            </div>
        </div>
    }
}
