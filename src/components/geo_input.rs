//! 经纬度输入（绑定 FormState 信号）。
use leptos::prelude::*;

use crate::native::input::FormState;

stylance::import_crate_style!(#[allow(dead_code)] form, "src/shared/form.module.css");

#[component]
pub fn GeoInput(state: RwSignal<FormState>) -> impl IntoView {
    view! {
        <div class=form::field>
            <label>"经度"</label>
            <div class=form::control>
                <input
                    type="number"
                    step="0.0001"
                    placeholder="东经为正"
                    value=move || state.get().long.to_string()
                    on:input=move |ev| {
                        let mut s = state.get();
                        s.long = event_target_value(&ev).parse().unwrap_or(s.long);
                        state.set(s);
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
                    value=move || state.get().lat.to_string()
                    on:input=move |ev| {
                        let mut s = state.get();
                        s.lat = event_target_value(&ev).parse().unwrap_or(s.lat);
                        state.set(s);
                    }
                />
            </div>
        </div>
    }
}
