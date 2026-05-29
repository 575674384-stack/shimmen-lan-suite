use serde::Serialize;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Connection {
    pub peer_id: String,
    pub stream: Arc<Mutex<TcpStream>>,
    pub id: String,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            peer_id: self.peer_id.clone(),
            stream: self.stream.clone(),
            id: self.id.clone(),
        }
    }
}

impl Connection {
    pub fn new(peer_id: String, stream: TcpStream) -> Self {
        Self { peer_id, stream: Arc::new(Mutex::new(stream)), id: uuid::Uuid::new_v4().to_string() }
    }

    /// 发送 JSON 消息：先写 4 字节长度前缀（大端），再写 JSON 内容
    /// 使用 Mutex 保护写操作，避免多线程并发写入导致数据交错
    pub fn send_message<T: Serialize>(&self, msg: &T) -> std::io::Result<()> {
        let json = serde_json::to_vec(msg)?;
        let len = json.len() as u32;
        let mut stream = self.stream.lock().unwrap_or_else(|e| e.into_inner());
        // 设置写超时，防止半开连接导致无限阻塞
        stream.set_write_timeout(Some(std::time::Duration::from_secs(8)))?;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&json)?;
        stream.flush()?;
        Ok(())
    }

    /// 读取一条消息：先读 4 字节长度，再读对应长度的 JSON
    pub fn read_message(&self) -> std::io::Result<Vec<u8>> {
        const MAX_MSG_LEN: usize = 10 * 1024 * 1024; // 10 MB cap
        let mut stream = self.stream.lock().unwrap_or_else(|e| e.into_inner());
        // Set a read timeout to prevent indefinite blocking on dead connections
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(8)));
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;
        if len > MAX_MSG_LEN {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("message too large: {} bytes", len)));
        }
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;
        Ok(buf)
    }
}
