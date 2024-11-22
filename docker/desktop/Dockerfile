# 基础镜像
FROM ubuntu:22.04

# 设置环境变量以避免交互式提示
ENV DEBIAN_FRONTEND=noninteractive
ENV DEBIAN_PRIORITY=high

# 设置语言环境
ENV LANG=en_US.UTF-8
ENV LANGUAGE=en_US:en
ENV LC_ALL=en_US.UTF-8

# 设置应用信息
ENV APP_NAME="ConsoleY"
ENV APP_VERSION="1.0.0"

# 设置用户信息
ENV USERNAME=consoley
ENV HOME=/home/$USERNAME

# 设置显示参数
ENV DISPLAY_NUM=1
ENV HEIGHT=768
ENV WIDTH=1024
ENV DISPLAY=:$DISPLAY_NUM

# 第一阶段：安装基础系统包
RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install --no-install-recommends \
    # 系统工具
    software-properties-common \
    gpg-agent \
    sudo \
    curl \
    wget \
    git \
    ca-certificates \
    locales \
    # UI 要求
    xvfb \
    xterm \
    xdotool \
    scrot \
    imagemagick \
    mutter \
    x11vnc \
    # 开发工具
    build-essential \
    pkg-config \
    libssl-dev \
    # 网络工具
    net-tools \
    netcat \
    && apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# 第二阶段：安装桌面环境
RUN add-apt-repository ppa:mozillateam/ppa && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    firefox-esr \
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

# 第三阶段：设置noVNC
RUN git clone --branch v1.5.0 https://github.com/novnc/noVNC.git /opt/noVNC && \
    git clone --branch v0.12.0 https://github.com/novnc/websockify /opt/noVNC/utils/websockify && \
    ln -s /opt/noVNC/vnc.html /opt/noVNC/index.html

# 第四阶段：设置用户和权限
RUN useradd -m -s /bin/bash -d $HOME $USERNAME && \
    echo "${USERNAME} ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers && \
    # 设置语言环境
    sed -i 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen && \
    locale-gen && \
    # 创建必要的目录
    mkdir -p $HOME/.vnc && \
    mkdir -p $HOME/.config && \
    mkdir -p $HOME/Desktop $HOME/Documents $HOME/Downloads && \
    mkdir -p $HOME/.config/pcmanfm/default && \
    mkdir -p /tmp/.X11-unix && \
    mkdir -p /var/log/supervisor && \
    # 设置VNC密码
    echo "consoley" | x11vnc -storepasswd -passfile $HOME/.vnc/passwd && \
    # 设置权限
    chmod 600 $HOME/.vnc/passwd && \
    chmod 1777 /tmp/.X11-unix && \
    chown -R $USERNAME:$USERNAME $HOME && \
    chown -R $USERNAME:$USERNAME /var/log/supervisor

# 第五阶段：安装Rust和构建API服务器
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sudo -u $USERNAME sh -s -- -y && \
    echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

# 复制配置文件
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf
COPY startup.sh /startup.sh
COPY --chown=$USERNAME:$USERNAME desktop-config/ $HOME/.config/
COPY --chown=$USERNAME:$USERNAME image/ $HOME/

# 安装 dos2unix 用于转换脚本文件的换行符
RUN sed -i 's/archive.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list && \
    sed -i 's/security.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y dos2unix && \
    dos2unix /startup.sh

# 设置API服务器构建
WORKDIR /build
COPY --chown=$USERNAME:$USERNAME api-server/Cargo.toml ./
COPY --chown=$USERNAME:$USERNAME api-server/src ./src/

# 构建API服务器
RUN chown -R $USERNAME:$USERNAME /build && \
    sudo -u $USERNAME bash -c 'source $HOME/.cargo/env && cargo build --release' && \
    cp target/release/api-server /usr/local/bin/ && \
    chmod +x /usr/local/bin/api-server && \
    rm -rf /build

# 设置权限
RUN chmod +x /startup.sh && \
    chown $USERNAME:$USERNAME /startup.sh

# 切换到非root用户
USER $USERNAME
WORKDIR $HOME

# 暴露端口
EXPOSE 5900 6080

# 启动命令
CMD ["/startup.sh"]