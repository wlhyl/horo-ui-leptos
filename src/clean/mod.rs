//! 清除缓存页（对应原版 horo-ui 的 CleanPage）。
//!
//! 点击按钮清空 localStorage 中的七份星盘缓存并恢复默认值；
//! `HoroStorage::clean()` 为同步操作，完成后直接显示提示。
use leptos::prelude::*;

use crate::storage::HoroStorage;

// 作用域样式：src/clean/clean.module.css
stylance::import_crate_style!(style, "src/clean/clean.module.css");
// 显式共享的卡片容器样式
stylance::import_crate_style!(card, "src/shared/card.module.css");
// 主操作按钮样式（模块内其余类未使用，关闭 dead_code 警告）
stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);

#[component]
pub fn Clean() -> impl IntoView {
    let storage = use_context::<HoroStorage>().expect("HoroStorage 未初始化");
    // 完成提示（原版 message；clean() 同步完成，无中间态）
    let message = RwSignal::new(String::new());

    let clean = move |_| {
        storage.clean();
        message.set("清除缓存完成".into());
    };

    view! {
        <div class=card::card>
            <h2>"清除缓存"</h2>
            <p class=style::desc>
                "清空本地保存的本命、推运、合盘、天象等输入数据，并恢复默认值。"
            </p>
            <button class=form::btn_primary on:click=clean>"清除缓存"</button>
            {move || {
                (!message.get().is_empty())
                    .then(|| view! { <div class=style::done>{message.get()}</div> })
            }}
        </div>
    }
}
