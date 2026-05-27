# 内网协同工具套件 V1 — 实施计划

## Goal

实现一个去中心化的 Windows 内网协同工具，支持用户发现、P2P 文件传输、共享聊天、共享文件夹双向同步、共享密码栏/公告栏、个人/团队看板。绿色免安装，开机自启，托盘常驻。

## Architecture

- **Frontend**: React 18 + TypeScript + Tailwind CSS，微信/QQ 风格 IM 界面
- **Backend**: Rust (Tauri v2)，处理网络、文件系统、数据库、系统托盘
- **Database**: SQLite (rusqlite)，本地存储
- **Network**: UDP 广播发现 + TCP P2P 长连接 + 动态端口文件传输
- **State Sync**: 向量时钟 + Last-Write-Wins 用于共享状态（密码/公告/看板）
- **File Sync**: ReadDirectoryChangesW 监控 + Blake3 块哈希比对 + 增量传输

## Tech Stack

| 组件 | 选型 |
|------|------|
| 应用框架 | Tauri v2 |
| 前端框架 | React 18 + TypeScript |
| 样式 | Tailwind CSS |
| 图标 | lucide-react |
| 拖拽 | @dnd-kit/core + @dnd-kit/sortable |
| Rust ORM/DB | rusqlite + r2d2_sqlite |
| 文件哈希 | blake3 |
| 序列化 | serde + serde_json |
| 配置持久化 | confy |

## Baseline / Authority Refs

- Spec Brief: `docs/aegis/specs/2026-05-25-lan-collaboration-suite-brief.md`
- 项目从零开始，无现有代码约束

## Compatibility Boundary

- Windows 10/11 独占
- 同网段局域网（/24 子网），不处理 NAT 穿透
- 所有节点对等，无版本回退/兼容旧协议需求（V1 无需考虑）
- 聊天/文件传输仅实时，不保证离线送达

## Verification

- 两台 Windows 机器同网段可互相发现
- 文件传输速度达到带宽 80%+
- 共享文件夹修改 3 秒内同步到订阅者
- 看板状态变更实时同步
- 清屏操作所有客户端同步执行
- 15 天快照自动清理验证

---

## Plan Pressure Test

- **Owner / contract / retirement**: 全新项目，无旧代码。Rust 模块分层清晰，前端按页面组件划分。
- **Verification scope**: 功能验证靠双机局域网测试；单元测试覆盖 Rust 核心算法（哈希、冲突解决、协议序列化）。
- **Task executability**: 每个任务 2-5 分钟动作，有完整代码和命令。
- **Pressure result**: proceed

---

## File Map

```
shimmen-lan-suite/
├── src/                              # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── index.css
│   ├── components/
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx           # 左侧图标导航栏
│   │   │   ├── MainContent.tsx       # 中间主内容区
│   │   │   └── TitleBar.tsx          # 顶部标题栏（显示用户名+IP）
│   │   ├── user/
│   │   │   ├── UserList.tsx          # 在线用户列表
│   │   │   └── UserCard.tsx          # 用户卡片
│   │   ├── chat/
│   │   │   ├── ChatWindow.tsx        # 共享聊天窗
│   │   │   ├── ChatMessage.tsx       # 单条消息气泡
│   │   │   └── TransferWindow.tsx    # 点对点传输窗口
│   │   ├── folder/
│   │   │   ├── FolderBrowser.tsx     # 共享文件夹浏览器
│   │   │   ├── FolderTree.tsx        # 左侧树形目录
│   │   │   └── FileList.tsx          # 右侧文件列表
│   │   ├── board/
│   │   │   ├── KanbanBoard.tsx       # 看板容器
│   │   │   ├── KanbanColumn.tsx      # 单列（待处理/处理中/已完成）
│   │   │   ├── TaskCard.tsx          # 任务卡片
│   │   │   └── TeamBoard.tsx         # 团队看板视图
│   │   ├── shared/
│   │   │   ├── PasswordVault.tsx     # 共享密码栏
│   │   │   ├── AnnouncementBoard.tsx # 共享公告栏
│   │   │   └── ClearScreenButton.tsx # 清屏按钮
│   │   └── common/
│   │       ├── IconButton.tsx        # 带 tooltip 的图标按钮
│   │       ├── Modal.tsx             # 通用弹窗
│   │       └── Toast.tsx             # 轻量通知
│   ├── hooks/
│   │   ├── useUsers.ts               # 用户列表状态
│   │   ├── useChat.ts                # 聊天状态
│   │   ├── useFolders.ts             # 共享文件夹状态
│   │   ├── useTasks.ts               # 看板任务状态
│   │   └── useSharedState.ts         # 密码栏/公告栏状态
│   ├── types/
│   │   └── index.ts                  # TypeScript 类型定义
│   └── utils/
│       └── format.ts                 # 格式化工具（IP、时间、文件大小）
├── src-tauri/                        # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands/                 # Tauri 前端可调用的命令
│       │   ├── user.rs               # 用户相关命令
│       │   ├── chat.rs               # 聊天相关命令
│       │   ├── file_transfer.rs      # 文件传输命令
│       │   ├── folder_sync.rs        # 文件夹同步命令
│       │   ├── board.rs              # 看板命令
│       │   ├── vault.rs              # 密码栏命令
│       │   ├── announcement.rs       # 公告栏命令
│       │   └── system.rs             # 系统命令（托盘、自启）
│       ├── network/
│       │   ├── discovery.rs          # UDP 广播发现
│       │   ├── peer.rs               # 对等节点管理
│       │   ├── protocol.rs           # 消息协议定义
│       │   ├── server.rs             # TCP 服务端
│       │   └── client.rs             # TCP 客户端连接
│       ├── sync/
│       │   ├── state.rs              # 共享状态 CRDT/向量时钟
│       │   ├── conflict.rs           # 冲突解决逻辑
│       │   └── broadcaster.rs        # 状态广播器
│       ├── file_sync/
│       │   ├── engine.rs             # 同步引擎主逻辑
│       │   ├── watcher.rs            # 文件系统监控
│       │   ├── indexer.rs            # 文件索引（哈希、元数据）
│       │   ├── transfer.rs           # 文件块传输
│       │   └── history.rs            # 历史快照管理
│       ├── db/
│       │   ├── mod.rs                # 数据库连接池初始化
│       │   ├── schema.rs             # 表结构定义
│       │   ├── user.rs               # 用户表操作
│       │   ├── task.rs               # 任务表操作
│       │   ├── password.rs           # 密码表操作
│       │   ├── announcement.rs       # 公告表操作
│       │   ├── shared_folder.rs      # 共享文件夹表操作
│       │   └── file_version.rs       # 文件版本表操作
│       ├── models/
│       │   ├── mod.rs
│       │   ├── user.rs
│       │   ├── message.rs
│       │   ├── task.rs
│       │   ├── password.rs
│       │   ├── announcement.rs
│       │   ├── shared_folder.rs
│       │   └── file_version.rs
│       ├── system/
│       │   ├── tray.rs               # 系统托盘
│       │   ├── autostart.rs          # 开机自启
│       │   └── crypto.rs             # 本地 AES 加密
│       └── config.rs                 # 应用配置（用户名、端口、版本）
├── docs/aegis/
│   ├── specs/...
│   └── plans/...
├── package.json
├── tailwind.config.js
├── tsconfig.json
├── vite.config.ts
└── index.html
```

