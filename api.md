# API Endpoints List

## 1. Computer Endpoint (`/computer`)

### Supported Operations:
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

### 1.1 Get Cursor Position
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"cursor_position"}'
```
Response:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Cursor position is: X=100, Y=200"
}
```

### 1.2 Keyboard Key Press
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"key", "text":"Return"}'
```
Response:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Key action executed successfully"
}
```

### 1.3 Type Text
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"type", "text":"Hello World"}'
```
Response:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Type action executed successfully"
}
```

### 1.4 Move Mouse
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"mouse_move", "coordinate":[100, 200]}'
```
Response:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Mouse move executed successfully"
}
```

### 1.5 Mouse Click Operations
```bash
# Left Click
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"left_click"}'

# Right Click
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"right_click"}'

# Middle Click
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"middle_click"}'

# Double Click
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"double_click"}'

# Left Click Drag
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"left_click_drag"}'
```

### 1.6 Screenshot
```bash
curl -X POST http://localhost:8090/computer \
  -H "Content-Type: application/json" \
  -d '{"action":"screenshot"}'
```
Response:
```json
{
  "type": "base64",
  "media_type": "image/png",
  "data": "base64_encoded_image_data..."
}
```

## 2. Edit Endpoint (`/edit`)

### Supported Operations:
- view
- create
- str_replace
- insert
- undo_edit

### 2.1 View File Content
```bash
# View entire file
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "view",
    "path": "/path/to/file"
  }'

# View specific line range
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "view",
    "path": "/path/to/file",
    "view_range": [1, 10]
  }'
```

### 2.2 Create File
```bash
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "create",
    "path": "/path/to/file",
    "file_text": "File content here"
  }'
```

### 2.3 String Replace
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

### 2.4 Insert Text
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

### 2.5 Undo Edit
```bash
curl -X POST http://localhost:8090/edit \
  -H "Content-Type: application/json" \
  -d '{
    "command": "undo_edit",
    "path": "/path/to/file"
  }'
```

## 3. Bash Endpoint (`/bash`)

### 3.1 Execute Command
```bash
curl -X POST http://localhost:8090/bash \
  -H "Content-Type: application/json" \
  -d '{
    "command": "ls -la"
  }'
```
Response:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "command_output_here"
}
```

### 3.2 Restart Bash Session
```bash
curl -X POST http://localhost:8090/bash \
  -H "Content-Type: application/json" \
  -d '{
    "restart": true
  }'
```
Response:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Bash session has been restarted"
}
```

## 4. Health Check (`/health`)
```bash
curl -X GET http://localhost:8090/health
```
Response:
```json
{
  "type": "success",
  "media_type": "text/plain",
  "data": "Service is running"
}
```

Notes:
1. All responses follow a unified format:
```json
{
  "type": "success|error|base64",
  "media_type": "text/plain|image/png",
  "data": "Response data"
}
```

2. Error responses will include specific error information:
```json
{
  "type": "error",
  "media_type": "text/plain",
  "data": "Error description"
}
```

3. Port number 8090 is used in examples, please adjust according to your actual configuration.
