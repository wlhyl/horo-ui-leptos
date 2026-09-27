//! 日期 / 时间 / 时区 / 夏令时输入（绑定 FormState Store，字段级读写）。
//!
//! 输入框必须用 `prop:value`（设 DOM property）而非 `value`（设 attribute）：
//! 用户输入过的输入框会置 dirty 标志，此后 attribute 更新不再影响显示值。
use leptos::prelude::*;
use reactive_stores::Store;

use crate::models::datetime::DateTimeData;
use crate::native::input::{FormState, FormStateStoreFields};

// 作用域样式：src/components/datetime_input/datetime_input.module.css
stylance::import_crate_style!(style, "src/components/datetime_input/datetime_input.module.css");
stylance::import_crate_style!(#[allow(dead_code)] form, "src/shared/form.module.css");

#[component]
pub fn DateTimeInput(state: Store<FormState>) -> impl IntoView {
    // 一键填充为当前本地时间（对应原版 nowDate）；时区取浏览器本地时区，夏令时保持不动。
    // 多字段一次整体写入：单次通知，避免逐字段 set 产生新旧混合的中间日期。
    let now = move |_| {
        let t = DateTimeData::now();
        state.update(|s| {
            s.year = t.year;
            s.month = t.month;
            s.day = t.day;
            s.hour = t.hour;
            s.minute = t.minute;
            s.second = t.second;
            s.tz = t.tz;
        });
    };

    view! {
        <div class=form::field>
            <label>"日期"</label>
            <div class=form::control>
                <div class=form::row_3>
                    <input
                        type="number"
                        min="1900"
                        placeholder="年"
                        prop:value=move || state.year().get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                state.year().set(v);
                            }
                        }
                    />
                    <input
                        type="number"
                        min="1"
                        max="12"
                        placeholder="月"
                        prop:value=move || state.month().get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                state.month().set(v);
                            }
                        }
                    />
                    <input
                        type="number"
                        min="1"
                        max="31"
                        placeholder="日"
                        prop:value=move || state.day().get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                state.day().set(v);
                            }
                        }
                    />
                </div>
                <button class=style::now_btn on:click=now>"现在"</button>
            </div>
        </div>

        <div class=form::field>
            <label>"时间"</label>
            <div class=form::control>
                <div class=form::row_3>
                    <input
                        type="number"
                        min="0"
                        max="23"
                        placeholder="时"
                        prop:value=move || state.hour().get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                state.hour().set(v);
                            }
                        }
                    />
                    <input
                        type="number"
                        min="0"
                        max="59"
                        placeholder="分"
                        prop:value=move || state.minute().get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                state.minute().set(v);
                            }
                        }
                    />
                    <input
                        type="number"
                        min="0"
                        max="59"
                        placeholder="秒"
                        prop:value=move || state.second().get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                state.second().set(v);
                            }
                        }
                    />
                </div>
            </div>
        </div>

        <div class=form::field>
            <label>"时区"</label>
            <div class=form::control>
                <input
                    type="number"
                    step="0.5"
                    class=style::tz_input
                    prop:value=move || state.tz().get().to_string()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse() {
                            state.tz().set(v);
                        }
                    }
                />
                <label class=form::toggle>
                    <input
                        type="checkbox"
                        bind:checked=state.st()
                    />
                    <span>"夏令时"</span>
                </label>
            </div>
        </div>
    }
}
