# 水门内网协同 (Shimmen LAN Suite)

一款基于 Tauri 构建的**内网星型拓扑协同工具**。无需服务器，在同一局域网内自动发现团队成员，自动选举中心节点（Leader）转发消息，实现即时通讯、文件传输、跨机文件搜索、屏幕共享等功能。

> ⚡ 零配置、零服务器、零公网依赖。下载即连，开机即用。
> 
> 🔒 五轮 Aegis 安全审计，累计修复 31 个安全与稳定性问题。

---

## ✨ 功能一览

### 🤝 核心协同
| 功能 | 说明 |
|------|------|
| **即时通讯** | 内网聊天，支持文字、文件附件、表情包 |
| **文件传输** | 大文件分块传输（256KB/块），不经过中转服务器 |
| **文件夹同步** | 指定共享文件夹，实时双向同步 |
| **跨机文件搜索** 🔥 | 搜索整个内网所有在线电脑的文件，一键请求传输 |
| **屏幕共享** | 一键分享屏幕画面给内网其他成员 |
| **公告白板** | 团队公告 + 便签式任务看板 |
| **共享密码栏** | 团队共享密码（本地 AES-GCM 加密，不网络同步）|

### 🧰 工具箱（9 款实用工具）
| 工具 | 功能 |
|------|------|
| DNS 优选 | 测试并自动切换到延迟最低的 DNS |
| Win10/11 激活 | 一键激活 Windows |
| 依赖检测 | 检查运行环境缺失项 |
| PS7 & UTF8 | 一键安装 PowerShell 7 并设置 UTF-8 |
| 系统优化 | Windows 常用优化项一键执行 |
| 文件查询 | 本地文件快速搜索 |
| 批量重命名 | 支持正则的批量文件重命名 |
| 打印机管理 | 打印机状态查看与管理 |
| 网络信息 | 本机网络配置一览 |

---

## 🏗️ 技术栈

- **前端**：React 18 + TypeScript + Tailwind CSS + Vite
- **桌面框架**：Tauri v2 (Rust)
- **数据库**：SQLite (rusqlite)
- **网络层**：
  - **星型拓扑**：`device_id` 字典序最小者自动当选为 Leader（中心节点）
  - **UDP 发现**：端口 23333，自动维护在线节点列表
  - **TCP 通信**：端口 23334，长度前缀 JSON 协议
  - **消息转发**：非 Leader 的消息经 Leader `Relay` 转发给所有 Peer
  - **故障转移**：Leader 离线后 8 秒内自动重新选举 + 3 秒内重连
- **构建产物**：单文件 NSIS 安装包 / 便携版 `.exe`

---

## 🚀 快速开始

### 环境要求
- [Node.js](https://nodejs.org/) ≥ 18
- [Rust](https://rustup.rs/) ≥ 1.75
- Windows 10/11

### 开发
```bash
# 安装前端依赖
npm install

# 启动开发服务器（同时启动前端 Vite + Tauri 开发模式）
npm run tauri dev
```

### 构建 Release
```bash
# 生成 NSIS 安装包 + 便携版 exe
npx tauri build
```

构建产物：
- 便携版：`src-tauri/target/release/shimmen-lan-suite.exe`
- 安装包：`src-tauri/target/release/bundle/nsis/水门内网协同_0.1.4_x64-setup.exe`

---

## 📁 项目结构

```
.
├── src/                          # 前端源码 (React/TS)
│   ├── components/
│   │   ├── chat/                 # 聊天模块
│   │   ├── board/                # 公告白板 + 任务看板
│   │   ├── folder/               # 文件夹同步
│   │   ├── screen/               # 屏幕共享
│   │   ├── tools/                # 工具箱 (9 个工具)
│   │   ├── settings/             # 设置页
│   │   └── icons/                # SVG 图标集合
│   ├── App.tsx                   # 主应用 + 路由
│   └── main.tsx                  # 入口
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # 应用入口 (setup + 启动网络服务)
│   │   ├── lib.rs                # Lib crate 入口 (模块声明)
│   │   ├── commands/             # Tauri 前端命令 (30+ 个)
│   │   ├── network/              # 网络层 (发现/连接/协议/Leader 选举)
│   │   ├── db/                   # SQLite 数据库 + Schema
│   │   ├── file_index/           # 跨机文件索引模块
│   │   ├── file_sync/            # 文件夹同步引擎
│   │   ├── models/               # 数据模型 + NetworkMessage 枚举
│   │   ├── system/               # 托盘/自启动/加密
│   │   └── config.rs             # 配置管理
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
└── AGENTS.md                     # 👈 AI 开发者必读指南
```

---

## 🔒 隐私与安全

- **纯内网通信**：所有数据仅在局域网内传输，不连接任何公网服务器。
- **星型拓扑中继**：非 Leader 之间的消息经 Leader 转发（`Relay` 包装），业务层无感知。
- **文件索引边界**：跨机搜索仅索引文件名和路径，不读取文件内容；远程索引存储在本地 SQLite，可一键清除。
- **密码加密**：共享密码使用 AES-256-GCM 加密，随机 nonce，密钥由设备 ID 派生。**密码绝不通过网络同步**。
- **无账号体系**：基于设备 ID 自动识别，无需注册登录。
- **安全审计**：经过五轮 Aegis 端到端安全审计，累计修复 31 个安全与稳定性问题。

---

## ⚠️ 已知限制

| 限制 | 说明 |
|------|------|
| 点对点文件传输广播 | `FileChunk` 无 `target_peer_id`，经 Leader 转发给所有 Peer（办公场景可接受）|
| 新设备历史数据 | 新加入的设备无法自动获取已有的 tasks/announcements（需等待后续变动触发同步）|
| 文件同步冲突 | 多设备同时修改同一文件时，后到达者覆盖（无冲突合并机制）|
| Schema 迁移 | 暂无数据库版本迁移逻辑，升级时可能需要删除旧数据库 |
| 最大设备数 | Leader 端 TCP 连接上限 128，超过后新设备无法接入 |

---

## 📄 License

MIT
