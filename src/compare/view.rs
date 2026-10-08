//! 比较盘结果视图（移植自 horo-ui 的 compare.component）。
//!
//! 八种模式共用一套视图：行运（Transit）/ 日返比本命（SolarcomparNative）/
//! 本命比日返（NativecomparSolar）/ 月返比本命 / 本命比月返 /
//! 每日回归比本命 / 本命比每日回归 / 次限比本命（SecondaryProgressionComparNative）。
//! 视图持出生与推运参数快照，推运时间编辑只影响本视图的请求，不写回
//! HoroStorage（对齐原版组件状态）。独立页（ComparePage）从 HoroStorage 取
//! 快照；工作台窗口直接传开窗快照。
use leptos::control_flow::Show;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;

use crate::api::client::{post_compare, post_secondary_progression};
use crate::api::request::{HoroscopeComparisonRequest, SecondaryProgressionRequest};
use crate::api::response::HoroscopeComparison;
use crate::astro::horo_math::degree_to_dms;
use crate::components::{AlertDialog, ChartTimeEditor, CompareAspectGrid, CompareWheel};
use crate::direction::utils::{format_date, process_title};
use crate::enums::house::HouseName;
use crate::enums::process_name::ProcessName;
use crate::enums::secondary_progression_method::SecondaryProgressionMethod;
use crate::models::data::{HoroData, ProcessData};
use crate::models::datetime::DateTimeData;
use crate::models::geo::GeoPosition;
use crate::render::glyphs::{planet_color, planet_glyph};
use crate::return_chart::fetch_return;
use crate::routes::AppRoute;
use crate::shared::sleep_ms;
use crate::storage::HoroStorage;

stylance::import_crate_style!(style, "src/compare/view.module.css");
stylance::import_crate_style!(card, "src/shared/card.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

/// 推运时间变更后的防抖间隔（毫秒，对齐原版 changeStepSubject 的防抖重取，
/// 与本项目其他结果页统一为 300ms）。
const DATE_DEBOUNCE_MS: i32 = 300;

/// 度数 → "D°MM′SS″"（详情列表用）。
fn dms_text(d: f64) -> String {
    let (dd, mm, ss) = degree_to_dms(d);
    format!("{dd}°{mm:02}′{ss:02}″")
}

/// 组装比较盘请求：本命侧与返照 / 推运侧各带自己的时间与经纬度。
async fn compare(
    original_date: DateTimeData,
    original_geo: GeoPosition,
    comparison_date: DateTimeData,
    comparison_geo: GeoPosition,
    house: HouseName,
) -> Result<HoroscopeComparison, String> {
    post_compare(&HoroscopeComparisonRequest {
        original_date,
        comparison_date,
        original_geo,
        comparison_geo,
        house,
    })
    .await
}

/// 按模式分流比较盘请求（对应原版 getHoroscopeComparisonData 分支链）。
///
/// - 行运：本命为原盘，比较盘时间为推运时间；比较盘经纬度用本命 geo
///   （对齐原版 getTransitData 注释）。
/// - 返照比较六种：先经返照链取返照时刻（st 置 false），XxxComparNative 组
///   本命为原盘、返照为比较盘，NativeComparXxx 组相反；geo 跟随各自的盘
///   （对齐原版 getReturnComparData）。
/// - 次限比本命：先取次限推运时刻，本命为原盘、次限为比较盘，比较盘 geo 用
///   推运居住地（对齐原版 getSecondaryProgressionComparData）。
async fn fetch_compare(
    mode: ProcessName,
    native_date: DateTimeData,
    native_geo: GeoPosition,
    process_date: DateTimeData,
    process_geo: GeoPosition,
    house: HouseName,
    is_solar_return: bool,
    sp_method: SecondaryProgressionMethod,
) -> Result<HoroscopeComparison, String> {
    match mode {
        ProcessName::Transit => {
            compare(native_date, native_geo, process_date, native_geo, house).await
        }
        ProcessName::SecondaryProgressionComparNative => {
            let sp = post_secondary_progression(&SecondaryProgressionRequest {
                native_date,
                process_date,
                geo: process_geo,
                method: sp_method,
                house,
            })
            .await?;
            let mut d = sp.progression_date;
            d.st = false;
            compare(native_date, native_geo, d, process_geo, house).await
        }
        // 返照比较：先取返照时刻，再按方向组对比
        ProcessName::SolarcomparNative
        | ProcessName::NativecomparSolar
        | ProcessName::LunarcomparNative
        | ProcessName::NativecomparLunar
        | ProcessName::DailycomparNative
        | ProcessName::NativecomparDaily => {
            let return_mode = match mode {
                ProcessName::SolarcomparNative | ProcessName::NativecomparSolar => {
                    ProcessName::SolarReturn
                }
                ProcessName::LunarcomparNative | ProcessName::NativecomparLunar => {
                    ProcessName::LunarReturn
                }
                _ => ProcessName::DailyReturn,
            };
            let return_horo = fetch_return(
                return_mode,
                native_date,
                process_date,
                process_geo,
                house,
                is_solar_return,
            )
            .await?;
            let mut return_date = return_horo.return_date;
            return_date.st = false;
            // XxxComparNative：本命为原盘；NativeComparXxx：返照为原盘
            let native_is_original = matches!(
                mode,
                ProcessName::SolarcomparNative
                    | ProcessName::LunarcomparNative
                    | ProcessName::DailycomparNative
            );
            if native_is_original {
                compare(native_date, native_geo, return_date, process_geo, house).await
            } else {
                compare(return_date, process_geo, native_date, native_geo, house).await
            }
        }
        _ => Err(format!("未知的比较盘类型：{mode:?}")),
    }
}

/// 该模式是否先取返照 / 推进时刻（决定摘要行文案）。
fn comparison_date_label(mode: ProcessName) -> Option<&'static str> {
    match mode {
        ProcessName::SolarcomparNative
        | ProcessName::NativecomparSolar
        | ProcessName::LunarcomparNative
        | ProcessName::NativecomparLunar
        | ProcessName::DailycomparNative
        | ProcessName::NativecomparDaily => Some("返照时间"),
        ProcessName::SecondaryProgressionComparNative => Some("次限时间"),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Wheel,
    Aspect,
    Detail,
}

/// 比较盘独立页（/compare/*）：页头 + 从缓存取快照。
#[component]
pub(crate) fn ComparePage(mode: ProcessName) -> impl IntoView {
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
            <CompareView mode horo process/>
        </div>
    }
}

