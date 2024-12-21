#!/bin/bash

# 启用调试输出
set -x

# 获取项目根目录的绝对路径
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "Project root: $PROJECT_ROOT"

# 设置输出文件
OUTPUT_FILE="${PROJECT_ROOT}/scripts/project_structure.md"
echo "Output file: $OUTPUT_FILE"

# 创建或清空输出文件
echo "# Project Structure" > "$OUTPUT_FILE"
echo "\`\`\`" >> "$OUTPUT_FILE"

# 切换到项目根目录
cd "$PROJECT_ROOT"
pwd

# 检查 tree 命令是否可用
if ! command -v tree &> /dev/null; then
    echo "Error: tree command not found. Please install it first." >> "$OUTPUT_FILE"
    echo "Ubuntu/Debian: sudo apt-get install tree" >> "$OUTPUT_FILE"
    echo "CentOS/RHEL: sudo yum install tree" >> "$OUTPUT_FILE"
    echo "macOS: brew install tree" >> "$OUTPUT_FILE"
    exit 1
fi

# 使用 tree 命令生成目录树
tree -I 'target|node_modules|.git|.idea|.vscode|*.pyc|__pycache__|dist|build|*.bak' \
     -a --dirsfirst -F \
     . 2>&1 | tee -a "$OUTPUT_FILE"

echo "\`\`\`" >> "$OUTPUT_FILE"

# 添加说明
cat << 'EOF' >> "$OUTPUT_FILE"

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
EOF

# 检查生成的文件
echo "Generated file contents:"
cat "$OUTPUT_FILE"

echo "Directory tree has been generated in $OUTPUT_FILE" 