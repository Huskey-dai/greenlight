# Greenlight Hook 配置指南

## 安装 Hook 脚本

### 1. 创建目录

```bash
mkdir -p ~/.greenlight/hooks/lib
```

### 2. 复制脚本

从项目根目录的 `hooks/` 复制到 `~/.greenlight/hooks/`：

```bash
cp hooks/post-tool-use.js ~/.greenlight/hooks/
cp hooks/needs-input.js ~/.greenlight/hooks/
cp hooks/stop.js ~/.greenlight/hooks/
cp hooks/error.js ~/.greenlight/hooks/
cp hooks/lib/api-client.js ~/.greenlight/hooks/lib/
cp hooks/lib/session.js ~/.greenlight/hooks/lib/
```

### 3. 配置 Claude Code

编辑 `~/.claude/settings.json`，添加 `hooks` 部分：

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "*",
        "command": "node ~/.greenlight/hooks/post-tool-use.js",
        "description": "AI working state"
      }
    ],
    "notification": [
      {
        "matcher": "*",
        "command": "node ~/.greenlight/hooks/needs-input.js",
        "description": "AI needs user input"
      }
    ],
    "Stop": [
      {
        "matcher": "*",
        "command": "node ~/.greenlight/hooks/stop.js",
        "description": "Session ended"
      }
    ]
  }
}
```

## Hook 脚本说明

### post-tool-use.js

当 Claude Code 完成一次工具调用时触发。将当前会话状态设置为 `WORKING`。

从 stdin 读取 Claude Code 传入的 JSON 数据，提取工具名称作为 detail 字段。

### needs-input.js

当 Claude Code 需要用户授权（如执行 Bash 命令、写入文件）时触发。将当前会话状态设置为 `NEEDS_INPUT`。

### stop.js

当 Claude Code 会话结束时触发。先将状态设为 `IDLE`，然后删除会话记录。

### error.js

当发生错误时触发。将当前会话状态设置为 `ERROR`。

## 环境变量

| 变量 | 必填 | 说明 | 示例 |
|------|------|------|------|
| `GREENLIGHT_SESSION_ID` | 可选 | 会话唯一标识。未设置时自动生成 UUID。 | `bugfix-1` |
| `GREENLIGHT_SESSION_LABEL` | 可选 | 会话可读标签。未设置时默认 `"Session"`。 | `Bug Fix` |

### 多会话配置示例

```bash
# 终端 1：Bug 修复
export GREENLIGHT_SESSION_ID="bugfix-1"
export GREENLIGHT_SESSION_LABEL="Bug Fix"
claude

# 终端 2：代码重构
export GREENLIGHT_SESSION_ID="refactor-2"
export GREENLIGHT_SESSION_LABEL="Refactor"
claude

# 终端 3：主分支
export GREENLIGHT_SESSION_ID="main-3"
export GREENLIGHT_SESSION_LABEL="Main"
claude
```

## 端口发现

Greenlight 的 HTTP API 默认监听 `localhost:17321`。如果端口被占用，会自动尝试 17322-17331。

实际使用的端口写入 `~/.greenlight/port.txt`。Hook 脚本会自动读取此文件发现端口，无需手动配置。

## 状态文件

Greenlight 运行时写入 `~/.greenlight/status.json`，格式如下：

```json
{
  "version": 1,
  "timestamp": "2026-06-04T10:30:00.000Z",
  "sessions": {
    "bugfix-1": {
      "state": "WORKING",
      "label": "Bug Fix",
      "detail": "Using Write",
      "startedAt": "2026-06-04T10:28:00.000Z",
      "updatedAt": "2026-06-04T10:30:00.000Z",
      "source": "claude_code"
    }
  },
  "meta": {
    "pollIntervalMs": 1000,
    "engineVersion": "1.0.0"
  }
}
```

此文件供硬件端等第三方消费者读取。前端通过 Tauri 事件接收更新，不轮询此文件。