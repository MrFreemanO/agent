# ConsoleY

ConsoleY 是一个轻量级的跨平台桌面应用，提供了一个隔离的 GUI 环境，允许 AI 像人类一样操作计算机。受 Anthropic 的 Computer Use 项目启发，ConsoleY 提供了一个安全、可控的环境，支持通过 API 进行屏幕截图、鼠标点击等操作。

## ✨ 特性

- 🖥️ 跨平台支持 (Windows, macOS, Linux)
- 🔒 隔离的 Docker 容器环境
- 🖱️ 完整的 GUI 桌面环境
- 🛠️ RESTful API 接口
- 📦 预装常用应用 (Firefox, LibreOffice 等)
- 🎨 可自定义的桌面设置
- 🔄 实时屏幕同步
- 🎯 精确的鼠标和键盘控制

## 🚀 快速开始

### 用户安装
从 [Releases](https://github.com/yourusername/consoley/releases) 页面下载适合您系统的安装包。

#### 运行要求
- Docker Desktop
  - Windows: [下载 Docker Desktop](https://www.docker.com/products/docker-desktop)
  - macOS: [下载 Docker Desktop](https://www.docker.com/products/docker-desktop)
  - Linux: 使用包管理器安装 Docker Engine

首次运行时，应用会自动下载所需的 Docker 镜像。

### 开发环境要求
以下依赖仅开发者需要安装：
- [Docker](https://www.docker.com/get-started)
- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) (>= 14.0.0)

### 开发者安装

1. 克隆仓库

```bash
git clone https://github.com/EvalsOne/consoley.git
cd consoley
```

2. 构建 Docker 镜像

```bash
docker build -t consoleai/desktop:latest ./docker/desktop
```

3. 安装依赖

```bash
#安装前端依赖
npm install
#安装 Rust 依赖
cd src-tauri
cargo build
cd ..
```

3. 启动开发环境
```bash
npm run tauri dev
```

## 📡 API 接口

ConsoleY 提供以下 API 接口：
```
GET /computer # 使用计算机功能
POST /edit # 编辑文件
POST /bash # 执行 bash 命令
```

详细的 API 文档请参考 [API.md](docs/API.md)。

## 🛠️ 开发

### 项目结构

```
consoley/
├── src-tauri/ # Tauri 后端代码
│ ├── src/ # Rust 源代码
│ ├── build.rs # 构建脚本
│ └── tauri.conf.json # Tauri 配置
├── docker/ # Docker 相关文件
│ └── desktop/ # 桌面环境容器
│ ├── Dockerfile # 生产环境镜像
│ ├── Dockerfile.dev # 开发环境镜像
│ ├── startup.sh # 生产环境启动脚本
│ ├── startup.dev.sh # 开发环境启动脚本
│ └── supervisord.conf # 进程管理配置
├── src/ # 前端源代码
├── public/ # 静态资源
├── docker-compose.yml # 生产环境容器编排
└── docker-compose.dev.yml # 开发环境容器编排
```

### Docker 构建

#### 开发环境
```bash
# 构建开发环境镜像
docker-compose -f docker-compose.dev.yml build

# 构建开发环境镜像（无缓存）
docker-compose -f docker-compose.dev.yml build --no-cache

# 启动开发环境容器
docker-compose -f docker-compose.dev.yml up -d

# 停止开发环境容器
docker-compose -f docker-compose.dev.yml down

# 查看开发环境日志
docker-compose -f docker-compose.dev.yml logs -f

# 进入Docker桌面环境
docker-compose -f docker-compose.dev.yml exec consoley bash
```

#### 生产环境
待补充

### 前端构建
```bash
# 开发模式
npm run tauri dev
# 构建发布版本
npm run tauri build
```

### 单元测试
```bash
# 运行所有测试
cargo test
# 运行指定测试
cargo test --test <test_name>
```

直接通过CURL测试API接口示例
```bash
# 测试健康检查接口
curl -X GET http://localhost:8090/health

# 测试截屏功能
curl -X POST http://localhost:8090/computer -H "Content-Type: application/json" -d '{"action":"screenshot"}'

# 测试编辑文件功能
curl -X POST http://localhost:8090/edit -H "Content-Type: application/json" -d '{"command":"create","path":"/home/consoley/test.txt","file_text":"Hello, World!"}'

# 测试执行bash命令功能
curl -X POST http://localhost:8090/bash -H "Content-Type: application/json" -d '{"command":"echo Hello, World!"}'
```

## 🤝 贡献

欢迎提交 Pull Request 和 Issue！在提交之前，请确保：

1. 更新测试用例
2. 更新文档
3. 遵循现有的代码风格

## 📄 许可证

本项目采用 [MIT](LICENSE) 许可证。

## 🙏 致谢

- [Tauri](https://tauri.app/)
- [Anthropic Computer Use](https://www.anthropic.com/)
- [noVNC](https://novnc.com/)

## 📞 联系方式

- 作者：EvalsOne
- Email：[contact@evalsone.com](mailto:contact@evalsone.com)
- GitHub：[@EvalsOne](https://github.com/EvalsOne)
