use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex};
use bevy::ecs::query::Or;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use crate::components::{Avatar, RemoteAvatar};
use crate::resources::AvatarState;

// Official Bevy fox model (models/animated/Fox.glb)
const FOX_GLB: &str = "models/animated/Fox.glb";
// Bevy fox animations: 0=Survey (idle), 1=Walk, 2=Run
const IDLE_ANIMATION_INDEX: usize = 0;
const RUN_ANIMATION_INDEX: usize = 2;

/// Online: blend visual toward authoritative sim position (see `smooth_online_avatar_display`).
const ONLINE_DISPLAY_SMOOTHING: f32 = 14.0;

const WALK_SPEED: f32 = 8.0;
const FLY_SPEED: f32 = 40.0;
const ROTATION_SPEED: f32 = 2.0;
const GRAVITY: f32 = -9.8;
const AVATAR_HEIGHT: f32 = 0.8; // Fox model height (scaled)
const GROUND_HEIGHT: f32 = 0.05; // Region tile top surface (cuboid half-extent y)

#[derive(Component)]
pub struct AvatarFoxLoaded;

/// Component storing animation data for the fox, used when scene is ready
#[derive(Component)]
pub(crate) struct FoxAnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    idle_index: AnimationNodeIndex,
    run_index: AnimationNodeIndex,
}

pub fn spawn_avatar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    avatar_query: Query<Entity, (Without<AvatarFoxLoaded>, Or<(With<Avatar>, With<RemoteAvatar>)>)>,
) {
    for entity in avatar_query.iter() {
        let (graph, indices) = AnimationGraph::from_clips([
            asset_server.load(GltfAssetLabel::Animation(IDLE_ANIMATION_INDEX).from_asset(FOX_GLB)),
            asset_server.load(GltfAssetLabel::Animation(RUN_ANIMATION_INDEX).from_asset(FOX_GLB)),
        ]);
        let graph_handle = graphs.add(graph);

        let mesh_scene = SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(FOX_GLB)),
        );

        commands.entity(entity).insert((
            AvatarFoxLoaded,
            FoxAnimationToPlay {
                graph_handle,
                idle_index: indices[0],
                run_index: indices[1],
            },
            mesh_scene,
        )).observe(play_fox_animation_when_ready);
    }
}

