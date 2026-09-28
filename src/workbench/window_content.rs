//! 窗口内容：以打开时刻的数据快照请求星盘，三分态（加载/错误重试/成功）。
//!
//! 快照随 props 传入且不变；同一窗口不随面板后续编辑刷新，
//! 重试按钮触发重新请求（对应原版 embedded 组件的错误处理增强）。
use leptos::prelude::*;

use wasm_bindgen_futures::spawn_local;

use crate::api::client::post_native;
use crate::api::request::HoroNativeRequest;
use crate::api::response::Horoscope;
use crate::components::ChartWheel;
use crate::models::data::HoroData;
use crate::workbench::window::ChartType;

stylance::import_crate_style!(style, "src/workbench/window_content.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

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

    let result = RwSignal::new(None::<Result<Horoscope, String>>);
    // 重试代数：自增触发 Effect 重新请求
    let reload = RwSignal::new(0u32);

    Effect::new(move |_| {
        reload.track();
        let request = request;
        spawn_local(async move {
            let res = post_native(&request).await;
            result.set(Some(res));
        });
    });

    view! {
        <div class=style::content>
            // 加载中：转圈动画
            <Show when=move || result.get().is_none()>
                <div class=style::loading>
                    <span class=style::spinner></span>
                    "星盘计算中…"
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