---

## Risks

| 风险 | 缓解 |
|------|------|
| Windows Defender 阻止网络通信 | 首次启动提示用户添加防火墙例外 |
| 文件同步冲突频繁 | 冲突副本机制 + 历史快照 |
| SQLite 并发写入 | 使用连接池 + WAL 模式 |
| 大文件传输阻塞 UI | Rust 端异步传输，前端进度通过事件推送 |
| 向量时钟漂移 | 定期全量状态同步作为保底 |

## Retirement

- 无旧代码需要退役

---

## Phase 1: 项目骨架与基础设施

### Task 1.1: 初始化 Tauri + React 项目

**Files**: Create `package.json`, `Cargo.toml`, `tauri.conf.json`, directory structure
**Why**: 建立项目基础骨架
**Impact**: 无兼容性问题
**Verification**: `cargo tauri dev` 能启动空白窗口

**Steps**:

1. **安装 Tauri CLI 并创建项目**
```bash
cargo install tauri-cli --version "^2.0"
cargo create-tauri-app shimmen-lan-suite --template vanilla-ts --manager npm
cd shimmen-lan-suite
```

2. **修改 package.json 添加 React + Tailwind + lucide-react**
```json
{
  "name": "shimmen-lan-suite",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "lucide-react": "^0.400.0",
    "@dnd-kit/core": "^6.1.0",
    "@dnd-kit/sortable": "^8.0.0",
    "@dnd-kit/utilities": "^3.2.2"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "autoprefixer": "^10.4.20",
    "postcss": "^8.4.40",
    "tailwindcss": "^3.4.7",
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  }
}
```

3. **安装依赖**
```bash
npm install
```

4. **配置 Tailwind CSS**
创建 `tailwind.config.js`:
```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
```

创建 `postcss.config.js`:
```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

创建 `src/index.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  margin: 0;
  padding: 0;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

/* 隐藏滚动条但保留滚动功能 */
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}
.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
```

5. **配置 Vite + React**
修改 `vite.config.ts`:
```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

修改 `tsconfig.json` 确保包含 React JSX:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

6. **修改 `src/main.tsx`**:
```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

7. **创建基础 `src/App.tsx`**:
```tsx
function App() {
  return (
    <div className="h-screen w-screen bg-gray-100 flex items-center justify-center">
      <h1 className="text-2xl font-bold text-gray-800">Shimmen LAN Suite</h1>
    </div>
  );
}

export default App;
```

8. **修改 `index.html` 标题**:
```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>水门内网协同</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

