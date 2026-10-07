//! 推运类型下拉（自定义分组下拉，对应原版 select + optgroup）。
//!
//! 原生 `<select>` 的弹开列表由浏览器渲染，无法画分隔线或定制样式，
//! 故以「触发按钮 + 弹层列表」实现：按 方向推运 / 返照盘 两组展示（组间为
//! 与工作台「添加星盘」按钮组同款的分隔线）；点选写回 `process_name`
//! 信号（工作台经既有 Effect 落 localStorage，输入页随提交读取）。
//! 弹层以 fixed 定位按触发器位置计算坐标（脱离侧栏滚动容器的裁剪），
//! 下方空间不足时向上弹出；点击遮罩、滚轮 / 触摸滚动或按 Escape 收起。
use leptos::control_flow::Show;
use leptos::portal::Portal;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::direction::utils::{PROCESS_GROUPS, process_title};
use crate::enums::process_name::ProcessName;

// 作用域样式：src/components/process_type_select/process_type_select.module.css
stylance::import_crate_style!(
    style,
    "src/components/process_type_select/process_type_select.module.css"
);

/// 弹层展开所需的最小下方空间（px）：不足时改为向上弹出。
const DROPDOWN_SPACE_PX: f64 = 320.0;
/// 弹层与触发器的间距、与视口边缘的最小留白（px）。
const POPUP_GAP_PX: f64 = 6.0;
const VIEWPORT_MARGIN_PX: f64 = 8.0;
/// 弹层内容全展开的高度上限（px）：空间充裕时整体可见，否则内部滚动。
const POPUP_MAX_HEIGHT_PX: f64 = 380.0;

#[component]
pub fn ProcessTypeSelect(process_name: RwSignal<ProcessName>) -> impl IntoView {
    let open = RwSignal::new(false);
    let trigger_ref = NodeRef::<leptos::html::Button>::new();
    // 弹层的行内定位样式（fixed）：展开时按触发器实时位置计算
    let popup_style = RwSignal::new(String::new());

    let toggle = move |_| {
        if open.get_untracked() {
            open.set(false);
            return;
        }
        let style_str = trigger_ref.get().map(|el| {
            let rect = el
                .unchecked_ref::<web_sys::Element>()
                .get_bounding_client_rect();
            let win = leptos::prelude::window();
            let vh = win
                .inner_height()
                .unwrap_or_default()
                .as_f64()
                .unwrap_or(0.0);
            let vw = win
                .inner_width()
                .unwrap_or_default()
                .as_f64()
                .unwrap_or(0.0);
            let width = rect.width();
            let left = rect.left().clamp(
                VIEWPORT_MARGIN_PX,
                (vw - width - VIEWPORT_MARGIN_PX).max(VIEWPORT_MARGIN_PX),
            );
            // 下方空间不足（或太贴近视口底部）时改为向上弹出；
            // 最大高度按可用空间动态收紧（不足时弹层内部滚动）
            let (edge_style, avail) = if rect.bottom() + DROPDOWN_SPACE_PX > vh {
                (
                    format!("bottom:{}px;", vh - rect.top() + POPUP_GAP_PX),
                    rect.top() - POPUP_GAP_PX - VIEWPORT_MARGIN_PX,
                )
            } else {
                (
                    format!("top:{}px;", rect.bottom() + POPUP_GAP_PX),
                    vh - rect.bottom() - POPUP_GAP_PX - VIEWPORT_MARGIN_PX,
                )
            };
            let max_h = POPUP_MAX_HEIGHT_PX.min(avail).max(160.0);
            format!("left:{left}px;width:{width}px;{edge_style}max-height:{max_h}px;")
        });
        popup_style.set(style_str.unwrap_or_default());
        open.set(true);
    };

    // Escape 收起：弹层打开期间挂 window keydown 监听；关闭或组件销毁时解绑
    // （handle 未实现 Drop 且不可 Clone，存 Vec 用 update 取出后 remove，
    // 对齐 window_frame 的监听管理）
    let esc_handles = RwSignal::new(Vec::<WindowListenerHandle>::new());
    Effect::new(move |_| {
        if open.get() {
            let open = open;
            let handle = window_event_listener(leptos::ev::keydown, move |ev| {
                if ev.key() == "Escape" {
                    open.set(false);
                }
            });
            esc_handles.update(|v| v.push(handle));
        } else {
            esc_handles.update(|v| {
                while let Some(h) = v.pop() {
                    h.remove();
                }
            });
        }
    });
    on_cleanup(move || {
        esc_handles.update(|v| {
            while let Some(h) = v.pop() {
                h.remove();
            }
        });
    });

    view! {
        <div class=style::wrap>
            <button
                type="button"
                node_ref=trigger_ref
                class=style::trigger
                aria-haspopup="listbox"
                aria-expanded=move || open.get()
                on:click=toggle
            >
                <span>{move || process_title(process_name.get())}</span>
                <svg
                    class=move || {
                        if open.get() { style::chevron_open } else { style::chevron }
                    }
                    viewBox="0 0 24 24" fill="none" stroke="currentColor"
                    stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                >
                    <path d="M6 9l6 6 6-6"/>
                </svg>
            </button>
            // 遮罩与弹层经 Portal 挂到 document.body：fixed 定位不受
            // transformed 祖先（如卡片的入场动画残留 transform）劫持
            <Portal>
                <Show when=move || open.get()>
                    // 透明遮罩：点击 / 滚轮 / 触摸滚动弹层以外任意处收起
                    <div
                        class=style::overlay
                        on:click=move |_| open.set(false)
                        on:wheel=move |_| open.set(false)
                        on:touchmove=move |_| open.set(false)
                    />
                    <div
                        role="listbox"
                        class=style::popup
                        style=move || popup_style.get()
                    >
                    {PROCESS_GROUPS
                        .iter()
                        .enumerate()
                        .map(|(i, (label, items))| {
                            view! {
                                <Show when=move || { i > 0 } fallback=|| ()>
                                    <div class=style::divider/>
                                </Show>
                                <div class=style::group_label>{*label}</div>
                                {items
                                    .iter()
                                    .copied()
                                    .map(|p| {
                                        let choose = move |_| {
                                            process_name.set(p);
                                            open.set(false);
                                        };
                                        view! {
                                            <button
                                                type="button"
                                                role="option"
                                                aria-selected=move || process_name.get() == p
                                                class=move || {
                                                    if process_name.get() == p {
                                                        style::option_active
                                                    } else {
                                                        style::option
                                                    }
                                                }
                                                on:click=choose
                                            >
                                                {process_title(p)}
                                                <Show when=move || process_name.get() == p fallback=|| ()>
                                                    <svg
                                                        class=style::check
                                                        viewBox="0 0 24 24" fill="none"
                                                        stroke="currentColor" stroke-width="2.4"
                                                        stroke-linecap="round" stroke-linejoin="round"
                                                    >
                                                        <path d="M20 6L9 17l-5-5"/>
                                                    </svg>
                                                </Show>
                                            </button>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            }
                        })
                        .collect::<Vec<_>>()}
                    </div>
                    </Show>
                </Portal>
        </div>
    }
}
