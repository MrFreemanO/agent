#!/bin/bash

# 构建 API 服务器
cargo build --release

# 复制二进制文件到 docker 构建目录
cp target/release/api-server docker/desktop/

# 构建 docker 镜像
cd docker/desktop
docker build -t consoley-desktop . 