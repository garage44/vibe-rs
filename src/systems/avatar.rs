use bevy::prelude::*;
use crate::components::Avatar;
use crate::resources::AvatarState;

const WALK_SPEED: f32 = 8.0;
const FLY_SPEED: f32 = 40.0;
const ROTATION_SPEED: f32 = 2.0;
const GRAVITY: f32 = -9.8;
const AVATAR_HEIGHT: f32 = 0.6;
const GROUND_HEIGHT: f32 = 0.0;

#[derive(Component)]
struct AvatarMesh;

pub fn spawn_avatar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    avatar_query: Query<Entity, (With<Avatar>, Without<AvatarMesh>)>,
) {
    for entity in avatar_query.iter() {
        // Create simple capsule avatar
        let body_mesh = meshes.add(Capsule3d::new(0.3, 1.2));
        let head_mesh = meshes.add(Sphere::new(0.25));

        let body_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.29, 0.56, 0.89), // #4a90e2
            ..default()
        });

        let head_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.99, 0.74, 0.71), // #fdbcb4
            ..default()
        });

        commands.entity(entity).insert((
            Transform::from_xyz(0.0, 2.2, 0.0),
            Visibility::default(),
            AvatarMesh,
        ));

        // Spawn body
        commands.spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_material),
            Transform::from_xyz(0.0, 1.0, 0.0),
            Parent(entity),
        ));

        // Spawn head
        commands.spawn((
            Mesh3d(head_mesh),
            MeshMaterial3d(head_material),
            Transform::from_xyz(0.0, 2.0, 0.0),
            Parent(entity),
        ));
    }
}

pub fn handle_avatar_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut avatar_query: Query<&mut Transform, With<Avatar>>,
    mut avatar_state: ResMut<AvatarState>,
) {
    if avatar_query.is_empty() {
        return;
    }

    let mut transform = avatar_query.single_mut();
    let delta_time = time.delta_secs();

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

        transform.translation.x += move_direction.x;
        transform.translation.z += move_direction.z;
        avatar_state.is_walking = true;

        // Handle rotation
        if move_left || move_right {
            let rotation_delta = if move_left { 1.0 } else { -1.0 } * ROTATION_SPEED * delta_time;
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

    // Update rotation
    transform.rotation = Quat::from_rotation_y(avatar_state.rotation);

    // Update avatar state position
    avatar_state.position = transform.translation;
}
