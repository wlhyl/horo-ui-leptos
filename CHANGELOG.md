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
