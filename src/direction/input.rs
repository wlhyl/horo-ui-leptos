//! 推运输入页（/process，对应原版 process.page，仅方向推运相关项）。
//!
//! 出生数据写回 `storage.horo_data`，推运参数（推运类型 / 推运时间 / 现居住地 /
//! 算法 / 日返月亮）写回 `storage.process_data`，提交后按推运类型导航到对应结果页。
use std::cell::Cell;

use leptos::control_flow::Show;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use reactive_stores::Store;

use crate::components::{
    AlertDialog, ArchiveSelector, DateTimeInput, FormState, FormStateStoreFields, GeoInput,
    HouseSelect, ProcessTypeSelect,
};
use crate::direction::utils::{
    ARC_TO_DATE_METHODS, DAILY_DIRECTION_METHODS, DIRECTION_METHODS, PROCESS_OPTIONS, method_select,
};
use crate::enums::process_name::ProcessName;
use crate::models::{
    data::{HoroData, ProcessData},
    datetime::DateTimeData,
    geo::GeoPosition,
};
use crate::routes::AppRoute;
use crate::storage::HoroStorage;

// 作用域样式：src/direction/input.module.css
stylance::import_crate_style!(style, "src/direction/input.module.css");
stylance::import_crate_style!(card, "src/shared/card.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);

#[component]
pub(crate) fn ProcessInput() -> impl IntoView {
    let storage = use_context::<HoroStorage>().expect("HoroStorage 未初始化");
    // 必须在渲染期（Router 上下文内）取导航器，事件闭包里调用会拿不到上下文
    let nav = use_navigate();

    // —— 出生数据：从本地缓存恢复（对应原版 horaData = storage.horoData）——
    let native_state = Store::new(FormState::from(storage.horo_data()));

    // —— 推运数据：从缓存恢复推运时间与居住地（对应原版 processData）。
    // FormState 在此仅作日期 + 经纬度字段的编辑载体，姓名 / 性别 / 宫位不参与。
    let base = storage.process_data();
    let process_state = Store::new(FormState {
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
    // 推运参数（对应原版 processData 各字段；缓存里可能是其他推运类型的旧值，
    // 恢复时钳制到本页支持的六种）
    let process_name = RwSignal::new(match base.process_name {
        p if PROCESS_OPTIONS.contains(&p) => p,
        _ => ProcessName::Direction,
    });
    let is_solar_return = RwSignal::new(base.is_solar_return);
    let direction_method = RwSignal::new(base.direction_method);
    let arc_to_date_method = RwSignal::new(base.arc_to_date_method);
    let daily_direction_method = RwSignal::new(base.daily_direction_method);

    let err = RwSignal::new(String::new());

    // 夏令时提示：出生日期字段变化且处于中国夏令时区间时弹框（对齐原版
    // onNativeDateChange，推运时间不提示）
    let dst_alert = RwSignal::new(String::new());
    let prev_date = Cell::new(None::<DateTimeData>);
    Effect::new(move |_| {
        let d = DateTimeData {
            year: native_state.year().get(),
            month: native_state.month().get(),
            day: native_state.day().get(),
            hour: native_state.hour().get(),
            minute: native_state.minute().get(),
            second: native_state.second().get(),
            tz: native_state.tz().get(),
            st: native_state.st().get(),
        };
        if let Some(prev) = prev_date.get() {
            if d != prev && d.is_in_chinese_dst() && (d.tz as i32) == 8 {
                dst_alert.set(format!(
                    "{}年{}月{}日处于中国夏令时实施期间，请确认是否需要勾选夏令时。",
                    d.year, d.month, d.day
                ));
            }
        }
        prev_date.set(Some(d));
    });

    // 提交：写回两份缓存并按推运类型导航（对应原版 getProcess）
    let submit = move |_| {
        let n = native_state.read();
        if n.year < 1900 {
            err.set("出生年份需 ≥ 1900".into());
            return;
        }
        if n.long < -180.0 || n.long > 180.0 || n.lat < -90.0 || n.lat > 90.0 {
            err.set("出生地经纬度超出有效范围".into());
            return;
        }
        let horo = HoroData::from(&*n);
        let p = process_state.read();
        if p.year < 1900 {
            err.set("推运年份需 ≥ 1900".into());
            return;
        }
        if p.long < -180.0 || p.long > 180.0 || p.lat < -90.0 || p.lat > 90.0 {
            err.set("现居住地经纬度超出有效范围".into());
            return;
        }
        let process = ProcessData {
            date: p.date(),
            geo_name: p.geo_name.clone(),
            geo: GeoPosition {
                long: p.long,
                lat: p.lat,
            },
            process_name: process_name.get_untracked(),
            is_solar_return: is_solar_return.get_untracked(),
            direction_method: direction_method.get_untracked(),
            arc_to_date_method: arc_to_date_method.get_untracked(),
            // 其余推运类型的算法字段本页不编辑，保持缓存原值
            profection_arc_to_date_method: base.profection_arc_to_date_method,
            daily_direction_method: daily_direction_method.get_untracked(),
            secondary_progression_method: base.secondary_progression_method,
            lunar_day_profection_method: base.lunar_day_profection_method,
        };
        storage.set_horo_data(horo);
        storage.set_process_data(process);
        err.set(String::new());
        let path = match process_name.get_untracked() {
            ProcessName::DailyDirection => AppRoute::DailyDirection.path(),
            ProcessName::SolarArc => AppRoute::SolarArc.path(),
            ProcessName::SolarReturn => AppRoute::ReturnSolar.path(),
            ProcessName::LunarReturn => AppRoute::ReturnLunar.path(),
            ProcessName::DailyReturn => AppRoute::ReturnDaily.path(),
            _ => AppRoute::Direction.path(),
        };
        nav(path, NavigateOptions::default());
    };

    // 日返月亮开关的标签与说明随推运类型变化（对应原版模板的三元标签）
    let solar_return_label = move || {
        if process_name.get() == ProcessName::DailyReturn {
            "日返月返"
        } else {
            "日返月亮"
        }
    };
    let solar_return_hint = move || match process_name.get() {
        ProcessName::LunarReturn => "开启后先求日返返照时刻，再以该时刻计算月返盘",
        ProcessName::DailyReturn => {
            "开启后按 日返→月返→每日回归 逐层求取返照时刻，再以该时刻计算每日回归盘"
        }
        _ => "开启后按 日返→月返→每日回归 逐层求取返照时刻，再以返照时刻计算方向弧",
    };

    view! {
        <div class=card::card>
            <div class=style::header>
                <h2>"推运"</h2>
                <ArchiveSelector state=native_state/>
            </div>
            <div class=style::form_grid>
                <h3 class=style::section_title>"出生数据"</h3>
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
                                    name="process-sex"
                                    prop:checked=move || native_state.sex().get()
                                    on:click=move |_| native_state.sex().set(true)
                                />
                                <span>"男"</span>
                            </label>
                            <label>
                                <input
                                    type="radio"
                                    name="process-sex"
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

                <h3 class=style::section_title>"推运数据"</h3>
                <div class=form::field>
                    <label>"推运类型"</label>
                    <div class=form::control>
                        <ProcessTypeSelect process_name/>
                    </div>
                </div>
                <DateTimeInput state=process_state/>
                <GeoInput state=process_state/>

                // 主限算法 + 换算方式：仅主向推运（对应原版 process.page 197-224 行）
                <Show when=move || process_name.get() == ProcessName::Direction fallback=|| ()>
                    {method_select("主限算法", direction_method, &DIRECTION_METHODS)}
                    {method_select("换算方式", arc_to_date_method, &ARC_TO_DATE_METHODS)}
                </Show>

                // 方向弧算法：仅每日回归方向弧（对应原版 process.page 226-240 行）
                <Show
                    when=move || process_name.get() == ProcessName::DailyDirection
                    fallback=|| ()
                >
                    {method_select("方向弧算法", daily_direction_method, &DAILY_DIRECTION_METHODS)}
                </Show>

                // 日返月亮 / 日返月返：月返 / 每日回归 / 每日回归方向弧可基于日返
                // 逐层取返照时刻（对应原版 process.page 182-195 行，每日回归的标签
                // 为「日返月返」，其余为「日返月亮」）
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
                        <label>{solar_return_label}</label>
                        <div class=form::control>
                            <label class=form::toggle>
                                <input type="checkbox" bind:checked=is_solar_return/>
                                <span>"开启"</span>
                            </label>
                        </div>
                    </div>
                    <p class=style::hint>{solar_return_hint}</p>
                </Show>

                <button class=form::btn_primary on:click=submit>"开始推运"</button>
            </div>
        </div>

        // 表单校验错误以对话框呈现（对应原版 alertController 弹窗）
        <AlertDialog header="错误" message=err/>
        // 夏令时提示弹框（对应原版 onNativeDateChange 的 alertController）
        <AlertDialog header="夏令时提示" message=dst_alert/>
    }
}
