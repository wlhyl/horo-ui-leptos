//! 输入页（本命 / 天象双模式）。
use std::cell::Cell;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::components::{AlertDialog, DateTimeInput, GeoInput, HouseSelect};
use crate::native::ChartMode;
use crate::enums::house::HouseName;
use crate::models::data::HoroData;
use crate::models::datetime::DateTimeData;
use crate::models::geo::GeoPosition;
use crate::routes::AppRoute;
use crate::storage::HoroStorage;

// 作用域样式：src/pages/input.module.css
stylance::import_crate_style!(style, "src/native/input.module.css");
// 显式共享的样式模块（内部并非每个类都被本文件用到，故关闭 dead_code 警告）
stylance::import_crate_style!(card, "src/shared/card.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);

/// 表单状态（保存在信号中）。
#[derive(Clone)]
pub(crate) struct FormState {
    /// 档案 ID（随缓存数据往返，表单不编辑）
    pub id: u32,
    pub name: String,
    pub sex: bool,
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub tz: f64,
    pub st: bool,
    /// 地点名称（随缓存数据往返，由地点选择功能写入）
    pub geo_name: String,
    pub long: f64,
    pub lat: f64,
    pub house: HouseName,
}

impl From<HoroData> for FormState {
    fn from(r: HoroData) -> Self {
        FormState {
            id: r.id,
            name: r.name,
            sex: r.sex,
            year: r.date.year,
            month: r.date.month,
            day: r.date.day,
            hour: r.date.hour,
            minute: r.date.minute,
            second: r.date.second,
            tz: r.date.tz,
            st: r.date.st,
            geo_name: r.geo_name,
            long: r.geo.long,
            lat: r.geo.lat,
            house: r.house,
        }
    }
}

impl FormState {
    /// 汇总日期时间字段（提交与夏令时提示共用）。
    fn date(&self) -> DateTimeData {
        DateTimeData {
            year: self.year,
            month: self.month,
            day: self.day,
            hour: self.hour,
            minute: self.minute,
            second: self.second,
            tz: self.tz,
            st: self.st,
        }
    }
}

#[component]
pub fn Input(mode: ChartMode) -> impl IntoView {
    let storage = use_context::<HoroStorage>().expect("HoroStorage 未初始化");

    // 必须在渲染期（Router 上下文内）取导航器，事件闭包里调用会拿不到上下文
    let nav = use_navigate();

    // 进入页面时从本地缓存恢复上次输入（对应原版 ionViewWillEnter）
    let restored = match mode {
        ChartMode::Event => storage.event_data(),
        ChartMode::Native => storage.horo_data(),
    };
    let state = RwSignal::new(FormState::from(restored));
    let err = RwSignal::new(String::new());

    // 夏令时提示：日期字段变化且处于中国夏令时区间时弹框（对应原版
    // onDateChange 的 alertController，不区分是否已勾选 st）。前值记的是
    // 整个日期而非布尔：姓名等无关字段变化不触发，区间内改日期仍会提示。
    let dst_alert = RwSignal::new(String::new());
    let prev_date = Cell::new(None::<DateTimeData>);
    Effect::new(move || {
        let d = state.get().date();
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

    let submit = move |_| {
        let s = state.get();
        if s.year < 1900 {
            err.set("年份需 ≥ 1900".into());
            return;
        }
        if s.long < -180.0 || s.long > 180.0 || s.lat < -90.0 || s.lat > 90.0 {
            err.set("经纬度超出有效范围".into());
            return;
        }
        let horo = HoroData {
            id: s.id,
            date: s.date(),
            geo_name: s.geo_name.clone(),
            geo: GeoPosition {
                long: s.long,
                lat: s.lat,
            },
            house: s.house,
            name: s.name.clone(),
            sex: s.sex,
        };
        // 写回本地缓存（对应原版 getHoro）；结果页据此取数并请求后台
        match mode {
            ChartMode::Event => storage.set_event_data(horo),
            ChartMode::Native => storage.set_horo_data(horo),
        }
        err.set(String::new());
        // 结果页按类型分路由，组件据此读取对应的缓存数据
        let path = match mode {
            ChartMode::Native => AppRoute::NativeChart.path(),
            ChartMode::Event => AppRoute::EventChart.path(),
        };
        nav(path, NavigateOptions::default());
    };

    view! {
        <div class=card::card>
            <h2>{if mode == ChartMode::Native { "本命星盘" } else { "天象盘" }}</h2>
            <div class=style::form_grid>
                // 姓名 / 性别：本命盘与天象盘都需要（对齐原版 native.page.html）
                <div class=form::field>
                    <label>"姓名"</label>
                    <div class=form::control>
                        <input
                            type="text"
                            placeholder="可选"
                            value=move || state.get().name
                            on:input=move |ev| {
                                let mut s = state.get();
                                s.name = event_target_value(&ev);
                                state.set(s);
                            }
                        />
                    </div>
                </div>
                <div class=form::field>
                    <label>"性别"</label>
                    <div class=form::control>
                        <div class=form::radio_group>
                            <label>
                                <input
                                    type="radio"
                                    name="sex"
                                    checked=move || state.get().sex
                                    on:change=move |_| {
                                        let mut s = state.get();
                                        s.sex = true;
                                        state.set(s);
                                    }
                                />
                                <span>"男"</span>
                            </label>
                            <label>
                                <input
                                    type="radio"
                                    name="sex"
                                    checked=move || !state.get().sex
                                    on:change=move |_| {
                                        let mut s = state.get();
                                        s.sex = false;
                                        state.set(s);
                                    }
                                />
                                <span>"女"</span>
                            </label>
                        </div>
                    </div>
                </div>

                <DateTimeInput state=state/>
                <GeoInput state=state/>
                <HouseSelect state=state/>

                <button class=style::btn_primary on:click=submit>
                    {if mode == ChartMode::Native { "生成本命星盘" } else { "生成天象盘" }}
                </button>
            </div>
        </div>

        // 表单校验错误以对话框呈现（对应原版 alertController 弹窗）
        <AlertDialog header="错误" message=err/>
        // 夏令时提示弹框（对应原版 onDateChange 的 alertController）
        <AlertDialog header="夏令时提示" message=dst_alert/>
    }
}
