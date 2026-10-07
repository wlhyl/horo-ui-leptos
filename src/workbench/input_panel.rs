//! 工作台左侧输入面板（对应原版 input-panel.component）。
//!
//! 出生数据 / 天象数据两段（与当前可打开的窗口类型匹配），分节可折叠；
//! 编辑实时写回 HoroStorage（跨页面持久化）；底部按钮组打开星盘窗口，
//! 打开时以当前表单数据为快照，后续编辑不影响已开窗口。
use std::cell::Cell;

use leptos::prelude::*;
use reactive_stores::Store;

use crate::components::{
    AlertDialog, ArchiveSelector, DateTimeInput, FormState, FormStateStoreFields, GeoInput,
    HouseSelect, ProcessTypeSelect,
};
use crate::direction::utils::{
    ARC_TO_DATE_METHODS, DAILY_DIRECTION_METHODS, DIRECTION_METHODS, PROCESS_OPTIONS, method_select,
};
use crate::enums::planet::TRADITIONAL_PLANETS;
use crate::enums::process_name::ProcessName;
use crate::models::datetime::DateTimeData;
use crate::render::glyphs::planet_glyph;
use crate::storage::HoroStorage;
use crate::workbench::window::{ChartType, WindowMgr, WindowRect};

// 作用域样式：src/workbench/input_panel.module.css
stylance::import_crate_style!(style, "src/workbench/input_panel.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);

#[component]
pub fn InputPanel(work_area: NodeRef<leptos::html::Div>) -> impl IntoView {
    let storage = use_context::<HoroStorage>().expect("HoroStorage 未初始化");
    let mgr = use_context::<WindowMgr>().expect("WindowMgr 未初始化");

    // 两段表单独立状态：进入页面时从本地缓存恢复
    let native_state: Store<FormState> = Store::new(storage.horo_data().into());
    let event_state: Store<FormState> = Store::new(storage.event_data().into());

    // 分节折叠
    let show_native = RwSignal::new(true);
    let show_derived = RwSignal::new(false);
    let show_process = RwSignal::new(false);
    let show_event = RwSignal::new(false);

    // —— 推运数据（对应原版 input-panel 的 processData 节）：日期 / 居住地共用
    // FormState 作编辑载体，推运类型 / 算法 / 日返月亮为独立信号，实时写回缓存 ——
    let base = storage.process_data();
    let process_state: Store<FormState> = Store::new(FormState {
        id: 0,
        name: String::new(),
        sex: true,
        year: base.date.year,
        month: base.date.month,
        day: base.date.day,
        hour: base.date.hour,
        minute: base.date.minute,
        second: base.date.second,
        tz: base.date.tz,
        st: base.date.st,
        geo_name: base.geo_name.clone(),
        long: base.geo.long,
        lat: base.geo.lat,
        house: storage.horo_data().house,
    });
    // 缓存里的推运类型可能是其他推运盘的旧值，恢复时钳制到本面板支持的六种
    let process_name = RwSignal::new(match base.process_name {
        p if PROCESS_OPTIONS.contains(&p) => p,
        _ => ProcessName::Direction,
    });
    let is_solar_return = RwSignal::new(base.is_solar_return);
    let direction_method = RwSignal::new(base.direction_method);
    let arc_to_date_method = RwSignal::new(base.arc_to_date_method);
    let daily_direction_method = RwSignal::new(base.daily_direction_method);
    // 其余推运类型的算法字段本面板不编辑，保持缓存原值
    let preserved = StoredValue::new((
        base.profection_arc_to_date_method,
        base.secondary_progression_method,
        base.lunar_day_profection_method,
    ));

    // 推运数据实时写回本地缓存（对应原版 onProcessDataChange）；Effect 首次运行
    // 会把恢复值原样写回一次，无副作用。
    {
        let storage = storage;
        Effect::new(move |_| {
            let process = crate::models::data::ProcessData {
                date: DateTimeData {
                    year: process_state.year().get(),
                    month: process_state.month().get(),
                    day: process_state.day().get(),
                    hour: process_state.hour().get(),
                    minute: process_state.minute().get(),
                    second: process_state.second().get(),
                    tz: process_state.tz().get(),
                    st: process_state.st().get(),
                },
                geo_name: process_state.geo_name().get(),
                geo: crate::models::geo::GeoPosition {
                    long: process_state.long().get(),
                    lat: process_state.lat().get(),
                },
                process_name: process_name.get(),
                is_solar_return: is_solar_return.get(),
                direction_method: direction_method.get(),
                arc_to_date_method: arc_to_date_method.get(),
                profection_arc_to_date_method: preserved.get_value().0,
                daily_direction_method: daily_direction_method.get(),
                secondary_progression_method: preserved.get_value().1,
                lunar_day_profection_method: preserved.get_value().2,
            };
            storage.set_process_data(process);
        });
    }

    // 夏令时提示（两段共用一个对话框，消息带前缀区分）
    let dst_alert = RwSignal::new(String::new());

    // —— 编辑实时写回本地缓存（对应原版 onHoroDataChange / onEventDataChange）——
    // Effect 首次运行会把恢复值原样写回一次，无副作用。
    {
        let storage = storage;
        Effect::new(move || {
            let horo = (&*native_state.read()).into();
            storage.set_horo_data(horo);
        });
    }
    {
        let storage = storage;
        Effect::new(move || {
            let horo = (&*event_state.read()).into();
            storage.set_event_data(horo);
        });
    }

    // —— 夏令时提示：日期字段变化且处于中国夏令时区间时弹框（复用 input 页模式）——
    let prev_native_date = Cell::new(None::<DateTimeData>);
    {
        let dst_alert = dst_alert;
        Effect::new(move || {
            let d = read_date(&native_state);
            maybe_alert_dst(&prev_native_date, &d, "出生时间", &dst_alert);
        });
    }
    let prev_event_date = Cell::new(None::<DateTimeData>);
    {
        let dst_alert = dst_alert;
        Effect::new(move || {
            let d = read_date(&event_state);
            maybe_alert_dst(&prev_event_date, &d, "天象时间", &dst_alert);
        });
    }

    // 打开窗口：取当前表单快照 + 工作区尺寸（级联摆放 / 贴边夹取用）。
    // 衍生盘以出生数据为基准（对齐原版 onOpenChart 取 horoData），
    // 基准行星取 storage 中的当前值一并快照进窗口；方向推运 / 返照盘类窗口
    // 另携推运数据快照（面板 Effect 实时写回，storage 即当前值）。
    let open_chart = move |chart_type: ChartType| {
        let snapshot = match chart_type {
            ChartType::Event => (&*event_state.read()).into(),
            ChartType::Native
            | ChartType::Derived
            | ChartType::Direction
            | ChartType::DailyDirection
            | ChartType::SolarArc
            | ChartType::SolarReturn
            | ChartType::LunarReturn
            | ChartType::DailyReturn => (&*native_state.read()).into(),
        };
        let area = work_area
            .get()
            .map(|el| {
                WindowRect::new(
                    0.0,
                    0.0,
                    el.client_width() as f64,
                    el.client_height() as f64,
                )
            })
            .unwrap_or_else(|| WindowRect::new(0.0, 0.0, 800.0, 600.0));
        let derived_planet =
            (chart_type == ChartType::Derived).then(|| storage.derived_planet_name());
        let process = chart_type.is_process().then(|| storage.process_data());
        mgr.open(chart_type, snapshot, area, derived_planet, process);
    };
    let open_native = move |_| open_chart(ChartType::Native);
    let open_event = move |_| open_chart(ChartType::Event);
    let open_derived = move |_| open_chart(ChartType::Derived);
    let open_direction = move |_| open_chart(ChartType::Direction);
    let open_daily_direction = move |_| open_chart(ChartType::DailyDirection);
    let open_solar_arc = move |_| open_chart(ChartType::SolarArc);
    let open_solar_return = move |_| open_chart(ChartType::SolarReturn);
    let open_lunar_return = move |_| open_chart(ChartType::LunarReturn);
    let open_daily_return = move |_| open_chart(ChartType::DailyReturn);

    view! {
        <div class=style::panel>
            // —— 出生数据段 ——
            <section class=style::section>
                <button
                    class=style::section_header
                    on:click=move |_| show_native.update(|v| *v = !*v)
                >
                    <span class=style::section_title>"出生数据"</span>
                    <svg
                        class=move || if show_native.get() { style::chevron_up } else { style::chevron }
                        viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                    >
                        <path d="M6 9l6 6 6-6"/>
                    </svg>
                </button>
                <Show when=move || show_native.get()>
                    <div class=style::section_body>
                        <div class=style::archive_row>
                            <ArchiveSelector state=native_state/>
                        </div>
                        <div class=form::field>
                            <label>"姓名"</label>
                            <div class=form::control>
                                <input type="text" placeholder="可选" bind:value=native_state.name()/>
                            </div>
                        </div>
                        <div class=form::field>
                            <label>"性别"</label>
                            <div class=form::control>
                                <div class=form::radio_group>
                                    <label>
                                        <input
                                            type="radio"
                                            name="wb-native-sex"
                                            prop:checked=move || native_state.sex().get()
                                            on:click=move |_| native_state.sex().set(true)
                                        />
                                        <span>"男"</span>
                                    </label>
                                    <label>
                                        <input
                                            type="radio"
                                            name="wb-native-sex"
                                            prop:checked=move || !native_state.sex().get()
                                            on:click=move |_| native_state.sex().set(false)
                                        />
                                        <span>"女"</span>
                                    </label>
                                </div>
                            </div>
                        </div>
                        <DateTimeInput state=native_state/>
                        <GeoInput state=native_state/>
                        <HouseSelect state=native_state/>
                    </div>
                </Show>
            </section>

            // —— 衍生盘数据段（对齐原版 workbench：仅基准行星一项，默认收起）——
            <section class=style::section>
                <button
                    class=style::section_header
                    on:click=move |_| show_derived.update(|v| *v = !*v)
                >
                    <span class=style::section_title>"衍生盘数据"</span>
                    <svg
                        class=move || if show_derived.get() { style::chevron_up } else { style::chevron }
                        viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                    >
                        <path d="M6 9l6 6 6-6"/>
                    </svg>
                </button>
                <Show when=move || show_derived.get()>
                    <div class=style::section_body>
                        <div class=form::field>
                            <label>"基准行星"</label>
                            <div class=form::control>
                                <select
                                    on:change=move |ev| {
                                        // 编辑实时写回 HoroStorage（对齐原版
                                        // onDerivedPlanetNameChange → storage）
                                        let v = event_target_value(&ev);
                                        if let Some(p) = TRADITIONAL_PLANETS
                                            .iter()
                                            .copied()
                                            .find(|p| p.to_string() == v)
                                        {
                                            storage.set_derived_planet_name(p);
                                        }
                                    }
                                >
                                    {TRADITIONAL_PLANETS
                                        .iter()
                                        .copied()
                                        .map(|p| {
                                            let selected =
                                                move || storage.derived_planet_name() == p;
                                            view! {
                                                <option value=p.to_string() selected=selected>{planet_glyph(p)}</option>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </select>
                            </div>
                        </div>
                        <p class=style::hint>
                            "衍生盘以出生数据为基准，基准行星取其斜升作为中天；切换行星只影响新打开的衍生盘窗口"
                        </p>
                    </div>
                </Show>
            </section>

            // —— 推运数据段（对应原版 input-panel 的 processData 节，默认收起）——
            <section class=style::section>
                <button
                    class=style::section_header
                    on:click=move |_| show_process.update(|v| *v = !*v)
                >
                    <span class=style::section_title>"推运数据"</span>
                    <svg
                        class=move || if show_process.get() { style::chevron_up } else { style::chevron }
                        viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                    >
                        <path d="M6 9l6 6 6-6"/>
                    </svg>
                </button>
                <Show when=move || show_process.get()>
                    <div class=style::section_body>
                        <div class=form::field>
                            <label>"推运类型"</label>
                            <div class=form::control>
                                // 自定义分组下拉（方向推运 / 返照盘两组，组间分隔线）；
                                // 选择写回 process_name 信号，下方 Effect 实时落 localStorage
                                <ProcessTypeSelect process_name/>
                            </div>
                        </div>
                        <DateTimeInput state=process_state/>
                        <GeoInput state=process_state/>
                        <Show when=move || process_name.get() == ProcessName::Direction fallback=|| ()>
                            {method_select("主限算法", direction_method, &DIRECTION_METHODS)}
                            {method_select("换算方式", arc_to_date_method, &ARC_TO_DATE_METHODS)}
                        </Show>
                        <Show
                            when=move || process_name.get() == ProcessName::DailyDirection
                            fallback=|| ()
                        >
                            {method_select("方向弧算法", daily_direction_method, &DAILY_DIRECTION_METHODS)}
                        </Show>
                        // 日返月亮 / 日返月返：月返 / 每日回归 / 每日回归方向弧可基于
                        // 日返逐层取返照时刻（标签随类型变化，对齐原版 process.page）
                        <Show
                            when=move || {
                                matches!(
                                    process_name.get(),
                                    ProcessName::DailyDirection
                                        | ProcessName::LunarReturn
                                        | ProcessName::DailyReturn
                                )
                            }
                            fallback=|| ()
                        >
                            <div class=form::field>
                                <label>{move || {
                                    if process_name.get() == ProcessName::DailyReturn {
                                        "日返月返"
                                    } else {
                                        "日返月亮"
                                    }
                                }}</label>
                                <div class=form::control>
                                    <label class=form::toggle>
                                        <input type="checkbox" bind:checked=is_solar_return/>
                                        <span>"开启"</span>
                                    </label>
                                </div>
                            </div>
                        </Show>
                        <p class=style::hint>
                            "推运数据实时写回本地缓存；方向推运 / 返照盘窗口按打开时刻的快照计算"
                        </p>
                    </div>
                </Show>
            </section>

            // —— 天象数据段（对齐原版 workbench：不含姓名 / 性别）——
            <section class=style::section>
                <button
                    class=style::section_header
                    on:click=move |_| show_event.update(|v| *v = !*v)
                >
                    <span class=style::section_title>"天象数据"</span>
                    <svg
                        class=move || if show_event.get() { style::chevron_up } else { style::chevron }
                        viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                    >
                        <path d="M6 9l6 6 6-6"/>
                    </svg>
                </button>
                <Show when=move || show_event.get()>
                    <div class=style::section_body>
                        <DateTimeInput state=event_state/>
                        <GeoInput state=event_state/>
                        <HouseSelect state=event_state/>
                    </div>
                </Show>
            </section>

            // —— 添加星盘 ——
            <section class=style::section>
                <div class=style::section_header_static>
                    <span class=style::section_title>"添加星盘"</span>
                </div>
                <div class=style::section_body>
                    <div class=style::chart_buttons>
                        <button class=style::chart_btn on:click=open_native>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linejoin="round">
                                <path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2z"/>
                            </svg>
                            "本命盘"
                        </button>
                        <button class=style::chart_btn on:click=open_event>
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                <circle cx="12" cy="12" r="10"/>
                                <path d="M2 12h20"/>
                                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                            </svg>
                            "天象盘"
                        </button>
                        <button class=style::chart_btn on:click=open_derived>
                            // 顺时针环形箭头，同首页衍生盘入口
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M21 12a9 9 0 1 1-2.64-6.36"/>
                                <path d="M21 3v6h-6"/>
                            </svg>
                            "衍生盘"
                        </button>
                        // 分组分隔线：命盘 / 方向推运 / 返照盘三组（与推运类型
                        // 下拉的 optgroup 分组对应）
                        <div class=style::chart_divider/>
                        <button class=style::chart_btn on:click=open_direction>
                            // 沙漏轮廓，同首页推运入口
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M6 3h12"/>
                                <path d="M6 21h12"/>
                                <path d="M7 3v3a5 5 0 0 0 5 5 5 5 0 0 0 5-5V3"/>
                                <path d="M7 21v-3a5 5 0 0 1 5-5 5 5 0 0 1 5 5v3"/>
                            </svg>
                            "主向推运"
                        </button>
                        <button class=style::chart_btn on:click=open_daily_direction>
                            // 每日回归：单向循环箭头加节点
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M4 12a8 8 0 0 1 14-5.3"/>
                                <path d="M18 2v5h-5"/>
                                <path d="M20 12a8 8 0 0 1-14 5.3"/>
                                <path d="M6 22v-5h5"/>
                            </svg>
                            "每日回归方向弧"
                        </button>
                        <button class=style::chart_btn on:click=open_solar_arc>
                            // 太阳弧：日轮加运动弧线
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="4"/>
                                <path d="M12 2v3"/>
                                <path d="M12 19v3"/>
                                <path d="M2 12h3"/>
                                <path d="M19 12h3"/>
                                <path d="M4.9 4.9l2.1 2.1"/>
                                <path d="M17 17l2.1 2.1"/>
                            </svg>
                            "太阳弧"
                        </button>
                        // 分组分隔线：方向推运与返照盘两组按钮之间
                        <div class=style::chart_divider/>
                        <button class=style::chart_btn on:click=open_solar_return>
                            // 日返：环形回到起点的箭头环绕日轮
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="3.5"/>
                                <path d="M21 12a9 9 0 1 1-2.64-6.36"/>
                                <path d="M21 3v6h-6"/>
                            </svg>
                            "日返"
                        </button>
                        <button class=style::chart_btn on:click=open_lunar_return>
                            // 月返：月牙轮廓
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/>
                            </svg>
                            "月返"
                        </button>
                        <button class=style::chart_btn on:click=open_daily_return>
                            // 每日回归：双向循环箭头
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M3 12a9 9 0 0 1 15-6.7L21 8"/>
                                <path d="M21 3v5h-5"/>
                                <path d="M21 12a9 9 0 0 1-15 6.7L3 16"/>
                                <path d="M3 21v-5h5"/>
                            </svg>
                            "每日回归"
                        </button>
                    </div>
                    <p class=style::hint>"窗口以打开时的数据为准，之后修改面板不影响已开窗口"</p>
                </div>
            </section>

            // 夏令时提示弹框
            <AlertDialog header="夏令时提示" message=dst_alert/>
        </div>
    }
}

/// 读取一段表单的日期字段（Effect 中逐字段读取，只追踪日期字段）。
fn read_date(state: &Store<FormState>) -> DateTimeData {
    DateTimeData {
        year: state.year().get(),
        month: state.month().get(),
        day: state.day().get(),
        hour: state.hour().get(),
        minute: state.minute().get(),
        second: state.second().get(),
        tz: state.tz().get(),
        st: state.st().get(),
    }
}

/// 日期变化且处于中国夏令时区间时弹提示（不区分是否已勾选 st，对齐 input 页）。
fn maybe_alert_dst(
    prev: &Cell<Option<DateTimeData>>,
    date: &DateTimeData,
    label: &str,
    alert: &RwSignal<String>,
) {
    if let Some(p) = prev.get() {
        if *date != p && date.is_in_chinese_dst() && (date.tz as i32) == 8 {
            alert.set(format!(
                "{label}：{}年{}月{}日处于中国夏令时实施期间，请确认是否需要勾选夏令时。",
                date.year, date.month, date.day
            ));
        }
    }
    prev.set(Some(*date));
}
