//! 宫位系统下拉（绑定 FormState 信号）。
use leptos::prelude::*;

use crate::enums::house::HouseName;
use crate::native::input::FormState;

stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);

#[component]
pub fn HouseSelect(state: RwSignal<FormState>) -> impl IntoView {
    view! {
        <div class=form::field>
            <label>"宫位系统"</label>
            <div class=form::control>
                <select
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        let mut s = state.get();
                        s.house = HouseName::from_str(&v).unwrap_or(HouseName::Regiomontanus);
                        state.set(s);
                    }
                >
                    {HouseName::ALL
                        .iter()
                        .map(|&h| {
                            let selected = move || state.get().house == h;
                            view! { <option value=h.as_str() selected=selected>{h.as_str()}</option> }
                        })
                        .collect::<Vec<_>>()}
                </select>
            </div>
        </div>
    }
}
