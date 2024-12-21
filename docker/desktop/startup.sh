#!/bin/bash

# Use temporary directory for configuration
SUPERVISOR_CONF_FILE=/tmp/supervisord.conf

# Create supervisord configuration file
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

# Create log directory
mkdir -p /var/log/supervisor
chown -R consoley:consoley /var/log/supervisor

# Setup desktop environment
mkdir -p $HOME/Desktop $HOME/Documents $HOME/Downloads

# Start supervisord
exec supervisord -n -c "$SUPERVISOR_CONF_FILE"
