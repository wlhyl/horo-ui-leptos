//! 从档案库选择天宫图记录（对应原版 archive-selector + archive-selection-modal）。
//!
//! 入口按钮点击时校验登录态，未登录以 AlertDialog 提示；已登录弹出模态选择框：
//! 支持按姓名搜索（300ms 防抖）与「加载更多」分页（每页 20 条）；点选记录后
//! 回填输入表单，宫位制保持表单当前值（对齐原版 defaultHouse 逻辑）。
//!
//! 状态一律用 `RwSignal`（而非 `Cell`/`Rc`）：视图与事件处理闭包要求 Send，
//! 信号类型满足且为 Copy，可直接在闭包间共享。
use leptos::control_flow::Show;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::client::{get_horoscopes, search_horoscopes};
use crate::api::response::{ChartType, HoroscopeRecord};
use crate::auth::AuthService;
use crate::components::AlertDialog;
use crate::native::input::FormState;

// 作用域样式：src/components/archive_selector/archive_selector.module.css
stylance::import_crate_style!(
    style,
    "src/components/archive_selector/archive_selector.module.css"
);

/// 每页记录数（与原版一致）。
const PAGE_SIZE: u64 = 20;
/// 搜索防抖间隔（毫秒）。
const SEARCH_DEBOUNCE_MS: i32 = 300;

