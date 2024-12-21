# Project Structure
```
./
├── docker/
│   ├── desktop/
│   │   ├── api-server/
│   │   │   ├── src/
│   │   │   │   ├── lib.rs*
│   │   │   │   └── main.rs*
│   │   │   ├── Cargo.lock*
│   │   │   └── Cargo.toml*
│   │   ├── desktop-config/
│   │   │   ├── dconf/
│   │   │   │   └── user
│   │   │   ├── enchant/
│   │   │   │   ├── en_US.dic
│   │   │   │   └── en_US.exc
│   │   │   ├── galculator/
│   │   │   │   └── galculator.conf
│   │   │   ├── gedit/
│   │   │   │   └── accels
│   │   │   ├── gtk-2.0/
│   │   │   │   └── gtkfilechooser.ini
│   │   │   ├── libreoffice/
│   │   │   │   └── 4/
│   │   │   │       └── user/
│   │   │   │           ├── autocorr/
│   │   │   │           ├── autotext/
│   │   │   │           │   └── mytexts.bau
│   │   │   │           ├── basic/
│   │   │   │           │   ├── Standard/
│   │   │   │           │   │   ├── Module1.xba
│   │   │   │           │   │   ├── dialog.xlb
│   │   │   │           │   │   └── script.xlb
│   │   │   │           │   ├── dialog.xlc
│   │   │   │           │   └── script.xlc
│   │   │   │           ├── config/
│   │   │   │           │   ├── soffice.cfg/
│   │   │   │           │   │   └── modules/
│   │   │   │           │   │       ├── scalc/
│   │   │   │           │   │       │   ├── images/
│   │   │   │           │   │       │   │   └── Bitmaps/
│   │   │   │           │   │       │   ├── menubar/
│   │   │   │           │   │       │   ├── popupmenu/
│   │   │   │           │   │       │   ├── statusbar/
│   │   │   │           │   │       │   └── toolbar/
│   │   │   │           │   │       └── swriter/
│   │   │   │           │   │           ├── images/
│   │   │   │           │   │           │   └── Bitmaps/
│   │   │   │           │   │           ├── menubar/
│   │   │   │           │   │           ├── popupmenu/
│   │   │   │           │   │           ├── statusbar/
│   │   │   │           │   │           └── toolbar/
│   │   │   │           │   └── autotbl.fmt
│   │   │   │           ├── database/
│   │   │   │           │   ├── biblio/
│   │   │   │           │   │   ├── biblio.dbf
│   │   │   │           │   │   └── biblio.dbt
│   │   │   │           │   └── biblio.odb
│   │   │   │           ├── extensions/
│   │   │   │           │   ├── bundled/
│   │   │   │           │   │   ├── registry/
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.bundle.PackageRegistryBackend/
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.component.PackageRegistryBackend/
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.configuration.PackageRegistryBackend/
│   │   │   │           │   │   │   │   └── backenddb.xml
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.executable.PackageRegistryBackend/
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.help.PackageRegistryBackend/
│   │   │   │           │   │   │   │   └── backenddb.xml
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.script.PackageRegistryBackend/
│   │   │   │           │   │   │   └── com.sun.star.comp.deployment.sfwk.PackageRegistryBackend/
│   │   │   │           │   │   └── lastsynchronized
│   │   │   │           │   ├── shared/
│   │   │   │           │   │   ├── registry/
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.bundle.PackageRegistryBackend/
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.component.PackageRegistryBackend/
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.configuration.PackageRegistryBackend/
│   │   │   │           │   │   │   │   └── backenddb.xml
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.executable.PackageRegistryBackend/
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.help.PackageRegistryBackend/
│   │   │   │           │   │   │   │   └── backenddb.xml
│   │   │   │           │   │   │   ├── com.sun.star.comp.deployment.script.PackageRegistryBackend/
│   │   │   │           │   │   │   └── com.sun.star.comp.deployment.sfwk.PackageRegistryBackend/
│   │   │   │           │   │   └── lastsynchronized
│   │   │   │           │   ├── tmp/
│   │   │   │           │   │   ├── extensions/
│   │   │   │           │   │   └── registry/
│   │   │   │           │   │       ├── com.sun.star.comp.deployment.bundle.PackageRegistryBackend/
│   │   │   │           │   │       ├── com.sun.star.comp.deployment.component.PackageRegistryBackend/
│   │   │   │           │   │       ├── com.sun.star.comp.deployment.configuration.PackageRegistryBackend/
│   │   │   │           │   │       │   └── backenddb.xml
│   │   │   │           │   │       ├── com.sun.star.comp.deployment.executable.PackageRegistryBackend/
│   │   │   │           │   │       ├── com.sun.star.comp.deployment.help.PackageRegistryBackend/
│   │   │   │           │   │       │   └── backenddb.xml
│   │   │   │           │   │       ├── com.sun.star.comp.deployment.script.PackageRegistryBackend/
│   │   │   │           │   │       └── com.sun.star.comp.deployment.sfwk.PackageRegistryBackend/
│   │   │   │           │   └── buildid
│   │   │   │           ├── gallery/
│   │   │   │           │   ├── sg30.sdv
│   │   │   │           │   └── sg30.thm
│   │   │   │           ├── pack/
│   │   │   │           │   ├── autotext/
│   │   │   │           │   │   └── mytexts.pack
│   │   │   │           │   ├── basic/
│   │   │   │           │   │   ├── Standard/
│   │   │   │           │   │   │   ├── Module1.pack
│   │   │   │           │   │   │   ├── dialog.pack
│   │   │   │           │   │   │   └── script.pack
│   │   │   │           │   │   ├── dialog.pack
│   │   │   │           │   │   └── script.pack
│   │   │   │           │   ├── config/
│   │   │   │           │   │   └── autotbl.pack
│   │   │   │           │   ├── database/
│   │   │   │           │   │   ├── biblio/
│   │   │   │           │   │   │   └── biblio.pack
│   │   │   │           │   │   └── biblio.pack
│   │   │   │           │   ├── ExtensionInfo.pack
│   │   │   │           │   └── registrymodifications.pack
│   │   │   │           ├── psprint/
│   │   │   │           ├── uno_packages/
│   │   │   │           │   └── cache/
│   │   │   │           │       ├── registry/
│   │   │   │           │       │   ├── com.sun.star.comp.deployment.bundle.PackageRegistryBackend/
│   │   │   │           │       │   ├── com.sun.star.comp.deployment.component.PackageRegistryBackend/
│   │   │   │           │       │   ├── com.sun.star.comp.deployment.configuration.PackageRegistryBackend/
│   │   │   │           │       │   │   └── backenddb.xml
│   │   │   │           │       │   ├── com.sun.star.comp.deployment.executable.PackageRegistryBackend/
│   │   │   │           │       │   ├── com.sun.star.comp.deployment.help.PackageRegistryBackend/
│   │   │   │           │       │   │   └── backenddb.xml
│   │   │   │           │       │   ├── com.sun.star.comp.deployment.script.PackageRegistryBackend/
│   │   │   │           │       │   └── com.sun.star.comp.deployment.sfwk.PackageRegistryBackend/
│   │   │   │           │       └── uno_packages/
│   │   │   │           ├── GraphicsRenderTests.log
│   │   │   │           └── registrymodifications.xcu
│   │   │   ├── mutter/
│   │   │   │   └── dconf*
│   │   │   ├── pcmanfm/
│   │   │   │   └── default/
│   │   │   │       └── desktop-items-0.conf*
│   │   │   ├── tint2/
│   │   │   ├── xfce4/
│   │   │   │   └── terminal/
│   │   │   │       └── accels.scm
│   │   │   └── mimeapps.list
│   │   ├── image/
│   │   │   ├── .cargo/
│   │   │   │   └── env*
│   │   │   ├── .config/
│   │   │   │   ├── applications/
│   │   │   │   │   ├── firefox-custom.desktop*
│   │   │   │   │   ├── gedit.desktop*
│   │   │   │   │   └── terminal.desktop
│   │   │   │   └── tint2/
│   │   │   │       └── tint2rc
│   │   │   ├── Desktop/
│   │   │   ├── Documents/
│   │   │   ├── Downloads/
│   │   │   └── .bash_history
│   │   ├── Dockerfile
│   │   ├── startup.sh
│   │   └── supervisord.conf
│   └── .DS_Store
├── public/
│   ├── favicon-16x16.png
│   ├── favicon-32x32.png
│   ├── favicon.ico
│   ├── tauri.svg
│   └── vite.svg
├── release/
│   └── 1.0.0/
├── scripts/
│   ├── generate_tree.sh*
│   └── project_structure.md
├── src/
│   ├── assets/
│   │   └── react.svg
│   ├── App.css
│   ├── App.tsx
│   ├── main.tsx
│   └── vite-env.d.ts
├── tests/
│   ├── api_tests.rs
│   └── bash_tests.rs
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── docker-compose.yml
├── index.html
├── package-lock.json
├── package.json
├── tsconfig.json
├── tsconfig.node.json
└── vite.config.ts

107 directories, 86 files
```

