//! 星盘时间编辑条（对应原版 image 页的时间控件 + date-time 选择面板）。
//!
//! 「选择面板」复刻原版 horo-date-time 的 ion-picker 弹层：点击日期标签弹出
//! 六列滚轮（年 1900–2199 / 月 / 日 / 时 / 分 / 秒），取消丢弃、确定才把草稿
//! 写入本地信号——弹层内滚动不触发重绘；日列候选项随草稿年月自适应当月天数，
//! 年 / 月变化把日钳制到当月最大。「现在」一键填当前时间（对应原版 nowDate）。
//! 「步进器」按档位 −/+ 一次一个单位（溢出进位，1月31日 +1月 → 3月2日）。
//! 全部只写传入的本地信号，不触碰 HoroStorage（不落盘）；防抖与重新请求
//! 由使用方（星盘页 / 工作台窗口）负责。
use leptos::control_flow::Show;
use leptos::portal::Portal;
use leptos::prelude::*;

use wasm_bindgen_futures::spawn_local;

use crate::models::datetime::{DateTimeData, StepUnit};
use crate::shared::sleep_ms;

// 作用域样式：src/components/chart_time_editor/chart_time_editor.module.css
stylance::import_crate_style!(
    style,
    "src/components/chart_time_editor/chart_time_editor.module.css"
);

/// 滚轮停止多久视为选定（毫秒；连续滚动只保留最后一格）。
const WHEEL_SETTLE_MS: u32 = 120;
/// 每个候选项的行高（px），与 CSS 的 .opt 行高、.col 高度严格对应。
const ITEM_H: f64 = 36.0;

/// 滚轮列对应的字段。
#[derive(Clone, Copy, PartialEq)]
enum Field {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

impl Field {
    /// 读字段值（滚轮定位用）。
    fn get(self, d: &DateTimeData) -> i64 {
        match self {
            Field::Year => d.year as i64,
            Field::Month => d.month as i64,
            Field::Day => d.day as i64,
            Field::Hour => d.hour as i64,
            Field::Minute => d.minute as i64,
            Field::Second => d.second as i64,
        }
    }

    /// 写回字段（候选值恒在合法范围内）。
    fn set(self, d: &mut DateTimeData, v: i64) {
        match self {
            Field::Year => d.year = v as i32,
            Field::Month => d.month = v as u8,
            Field::Day => d.day = v as u8,
            Field::Hour => d.hour = v as u8,
            Field::Minute => d.minute = v as u8,
            Field::Second => d.second = v as u8,
        }
    }

    /// 列标签。
    fn label(self) -> &'static str {
        match self {
            Field::Year => "年",
            Field::Month => "月",
            Field::Day => "日",
            Field::Hour => "时",
            Field::Minute => "分",
            Field::Second => "秒",
        }
    }
}

/// 候选项列表（升序）；日列随年月自适应当月天数，其余与原版一致：
/// 年为 1900 年起共 300 个，月 1–12，时 0–23，分 / 秒 0–59。
fn candidates(field: Field, d: DateTimeData) -> Vec<i64> {
    match field {
        Field::Year => (1900..=2199).collect(),
        Field::Month => (1..=12).collect(),
        Field::Day => (1..=i64::from(d.days_in_month_of())).collect(),
        Field::Hour => (0..=23).collect(),
        Field::Minute | Field::Second => (0..=59).collect(),
    }
}

/// 值在升序候选中的下标；不在候选里（如日被钳制前的旧值）时取不超过它的最近项。
fn locate(vals: &[i64], v: i64) -> usize {
    match vals.binary_search(&v) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    }
}

/// 单列滚轮：滚动落定 / 点选候选项后写回草稿；外部值变化（确定回填、
/// 日被钳制）时自动滚到对应项。已在目标位则不重设，避免与用户滚动打架。
#[component]
fn WheelColumn(draft: RwSignal<DateTimeData>, field: Field) -> impl IntoView {
    let values = Signal::derive(move || candidates(field, draft.get()));
    let value = Signal::derive(move || field.get(&draft.get()));
    let scroll_ref = NodeRef::<leptos::html::Div>::new();
    // 滚动落定代数：每次滚动 / 点选自增，休眠结束代数不符说明又被滚动了，丢弃
    let scroll_gen = RwSignal::new(0u64);

    // 写回草稿（滚动落定与点选共用）；年 / 月变化把日钳制到当月
    let apply = move |v: i64| {
        draft.update(|d| {
            if field.get(d) != v {
                field.set(d, v);
                d.clamp_day();
            }
        });
    };

    // 外部值变化 → 滚到对应项
    Effect::new(move |_| {
        let v = value.get();
        let vals = values.get();
        let Some(el) = scroll_ref.get() else {
            return;
        };
        let target = (locate(&vals, v) as f64 * ITEM_H) as i32;
        if (el.scroll_top() - target).abs() > 1 {
            el.set_scroll_top(target);
        }
    });

    // 滚动落定后写回：代数防抖只保留最后一次
    let on_scroll = move |_| {
        scroll_gen.update(|g| *g += 1);
        let curr_gen = scroll_gen.get_untracked();
        spawn_local(async move {
            sleep_ms(WHEEL_SETTLE_MS as i32).await;
            if scroll_gen.get_untracked() != curr_gen {
                return;
            }
            let Some(el) = scroll_ref.get() else {
                return;
            };
            let vals = values.get_untracked();
            if vals.is_empty() {
                return;
            }
            let idx = ((el.scroll_top() as f64 / ITEM_H).round() as usize).min(vals.len() - 1);
            apply(vals[idx]);
        });
    };

    view! {
        <div class=style::col>
            <div class=style::col_scroll node_ref=scroll_ref on:scroll=on_scroll>
                <For
                    each=move || values.get()
                    key=|v| *v
                    children=move |v| {
                        view! {
                            <div
                                class=move || {
                                    if value.get() == v { style::opt_active } else { style::opt }
                                }
                                on:click=move |_| {
                                    scroll_gen.update(|g| *g += 1);
                                    apply(v);
                                }
                            >{v}</div>
                        }
                    }
                />
            </div>
            <div class=style::col_mask></div>
            <span class=style::col_label>{field.label()}</span>
        </div>
    }
}