9. **修改 `src-tauri/Cargo.toml` 添加依赖**:
```toml
[package]
name = "shimmen-lan-suite"
version = "0.1.0"
description = "内网去中心化协同工具"
edition = "2021"
rust-version = "1.75"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled", "chrono"] }
r2d2_sqlite = "0.25"
blake3 = "1.5"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.8", features = ["v4", "serde"] }
confy = "0.6"
windows-sys = { version = "0.52", features = ["Win32_System_Threading", "Win32_UI_WindowsAndMessaging"] }
aes-gcm = "0.10"
rand = "0.8"
walkdir = "2"
crossbeam-channel = "0.5"
tokio = { version = "1", features = ["rt-multi-thread", "net", "sync", "time", "macros"] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

10. **修改 `src-tauri/tauri.conf.json` 启用托盘和权限**:
```json
{
  "$schema": "../node_modules/@tauri-apps/cli/schema.json",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "nsis",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.ico"
    ]
  },
  "productName": "水门内网协同",
  "version": "0.1.0",
  "identifier": "com.shimmen.lan-suite",
  "app": {
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "水门内网协同",
        "width": 1100,
        "height": 750,
        "minWidth": 900,
        "minHeight": 600,
        "center": true
      }
    ],
    "withGlobalTauri": true
  }
}
```

11. **运行验证**
```bash
cargo tauri dev
```
预期：弹出窗口显示 "Shimmen LAN Suite"

---

### Task 1.2: 配置 Rust 数据库模块与 Schema

**Files**: `src-tauri/src/db/mod.rs`, `src-tauri/src/db/schema.rs`
**Why**: 所有业务功能依赖本地持久化
**Impact**: 后续所有模块的基础
**Verification**: 应用启动时自动创建数据库文件，无报错

**Steps**:

1. **创建 `src-tauri/src/db/mod.rs`**:
```rust
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db(app_dir: &std::path::Path) -> Result<DbPool> {
    let db_path = app_dir.join("shimmen.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;"
    )?;
    schema::create_tables(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

mod schema;
pub mod user;
pub mod task;
pub mod password;
pub mod announcement;
pub mod shared_folder;
pub mod file_version;
```

2. **创建 `src-tauri/src/db/schema.rs`**:
```rust
use rusqlite::{Connection, Result};

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            ip TEXT NOT NULL,
            last_seen INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'offline'
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            project TEXT NOT NULL DEFAULT '',
            deadline INTEGER,
            contact TEXT NOT NULL DEFAULT '',
            priority TEXT NOT NULL DEFAULT 'medium',
            description TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'todo',
            creator_id TEXT NOT NULL,
            assignee_id TEXT,
            is_team_visible INTEGER NOT NULL DEFAULT 0,
            attached_files TEXT NOT NULL DEFAULT '[]',
            archived_to_folder_id TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            version TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS password_entries (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            account TEXT NOT NULL DEFAULT '',
            password TEXT NOT NULL,
            note TEXT NOT NULL DEFAULT '',
            created_by TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            version TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS announcements (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            is_pinned INTEGER NOT NULL DEFAULT 0,
            created_by TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            version TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS shared_folders (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            owner_name TEXT NOT NULL,
            local_path TEXT NOT NULL,
            name TEXT NOT NULL,
            subscribers TEXT NOT NULL DEFAULT '[]',
            sync_status TEXT NOT NULL DEFAULT 'paused'
        );

        CREATE TABLE IF NOT EXISTS file_versions (
            file_id TEXT NOT NULL,
            shared_folder_id TEXT NOT NULL,
            version_id INTEGER NOT NULL,
            file_hash TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            modified_at INTEGER NOT NULL,
            modified_by TEXT NOT NULL,
            snapshot_path TEXT NOT NULL,
            PRIMARY KEY (file_id, version_id)
        );

        CREATE TABLE IF NOT EXISTS transfer_records (
            id TEXT PRIMARY KEY,
            peer_id TEXT NOT NULL,
            peer_name TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            direction TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at INTEGER NOT NULL,
            completed_at INTEGER
        );"
    )?;
    Ok(())
}
```

3. **修改 `src-tauri/src/main.rs` 初始化数据库**:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod commands;
mod db;
mod models;
mod network;
mod sync;
mod file_sync;
mod system;
mod config;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_pool = db::init_db(&app_dir)?;
            app.manage(db_pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

4. **创建空模块文件占位**（避免编译报错）:
```bash
touch src-tauri/src/commands/mod.rs
touch src-tauri/src/models/mod.rs
touch src-tauri/src/network/mod.rs
touch src-tauri/src/sync/mod.rs
touch src-tauri/src/file_sync/mod.rs
touch src-tauri/src/system/mod.rs
touch src-tauri/src/config.rs
```

5. **在 `src-tauri/src/lib.rs` 声明模块**:
```rust
pub mod commands;
pub mod db;
pub mod models;
pub mod network;
pub mod sync;
pub mod file_sync;
pub mod system;
pub mod config;
```

6. **运行验证**
```bash
cargo check
```
预期：编译通过，无报错

---

## Phase 2: 网络发现与用户列表

### Task 2.1: 实现 UDP 广播发现协议（Rust）

**Files**: `src-tauri/src/network/discovery.rs`, `src-tauri/src/network/protocol.rs`, `src-tauri/src/network/peer.rs`
**Why**: 核心基础功能，所有其他功能依赖用户发现
**Impact**: 无兼容性问题
**Verification**: 两台机器运行后能在 3 秒内互相发现

**Steps**:

1. **创建 `src-tauri/src/config.rs`**:
```rust
use serde::{Deserialize, Serialize};

pub const DISCOVERY_PORT: u16 = 23333;
pub const CONTROL_PORT: u16 = 23334;
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub username: String,
    pub device_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            username: whoami::username(),
            device_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

pub fn load_config() -> AppConfig {
    confy::load("shimmen-lan-suite", "config").unwrap_or_default()
}

pub fn save_config(cfg: &AppConfig) -> Result<(), confy::ConfyError> {
    confy::store("shimmen-lan-suite", "config", cfg)
}
```

2. **创建 `src-tauri/src/models/mod.rs`**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub ip: String,
    pub status: UserStatus,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Online,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum NetworkMessage {
    Discovery { id: String, username: String, ip: String, version: String },
    DiscoveryResponse { id: String, username: String, ip: String, version: String },
    Heartbeat { id: String },
    ChatMessage { id: String, sender_id: String, sender_name: String, content: String, message_type: String },
    ClearScreen,
    StateSync { table: String, data: serde_json::Value, version: serde_json::Value },
    FileRequest { file_id: String, file_name: String, file_size: u64 },
    FileChunk { file_id: String, offset: u64, data: Vec<u8> },
}
```

3. **创建 `src-tauri/src/network/protocol.rs`**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryPacket {
    pub id: String,
    pub username: String,
    pub ip: String,
    pub version: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPacket {
    pub id: String,
    pub timestamp: i64,
}

pub const DISCOVERY_INTERVAL_SECS: u64 = 2;
pub const PEER_TIMEOUT_SECS: u64 = 6;
```

4. **创建 `src-tauri/src/network/peer.rs`**:
```rust
use crate::models::User;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type PeerMap = Arc<Mutex<HashMap<String, PeerInfo>>>;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub user: User,
    pub last_seen: Instant,
}

pub fn create_peer_map() -> PeerMap {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn update_peer(peers: &PeerMap, user: User) {
    let mut map = peers.lock().unwrap();
    map.insert(user.id.clone(), PeerInfo {
        user,
        last_seen: Instant::now(),
    });
}

pub fn get_online_users(peers: &PeerMap) -> Vec<User> {
    let map = peers.lock().unwrap();
    map.values()
        .filter(|p| p.last_seen.elapsed() < Duration::from_secs(6))
        .map(|p| p.user.clone())
        .collect()
}

pub fn remove_stale_peers(peers: &PeerMap) {
    let mut map = peers.lock().unwrap();
    let stale: Vec<String> = map.iter()
        .filter(|(_, p)| p.last_seen.elapsed() >= Duration::from_secs(6))
        .map(|(k, _)| k.clone())
        .collect();
    for id in stale {
        map.remove(&id);
    }
}
```

5. **创建 `src-tauri/src/network/discovery.rs`**:
```rust
use crate::config::{AppConfig, DISCOVERY_PORT};
use crate::models::{User, UserStatus};
use crate::network::peer::{PeerMap, update_peer};
use crate::network::protocol::{DiscoveryPacket, HeartbeatPacket, DISCOVERY_INTERVAL_SECS, PEER_TIMEOUT_SECS};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn start_discovery(config: AppConfig, peers: PeerMap) -> std::io::Result<()> {
    let socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT))?);
    socket.set_broadcast(true)?;
    
    let my_id = config.device_id.clone();
    let my_username = config.username.clone();
    let my_version = crate::config::APP_VERSION.to_string();
    
    // 获取本机局域网 IP
    let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    
    // 发送线程：广播发现包
    let send_socket = socket.clone();
    thread::spawn(move || {
        let broadcast_addr = format!("255.255.255.255:{}", DISCOVERY_PORT);
        loop {
            let packet = DiscoveryPacket {
                id: my_id.clone(),
                username: my_username.clone(),
                ip: local_ip.clone(),
                version: my_version.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            let json = serde_json::to_string(&packet).unwrap();
            let _ = send_socket.send_to(json.as_bytes(), &broadcast_addr);
            thread::sleep(Duration::from_secs(DISCOVERY_INTERVAL_SECS));
        }
    });
    
    // 接收线程：处理发现包和心跳
    let recv_socket = socket.clone();
    let recv_peers = peers.clone();
    let recv_id = my_id.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match recv_socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    let msg = String::from_utf8_lossy(&buf[..len]);
                    if let Ok(packet) = serde_json::from_str::<DiscoveryPacket>(&msg) {
                        if packet.id != recv_id {
                            let user = User {
                                id: packet.id,
                                username: packet.username,
                                ip: packet.ip,
                                status: UserStatus::Online,
                                version: packet.version,
                            };
                            update_peer(&recv_peers, user);
                            // 回复发现响应
                            let response = DiscoveryPacket {
                                id: recv_id.clone(),
                                username: my_username.clone(),
                                ip: local_ip.clone(),
                                version: my_version.clone(),
                                timestamp: chrono::Utc::now().timestamp(),
                            };
                            let json = serde_json::to_string(&response).unwrap();
                            let _ = recv_socket.send_to(json.as_bytes(), addr);
                        }
                    }
                }
                Err(_) => {}
            }
        }
    });
    
    // 清理线程：移除超时节点
    let clean_peers = peers.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(PEER_TIMEOUT_SECS));
            crate::network::peer::remove_stale_peers(&clean_peers);
        }
    });
    
    Ok(())
}

