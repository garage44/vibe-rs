use bevy::pbr::light_consts::lux::AMBIENT_DAYLIGHT;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

mod components;
mod resources;
mod systems;
mod db;
mod utils;

use components::Avatar;
use resources::{Database, GameState, AvatarState, CameraState, MouseState};
use systems::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vibers RS".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }),
            AtmospherePlugin,
        ))
        .insert_resource(AtmosphereModel::default())
        .init_resource::<GameState>()
        .init_resource::<AvatarState>()
        .init_resource::<CameraState>()
        .init_resource::<MouseState>()
        .init_resource::<systems::tile_loader::TileCache>()
        .add_systems(Startup, (
            database::init_database,
            systems::free_camera::setup_camera,
            spawn_avatar_entity,
            setup_sky,
        ))
        .add_systems(Update, (
            database::load_regions.run_if(|db: Option<Res<Database>>| db.is_some()),
            database::load_prims.run_if(|db: Option<Res<Database>>| db.is_some()),
        ))
        .add_systems(
            Update,
            (
                rendering::spawn_regions.after(database::load_regions),
                rendering::spawn_prims.after(database::load_prims),
            ),
        )
        .add_systems(Update, (
            systems::tile_loader::load_region_tiles,
            rendering::update_region_materials,
        ))
        .add_systems(Update, (
            systems::free_camera::camera_mode_toggle,
            avatar::handle_avatar_movement, // Run avatar movement before camera so camera can follow
            systems::free_camera::camera_controls.after(avatar::handle_avatar_movement),
            avatar::spawn_avatar,
            avatar::update_fox_animation.after(avatar::handle_avatar_movement),
            systems::debug::debug_region_entities.after(rendering::spawn_regions),
        ))
        .run();
}

fn spawn_avatar_entity(mut commands: Commands) {
    // Spawn avatar entity - Bevy fox ~40 units, scale to ~0.8 units (20% smaller)
    commands.spawn((
        Avatar,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::splat(0.02)),
    ));
}

fn setup_sky(
    mut commands: Commands,
    mut atmosphere: AtmosphereMut<Nishita>,
) {
    // Sun position: afternoon sun from upper-right (matches sky gradient)
    let sun_position = Vec3::new(0.3, 0.8, 0.5).normalize();
    atmosphere.sun_position = sun_position;

    // Directional light aligned with sun (Bevy light illuminates along -Z in local space)
    commands.spawn((
        DirectionalLight {
            illuminance: AMBIENT_DAYLIGHT,
            ..default()
        },
        Transform::from_translation(Vec3::ZERO).looking_to(-sun_position, Vec3::Y),
    ));

    // Ambient light for fill (sky provides additional ambient)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
        affects_lightmapped_meshes: true,
    });
}