/// 步进档位（循环顺序即点击切换顺序，初值取第一档「年」，对齐原版 stepUnit）。
const UNITS: [(StepUnit, &str); 6] = [
    (StepUnit::Year, "年"),
    (StepUnit::Month, "月"),
    (StepUnit::Day, "日"),
    (StepUnit::Hour, "时"),
    (StepUnit::Minute, "分"),
    (StepUnit::Second, "秒"),
];

/// 星盘时间编辑条：日期标签（弹选择面板）+ 现在按钮 + 档位步进器。
#[component]
pub fn ChartTimeEditor(date: RwSignal<DateTimeData>) -> impl IntoView {
    // 当前步进档位
    let unit = RwSignal::new(StepUnit::Year);
    let do_step = move |amount: i32| {
        let u = unit.get_untracked();
        date.update(|d| *d = d.stepped(u, amount));
    };

    // 点击档位按钮在年→月→日→时→分→秒→年之间循环（对应原版 changeStepUnit）
    let cycle_unit = move |_| {
        unit.update(|u| {
            let i = UNITS.iter().position(|&(s, _)| s == *u).unwrap_or(0);
            *u = UNITS[(i + 1) % UNITS.len()].0;
        });
    };
    // 档位按钮文案：「1年」「1月」…（对齐原版 `1{{ stepUnit }}`）
    let unit_label = move || {
        let i = UNITS.iter().position(|&(s, _)| s == unit.get()).unwrap_or(0);
        format!("1{}", UNITS[i].1)
    };

    // 选择面板：打开时复制当前日期为草稿；弹层内滚动只改草稿，确定才写回，
    // 写回后由使用方的防抖 Effect 重新请求并重绘（取消则丢弃草稿）。
    let open = RwSignal::new(false);
    let draft = RwSignal::new(date.get_untracked());
    let open_panel = move |_| {
        draft.set(date.get_untracked());
        open.set(true);
    };
    let cancel = move |_| open.set(false);
    let confirm = move |_| {
        let d = draft.get_untracked();
        // 值未变时不写回，避免触发一次无谓的重算请求
        if d != date.get_untracked() {
            date.set(d);
        }
        open.set(false);
    };
    // 「现在」：与原版 nowDate 一致，立即生效
    let now = move |_| date.set(DateTimeData::now());

    view! {
        <div class=style::editor>
            <div class=style::label_row>
                <button class=style::date_label data-tip="选择日期时间" on:click=open_panel>
                    {move || {
                        let d = date.get();
                        format!(
                            "{}-{}-{} {}:{}:{}",
                            d.year, d.month, d.day, d.hour, d.minute, d.second
                        )
                    }}
                </button>
                <button class=style::now_btn aria-label="现在" data-tip="现在" on:click=now>
                    // 时钟图标（对应原版 ionicons time-outline）
                    <svg viewBox="0 0 512 512" fill="none" stroke="currentColor"
                        stroke-width="32" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="256" cy="256" r="208"/>
                        <polyline points="256 148 256 268 336 268"/>
                    </svg>
                </button>
            </div>

            <div class=style::stepper>
                <button class=style::arrow aria-label="减少" data-tip="减少" on:click=move |_| do_step(-1)>"−"</button>
                <button class=style::unit_cycle data-tip="点击切换步进单位" on:click=cycle_unit>
                    {unit_label}
                </button>
                <button class=style::arrow aria-label="增加" data-tip="增加" on:click=move |_| do_step(1)>"+"</button>
            </div>
        </div>

        // 选择面板：遮罩 + 底部弹层（原版 ion-modal 底部对齐，确定 / 取消收尾）。
        // Portal 把弹层挂到 body：卡片容器带入场动画的 transform（fill-mode: both
        // 使其永久生效），会让 position:fixed 相对卡片而非视口定位，把弹层挤到屏外。
        <Show when=move || open.get() fallback=|| ()>
            <Portal>
                <div class=style::backdrop on:click=cancel></div>
                <div class=style::sheet>
                <div class=style::sheet_head>
                    <button class=style::head_btn on:click=cancel>"取消"</button>
                    <span class=style::sheet_title>"选择日期时间"</span>
                    <button
                        class=move || format!("{} {}", style::head_btn, style::confirm_btn)
                        on:click=confirm
                    >"确定"</button>
                </div>
                <div class=style::picker>
                    <WheelColumn draft field=Field::Year />
                    <WheelColumn draft field=Field::Month />
                    <WheelColumn draft field=Field::Day />
                    <WheelColumn draft field=Field::Hour />
                    <WheelColumn draft field=Field::Minute />
                    <WheelColumn draft field=Field::Second />
                </div>
                </div>
            </Portal>
        </Show>
    }
}