/// 基于 Promise 的毫秒级休眠（wasm 无标准线程，借浏览器 setTimeout 实现）。
/// 直接把 Promise 的 resolve 作为定时回调，无需构造额外闭包。
async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve: js_sys::Function, _| {
        let _ = web_sys::window()
            .map(|w| w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms));
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// 从档案库选择天宫图记录：入口按钮 + 模态选择框。
#[component]
pub fn ArchiveSelector(state: RwSignal<FormState>) -> impl IntoView {
    let auth = use_context::<AuthService>().expect("AuthService 未初始化");

    let open = RwSignal::new(false);
    let records = RwSignal::new(Vec::<HoroscopeRecord>::new());
    let query = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(String::new());
    // 未登录提示（复用 AlertDialog）
    let notice = RwSignal::new(String::new());
    let page = RwSignal::new(0u64);
    let total_pages = RwSignal::new(0u64);
    let is_search_mode = RwSignal::new(false);
    // 搜索防抖代数：每次输入自增，休眠结束后代数不一致说明已被更新的输入取代
    let search_generation = RwSignal::new(0u64);
    // 请求代数：发起请求时自增，响应回来时代数不符说明已被更新的请求取代，丢弃即可。
    // 取代「把 loading 当并发锁」的做法，避免搜索在途时重开弹框等场景下旧响应覆盖新列表。
    let request_generation = RwSignal::new(0u64);

    // 加载一页记录：`search_name` 为 Some 走搜索接口，否则走分页列表；
    // `reset` 为 true 表示用响应整体替换列表（第 0 页），false 为追加到尾部。
    // loading / error 的置位统一在这里处理，调用方只管业务参数；
    // 注意 reset 时不预先清空 records：等响应到达再整体替换，避免搜索时
    // 列表先塌缩成「加载中」再撑开造成模态框高度来回跳（闪烁）。
    // 只捕获 Copy 的信号，闭包自身也是 Copy，可被多个事件闭包各自 move 走。
    let load = move |page: u64, search_name: Option<String>, reset: bool| {
        let Some(token) = auth.token() else { return };
        request_generation.update(|g| *g += 1);
        let seq = request_generation.get_untracked();
        loading.set(true);
        error.set(String::new());
        spawn_local(async move {
            let res = match &search_name {
                Some(name) => search_horoscopes(page, PAGE_SIZE, name, &token).await,
                None => get_horoscopes(page, PAGE_SIZE, &token).await,
            };
            if request_generation.get_untracked() != seq {
                return;
            }
            match res {
                Ok(r) => {
                    if reset {
                        records.set(r.data);
                    } else {
                        records.update(|v| v.extend(r.data));
                    }
                    total_pages.set(r.total);
                    error.set(String::new());
                }
                Err(msg) => error.set(msg),
            }
            loading.set(false);
        });
    };

    // 打开选择框：重置为列表模式并加载第 0 页
    let open_modal = move |_| {
        // user() 已登录时 token 必然存在；取不到视为登录态异常，按未登录提示
        // （load 内部自行取 token，这里只做登录态校验）
        if auth.user().is_none() || auth.token().is_none() {
            notice.set("请先登录后再从档案库选择记录".into());
            return;
        }
        open.set(true);
        is_search_mode.set(false);
        page.set(0);
        query.set(String::new());
        // 打开瞬间先清旧列表（此时弹框尚未可见，不会看到塌缩），
        // 首屏由 loading + 空列表呈现「加载中…」
        records.set(Vec::new());
        total_pages.set(0);
        // 作废可能在途的搜索防抖任务
        search_generation.update(|g| *g += 1);
        load(0, None, true);
    };

    let close = move |_| open.set(false);

    // 搜索输入：空串回到分页列表模式，非空走姓名搜索；统一经防抖触发，
    // 也顺带解决「清空输入时上一次防抖任务仍在排队」的竞态。
    let on_search_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        query.set(value.clone());
        let name = value.trim().to_owned();
        // 代数自增：让仍在休眠的旧防抖任务作废
        search_generation.update(|g| *g += 1);
        let snapshot = search_generation.get_untracked();
        spawn_local(async move {
            sleep_ms(SEARCH_DEBOUNCE_MS).await;
            // 已有更新的输入则本次作废（请求侧另有 request_generation 兜底）
            if search_generation.get_untracked() != snapshot {
                return;
            }
            let search = (!name.is_empty()).then_some(name);
            is_search_mode.set(search.is_some());
            page.set(0);
            load(0, search, true);
        });
    };

    // 加载更多：还有下一页时追加（列表 / 搜索模式分别走对应接口）。
    // loading 守卫只防重复点击同一页——追加必须串行，否则会漏掉中间页。
    let load_more = move |_| {
        if loading.get_untracked() || page.get_untracked() + 1 >= total_pages.get_untracked() {
            return;
        }
        let next = page.get_untracked() + 1;
        page.set(next);
        let search = is_search_mode
            .get_untracked()
            .then(|| query.get_untracked().trim().to_owned());
        load(next, search, false);
    };

    // 点选记录：回填表单（house 保持表单当前值不覆盖）并关闭选择框。
    // 加载中禁止点选：列表可能还是上一次搜索的旧数据
    let select = move |i: usize| {
        if loading.get_untracked() {
            return;
        }
        let Some(r) = records.get_untracked().into_iter().nth(i) else {
            return;
        };
        // 度分秒 → 十进制；西经 / 南纬取负
        let mut long = f64::from(r.location.longitude_degree)
            + f64::from(r.location.longitude_minute) / 60.0
            + f64::from(r.location.longitude_second) / 3600.0;
        let mut lat = f64::from(r.location.latitude_degree)
            + f64::from(r.location.latitude_minute) / 60.0
            + f64::from(r.location.latitude_second) / 3600.0;
        if !r.location.is_east {
            long = -long;
        }
        if !r.location.is_north {
            lat = -lat;
        }

        let mut s = state.get();
        s.id = r.id;
        s.name = r.name;
        s.sex = r.gender;
        s.year = r.birth_year;
        s.month = r.birth_month;
        s.day = r.birth_day;
        s.hour = r.birth_hour;
        s.minute = r.birth_minute;
        s.second = r.birth_second;
        s.tz = r.time_zone_offset;
        s.st = r.is_dst;
        s.geo_name = r.location.name.clone();
        s.long = long;
        s.lat = lat;
        state.set(s);
        open.set(false);
    };

    view! {
        // 入口按钮（表单内的次级操作）
        <button class=style::entry_btn on:click=open_modal>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M8 6h13M8 12h13M8 18h13"/>
                <path d="M3 6h.01M3 12h.01M3 18h.01"/>
            </svg>
            "从档案库选择"
        </button>

        // 模态选择框：点遮罩不关闭（避免误触丢搜索状态），经右上角按钮关闭
        <Show when=move || open.get() fallback=|| ()>
            <div class=style::backdrop>
                <div class=style::modal role="dialog" aria-modal="true">
                    <div class=style::modal_header>
                        <h3 class=style::modal_title>"选择档案记录"</h3>
                        <button class=style::close_btn on:click=close aria-label="关闭">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.5" stroke-linecap="round">
                                <path d="M6 6l12 12M18 6L6 18"/>
                            </svg>
                        </button>
                    </div>

                    <div class=style::search_wrap>
                        <div class=style::search_field>
                            <svg class=style::search_icon viewBox="0 0 24 24" fill="none"
                                stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                                <circle cx="11" cy="11" r="7"/>
                                <path d="m20 20-3.5-3.5"/>
                            </svg>
                            <input
                                class=style::search_input
                                type="text"
                                placeholder="按姓名搜索…"
                                prop:value=move || query.get()
                                on:input=on_search_input
                            />
                        </div>
                    </div>

                    // 接口错误提示
                    {move || {
                        (!error.get().is_empty())
                            .then(|| view! { <div class=style::error>{move || error.get()}</div> })
                    }}

                    // 加载中且列表非空（搜索替换场景）：旧列表半透明提示数据在途
                    <div class=move || {
                        if loading.get() && !records.get().is_empty() {
                            format!("{} {}", style::list, style::list_loading)
                        } else {
                            style::list.into()
                        }
                    }>
                        {move || {
                            records
                                .get()
                                .into_iter()
                                .enumerate()
                                .map(|(i, r)| {
                                    let horary = r.chart_type == ChartType::Horary;
                                    let precise = r.is_time_precise;
                                    let date = format!(
                                        "{}/{}/{}",
                                        r.birth_year, r.birth_month, r.birth_day
                                    );
                                    view! {
                                        <div class=style::item on:click=move |_| select(i)>
                                            <div class=style::item_main>
                                                {horary
                                                    .then(|| {
                                                        view! { <span class=style::badge_h>"H"</span> }
                                                    })}
                                                <span class=move || if precise {
                                                    style::name_precise
                                                } else {
                                                    style::name
                                                }>
                                                    {r.name}
                                                </span>
                                            </div>
                                            <span class=style::item_date>{date}</span>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()
                        }}
                    </div>

                    // 首屏加载 / 空态 / 追加加载 / 加载更多
                    <div class=style::footer>
                        {move || {
                            (loading.get() && records.get().is_empty())
                                .then(|| view! { <span class=style::empty>"加载中…"</span> })
                        }}
                        {move || {
                            (!loading.get()
                                && records.get().is_empty()
                                && error.get().is_empty())
                                .then(|| view! { <span class=style::empty>"暂无数据"</span> })
                        }}
                        {move || {
                            (loading.get() && !records.get().is_empty())
                                .then(|| view! { <span class=style::empty>"加载中…"</span> })
                        }}
                        {move || {
                            (!loading.get()
                                && !records.get().is_empty()
                                && page.get() + 1 < total_pages.get())
                                .then(|| {
                                    view! {
                                        <button class=style::more_btn on:click=load_more>
                                            "加载更多"
                                        </button>
                                    }
                                })
                        }}
                    </div>
                </div>
            </div>
        </Show>

        // 未登录提示
        <AlertDialog header="提示" message=notice/>
    }
}
