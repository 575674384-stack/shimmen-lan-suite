# Plan: Network Layer Bug Fix — P2P Communication Restoration

**Date**: 2026-05-27  
**Author**: Kimi Code CLI  
**Status**: Pending Approval  
**Route**: Subagent-Driven (recommended) — 13 tasks, cross-module network-layer repair

---

## TaskIntentDraft

- **Requested outcome**: Fix all P2P communication bugs so that file transfer, chat, shared folders, announcements, and board tasks work correctly across peers on the same LAN segment.
- **Goal**: Make the network layer reliable enough for day-to-day LAN collaboration.
- **Success evidence**:
  - Two instances on the same subnet can send chat messages and receive them in real time.
  - File transfer (`send_file_to_peer`) completes and the receiver gets a `file-received` event.
  - Creating a shared folder broadcasts it; other peers see it in `get_remote_shared_folders`.
  - Deleting an announcement / task on one machine removes it on all peers.
  - `cargo check` and `npx tsc --noEmit` pass.
- **Stop condition**: All 18 catalogued bugs are fixed, verified by compilation, and the plan has no remaining tasks.
- **Non-goals**:
  - Do NOT redesign the entire network architecture (keep length-prefix JSON over TCP).
  - Do NOT implement folder-sync bidirectional file sync (subscribe_shared_folder stays a stub; only the broadcast-on-create fix is in scope).
  - Do NOT add new features (clipboard sharing, screenshots, etc.).
  - Do NOT fix the updater plugin installation (not related to LAN communication).
- **Constraints**:
  - Maintain backward compatibility with existing `NetworkMessage` variants that are actively used.
  - Keep the `ConnectionPool = Arc<Mutex<HashMap<String, Connection>>>` type; do not refactor into channels or MPMC queues.
  - Windows-only target.
- **Scope**: Rust backend (`src-tauri/src/network/`, `src-tauri/src/commands/`, `src-tauri/src/config.rs`, `src-tauri/src/main.rs`) + TypeScript frontend callers (`src/hooks/`, `src/components/`).
- **Risk hints**:
  - The TCP connection model (inbound vs outbound) is the deepest root cause; fixing it without breaking discovery ordering is tricky.
  - Parameter-name mismatches are scattered across frontend hooks and backend commands; missing one leaves a feature still broken.
  - `send_file_to_peer` has a double-serialization bug layered on top of the param-name bug; both must be fixed together.
- **Route**: writing-plans → subagent-driven-development → verification-before-completion.
- **Next**: Execute plan after user approval.

---

## Plan Pressure Test

- **Owner / contract / retirement**: Network layer contract is owned by `network/` module. `ConnectionPool` is the source-of-truth for active TCP connections. Retirement: old double-connection model is replaced by a single-active-connection-per-peer model with read loops on both sides.
- **Verification scope**: Compile-time (`cargo check`, `tsc --noEmit`) + runtime cross-machine test (two instances on same subnet).
- **Task executability**: Each task is one file or one cross-file rename; all are 2–5 min inline edits. No external dependencies.
- **Pressure result**: proceed

---

## Architecture

- **Transport**: TCP port 23334, length-prefixed JSON (4-byte big-endian length header).
- **Discovery**: UDP broadcast port 23333, independent `DiscoveryPacket` / `HeartbeatPacket` structs (NOT `NetworkMessage`).
- **Connection model** (post-fix): Each peer pair maintains **one** active TCP connection. The side that accepts the inbound connection stores it in `ConnectionPool`. The side that initiated `connect_to_peer` also keeps the connection but **must run a `read_message` loop** so the other side can send on it. If a second connection for the same `peer_id` arrives, the older one is closed and replaced.
- **Concurrency**: `ConnectionPool` is `Arc<Mutex<HashMap<...>>>`. All I/O happens **after** releasing the lock (clone `Connection` out of the map, drop the lock, then `send_message`).
- **Identity**: `peer_id` is the device UUID from `config.rs`. TCP handshake sends `{"peer_id":"..."}`; server uses this to key the connection in the pool.

## Tech Stack

- Tauri v2 (Rust 1.75+)
- React 18 + TypeScript + Tailwind
- SQLite (rusqlite)

## Baseline / Authority Refs

- `docs/aegis/specs/2026-05-25-lan-collaboration-suite-brief.md` — original feature brief
- `AGENTS.md` — coding conventions (module system, `Result<T,String>`, Emitter import)
- `src-tauri/src/network/protocol.rs` — discovery constants