fn get_local_ip() -> Option<String> {
    use std::net::TcpStream;
    if let Ok(socket) = TcpStream::connect("8.8.8.8:80") {
        if let Ok(addr) = socket.local_addr() {
            return Some(addr.ip().to_string());
        }
    }
    None
}
```

6. **修改 `src-tauri/src/network/mod.rs`**:
```rust
pub mod discovery;
pub mod peer;
pub mod protocol;
pub mod server;
pub mod client;
```

7. **在 `src-tauri/Cargo.toml` 添加 whoami**:
```toml
whoami = "1.5"
```

8. **运行验证**:
```bash
cargo check
```
预期：编译通过

---

由于计划非常长，后续 Phase 3-9 的任务将在执行过程中逐步展开。以上为完整的 Phase 1-2 计划，包含可直接执行的所有代码和命令。
## Phase 3: 前端主界面骨架

### Task 3.1: 搭建微信风格布局（React）

**Files**: `src/App.tsx`, `src/components/layout/Sidebar.tsx`, `src/components/layout/MainContent.tsx`, `src/components/layout/TitleBar.tsx`
**Why**: 所有功能页面的容器
**Impact**: 定义全局 UI 结构
**Verification**: 运行后能看到左侧图标栏 + 中间内容区 + 顶部标题栏

**Steps**:

1. **创建 `src/types/index.ts`**:
```typescript
export interface User {
  id: string;
  username: string;
  ip: string;
  status: 'online' | 'offline';
  version: string;
}

