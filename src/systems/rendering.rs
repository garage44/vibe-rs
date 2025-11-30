use bevy::prelude::*;
use crate::components::{Region, Prim, PrimShape};

#[derive(Component)]
struct RegionMesh;

#[derive(Component)]
struct PrimMesh;

pub fn spawn_regions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    region_query: Query<(Entity, &Region), (Without<RegionMesh>, Without<Prim>)>,
) {
    for (entity, region) in region_query.iter() {
        // Calculate grid position
        let grid_size = 1.0; // Simplified for now - could calculate based on total regions
        let spacing = 300.0;
        let row = 0;
        let col = 0;
        let position = Vec3::new(
            (col as f32 - grid_size / 2.0) * spacing,
            0.2, // Slightly above water level
            (row as f32 - grid_size / 2.0) * spacing,
        );

        // Create plane mesh (256x256 units)
        let plane_mesh = meshes.add(Plane3d::default().mesh().size(256.0, 256.0));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            ..default()
        });

        // Spawn region as a plane
        commands.entity(entity).insert((
            Mesh3d(plane_mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            RegionMesh,
        ));
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
            PrimShape::Box => meshes.add(Cuboid::new(transform.scale.x, transform.scale.y, transform.scale.z)),
            PrimShape::Sphere => {
                let radius = transform.scale.x.max(transform.scale.y).max(transform.scale.z) / 2.0;
                meshes.add(Sphere::new(radius))
            }
            PrimShape::Cylinder => meshes.add(Cylinder::new(transform.scale.x / 2.0, transform.scale.y)),
            PrimShape::Cone => meshes.add(Cone::new(transform.scale.x / 2.0, transform.scale.y)),
            PrimShape::Torus => meshes.add(Torus::new(transform.scale.x / 2.0, transform.scale.y / 4.0)),
        };

        let material_handle = materials.add(StandardMaterial {
            base_color: prim.color,
            ..default()
        });

        commands.entity(entity).insert((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            PrimMesh,
        ));
    }
}
