# 水门内网协同 (Shimmen LAN Suite)

一款基于 Tauri 构建的**内网去中心化协同工具**。无需服务器，纯 P2P 直连，在同一局域网内自动发现团队成员，实现即时通讯、文件传输、跨机文件搜索、屏幕共享等功能。

> ⚡ 零配置、零服务器、零公网依赖。下载即连，开机即用。

---

## ✨ 功能一览

### 🤝 核心协同
| 功能 | 说明 |
|------|------|
| **即时通讯** | 内网 P2P 聊天，支持文字、表情包、文件附件 |
| **文件传输** | 点对点直接收发文件，不经过任何中转服务器 |
| **文件夹同步** | 指定共享文件夹，实时双向同步 |
| **跨机文件搜索** 🔥 | 搜索整个内网所有在线电脑的文件，一键请求传输 |
| **屏幕共享** | 一键分享屏幕画面给内网其他成员 |
| **公告白板** | 团队公告 + 便签式任务看板 |

### 🧰 工具箱（9 款实用工具）
| 工具 | 功能 |
|------|------|
| DNS 优选 | 测试并自动切换到延迟最低的 DNS |
| Win10 激活 | 一键激活 Windows |
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
- **数据库**：SQLite (rusqlite + r2d2 连接池)
- **网络层**：原生 TCP (端口 23334) + UDP 发现 (端口 23333)，长度前缀 JSON 协议
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
- 安装包：`src-tauri/target/release/bundle/nsis/水门内网协同_0.1.0_x64-setup.exe`

---

## 📁 项目结构

```
.
├── src/                          # 前端源码 (React/TS)
│   ├── components/
│   │   ├── chat/                 # 聊天模块
│   │   ├── board/                # 公告白板
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
│   │   ├── commands/             # Tauri 前端命令 (20+ 个)
│   │   ├── network/              # P2P 网络层 (发现/连接/协议)
│   │   ├── db/                   # SQLite 数据库 + Schema
│   │   ├── file_index/           # 跨机文件索引模块
│   │   ├── file_sync/            # 文件夹同步引擎
│   │   ├── models/               # 数据模型 + NetworkMessage 枚举
│   │   ├── system/               # 托盘/自启动/加密
│   │   └── config.rs             # 配置管理
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

---

## 🔒 隐私说明

- **纯内网通信**：所有数据仅在局域网内点对点传输，不连接任何公网服务器。
- **文件索引边界**：跨机搜索仅索引文件名和路径，不读取文件内容；远程索引存储在本地 SQLite，可一键清除。
- **无账号体系**：基于设备 ID 自动识别，无需注册登录。

---

## 📄 License

MIT
