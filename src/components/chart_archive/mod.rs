//! 星盘存档（对应原版 image 组件的「存档」按钮与 alertController 流程，无笔记功能）。
//!
//! 按钮常驻，未登录点击以 AlertDialog 提示（对齐「从档案库选择」的登录提示模式）；
//! 已登录时：档案未存过（id == 0）直接新增，已存过先弹「更新 / 新增 / 取消」三选
//! （对齐原版 alertController 三按钮）。时间取时间编辑条的当前值（页面内改动不落盘，
//! 存档时以此为准）；成功后把档案 id 与当前时间回写 HoroStorage（对齐原版把
//! currentHoroData 写回）。衍生盘不提供存档（对齐原版），由父级按模式控制不渲染。
//!
//! 数据快照放在 StoredValue 中：保存逻辑闭包会被按钮与弹窗回调多处复制共享，
//! 闭包内仅捕获 Copy 的信号与 StoredValue，避免所有权冲突。
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::client::{add_horoscope, update_horoscope};
use crate::api::request::{
    HoroscopeRecordRequest, RecordLocationRequest, UpdateHoroscopeRecordRequest,
};
use crate::api::response::ChartType;
use crate::astro::horo_math::degree_to_dms;
use crate::auth::AuthService;
use crate::components::{AlertAction, AlertDialog};
use crate::models::data::HoroData;
use crate::models::datetime::DateTimeData;
use crate::native::ChartMode;
use crate::storage::HoroStorage;

// 作用域样式：src/components/chart_archive/chart_archive.module.css
stylance::import_crate_style!(
    style,
    "src/components/chart_archive/chart_archive.module.css"
);

/// 十进制经纬度 → 度分秒出生地（半球标志按正负号，对齐原版 addRecord）。
fn record_location(data: &HoroData) -> RecordLocationRequest {
    let long = degree_to_dms(data.geo.long.abs());
    let lat = degree_to_dms(data.geo.lat.abs());
    RecordLocationRequest {
        name: data.geo_name.clone(),
        is_east: data.geo.long >= 0.0,
        longitude_degree: long.0 as u16,
        longitude_minute: long.1 as u8,
        longitude_second: long.2 as u8,
        is_north: data.geo.lat >= 0.0,
        latitude_degree: lat.0 as u8,
        latitude_minute: lat.1 as u8,
        latitude_second: lat.2 as u8,
    }
}

/// 新增档案请求体：时间取当前编辑值；盘类型本命 natal / 天象 horary（按语义，
/// 原版写死 natal）；无笔记功能 description 固定空串，lock false、时间未校准
/// is_time_precise false（均对齐原版 addRecord）。
fn add_request(mode: ChartMode, data: &HoroData, date: DateTimeData) -> HoroscopeRecordRequest {
    HoroscopeRecordRequest {
        name: data.name.clone(),
        gender: data.sex,
        birth_year: date.year,
        birth_month: date.month,
        birth_day: date.day,
        birth_hour: date.hour,
        birth_minute: date.minute,
        birth_second: date.second,
        time_zone_offset: date.tz,
        is_dst: date.st,
        chart_type: match mode {
            ChartMode::Event => ChartType::Horary,
            ChartMode::Native | ChartMode::Derived => ChartType::Natal,
        },
        is_time_precise: false,
        location: record_location(data),
        description: String::new(),
        lock: false,
    }
}

/// 更新档案请求体：表单覆盖的字段全 Some，description / chart_type /
/// is_time_precise / lock 传 null 由后台保持原值（锁定记录后台会拒绝并返回文案）。
fn update_request(data: &HoroData, date: DateTimeData) -> UpdateHoroscopeRecordRequest {
    UpdateHoroscopeRecordRequest {
        name: Some(data.name.clone()),
        gender: Some(data.sex),
        birth_year: Some(date.year),
        birth_month: Some(date.month),
        birth_day: Some(date.day),
        birth_hour: Some(date.hour),
        birth_minute: Some(date.minute),
        birth_second: Some(date.second),
        time_zone_offset: Some(date.tz),
        is_dst: Some(date.st),
        location: Some(record_location(data)),
        description: None,
        chart_type: None,
        is_time_precise: None,
        lock: None,
    }
}

/// 存档成功后回写存储：档案 id 与当前编辑时间一并写入对应缓存（对齐原版把
/// currentHoroData 写回 storage.horoData / eventData），输入页与再次存档都能取到。
fn write_back(
    storage: &HoroStorage,
    mode: ChartMode,
    mut data: HoroData,
    id: u32,
    date: DateTimeData,
) {
    data.id = id;
    data.date = date;
    match mode {
        ChartMode::Event => storage.set_event_data(data),
        ChartMode::Native | ChartMode::Derived => storage.set_horo_data(data),
    }
}

