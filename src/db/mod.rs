pub mod schema;

/// Helper function to calculate tile coordinates from lat/lng
/// This can be used when creating or updating regions
pub fn calculate_tile_coordinates(lat: f64, lng: f64) -> (i64, i64, u32) {
    use crate::utils::tile_utils::{lat_lng_to_tile, REGION_ZOOM_LEVEL};
    let (x, y) = lat_lng_to_tile(lat, lng, REGION_ZOOM_LEVEL);
    (x, y, REGION_ZOOM_LEVEL)
}
