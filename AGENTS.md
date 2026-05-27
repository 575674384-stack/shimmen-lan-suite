# Agent 开发指南

> 本文件面向接手此项目的 AI 助手 / 开发者。阅读此文件可帮助你快速理解项目结构、避免常见陷阱。

---

## 1. 项目概述

- **名称**：水门内网协同 (shimmen-lan-suite)
- **类型**：Tauri v2 桌面应用（Windows）
- **核心特性**：P2P 去中心化，零服务器，内网自发现

---

## 2. 关键架构陷阱 ⚠️

### 2.1 双入口模块系统（最容易踩坑！）

本项目同时存在 `main.rs` 和 `lib.rs`。这意味着：

```
❌ 错误：只在 main.rs 顶部写 mod xxx;
✅ 正确：新增模块必须在 lib.rs 中声明，才能被 lib crate 访问
```

**规则：**
- `main.rs`：用于应用启动逻辑（setup、线程启动、manage state）
- `lib.rs`：用于声明所有模块，供 `crate::xxx` 路径引用
- **任何新增模块，都必须在 `lib.rs` 中写 `pub mod xxx;`**

示例错误：`error[E0433]: cannot find file_index in crate`

### 2.2 模块路径约定

```rust
// lib.rs 中声明
pub mod file_index;

// 其他文件中使用
use crate::file_index::indexer::scan_directories;
```

---

## 3. 如何新增一个功能

### 3.1 新增后端命令（最常用）

1. 在 `src-tauri/src/commands/` 下新建（或修改现有）`.rs` 文件
2. 在 `src-tauri/src/commands/mod.rs` 中 `pub mod xxx;`
3. 在 `src-tauri/src/main.rs` 的 `invoke_handler` 中注册命令
4. 确保任何新增的内部模块也在 `lib.rs` 中声明

### 3.2 新增网络消息类型

1. 在 `src-tauri/src/models/mod.rs` 的 `NetworkMessage` 枚举中添加变体
2. 在 `src-tauri/src/network/server.rs` 的 `process_message` 中匹配处理
3. 在 `src-tauri/src/network/client.rs` 中如需要可添加发送辅助函数

### 3.3 新增工具箱工具

1. 在 `src/components/tools/` 下新建 `XxxTool.tsx`
2. 在 `src/components/tools/ToolsPanel.tsx` 的 `tools` 数组中添加卡片
3. 在 `src/components/icons/AppIcons.tsx` 中新建 SVG 图标（或直接用 Lucide）
4. 如需后端支持，按 3.1 添加命令

### 3.4 新增侧边栏 Tab

1. 在 `src/App.tsx` 的 `tabs` 数组中添加 `{ id: 'xxx', label: 'xxx', icon: ... }`
2. 在 `switch(activeTab)` 分支中添加渲染逻辑
3. 窗口尺寸参考：主窗口 `1280×860`，最小 `900×600`

---

## 4. 网络层速查

| 组件 | 端口/方式 | 说明 |
|------|-----------|------|
| UDP 发现 | `23333` | 广播心跳包，维护在线节点列表 |
| TCP 通信 | `23334` | 长度前缀 JSON 协议，所有业务消息 |
| 消息格式 | `4字节长度 + JSON` | 见 `network/protocol.rs` |

**现有消息类型（NetworkMessage）：**
- `Heartbeat` / `Discovery` / `PeerList`
- `ChatMessage` / `FileRequest` / `FileResponse`
- `FolderSync` / `FileList` / `FileContent`
- `ScreenFrame`
- `FileIndexBroadcast` / `FileSearchRequest` / `FileSearchResponse` / `FileTransferRequest`

---

## 5. 数据库

- **引擎**：SQLite (bundled)
- **连接池**：`r2d2_sqlite`
- **类型**：`DbPool = Arc<Mutex<Connection>>`（简化版，非多连接池）
- **Schema**：`src-tauri/src/db/schema.rs`

**主要表：**
- `users` - 本机用户信息
- `messages` - 聊天记录
- `tasks` - 看板任务
- `announcements` - 公告
- `shared_folders` - 共享文件夹配置
- `file_index` - 跨机文件索引
- `file_versions` - 文件夹同步版本历史

---

## 6. 在线更新（Updater）

采用**自定义 GitHub Release 检测方案**，未使用 Tauri 官方 updater 插件的签名机制。

**流程：**
1. 启动 15 秒后，前端自动调用 `check_update` 命令
2. 后端通过 GitHub API (`repos/.../releases/latest`) 获取最新 release
3. 比较版本号（semver），有新版本则弹出提示条
4. 用户点击"立即更新" → `download_and_install` 后台下载 exe
5. 下载完成后创建自删除批处理脚本，启动安装程序，应用自动退出

**相关文件：**
- `src-tauri/src/commands/update.rs` - 更新检查 / 下载安装 / 退出命令
- `src/components/common/UpdatePrompt.tsx` - 前端更新提示条组件