## Directory Structure Explanation

- `docker/`: Docker 相关配置和构建文件
  - `desktop/`: 桌面环境相关配置
    - `api-server/`: Rust API 服务器代码
      - `src/`: 源代码目录
        - `lib.rs`: API 实现
        - `main.rs`: 程序入口
      - `Cargo.toml`: Rust 项目配置
    - `supervisord/`: 进程管理配置
      - `supervisord.conf`: Supervisor 配置文件
    - `x11/`: X11 显示服务配置
      - `xorg.conf`: X11 配置文件
    - `Dockerfile`: 桌面环境镜像构建文件
- `scripts/`: 项目相关脚本
  - `generate_tree.sh`: 生成项目目录树脚本
- `tests/`: 测试文件
  - `api_tests.rs`: API 测试用例
- `docs/`: 项目文档
  - `api.md`: API 文档
  - `deployment.md`: 部署文档
- `docker-compose.yml`: 容器编排配置
- `README.md`: 项目说明文档

# Project Structure
```
./
├── docker/                          # Docker 相关配置目录
│   ├── desktop/                     # 桌面环境配置
│   │   ├── api-server/             # Rust API 服务器
│   │   │   ├── src/               # Rust 源代码
│   │   │   │   ├── lib.rs*        # API 实现，包含所有接口逻辑
│   │   │   │   └── main.rs*       # 服务器入口文件
│   │   │   ├── Cargo.lock*        # Rust 依赖版本锁定文件
│   │   │   └── Cargo.toml*        # Rust 项目配置和依赖管理
│   │   ├── desktop-config/         # 桌面环境配置文件
│   │   │   ├── dconf/             # GNOME 配置数据库
│   │   │   ├── enchant/           # 拼写检查配置
│   │   │   ├── galculator/        # 计算器配置
│   │   │   ├── gedit/             # 文本编辑器配置
│   │   │   ├── gtk-2.0/          # GTK2 界面配置
│   │   │   ├── libreoffice/      # LibreOffice 配置
│   │   │   ├── mutter/           # 窗口管理器配置
│   │   │   ├── pcmanfm/          # 文件管理器配置
│   │   │   ├── tint2/            # 任务栏配置
│   │   │   └── xfce4/            # XFCE4 终端配置
│   │   ├── image/                 # 容器镜像相关文件
│   │   ├── Dockerfile            # 桌面环境容器构建文件
│   │   ├── startup.sh            # 容器启动脚本
│   │   └── supervisord.conf      # 进程管理配置文件
├── public/                        # 前端静态资源
├── release/                       # 发布版本目录
├── scripts/                       # 项目脚本
│   ├── generate_tree.sh*         # 生成项目结构树脚本
│   └── project_structure.md      # 生成的项目结构文档
├── src/                          # 前端源代码
│   ├── assets/                   # 前端资源文件
│   ├── App.css                   # 主应用样式
│   ├── App.tsx                   # 主应用组件
│   ├── main.tsx                  # 前端入口文件
│   └── vite-env.d.ts            # TypeScript 环境声明
├── tests/                        # 测试文件目录
│   ├── api_tests.rs             # API 接口测试
│   └── bash_tests.rs            # Shell 命令测试
├── .gitignore                    # Git 忽略配置
├── Cargo.lock                    # Rust 工作空间依赖锁定
├── Cargo.toml                    # Rust 工作空间配置
├── LICENSE                       # 项目许可证
├── README.md                     # 项目说明文档
├── docker-compose.yml            # Docker 容器编排配置
├── index.html                    # 前端入口 HTML
├── package.json                  # Node.js 项目配置
├── tsconfig.json                 # TypeScript 配置
└── vite.config.ts               # Vite 构建工具配置
```

这个项目是一个完整的远程桌面环境系统，主要包含三个核心部分：

1. 后端 API 服务（Rust）：
```rust:docker/desktop/api-server/src/lib.rs
startLine: 1
endLine: 50
```

2. 桌面环境配置（Docker）：
```shell:docker/desktop/startup.sh
startLine: 36
endLine: 93
```

3. 前端界面（React + TypeScript）：
位于 `src/` 目录下的前端代码，使用 Vite 构建工具。

整个项目通过 Docker 进行容器化管理，使用 Supervisor 进行进程管理，实现了一个可远程控制的桌面环境系统。