## Compatibility Boundary

- `NetworkMessage` enum: keep all **active** variants unchanged. Only remove the 3 dead variants (`Discovery`, `DiscoveryResponse`, `Heartbeat`).
- `ConnectionPool` type alias must stay the same; internal usage changes only.
- `DiscoveryPacket` struct and UDP broadcast format must stay unchanged (peers may run old versions).
- Frontend invoke signatures must match backend command parameter names (this is the fix, not a breaking change).

## Verification

- `cd src-tauri && cargo check` — zero errors
- `npx tsc --noEmit` — zero errors
- Two-machine test: start two instances, verify chat + file transfer + folder broadcast

---

## File Map

| File | Action | Why |
|------|--------|-----|
| `src-tauri/src/network/discovery.rs` | modify | fix `get_local_ip()`, use UDP `addr.ip()` |
| `src-tauri/src/network/client.rs` | modify | add read loop in `connect_to_peer`, fix lock-in-IO |
| `src-tauri/src/network/server.rs` | modify | connection replace-on-duplicate, send-after-unlock, error logging |
| `src-tauri/src/network/connection.rs` | modify | add max-length guard |
| `src-tauri/src/main.rs` | modify | start TCP before UDP, fix reconnection set |
| `src-tauri/src/config.rs` | modify | device ID persistence fallback |
| `src-tauri/src/commands/chat.rs` | modify | save received ChatMessage to DB |
| `src-tauri/src/commands/file_transfer.rs` | modify | remove double serialization |
| `src-tauri/src/commands/folder_sync.rs` | modify | broadcast after create |
| `src-tauri/src/commands/announcement.rs` | modify | broadcast on delete |
| `src-tauri/src/commands/board.rs` | modify | broadcast on delete/status/archive |
| `src-tauri/src/models/mod.rs` | modify | remove 3 dead NetworkMessage variants |
| `src/hooks/useChat.ts` | modify | rename `messageType` → `message_type` |
| `src/components/chat/ChatWindow.tsx` | modify | rename `filePath` → `file_path` |
| `src/components/chat/TransferWindow.tsx` | modify | rename params to snake_case |
| `src/hooks/useFolders.ts` | modify | rename params to snake_case |
| `src/components/shared/AnnouncementBoard.tsx` | modify | add `network-message` listener |
| `src/components/board/KanbanBoard.tsx` | modify | add `network-message` listener |
| `src/components/board/TeamBoard.tsx` | modify | add `network-message` listener |

---

## Tasks

### Task 1 — Fix LAN IP discovery (no external internet dependency)
**Files**: `src-tauri/src/network/discovery.rs`  
**Why**: Pure LAN segments have no route to 8.8.8.8; `get_local_ip()` returns None → fallback 127.0.0.1 → all peers connect to themselves.  
**Impact**: Discovery now works in air-gapped networks.  
**Verification**: `cargo check`

- [ ] Replace `get_local_ip()` with interface enumeration (skip loopback, pick first non-169.254.x.x IPv4):
```rust
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    // Try external route first (fast path)
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "127.0.0.1" && !ip.starts_with("169.254.") {
                    return Some(ip);
                }
            }
        }
    }
    // Fallback: enumerate interfaces via local broadcast socket
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("255.255.255.255:1").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "127.0.0.1" && !ip.starts_with("169.254.") {
                    return Some(ip);
                }
            }
        }
    }
    None
}
```
- [ ] In the receive thread, replace `packet.ip` with `addr.ip().to_string()` when storing the peer:
```rust
let user = User {
    id: packet.id,
    username: packet.username,
    ip: addr.ip().to_string(), // ← use actual UDP source address
    status: UserStatus::Online,
    version: packet.version,
};
```
- [ ] `cargo check`

---

### Task 2 — Fix application startup order
**Files**: `src-tauri/src/main.rs`  
**Why**: UDP discovery starts before TCP server is bound; peers try to connect to a closed port.  
**Impact**: Eliminates race-condition connection failures on startup.  
**Verification**: `cargo check`

- [ ] Move TCP `start_server` block **before** `start_discovery` block in `main.rs` setup.
- [ ] `cargo check`

---

