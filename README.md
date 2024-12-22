[English](README.md) | [中文](README-zh.md) | [日本語](README-jp.md)

# ConsoleY

```
 ______   ______   __   __   ______   ______   __       ______   __  __   
/\  ___\ /\  __ \ /\ "-.\ \ /\  ___\ /\  __ \ /\ \     /\  ___\ /\ \_\ \  
\ \ \____\ \ \/\ \\ \ \-.  \\ \___  \\ \ \/\ \\ \ \____\ \  __\ \ \____ \ 
 \ \_____\\ \_____\\ \_\\"\_\\/\_____\\ \_____\\ \_____\\ \_____\\/\_____\
  \/_____/ \/_____/ \/_/ \/_/ \/_____/ \/_____/ \/_____/ \/_____/ \/_____/
```

Anthropic's Computer use feature allows users to operate their computer through tool calls, becoming a super agent for computer operations. However, in the official demo, the chat interface and desktop are integrated and cannot be used separately or accessed remotely through API calls.

ConsoleY is a Docker-based remote desktop environment that containerizes the Ubuntu desktop environment. It can not only be accessed through a browser but also provides remote control capabilities through an API service implemented in Rust, making it compatible with any local or cloud-based AI assistant that supports tool calls.

![demo](public/demo.png)

## Features

- 🖥️ Docker-based Ubuntu desktop environment, accessible via browser
- 🚀 Complete computer operation API service built with Rust, including desktop operations, file operations, Shell command execution, etc., with hot-reload support

## Quick Start

### Prerequisites
- Docker
- Docker Desktop
- Docker Compose

### Build and Run

```bash
# Build development environment image
docker-compose -f docker-compose.yml build

# Start development environment container
docker-compose -f docker-compose.yml up -d

# Stop development environment container
docker-compose -f docker-compose.yml down
```

### Open Desktop

```bash
# Install frontend dependencies
npm install

# Start desktop
npm run dev
```
After starting the desktop, you can access it through your browser at `http://localhost:1420`.

## API Access

After the Docker container starts, the API service will automatically start and can be accessed at `http://localhost:8090`.

The API interface design follows Anthropic's Computer use feature, supporting desktop operations, file operations, and Shell commands through a single endpoint. For more details, please refer to the [API Documentation](api.md).

## License

[MIT License](LICENSE) 