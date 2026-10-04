//! 窗口内容：以打开时刻的数据快照请求星盘，三分态（加载/错误重试/成功）。
//!
//! 快照随 props 传入且不变；同一窗口不随面板后续编辑刷新。
//! 窗口内可编辑时间（对应原版 embedded 模式的时间控件）：改动只写窗口本地
//! 的日期信号，防抖后重新请求并重绘，不回写快照 / 面板 / localStorage；
//! 重试按钮同样以当前日期重新请求（对应原版 embedded 组件的错误处理增强）。
use std::cell::Cell;

use leptos::prelude::*;

use wasm_bindgen_futures::spawn_local;

use crate::api::client::post_native;
use crate::api::request::HoroNativeRequest;
use crate::api::response::Horoscope;
use crate::components::{ChartTimeEditor, ChartWheel};
use crate::models::data::HoroData;
use crate::shared::sleep_ms;
use crate::workbench::window::ChartType;

stylance::import_crate_style!(style, "src/workbench/window_content.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

/// 时间变更 / 重试的防抖间隔（毫秒，对齐原版 applyStepChange 的 debounceTime(300)）。
const DATE_DEBOUNCE_MS: i32 = 300;

#[component]
pub fn WindowContent(chart_type: ChartType, snapshot: HoroData) -> impl IntoView {
    // 本命 / 天象共用同一后台接口，仅快照来源不同（面板按类型取对应表单）。
    // chart_type 保留为 prop 以便后续接入衍生盘 / 推运盘时区分请求与渲染分支。
    let _ = chart_type;
    let request = HoroNativeRequest {
        date: snapshot.date,
        geo: snapshot.geo,
        house: snapshot.house,
    };
    // 窗口本地的日期信号（对应原版 embedded image 组件的 currentHoroData）：
    // 只影响本窗口的请求与显示，不回写快照 / 面板 / localStorage。
    let date = RwSignal::new(snapshot.date);

    let result = RwSignal::new(None::<Result<Horoscope, String>>);
    let loading = RwSignal::new(false);
    // 重试代数：自增触发 Effect 重新请求
    let reload = RwSignal::new(0u32);
    // 防抖代数：日期变更 / 重试共用；休眠结束时代数不符说明已被更新的触发
    // 取代，本次任务直接作废（不请求、不动 loading），由最新的任务负责收尾。
    let generation = RwSignal::new(0u64);

    // 打开窗口立即请求（保持首屏速度）
    spawn_local(async move {
        loading.set(true);
        let res = post_native(&request).await;
        loading.set(false);
        result.set(Some(res));
    });

    // 此后日期变更 / 重试经 300ms 防抖重新请求并重绘，请求参数取当前日期
    let effect_armed = Cell::new(false);
    Effect::new(move |_| {
        let d = date.get();
        reload.track();
        if !effect_armed.get() {
            // Effect 挂载即跑一次：只用于建立追踪，首次请求已由上方发出
            effect_armed.set(true);
            return;
        }
        generation.update(|g| *g += 1);
        let curr_gen = generation.get_untracked();
        let request = HoroNativeRequest { date: d, ..request };
        spawn_local(async move {
            loading.set(true);
            sleep_ms(DATE_DEBOUNCE_MS).await;
            if generation.get_untracked() != curr_gen {
                return;
            }
            let res = post_native(&request).await;
            if generation.get_untracked() != curr_gen {
                return;
            }
            loading.set(false);
            result.set(Some(res));
        });
    });

    view! {
        <div class=style::content>
            <ChartTimeEditor date=date />

            // 加载中：转圈动画
            <Show when=move || result.get().is_none()>
                <div class=style::loading>
                    <span class=style::spinner></span>
                    "星盘计算中…"
                </div>
            </Show>

            // 变更时间后的重算提示：保留旧星盘、不打断浏览
            <Show when=move || loading.get() && result.get().is_some_and(|r| r.is_ok())>
                <div class=style::loading>
                    <span class=style::spinner></span>
                    "时间变更，重新计算中…"
                </div>
            </Show>

            // 错误：文案 + 重试
            {move || result.get().and_then(|r| r.err()).map(|e| {
                view! {
                    <div class=style::error_box>
                        <p class=feedback::error>{e}</p>
                        <button class=style::retry on:click=move |_| reload.update(|n| *n += 1)>
                            "重试"
                        </button>
                    </div>
                }
            })}

            // 成功：星盘轮
            {move || result.get().and_then(|r| r.ok()).map(|h| {
                view! { <ChartWheel horoscope=h/> }
            })}
        </div>
    }
}
