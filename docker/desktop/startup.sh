#!/bin/bash

# Set up the environment
export DISPLAY=:1
export HOME=/home/consoley
export USER=consoley

# Create necessary directories
mkdir -p /var/log/supervisor
mkdir -p /var/run
mkdir -p $HOME/.config/pcmanfm/default
mkdir -p /app/api-server/target

if [ -d "/home/consoley/.cargo" ]; then
    chmod -R 755 /home/consoley/.cargo
    chown -R consoley:consoley /home/consoley/.cargo
fi

if [ -d "/app/api-server/target" ]; then
    chown -R consoley:consoley /app/api-server/target
    chmod -R 755 /app/api-server/target
fi

# Set up desktop background
cat > $HOME/.config/pcmanfm/default/desktop-items-0.conf << EOF
[*]
wallpaper_mode=crop
wallpaper=/usr/share/backgrounds/christmas.png
desktop_bg=#000000
desktop_shadow=#000000
desktop_font=Sans 12
folder=
show_wm_menu=0
sort=mtime;ascending;
show_documents=0
show_trash=0
show_mounts=0
EOF

# Set up the correct permissions
chown -R consoley:consoley $HOME/.config
chmod -R 755 $HOME/.config
chmod 644 $HOME/.config/pcmanfm/default/desktop-items-0.conf

# Start supervisor
exec /usr/bin/supervisord -n -c /etc/supervisor/conf.d/supervisord.conf
