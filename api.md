# API 接口列表

## 1. Computer 端点 (`/computer`)

### 支持的操作类型：
- cursor_position
- key
- type
- mouse_move
- left_click
- left_click_drag
- right_click
- middle_click
- double_click
- screenshot

### 1.1 获取鼠标位置
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"cursor_position"}'
```
响应:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Cursor position is: X=100, Y=200"
}
```

### 1.2 键盘按键
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"key", "text":"Return"}'
```
响应:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Key action executed successfully"
}
```

### 1.3 输入文本
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"type", "text":"Hello World"}'
```
响应:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Type action executed successfully"
}
```

### 1.4 移动鼠标
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"mouse_move", "coordinate":[100, 200]}'
```
响应:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Mouse move executed successfully"
}
```

### 1.5 鼠标点击操作
```bash
# 左键点击
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"left_click"}'

# 右键点击
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"right_click"}'

# 中键点击
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"middle_click"}'

# 双击
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"double_click"}'

# 左键拖拽
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"left_click_drag"}'
```

### 1.6 截图
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"screenshot"}'
```
响应:
```json
{
  "type": "base64",
  "media_type": "image/png",
  "data": "base64_encoded_image_data..."
}
```

## 2. Edit 端点 (`/edit`)

### 支持的操作类型：
- view
- create
- str_replace
- insert
- undo_edit

### 2.1 查看文件内容
```bash
# 查看整个文件
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "view",
    "path": "/path/to/file"
  }'

# 查看指定行范围
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "view",
    "path": "/path/to/file",
    "view_range": [1, 10]
  }'
```

### 2.2 创建文件
```bash
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "create",
    "path": "/path/to/file",
    "file_text": "File content here"
  }'
```

### 2.3 字符串替换
```bash
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "str_replace",
    "path": "/path/to/file",
    "old_str": "old text",
    "new_str": "new text"
  }'
```

### 2.4 插入文本
```bash
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "insert",
    "path": "/path/to/file",
    "file_text": "text to insert",
    "insert_line": 5
  }'
```

### 2.5 撤销编辑
```bash
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "undo_edit",
    "path": "/path/to/file"
  }'
```

## 3. Bash 端点 (`/bash`)

### 3.1 执行命令
```bash
curl -X POST http://localhost:8090/bash \
  -H "Content-Type: application/json" \
  -d '{
    "command": "ls -la"
  }'
```
响应:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "command_output_here"
}
```

### 3.2 重启 bash 会话
```bash
curl -X POST http://localhost:8090/bash \
  -H "Content-Type: application/json" \
  -d '{
    "restart": true
  }'
```
响应:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Bash session has been restarted"
}
```

## 4. 健康检查 (`/health`)
```bash
curl -X GET http://localhost:8090/health
```
响应:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Service is running"
}
```

注意：
1. 所有响应都遵循统一的格式：
```json
{
  "type": "success|error|base64",
  "media_type": "text/plain|image/png",
  "data": "响应数据"
}
```

2. 错误响应会包含具体的错误信息：
```json
{
  "type": "error",
  "media_type": "text/plain",
  "data": "错误描述"
}
```

3. 端口号在示例中使用 8090，请根据实际配置调整。
