use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshPacketType {
    HandshakeInit = 0x01,
    HandshakeResp = 0x02,
    DataFrame = 0x03,
    Heartbeat = 0x04,
}

impl MeshPacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(MeshPacketType::HandshakeInit),
            0x02 => Some(MeshPacketType::HandshakeResp),
            0x03 => Some(MeshPacketType::DataFrame),
            0x04 => Some(MeshPacketType::Heartbeat),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSocketFrame {
    pub packet_type: MeshPacketType,
    pub sender_node_id: String,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}