export interface ChatMessage {
  id: string;
  sender_id: string;
  sender_name: string;
  type: 'text' | 'file' | 'image';
  content: string;
  timestamp: number;
}

export interface Task {
  id: string;
  title: string;
  project: string;
  deadline: string | null;
  contact: string;
  priority: 'low' | 'medium' | 'high';
  description: string;
  status: 'todo' | 'doing' | 'done';
  creator_id: string;
  assignee_id: string | null;
  is_team_visible: boolean;
  attached_files: string[];
}

export interface PasswordEntry {
  id: string;
  name: string;
  account: string;
  password: string;
  note: string;
  created_by: string;
  updated_at: number;
}

export interface Announcement {
  id: string;
  title: string;
  content: string;
  is_pinned: boolean;
  created_by: string;
  updated_at: number;
}

export interface SharedFolder {
  id: string;
  owner_id: string;
  owner_name: string;
  local_path: string;
  name: string;
  sync_status: 'syncing' | 'paused' | 'error';
}
```

2. **创建 `src/components/layout/Sidebar.tsx`**:
```tsx
import { Users, MessageSquare, FolderOpen, Key, ClipboardList, Layout, Settings } from 'lucide-react';

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
}

const tabs = [
  { id: 'users', icon: Users, label: '用户' },
  { id: 'chat', icon: MessageSquare, label: '群聊' },
  { id: 'folders', icon: FolderOpen, label: '文件夹' },
  { id: 'passwords', icon: Key, label: '密码' },
  { id: 'announcements', icon: ClipboardList, label: '公告' },
  { id: 'board', icon: Layout, label: '看板' },
];

export default function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  return (
    <div className="w-16 bg-gray-900 flex flex-col items-center py-4 select-none">
      <div className="mb-6 text-cyan-400 font-bold text-lg">S</div>
      {tabs.map((tab) => {
        const Icon = tab.icon;
        const isActive = activeTab === tab.id;
        return (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={`relative p-3 mb-2 rounded-xl transition-all duration-200 group ${
              isActive
                ? 'bg-cyan-600 text-white'
                : 'text-gray-400 hover:text-white hover:bg-gray-800'
            }`}
            title={tab.label}
          >
            <Icon size={22} strokeWidth={1.5} />
            {isActive && (
              <span className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-6 bg-cyan-400 rounded-r-full" />
            )}
            <span className="absolute left-14 bg-gray-800 text-white text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50">
              {tab.label}
            </span>
          </button>
        );
      })}
    </div>
  );
}
```

3. **创建 `src/components/layout/TitleBar.tsx`**:
```tsx
interface TitleBarProps {
  username: string;
  ip: string;
  onlineCount: number;
}

