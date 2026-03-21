//! Shared types for vibers sim and client (ADR-006, ADR-009, ADR-015).

pub mod error;
pub mod protocol;
pub mod world;

pub use error::ProtocolError;
pub use protocol::{
    decode_message, encode_message, AvatarStateDto, NetMessage, PrimDto, RegionDto,
    PROTOCOL_VERSION,
};
pub use world::{
    find_optimal_zoom, lat_lng_to_tile, tile_to_lat_lng, tile_to_meters, TileKey,
    REGION_SIZE_METERS, REGION_ZOOM_LEVEL,
};
