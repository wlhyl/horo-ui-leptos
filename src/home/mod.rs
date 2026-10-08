//! 入口页：选择进入本命星盘或天象盘。
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;

use crate::routes::AppRoute;

// 作用域样式：src/home/home.module.css
stylance::import_crate_style!(style, "src/home/home.module.css");

#[component]
pub fn Home() -> impl IntoView {
    // 必须在渲染期取导航器（事件闭包里调用会拿不到 Router 上下文）
    let nav = use_navigate();

    let go_native = {
        let nav = nav.clone();
        move |_| {
            nav(AppRoute::Native.path(), NavigateOptions::default());
        }
    };
    let go_event = {
        let nav = nav.clone();
        move |_| {
            nav(AppRoute::Event.path(), NavigateOptions::default());
        }
    };
    let go_derived = {
        let nav = nav.clone();
        move |_| {
            nav(AppRoute::Derived.path(), NavigateOptions::default());
        }
    };
    let go_process = {
        let nav = nav.clone();
        move |_| {
            nav(AppRoute::Process.path(), NavigateOptions::default());
        }
    };
    let go_clean = {
        let nav = nav.clone();
        move |_| {
            nav(AppRoute::Clean.path(), NavigateOptions::default());
        }
    };
    let go_power = {
        let nav = nav.clone();
        move |_| {
            nav(AppRoute::Power.path(), NavigateOptions::default());
        }
    };
    let go_workbench = {
        let nav = nav.clone();
        move |_| {
            nav(AppRoute::Workbench.path(), NavigateOptions::default());
        }
    };

    view! {
        <div class=style::home>
            <div class=style::home_grid>
                <button class=style::home_card on:click=go_workbench>
                    <div class=style::home_icon>
                        // 多窗口网格（工作台）
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="1.5" stroke-linejoin="round">
                            <rect x="3" y="3" width="8" height="12" rx="1.5"/>
                            <rect x="13" y="7" width="8" height="12" rx="1.5"/>
                        </svg>
                    </div>
                    <div class=style::home_name>"工作台"</div>
                    <div class=style::home_desc>"多窗口星盘对比"</div>
                </button>
                <button class=style::home_card on:click=go_native>
                    <div class=style::home_icon>
                        // 五角星轮廓，同 horo-ui 本命入口的 star-outline 图案
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="1.5" stroke-linejoin="round">
                            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2z"/>
                        </svg>
                    </div>
                    <div class=style::home_name>"本命盘"</div>
                    <div class=style::home_desc>"出生时间 · 出生地点"</div>
                </button>
                // 推运入口：主向推运 / 每日回归方向弧 / 太阳弧（对应原版推运菜单）
                <button class=style::home_card on:click=go_process>
                    <div class=style::home_icon>
                        // 沙漏轮廓，象征时间维度上的推进
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M6 3h12"/>
                            <path d="M6 21h12"/>
                            <path d="M7 3v3a5 5 0 0 0 5 5 5 5 0 0 0 5-5V3"/>
                            <path d="M7 21v-3a5 5 0 0 1 5-5 5 5 0 0 1 5 5v3"/>
                        </svg>
                    </div>
                    <div class=style::home_name>"推运"</div>
                    <div class=style::home_desc>"方向推运 · 返照盘 · 比较盘"</div>
                </button>
                <button class=style::home_card on:click=go_event>
                    <div class=style::home_icon>
                        // 地球仪轮廓，同 horo-ui 天象盘入口的 globe-outline 图案
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="12" cy="12" r="10"/>
                            <path d="M2 12h20"/>
                            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                        </svg>
                    </div>
                    <div class=style::home_name>"天象盘"</div>
                    <div class=style::home_desc>"任意时刻天象"</div>
                </button>
                // 衍生盘入口：以出生数据为基准的旋转盘（基准行星斜升为中天）
                <button class=style::home_card on:click=go_derived>
                    <div class=style::home_icon>
                        // 顺时针环形箭头，象征整盘绕基准行星旋转
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M21 12a9 9 0 1 1-2.64-6.36"/>
                            <path d="M21 3v6h-6"/>
                        </svg>
                    </div>
                    <div class=style::home_name>"衍生盘"</div>
                    <div class=style::home_desc>"基准行星斜升为中天"</div>
                </button>
                // 行星力量表入口：静态占星参考（庙/旺/三分/界/面/陷/落）
                <button class=style::home_card on:click=go_power>
                    <div class=style::home_icon>
                        // 天平轮廓，象征尊贵力量对照
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M12 3v18"/>
                            <path d="M7 21h10"/>
                            <path d="M5 7h14"/>
                            <path d="M5 7l-3 6a3 3 0 0 0 6 0z"/>
                            <path d="M19 7l-3 6a3 3 0 0 0 6 0z"/>
                            <circle cx="12" cy="4" r="1.4"/>
                        </svg>
                    </div>
                    <div class=style::home_name>"行星力量表"</div>
                    <div class=style::home_desc>"庙旺休囚 · 界面尊贵"</div>
                </button>
                // 清除缓存入口（对应原版 home 页的 Clean 菜单项）
                <button class=style::home_card on:click=go_clean>
                    <div class=style::home_icon>
                        // 垃圾桶轮廓，表示清除/缓存清理
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M3 6h18"/>
                            <path d="M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2"/>
                            <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
                            <path d="M10 11v6"/>
                            <path d="M14 11v6"/>
                        </svg>
                    </div>
                    <div class=style::home_name>"清除缓存"</div>
                    <div class=style::home_desc>"清空本地星盘数据"</div>
                </button>
            </div>
        </div>
    }
}