/// 比较盘结果视图：摘要（推运时间 / 返照或次限时间）+ 时间编辑 + 星盘 / 相位 / 详情标签页。
#[component]
pub(crate) fn CompareView(mode: ProcessName, horo: HoroData, process: ProcessData) -> impl IntoView {
    let (tab, set_tab) = signal(Tab::Wheel);

    // —— 页面本地状态（不写回 HoroStorage，对齐原版组件字段）——
    // 推运时间（对应原版 currentProcessData.date）；出生时间 / 宫位不可编辑，
    // 取快照；居住地一律取推运数据（对应原版 processData.geo）
    let process_date = RwSignal::new(process.date);
    // 「日返月亮 / 日返月返」在推运输入页设置，本视图只读快照（对应原版 isSolarReturn）
    let is_solar_return = process.is_solar_return;
    let sp_method = process.secondary_progression_method;
    let (native_date, native_geo, house, process_geo) =
        (horo.date, horo.geo, horo.house, process.geo);

    let (result, set_result) = signal(None::<Result<HoroscopeComparison, String>>);
    let loading = RwSignal::new(false);
    // 防抖代数：推运时间每变一次自增；休眠结束时代数不符说明已被更新的变更取代，
    // 本次任务直接作废（不请求、不动 loading），由最新的任务负责收尾。
    let generation = RwSignal::new(0u64);
    // 请求失败以对话框提示（对应原版 alertController 弹窗）
    let alert_msg = RwSignal::new(String::new());

    // 请求（对应原版 drawHoroscope → getHoroscopeComparisonData）
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
            let res = fetch_compare(
                mode,
                native_date,
                native_geo,
                process_date,
                process_geo,
                house,
                is_solar_return,
                sp_method,
            )
            .await;
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
    let effect_armed = std::cell::Cell::new(false);
    Effect::new(move |_| {
        let d = process_date.get();
        if !effect_armed.get() {
            // Effect 挂载即跑一次：只用于建立追踪，首次请求已由上方发出
            effect_armed.set(true);
            return;
        }
        request(DATE_DEBOUNCE_MS, d);
    });

    // 摘要第二行：返照 / 次限模式显示比较盘时刻（对齐原版 returnData 的返照时间行）
    let date_label = comparison_date_label(mode);

    view! {
        <div class=card::card>
            <div class=style::summary>
                <span>
                    <b>{process_title(mode)}</b>
                </span>
                <span>{move || format!("推运时间：{}", format_date(&process_date.get()))}</span>
                {move || {
                    date_label
                        .map(|label| {
                            result
                                .get()
                                .and_then(Result::ok)
                                .map(|r| view! { <span>{format!("{}：{}", label, format_date(&r.comparison_date))}</span> })
                        })
                }}
            </div>

            <ChartTimeEditor date=process_date/>

            <Show when=move || result.get().is_none() fallback=|| ()>
                {view! { <div class=feedback::loading>"比较盘计算中…"</div> }}
            </Show>

            // 变更推运时间后的重算提示：保留旧星盘、不打断浏览
            <Show when=move || loading.get() && result.get().is_some() fallback=|| ()>
                {view! { <div class=feedback::loading>"推运时间变更，重新计算中…"</div> }}
            </Show>

            {move || result.get().and_then(Result::ok).map(|r| {
                let hw = r.clone();
                let ha = r.clone();
                let hd = r.clone();
                view! {
                    // 标签行：星盘 / 相位 / 详情 分段控件居左（比较盘无存档入口）
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
                            {view! { <CompareWheel comparison=hw.clone()/> }}
                        </Show>
                        <Show when=move || tab.get() == Tab::Aspect fallback=|| ()>
                            {view! {
                                <CompareAspectGrid comparison=ha.clone()/>
                                <p class=style::aspect_hint>"说明：比较盘行星横看，原盘行星竖看"</p>
                            }}
                        </Show>
                        <Show when=move || tab.get() == Tab::Detail fallback=|| ()>
                            {view! { <CompareDetail comparison=hd.clone()/> }}
                        </Show>
                    </div>
                }
            })}
        </div>

        // 请求失败以对话框呈现（对应原版 alertController 弹窗）
        <AlertDialog header="错误" message=alert_msg/>
    }
}

