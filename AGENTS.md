# Agent 开发指南

> 本文件面向接手此项目的 AI 助手 / 开发者。阅读此文件可帮助你快速理解项目结构、避免常见陷阱。

---

## 1. 项目概述

- **名称**：水门内网协同 (shimmen-lan-suite)
- **类型**：Tauri v2 桌面应用（Windows）
- **当前版本**：v0.1.5
- **核心特性**：星型拓扑内网协同，自动 Leader 选举，零服务器，内网自发现
- **GitHub**：`575674384-stack/shimmen-lan-suite`

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

### 2.3 星型拓扑核心规则（切勿违反）

- `device_id` 字典序最小者自动当选为 **Leader**
- **非 Leader 只维护一条到 Leader 的 TCP 连接**
- **Leader 维护到所有 Peer 的连接**
- 非 Leader → 非 Leader 的消息必须经 Leader `Relay` 转发
- Leader 离线后 8 秒内自动重新选举（read timeout）+ 3 秒内重连
- 业务层（commands）调用 `broadcast_message` 无需感知拓扑变化

### 2.4 消息转发路径

```
非 Leader A 发送消息
  → A 的 broadcast_message 只发给 Leader
  → Leader 收到后包装为 Relay { origin_peer_id: "A", payload: msg }
  → Leader 转发给所有其他 Peer
  → Peer B 收到 Relay，解包后用 origin_peer_id 调用 process_message
```

**来源可信**：`process_message` 中的 `peer_id` 来自 TCP handshake，无法伪造。`Relay` 解包后的 `origin_peer_id` 继承该可信 ID。

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
4. **如果消息是点对点意图**：Leader 转发逻辑默认会广播给所有人。如需精确点对点，需在消息中添加 `target_peer_id` 字段，并修改 Leader 转发逻辑。

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
| UDP 发现 | `23333` | 广播心跳包，维护在线节点列表，12 秒超时清理 |
| TCP 控制 | `23334` | 长度前缀 JSON 协议，所有业务消息 |
| 消息格式 | `4字节长度(u32 BE) + JSON` | 见 `network/connection.rs` |
| 最大消息 | 50 MB | 超过则断开连接 |
| Read Timeout | 8 秒 | 检测死连接/Leader 离线 |
| Write Timeout | 8 秒 | 防止半开连接无限阻塞 |
| 最大连接数 | 128 | Leader 服务端软限制 |

**现有消息类型（NetworkMessage）：**
- `ChatMessage` / `ClearScreen`
- `StateSync { table, data, version }` — tasks / announcements / shared_folders / ai_config
- `FileList` / `FileRequest` / `FileResponse` / `FileChunk`
- `ScreenShare`
- `FileIndexBroadcast` / `FileIndexRequest` / `FileSearchRequest` / `FileSearchResponse` / `FileTransferRequest`
- `Heartbeat`
- `Relay { origin_peer_id, payload }` — Leader 转发包装

### 4.1 文件分块传输

- `CHUNK_SIZE = 256KB`
- 发送：`send_file_in_chunks`（点对点）/ `broadcast_file_in_chunks`（广播）
- 接收：`FileChunk` 按 `peer_id:folder_id:file_path` key 聚合到 `.part` 临时文件
- 收齐后 `rename` `.part` → 最终文件（避免多 peer 写竞争）
- 连接断开时自动清理未完成传输（`cleanup_peer_chunks`）

---

## 5. 数据库

- **引擎**：SQLite (bundled)
- **连接池**：`DbPool = Arc<Mutex<Connection>>`（单连接串行化，简化但已够用）
- **Schema**：`src-tauri/src/db/mod.rs`（内联创建，无独立 schema 文件）
- ⚠️ **无 Schema 版本迁移逻辑**。新增列/改约束需手动处理或提示用户删除旧数据库。

**主要表：**
- `chat_messages` — 聊天记录（前端显示最近 200 条）
- `tasks` — 看板任务（`version` 列当前未使用，仅 `updated_at` 做冲突检查）
- `announcements` — 公告
- `password_entries` — 密码（**绝不网络同步**，本地 AES-GCM 加密）
- `shared_folders` — 共享文件夹配置
- `file_index` — 跨机文件索引（本地表，`AUTOINCREMENT`）
- `users` — 预留表，当前未实际使用

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

