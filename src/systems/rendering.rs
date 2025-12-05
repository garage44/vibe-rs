use bevy::prelude::*;
use bevy::render::mesh::shape::{UVSphere, Cylinder, Torus};
use crate::components::{Region, Prim, PrimShape};
use crate::systems::tile_loader::{RegionTile, TileKey};
use crate::utils::tile_utils::REGION_SIZE_METERS;

#[derive(Component)]
pub struct RegionMesh;

#[derive(Component)]
pub struct PrimMesh;

/// Spawn region meshes with tile loading setup
pub fn spawn_regions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    region_query: Query<(Entity, &Region), (Without<RegionMesh>, Without<Prim>)>,
    all_regions: Query<&Region>,
) {
    // Calculate grid size based on total regions
    let total_regions = all_regions.iter().count();
    let query_count = region_query.iter().count();

    if total_regions == 0 {
        return; // No regions to spawn
    }

    if query_count == 0 && total_regions > 0 {
        // Regions exist but query isn't matching - this means entities already have RegionMesh or Prim
        println!("WARNING: Found {} regions but query matched 0 entities (entities may already have RegionMesh)", total_regions);
        return;
    }

    let grid_size = (total_regions as f32).sqrt().ceil();

    // Collect all region IDs to calculate index
    let mut region_ids: Vec<i64> = all_regions.iter().map(|r| r.id).collect();
    region_ids.sort();

    let mut spawned_count = 0;
    for (entity, region) in region_query.iter() {
        spawned_count += 1;
        // For single region, place at origin. For multiple regions, use grid layout
        let position = if total_regions == 1 {
            Vec3::new(0.0, 0.0, 0.0) // Place single region at origin
        } else {
            // Find index of this region in sorted list
            let index = region_ids.iter().position(|&id| id == region.id).unwrap_or(0);
            let row = (index as f32 / grid_size).floor() as i32;
            let col = index % grid_size as usize;
            let spacing = 300.0;
            Vec3::new(
                (col as f32 - grid_size / 2.0) * spacing,
                0.0,
                (row as f32 - grid_size / 2.0) * spacing,
            )
        };

        println!("Spawning region '{}' at position {:?}", region.name, position);

        // Create a simple flat box as the region (easier than plane rotation)
        // Box with very small height to act as a flat plane
        let region_size = REGION_SIZE_METERS as f32;
        let region_mesh = meshes.add(Mesh::from(bevy::render::mesh::shape::Box::new(
            region_size,
            0.1, // Very thin - acts like a plane
            region_size,
        )));

        // Create simple untextured material
        let default_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.7, 0.7, 0.7), // Light gray
            ..default()
        });

        // Create tile key for this region
        let tile_key = TileKey::new(region.tile_x, region.tile_y, region.tile_z as u32);

        // Spawn region as a flat box at y=0
        let transform = Transform::from_translation(position);

        commands.entity(entity).insert((
            MaterialMeshBundle {
                mesh: region_mesh,
                material: default_material,
                transform,
                visibility: Visibility::Visible,
                ..default()
            },
            RegionMesh,
            RegionTile {
                tile_key,
                lod_level: 1, // Start with medium quality
            },
        ));
    }

    if spawned_count > 0 {
        println!("Spawned {} region meshes", spawned_count);
    }
}

/// Update region materials when tile textures are loaded
pub fn update_region_materials(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut region_query: Query<(&mut Handle<StandardMaterial>, &crate::systems::tile_loader::RegionTileTexture), (With<RegionMesh>, Changed<crate::systems::tile_loader::RegionTileTexture>)>,
    images: Res<Assets<Image>>,
) {
    for (mut material_handle, tile_texture) in region_query.iter_mut() {
        if images.get(&tile_texture.handle).is_some() {
            // Create new material with tile texture
            let new_material = materials.add(StandardMaterial {
                base_color_texture: Some(tile_texture.handle.clone()),
                ..default()
            });
            *material_handle = new_material;
        }
    }
}

pub fn spawn_prims(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    prim_query: Query<(Entity, &Prim, &Transform), (Without<PrimMesh>, Without<RegionMesh>)>,
) {
    for (entity, prim, transform) in prim_query.iter() {
        let mesh_handle = match prim.shape {
            PrimShape::Box => meshes.add(Mesh::from(bevy::render::mesh::shape::Box::new(transform.scale.x, transform.scale.y, transform.scale.z))),
            PrimShape::Sphere => {
                meshes.add(Mesh::from(UVSphere::default()))
            }
            PrimShape::Cylinder => meshes.add(Mesh::from(Cylinder::default())),
            PrimShape::Cone => meshes.add(Mesh::from(Cylinder::default())), // Use cylinder as cone substitute
            PrimShape::Torus => meshes.add(Mesh::from(Torus::default())),
        };

        let material_handle = materials.add(StandardMaterial {
            base_color: prim.color,
            ..default()
        });

        commands.entity(entity).insert((
            MaterialMeshBundle {
                mesh: mesh_handle,
                material: material_handle,
                transform: *transform,
                ..default()
            },
            PrimMesh,
        ));
    }
}
