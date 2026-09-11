//! 提示对话框（对应原版 Ionic AlertController 的 header / message / buttons 模式）。
use leptos::control_flow::Show;
use leptos::prelude::*;

stylance::import_crate_style!(alert, "src/components/alert/alert.module.css");

/// 模态提示对话框：`message` 为空时隐藏；点「确定」或遮罩关闭。
#[component]
pub fn AlertDialog(header: &'static str, message: RwSignal<String>) -> impl IntoView {
    let close = move || message.set(String::new());
    view! {
        <Show when=move || !message.get().is_empty() fallback=|| ()>
            <div class=alert::backdrop on:click=move |_| close()>
                <div
                    class=alert::dialog
                    role="alertdialog"
                    aria-modal="true"
                    on:click=|ev| ev.stop_propagation()
                >
                    <h3 class=alert::header>{header}</h3>
                    <p class=alert::message>{move || message.get()}</p>
                    <button class=alert::btn on:click=move |_| close()>"确定"</button>
                </div>
            </div>
        </Show>
    }
}
