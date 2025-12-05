use bevy::prelude::*;
use crate::components::Avatar;
use crate::resources::AvatarState;

#[derive(Component)]
pub struct AvatarAnimationState {
    pub walk_animation: Handle<AnimationClip>,
    pub idle_animation: Handle<AnimationClip>,
    pub current_animation: Option<Handle<AnimationClip>>,
}

const WALK_SPEED: f32 = 8.0;
const FLY_SPEED: f32 = 40.0;
const ROTATION_SPEED: f32 = 2.0;
const GRAVITY: f32 = -9.8;
const AVATAR_HEIGHT: f32 = 0.6;
const GROUND_HEIGHT: f32 = 0.0;

#[derive(Component)]
pub struct AvatarMesh;

pub fn spawn_avatar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    avatar_query: Query<Entity, (With<Avatar>, Without<AvatarMesh>)>,
) {
    for entity in avatar_query.iter() {
        // Load the fox GLTF model - using the same path structure as foxtrot
        // The model path is "models/fox/Fox.gltf#Scene0"
        // Note: Bevy 0.12 doesn't have load_with_settings, so we use simple load
        let fox_handle: Handle<Scene> = asset_server.load("models/fox/Fox.gltf#Scene0");

        // Load animations from the GLTF file
        // Based on foxtrot: Animation0 = run, Animation1 = idle, Animation2 = walk
        let walk_animation: Handle<AnimationClip> = asset_server.load("models/fox/Fox.gltf#Animation2");
        let idle_animation: Handle<AnimationClip> = asset_server.load("models/fox/Fox.gltf#Animation1");

        commands.entity(entity).insert((
            Visibility::Visible,
            AvatarMesh,
            AvatarAnimationState {
                walk_animation: walk_animation.clone(),
                idle_animation: idle_animation.clone(),
                current_animation: None,
            },
        ));

        // Spawn the fox scene as a child - it will automatically follow the avatar's transform
        // Rotate 180 degrees around Y axis to face forward (fox model faces backward by default)
        commands.entity(entity).with_children(|parent| {
            parent.spawn(SceneBundle {
                scene: fox_handle.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)), // Rotate to face forward
                ..default()
            });
        });
    }
}

pub fn play_avatar_animations(
    mut animation_query: Query<&mut AnimationPlayer>,
    mut avatar_query: Query<(Entity, &mut AvatarAnimationState), With<Avatar>>,
    avatar_state: Res<AvatarState>,
    children_query: Query<&Children>,
) {
    for (avatar_entity, mut anim_state) in avatar_query.iter_mut() {
        // Find AnimationPlayer in the scene hierarchy by traversing children recursively
        if let Ok(children) = children_query.get(avatar_entity) {
            // Recursively search for AnimationPlayer in children
            let mut entities_to_check = children.iter().copied().collect::<Vec<_>>();

            while let Some(entity) = entities_to_check.pop() {
                // Check if this entity has an AnimationPlayer
                if let Ok(mut player) = animation_query.get_mut(entity) {
                    // Determine which animation to play
                    let target_animation = if avatar_state.is_walking && !avatar_state.is_flying {
                        &anim_state.walk_animation
                    } else {
                        &anim_state.idle_animation
                    };

                    // Only change animation if it's different from current
                    if anim_state.current_animation.as_ref() != Some(target_animation) {
                        player.start(target_animation.clone()).repeat();
                        anim_state.current_animation = Some(target_animation.clone());
                    }
                    break; // Found and updated, no need to check other entities
                }

                // Add children of this entity to the search queue
                if let Ok(child_children) = children_query.get(entity) {
                    entities_to_check.extend(child_children.iter());
                }
            }
        }
    }
}

pub fn handle_avatar_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut avatar_query: Query<&mut Transform, With<Avatar>>,
    mut avatar_state: ResMut<AvatarState>,
    camera_state: Res<crate::resources::CameraState>,
) {
    // Don't move avatar if in free camera mode (camera handles movement)
    if camera_state.mode == crate::resources::CameraMode::Free {
        return;
    }
    if avatar_query.is_empty() {
        return;
    }

    let mut transform = avatar_query.single_mut();
    let delta_time = time.delta().as_secs_f32();

    // Sync initial position from transform if needed
    if (avatar_state.position - transform.translation).length() > 0.1 {
        avatar_state.position = transform.translation;
    }

    // Toggle fly mode with F key
    if keyboard_input.just_pressed(KeyCode::F) {
        avatar_state.is_flying = !avatar_state.is_flying;
        println!("Fly mode: {}", if avatar_state.is_flying { "ON" } else { "OFF" });
    }

    // Movement input
    let move_forward = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    let move_backward = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
    let move_left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let move_right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    let fly_up = keyboard_input.pressed(KeyCode::Space);
    let fly_down = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    // Calculate movement direction in world space
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

    // Calculate movement delta
    let mut move_delta = Vec3::ZERO;
    if move_direction.length() > 0.0 {
        move_direction = move_direction.normalize();
        let speed = if avatar_state.is_flying { FLY_SPEED } else { WALK_SPEED };
        let move_vector = move_direction * speed * delta_time;

        // Rotate movement direction based on avatar rotation (Y-axis rotation)
        let rotation_quat = Quat::from_rotation_y(avatar_state.rotation);
        move_delta = rotation_quat * move_vector;
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
    let mut final_translation = transform.translation + move_delta;
    if avatar_state.is_flying {
        if fly_up {
            final_translation.y += FLY_SPEED * delta_time;
        } else if fly_down {
            let min_height = GROUND_HEIGHT + AVATAR_HEIGHT / 2.0;
            if final_translation.y > min_height + 0.1 {
                final_translation.y -= FLY_SPEED * delta_time;
            }
        }
    } else {
        // Walking mode: apply gravity
        final_translation.y += GRAVITY * delta_time;

        // Enforce minimum height
        let min_height = GROUND_HEIGHT + AVATAR_HEIGHT / 2.0;
        if final_translation.y < min_height {
            final_translation.y = min_height;
        }
    }

    // Update transform fields directly - this ensures Bevy's change detection works
    // Modifying fields directly is more reliable than replacing the entire struct
    let old_pos = transform.translation;
    transform.translation = final_translation;
    transform.rotation = Quat::from_rotation_y(avatar_state.rotation);

    // Debug: print movement and verify transform is actually changing
    if move_delta.length() > 0.001 || (old_pos - final_translation).length() > 0.001 {
        println!("Avatar transform updated: {:?} -> {:?}, move_delta={:?}", old_pos, transform.translation, move_delta);
    }

    // Update avatar state position to match transform (important for camera following)
    avatar_state.position = transform.translation;
}
