//! 返照盘结果视图（对应原版 return.component）。
//!
//! 三种模式共用：日返（SolarReturn）/ 月返（LunarReturn）/ 每日回归（DailyReturn）。
//! 视图持出生与推运参数快照，全部编辑只影响本视图的请求，不写回 HoroStorage
//! （对齐原版组件状态）。独立页（ReturnPage）从 HoroStorage 取快照；工作台窗口
//! 直接传开窗快照。
use std::cell::Cell;

use leptos::control_flow::Show;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;

use crate::api::client::{post_daily_return, post_lunar_return, post_solar_return};
use crate::api::request::ReturnRequest;
use crate::api::response::{Horoscope, ReturnHoroscope};
use crate::components::{AlertDialog, AspectGrid, ChartTimeEditor, ChartWheel, Detail};
use crate::direction::utils::{format_date, process_title};
use crate::enums::house::HouseName;
use crate::enums::process_name::ProcessName;
use crate::models::data::{HoroData, ProcessData};
use crate::models::datetime::DateTimeData;
use crate::models::geo::GeoPosition;
use crate::routes::AppRoute;
use crate::shared::sleep_ms;
use crate::storage::HoroStorage;

stylance::import_crate_style!(style, "src/return_chart/view.module.css");
stylance::import_crate_style!(card, "src/shared/card.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

/// 推运时间变更后的防抖间隔（毫秒，对齐原版 changeStepSubject 的防抖重取，
/// 与本项目其他结果页统一为 300ms）。
const DATE_DEBOUNCE_MS: i32 = 300;

/// 按模式分流返照请求（对应原版 getSolarReturnData / getLunarReturnData /
/// getDailyReturnData）。开启「日返月亮 / 日返月返」时按 日返→月返→每日回归
/// 逐层链式取返照时刻：每步以上一步的返照时刻为下一步的本命时间、st 固定 false。
/// 经纬度一律用推运数据的居住地、宫位用出生数据的宫位制（对齐原版 getReturnData）。
/// 返照比较盘（compare 模块）复用同一链路取返照时刻。
pub(crate) async fn fetch_return(
    mode: ProcessName,
    native_date: DateTimeData,
    process_date: DateTimeData,
    geo: GeoPosition,
    house: HouseName,
    is_solar_return: bool,
) -> Result<ReturnHoroscope, String> {
    let req = |native_date: DateTimeData| ReturnRequest {
        native_date,
        process_date,
        geo,
        house,
    };
    match mode {
        ProcessName::SolarReturn => post_solar_return(&req(native_date)).await,
        ProcessName::LunarReturn => {
            if is_solar_return {
                let solar = post_solar_return(&req(native_date)).await?;
                let mut d = solar.return_date;
                d.st = false;
                post_lunar_return(&req(d)).await
            } else {
                post_lunar_return(&req(native_date)).await
            }
        }
        // 每日回归（默认分支）
        _ => {
            if is_solar_return {
                let solar = post_solar_return(&req(native_date)).await?;
                let mut d = solar.return_date;
                d.st = false;
                let lunar = post_lunar_return(&req(d)).await?;
                let mut d = lunar.return_date;
                d.st = false;
                post_daily_return(&req(d)).await
            } else {
                post_daily_return(&req(native_date)).await
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Wheel,
    Aspect,
    Detail,
}

/// 返照盘独立页（/return/solar /return/lunar /return/daily）：页头 + 从缓存取快照。
#[component]
pub(crate) fn ReturnPage(mode: ProcessName) -> impl IntoView {
    let storage = use_context::<HoroStorage>().expect("HoroStorage 未初始化");
    // 必须在渲染期（Router 上下文内）取导航器，事件闭包里调用会拿不到上下文
    let nav = use_navigate();
    let go_back = move |_| {
        nav(AppRoute::Home.path(), NavigateOptions::default());
    };
    // 进入页面时从缓存取出生与推运参数快照（对应原版组件读 storage.horoData /
    // storage.processData）；此后页面编辑不回写。
    let horo = storage.horo_data();
    let process = storage.process_data();

    view! {
        <div class=style::page>
            <header class=style::header>
                <button class=style::back_btn on:click=go_back aria-label="返回">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M15 18l-6-6 6-6"/>
                    </svg>
                </button>
                <h1 class=style::title>{process_title(mode)}</h1>
            </header>
            <ReturnView mode horo process/>
        </div>
    }
}

/// 返照盘结果视图：摘要（推运时间 / 返照时间）+ 时间编辑 + 星盘 / 相位 / 详情标签页。
#[component]
pub(crate) fn ReturnView(mode: ProcessName, horo: HoroData, process: ProcessData) -> impl IntoView {
    let (tab, set_tab) = signal(Tab::Wheel);

    // —— 页面本地状态（不写回 HoroStorage，对齐原版组件字段）——
    // 推运时间（对应原版 currentProcessData.date：原版经 footer 步进器修改，
    // 此处用星盘页同款时间编辑条）；出生时间与宫位不可编辑，取快照。
    let process_date = RwSignal::new(process.date);
    // 「日返月亮 / 日返月返」在推运输入页设置，本视图只读快照（对应原版 isSolarReturn）；
    // 居住地一律取推运数据（对应原版 geo 一律用 processData.geo）
    let is_solar_return = process.is_solar_return;
    let (native_date, house, geo) = (horo.date, horo.house, process.geo);

    let (result, set_result) = signal(None::<Result<ReturnHoroscope, String>>);
    let loading = RwSignal::new(false);
    // 防抖代数：推运时间每变一次自增；休眠结束时代数不符说明已被更新的变更取代，
    // 本次任务直接作废（不请求、不动 loading），由最新的任务负责收尾。
    let generation = RwSignal::new(0u64);
    // 请求失败以对话框提示（对应原版 alertController 弹窗）
    let alert_msg = RwSignal::new(String::new());

    // 请求（对应原版 drawHoroscope → getReturnData）
    let request = move |debounce_ms: i32, process_date: DateTimeData| {
        generation.update(|g| *g += 1);
        let curr_gen = generation.get_untracked();
        loading.set(true);
        spawn_local(async move {
            if debounce_ms > 0 {
                sleep_ms(debounce_ms).await;
                if generation.get_untracked() != curr_gen {
                    return;
                }
            }
            let res =
                fetch_return(mode, native_date, process_date, geo, house, is_solar_return).await;
            if generation.get_untracked() != curr_gen {
                return;
            }
            loading.set(false);
            match res {
                Ok(r) => set_result.set(Some(Ok(r))),
                Err(e) => alert_msg.set(e),
            }
        });
    };

    // 首次立即请求（对齐原版 ngOnInit 的 drawHoroscope）
    request(0, process.date);

    // 此后推运时间每次变更经 300ms 防抖重新请求；重算期间保留旧星盘
    // （对齐原版 loading 保留旧 canvas），失败也保留旧图、仅弹错误对话框。
    let effect_armed = Cell::new(false);
    Effect::new(move |_| {
        let d = process_date.get();
        if !effect_armed.get() {
            // Effect 挂载即跑一次：只用于建立追踪，首次请求已由上方发出
            effect_armed.set(true);
            return;
        }
        request(DATE_DEBOUNCE_MS, d);
    });

    view! {
        <div class=card::card>
            <div class=style::summary>
                <span>
                    <b>{process_title(mode)}</b>
                </span>
                <span>{move || format!("推运时间：{}", format_date(&process_date.get()))}</span>
                {move || {
                    result
                        .get()
                        .and_then(Result::ok)
                        .map(|r| view! { <span>{format!("返照时间：{}", format_date(&r.return_date))}</span> })
                }}
            </div>

            <ChartTimeEditor date=process_date/>

            <Show when=move || result.get().is_none() fallback=|| ()>
                {view! { <div class=feedback::loading>"返照盘计算中…"</div> }}
            </Show>

            // 变更推运时间后的重算提示：保留旧星盘、不打断浏览
            <Show when=move || loading.get() && result.get().is_some() fallback=|| ()>
                {view! { <div class=feedback::loading>"推运时间变更，重新计算中…"</div> }}
            </Show>

            {move || result.get().and_then(Result::ok).map(|r| {
                let h = Horoscope::from(r);
                let hw = h.clone();
                let ha = h.clone();
                let hd = h.clone();
                view! {
                    // 标签行：星盘 / 相位 / 详情 分段控件居左（返照盘无存档入口）
                    <div class=style::tabs_row>
                        <div class=style::tabs>
                            <button
                                class=move || if tab.get() == Tab::Wheel { style::active } else { "" }
                                on:click=move |_| set_tab.set(Tab::Wheel)
                            >
                                "星盘"
                            </button>
                            <button
                                class=move || if tab.get() == Tab::Aspect { style::active } else { "" }
                                on:click=move |_| set_tab.set(Tab::Aspect)
                            >
                                "相位"
                            </button>
                            <button
                                class=move || if tab.get() == Tab::Detail { style::active } else { "" }
                                on:click=move |_| set_tab.set(Tab::Detail)
                            >
                                "详情"
                            </button>
                        </div>
                    </div>
                    <div class=style::tab_content>
                        <Show when=move || tab.get() == Tab::Wheel fallback=|| ()>
                            {view! { <ChartWheel horoscope=hw.clone()/> }}
                        </Show>
                        <Show when=move || tab.get() == Tab::Aspect fallback=|| ()>
                            {view! { <AspectGrid horoscope=ha.clone()/> }}
                        </Show>
                        <Show when=move || tab.get() == Tab::Detail fallback=|| ()>
                            {view! { <Detail horoscope=hd.clone()/> }}
                        </Show>
                    </div>
                }
            })}
        </div>

        // 请求失败以对话框呈现（对应原版 alertController 弹窗）
        <AlertDialog header="错误" message=alert_msg/>
    }
}
