use bevy::prelude::*;
use bevy::render::mesh::shape::UVSphere;
use crate::components::Avatar;
use crate::resources::AvatarState;

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    avatar_query: Query<Entity, (With<Avatar>, Without<AvatarMesh>)>,
) {
    for entity in avatar_query.iter() {
        // Create simple avatar (using box for body, sphere for head)
        let body_mesh = meshes.add(Mesh::from(bevy::render::mesh::shape::Box::new(0.6, 1.2, 0.6)));
        let head_mesh = meshes.add(Mesh::from(UVSphere::default()));

        let body_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.29, 0.56, 0.89), // #4a90e2
            ..default()
        });

        let head_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.99, 0.74, 0.71), // #fdbcb4
            ..default()
        });

        // Add mesh components (transform should already exist from spawn_avatar_entity)
        // Make sure avatar is visible
        commands.entity(entity).insert((
            Visibility::Visible,
            AvatarMesh,
        ));

        // Spawn children using with_children - this ensures proper parent-child relationship
        commands.entity(entity).with_children(|parent| {
            // Spawn body as child
            parent.spawn(MaterialMeshBundle {
                mesh: body_mesh,
                material: body_material,
                transform: Transform::from_xyz(0.0, 1.0, 0.0), // Local transform relative to parent
                ..default()
            });

            // Spawn head (sphere) as child
            parent.spawn(MaterialMeshBundle {
                mesh: head_mesh,
                material: head_material,
                transform: Transform::from_xyz(0.0, 2.0, 0.0), // Local transform relative to parent
                ..default()
            });
        });
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
