//! 通用自定义下拉（触发按钮 + 主题化弹层选项列表，替代原生 `<select>`）。
//!
//! 原生 `<select>` 的弹开列表由浏览器渲染，无法定制分组 / 悬停 / 选中样式，
//! 故全项目下拉统一以「触发按钮 + Portal 弹层」实现：选项支持扁平或分组
//! （组标题为空串时不渲染标题，多组间渲染分隔线）；触发器外观对齐
//! styles/main.css 的 select 底座。弹层以 fixed 定位按触发器实时位置计算
//! 坐标（脱离侧栏滚动容器的裁剪），下方空间不足时向上弹出；点击遮罩、
//! 滚轮 / 触摸滚动或按 Escape 收起。
use leptos::control_flow::Show;
use leptos::portal::Portal;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

// 作用域样式：src/components/select_popup/select_popup.module.css
stylance::import_crate_style!(
    style,
    "src/components/select_popup/select_popup.module.css"
);

/// 弹层展开所需的最小下方空间（px）：不足时改为向上弹出。
const DROPDOWN_SPACE_PX: f64 = 320.0;
/// 弹层与触发器的间距、与视口边缘的最小留白（px）。
const POPUP_GAP_PX: f64 = 6.0;
const VIEWPORT_MARGIN_PX: f64 = 8.0;
/// 弹层内容全展开的高度上限（px）：空间充裕时整体可见，否则内部滚动。
const POPUP_MAX_HEIGHT_PX: f64 = 380.0;

/// 下拉选项组：组标题为空串时不渲染标题（扁平选项），多组间渲染分隔线。
pub struct SelectGroup<T> {
    pub label: &'static str,
    pub items: Vec<T>,
}

/// 静态扁平选项 → 单组弹层选项（无组标题）。
pub fn flat_group<T: Copy>(items: impl IntoIterator<Item = T>) -> Vec<SelectGroup<T>> {
    vec![SelectGroup {
        label: "",
        items: items.into_iter().collect(),
    }]
}

/// 常量分组表 `(组标题, 组内选项)` → 弹层选项组。
pub fn grouped_items<T: Copy>(src: &[(&'static str, &'static [T])]) -> Vec<SelectGroup<T>> {
    src.iter()
        .map(|(label, items)| SelectGroup {
            label,
            items: items.to_vec(),
        })
        .collect()
}

/// 通用自定义下拉：`current` 给当前值（None 时触发器显示占位文案），点选经
/// `on_pick` 写回调用方的信号；`groups` 在弹层展开时求值（支持运行期增减
/// 选项），`label` 生成触发器与选项共用的文案。
#[component]
pub fn SelectPopup<T, C, P, G, L>(
    current: C,
    on_pick: P,
    groups: G,
    label: L,
    /// 无值时的触发器占位文案
    #[prop(optional, into)] placeholder: String,
    /// 紧凑变体：行内场景（如「添加行星」按钮排）不占满整行
    #[prop(optional)] compact: bool,
) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
    C: Fn() -> Option<T> + Send + Sync + Copy + 'static,
    P: Fn(T) + Send + Sync + Copy + 'static,
    G: Fn() -> Vec<SelectGroup<T>> + Send + Sync + Clone + 'static,
    L: Fn(T) -> String + Send + Sync + Copy + 'static,
{
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

    // 弹层内容闭包经 Copy 的 StoredValue 捕获 groups：若直接 move 捕获泛型
    // G（非 Copy），创建闭包时会把它从 Show children（须为 Fn）的环境中
    // 按值移走，令外层闭包降级为 FnOnce（见 view! 内注释）
    let groups_stored = StoredValue::new(groups);

    let wrap_class = if compact { style::compact } else { style::wrap };

    view! {
        <div class=wrap_class>
            <button
                type="button"
                node_ref=trigger_ref
                class=style::trigger
                aria-haspopup="listbox"
                aria-expanded=move || open.get()
                on:click=toggle
            >
                <span>{move || {
                    current()
                        .map(label)
                        .unwrap_or_else(|| placeholder.clone())
                }}</span>
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
                        {move || {
                            // groups_stored 是 Copy 句柄，弹层闭包以 Copy 捕获，
                            // 不会破坏外层 Show children 的 Fn 约束；groups 本体
                            // 在弹层展开时经 get_value 取出求值（支持动态分组）
                            let groups = groups_stored.get_value();
                            groups()
                                .iter()
                                .enumerate()
                                .map(|(i, group)| {
                                    let group_label = group.label;
                                    let items = group.items.clone();
                                    view! {
                                        <Show when=move || { i > 0 } fallback=|| ()>
                                            <div class=style::divider/>
                                        </Show>
                                        <Show when=move || !group_label.is_empty() fallback=|| ()>
                                            <div class=style::group_label>{group_label}</div>
                                        </Show>
                                        {items
                                            .iter()
                                            .copied()
                                            .map(|item| {
                                                let pick = move |_| {
                                                    on_pick(item);
                                                    open.set(false);
                                                };
                                                view! {
                                                    <button
                                                        type="button"
                                                        role="option"
                                                        aria-selected=move || current() == Some(item)
                                                        class=move || {
                                                            if current() == Some(item) {
                                                                style::option_active
                                                            } else {
                                                                style::option
                                                            }
                                                        }
                                                        on:click=pick
                                                    >
                                                        {label(item)}
                                                        <Show when=move || current() == Some(item) fallback=|| ()>
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
                        }
                    </div>
                </Show>
            </Portal>
        </div>
    }
}
