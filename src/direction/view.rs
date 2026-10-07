//! 方向推运结果视图（对应原版 direction.component）。
//!
//! 三种模式共用：主向推运 / 每日回归方向弧 / 太阳弧。视图持出生与推运参数快照，
//! 全部编辑只影响本视图的请求，不写回 HoroStorage（对齐原版组件状态）。
//! 独立页（DirectionPage）从 HoroStorage 取快照；工作台窗口直接传开窗快照，
//! 选中恰一个显著星时经 `on_title` 回调更新窗口标题（对齐原版 titleChange）。
use leptos::control_flow::Show;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use reactive_stores::Store;
use wasm_bindgen_futures::spawn_local;

use crate::api::client::{
    post_daily_direction, post_daily_return, post_direction, post_lunar_return, post_solar_arc,
    post_solar_return,
};
use crate::api::request::{
    DailyDirectionRequest, DirectionRequest, ReturnRequest, SolarArcRequest,
};
use crate::api::response::{Direction, Promittor, Significator};
use crate::components::{
    AlertDialog, ChartTimeEditor, FormState, FormStateStoreFields, GeoInput, HouseSelect,
};
use crate::direction::utils::{
    ALL_PROMITTOR_TYPES, ALL_SIGNIFICATORS, ARC_TO_DATE_METHODS, ArcDirectionFilter,
    DAILY_DIRECTION_METHODS, DIRECTION_METHODS, PromittorType, add_years, check_date_range,
    check_promittor_planet, check_promittor_type, check_significator, format_arc, format_date,
    method_select, now_minus_years, process_title, promittor_antiscia, promittor_aspect,
    promittor_cusp, promittor_planet, promittor_sign, significator_cusp, significator_planet,
    term_info,
};
use crate::enums::{
    arc_to_date_method::ArcToDateMethod, daily_direction_method::DailyDirectionMethod,
    direction_method::DirectionMethod, house::HouseName, planet::PlanetName,
    process_name::ProcessName, zodiac::Zodiac,
};
use crate::models::data::{HoroData, ProcessData};
use crate::models::datetime::DateTimeData;
use crate::models::geo::GeoPosition;
use crate::render::glyphs::{aspect_glyph, planet_color, planet_glyph, zodiac_glyph};
use crate::routes::AppRoute;
use crate::shared::sleep_ms;
use crate::storage::HoroStorage;

stylance::import_crate_style!(style, "src/direction/view.module.css");
stylance::import_crate_style!(
    #[allow(dead_code)]
    form,
    "src/shared/form.module.css"
);
stylance::import_crate_style!(
    #[allow(dead_code)]
    feedback,
    "src/shared/feedback.module.css"
);

/// 更新按钮的防抖间隔（毫秒，对齐原版 updateNativeDateSubject 的 debounceTime(300)）。
const UPDATE_DEBOUNCE_MS: i32 = 300;

/// 默认勾选的显著星（对应原版默认 selectedSignificatorPlanets）。
const DEFAULT_SIGNIFICATORS: [PlanetName; 5] = [
    PlanetName::ASC,
    PlanetName::MC,
    PlanetName::Sun,
    PlanetName::Moon,
    PlanetName::PartOfFortune,
];

/// 按模式分流后台请求。返回 (方向记录, 每日返照时刻——仅每日回归模式)。
async fn fetch_directions(
    mode: ProcessName,
    native_date: DateTimeData,
    geo: GeoPosition,
    house: HouseName,
    process: ProcessData,
    direction_method: DirectionMethod,
    arc_to_date_method: ArcToDateMethod,
    daily_direction_method: DailyDirectionMethod,
) -> Result<(Vec<Direction>, Option<DateTimeData>), String> {
    match mode {
        ProcessName::SolarArc => {
            let list = post_solar_arc(&SolarArcRequest {
                native_date,
                geo,
                house,
            })
            .await?;
            Ok((list, None))
        }
        ProcessName::DailyDirection => {
            // 先取每日返照时刻：开启「日返月亮」时按 日返→月返→每日回归 逐层链式
            // （每步以返照时刻为下一步的本命时间、st 固定 false，对齐原版
            // fetchDailyReturnChain / getLunarDailyReturnData）
            let return_req = |native_date: DateTimeData| ReturnRequest {
                native_date,
                process_date: process.date,
                geo,
                house,
            };
            let ret = if process.is_solar_return {
                let solar = post_solar_return(&return_req(native_date)).await?;
                let mut d = solar.return_date;
                d.st = false;
                let lunar = post_lunar_return(&return_req(d)).await?;
                let mut d = lunar.return_date;
                d.st = false;
                post_daily_return(&return_req(d)).await?
            } else {
                post_daily_return(&return_req(native_date)).await?
            };
            let mut return_date = ret.return_date;
            return_date.st = false;
            let list = post_daily_direction(&DailyDirectionRequest {
                native_date: return_date,
                geo,
                method: daily_direction_method,
                house,
            })
            .await?;
            Ok((list, Some(return_date)))
        }
        // 主向推运（默认分支）
        _ => {
            let list = post_direction(&DirectionRequest {
                native_date,
                geo,
                method: direction_method,
                arc_to_date_method,
                house,
            })
            .await?;
            Ok((list, None))
        }
    }
}

