# Greenlight 🚦

> AI 编程助手工作状态红绿灯指示系统 — 实时显示 Claude Code / Codex 的工作状态

![状态示例](https://img.shields.io/badge/状态-🟢_IDLE-22C55E?style=flat-square)
![版本](https://img.shields.io/badge/版本-1.0.0-blue?style=flat-square)
![平台](https://img.shields.io/badge/平台-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)

## 这是什么？

Greenlight 是一个桌面端可视化工具，通过系统托盘图标实时显示 AI 编程助手（Claude Code、Codex）的工作状态。当 AI 正在生成代码时亮黄灯，需要你确认权限时闪红灯，任务完成后回到绿灯。

**核心问题**：使用多个 AI 会话并行工作时，你很难同时监控所有终端窗口的状态。Greenlight 让你在系统托盘一角一目了然。

## 状态定义

| 状态 | 灯光 | 含义 | 用户动作 |
|------|------|------|----------|
| `IDLE` | 🟢 绿灯常亮 | 等待输入 / 任务完成 | 输入指令 / Review 代码 |
| `WORKING` | 🟡 黄灯闪烁 | AI 正在工作中 | 可以离开 |
| `NEEDS_INPUT` | 🔴 红灯闪烁 | 需要用户授权/决策 | 回到终端确认 |
| `ERROR` | 🔴 红灯常亮 | 任务执行失败 | 检查日志 / 重启 |

**优先级**：ERROR > NEEDS_INPUT > WORKING > IDLE。多会话时，托盘显示最高优先级状态。

## 快速开始

### 前置条件

- **Rust** 1.70+ (通过 [rustup](https://rustup.rs/) 安装)
- **Node.js** 18 LTS+
- **MSVC Build Tools** (Windows) 或 Xcode Command Line Tools (macOS)
- **操作系统**：Windows 10 1903+, macOS 13+, Linux (glibc 2.31+)

### 安装与运行

```bash
# 克隆仓库
git clone https://github.com/Huskey-dai/greenlight.git
cd greenlight

# 安装前端依赖
npm install

# 开发模式运行
npm run tauri dev
```

首次运行会自动编译 Rust 后端和前端，可能需要几分钟。

### 配置 Claude Code Hook

安装后，需要配置 Claude Code 的 Hook 来推送状态更新：

1. 编辑 `~/.claude/settings.json`
2. 添加以下 Hook 配置：

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "*",
        "command": "node ~/.greenlight/hooks/post-tool-use.js",
        "description": "AI开始工作，状态切为WORKING"
      }
    ],
    "notification": [
      {
        "matcher": "*",
        "command": "node ~/.greenlight/hooks/needs-input.js",
        "description": "AI等待授权，状态切为NEEDS_INPUT"
      }
    ],
    "Stop": [
      {
        "matcher": "*",
        "command": "node ~/.greenlight/hooks/stop.js",
        "description": "会话结束，状态切为IDLE"
      }
    ]
  }
}
```

3. 将 Hook 脚本安装到本地：

```bash
# 创建目录
mkdir -p ~/.greenlight/hooks/lib

# 复制 Hook 脚本
cp hooks/post-tool-use.js ~/.greenlight/hooks/
cp hooks/needs-input.js ~/.greenlight/hooks/
cp hooks/stop.js ~/.greenlight/hooks/
cp hooks/error.js ~/.greenlight/hooks/
cp hooks/lib/api-client.js ~/.greenlight/hooks/lib/
cp hooks/lib/session.js ~/.greenlight/hooks/lib/
```

### 多会话支持

使用 Git Worktree 运行多个 Claude Code 实例时，为每个会话设置唯一标识：

```bash
# 在不同的 worktree 中启动 Claude Code
GREENLIGHT_SESSION_ID="bugfix-1" GREENLIGHT_SESSION_LABEL="Bug Fix" claude
GREENLIGHT_SESSION_ID="refactor-2" GREENLIGHT_SESSION_LABEL="Refactor" claude
GREENLIGHT_SESSION_ID="main-3" GREENLIGHT_SESSION_LABEL="Main" claude
```

## 架构

```
  Claude Code              HTTP API
     │                   localhost:17321
  Hook Scripts ──────────────▶│
                                │
                     ┌─────────┴─────────┐
                     │   Tauri 桌面应用    │
                     │                   │
                     │  ┌─────────────┐  │
                     │  │ AppState     │  │
                     │  │ (内存会话表)  │  │
                     │  └──────┬──────┘  │
                     │         │         │
                     │    ┌────┴────┐    │
                     │    │         │    │
                     │  写入文件  发射事件 │
                     │ status.json state- │
                     │           updated │
                     │    │         │    │
                     │    ▼         ▼    │
                     │ 硬件端   React前端 │
                     │ (预留)  (弹窗UI)  │
                     └───────────────────┘
```

### 核心组件

| 组件 | 文件 | 功能 |
|------|------|------|
| 状态引擎 | `state.rs` | 会话管理、状态转换、聚合计算、TTL 清理 |
| HTTP API | `http.rs` | 接收 Hook 推送的 4 个端点 |
| 系统托盘 | `tray.rs` | 托盘图标、右键菜单、弹窗管理 |
| 状态灯 | `icons.rs` | 按状态选择/生成托盘图标 |
| 文件写入 | `status_file.rs` | 原子写入 `~/.greenlight/status.json` |
| 主题检测 | `theme.rs` | OS 暗/亮主题检测与通知 |
| 错误类型 | `error.rs` | 统一错误处理 |

### HTTP API

| 方法 | 路径 | 功能 |
|------|------|------|
| `POST` | `/api/sessions/:id/state` | 更新/创建会话状态 |
| `DELETE` | `/api/sessions/:id` | 删除会话 |
| `GET` | `/api/sessions` | 列出所有会话 |
| `GET` | `/api/health` | 健康检查 |

### 数据流向

1. **Hook 脚本** → HTTP POST → **AppState** 更新
2. **AppState** 更新 → 三个副作用并行：
   - 更新系统托盘图标和提示文字
   - 原子写入 `~/.greenlight/status.json`
   - 发射 `state-updated` Tauri 事件到前端
3. **React 前端** 通过事件驱动接收更新（不轮询文件）

## 技术栈

| 层 | 技术 |
|------|------|
| 后端 | Rust + Tauri 2 + axum + tokio |
| 前端 | React 19 + TypeScript + Vite |
| 状态灯 | SVG + CSS 动画（无 GIF） |
| 通信 | Hook 脚本 → HTTP API → Tauri 事件 → React |

## 无障碍

- **WCAG 2.2 合规**：闪烁频率 ≤ 3Hz
- **`prefers-reduced-motion`**：动画降级为 3 秒渐变
- **屏幕阅读器**：`aria-live` 区域播报状态变化
- **非颜色传达**：每个状态灯旁有文字标签

## 配置

配置文件路径：`~/.greenlight/config.json`（自动生成，不存在则使用默认值）

```json
{
  "engine": {
    "session_ttl_minutes": 30,
    "needs_input_ttl_minutes": 120,
    "http_port": 17321
  }
}
```

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `session_ttl_minutes` | 30 | 普通会话超时清理时间 |
| `needs_input_ttl_minutes` | 120 | NEEDS_INPUT 状态超时时间（更长，因为用户可能在思考） |
| `http_port` | 17321 | HTTP API 端口（端口冲突时自动尝试 17322-17331） |

## 构建生产版本

```bash
# 构建前端
npm run build

# 构建 Tauri 应用（生成安装包）
npm run tauri build
```

输出目录：`src-tauri/target/release/bundle/`

## 开发

```bash
# 启动开发服务器（带热重载）
npm run tauri dev

# 检查 TypeScript
npx tsc --noEmit

# 检查 Rust
cargo check --manifest-path src-tauri/Cargo.toml

# 运行 Rust 测试
cargo test --manifest-path src-tauri/Cargo.toml
```

## 许可证

MIT License - 详见 [LICENSE](LICENSE)

---

**让 AI 的工作状态，一目了然。** 🚦