**发版 checklist（必须严格执行）：**
1. **同步修改三个文件的版本号**（缺一不可，否则更新检测会异常）：
   - `package.json` → `"version": "x.y.z"`
   - `src-tauri/Cargo.toml` → `version = "x.y.z"`
   - `src-tauri/tauri.conf.json` → `"version": "x.y.z"`
2. `npx tauri build`
3. 在 GitHub 创建 Release，tag 格式 `vx.y.z`
4. 上传两个 asset：
   - **Portable**：`src-tauri/target/release/shimmen-lan-suite.exe` → 重命名为 `shimmen-lan-suite-vx.y.z-portable.exe`
   - **Setup**：`src-tauri/target/release/bundle/nsis/水门内网协同_x.y.z_x64-setup.exe` → 重命名为 `shimmen-lan-suite-vx.y.z-setup.exe`（asset 名必须以 `setup.exe` 结尾，否则更新检测匹配不到）

> ⚠️ **版本号同步是强制的**：`Cargo.toml` 的版本会编译进二进制，`tauri.conf.json` 的版本用于 Tauri 内部，`package.json` 的版本用于前端显示。任何一处不同步都会导致更新检测或关于页版本显示错误。

---

## 7. 构建与验证

```bash
# 前端类型检查
cd src && npx tsc --noEmit

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
- **时间戳统一**：前端 `Math.floor(Date.now() / 1000)`（秒），后端 `chrono::Utc::now().timestamp()`（秒），显示时 `* 1000`

---

## 9. 特别注意事项

- **base64**：使用 `base64::engine::general_purpose::STANDARD.encode(content)`，需 `use base64::{engine::general_purpose::STANDARD, Engine as _};`
- **Tauri Emitter**：`app_handle.emit()` 需要 `use tauri::Emitter;`
- **文件传输复用**：跨机搜索的文件传输通过构造 `FileTransferRequest` → 服务端响应 `FileResponse`（base64），复用现有文件接收逻辑保存到 downloads 目录
- **WebView2 兼容**：启动时检测 WebView2 运行时，NSIS 安装包已嵌入 `embedBootstrapper`
- **自启动**：支持 `--minimized` 参数静默启动，配合注册表实现开机自启。设置面板提供开关控制
- **密码加密**：AES-256-GCM，随机 nonce（12 字节），格式 `base64(nonce || ciphertext)`。`decrypt_password` 向后兼容旧格式（固定 nonce `b"shimmen-12!!"`）
- **路径安全**：`is_path_safe()` 检查 `..` 和绝对路径。Windows 设备名（CON/PRN/NUL 等）当前未过滤。

---

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
| `screen_fps` | 屏幕分享帧率 | 10 |
| `screen_resolution` | 屏幕分享分辨率 | 720 |
| `auto_update` | 自动检查更新 | true |

新增配置命令：
- `set_download_dir` / `set_sync_interval` / `set_autostart` / `get_autostart_status`
- `set_screen_fps` / `set_screen_resolution` / `set_auto_update`

---

## 11. Aegis 审查历史

### Round 1 — 基础安全 + 架构重构（v0.1.3 前）
- 明文广播密码 → 设备 ID 派生密钥加密
- ChatMessage sender_id 伪造 → TCP peer_id 作为可信来源
- ai_config 远程覆盖 → 直接拒绝
- 任意文件读取 (`../../`) → 路径遍历检查
- TcpStream 并发写入 → `Arc<Mutex<TcpStream>>`
- FolderWatcher CPU 100% → `RecvError` 后 break
- DbPool 锁跨 I/O → 先 drop 锁再执行网络操作
- SyncEngine 线程泄漏 → `AtomicBool` 控制退出
- 前端时间戳毫秒/秒混乱 → 统一为秒
- **架构重构**：P2P → 星型拓扑，自动 Leader 选举，Relay 转发

### Round 2 — 网络拓扑修复（v0.1.3 → v0.1.4）
- `pending` 设计缺陷 → Leader 断开后无法重连 → 移除 `pending`，直接检查 pool
- `Relay` 嵌套栈溢出 → 拒绝嵌套 Relay（两处防护）

### Round 3 — 数据库并发 + 线程安全（v0.1.4）
- `archive_task` 长时间持锁 → 文件IO前释放锁
- `chat.rs` 静默失败 → 锁 poison 时返回错误
- Leader 转发带宽浪费 → 排除 `FileIndexRequest`/`FileSearchRequest`
- `FileIndexBroadcast` 伪造 → 验证 sender_id == peer_id
- `password-saved` 前端未监听 → PasswordVault 自动刷新
- `active_count` panic 漂移 → `catch_unwind` 确保计数器递减
- `screen_share` 状态卡死 → `catch_unwind` 确保 SHARING 重置
- `chunk_receives` 断线泄漏 → 连接断开时清理未完成传输

### Round 4 — 密码学 + 网络安全 + 数据一致性（v0.1.4）
- AES-GCM 固定 nonce → 随机 nonce（`nonce || ciphertext` 格式）
- `send_message` 无 write timeout → 8 秒写超时
- `FileChunk` 路径遍历 → `..` / 绝对路径拒绝
- `FileChunk` 多 peer 写竞争 → `.part` 临时文件 + 收齐后 rename
- JSON 解析失败静默丢弃 → handshake 和 `process_message` 添加错误日志
- `StateSync` 无版本检查 → `updated_at` 比较拒绝旧版本覆盖

### Round 5 — 回归测试 + 前端完整性（v0.1.4）
- 旧密码无法解密 → `decrypt_password` 向后兼容旧格式（固定 nonce）
- Leader 断连震荡 → Peer 收到 Heartbeat 后发送 Heartbeat 回复
- `FileChunk` 缺少 `.truncate(true)` → 新传输时清空旧文件
- 前端 `network-message` 无差别 reload → `useTasks`/`AnnouncementBoard` 按 table 过滤
- 聊天记录无限增长 → 限制保留最近 500 条

### Round 6 — 日志系统 + 可观测性 + 用户可控更新（v0.1.5）
- 运行时故障无日志 → 引入 `tracing` + `tracing-appender`，按天轮转写入 `%APPDATA%\shimmen-lan-suite\shimmen.log`
- 聊天发送失败原因不明 → `chat.rs` 关键步骤加 `info!`/`error!`/`warn!` 日志，暴露 DB 锁/INSERT/广播各阶段状态
- 自动更新不可关闭 → `AppConfig` 新增 `auto_update`（默认 true），设置面板增加开关，`UpdatePrompt.tsx` 启动检查前读取配置

---

## 12. 已知限制（当前版本）

| # | 限制 | 影响 | 缓解措施 |
|---|------|------|---------|
| 1 | `FileChunk` 无 `target_peer_id` | 点对点文件传输经 Leader 广播给所有 peer | 办公场景可接受；恶意场景需扩展协议 |
| 2 | 新设备无法获取历史 tasks/announcements | 只能等待未来变动触发同步 | 手动刷新页面可重新加载本地 DB（但无网络历史）|
| 3 | 文件同步冲突处理未实现 | 多设备同时改同一文件，后到达者覆盖 | `updated_at` 比较已应用于 tasks/announcements；文件同步仍缺 |
| 4 | 无优雅退出机制 | `.part` 文件可能残留 | 连接断开时 `cleanup_peer_chunks` 清理；应用强杀时无法保证 |
| 5 | 无 Schema 迁移逻辑 | 后续版本新增列需手动处理 | `CREATE TABLE IF NOT EXISTS` 保证新安装正常；升级需删除旧 DB |
| 6 | Windows 保留设备名未过滤 | `NUL`/`CON` 等特殊文件名可能打开系统设备 | `is_path_safe` 已检查 `..` 和绝对路径；设备名过滤待补充 |
| 7 | 未使用函数警告 | 7 个预留接口 | 不影响运行 |

---

## 13. 关键代码路径速查

```
启动流程（main.rs setup）：
  1. 创建 app_dir
  2. init_db → DbPool
  3. manage(DbPool)
  4. setup_tray
  5. create_peer_map → manage(PeerMap)
  6. create_connection_pool → manage(ConnectionPool)
  7. create_folder_cache → manage(RemoteFolderCache)
  8. spawn start_server (TCP)
  9. spawn start_discovery (UDP, delay 1s)
  10. create SyncEngine → manage(SyncEngine)
  11. spawn 恢复监控已同步文件夹
  12. spawn 广播共享文件夹 (delay 2s)
  13. spawn 文件索引扫描 + 广播 (delay 5s)
  14. spawn 重连线程 (loop 3s)

网络消息处理（server.rs）：
  handle_incoming → read handshake → insert pool → read loop → process_message
  process_message → Leader 转发 → match msg → 各分支处理

Leader 选举（leader.rs）：
  elect_leader = online peers + my_id → sort → 最小 ID
  ensure_leader = 重新计算并设置
  should_connect_to = 非 Leader 只连接 Leader
```