export default function TitleBar({ username, ip, onlineCount }: TitleBarProps) {
  return (
    <div className="h-12 bg-white border-b border-gray-200 flex items-center justify-between px-4 shrink-0">
      <div className="flex items-center gap-2 text-sm text-gray-600">
        <span className="font-medium text-gray-800">{username}</span>
        <span className="text-gray-400">({ip})</span>
        <span className="ml-3 flex items-center gap-1.5 text-xs">
          <span className="w-2 h-2 bg-green-500 rounded-full"></span>
          在线 {onlineCount} 人
        </span>
      </div>
      <div className="flex items-center gap-2">
        <button className="w-8 h-8 flex items-center justify-center text-gray-500 hover:text-gray-800 hover:bg-gray-100 rounded-lg transition-colors">
          −
        </button>
        <button className="w-8 h-8 flex items-center justify-center text-gray-500 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors">
          ×
        </button>
      </div>
    </div>
  );
}
```

4. **创建 `src/components/layout/MainContent.tsx`**:
```tsx
import { useState } from 'react';
import Sidebar from './Sidebar';
import TitleBar from './TitleBar';

export default function MainContent() {
  const [activeTab, setActiveTab] = useState('users');

  return (
    <div className="h-screen w-screen flex flex-col bg-gray-50 overflow-hidden">
      <TitleBar username="水门" ip="192.168.1.12" onlineCount={5} />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
        <div className="flex-1 p-6 overflow-auto">
          {/* 页面内容占位 */}
          <div className="text-2xl font-bold text-gray-800 capitalize">{activeTab}</div>
        </div>
      </div>
    </div>
  );
}
```

5. **修改 `src/App.tsx`**:
```tsx
import MainContent from './components/layout/MainContent';

function App() {
  return <MainContent />;
}

export default App;
```

6. **运行验证**:
```bash
npm run dev
```
预期：左侧深色图标栏，顶部标题栏显示用户名和 IP，中间可切换页面

---

## Phase 4: 系统托盘与开机自启

### Task 4.1: Rust 端实现系统托盘

**Files**: `src-tauri/src/system/tray.rs`
**Why**: 必须功能，应用常驻后台
**Impact**: 涉及 Tauri 托盘 API
**Verification**: 最小化后缩到托盘，左键唤醒，右键菜单正常

**Steps**:

1. **创建 `src-tauri/src/system/tray.rs`**:
```rust
use tauri::{tray::TrayIconBuilder, menu::Menu, image::Image, Manager, AppHandle, Wry};

pub fn setup_tray(app: &AppHandle<Wry>) -> Result<(), Box<dyn std::error::Error>> {
    let menu = Menu::new(app)?;
    
    let tray = TrayIconBuilder::new()
        .icon(Image::from_bytes(include_bytes!("../../../icons/icon.png").as_slice())?)
        .tooltip("水门内网协同")
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.0.as_str() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(true) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;
    
    Ok(())
}
```

2. **创建 `src-tauri/src/system/autostart.rs`**:
```rust
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

pub fn setup_autostart(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let (key, _) = hkcu.create_subkey(path)?;
        
        let app_path = std::env::current_exe()?.to_string_lossy().to_string();
        key.set_value("ShimmenLanSuite", &format!("\"{}\" --minimized", app_path))?;
    }
    Ok(())
}

pub fn remove_autostart() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let key = hkcu.open_subkey_with_flags(path, KEY_WRITE)?;
        key.delete_value("ShimmenLanSuite")?;
    }
    Ok(())
}
```

3. **修改 `src-tauri/Cargo.toml` 添加 winreg**:
```toml
winreg = "0.52"
```

4. **修改 `src-tauri/src/main.rs` 集成托盘**:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod commands;
mod db;
mod models;
mod network;
mod sync;
mod file_sync;
mod system;
mod config;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_pool = db::init_db(&app_dir)?;
            app.manage(db_pool);
            
            system::tray::setup_tray(app.app_handle())?;
            
            // 检查是否带 --minimized 参数启动
            let args: Vec<String> = std::env::args().collect();
            if args.contains(&"--minimized".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

5. **运行验证**:
```bash
cargo tauri dev
```
预期：应用启动后右下角出现托盘图标，点击可隐藏/显示窗口

---

## Phase 5: 用户列表与文件传输窗口

### Task 5.1: 前端用户列表 + 传输窗口 UI

**Files**: `src/components/user/UserList.tsx`, `src/components/user/UserCard.tsx`, `src/components/chat/TransferWindow.tsx`
**Why**: 核心功能 F1 + F2
**Impact**: 前端状态管理雏形
**Verification**: 能看到模拟用户列表，点击弹出传输窗口

**Steps**:

1. **创建 `src/components/user/UserCard.tsx`**:
```tsx
import { Monitor, Circle } from 'lucide-react';
import type { User } from '../../types';

interface UserCardProps {
  user: User;
  onClick: () => void;
}

