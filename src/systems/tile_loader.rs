use bevy::prelude::*;
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor, ImageFilterMode};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::Read;

/// Cache key for tiles
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TileKey {
    pub x: i64,
    pub y: i64,
    pub z: u32,
}

impl TileKey {
    pub fn new(x: i64, y: i64, z: u32) -> Self {
        Self { x, y, z }
    }

    pub fn to_string(&self) -> String {
        format!("{}/{}/{}", self.z, self.x, self.y)
    }
}

/// Resource for managing OSM tile loading and caching
#[derive(Resource)]
pub struct TileCache {
    /// Cache of loaded tile handles
    pub handles: Arc<Mutex<HashMap<TileKey, Handle<Image>>>>,
    /// Cache of loading states (tiles currently being loaded)
    pub loading: Arc<Mutex<HashMap<TileKey, bool>>>,
}

impl Default for TileCache {
    fn default() -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
            loading: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Component to mark regions that need tile textures
#[derive(Component)]
pub struct RegionTile {
    pub tile_key: TileKey,
    pub lod_level: u32, // 0 = high-res (2x2), 1 = medium-res (1x1), 2 = low-res (1x1)
}

/// Get OSM tile URL for given tile coordinates
/// Uses OpenStreetMap tile server (you may want to use your own proxy/server)
pub fn get_osm_tile_url(x: i64, y: i64, z: u32) -> String {
    // Using OpenStreetMap tile server
    // Note: In production, you should use your own tile server or proxy to avoid rate limiting
    format!("https://tile.openstreetmap.org/{}/{}/{}.png", z, x, y)
}

/// Load a single OSM tile image
pub fn load_tile_image(x: i64, y: i64, z: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let url = get_osm_tile_url(x, y, z);

    // Use blocking HTTP client
    let response = ureq::get(&url).call()?;
    let mut bytes = Vec::new();
    response.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// System to load OSM tiles for regions
pub fn load_region_tiles(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    tile_cache: Res<TileCache>,
    region_query: Query<(Entity, &RegionTile), Without<RegionTileTexture>>,
) {
    for (entity, region_tile) in region_query.iter() {
        let tile_key = region_tile.tile_key.clone();

        // Check if already cached
        {
            let handles = tile_cache.handles.lock().unwrap();
            if let Some(handle) = handles.get(&tile_key) {
                // Tile already loaded, attach handle to entity
                commands.entity(entity).insert(RegionTileTexture {
                    handle: handle.clone(),
                });
                continue;
            }
        }

        // Check if already loading
        {
            let mut loading = tile_cache.loading.lock().unwrap();
            if loading.contains_key(&tile_key) {
                continue; // Already loading
            }
            loading.insert(tile_key.clone(), true);
        }

        // Load tile asynchronously (for now, we'll do blocking load)
        // In a real implementation, you'd want to use async/await with Bevy's async support
        match load_tile_image(tile_key.x, tile_key.y, tile_key.z) {
            Ok(bytes) => {
                // Decode image
                match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let size = bevy::render::render_resource::Extent3d {
                            width: rgba.width(),
                            height: rgba.height(),
                            depth_or_array_layers: 1,
                        };

                        // Create Bevy image
                        let mut bevy_image = Image::new(
                            size,
                            bevy::render::render_resource::TextureDimension::D2,
                            rgba.into_raw(),
                            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                        );

                        // Configure sampler for OSM tiles
                        bevy_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            min_filter: ImageFilterMode::Linear,
                            mag_filter: ImageFilterMode::Linear,
                            mipmap_filter: ImageFilterMode::Linear,
                            ..default()
                        });

                        let handle = images.add(bevy_image);

                        // Cache the handle
                        {
                            let mut handles = tile_cache.handles.lock().unwrap();
                            handles.insert(tile_key.clone(), handle.clone());
                        }

                        // Attach to entity
                        commands.entity(entity).insert(RegionTileTexture {
                            handle: handle.clone(),
                        });

                        // Remove from loading
                        {
                            let mut loading = tile_cache.loading.lock().unwrap();
                            loading.remove(&tile_key);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to decode tile image: {}", e);
                        let mut loading = tile_cache.loading.lock().unwrap();
                        loading.remove(&tile_key);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to load tile {}: {}", tile_key.to_string(), e);
                let mut loading = tile_cache.loading.lock().unwrap();
                loading.remove(&tile_key);
            }
        }
    }
}

/// Component to store the tile texture handle for a region
#[derive(Component)]
pub struct RegionTileTexture {
    pub handle: Handle<Image>,
}
