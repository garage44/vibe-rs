use bevy::prelude::*;

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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vibers RS".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<GameState>()
        .init_resource::<AvatarState>()
        .init_resource::<CameraState>()
        .init_resource::<MouseState>()
        .init_resource::<systems::tile_loader::TileCache>()
        .add_systems(Startup, (
            database::init_database,
            systems::free_camera::setup_camera,
            spawn_avatar_entity,
            setup_lighting,
        ))
        .add_systems(Update, (
            database::load_regions.run_if(|db: Option<Res<Database>>| db.is_some()),
            database::load_prims.run_if(|db: Option<Res<Database>>| db.is_some()),
        ))
        .add_systems(Update, (
            rendering::spawn_regions.after(database::load_regions),
            rendering::spawn_prims.after(database::load_prims),
            systems::tile_loader::load_region_tiles,
            rendering::update_region_materials,
            systems::free_camera::camera_mode_toggle,
            avatar::handle_avatar_movement, // Run avatar movement before camera so camera can follow
            systems::free_camera::camera_controls.after(avatar::handle_avatar_movement),
            avatar::spawn_avatar,
            systems::debug::debug_region_entities.after(rendering::spawn_regions),
        ))
        .run();
}

fn spawn_avatar_entity(mut commands: Commands) {
    // Spawn avatar entity with initial transform and GlobalTransform
    // GlobalTransform is required for children to follow parent
    commands.spawn((
        Avatar,
        Transform::from_translation(Vec3::new(0.0, 2.2, 0.0)),
        GlobalTransform::default(),
    ));
}

fn setup_lighting(mut commands: Commands) {
    // Directional light (sun)
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.5,
            -0.5,
            0.0,
        )),
        ..default()
    });

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.15,
    });
}
