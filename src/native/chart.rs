//! 结果页：读取 Horoscope，星盘 / 相位 / 详情 标签页切换。
//!
//! 时间编辑只改页面本地的日期信号（对应原版 image 页只改内存副本
//! currentHoroData）：首次立即请求，之后变更经 300ms 防抖重新请求并重绘，
//! 全程不写 HoroStorage（不落盘）。
use std::cell::Cell;

use leptos::control_flow::Show;
use leptos::prelude::*;

use wasm_bindgen_futures::spawn_local;

use crate::api::client::{post_derived, post_native};
use crate::api::request::{DerivedHoroRequest, HoroNativeRequest};
use crate::api::response::Horoscope;
use crate::components::{AlertDialog, AspectGrid, ChartArchive, ChartTimeEditor, ChartWheel, Detail};
use crate::enums::house::HouseName;
use crate::enums::planet::PlanetName;
use crate::models::datetime::DateTimeData;
use crate::models::geo::GeoPosition;
use crate::native::ChartMode;
use crate::render::glyphs::{planet_color, planet_glyph};
use crate::shared::sleep_ms;
use crate::storage::HoroStorage;

// 作用域样式：src/pages/chart.module.css
stylance::import_crate_style!(style, "src/native/chart.module.css");
stylance::import_crate_style!(card, "src/shared/card.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

/// 时间变更后的防抖间隔（毫秒，对齐原版 applyStepChange 的 debounceTime(300)）。
const DATE_DEBOUNCE_MS: i32 = 300;

/// 按盘类型分流后台请求：本命 / 天象走 native 接口，衍生盘走 derived 接口。
/// 衍生盘的基准行星取页面进入时的快照，页面内不切换（对齐原版 image 组件）。
async fn fetch(
    mode: ChartMode,
    date: DateTimeData,
    geo: GeoPosition,
    house: HouseName,
    derived_planet: PlanetName,
) -> Result<Horoscope, String> {
    match mode {
        ChartMode::Native | ChartMode::Event => {
            post_native(&HoroNativeRequest { date, geo, house }).await
        }
        ChartMode::Derived => {
            post_derived(&DerivedHoroRequest {
                date,
                geo,
                house,
                planet_name: derived_planet,
            })
            .await
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Wheel,
    Aspect,
    Detail,
}

#[component]
pub fn Chart(mode: ChartMode) -> impl IntoView {
    let storage = use_context::<HoroStorage>().expect("HoroStorage 未初始化");
    let (tab, set_tab) = signal(Tab::Wheel);

    // 输入页已把数据写入存储，这里按本页类型取回并请求后台。
    // date 单独放进页面本地信号供时间编辑条修改（对应原版 image 页的
    // currentHoroData 内存副本，改完不写回 storage）；geo / house 不可编辑，取快照。
    // 衍生盘以出生数据为基准（与本命盘共用 horo_data），基准行星一并取快照。
    let data = match mode {
        ChartMode::Event => storage.event_data(),
        ChartMode::Native | ChartMode::Derived => storage.horo_data(),
    };
    let (geo, house) = (data.geo, data.house);
    let derived_planet = storage.derived_planet_name();
    let date = RwSignal::new(data.date);
    // 快照入 StoredValue：存档入口在动态视图内按需重建，闭包只捕获 Copy 句柄
    let data = StoredValue::new(data);

    let (result, set_result) = signal(None::<Result<Horoscope, String>>);
    let loading = RwSignal::new(false);
    // 防抖代数：日期每变一次自增；休眠结束时代数不符说明已被更新的变更取代，
    // 本次任务直接作废（不请求、不动 loading），由最新的任务负责收尾。
    let generation = RwSignal::new(0u64);
    // 请求失败以对话框提示（对应原版 alertController 弹窗）
    let alert_msg = RwSignal::new(String::new());

    // 首次立即请求（保持首屏速度）
    spawn_local(async move {
        loading.set(true);
        let res = fetch(mode, data.get_value().date, geo, house, derived_planet).await;
        loading.set(false);
        if let Err(e) = &res {
            alert_msg.set(e.clone());
        }
        set_result.set(Some(res));
    });

    // 此后日期每次变更经 300ms 防抖重新请求并重绘；重算期间保留旧星盘
    // （对齐原版 loading 保留旧 canvas），失败也保留旧图、仅弹错误对话框。
    let effect_armed = Cell::new(false);
    Effect::new(move |_| {
        let d = date.get();
        if !effect_armed.get() {
            // Effect 挂载即跑一次：只用于建立追踪，首次请求已由上方发出
            effect_armed.set(true);
            return;
        }
        generation.update(|g| *g += 1);
        let curr_gen = generation.get_untracked();
        spawn_local(async move {
            loading.set(true);
            sleep_ms(DATE_DEBOUNCE_MS).await;
            if generation.get_untracked() != curr_gen {
                return;
            }
            let res = fetch(mode, d, geo, house, derived_planet).await;
            if generation.get_untracked() != curr_gen {
                return;
            }
            loading.set(false);
            match res {
                Ok(h) => set_result.set(Some(Ok(h))),
                Err(e) => alert_msg.set(e),
            }
        });
    });

    // 摘要日期随编辑实时更新
    let summary = move || {
        let d = date.get();
        format!(
            "{}年{}月{}日 {}:{}:{} · 时区 {}",
            d.year, d.month, d.day, d.hour, d.minute, d.second, d.tz
        )
    };

    view! {
        <div class=card::card>
            <div class=style::summary>
                <span>
                    <b>{match mode {
                        ChartMode::Native => "本命星盘",
                        ChartMode::Event => "天象盘",
                        ChartMode::Derived => "衍生盘",
                    }}</b>
                </span>
                <span>{summary}</span>
                // 基准行星：仅衍生盘显示（对应原版 image.component.html 32-37 行）
                <Show when=move || mode == ChartMode::Derived fallback=|| ()>
                    <span class=style::derived_planet>
                        "基准行星："
                        <span style=format!("color:{}", planet_color(derived_planet))>
                            {planet_glyph(derived_planet)}
                        </span>
                    </span>
                </Show>
            </div>

            <ChartTimeEditor date=date />

            <Show when=move || result.get().is_none() fallback=|| ()>
                {view! { <div class=feedback::loading>"星盘计算中…"</div> }}
            </Show>

            // 变更时间后的重算提示：保留旧星盘、不打断浏览
            <Show when=move || loading.get() && result.get().is_some() fallback=|| ()>
                {view! { <div class=feedback::loading>"时间变更，重新计算中…"</div> }}
            </Show>

            {move || result.get().and_then(Result::ok).map(|h| {
                view! {
                    // 标签行：星盘 / 相位 / 详情 分段控件居左，存档入口居右（衍生盘无存档）
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
                        // 存档入口：本命 / 天象可用，衍生盘不提供（对齐原版 mode !== Derived）
                        <Show when=move || mode != ChartMode::Derived fallback=|| ()>
                            <ChartArchive mode data=data.get_value() date=date/>
                        </Show>
                    </div>
                    <div class=style::tab_content>
                        {{
                            let hw = h.clone();
                            let ha = h.clone();
                            let hd = h.clone();
                            view! {
                                <Show when=move || tab.get() == Tab::Wheel fallback=|| ()>
                                    {view! { <ChartWheel horoscope=hw.clone()/> }}
                                </Show>
                                <Show when=move || tab.get() == Tab::Aspect fallback=|| ()>
                                    {view! { <AspectGrid horoscope=ha.clone()/> }}
                                </Show>
                                <Show when=move || tab.get() == Tab::Detail fallback=|| ()>
                                    {view! { <Detail horoscope=hd.clone()/> }}
                                </Show>
                            }
                        }}
                    </div>
                }
            })}
        </div>

        // 后台请求失败以对话框呈现（对应原版 alertController 弹窗）
        <AlertDialog header="错误" message=alert_msg/>
    }
}
