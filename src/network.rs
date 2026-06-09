use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// 协议常量
pub const MAGIC: [u8; 2] = [0xD0, 0x9A];
pub const PROTOCOL_VERSION: u8 = 0x01;
const MAX_PAYLOAD_LEN: u32 = 0xFFFFFF; // 16MB
const HEADER_SIZE: usize = 8;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NetPacketState {
    IGNORE = 0x00,
    OK = 0x01,
    ERR = 0x02,
    EMPTY = 0x03,
    NOAUTH = 0x04,
    PIPELINE = 0x05,  // Pipeline 批量命令标记
}

impl NetPacketState {
    fn from_u8(v: u8) -> Self {
        match v {
            0x00 => Self::IGNORE,
            0x01 => Self::OK,
            0x02 => Self::ERR,
            0x03 => Self::EMPTY,
            0x04 => Self::NOAUTH,
            0x05 => Self::PIPELINE,
            _ => Self::EMPTY,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

pub struct NetPacket {
    body: Vec<u8>,
    state: NetPacketState,
}

impl NetPacket {
    pub(crate) fn make(body: Vec<u8>, state: NetPacketState) -> Self {
        Self { body, state }
    }

    pub(crate) async fn send(&self, socket: &mut TcpStream) -> crate::Result<()> {
        socket.write_u8(MAGIC[0]).await?;
        socket.write_u8(MAGIC[1]).await?;
        socket.write_u8(PROTOCOL_VERSION).await?;
        socket.write_u8(self.state.to_u8()).await?;
        socket.write_u32(self.body.len() as u32).await?;
        if !self.body.is_empty() {
            socket.write_all(&self.body).await?;
        }
        Ok(())
    }

    /// 批量发送多个包，只做一次 write_all 系统调用
    pub(crate) fn serialize_batch(bodies: &[Vec<u8>], state: NetPacketState) -> Vec<u8> {
        let mut buffer = Vec::new();
        for body in bodies {
            // MAGIC
            buffer.extend_from_slice(&MAGIC);
            // VERSION + STATE
            buffer.push(PROTOCOL_VERSION);
            buffer.push(state.to_u8());
            // LENGTH (4 bytes, big endian)
            buffer.extend_from_slice(&(body.len() as u32).to_be_bytes());
            // BODY
            buffer.extend_from_slice(body);
        }
        buffer
    }
}

pub struct Frame {
    pub latest_state: NetPacketState,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            latest_state: NetPacketState::EMPTY,
        }
    }
}

impl Frame {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn parse_frame(&mut self, socket: &mut TcpStream) -> crate::Result<Vec<u8>> {
        // 1. 读 8 字节头部
        let mut header = [0u8; HEADER_SIZE];
        socket.read_exact(&mut header).await?;

        // 2. 校验 MAGIC
        if header[0] != MAGIC[0] || header[1] != MAGIC[1] {
            return Err(anyhow::anyhow!(
                "invalid magic bytes, stream corrupted"
            ));
        }

        // 3. 校验 VERSION
        if header[2] != PROTOCOL_VERSION {
            return Err(anyhow::anyhow!(
                "unsupported protocol version: {}",
                header[2]
            ));
        }

        // 4. 解析 STATE
        self.latest_state = NetPacketState::from_u8(header[3]);

        // 5. 解析 LEN (大端序)
        let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;

        // 6. 校验大小限制
        if len as u32 > MAX_PAYLOAD_LEN {
            return Err(anyhow::anyhow!("payload too large: {} bytes", len));
        }

        // 7. 读取 payload
        let mut buf = vec![0u8; len];
        if len > 0 {
            socket.read_exact(&mut buf).await?;
        }

        Ok(buf)
    }
}