/// 结果独立页（/direction /daily_direction /solar_arc）：页头 + 从缓存取快照。
#[component]
pub(crate) fn DirectionPage(mode: ProcessName) -> impl IntoView {
    let storage = use_context::<HoroStorage>().expect("HoroStorage 未初始化");
    // 必须在渲染期（Router 上下文内）取导航器，事件闭包里调用会拿不到上下文
    let nav = use_navigate();
    let go_back = move |_| {
        nav(AppRoute::Home.path(), NavigateOptions::default());
    };
    // 进入页面时从缓存取出生与推运参数快照（对应原版组件读 storage.horoData /
    // storage.processData）；此后页面编辑不回写。
    let horo = storage.horo_data();
    let process = storage.process_data();

    view! {
        <div class=style::page>
            <header class=style::header>
                <button class=style::back_btn on:click=go_back aria-label="返回">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M15 18l-6-6 6-6"/>
                    </svg>
                </button>
                <h1 class=style::title>{process_title(mode)}</h1>
            </header>
            <DirectionView mode horo process on_title=None/>
        </div>
    }
}

/// 单元格：行星 / 四轴福点符号（着色）。
fn planet_span(p: PlanetName) -> impl IntoView {
    view! {
        <span class=style::glyph style=format!("color: {}", planet_color(p))>
            {planet_glyph(p)}
        </span>
    }
}

/// 单元格：相位符号。
fn aspect_span(deg: u8) -> impl IntoView {
    view! {
        <span class=style::glyph>{aspect_glyph(deg)}</span>
    }
}

/// Significator 单元格：行星符号或宫头 "3C"。
fn significator_cell(sig: &Significator) -> impl IntoView {
    let planet = significator_planet(sig);
    let cusp_text = significator_cusp(sig).map(|c| format!("{c}C"));
    view! {
        {planet.map(planet_span)}
        {cusp_text.map(|t| view! { <span class=style::plain>{t}</span> })}
    }
}

/// Promittor 单元格，顺序对齐原版模板：
/// 左相位符号 → Ant/C-Ant → 宫头 3C → 星座符号 → 行星符号 → 右相位符号 → 界（星座符号 + 界内度数）。
fn promittor_cell(p: &Promittor) -> impl IntoView {
    let aspect = promittor_aspect(p);
    let left = aspect
        .filter(|&(_, is_left)| is_left)
        .map(|(deg, _)| aspect_span(deg));
    let right = aspect
        .filter(|&(_, is_left)| !is_left)
        .map(|(deg, _)| aspect_span(deg));
    let antiscia = promittor_antiscia(p);
    let cusp = promittor_cusp(p);
    let sign = promittor_sign(p).map(|s| zodiac_glyph(Zodiac::from_index(s)));
    let planet = promittor_planet(p);
    let term = term_info(p);

    view! {
        {left}
        {antiscia.map(|t| view! { <span class=style::plain>{t}</span> })}
        {cusp.map(|c| view! { <span class=style::plain>{format!("{c}C")}</span> })}
        {sign.map(|g| view! { <span class=style::glyph>{g}</span> })}
        {planet.map(planet_span)}
        {right}
        {term.map(|(z, dms)| view! {
            <span class=style::glyph>{zodiac_glyph(z)}</span>
            <span class=style::deg>{dms}</span>
        })}
    }
}

