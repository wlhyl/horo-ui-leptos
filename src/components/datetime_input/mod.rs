//! 日期 / 时间 / 时区 / 夏令时输入（绑定 FormState 信号）。
//!
//! 输入框必须用 `prop:value`（设 DOM property）而非 `value`（设 attribute）：
//! 用户输入过的输入框会置 dirty 标志，此后 attribute 更新不再影响显示值。
use leptos::prelude::*;

use crate::models::datetime::DateTimeData;
use crate::native::input::FormState;

// 作用域样式：src/components/datetime_input/datetime_input.module.css
stylance::import_crate_style!(style, "src/components/datetime_input/datetime_input.module.css");
stylance::import_crate_style!(#[allow(dead_code)] form, "src/shared/form.module.css");

#[component]
pub fn DateTimeInput(state: RwSignal<FormState>) -> impl IntoView {
    // 一键填充为当前本地时间（对应原版 nowDate）；时区取浏览器本地时区，夏令时保持不动
    let now = move |_| {
        let t = DateTimeData::now();
        let mut s = state.get();
        s.year = t.year;
        s.month = t.month;
        s.day = t.day;
        s.hour = t.hour;
        s.minute = t.minute;
        s.second = t.second;
        s.tz = t.tz;
        state.set(s);
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
                        prop:value=move || state.get().year.to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                let mut s = state.get();
                                s.year = v;
                                state.set(s);
                            }
                        }
                    />
                    <input
                        type="number"
                        min="1"
                        max="12"
                        placeholder="月"
                        prop:value=move || state.get().month.to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                let mut s = state.get();
                                s.month = v;
                                state.set(s);
                            }
                        }
                    />
                    <input
                        type="number"
                        min="1"
                        max="31"
                        placeholder="日"
                        prop:value=move || state.get().day.to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                let mut s = state.get();
                                s.day = v;
                                state.set(s);
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
                        prop:value=move || state.get().hour.to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                let mut s = state.get();
                                s.hour = v;
                                state.set(s);
                            }
                        }
                    />
                    <input
                        type="number"
                        min="0"
                        max="59"
                        placeholder="分"
                        prop:value=move || state.get().minute.to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                let mut s = state.get();
                                s.minute = v;
                                state.set(s);
                            }
                        }
                    />
                    <input
                        type="number"
                        min="0"
                        max="59"
                        placeholder="秒"
                        prop:value=move || state.get().second.to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse() {
                                let mut s = state.get();
                                s.second = v;
                                state.set(s);
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
                    prop:value=move || state.get().tz.to_string()
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse() {
                            let mut s = state.get();
                            s.tz = v;
                            state.set(s);
                        }
                    }
                />
                <label class=form::toggle>
                    <input
                        type="checkbox"
                        checked=move || state.get().st
                        on:change=move |ev| {
                            let mut s = state.get();
                            s.st = event_target_checked(&ev);
                            state.set(s);
                        }
                    />
                    <span>"夏令时"</span>
                </label>
            </div>
        </div>
    }
}
