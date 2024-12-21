#!/bin/bash

# Create and set permissions using sudo
sudo mkdir -p /var/log/supervisor
sudo chown -R consoley:consoley /var/log/supervisor

# Setup desktop environment
setup_desktop() {
    # 创建必要的目录
    mkdir -p $HOME/.config/pcmanfm/default
    mkdir -p $HOME/Templates
    mkdir -p $HOME/.local/share/pcmanfm
    
    # 配置 pcmanfm
    cat > $HOME/.config/pcmanfm/default/desktop-items-0.conf << EOF
[*]
wallpaper_mode=color
wallpaper_color=#cc0000
desktop_bg=#cc0000
desktop_fg=#ffffff
desktop_shadow=#000000
show_wm_menu=0
sort=mtime;ascending;
show_documents=1
show_trash=1
show_mounts=1
EOF

    # 设置正确的权限
    chown -R consoley:consoley $HOME/.config/pcmanfm
    chown -R consoley:consoley $HOME/Templates
    chown -R consoley:consoley $HOME/.local/share/pcmanfm
    
    # 创建 .Xauthority
    touch $HOME/.Xauthority
    chown consoley:consoley $HOME/.Xauthority
    
    # 重启 pcmanfm 使配置生效
    if pgrep pcmanfm > /dev/null; then
        pkill pcmanfm
    fi
}

# 等待 X server 就绪
wait_for_x() {
    for i in {1..30}; do
        if [ -e "/tmp/.X11-unix/X1" ]; then
            sleep 2
            return 0
        fi
        sleep 1
    done
    return 1
}

# 设置桌面环境
if wait_for_x; then
    setup_desktop
    
    # 以 consoley 用户身份启动 pcmanfm
    sudo -u consoley bash -c 'DISPLAY=:1 pcmanfm --desktop --profile default'
else
    echo "Failed to connect to X server"
fi

# Ensure tint2 configuration directory exists
if [ ! -f "$HOME/.config/tint2/tint2rc" ]; then
    mkdir -p "$HOME/.config/tint2"
    cp /home/consoley/.config/tint2/tint2rc "$HOME/.config/tint2/tint2rc"
fi

# Setup mutter configuration (集中mutter相关配置)
if [ ! -d "$HOME/.config/mutter" ]; then
    sudo mkdir -p $HOME/.config/mutter
    sudo chown -R consoley:consoley $HOME/.config/mutter
    sudo chmod -R 755 $HOME/.config/mutter

    # 只在目录不存在时创建配置
    sudo -u consoley bash -c 'cat > $HOME/.config/mutter/dconf << EOF
[/]
dynamic-workspaces=false
workspaces-only-on-primary=true
background-color="#cc0000"  # 圣诞红色
EOF'
fi

# Ensure mutter configuration directory exists and set permissions
mkdir -p $HOME/.config/mutter
sudo chown -R consoley:consoley $HOME/.config/mutter

# Create and set permissions for mutter/dconf directory
# sudo mkdir -p /home/consoley/.config/mutter/dconf
sudo chown -R consoley:consoley /home/consoley/.config/mutter
sudo chmod -R 755 /home/consoley/.config/mutter

# Ensure Rust environment is installed
if [ ! -d "/home/consoley/.cargo" ]; then
    sudo -u consoley bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
    sudo -u consoley bash -c 'source /home/consoley/.cargo/env'
fi

# Install cargo-watch (if not already installed)
if ! sudo -u consoley bash -c 'source /home/consoley/.cargo/env && which cargo-watch'; then
    sudo -u consoley bash -c 'source /home/consoley/.cargo/env && cargo install cargo-watch'
fi

# Set directory permissions
sudo chown -R consoley:consoley /app/api-server
sudo chown -R consoley:consoley /home/consoley/.cargo
sudo chown -R consoley:consoley $HOME/.config
sudo chown -R consoley:consoley $HOME/.local
sudo chown -R consoley:consoley $HOME/.config/pcmanfm
sudo chmod -R 755 $HOME/.config/pcmanfm

# Configure PCManFM desktop settings
mkdir -p $HOME/.config/pcmanfm/default
cat > $HOME/.config/pcmanfm/default/desktop-items-0.conf << EOF
[*]
wallpaper_mode=color
wallpaper_color=#cc3300
show_documents=1
show_trash=1
show_mounts=1
EOF

# 添加主配置文件
cat > $HOME/.config/pcmanfm/default/pcmanfm.conf << EOF
[config]
bm_open_method=0
desktop_bg=#cc3300
desktop_fg=#ffffff
desktop_shadow=#000000
desktop_font=Sans 12
show_wm_menu=0

[volume]
mount_on_startup=1
mount_removable=1
autorun=1

[desktop]
wallpaper_mode=color
wallpaper_color=#cc3300
show_documents=1
show_trash=1
show_mounts=1
EOF

# 确保配置文件权限正确
sudo chown -R consoley:consoley $HOME/.config/pcmanfm

# Ensure api-server directory exists and has correct permissions
sudo mkdir -p /app/api-server/target
sudo chown -R consoley:consoley /app/api-server
sudo chmod -R 755 /app/api-server

# Check if Cargo.toml exists
if [ ! -f "/app/api-server/Cargo.toml" ]; then
    echo "Cargo.toml does not exist, please ensure Cargo.toml is present in /app/api-server directory."
    exit 1
fi

# Ensure Cargo.lock exists
if [ ! -f "/app/api-server/Cargo.lock" ] && [ -f "/app/api-server/Cargo.toml" ]; then
    cd /app/api-server && sudo -u consoley cargo generate-lockfile
fi

# Pre-build dependencies
sudo -u consoley bash -c 'cd /app/api-server && source $HOME/.cargo/env && cargo check'
# Start supervisord as root
# sudo -E supervisord -n -c /etc/supervisor/conf.d/supervisord.conf
sudo -E supervisord -n -c /etc/supervisor/conf.d/supervisord.conf

# Set necessary environment variables
export XDG_CURRENT_DESKTOP=GNOME
export XDG_DATA_DIRS="/usr/share:/usr/local/share:$HOME/.local/share"
