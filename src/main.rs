use bevy::prelude::*;

mod components;
mod resources;
mod systems;
pub mod db;

use components::Avatar;
use resources::{Database, GameState, AvatarState};
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
        .add_systems(Startup, (
            database::init_database,
            camera::setup_camera,
            spawn_avatar_entity,
            setup_lighting,
        ))
        .add_systems(Update, (
            database::load_regions.run_if(resource_exists::<Database>),
            database::load_prims.run_if(resource_exists::<Database>),
            rendering::spawn_regions,
            rendering::spawn_prims,
            avatar::spawn_avatar,
            avatar::handle_avatar_movement,
            camera::follow_avatar_camera,
        ))
        .run();
}

fn spawn_avatar_entity(mut commands: Commands) {
    commands.spawn(Avatar);
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
