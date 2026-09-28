//! 宫位系统下拉（绑定 FormState Store，字段级读写）。
use leptos::prelude::*;
use reactive_stores::Store;

use crate::enums::house::HouseName;
use crate::components::{FormState, FormStateStoreFields};

stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);

#[component]
pub fn HouseSelect(state: Store<FormState>) -> impl IntoView {
    view! {
        <div class=form::field>
            <label>"宫位系统"</label>
            <div class=form::control>
                <select
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        state
                            .house()
                            .set(HouseName::from_str(&v).unwrap_or(HouseName::Regiomontanus));
                    }
                >
                    {HouseName::ALL
                        .iter()
                        .map(|&h| {
                            let selected = move || state.house().get() == h;
                            view! { <option value=h.as_str() selected=selected>{h.as_str()}</option> }
                        })
                        .collect::<Vec<_>>()}
                </select>
            </div>
        </div>
    }
}
