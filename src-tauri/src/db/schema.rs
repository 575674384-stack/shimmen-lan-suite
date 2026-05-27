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
        );

        CREATE TABLE IF NOT EXISTS ai_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            api_key TEXT NOT NULL DEFAULT '',
            base_url TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            updated_at INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS chat_messages (
            id TEXT PRIMARY KEY,
            sender_id TEXT NOT NULL,
            sender_name TEXT NOT NULL,
            message_type TEXT NOT NULL DEFAULT 'text',
            content TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS file_index (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            peer_id TEXT NOT NULL,
            peer_name TEXT NOT NULL DEFAULT '',
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL DEFAULT 0,
            modified_at INTEGER NOT NULL DEFAULT 0,
            is_local INTEGER NOT NULL DEFAULT 0,
            indexed_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );
        CREATE INDEX IF NOT EXISTS idx_file_index_name ON file_index(file_name);
        CREATE INDEX IF NOT EXISTS idx_file_index_peer ON file_index(peer_id);
        CREATE INDEX IF NOT EXISTS idx_file_index_search ON file_index(peer_id, file_name);
    ")?;
    Ok(())
}