/// 详情：映点 / 反映点两列卡片（对应原版 compare/detail 组件）。
/// p0 为比较盘星体、p1 为原盘星体（与相位矩阵的读图方向一致）。
#[component]
fn CompareDetail(comparison: HoroscopeComparison) -> impl IntoView {
    // props 非 Copy 且 Show 的 when 闭包要求 'static，把两份列表存入
    // StoredValue，按值取用（空列表就地渲染占位文案）
    let antiscoins = StoredValue::new(comparison.antiscoins.clone());
    let contraantiscias = StoredValue::new(comparison.contraantiscias.clone());

    fn list(items: Vec<crate::api::response::Aspect>) -> impl IntoView {
        items
            .iter()
            .map(|a| {
                let g0 = planet_glyph(a.p0);
                let c0 = planet_color(a.p0);
                let g1 = planet_glyph(a.p1);
                let c1 = planet_color(a.p1);
                let dms = dms_text(a.d);
                view! {
                    <li>
                        <span class=style::glyph style=format!("color:{c0}")>{g0}</span>
                        " 与 "
                        <span class=style::glyph style=format!("color:{c1}")>{g1}</span>
                        "，" {dms}
                    </li>
                }
            })
            .collect_view()
    }

    view! {
        <div class=style::detail_cols>
            <div class=style::detail_card>
                <h4 class=style::detail_title>"映点"</h4>
                {move || {
                    let items = antiscoins.get_value();
                    if items.is_empty() {
                        view! { <p class=style::detail_empty>"没有映点数据。"</p> }.into_any()
                    } else {
                        view! { <ul class=style::detail_list>{list(items)}</ul> }.into_any()
                    }
                }}
            </div>
            <div class=style::detail_card>
                <h4 class=style::detail_title>"反映点"</h4>
                {move || {
                    let items = contraantiscias.get_value();
                    if items.is_empty() {
                        view! { <p class=style::detail_empty>"没有反映点数据。"</p> }.into_any()
                    } else {
                        view! { <ul class=style::detail_list>{list(items)}</ul> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
