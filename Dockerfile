# syntax=docker/dockerfile:1

# ---------- 构建阶段：用 trunk 把 Leptos(WASM) 前端编译成静态资源 ----------
FROM rust:1.99.0-alpine AS builder

# 基础编译/依赖工具；gcompat 让 trunk 下载的 glibc 预编译二进制(wasm-bindgen/wasm-opt)可在 musl 上运行
# RUN apk add --no-cache \
#         build-base pkgconf openssl-dev ca-certificates curl git gcompat

# 添加 wasm 目标并安装 trunk（release 构建工具链）
# stylance-cli 用于把 src/**/*.module.css 编译成带哈希作用域类名的样式包
RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk --locked --version 0.21.14 \
    && cargo install stylance-cli --locked --version 0.8.4

WORKDIR /app
COPY . .

# 镜像内前端使用同源相对路径 /api，由 ingress 路由到后台，本镜像不做反向代理。
# 如需直连后台（走 CORS），可改为：ENV API_BASE_URL=https://api.example.com
ENV API_BASE_URL=
# 登录后台（horo-storage）同理：同源 /api/horo-admin 由 ingress 转发
ENV ADMIN_API_BASE_URL=
RUN trunk build --release

# 预压缩产物（-k 保留原文件）；woff2/图片等本身已压缩的格式跳过，避免白白增大镜像
# 运行阶段由 nginx gzip_static 直接下发 .gz，省去每次请求的实时压缩 CPU 开销
RUN find dist -type f \( -name '*.html' -o -name '*.js' -o -name '*.css' -o -name '*.map' \
        -o -name '*.wasm' -o -name '*.json' -o -name '*.svg' -o -name '*.txt' \) \
        -exec gzip -9 -k {} +

# ---------- 运行阶段：nginx 仅托管静态资源 ----------
FROM nginx:1.31-alpine AS runtime

COPY docker/nginx.conf /etc/nginx/nginx.conf

COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
