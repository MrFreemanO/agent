#!/bin/bash

# Create and set permissions using sudo
sudo mkdir -p /var/log/supervisor
sudo chown -R consoley:consoley /var/log/supervisor

# Setup desktop environment
mkdir -p $HOME/Desktop $HOME/Documents $HOME/Downloads
mkdir -p $HOME/.config/tint2
mkdir -p $HOME/.config/pcmanfm/default
mkdir -p $HOME/.config/mutter
mkdir -p $HOME/.local/share/mutter

# Ensure tint2 configuration directory exists
if [ ! -f "$HOME/.config/tint2/tint2rc" ]; then
    mkdir -p "$HOME/.config/tint2"
    cp /home/consoley/.config/tint2/tint2rc "$HOME/.config/tint2/tint2rc"
fi

# Setup mutter configuration (modify this section)
sudo rm -rf $HOME/.config/mutter/dconf  # First remove existing file
sudo mkdir -p $HOME/.config/mutter
sudo chown -R consoley:consoley $HOME/.config/mutter
sudo chmod -R 755 $HOME/.config/mutter

# Now create configuration as consoley user
sudo -u consoley bash -c 'mkdir -p $HOME/.config/mutter && cat > $HOME/.config/mutter/dconf << EOF
[/]
dynamic-workspaces=false
workspaces-only-on-primary=true
EOF'

# Ensure mutter configuration directory exists and set permissions
mkdir -p $HOME/.config/mutter
sudo chown -R consoley:consoley $HOME/.config/mutter

# Create and set permissions for mutter/dconf directory
sudo mkdir -p /home/consoley/.config/mutter/dconf
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
wallpaper_color=#000000
show_documents=1
show_trash=1
show_mounts=1
EOF

# Ensure mutter configuration directory exists and set permissions
mkdir -p $HOME/.config/mutter
sudo chown -R consoley:consoley $HOME/.config/mutter

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
sudo -E supervisord -n -c /etc/supervisor/conf.d/supervisord.conf

# Set necessary environment variables
export XDG_CURRENT_DESKTOP=GNOME
export XDG_DATA_DIRS="/usr/share:/usr/local/share:$HOME/.local/share"