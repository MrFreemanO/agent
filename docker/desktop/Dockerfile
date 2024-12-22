# Base image
FROM ubuntu:22.04

# Set environment variables to avoid interactive prompts
ENV DEBIAN_FRONTEND=noninteractive
ENV DEBIAN_PRIORITY=high

# Set language environment
ENV LANG=en_US.UTF-8
ENV LANGUAGE=en_US:en
ENV LC_ALL=en_US.UTF-8

# Set application information
ENV APP_NAME="ConsoleY"
ENV APP_VERSION="0.0.1"

# Set user information
ENV USERNAME=consoley
ENV HOME=/home/$USERNAME

# Set display parameters
ENV DISPLAY_NUM=1
ENV HEIGHT=768
ENV WIDTH=1024
ENV DISPLAY=:$DISPLAY_NUM

# Stage 1: Install base system packages
RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install --no-install-recommends \
    # System tools
    software-properties-common \
    gpg-agent \
    sudo \
    curl \
    wget \
    git \
    ca-certificates \
    locales \
    # UI requirements
    xvfb \
    xdotool \
    scrot \
    imagemagick \
    mutter \
    x11vnc \
    xfce4-terminal \
    # Development tools
    build-essential \
    pkg-config \
    libssl-dev \
    # Network tools
    net-tools \
    netcat \
    libfm-gtk4 \
    gvfs \
    dconf-cli \
    gnome-settings-daemon \
    pcmanfm \
    unzip \
    && apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Stage 2: Install desktop environment
RUN add-apt-repository ppa:mozillateam/ppa && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    firefox-esr \
    x11-apps \
    libreoffice \
    xpaint \
    galculator \
    xpdf \
    gedit \
    tint2 \
    mutter \
    supervisor \
    xfce4-terminal \
    dbus-x11 \
    gtk2-engines \
    gtk2-engines-pixbuf \
    gtk2-engines-murrine \
    gnome-themes-standard \
    libcanberra-gtk-module \
    libcanberra-gtk3-module \
    fonts-liberation \
    && apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# After installing desktop environment
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gedit \
    gnome-menus && \
    update-desktop-database && \
    # Verify gedit.desktop location
    find /usr/share/applications -name "*gedit*.desktop" -ls && \
    # Create symlink if needed
    if [ -f "/usr/share/applications/org.gnome.gedit.desktop" ]; then \
        ln -sf /usr/share/applications/org.gnome.gedit.desktop /usr/share/applications/gedit.desktop; \
    fi

# Stage 3: Setup noVNC
RUN git clone --branch v1.5.0 https://github.com/novnc/noVNC.git /opt/noVNC && \
    git clone --branch v0.12.0 https://github.com/novnc/websockify /opt/noVNC/utils/websockify && \
    ln -s /opt/noVNC/vnc.html /opt/noVNC/index.html

# Stage 4: Setup user and permissions
RUN useradd -m -s /bin/bash -d $HOME $USERNAME && \
    echo "${USERNAME} ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers && \
    # Set language environment
    sed -i 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen && \
    locale-gen && \
    # Create necessary directories
    mkdir -p $HOME/.vnc && \
    mkdir -p $HOME/.config && \
    mkdir -p $HOME/Desktop $HOME/Documents $HOME/Downloads && \
    mkdir -p /tmp/.X11-unix && \
    mkdir -p /var/log/supervisor && \
    # Set VNC password
    echo "consoley" | x11vnc -storepasswd -passfile $HOME/.vnc/passwd && \
    # Set permissions
    chmod 600 $HOME/.vnc/passwd && \
    chmod 1777 /tmp/.X11-unix && \
    chown -R $USERNAME:$USERNAME $HOME && \
    chown -R $USERNAME:$USERNAME /var/log/supervisor

# Stage 5: Install Rust development environment
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sudo -u $USERNAME sh -s -- -y && \
    echo 'source $HOME/.cargo/env' >> $HOME/.bashrc && \
    sudo -u $USERNAME bash -c 'source $HOME/.cargo/env && cargo install cargo-watch'

# Create necessary directories and permissions
RUN mkdir -p /var/run/dbus && \
    mkdir -p /run/user/1000 && \
    chown -R consoley:consoley /run/user/1000 && \
    chmod 700 /run/user/1000

# Copy configuration files
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf
COPY startup.sh /startup.sh
COPY desktop-config/ /home/consoley/.config/
COPY image/.config/tint2/tint2rc /home/consoley/.config/tint2/

# Create working directory
WORKDIR /app/api-server

# Set volume mount points
VOLUME ["/app/api-server"]

# Set permissions
RUN chmod +x /startup.sh && \
    chown $USERNAME:$USERNAME /startup.sh

# Switch to non-root user
USER $USERNAME

# Expose ports
EXPOSE 5900 6080 8080

# Startup command
CMD ["/startup.sh"]

# Create system directories before user setup
USER root
RUN mkdir -p /usr/lib/pcmanfm && \
    mkdir -p /usr/lib/x86_64-linux-gnu/pcmanfm && \
    chmod 755 /usr/lib/pcmanfm && \
    chmod 755 /usr/lib/x86_64-linux-gnu/pcmanfm

# Create user directories and files
RUN mkdir -p /home/consoley/.config/pcmanfm/default && \
    mkdir -p /home/consoley/Templates && \
    mkdir -p /home/consoley/.local/share/pcmanfm && \
    touch /home/consoley/.xsession-errors && \
    chown consoley:consoley /home/consoley/.xsession-errors && \
    chmod 644 /home/consoley/.xsession-errors && \
    chown -R consoley:consoley /home/consoley/.config && \
    chown -R consoley:consoley /home/consoley/Templates && \
    chown -R consoley:consoley /home/consoley/.local

# Copy desktop configuration files
COPY desktop-config/pcmanfm/default/pcmanfm.conf /home/consoley/.config/pcmanfm/default/
COPY desktop-config/pcmanfm/default/desktop-items-0.conf /home/consoley/.config/pcmanfm/default/

# Set configuration file permissions
RUN chown consoley:consoley /home/consoley/.config/pcmanfm/default/pcmanfm.conf && \
    chmod 644 /home/consoley/.config/pcmanfm/default/pcmanfm.conf && \
    chown consoley:consoley /home/consoley/.config/pcmanfm/default/desktop-items-0.conf && \
    chmod 644 /home/consoley/.config/pcmanfm/default/desktop-items-0.conf

# Switch to non-root user
USER consoley

# Create necessary directory structure
RUN mkdir -p /home/consoley/.config/pcmanfm/default && \
    chown -R consoley:consoley /home/consoley/.config && \
    chmod -R 755 /home/consoley/.config

# Create wallpaper directory and copy wallpaper before switching user
USER root
RUN mkdir -p /usr/share/backgrounds
COPY wallpapers/christmas.png /usr/share/backgrounds/
RUN chown root:root /usr/share/backgrounds/christmas.png && \
    chmod 644 /usr/share/backgrounds/christmas.png

# Finally switch back to non-root user
USER consoley