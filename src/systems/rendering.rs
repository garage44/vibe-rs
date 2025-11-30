use bevy::prelude::*;
use bevy::render::mesh::shape::{UVSphere, Cylinder, Torus, Plane};
use crate::components::{Region, Prim, PrimShape};

#[derive(Component)]
pub struct RegionMesh;

#[derive(Component)]
pub struct PrimMesh;

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

        // Create plane mesh (256x256 units) - scale will be applied via transform
        let plane_mesh = meshes.add(Mesh::from(Plane::default()));
        let material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.8, 0.8),
            ..default()
        });

        // Spawn region as a plane
        commands.entity(entity).insert((
            MaterialMeshBundle {
                mesh: plane_mesh,
                material,
                transform: Transform::from_translation(position)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::new(256.0, 1.0, 256.0)),
                ..default()
            },
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
