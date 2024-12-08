#!/bin/bash

# 使用 sudo 创建和设置权限
sudo mkdir -p /var/log/supervisor
sudo chown -R consoley:consoley /var/log/supervisor

# 设置桌面环境
mkdir -p $HOME/Desktop $HOME/Documents $HOME/Downloads
mkdir -p $HOME/.config/tint2
mkdir -p $HOME/.config/pcmanfm/default
mkdir -p $HOME/.config/mutter
mkdir -p $HOME/.local/share/mutter

# 确保 tint2 配置目录存在
if [ ! -f "$HOME/.config/tint2/tint2rc" ]; then
    mkdir -p "$HOME/.config/tint2"
    cp /home/consoley/.config/tint2/tint2rc "$HOME/.config/tint2/tint2rc"
fi

# 设置 mutter 配置（以 consoley 用户身份执行）
mkdir -p $HOME/.config/mutter
sudo chown consoley:consoley $HOME/.config/mutter
sudo -u consoley bash -c 'cat > $HOME/.config/mutter/dconf << EOF
[/]
dynamic-workspaces=false
workspaces-only-on-primary=true
EOF'

# 确保 mutter 配置目录存在并设置权限
mkdir -p $HOME/.config/mutter
sudo chown -R consoley:consoley $HOME/.config/mutter

# 确保 Rust 环境安装完成
if [ ! -d "/home/consoley/.cargo" ]; then
    sudo -u consoley bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
    sudo -u consoley bash -c 'source /home/consoley/.cargo/env'
fi

# 安装 cargo-watch (如果尚未安装)
if ! sudo -u consoley bash -c 'source /home/consoley/.cargo/env && which cargo-watch'; then
    sudo -u consoley bash -c 'source /home/consoley/.cargo/env && cargo install cargo-watch'
fi

# 设置目录权限
sudo chown -R consoley:consoley /app/api-server
sudo chown -R consoley:consoley /home/consoley/.cargo
sudo chown -R consoley:consoley $HOME/.config
sudo chown -R consoley:consoley $HOME/.local
sudo chown -R consoley:consoley $HOME/.config/pcmanfm
sudo chmod -R 755 $HOME/.config/pcmanfm

# 配置 PCManFM 桌面设置
mkdir -p $HOME/.config/pcmanfm/default
cat > $HOME/.config/pcmanfm/default/desktop-items-0.conf << EOF
[*]
wallpaper_mode=color
wallpaper_color=#000000
show_documents=1
show_trash=1
show_mounts=1
EOF

# 确保 mutter 配置目录存在并设置权限
mkdir -p $HOME/.config/mutter
sudo chown -R consoley:consoley $HOME/.config/mutter

# 确保 api-server 目录存在并具有正确的权限
sudo mkdir -p /app/api-server/target
sudo chown -R consoley:consoley /app/api-server
sudo chmod -R 755 /app/api-server

# 检查 Cargo.toml 是否存在
if [ ! -f "/app/api-server/Cargo.toml" ]; then
    echo "Cargo.toml 不存在，请确保 /app/api-server 目录中包含 Cargo.toml 文件。"
    exit 1
fi

# 确保 Cargo.lock 存在
if [ ! -f "/app/api-server/Cargo.lock" ] && [ -f "/app/api-server/Cargo.toml" ]; then
    cd /app/api-server && sudo -u consoley cargo generate-lockfile
fi

# 预先构建依赖
sudo -u consoley bash -c 'cd /app/api-server && source $HOME/.cargo/env && cargo check'

# 以 root 用户启动 supervisord
sudo -E supervisord -n -c /etc/supervisor/conf.d/supervisord.conf

# 设置必要的环境变量
export XDG_CURRENT_DESKTOP=GNOME
export XDG_DATA_DIRS="/usr/share:/usr/local/share:$HOME/.local/share"