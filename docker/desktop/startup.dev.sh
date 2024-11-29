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

# 设置 mutter 配置
cat > $HOME/.config/mutter/dconf << EOF
[/]
dynamic-workspaces=false
workspaces-only-on-primary=true
EOF

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

# 创建桌面图标
echo "Creating desktop shortcuts..."
mkdir -p $HOME/Desktop
sudo chown -R consoley:consoley $HOME/Desktop

# 处理 Firefox 快捷方式
if [ ! -f "/usr/share/applications/firefox.desktop" ]; then
    echo "Firefox desktop file not found in default location."
    firefox_file=$(find /usr/share/applications -name "*firefox*.desktop" | head -n 1)
    if [ -n "$firefox_file" ]; then
        echo "Found alternative Firefox desktop file: $firefox_file"
        cp "$firefox_file" $HOME/Desktop/firefox.desktop
        echo "Copied Firefox desktop file. Status: $?"
    else
        echo "Creating a generic Firefox desktop file."
        cat > $HOME/Desktop/firefox.desktop << EOF
[Desktop Entry]
Version=1.0
Name=Firefox Web Browser
Exec=firefox-esr %u
Terminal=false
Type=Application
Icon=firefox-esr
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;
StartupNotify=true
EOF
        echo "Created generic Firefox desktop file. Status: $?"
    fi
else
    cp /usr/share/applications/firefox.desktop $HOME/Desktop/
    echo "Copied default Firefox desktop file. Status: $?"
fi

# 处理终端快捷方式
if [ ! -f "/usr/share/applications/terminal.desktop" ]; then
    echo "Terminal desktop file not found. Creating a generic one."
    cat > $HOME/Desktop/terminal.desktop << EOF
[Desktop Entry]
Version=1.0
Name=Terminal
Exec=x-terminal-emulator
Terminal=false
Type=Application
Icon=terminal
Categories=System;TerminalEmulator;
StartupNotify=true
EOF
    echo "Created generic terminal desktop file. Status: $?"
else
    cp /usr/share/applications/terminal.desktop $HOME/Desktop/
    echo "Copied terminal desktop file. Status: $?"
fi

# 确保文件存在并设置权限
if [ -d "$HOME/Desktop" ]; then
    chmod +x $HOME/Desktop/*.desktop 2>/dev/null
    sudo chown -R consoley:consoley $HOME/Desktop
    echo "Contents of Desktop folder:"
    ls -la $HOME/Desktop
    
    # 验证文件内容
    echo "Firefox desktop file contents:"
    cat $HOME/Desktop/firefox.desktop 2>/dev/null || echo "Firefox desktop file not found"
    echo "Terminal desktop file contents:"
    cat $HOME/Desktop/terminal.desktop 2>/dev/null || echo "Terminal desktop file not found"
    
    # 检查目录权限
    echo "Desktop directory permissions:"
    ls -ld $HOME/Desktop
fi

# 初始化 Firefox 配置
sudo -u consoley bash -c 'echo "Initializing Firefox profile..."'
# echo "Initializing Firefox profile..."
FIREFOX_PROFILE_DIR="$HOME/.mozilla/firefox"
mkdir -p "$FIREFOX_PROFILE_DIR"

# 如果配置文件不存在，创建一个新的配置文件
if [ ! -f "$FIREFOX_PROFILE_DIR/profiles.ini" ]; then
    echo "Creating new Firefox profile..."
    mkdir -p "$FIREFOX_PROFILE_DIR/default"
    
    # 创建 profiles.ini
    cat > "$FIREFOX_PROFILE_DIR/profiles.ini" << EOF
[Profile0]
Name=default
IsRelative=1
Path=default
Default=1

[General]
StartWithLastProfile=1
Version=2
EOF
    
    # 创建基本配置文件¯
    cat > "$FIREFOX_PROFILE_DIR/default/prefs.js" << EOF
user_pref("browser.startup.homepage", "about:blank");
user_pref("browser.shell.checkDefaultBrowser", false);
user_pref("browser.tabs.warnOnClose", false);
EOF
fi

# 设置正确的权限
sudo chown -R consoley:consoley "$HOME/.mozilla"
chmod -R 700 "$HOME/.mozilla"

# 以 root 用户启动 supervisord
sudo -E supervisord -n -c /etc/supervisor/conf.d/supervisord.conf

# 设置必要的环境变量
export XDG_CURRENT_DESKTOP=GNOME
export XDG_DATA_DIRS="/usr/share:/usr/local/share:$HOME/.local/share"