fn play_fox_animation_when_ready(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&FoxAnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if let Ok(animation_to_play) = animations_to_play.get(trigger.target()) {
        for child in children.iter_descendants(trigger.target()) {
            if let Ok(mut player) = players.get_mut(child) {
                // Start with idle; update_fox_animation will switch based on movement
                player.play(animation_to_play.idle_index).repeat().set_speed(1.5);
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}

/// Switch fox animation between idle and run based on movement
pub fn update_fox_animation(
    avatar_state: Res<AvatarState>,
    children: Query<&Children>,
    animations_to_play: Query<&FoxAnimationToPlay, With<Avatar>>,
    mut players: Query<&mut AnimationPlayer>,
    avatar_query: Query<Entity, With<Avatar>>,
) {
    for avatar_entity in avatar_query.iter() {
        let Ok(animation_to_play) = animations_to_play.get(avatar_entity) else {
            continue;
        };
        for child in children.iter_descendants(avatar_entity) {
            if let Ok(mut player) = players.get_mut(child) {
                if avatar_state.is_walking {
                    player.stop(animation_to_play.idle_index);
                    if !player.is_playing_animation(animation_to_play.run_index) {
                        player.play(animation_to_play.run_index).repeat().set_speed(1.5);
                    }
                } else {
                    player.stop(animation_to_play.run_index);
                    if !player.is_playing_animation(animation_to_play.idle_index) {
                        player.play(animation_to_play.idle_index).repeat().set_speed(1.5);
                    }
                }
            }
        }
    }
}

/// When online, the server updates `AvatarState::position` at tick rate; smooth `display_position`
/// and the avatar transform so the third-person camera and world do not jitter.
pub fn smooth_online_avatar_display(
    online: Option<Res<crate::resources::OnlineSession>>,
    mut avatar_state: ResMut<AvatarState>,
    time: Res<Time>,
    mut avatar_query: Query<&mut Transform, With<Avatar>>,
) {
    let dt = time.delta_secs();
    if online.is_some() {
        let alpha = 1.0 - (-ONLINE_DISPLAY_SMOOTHING * dt).exp();
        avatar_state.display_position = avatar_state.display_position.lerp(avatar_state.position, alpha);
        if let Ok(mut tf) = avatar_query.single_mut() {
            tf.translation = avatar_state.display_position;
        }
    } else {
        avatar_state.display_position = avatar_state.position;
    }
}

pub fn handle_avatar_movement(
    online: Option<Res<crate::resources::OnlineSession>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut avatar_query: Query<&mut Transform, With<Avatar>>,
    mut avatar_state: ResMut<AvatarState>,
    camera_state: Res<crate::resources::CameraState>,
) {
    // Don't move avatar if in free camera mode (camera handles movement)
    if camera_state.mode == crate::resources::CameraMode::Free {
        avatar_state.is_walking = false;
        return;
    }
    if avatar_query.is_empty() {
        return;
    }

    let Ok(mut transform) = avatar_query.single_mut() else {
        return;
    };
    let delta_time = time.delta().as_secs_f32();

    // Online: server owns translation; keep local rotation + fly toggle for intent encoding.
    if online.is_some() {
        if keyboard_input.just_pressed(KeyCode::KeyF) {
            avatar_state.is_flying = !avatar_state.is_flying;
        }
        let move_left =
            keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft);
        let move_right =
            keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight);
        let move_forward =
            keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp);
        let move_backward =
            keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown);
        avatar_state.is_walking =
            move_forward || move_backward || move_left || move_right;
        if move_left {
            avatar_state.rotation += ROTATION_SPEED * delta_time;
        }
        if move_right {
            avatar_state.rotation -= ROTATION_SPEED * delta_time;
        }
        transform.rotation = Quat::from_rotation_y(avatar_state.rotation + std::f32::consts::PI);
        return;
    }

    // Always sync avatar state position with transform
    // This ensures camera can follow the avatar correctly
    avatar_state.position = transform.translation;
    avatar_state.display_position = transform.translation;

    // Toggle fly mode with F key
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        avatar_state.is_flying = !avatar_state.is_flying;
        println!("Fly mode: {}", if avatar_state.is_flying { "ON" } else { "OFF" });
    }

    // Movement input
    let move_forward = keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp);
    let move_backward = keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown);
    let move_left = keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft);
    let move_right = keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight);
    let fly_up = keyboard_input.pressed(KeyCode::Space);
    let fly_down = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    // Calculate movement direction
    let mut move_direction = Vec3::ZERO;
    if move_forward {
        move_direction.z -= 1.0;
    }
    if move_backward {
        move_direction.z += 1.0;
    }
    if move_left {
        move_direction.x -= 1.0;
    }
    if move_right {
        move_direction.x += 1.0;
    }

    // Normalize and apply speed
    if move_direction.length() > 0.0 {
        move_direction = move_direction.normalize();
        let speed = if avatar_state.is_flying { FLY_SPEED } else { WALK_SPEED };
        move_direction *= speed * delta_time;

        // Rotate movement direction based on avatar rotation
        let rotation_matrix = Mat3::from_rotation_y(avatar_state.rotation);
        move_direction = rotation_matrix * move_direction;

        // Apply movement to transform
        transform.translation.x += move_direction.x;
        transform.translation.z += move_direction.z;
        avatar_state.is_walking = true;

        // Handle rotation (only rotate when moving)
        if move_left {
            let rotation_delta = ROTATION_SPEED * delta_time;
            avatar_state.rotation += rotation_delta;
        }
        if move_right {
            let rotation_delta = -ROTATION_SPEED * delta_time;
            avatar_state.rotation += rotation_delta;
        }
    } else {
        avatar_state.is_walking = false;
    }

    // Handle vertical movement
    if avatar_state.is_flying {
        if fly_up {
            transform.translation.y += FLY_SPEED * delta_time;
        } else if fly_down {
            let min_height = GROUND_HEIGHT + AVATAR_HEIGHT / 2.0;
            if transform.translation.y > min_height + 0.1 {
                transform.translation.y -= FLY_SPEED * delta_time;
            }
        }
    } else {
        // Walking mode: apply gravity
        transform.translation.y += GRAVITY * delta_time;

        // Enforce minimum height
        let min_height = GROUND_HEIGHT + AVATAR_HEIGHT / 2.0;
        if transform.translation.y < min_height {
            transform.translation.y = min_height;
        }
    }

    // Update rotation - add PI so fox head faces movement direction (model faces -Z)
    transform.rotation = Quat::from_rotation_y(avatar_state.rotation + std::f32::consts::PI);

    // Update avatar state position to match transform (important for camera following)
    avatar_state.position = transform.translation;
    avatar_state.display_position = transform.translation;
}
