//! 宫位系统下拉（绑定 FormState Store，字段级读写）。
use leptos::prelude::*;
use reactive_stores::Store;

use crate::components::{FormState, FormStateStoreFields, SelectPopup, flat_group};
use crate::enums::house::HouseName;

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
                <SelectPopup
                    current=move || Some(state.house().get())
                    on_pick=move |h| state.house().set(h)
                    groups=move || flat_group(HouseName::ALL)
                    label=|h| h.as_str().to_string()
                />
            </div>
        </div>
    }
}
