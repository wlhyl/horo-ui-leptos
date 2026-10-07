//! 提示对话框（对应原版 Ionic AlertController 的 header / message / buttons 模式）。
use leptos::control_flow::Show;
use leptos::prelude::*;

stylance::import_crate_style!(alert, "src/components/alert/alert.module.css");

/// 自定义按钮（对应 Ionic Alert 的 button）：文案 + 点击回调。
/// 回调执行后对话框不自动关闭，需要关闭时由回调清空 `message`。
#[derive(Clone, Copy)]
pub struct AlertAction {
    pub text: &'static str,
    pub on_click: Callback<()>,
}

/// 模态提示对话框：`message` 为空时隐藏；点遮罩关闭。
/// 默认只有单个「确定」按钮；传入 `buttons` 时渲染自定义按钮组
/// （点遮罩同样关闭，充当取消语义）。
#[component]
pub fn AlertDialog(
    header: &'static str,
    message: RwSignal<String>,
    #[prop(optional)] buttons: Option<Vec<AlertAction>>,
) -> impl IntoView {
    let close = move || message.set(String::new());
    // 按钮数据入 StoredValue：Show 的 children 闭包要求 Fn（可多次重建视图），
    // 闭包内只捕获 Copy 的句柄，每次渲染时从存储取值重建按钮
    let buttons = StoredValue::new(buttons);
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
                    {move || match buttons.get_value() {
                        None => view! {
                            <button class=alert::btn on:click=move |_| close()>"确定"</button>
                        }
                        .into_any(),
                        Some(actions) => actions
                            .into_iter()
                            .map(|action| {
                                view! {
                                    <button
                                        class=alert::btn
                                        on:click=move |_| action.on_click.run(())
                                    >
                                        {action.text}
                                    </button>
                                }
                            })
                            .collect::<Vec<_>>()
                            .into_any(),
                    }}
                </div>
            </div>
        </Show>
    }
}
