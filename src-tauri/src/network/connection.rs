use serde::Serialize;
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    pub peer_id: String,
    pub stream: TcpStream,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            peer_id: self.peer_id.clone(),
            stream: self.stream.try_clone().expect("Failed to clone TcpStream"),
        }
    }
}

impl Connection {
    pub fn new(peer_id: String, stream: TcpStream) -> Self {
        Self { peer_id, stream }
    }

    /// 发送 JSON 消息：先写 4 字节长度前缀（大端），再写 JSON 内容
    pub fn send_message<T: Serialize>(&self, msg: &T) -> std::io::Result<()> {
        let json = serde_json::to_vec(msg)?;
        let len = json.len() as u32;
        let mut stream = &self.stream;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&json)?;
        stream.flush()?;
        Ok(())
    }

    /// 读取一条消息：先读 4 字节长度，再读对应长度的 JSON
    pub fn read_message(&self) -> std::io::Result<Vec<u8>> {
        let mut stream = &self.stream;
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;
        Ok(buf)
    }
}
