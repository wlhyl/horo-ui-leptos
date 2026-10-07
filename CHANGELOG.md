# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- 重新实现本命星盘与天象盘
- 用户登录功能：`/user` 登录页（登录/注销、错误提示、防重复提交），对接 horo-storage-api 的 `/api/horo-admin/login`
- 登录态管理：JWT 解析与过期校验，token 持久化于 `localStorage['token']`（与原版 horo-ui 互通）
- 顶栏用户入口：未登录显示「登录」，已登录显示用户名，点击进入用户页
- 新增 `ADMIN_API_BASE_URL` 构建配置（Docker 内同源由 ingress 转发）
- 地名搜索经纬度：输入页地理区域新增地名输入与搜索（回车/按钮触发），对接 horo-storage-api 的 `/api/horo-admin/location_search`（`token` header 鉴权，中文地名 URL 编码），结果列表点选后回填地名与经纬度；`GeoInput` 组件目录化，经纬度输入改用 `prop:value` 修复程序化回写不刷新的问题
- 从档案库选择天宫图：本命盘与天象盘输入页新增「从档案库选择」入口，弹出选择对话框，对接 horo-storage-api 的 `/api/horo-admin/horoscopes`（分页列表）与 `/horoscopes/search`（姓名模糊搜索）；支持 300ms 防抖、加载更多分页、未登录提示；点选记录回填姓名/性别/出生日期时间/时区/夏令时/地点经纬度（卜卦盘显示 H 徽标，时间精准姓名高亮，宫位制保持表单当前值）
- 清除缓存页：`/clean` 页面（标题/说明/清除按钮/完成提示），点击调用 `HoroStorage::clean()` 清空 localStorage 七份星盘缓存并恢复默认值；首页新增「清除缓存」次级入口
- 行星力量表页：`/power` 页面，列出 12 黄道星座的先天尊贵主星（庙/旺/三分主星/界/面/陷/落）；支持「本命所用」（标准三分 + 埃及界）与「Lily所用」（Lily 三分 + 托勒密界）两种体系切换；符号以 Unicode glyph 渲染并着色；首页新增「行星力量表」入口卡片。尊贵数据补充标准三分主星与埃及界两张静态表（见 `src/astro/dignity.rs`）
- 多窗口工作台页：`/workbench` 页面（全幅布局），左侧输入面板编辑出生数据/天象数据并实时写回 `HoroStorage`，右侧工作区可添加多个本命盘与天象盘窗口，支持拖拽移动、八向缩放、最大化/最小化/隐藏/关闭与层级聚焦；窗口打开时快照数据请求星盘，加载与错误态由 `ChartWheel` 渲染（`src/workbench/`，状态管理见 `window.rs`）。首页新增「工作台」入口卡片并置于命盘之前
- 星盘时间编辑重绘：星盘结果页与工作台窗口新增 `ChartTimeEditor` 时间编辑条（日期时间滚轮选择、年/月/日/时/分/秒档位步进、「现在」按钮），改动只写页面本地日期信号，300ms 防抖后重新请求并重绘，全程不落盘（不写回 `HoroStorage`/面板/localStorage）
- 衍生盘：以出生数据为基准、基准行星的斜升（OA）为中天的旋转盘。`/derived` 输入页（表单含「基准行星」下拉，七传统星，符号+中文名选项）与 `/derived/chart` 结果页（摘要行显示基准行星，时间编辑防抖重绘同样不落盘），对接 horo-api 的 `/api/horo/derived`；衍生盘响应不含日主星/时主星，`Horoscope` 对应字段改为 `Option` 并在盘面注释按缺省跳过。工作台输入面板新增「衍生盘数据」节（基准行星实时写回 `HoroStorage`，切换只影响新开窗口）与衍生盘开窗按钮，基准行星随窗口打开快照、窗口间独立，窗口标题带中文行星名（如「衍生盘·土星」）；首页新增「衍生盘」入口卡片
- 星盘存档：本命盘与天象盘结果页新增「存档」按钮（衍生盘不提供，对齐原版），对接 horo-storage-api 的 `POST /api/horo-admin/horoscopes`（新增）与 `PUT /api/horo-admin/horoscopes/{id}`（更新，diff 语义：null 字段后台不修改）；需登录，未登录点击弹「请先登录后再存档」；已存过档案（id ≠ 0）先弹「更新记录 / 新增记录 / 取消」三选；姓名/性别取输入快照，时间取时间编辑条当前值（页面内改时间后存档按改后时间保存），经纬度转度分秒（半球标志按正负）；本命盘存 natal、天象盘存 horary；成功后把档案 id 与当前时间回写 `HoroStorage` 并弹「存档成功」，后台校验（空姓名等）与锁定记录的错误文案原样透出；无笔记功能（按需求裁剪）。`AlertDialog` 扩展可选自定义按钮组以承载三选弹窗，存量单「确定」用法不受影响；提交中防重复点击
- 方向推运：主向推运 / 每日回归方向弧 / 太阳弧三种模式（对应原版 horo-ui 的 process.page + direction.component），计算在后端 horo-api，前端组装请求、过滤与渲染。`/process` 推运输入页（出生数据 + 推运类型 + 推运时间/居住地 + 按类型显示的主限算法/换算方式/方向弧算法/日返月亮，提交写回 `HoroStorage` 并按类型导航）；`/direction`、`/daily_direction`、`/solar_arc` 结果页共用 `DirectionView`（顶部说明、出生时间/宫位/算法/日期范围参数区（地名常显，经度/纬度默认收起，点「显示经纬度」展开，对应原版 showGeoInput 折叠；`GeoInput` 新增 `collapsible` prop 承载该行为，其他页面不受影响）、「更新」300ms 防抖 + 代数失效、Significator/弧方向/相位类型/Promittor 筛选区、「重置」、每日回归模式的返照时刻横幅、日期/Significator/Promittor/方向弧四列结果表，筛选为 Memo 五重条件：日期范围、显著星、弧方向、相位类型、承诺星行星；promittor 单元格按左相位符号→映点/反映点→宫头→星座→行星→右相位符号→界的顺序渲染）；每日回归方向弧按「日返月亮」开关走 日返→月返→每日回归 链式取返照时刻（每步 st 固定 false）再算方向弧。工作台新增「推运数据」折叠节（实时写回 `HoroStorage`）与主向推运/每日回归/太阳弧三个开窗按钮，窗口携带开窗时刻的出生 + 推运数据快照、窗口间独立，选中恰一个显著星时窗口标题附加该星（如「主向推运-MC」，对齐原版 titleChange）。API 契约层新增 `Direction/Significator/Promittor/ReturnHoroscope` 响应类型（untagged 承接后端单键对象形态）与 directions/solar_arc/daily_directions/return×3 六个请求封装；响应日期无 st 字段，`DateTimeData.st` 加 `#[serde(default)]`；三种推运方法枚举补中文 `Display`；结果页日期编辑复用星盘页 `ChartTimeEditor` 时间编辑条（点日期标签弹六列滚轮面板，确定才写回，步进器微调，「现在」一键填当前时间）。首页新增「推运」入口卡片（置于本命盘之后）