/// 方向推运结果视图：说明 + 参数 / 筛选 + 返照横幅 + 结果表。
#[component]
pub(crate) fn DirectionView(
    mode: ProcessName,
    horo: HoroData,
    process: ProcessData,
    /// 工作台标题回调：显著星筛选变化时输出标题后缀（如 "-MC"，空串表示不附加），
    /// 由窗口框架拼出完整标题；独立页传 None。
    on_title: Option<Callback<String>>,
) -> impl IntoView {
    // —— 页面本地状态（不写回 HoroStorage，对齐原版组件字段）——
    // 出生时间（对应原版 nativeDate）
    let native_date = RwSignal::new(horo.date);
    // 经纬度 + 宫位：FormState 仅作 GeoInput / HouseSelect 的编辑载体。
    // 每日回归模式的经纬度初始值取推运地（对齐原版 setGeoFromHoroData 的分支）
    let (geo_init, geo_name_init) = if mode == ProcessName::DailyDirection {
        (process.geo, process.geo_name.clone())
    } else {
        (horo.geo, horo.geo_name.clone())
    };
    let form = Store::new(FormState {
        id: 0,
        name: String::new(),
        sex: true,
        year: horo.date.year,
        month: horo.date.month,
        day: horo.date.day,
        hour: horo.date.hour,
        minute: horo.date.minute,
        second: horo.date.second,
        tz: horo.date.tz,
        st: horo.date.st,
        geo_name: geo_name_init.clone(),
        long: geo_init.long,
        lat: geo_init.lat,
        house: horo.house,
    });
    // 日期范围默认：当前-5年 ~ 出生+120年（对应原版 startDate / endDate）
    let start = RwSignal::new(now_minus_years(5, &horo.date));
    let end = RwSignal::new(add_years(horo.date, 120));

    // 算法选择（对应原版 directionMethod / arcToDateMethod / dailyDirectionMethod，
    // 初始值来自推运参数快照）
    let direction_method = RwSignal::new(process.direction_method);
    let arc_to_date_method = RwSignal::new(process.arc_to_date_method);
    let daily_direction_method = RwSignal::new(process.daily_direction_method);

    // 筛选条件（对应原版 selectedSignificatorPlanets / arcDirectionFilter /
    // promittorTypeFilter / selectedPromittorPlanets）
    let selected_significators = RwSignal::new(DEFAULT_SIGNIFICATORS.to_vec());
    let arc_filter = RwSignal::new(ArcDirectionFilter::Direct);
    let promittor_types = RwSignal::new(Vec::<PromittorType>::new());
    let promittor_planets = RwSignal::new(Vec::<PlanetName>::new());

    // 结果与请求状态
    let data = RwSignal::new(Vec::<Direction>::new());
    // 每日回归模式的返照时刻（横幅展示用）
    let return_time = RwSignal::new(None::<DateTimeData>);
    let loading = RwSignal::new(false);
    // 防抖代数：更新按钮每点一次自增；休眠结束时代数不符说明已被更新的点击取代，
    // 本次任务直接作废（不请求、不动 loading），由最新的任务负责收尾。
    let generation = RwSignal::new(0u64);
    let alert_msg = RwSignal::new(String::new());

    let process = StoredValue::new(process);

    // 请求（对应原版 fetchDirectionData）：进入页面立即请求，
    // 「更新」按钮经 300ms 防抖 + 代数失效后重新请求
    let request = move |debounce_ms: i32| {
        generation.update(|g| *g += 1);
        let curr_gen = generation.get_untracked();
        let native = native_date.get_untracked();
        let geo = GeoPosition {
            long: form.long().get_untracked(),
            lat: form.lat().get_untracked(),
        };
        let house = form.house().get_untracked();
        let dm = direction_method.get_untracked();
        let am = arc_to_date_method.get_untracked();
        let dm_daily = daily_direction_method.get_untracked();
        let process = process.get_value();
        loading.set(true);
        spawn_local(async move {
            if debounce_ms > 0 {
                sleep_ms(debounce_ms).await;
                if generation.get_untracked() != curr_gen {
                    return;
                }
            }
            let res = fetch_directions(mode, native, geo, house, process, dm, am, dm_daily).await;
            if generation.get_untracked() != curr_gen {
                return;
            }
            loading.set(false);
            match res {
                Ok((list, ret)) => {
                    data.set(list);
                    return_time.set(ret);
                }
                Err(e) => alert_msg.set(e),
            }
        });
    };

    // 首次立即请求（对齐原版 ngOnInit 的 fetchDirectionData）
    request(0);
    let update = move |_| request(UPDATE_DEBOUNCE_MS);

    // 重置筛选（对应原版 resetFilters）：恢复默认条件与推运地，不重新请求
    let reset = move |_| {
        let native = native_date.get_untracked();
        start.set(native);
        end.set(add_years(native, 120));
        selected_significators.set(Vec::new());
        arc_filter.set(ArcDirectionFilter::Direct);
        promittor_types.set(Vec::new());
        promittor_planets.set(Vec::new());
        form.long().set(geo_init.long);
        form.lat().set(geo_init.lat);
        form.geo_name().set(geo_name_init.clone());
    };

    // 过滤后的结果（五重条件：日期范围 / 显著星 / 弧方向 / 相位类型 / 承诺星行星）
    let filtered = Memo::new(move |_| {
        let s = start.get();
        let e = end.get();
        let sigs = selected_significators.get();
        let arc = arc_filter.get();
        let types = promittor_types.get();
        let planets = promittor_planets.get();
        data.get()
            .into_iter()
            .filter(|item| {
                check_date_range(&item.date, &s, &e)
                    && check_significator(&item.significator, &sigs)
                    && arc.check(item.arc)
                    && check_promittor_type(&item.promittor, &types)
                    && check_promittor_planet(&item.promittor, &planets)
            })
            .collect::<Vec<_>>()
    });

    // 工作台窗口标题后缀：主向推运 / 每日回归方向弧模式下选中恰一个显著星时附加
    // 该星（对齐原版 windowTitle + titleChange；太阳弧不附加）
    if let Some(on_title) = on_title {
        Effect::new(move |_| {
            let planets = selected_significators.get();
            let suffix = if planets.len() == 1
                && matches!(mode, ProcessName::Direction | ProcessName::DailyDirection)
            {
                format!("-{}", planet_glyph(planets[0]))
            } else {
                String::new()
            };
            on_title.run(suffix);
        });
    }

    // Significator / Promittor 行星 chips（点选切换，激活高亮）
    let planet_chips = |selected: RwSignal<Vec<PlanetName>>| {
        ALL_SIGNIFICATORS
            .iter()
            .copied()
            .map(|p| {
                let toggle = move |_| {
                    selected.update(|list| {
                        if list.contains(&p) {
                            list.retain(|x| *x != p);
                        } else {
                            list.push(p);
                        }
                    });
                };
                view! {
                    <button
                        class=move || {
                            if selected.get().contains(&p) {
                                style::chip_active
                            } else {
                                style::chip
                            }
                        }
                        on:click=toggle
                    >
                        <span class=style::glyph style=format!("color: {}", planet_color(p))>
                            {planet_glyph(p)}
                        </span>
                    </button>
                }
            })
            .collect::<Vec<_>>()
    };

    // 相位类型 chips
    let type_chips = ALL_PROMITTOR_TYPES
        .iter()
        .copied()
        .map(|t| {
            let toggle = move |_| {
                promittor_types.update(|list| {
                    if list.contains(&t) {
                        list.retain(|x| *x != t);
                    } else {
                        list.push(t);
                    }
                });
            };
            view! {
                <button
                    class=move || {
                        if promittor_types.get().contains(&t) {
                            style::chip_active
                        } else {
                            style::chip
                        }
                    }
                    on:click=toggle
                >
                    {t.to_string()}
                </button>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <ul class=style::intro>
            <li>"主向星为 MC（中天）"</li>
            <li>"弧度正值为正向推运，负值为反向推运"</li>
            <li>
                {if mode == ProcessName::DailyDirection {
                    "360度等于24小时（1度≈4分钟）"
                } else {
                    "一度约等于一年"
                }}
            </li>
            <li>"相位符号在行星左侧为承诺星的左相位，在右侧为右相位"</li>
        </ul>

        // —— 参数区：出生时间 / 经纬度 / 宫位 / 算法 / 日期范围 / 更新 ——
        // 日期用星盘页同款时间编辑条（点日期标签弹六列滚轮面板，确定才写回；
        // 步进器微调），改动仍需点「更新」才重新请求
        <div class=style::panel>
            <div class=form::field>
                <label>"出生时间"</label>
                <ChartTimeEditor date=native_date/>
            </div>
            // 地名常显，经度 / 纬度默认收起（对应原版 showGeoInput 折叠）
            <GeoInput state=form collapsible=true/>
            <HouseSelect state=form/>
            <Show when=move || mode == ProcessName::Direction fallback=|| ()>
                {method_select("主限算法", direction_method, &DIRECTION_METHODS)}
                {method_select("换算方式", arc_to_date_method, &ARC_TO_DATE_METHODS)}
            </Show>
            <Show when=move || mode == ProcessName::DailyDirection fallback=|| ()>
                {method_select("方向弧算法", daily_direction_method, &DAILY_DIRECTION_METHODS)}
            </Show>
            <div class=form::field>
                <label>"开始日期"</label>
                <ChartTimeEditor date=start/>
            </div>
            <div class=form::field>
                <label>"结束日期"</label>
                <ChartTimeEditor date=end/>
            </div>
            <div class=style::btn_row>
                <button class=style::btn_primary on:click=update disabled=move || loading.get()>
                    "更新"
                </button>
            </div>
        </div>

        // —— 筛选区：显著星 / 弧方向 / 相位类型 / 承诺星行星 / 重置 ——
        <div class=style::panel>
            <div class=form::field>
                <label>"Significator"</label>
                <div class=style::chips>{planet_chips(selected_significators)}</div>
            </div>
            <div class=form::field>
                <label>"弧方向"</label>
                <div class=style::segment>
                    <button
                        class=move || {
                            if arc_filter.get() == ArcDirectionFilter::All {
                                style::segment_btn_active
                            } else {
                                style::segment_btn
                            }
                        }
                        on:click=move |_| arc_filter.set(ArcDirectionFilter::All)
                    >
                        "全部"
                    </button>
                    <button
                        class=move || {
                            if arc_filter.get() == ArcDirectionFilter::Direct {
                                style::segment_btn_active
                            } else {
                                style::segment_btn
                            }
                        }
                        on:click=move |_| arc_filter.set(ArcDirectionFilter::Direct)
                    >
                        "正向弧"
                    </button>
                    <button
                        class=move || {
                            if arc_filter.get() == ArcDirectionFilter::Converse {
                                style::segment_btn_active
                            } else {
                                style::segment_btn
                            }
                        }
                        on:click=move |_| arc_filter.set(ArcDirectionFilter::Converse)
                    >
                        "反向弧"
                    </button>
                </div>
            </div>
            <div class=form::field>
                <label>"相位类型"</label>
                <div class=style::chips>{type_chips}</div>
            </div>
            <div class=form::field>
                <label>"Promittor"</label>
                <div class=style::chips>{planet_chips(promittor_planets)}</div>
            </div>
            <div class=style::btn_row>
                <button class=style::btn on:click=reset disabled=move || loading.get()>"重置"</button>
            </div>
        </div>

        // 每日回归模式：返照时刻横幅
        <Show
            when=move || mode == ProcessName::DailyDirection && return_time.get().is_some()
            fallback=|| ()
        >
            <div class=style::return_banner>
                {move || {
                    format!(
                        "每日回归时间：{}",
                        return_time.get().map(|d| format_date(&d)).unwrap_or_default()
                    )
                }}
            </div>
        </Show>

        // —— 结果表 ——
        <div class=style::table_card>
            // 变更后的重算提示：保留旧表格、不打断浏览
            <Show when=move || loading.get() && !data.get().is_empty() fallback=|| ()>
                <div class=feedback::loading>"更新中…"</div>
            </Show>
            <div class=style::table_scroll>
                <table class=style::table>
                    <thead>
                        <tr>
                            <th>"日期"</th>
                            <th>"Significator"</th>
                            <th>"Promittor"</th>
                            <th>"方向弧"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = filtered.get();
                            if rows.is_empty() {
                                let text = if loading.get() && data.get().is_empty() {
                                    "推运计算中…"
                                } else {
                                    "无匹配的方向记录"
                                };
                                view! {
                                    <tr>
                                        <td class=style::empty colspan="4">{text}</td>
                                    </tr>
                                }
                                .into_any()
                            } else {
                                rows.iter()
                                    .map(|item| {
                                        view! {
                                            <tr>
                                                <td class=style::nowrap>{format_date(&item.date)}</td>
                                                <td>{significator_cell(&item.significator)}</td>
                                                <td>{promittor_cell(&item.promittor)}</td>
                                                <td class=style::nowrap>{format_arc(item.arc)}</td>
                                            </tr>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .into_any()
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </div>

        // 请求失败以对话框呈现（对应原版 alertController 弹窗）
        <AlertDialog header="错误" message=alert_msg/>
    }
}