/// 星盘存档入口：按钮 + 三选 / 成功 / 提示 / 错误弹窗。
#[component]
pub fn ChartArchive(
    /// 盘类型：决定 chart_type（本命 natal / 天象 horary）与回写的存储键
    mode: ChartMode,
    /// 星盘页进入时的数据快照（档案 id / 姓名 / 性别 / 地点）
    data: HoroData,
    /// 时间编辑条的当前值（页面内改动不落盘，存档时以此为准）
    date: RwSignal<DateTimeData>,
) -> impl IntoView {
    let auth = use_context::<AuthService>().expect("AuthService 未初始化");
    let storage = use_context::<HoroStorage>().expect("HoroStorage 未初始化");

    // 档案 ID：新增成功后写回，此后再点存档走更新路径（对齐原版回写 native.id）
    let record_id = RwSignal::new(data.id);
    // 数据快照入 StoredValue 供各保存闭包共享
    let data = StoredValue::new(data);
    // 提交中防重复点击：慢网络下连点会新增出多条记录
    let submitting = RwSignal::new(false);
    // 未登录提示 / 三选弹窗 / 成功提示 / 错误透出
    let notice = RwSignal::new(String::new());
    let choice = RwSignal::new(String::new());
    let success = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());

    // 新增档案：成功后记录新 id 并回写存储
    let do_add = move || {
        if submitting.get_untracked() {
            return;
        }
        submitting.set(true);
        let data = data.get_value();
        let token = auth.token().unwrap_or_default();
        let current_date = date.get_untracked();
        spawn_local(async move {
            let res = add_horoscope(&add_request(mode, &data, current_date), &token).await;
            submitting.set(false);
            match res {
                Ok(record) => {
                    record_id.set(record.id);
                    write_back(&storage, mode, data, record.id, current_date);
                    success.set("存档成功".into());
                }
                Err(msg) => error.set(msg),
            }
        });
    };

    // 更新档案：diff 语义请求（表单未覆盖的字段传 null 保持原值）
    let do_update = move || {
        if submitting.get_untracked() {
            return;
        }
        submitting.set(true);
        let id = record_id.get_untracked();
        let data = data.get_value();
        let token = auth.token().unwrap_or_default();
        let current_date = date.get_untracked();
        spawn_local(async move {
            let res = update_horoscope(id, &update_request(&data, current_date), &token).await;
            submitting.set(false);
            match res {
                Ok(()) => {
                    write_back(&storage, mode, data, id, current_date);
                    success.set("存档成功".into());
                }
                Err(msg) => error.set(msg),
            }
        });
    };

    // 存档按钮：未登录提示；从未存过直接新增，已存过先弹三选
    let on_archive = move |_| {
        // user() 已登录时 token 必然存在；任一缺失按未登录处理（同档案选择器）
        if auth.user().is_none() || auth.token().is_none() {
            notice.set("请先登录后再存档".into());
            return;
        }
        if submitting.get_untracked() {
            return;
        }
        if record_id.get_untracked() != 0 {
            choice.set("您想更新现有记录还是新增记录？".into());
        } else {
            do_add();
        }
    };

    // 三选弹窗按钮（对齐原版 alertController：更新记录 / 新增记录 / 取消）
    let choice_buttons = vec![
        AlertAction {
            text: "更新记录",
            on_click: Callback::new(move |_: ()| {
                choice.set(String::new());
                do_update();
            }),
        },
        AlertAction {
            text: "新增记录",
            on_click: Callback::new(move |_: ()| {
                choice.set(String::new());
                do_add();
            }),
        },
        AlertAction {
            text: "取消",
            on_click: Callback::new(move |_: ()| {
                choice.set(String::new());
            }),
        },
    ];

    view! {
        // 存档入口（对齐原版图标在上、文字在下的按钮）
        <button class=style::archive_btn on:click=on_archive disabled=move || submitting.get()>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="2" y="3" width="20" height="5" rx="1"/>
                <path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8"/>
                <path d="M10 12h4"/>
            </svg>
            "存档"
        </button>

        // 「更新 / 新增 / 取消」三选弹窗
        <AlertDialog header="选择操作" message=choice buttons=choice_buttons/>

        // 成功提示（对齐原版「存档 / 存档成功」）
        <AlertDialog header="存档" message=success/>

        // 未登录提示
        <AlertDialog header="提示" message=notice/>

        // 存档失败（网络 / 后台校验与锁定文案原样透出）
        <AlertDialog header="错误" message=error/>
    }
}