### Task 3 — Fix ConnectionPool: never hold lock during I/O
**Files**: `src-tauri/src/network/client.rs`  
**Why**: `broadcast_message` and `send_to_peer` hold the mutex while writing to `TcpStream`, causing deadlocks when a peer is slow.  
**Impact**: All sends become non-blocking w.r.t. the pool lock.  
**Verification**: `cargo check`

- [ ] Rewrite `broadcast_message`:
```rust
pub fn broadcast_message<T: serde::Serialize>(pool: &ConnectionPool, msg: &T) {
    let conns: Vec<Connection> = {
        let p = pool.lock().unwrap();
        p.values().cloned().collect()
    };
    for mut conn in conns {
        if conn.send_message(msg).is_err() {
            let mut p = pool.lock().unwrap();
            p.remove(&conn.peer_id);
        }
    }
}
```
- [ ] Rewrite `send_to_peer`:
```rust
pub fn send_to_peer<T: serde::Serialize>(pool: &ConnectionPool, peer_id: &str, msg: &T) -> std::io::Result<()> {
    let conn = {
        let p = pool.lock().unwrap();
        p.get(peer_id).cloned()
    };
    if let Some(mut conn) = conn {
        conn.send_message(msg)?;
        Ok(())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "peer not connected"))
    }
}
```
- [ ] `cargo check`

---

### Task 4 — Fix connect_to_peer: add read loop, prevent connection leak
**Files**: `src-tauri/src/network/client.rs`, `src-tauri/src/network/server.rs`  
**Why**: `connect_to_peer` spawned a thread that exited after handshake; outbound connections had no reader, so messages sent back on them were lost.  
**Impact**: Both sides of every TCP connection now have an active read loop.  
**Verification**: `cargo check`

- [ ] In `connect_to_peer`, after inserting into pool, start a `read_message` loop identical to `server.rs`. On read error or disconnect, remove from pool:
```rust
{
    let mut p = pool.lock().unwrap();
    p.insert(peer_id.clone(), conn.clone());
}
loop {
    match conn.read_message() {
        Ok(data) => {
            // Re-use server message processor; need AppHandle here.
            // Simpler: just drop the connection on EOF; server thread handles all inbound business messages.
        }
        Err(_) => break,
    }
}
let mut p = pool.lock().unwrap();
p.remove(&peer_id);
```
- [ ] In `server.rs` `handle_incoming`, before `p.insert`, remove any existing connection for the same `peer_id` (close old, keep new):
```rust
{
    let mut p = pool.lock().unwrap();
    p.remove(&peer_id); // old connection drops here
    p.insert(peer_id.clone(), conn.clone());
}
```
- [ ] `cargo check`

---

### Task 5 — Fix main.rs reconnection logic
**Files**: `src-tauri/src/main.rs`  
**Why**: `connected: HashSet<String>` is append-only; peers that restart or change IP are never reconnected.  
**Impact**: TCP connections recover after peer restart.  
**Verification**: `cargo check`

- [ ] Change the reconnect loop to check pool membership instead of a separate set:
```rust
std::thread::spawn(move || {
    loop {
        std::thread::sleep(Duration::from_secs(3));
        let online = network::peer::get_online_users(&peers_for_connect);
        let pool_ids: std::collections::HashSet<String> = {
            let p = pool_for_connect.lock().unwrap();
            p.keys().cloned().collect()
        };
        for user in online {
            if user.id != my_id_for_connect && !pool_ids.contains(&user.id) {
                network::client::connect_to_peer(
                    user.id.clone(),
                    user.ip.clone(),
                    config::CONTROL_PORT,
                    pool_for_connect.clone(),
                    my_id_for_connect.clone(),
                );
            }
        }
    }
});
```
- [ ] Remove the `connected` variable entirely.
- [ ] `cargo check`

---

### Task 6 — Add message length上限 and FileResponse path sanitization
**Files**: `src-tauri/src/network/connection.rs`, `src-tauri/src/network/server.rs`  
**Why**: 4-byte length prefix can request up to 4GB allocation (OOM). `FileResponse` file_path can traverse out of download dir.  
**Impact**: Prevents malicious/corrupted peers from crashing or writing outside sandbox.  
**Verification**: `cargo check`

- [ ] In `connection.rs` `read_message`, cap length at 50MB:
```rust
const MAX_MSG_LEN: usize = 50 * 1024 * 1024;
let len = u32::from_be_bytes(len_buf) as usize;
if len > MAX_MSG_LEN {
    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "message too large"));
}
```
- [ ] In `server.rs` `FileResponse` handler, sanitize path:
```rust
let file_name = std::path::Path::new(&file_path)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("unknown");
let file_path_full = download_dir.join(file_name);
```
- [ ] `cargo check`