export default function UserCard({ user, onClick }: UserCardProps) {
  return (
    <button
      onClick={onClick}
      className="w-full flex items-center gap-3 p-3 rounded-xl hover:bg-white hover:shadow-sm transition-all text-left group"
    >
      <div className="relative">
        <div className="w-10 h-10 bg-gradient-to-br from-cyan-500 to-blue-600 rounded-full flex items-center justify-center text-white font-medium text-sm">
          {user.username.slice(0, 2)}
        </div>
        <Circle
          size={10}
          className={`absolute -bottom-0.5 -right-0.5 fill-current ${
            user.status === 'online' ? 'text-green-500' : 'text-gray-400'
          }`}
          strokeWidth={0}
        />
      </div>
      <div className="flex-1 min-w-0">
        <div className="font-medium text-gray-800 text-sm truncate">{user.username}</div>
        <div className="text-xs text-gray-400 flex items-center gap-1">
          <Monitor size={10} />
          {user.ip}
        </div>
      </div>
      <div className="text-[10px] text-gray-300 opacity-0 group-hover:opacity-100 transition-opacity">
        v{user.version}
      </div>
    </button>
  );
}
```

2. **创建 `src/components/user/UserList.tsx`**:
```tsx
import { useState } from 'react';
import { Users, RefreshCw } from 'lucide-react';
import UserCard from './UserCard';
import TransferWindow from '../chat/TransferWindow';
import type { User } from '../../types';

const mockUsers: User[] = [
  { id: '1', username: '张三', ip: '192.168.1.10', status: 'online', version: '0.1.0' },
  { id: '2', username: '李四', ip: '192.168.1.11', status: 'online', version: '0.1.0' },
  { id: '3', username: '王五', ip: '192.168.1.13', status: 'offline', version: '0.1.0' },
];

export default function UserList() {
  const [selectedUser, setSelectedUser] = useState<User | null>(null);

  return (
    <div className="h-full flex">
      <div className="w-72 bg-gray-50 border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h2 className="font-bold text-gray-800 flex items-center gap-2">
              <Users size={18} className="text-cyan-600" />
              在线用户
            </h2>
            <button className="p-1.5 text-gray-400 hover:text-cyan-600 hover:bg-cyan-50 rounded-lg transition-colors">
              <RefreshCw size={14} />
            </button>
          </div>
          <p className="text-xs text-gray-400 mt-1">点击用户开始传输文件</p>
        </div>
        <div className="flex-1 overflow-auto p-2 space-y-1">
          {mockUsers.map((user) => (
            <UserCard key={user.id} user={user} onClick={() => setSelectedUser(user)} />
          ))}
        </div>
      </div>
      
      {selectedUser && (
        <TransferWindow
          user={selectedUser}
          onClose={() => setSelectedUser(null)}
        />
      )}
    </div>
  );
}
```

3. **创建 `src/components/chat/TransferWindow.tsx`**:
```tsx
import { useState, useRef } from 'react';
import { X, Send, Paperclip, Image, FolderUp, Download } from 'lucide-react';
import type { User } from '../../types';

interface TransferWindowProps {
  user: User;
  onClose: () => void;
}

interface TransferRecord {
  id: string;
  type: 'sent' | 'received';
  fileName: string;
  fileSize: string;
  progress: number;
  status: 'pending' | 'transferring' | 'completed' | 'failed';
}

