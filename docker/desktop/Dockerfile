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
ENV APP_VERSION="1.0.0"

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
    xterm \
    xdotool \
    scrot \
    imagemagick \
    mutter \
    x11vnc \
    # Development tools
    build-essential \
    pkg-config \
    libssl-dev \
    # Network tools
    net-tools \
    netcat \
    && apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Stage 2: Install desktop environment
RUN add-apt-repository ppa:mozillateam/ppa && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    firefox-esr \
    libreoffice \
    xpaint \
    galculator \
    xpdf \
    x11-apps \
    gedit \
    pcmanfm \
    tint2 \
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
    mkdir -p $HOME/.config/pcmanfm/default && \
    mkdir -p /tmp/.X11-unix && \
    mkdir -p /var/log/supervisor && \
    # Set VNC password
    echo "consoley" | x11vnc -storepasswd -passfile $HOME/.vnc/passwd && \
    # Set permissions
    chmod 600 $HOME/.vnc/passwd && \
    chmod 1777 /tmp/.X11-unix && \
    chown -R $USERNAME:$USERNAME $HOME && \
    chown -R $USERNAME:$USERNAME /var/log/supervisor

# Stage 5: Install Rust and build API server
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sudo -u $USERNAME sh -s -- -y && \
    echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

# Copy configuration files
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf
COPY startup.sh /startup.sh
COPY --chown=$USERNAME:$USERNAME desktop-config/ $HOME/.config/
COPY --chown=$USERNAME:$USERNAME image/ $HOME/

# Install dos2unix for converting script file line endings
RUN sed -i 's/archive.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list && \
    sed -i 's/security.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y dos2unix && \
    dos2unix /startup.sh

# Setup API server build
WORKDIR /build
COPY --chown=$USERNAME:$USERNAME api-server/Cargo.toml ./
COPY --chown=$USERNAME:$USERNAME api-server/src ./src/

# Build API server
RUN chown -R $USERNAME:$USERNAME /build && \
    sudo -u $USERNAME bash -c 'source $HOME/.cargo/env && cargo build --release' && \
    cp target/release/api-server /usr/local/bin/ && \
    chmod +x /usr/local/bin/api-server && \
    rm -rf /build

# Set permissions
RUN chmod +x /startup.sh && \
    chown $USERNAME:$USERNAME /startup.sh

# Switch to non-root user
USER $USERNAME
WORKDIR $HOME

# Expose ports
EXPOSE 5900 6080 8080

# Startup command
CMD ["/startup.sh"]