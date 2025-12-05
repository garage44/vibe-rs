use bevy::prelude::*;
use crate::components::Region;
use crate::systems::rendering::RegionMesh;

pub fn debug_region_entities(
    all_regions: Query<(Entity, &Region)>,
    region_with_mesh: Query<Entity, (With<Region>, With<RegionMesh>)>,
    region_without_mesh: Query<Entity, (With<Region>, Without<RegionMesh>)>,
) {
    println!("=== REGION DEBUG ===");
    println!("Total regions: {}", all_regions.iter().count());
    println!("Regions WITH RegionMesh: {}", region_with_mesh.iter().count());
    println!("Regions WITHOUT RegionMesh: {}", region_without_mesh.iter().count());

    for (entity, region) in all_regions.iter() {
        println!("Region entity {:?}: id={}, name='{}'", entity, region.id, region.name);
    }
    println!("===================");
}
