//! 地名搜索与经纬度输入（绑定 FormState 信号）。
//!
//! 地名可直接输入并随表单提交缓存；点击「搜索」或回车调用后台
//! location_search 查询，从结果列表点选后回填地名与经纬度。
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::client::get_location_search;
use crate::api::response::LocationResponse;
use crate::auth::AuthService;
use crate::native::input::FormState;

// 作用域样式：src/components/geo_input/geo_input.module.css
stylance::import_crate_style!(style, "src/components/geo_input/geo_input.module.css");
stylance::import_crate_style!(#[allow(dead_code)] form, "src/shared/form.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

#[component]
pub fn GeoInput(state: RwSignal<FormState>) -> impl IntoView {
    let auth = use_context::<AuthService>().expect("AuthService 未初始化");

    let querying = RwSignal::new(false);
    let locations = RwSignal::new(Vec::<LocationResponse>::new());
    let search_err = RwSignal::new(String::new());

    // 搜索地名：请求中禁用重复提交，结果与错误互斥展示
    let search = move || {
        let name = state.get().geo_name.trim().to_owned();
        if name.is_empty() || querying.get() {
            return;
        }
        let Some(token) = auth.token() else {
            search_err.set("请先登录后再搜索地点".into());
            return;
        };
        querying.set(true);
        locations.set(Vec::new());
        search_err.set(String::new());
        spawn_local(async move {
            match get_location_search(&name, &token).await {
                Ok(res) => locations.set(res),
                Err(msg) => search_err.set(msg),
            }
            querying.set(false);
        });
    };

    // 点选结果：回填地名与经纬度（后台返回字符串，此处转数值）
    let select = move |i: usize| {
        if let Some(loc) = locations.get().into_iter().nth(i) {
            match (loc.longitude.parse::<f64>(), loc.latitude.parse::<f64>()) {
                (Ok(long), Ok(lat)) => {
                    let mut s = state.get();
                    s.geo_name = loc.name.clone();
                    s.long = long;
                    s.lat = lat;
                    state.set(s);
                    locations.set(Vec::new());
                }
                _ => leptos::logging::warn!("经纬度解析失败，跳过：{}", loc.name),
            }
        }
    };

    view! {
        <div class=form::field>
            <label>"地名"</label>
            <div class=form::control>
                <input
                    type="text"
                    placeholder="输入地名搜索经纬度"
                    class=style::name_input
                    prop:value=move || state.get().geo_name
                    on:input=move |ev| {
                        let mut s = state.get();
                        s.geo_name = event_target_value(&ev);
                        state.set(s);
                    }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            ev.prevent_default();
                            search();
                        }
                    }
                />
                <button
                    class=style::search_btn
                    disabled=move || querying.get()
                    on:click=move |_| search()
                >
                    {move || if querying.get() { "搜索中…" } else { "搜索" }}
                </button>
            </div>

            // 搜索结果列表：占字段第 2 列，与输入框左对齐；点选后回填并收起
            {move || {
                let list = locations.get();
                (!list.is_empty()).then(move || {
                    view! {
                        <div class=style::results>
                            {list
                                .into_iter()
                                .enumerate()
                                .map(|(i, loc)| {
                                    view! {
                                        <div class=style::result on:click=move |_| select(i)>
                                            {loc.name}
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }
                })
            }}
        </div>

        // 搜索失败 / 未登录反馈
        {move || {
            (!search_err.get().is_empty())
                .then(|| view! { <div class=feedback::error>{move || search_err.get()}</div> })
        }}

        <div class=form::field>
            <label>"经度"</label>
            <div class=form::control>
                <input
                    type="number"
                    step="0.0001"
                    placeholder="东经为正"
                    prop:value=move || state.get().long.to_string()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse() {
                            let mut s = state.get();
                            s.long = v;
                            state.set(s);
                        }
                    }
                />
            </div>
        </div>

        <div class=form::field>
            <label>"纬度"</label>
            <div class=form::control>
                <input
                    type="number"
                    step="0.0001"
                    placeholder="北纬为正"
                    prop:value=move || state.get().lat.to_string()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse() {
                            let mut s = state.get();
                            s.lat = v;
                            state.set(s);
                        }
                    }
                />
            </div>
        </div>
    }
}
