#!/bin/bash

# Set up the environment
export DISPLAY=:1
export HOME=/home/consoley
export USER=consoley

# Create necessary directories and files
mkdir -p /var/log/supervisor
mkdir -p /var/run
mkdir -p $HOME/.config

# Set up the correct permissions
chown -R consoley:consoley $HOME
chmod -R 755 $HOME/.config

# Start supervisor
exec /usr/bin/supervisord -n -c /etc/supervisor/conf.d/supervisord.conf
