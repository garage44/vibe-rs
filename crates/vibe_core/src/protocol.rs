//! Wire messages (ADR-009). Framing is length-delimited bytes (ADR-008); body is postcard.

use glam::Vec3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegionDto {
    pub id: i64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub tile_x: i64,
    pub tile_y: i64,
    pub tile_z: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimDto {
    pub id: i64,
    pub region_id: i64,
    pub name: String,
    pub shape: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvatarStateDto {
    pub id: u64,
    pub position: Vec3,
    pub yaw: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetMessage {
    ClientHello {
        protocol_version: u16,
        client_token: String,
    },
    ServerHelloAck {
        session_id: Uuid,
        tick_hz: f32,
        your_avatar_id: u64,
    },
    ServerError {
        request_id: u32,
        code: u32,
        message: String,
    },
    ClientIntent {
        request_id: u32,
        move_x: f32,
        move_z: f32,
        fly_up: bool,
        fly_down: bool,
    },
    ObserverUpdate {
        position: Vec3,
    },
    WorldSnapshot {
        tick: u64,
        regions: Vec<RegionDto>,
        prims: Vec<PrimDto>,
        avatars: Vec<AvatarStateDto>,
    },
}

#[must_use]
pub fn encode_message(msg: &NetMessage) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(msg)
}

#[must_use]
pub fn decode_message(bytes: &[u8]) -> Result<NetMessage, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_hello() {
        let m = NetMessage::ClientHello {
            protocol_version: PROTOCOL_VERSION,
            client_token: "test".into(),
        };
        let b = encode_message(&m).unwrap();
        let m2 = decode_message(&b).unwrap();
        assert_eq!(m, m2);
    }
}