---

### Task 7 — Fix device ID persistence
**Files**: `src-tauri/src/config.rs`  
**Why**: If `confy::load` fails, a new UUID is generated; the same machine appears as two users.  
**Impact**: Stable identity across config corruption or app reinstalls.  
**Verification**: `cargo check`

- [ ] In `load_config`, after `confy::load`, if device_id is empty, read from a dedicated `.device_id` file in app_data_dir; if that also fails, generate and save:
```rust
pub fn load_config() -> AppConfig {
    let mut cfg: AppConfig = confy::load("shimmen-lan-suite", "config").unwrap_or_default();
    if cfg.device_id.is_empty() {
        // Try fallback file
        if let Some(app_dir) = dirs::data_dir().map(|d| d.join("shimmen-lan-suite")) {
            let id_file = app_dir.join(".device_id");
            if let Ok(id) = std::fs::read_to_string(&id_file) {
                cfg.device_id = id.trim().to_string();
            }
        }
        if cfg.device_id.is_empty() {
            cfg.device_id = uuid::Uuid::new_v4().to_string();
            // Save fallback
            if let Some(app_dir) = dirs::data_dir().map(|d| d.join("shimmen-lan-suite")) {
                let _ = std::fs::create_dir_all(&app_dir);
                let _ = std::fs::write(app_dir.join(".device_id"), &cfg.device_id);
            }
        }
    }
    cfg
}
```
- [ ] `cargo check`

---

### Task 8 — Fix all frontend→backend parameter name mismatches
**Files**: `src/hooks/useChat.ts`, `src/components/chat/ChatWindow.tsx`, `src/components/chat/TransferWindow.tsx`, `src/hooks/useFolders.ts`  
**Why**: Tauri v2 command macro matches by field name; camelCase ≠ snake_case causes deserialization failure.  
**Impact**: Chat, file transfer, folder creation, and folder subscription commands become callable.  
**Verification**: `npx tsc --noEmit`

- [ ] `useChat.ts:94`: change `messageType: type` → `message_type: type`
- [ ] `ChatWindow.tsx:62`: change `filePath` → `file_path`
- [ ] `TransferWindow.tsx:57`: change `peerId` → `peer_id`, `filePath` → `file_path`
- [ ] `TransferWindow.tsx:60`: change `messageType` → `message_type`
- [ ] `useFolders.ts:17`: change `localPath` → `local_path`
- [ ] `useFolders.ts:22`: change `folderId` → `folder_id`, `localPath` → `local_path`
- [ ] `npx tsc --noEmit`

---

### Task 9 — Fix file_transfer.rs double serialization
**Files**: `src-tauri/src/commands/file_transfer.rs`  
**Why**: `serde_json::to_vec(&msg)` produces raw JSON bytes; passing those bytes to `send_message(&json)` serializes the byte array a second time into `[123,34,70,...]`. Receiver cannot parse it as `NetworkMessage`.  
**Impact**: Point-to-point file transfer actually works.  
**Verification**: `cargo check`

- [ ] Replace the manual serialization with direct `send_message`:
```rust
// BEFORE (bug):
let json = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
conn.send_message(&json).map_err(|e| e.to_string())?;

// AFTER (fix):
conn.send_message(&msg).map_err(|e| e.to_string())?;
```
- [ ] `cargo check`

---

### Task 10 — Save received ChatMessage to database
**Files**: `src-tauri/src/network/server.rs`  
**Why**: Received chat messages are only emitted to the frontend, never persisted. Refreshing the page loses all received history.  
**Impact**: Chat history is complete after page reload.  
**Verification**: `cargo check`

- [ ] In `process_message` `ChatMessage` branch, insert into DB before emitting:
```rust
NetworkMessage::ChatMessage { id, sender_id, sender_name, content, message_type } => {
    if let Some(db) = app_handle.try_state::<crate::db::DbPool>() {
        if let Ok(conn) = db.lock() {
            let _ = conn.execute(
                "INSERT INTO chat_messages (sender_id, sender_name, content, message_type, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![sender_id, sender_name, content, message_type, chrono::Utc::now().timestamp()],
            );
        }
    }
    let _ = app_handle.emit("network-message", ...);
}
```
- [ ] `cargo check`

