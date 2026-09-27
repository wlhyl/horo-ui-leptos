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
