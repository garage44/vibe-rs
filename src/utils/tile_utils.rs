/**
 * OSM Tile Utilities
 * Converts between geographic coordinates (lat/lng) and OSM tile coordinates
 * Uses Web Mercator projection
 */

const EARTH_RADIUS: f64 = 6_378_137.0; // meters
const EARTH_CIRCUMFERENCE: f64 = 2.0 * std::f64::consts::PI * EARTH_RADIUS; // meters

/**
 * Convert latitude/longitude to OSM tile coordinates
 */
pub fn lat_lng_to_tile(lat: f64, lng: f64, zoom: u32) -> (i64, i64) {
    let n = 2.0_f64.powi(zoom as i32);
    let x = ((lng + 180.0) / 360.0 * n).floor() as i64;
    let lat_rad = lat.to_radians();
    let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n).floor() as i64;
    (x, y)
}

/**
 * Convert OSM tile coordinates to latitude/longitude (top-left corner of tile)
 */
pub fn tile_to_lat_lng(x: i64, y: i64, zoom: u32) -> (f64, f64) {
    let n = 2.0_f64.powi(zoom as i32);
    let lng = (x as f64 / n) * 360.0 - 180.0;
    let lat_rad = ((1.0 - (2.0 * y as f64) / n) * std::f64::consts::PI).sinh().atan();
    let lat = lat_rad.to_degrees();
    (lat, lng)
}

/**
 * Get the real-world meters per tile at a given zoom level
 */
pub fn tile_to_meters(zoom: u32, lat: f64) -> f64 {
    let lat_rad = lat.to_radians();
    let meters_per_pixel = (EARTH_CIRCUMFERENCE * lat_rad.cos()) / (256.0 * 2.0_f64.powi(zoom as i32));
    meters_per_pixel * 256.0 // 256 pixels per tile
}

/**
 * Find the optimal zoom level closest to target meters per tile
 */
pub fn find_optimal_zoom(target_meters: f64, lat: f64) -> u32 {
    let mut best_zoom = 0;
    let mut best_diff = f64::INFINITY;

    for zoom in 0..=19 {
        let meters = tile_to_meters(zoom, lat);
        let diff = (meters - target_meters).abs();
        if diff < best_diff {
            best_diff = diff;
            best_zoom = zoom;
        }
    }

    best_zoom
}

// Fixed zoom level for regions (256m target, zoom 17 ≈ 305.7m per tile)
pub const REGION_ZOOM_LEVEL: u32 = 17;
pub const REGION_SIZE_METERS: f64 = 256.0;
