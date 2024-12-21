#!/bin/bash

# Ensure docker image is built
docker build -t consoleai/desktop:latest ./docker/desktop

# Run tauri build
npm run tauri build

# For macOS, create DMG
if [[ "$OSTYPE" == "darwin"* ]]; then
    # Create DMG
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