export default function TransferWindow({ user, onClose }: TransferWindowProps) {
  const [records, setRecords] = useState<TransferRecord[]>([
    { id: '1', type: 'received', fileName: '项目方案.docx', fileSize: '2.4 MB', progress: 100, status: 'completed' },
    { id: '2', type: 'sent', fileName: '截图_01.png', fileSize: '1.1 MB', progress: 100, status: 'completed' },
  ]);
  const fileInputRef = useRef<HTMLInputElement>(null);

  return (
    <div className="flex-1 flex flex-col bg-white">
      <div className="h-14 border-b border-gray-100 flex items-center justify-between px-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 bg-gradient-to-br from-cyan-500 to-blue-600 rounded-full flex items-center justify-center text-white text-xs font-medium">
            {user.username.slice(0, 2)}
          </div>
          <div>
            <div className="font-medium text-sm text-gray-800">{user.username}</div>
            <div className="text-xs text-gray-400">{user.ip}</div>
          </div>
        </div>
        <button onClick={onClose} className="p-1.5 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors">
          <X size={18} />
        </button>
      </div>
      
      <div className="flex-1 overflow-auto p-4 space-y-3">
        {records.map((record) => (
          <div
            key={record.id}
            className={`flex items-start gap-3 ${record.type === 'sent' ? 'flex-row-reverse' : ''}`}
          >
            <div className={`max-w-[70%] p-3 rounded-2xl ${
              record.type === 'sent'
                ? 'bg-cyan-600 text-white rounded-tr-md'
                : 'bg-gray-100 text-gray-800 rounded-tl-md'
            }`}>
              <div className="flex items-center gap-2 mb-1">
                <FolderUp size={14} className={record.type === 'sent' ? 'text-cyan-200' : 'text-gray-400'} />
                <span className="text-sm font-medium">{record.fileName}</span>
              </div>
              <div className="text-xs opacity-75">{record.fileSize}</div>
              {record.status === 'transferring' && (
                <div className="mt-2 h-1 bg-black/10 rounded-full overflow-hidden">
                  <div className="h-full bg-white/80 rounded-full transition-all" style={{ width: `${record.progress}%` }} />
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
      
      <div className="p-3 border-t border-gray-100">
        <div className="flex items-center gap-2">
          <button className="p-2 text-gray-400 hover:text-cyan-600 hover:bg-cyan-50 rounded-lg transition-colors">
            <Paperclip size={18} />
          </button>
          <button className="p-2 text-gray-400 hover:text-cyan-600 hover:bg-cyan-50 rounded-lg transition-colors">
            <Image size={18} />
          </button>
          <input
            type="text"
            placeholder="拖拽文件到此处，或输入消息..."
            className="flex-1 px-3 py-2 bg-gray-50 border-0 rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-500/20 placeholder:text-gray-400"
          />
          <button className="p-2 bg-cyan-600 text-white rounded-lg hover:bg-cyan-700 transition-colors">
            <Send size={16} />
          </button>
        </div>
      </div>
    </div>
  );
}
```

4. **修改 `MainContent.tsx` 集成用户列表**:
```tsx
import { useState } from 'react';
import Sidebar from './Sidebar';
import TitleBar from './TitleBar';
import UserList from '../user/UserList';

export default function MainContent() {
  const [activeTab, setActiveTab] = useState('users');

  const renderContent = () => {
    switch (activeTab) {
      case 'users':
        return <UserList />;
      default:
        return <div className="p-6 text-2xl font-bold text-gray-800">{activeTab}</div>;
    }
  };

  return (
    <div className="h-screen w-screen flex flex-col bg-gray-50 overflow-hidden">
      <TitleBar username="水门" ip="192.168.1.12" onlineCount={5} />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
        {renderContent()}
      </div>
    </div>
  );
}
```

5. **运行验证**:
```bash
npm run dev
```
预期：左侧用户列表显示张三/李四/王五，点击张三右侧弹出传输窗口，类似微信聊天界面

---

## Phase 6-9 概要

由于完整计划篇幅极长，Phase 6-9 的核心任务在这里以紧凑形式列出，执行时每个 Task 会展开为完整的代码和验证步骤：

### Phase 6: 共享聊天窗（F3）
- **Task 6.1**: 前端聊天窗 UI（消息气泡、清屏按钮、文件/图片预览）
- **Task 6.2**: Rust 端群播协议（一条消息广播给所有已知 peers）
- **Task 6.3**: 清屏指令同步（ClearScreen 广播，所有客户端执行）
- **Task 6.4**: 聊天文件处理（接收文件保存到下载目录，发送文件走 P2P 传输通道）

### Phase 7: 共享文件夹同步（F4）
- **Task 7.1**: 前端共享文件夹浏览器（树形目录、文件列表、订阅按钮）
- **Task 7.2**: Rust 文件系统监控（ReadDirectoryChangesW 封装）
- **Task 7.3**: 文件索引与哈希（Blake3 块级哈希，生成文件指纹）
- **Task 7.4**: 块级增量同步协议（对比哈希列表，只传差异块）
- **Task 7.5**: 冲突解决（最后写入胜出 + 冲突副本 `文件名 (冲突 来自 用户名).ext`）
- **Task 7.6**: 15 天历史快照（文件修改前复制到 `.shimmen_history/` 目录，定时清理）
- **Task 7.7**: 同步引擎调度（订阅关系管理、上传/下载队列、错误重试）

### Phase 8: 共享密码栏（F5）+ 共享公告栏（F6）
- **Task 8.1**: 前端密码栏 UI（表格、新增/编辑/删除、密码显示切换）
- **Task 8.2**: 前端公告栏 UI（富文本编辑器、置顶、列表）
- **Task 8.3**: Rust 向量时钟实现（VClock 结构体、合并规则、冲突检测）
- **Task 8.4**: 共享状态同步协议（StateSync 消息：table + data + version）
- **Task 8.5**: 本地 AES-256 加密（密码栏落盘加密，密钥派生自机器指纹）

### Phase 9: 个人看板（F7）+ 团队看板（F8）
- **Task 9.1**: 前端看板 UI（三列布局、@dnd-kit 拖拽、任务卡片）
- **Task 9.2**: 任务 CRUD（创建、编辑、删除、状态变更）
- **Task 9.3**: 团队看板视图（按项目分组、过滤、认领按钮）
- **Task 9.4**: 看板状态同步（任务变更实时广播，向量时钟解决冲突）
- **Task 9.5**: 归档到共享文件夹（完成后自动创建文件夹并复制附件）

### Phase 10: 收尾
- **Task 10.1**: 版本检测（发现节点时对比版本号，不一致弹提示）
- **Task 10.2**: 前端 Tauri 命令绑定（所有 Rust 命令通过 invoke 暴露给 React）
- **Task 10.3**: 配置持久化（用户名、自启开关、下载路径保存到 confy）
- **Task 10.4**: Tauri 打包配置（Portable 模式、NSIS 安装包可选、图标替换）
- **Task 10.5**: 双机联调测试（同网段两台 Windows 机器完整功能验证）

---

## 执行选项

**Plan complete and saved to `docs/aegis/plans/2026-05-25-lan-suite-v1-implementation.md`.**

**Two execution options:**

**1. Subagent-Driven (recommended)** — 我为每个 Task  dispatch 一个专门的子代理执行，每完成一个 Task 我 review 代码质量，确保不偏离设计，快速迭代

**2. Inline Execution** — 我在当前会话中直接按顺序执行 Task，批量写代码，关键节点给你看效果

**Which approach do you prefer?**
