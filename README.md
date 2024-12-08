# ConsoleY

受Anthropic的 Computer Use 项目启发，我开发了ConsoleY，它提供了一个在Docker Container中运行的Ubuntu操作系统和GUI桌面，并且提供Rust API 接口，允许API通过本地或远程调用接口操作计算机进行截图、编辑文件、执行bash命令等操作。

使用者还可以将Docker Container中的ConsoleY构建为桌面应用程序，方便使用和分发。桌面应用程序基于Tauri开发，支持跨平台。Docker内的API Server使用Rust语言开发。

这是我第一次使用Rust语言开发项目，很幸运在AI的辅助下，我最终成功完成了ConsoleY。

## 使用方式

ConsoleY可以分为开发环境和生产环境：
- 开发环境：通过开发配置构建并启动，通过Cargo watch实时编译，方便调试API Server。
- 生产环境：通过生产配置构建并启动，会将构建好的API Server和桌面环境打包成一个镜像，更方便使用和分发。

### 在开发环境中使用

#### 需要安装的依赖
- [Docker Desktop](https://www.docker.com/get-started)
- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) (>= 14.0.0)

#### 安装和使用步骤

1. 克隆仓库

```bash
git clone https://github.com/EvalsOne/consoley.git
cd consoley
```

2. 构建 Docker 镜像

```bash
# 构建开发环境镜像
docker-compose -f docker-compose.dev.yml build

# 启动开发环境容器
docker-compose -f docker-compose.dev.yml up -d
```

这时，你已经可以通过6070端口访问桌面环境，并通过8090端口访问API Server。

3. 启动桌面程序

```bash
#安装前端依赖
npm install

#启动桌面程序
npm run tauri dev
```

4. API接口测试

```bash
# 运行所有测试所有API接口
cargo test

# 运行指定测试
cargo test --test <test_name>
```

### 构建和使用生产环境

#### 构建Docker镜像

```bash
docker-compose -f docker-compose.yml build
```

#### 打包Mac桌面应用程序

```bash
# 构建镜像
docker build -t consoleai/desktop:latest-arm64 -f docker/desktop/Dockerfile docker/desktop

# 打包桌面应用程序
cargo tauri build --target aarch64-apple-darwin

# 通过命令行启动桌面应用程序
/Applications/ConsoleY.app/Contents/MacOS/consoley
```

### 使用API调用Computer功能

#### 本地调用

使用者可以通过computer、edit、bash三个本地环境API接口调用桌面环境，以及health接口检查API Server状态。访问示例如下：

```bash 
# 截屏
curl -X POST http://localhost:8090/computer -H "Content-Type: application/json" -d '{"action":"screenshot"}'

# 编辑文件
curl -X POST http://localhost:8090/edit -H "Content-Type: application/json" -d '{"command":"create","path":"/home/consoley/test.txt","file_text":"Hello, World!"}'

# 执行bash命令
curl -X POST http://localhost:8090/bash -H "Content-Type: application/json" -d '{"command":"echo Hello, World!"}'

# 检查API Server状态
curl -X GET http://localhost:8090/health
```

接口的具体调用参数与Anthropic的 Computer Use 项目中的定义基本相同，请参考[API.md](docs/API.md)。

#### 远程调用
需要通过tunnel隧道将本地8090端口映射到远程服务器，然后通过远程服务器地址调用API。

#### 通过ConsoleX调用

待补充

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
