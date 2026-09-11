//! 结果页：读取 Horoscope，星盘 / 相位 / 详情 标签页切换。
use leptos::control_flow::Show;
use leptos::prelude::*;

use wasm_bindgen_futures::spawn_local;

use crate::api::client::post_native;
use crate::api::request::HoroNativeRequest;
use crate::api::response::Horoscope;
use crate::components::{AlertDialog, AspectGrid, ChartWheel, Detail};
use crate::native::ChartMode;
use crate::storage::HoroStorage;

// 作用域样式：src/pages/chart.module.css
stylance::import_crate_style!(style, "src/native/chart.module.css");
stylance::import_crate_style!(card, "src/shared/card.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

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

    // 输入页已把数据写入存储，这里按本页类型取回并请求后台
    let data = match mode {
        ChartMode::Native => storage.horo_data(),
        ChartMode::Event => storage.event_data(),
    };
    let request = HoroNativeRequest {
        date: data.date,
        geo: data.geo,
        house: data.house,
    };

    let (result, set_result) = signal(None::<Result<Horoscope, String>>);
    // 请求失败以对话框提示（对应原版 alertController 弹窗）
    let alert_msg = RwSignal::new(String::new());
    spawn_local(async move {
        let res = post_native(&request).await;
        if let Err(e) = &res {
            alert_msg.set(e.clone());
        }
        set_result.set(Some(res));
    });
    let summary = format!(
        "{}年{}月{}日 {}:{}:{} · 时区 {}",
        request.date.year,
        request.date.month,
        request.date.day,
        request.date.hour,
        request.date.minute,
        request.date.second,
        request.date.tz
    );

    view! {
        <div class=card::card>
            <div class=style::summary>
                <span>
                    <b>{if mode == ChartMode::Native { "本命星盘" } else { "天象盘" }}</b>
                </span>
                <span>{summary}</span>
            </div>

            <Show when=move || result.get().is_none() fallback=|| ()>
                {view! { <div class=feedback::loading>"星盘计算中…"</div> }}
            </Show>

            {move || result.get().and_then(Result::ok).map(|h| {
                view! {
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
                    <div>
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
