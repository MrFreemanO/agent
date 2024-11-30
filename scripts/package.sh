#!/bin/bash

# 确保docker镜像已经构建
docker build -t consoleai/desktop:latest ./docker/desktop

# 运行tauri构建
npm run tauri build

# 对于macOS，创建DMG
if [[ "$OSTYPE" == "darwin"* ]]; then
    # 创建DMG
    create-dmg \
        --volname "ConsoleY" \
        --window-pos 200 120 \
        --window-size 800 400 \
        --icon-size 100 \
        --icon "ConsoleY.app" 200 190 \
        --hide-extension "ConsoleY.app" \
        --app-drop-link 600 185 \
        "ConsoleY.dmg" \
        "src-tauri/target/release/bundle/macos/ConsoleY.app"
fi
