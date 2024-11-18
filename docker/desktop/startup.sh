#!/bin/bash

# 使用临时目录存储配置
SUPERVISOR_CONF_FILE=/tmp/supervisord.conf

# 创建supervisord配置文件
cat > "$SUPERVISOR_CONF_FILE" << EOF
[supervisord]
nodaemon=true
logfile=/var/log/supervisor/supervisord.log
pidfile=/var/run/supervisord.pid

[program:xvfb]
command=/usr/bin/Xvfb :1 -screen 0 1024x768x24
autorestart=true
stdout_logfile=/var/log/supervisor/xvfb.log
stderr_logfile=/var/log/supervisor/xvfb.err

[program:mutter]
command=/usr/bin/mutter --sm-disable --replace --x11
environment=DISPLAY=:1
autorestart=true
stdout_logfile=/var/log/supervisor/mutter.log
stderr_logfile=/var/log/supervisor/mutter.err

[program:x11vnc]
command=/usr/bin/x11vnc -display :1 -no6 -forever -shared -wait 50 -rfbport 5900 -nopw 

autorestart=true
stdout_logfile=/var/log/supervisor/x11vnc.log
stderr_logfile=/var/log/supervisor/x11vnc.err

[program:novnc]
command=/opt/noVNC/utils/novnc_proxy --vnc localhost:5900 --listen 6080
autorestart=true
stdout_logfile=/var/log/supervisor/novnc.log
stderr_logfile=/var/log/supervisor/novnc.err

[program:tint2]
command=/usr/bin/tint2
environment=DISPLAY=:1
autorestart=true
stdout_logfile=/var/log/supervisor/tint2.log
stderr_logfile=/var/log/supervisor/tint2.err

[program:pcmanfm]
command=/usr/bin/pcmanfm --desktop
environment=DISPLAY=:1
autorestart=true
stdout_logfile=/var/log/supervisor/pcmanfm.log
stderr_logfile=/var/log/supervisor/pcmanfm.err

[program:api-server]
command=/usr/local/bin/api-server
user=consoley
environment=DISPLAY=":%(ENV_DISPLAY_NUM)s",HOME="/home/consoley"
autostart=true
autorestart=true
stdout_logfile=/var/log/supervisor/api-server.log
stderr_logfile=/var/log/supervisor/api-server.err
EOF

# 创建日志目录
mkdir -p /var/log/supervisor
chown -R consoley:consoley /var/log/supervisor

# 设置桌面环境
mkdir -p $HOME/Desktop $HOME/Documents $HOME/Downloads

# 创建桌面快捷方式
# ... (保持原有的桌面快捷方式创建代码)

# 启动supervisord
exec supervisord -n -c "$SUPERVISOR_CONF_FILE"