**发版 checklist：**
1. 同时修改三个文件的版本号：
   - `package.json` → `"version": "x.y.z"`
   - `src-tauri/Cargo.toml` → `version = "x.y.z"`
   - `src-tauri/tauri.conf.json` → `"version": "x.y.z"`
2. `npx tauri build`
3. 在 GitHub 创建 Release，tag 格式 `vx.y.z`
4. 上传两个 asset：
   - **Portable**：`src-tauri/target/release/shimmen-lan-suite.exe` → 重命名为 `shimmen-lan-suite-vx.y.z-portable.exe`
   - **Setup**：`src-tauri/target/release/bundle/nsis/水门内网协同_x.y.z_x64-setup.exe` → 重命名为 `shimmen-lan-suite-vx.y.z-setup.exe`（asset 名必须以 `setup.exe` 结尾，否则更新检测匹配不到）

---

## 7. 构建与验证

```bash
# 前端类型检查
npx tsc --noEmit

# Rust 开发检查
cd src-tauri && cargo check

# 完整开发运行
npm run tauri dev

# Release 构建
npx tauri build
```

**注意**：不要直接运行 `cargo tauri build`，项目中未全局安装 `cargo-tauri` CLI，应使用 `npx tauri build`。

---

## 8. 编码规范

- **Rust**：使用 `Result<T, String>` 作为命令返回类型；错误用 `.map_err(|e| e.to_string())?`
- **TypeScript**：严格模式开启；未使用变量/导入会导致构建失败
- **前端状态**：React `useState` + 少量 `useCallback`；复杂状态暂未引入全局管理
- **图标**：优先使用 `lucide-react`；自定义图标放在 `AppIcons.tsx`，支持渐变色 SVG
- **样式**：Tailwind CSS 优先；无边框窗口（`decorations: false`），自定义标题栏

---

## 9. 特别注意事项

- **base64**：使用 `base64::engine::general_purpose::STANDARD.encode(content)`，需 `use base64::{engine::general_purpose::STANDARD, Engine as _};`
- **Tauri Emitter**：`app_handle.emit()` 需要 `use tauri::Emitter;`
- **文件传输复用**：跨机搜索的文件传输通过构造 `FileTransferRequest` → 服务端响应 `FileResponse`（base64），复用现有文件接收逻辑保存到 downloads 目录
- **WebView2 兼容**：启动时检测 WebView2 运行时，NSIS 安装包已嵌入 `embedBootstrapper`
- **自启动**：支持 `--minimized` 参数静默启动，配合注册表实现开机自启。设置面板提供开关控制

## 10. 配置项清单

`AppConfig`（通过 confy 持久化）包含以下字段：

| 字段 | 说明 | 默认值 |
|------|------|--------|
| `username` | 显示用户名 | 系统用户名 |
| `device_id` | 设备唯一标识 | UUID |
| `avatar_preset` | 头像预设 | 空 |
| `download_dir` | 文件接收保存路径 | 空（fallback 到 app_data_dir/downloads）|
| `sync_interval_secs` | 共享文件夹同步间隔 | 0（实时）|
| `autostart` | 开机自启 | false |

新增配置命令：
- `set_download_dir` / `set_sync_interval` / `set_autostart` / `get_autostart_status`

## 11. 代码审查历史

### Round 4 (本轮)
- **chat.rs 锁内 I/O**：`send_chat_message` / `send_chat_file` / `clear_chat_screen` 改为使用 `client::broadcast_message`（连接在锁外 clone，避免网络阻塞时卡住整个 pool）
- **screen_share.rs JPEG 兼容性**：截屏 `Rgba8` 先 `to_rgb8()` 再编码，修复 JPEG 编码器不支持 RGBA 的 panic
- **board.rs 保留 created_at**：`save_task` 先查询原有 `created_at`，`INSERT OR REPLACE` 时不再覆盖创建时间
- **file_index 大文件 OOM**：`indexer::index_folder` 分块读取（8MB chunks）计算 blake3 hash，避免一次性载入大文件到内存
- **冗余 clone 清理**：`file_index/network.rs` 中 `&str` 的无意义 `.clone()` 移除
- **discovery UDP 缓冲区**：1024 → 4096 字节，避免大包截断
- **config 写失败加日志**：`save_config` / `.device_id` 写入失败时 `eprintln` 告警，不再静默吞掉
- **共享文件夹定时同步**：`sync_interval_secs = 0` 启动实时 `FolderWatcher`；>0 时启动定时扫描线程，按间隔广播 `FileList`
- **截屏质量持久化**：`screen_fps` / `screen_resolution` 加入 `AppConfig`，设置面板支持 5~30 FPS 和 450p/540p/720p 选择
- **新增设置项**：文件接收路径、共享文件夹同步间隔、开机自启开关、截屏分享质量
