//! 输入页（本命 / 天象双模式）。
use std::cell::Cell;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use reactive_stores::Store;

use crate::components::{
    AlertDialog, ArchiveSelector, DateTimeInput, FormState, FormStateStoreFields, GeoInput,
    HouseSelect,
};
use crate::native::ChartMode;
use crate::models::data::HoroData;
use crate::models::datetime::DateTimeData;
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
    let state = Store::new(FormState::from(restored));
    let err = RwSignal::new(String::new());

    // 夏令时提示：日期字段变化且处于中国夏令时区间时弹框（对应原版
    // onDateChange 的 alertController，不区分是否已勾选 st）。前值记的是
    // 整个日期而非布尔：区间内改日期仍会提示；逐字段读取只追踪日期字段，
    // 姓名、地点等无关字段变化连本 Effect 都不会重跑。
    let dst_alert = RwSignal::new(String::new());
    let prev_date = Cell::new(None::<DateTimeData>);
    Effect::new(move || {
        let d = DateTimeData {
            year: state.year().get(),
            month: state.month().get(),
            day: state.day().get(),
            hour: state.hour().get(),
            minute: state.minute().get(),
            second: state.second().get(),
            tz: state.tz().get(),
            st: state.st().get(),
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

    // Store 没有 With（Get）实现，读整体快照用 read() 拿 guard（零 clone）
    let submit = move |_| {
        let s = state.read();
        if s.year < 1900 {
            err.set("年份需 ≥ 1900".into());
            return;
        }
        if s.long < -180.0 || s.long > 180.0 || s.lat < -90.0 || s.lat > 90.0 {
            err.set("经纬度超出有效范围".into());
            return;
        }
        let horo = HoroData::from(&*s);
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
            // 标题行：右侧放「从档案库选择」（整表级回填入口，两种盘模式均可用）
            <div class=style::header>
                <h2>{if mode == ChartMode::Native { "本命星盘" } else { "天象盘" }}</h2>
                <ArchiveSelector state=state/>
            </div>
            <div class=style::form_grid>
                // 姓名 / 性别：本命盘与天象盘都需要（对齐原版 native.page.html）
                <div class=form::field>
                    <label>"姓名"</label>
                    <div class=form::control>
                        // 字段直接绑定：Subfield 实现了 IntoSplitSignal，无需派生 Signal 的样板
                        <input type="text" placeholder="可选" bind:value=state.name()/>
                    </div>
                </div>
                <div class=form::field>
                    <label>"性别"</label>
                    <div class=form::control>
                        <div class=form::radio_group>
                            // on:click 只在点击本颗时触发（键盘方向键选组同样派发 click），
                            // 并写入固定值：change 会连带到被取消选中的那颗，读事件状态会相互覆盖
                            <label>
                                <input
                                    type="radio"
                                    name="sex"
                                    prop:checked=move || state.sex().get()
                                    on:click=move |_| state.sex().set(true)
                                />
                                <span>"男"</span>
                            </label>
                            <label>
                                <input
                                    type="radio"
                                    name="sex"
                                    prop:checked=move || !state.sex().get()
                                    on:click=move |_| state.sex().set(false)
                                />
                                <span>"女"</span>
                            </label>
                        </div>
                    </div>
                </div>

                <DateTimeInput state=state/>
                <GeoInput state=state/>
                <HouseSelect state=state/>

                <button class=form::btn_primary on:click=submit>
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
