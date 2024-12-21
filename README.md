

# ConsoleY

```
 ______   ______   __   __   ______   ______   __       ______   __  __   
/\  ___\ /\  __ \ /\ "-.\ \ /\  ___\ /\  __ \ /\ \     /\  ___\ /\ \_\ \  
\ \ \____\ \ \/\ \\ \ \-.  \\ \___  \\ \ \/\ \\ \ \____\ \  __\ \ \____ \ 
 \ \_____\\ \_____\\ \_\\"\_\\/\_____\\ \_____\\ \_____\\ \_____\\/\_____\
  \/_____/ \/_____/ \/_/ \/_/ \/_____/ \/_____/ \/_____/ \/_____/ \/_____/
```

ConsoleY 是一个基于 Docker 的远程桌面环境，提供 HTTP API 接口实现远程操作。它将 Ubuntu 桌面环境容器化，并通过 Rust 实现的 API 服务提供远程控制能力，可以与Claude-3.5-sonnet等支持计算机使用和工具调用的AI大模型配合使用。

<!-- 画一张示意图 -->

## 功能特点

- 🖥️ 基于Docker的 Ubuntu 桌面环境，可通过浏览器访问
- 🚀 基于 Rust 提供完整的计算机操作 API 服务，包括桌面操作、文件操作、Shell 命令执行等，支持热加载。

### 组件说明

1. **桌面环境**
   - X11 Server: 提供图形显示服务
   - Mutter: 窗口管理器
   - x11vnc: VNC 服务器

2. **服务层**
   - Rust API Server: 提供 HTTP API 接口
   - noVNC Proxy: 提供 Web VNC 访问

3. **进程管理**
   - Supervisord: 管理所有服务进程

## API 接口

### 桌面操作 `/computer`
- 鼠标控制（移动、点击）
- 键盘输入
- 屏幕截图
- 光标位置获取

### 文件操作 `/edit`
- 文件创建和查看
- 文件内容修改
- 操作撤销

### Shell 操作 `/bash`
- 命令执行
- 会话管理
- 环境变量处理

## 快速开始

### 环境要求
- Docker
- Docker Compose

### 构建和运行

```bash
# 构建开发环境镜像
docker-compose -f docker-compose.yml build

# 启动开发环境容器
docker-compose -f docker-compose.yml up -d

# 停止开发环境容器
docker-compose -f docker-compose.yml down
```

### 打开桌面

```bash
# 安装前端依赖
npm install

# 启动桌面
npm run dev
```

### 端口映射
- 5800:5900 - VNC 服务
- 6070:6080 - noVNC Web 访问
- 8090:8080 - API 服务

### 单元测试和调试
```bash
# 运行所有测试
cargo test

# 运行指定测试
cargo test --test <test_name>

# 查看日志
docker-compose -f docker-compose.yml logs -f

# 进入容器
docker-compose -f docker-compose.yml exec consoley bash
```

### API 测试示例
```bash
# 健康检查
curl -X GET http://localhost:8090/health

# 执行点击操作
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"left_click"}'

# 执行 Shell 命令
curl -X POST http://localhost:8090/bash \
  -H "Content-Type: application/json" \
  -d '{"command":"ls -la"}'
```

## 技术栈

- 容器化：Docker
- API 服务：Rust + Actix-web
- 远程桌面：noVNC + x11vnc
- 窗口管理：Mutter
- 进程管理：Supervisor
- 开发工具：cargo-watch（热重载）

## 贡献指南

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

## 许可证

[MIT License](LICENSE)

## 联系方式

项目链接：[https://github.com/EvalsOne/consoley](https://github.com/EvalsOne/consoley)