---

### Task 11 — Broadcast missing state changes (folder create, announcement delete, board changes)
**Files**: `src-tauri/src/commands/folder_sync.rs`, `src-tauri/src/commands/announcement.rs`, `src-tauri/src/commands/board.rs`  
**Why**: Creating a folder, deleting an announcement, deleting/archiving/updating a task only touch local DB; other peers never learn about the change.  
**Impact**: State changes propagate across the LAN.  
**Verification**: `cargo check`

- [ ] `folder_sync.rs` `create_shared_folder`: after DB insert, broadcast the new folder list:
```rust
// After insert, re-query and broadcast
let folders = get_my_shared_folders(db)?;
let msg = crate::models::NetworkMessage::StateSync {
    table: "shared_folders".to_string(),
    data: serde_json::to_value(&folders).unwrap_or(serde_json::Value::Null),
    version: serde_json::json!({}),
};
if let Some(pool) = app_handle.try_state::<crate::network::server::ConnectionPool>() {
    crate::network::client::broadcast_message(&pool, &msg);
}
```
- [ ] `announcement.rs` `delete_announcement`: after DELETE, broadcast empty/updated list via `StateSync { table: "announcements" }`.
- [ ] `board.rs` `delete_task`: after DELETE, broadcast `StateSync { table: "tasks" }`.
- [ ] `board.rs` `update_task_status`: after UPDATE, broadcast `StateSync { table: "tasks" }`.
- [ ] `board.rs` `archive_task`: after archive, broadcast `StateSync { table: "tasks" }`.
- [ ] `cargo check`

---

### Task 12 — Frontend: add network-message and file-sync listeners
**Files**: `src/components/shared/AnnouncementBoard.tsx`, `src/components/board/KanbanBoard.tsx`, `src/components/board/TeamBoard.tsx`, `src/components/folder/FolderBrowser.tsx`  
**Why**: Frontend only loads data on mount; `StateSync` emits are ignored, so UI never refreshes when peers make changes.  
**Impact**: UI auto-refreshes when remote state changes arrive.  
**Verification**: `npx tsc --noEmit`

- [ ] In each component, add a `useEffect` that listens to `network-message` (for board/announcement) or `file-sync` (for folder) and re-fetches data:
```tsx
useEffect(() => {
    const unlisten = listen('network-message', () => {
        loadData(); // existing load function
    });
    return () => { unlisten.then(f => f()); };
}, []);
```
- [ ] `npx tsc --noEmit`

---

### Task 13 — Cleanup: remove dead NetworkMessage variants, add error logging
**Files**: `src-tauri/src/models/mod.rs`, `src-tauri/src/network/server.rs`  
**Why**: `Discovery` / `DiscoveryResponse` / `Heartbeat` are dead code. Silent `Err(_) => {}` makes debugging impossible.  
**Impact**: Cleaner codebase, observable errors.  
**Verification**: `cargo check`

- [ ] `models/mod.rs`: remove the three unused `NetworkMessage` variants.
- [ ] `server.rs`: replace silent error swallowing with `eprintln!` or `println!`:
```rust
Err(e) => {
    eprintln!("[network] read error from {}: {}", peer_id, e);
    break;
}
```
```rust
if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(data) {
    ...
} else {
    eprintln!("[network] failed to parse message from {}", peer_id);
}
```
- [ ] `cargo check`

---

## Risks & Rollback

| Risk | Mitigation |
|------|------------|
| Removing dead `NetworkMessage` variants breaks an old peer's compile | These variants are never constructed; removing them only affects the enum definition. Old peers running compiled binaries are unaffected. |
| Single-connection model still has edge cases with NAT/firewall | This is a LAN tool; NAT is not expected. If needed, a future plan can add UPnP hole punching. |
| Frontend listeners may cause double-fetch on local changes | Acceptable; the extra fetch is idempotent. Can be optimized later with dedup. |

**Rollback**: Every task is a self-contained file edit. If a task introduces a regression, revert that single file. No database migrations are required.

---

## Execution Options

**1. Subagent-Driven (recommended)** — Dispatch a fresh subagent per task. Review between tasks. Fast iteration with parallelization where safe.

**2. Inline Execution** — Execute tasks sequentially in this session using `executing-plans` skill. Better for tracking state across tasks but slower.

**Which approach?**
