use bevy::prelude::*;
use crate::components::Avatar;
use crate::resources::{AvatarState, CameraState, CameraMode};
use crate::systems::rendering::RegionMesh;

#[derive(Component)]
pub struct FreeCamera;

// Simple constants
const FREE_CAMERA_SPEED: f32 = 20.0;
const FREE_CAMERA_SPEED_FAST: f32 = 50.0;
const MOUSE_SENSITIVITY: f32 = 0.002;
const MIN_CAMERA_HEIGHT: f32 = 0.5;

pub fn setup_camera(mut commands: Commands, mut camera_state: ResMut<CameraState>) {
    camera_state.mode = CameraMode::Avatar;
    camera_state.distance = 5.0;
    camera_state.azimuth = 0.0;
    camera_state.pitch = std::f32::consts::PI / 6.0;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FreeCamera,
    ));
}

pub fn camera_mode_toggle(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_state: ResMut<CameraState>,
    camera_query: Query<&Transform, With<FreeCamera>>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        camera_state.mode = match camera_state.mode {
            CameraMode::Avatar => {
                // Initialize free camera rotation from current camera orientation
                if let Ok(camera_transform) = camera_query.get_single() {
                    let euler = camera_transform.rotation.to_euler(EulerRot::YXZ);
                    camera_state.free_camera_rotation = Vec2::new(euler.1, euler.0);
                }
                CameraMode::Free
            }
            CameraMode::Free => CameraMode::Avatar,
        };
        println!("Camera mode: {:?}", camera_state.mode);
    }
}

pub fn camera_controls(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut Transform, With<FreeCamera>>,
    mut camera_state: ResMut<CameraState>,
    avatar_state: Res<AvatarState>,
    time: Res<Time>,
    region_mesh_query: Query<&GlobalTransform, With<RegionMesh>>,
) {
    if camera_query.is_empty() {
        return;
    }

    let mut camera_transform = camera_query.single_mut();
    let delta_time = time.delta().as_secs_f32();

    match camera_state.mode {
        CameraMode::Avatar => {
            // Third-person camera following avatar
            // Use avatar_state.position which is synced from transform by avatar movement system
            let avatar_pos = avatar_state.position;

            // Handle mouse wheel zoom
            for event in mouse_wheel_events.read() {
                camera_state.distance += event.y * 0.1;
                camera_state.distance = camera_state.distance.max(2.0).min(100.0);
            }

            // Handle mouse drag for rotation
            if mouse_input.pressed(MouseButton::Left) {
                for event in cursor_moved_events.read() {
                    if let Some(last_pos) = camera_state.pan_offset {
                        let delta = event.position - last_pos;
                        camera_state.azimuth -= delta.x * 0.005;
                        camera_state.pitch -= delta.y * 0.005;
                        camera_state.pitch = camera_state.pitch
                            .max(-std::f32::consts::PI / 3.0)
                            .min(std::f32::consts::PI / 2.5);
                    }
                    camera_state.pan_offset = Some(event.position);
                }
            } else if mouse_input.just_released(MouseButton::Left) {
                camera_state.pan_offset = None;
            }

            // Calculate camera position
            let horizontal_distance = camera_state.distance * camera_state.pitch.cos();
            let vertical_offset = camera_state.distance * camera_state.pitch.sin();

            let camera_offset = Vec3::new(
                camera_state.azimuth.sin() * horizontal_distance,
                vertical_offset + 1.5,
                camera_state.azimuth.cos() * horizontal_distance,
            );

            let mut target_position = avatar_pos + camera_offset;

            // Prevent camera from going below ground
            let ground_height = get_ground_height(target_position, &region_mesh_query);
            let min_height = ground_height + MIN_CAMERA_HEIGHT;
            if target_position.y < min_height {
                target_position.y = min_height;
            }

            // Smooth camera movement
            camera_transform.translation = camera_transform.translation.lerp(target_position, 0.1);
            camera_transform.look_at(avatar_pos + Vec3::Y * 1.5, Vec3::Y);
        }
        CameraMode::Free => {
            // Free camera mode - simple FPS style
            let mut rotation_delta = Vec2::ZERO;

            // Mouse look (right mouse button)
            if mouse_input.pressed(MouseButton::Right) {
                for event in cursor_moved_events.read() {
                    if let Some(last_pos) = camera_state.pan_offset {
                        let delta = event.position - last_pos;
                        rotation_delta.x += delta.x;
                        rotation_delta.y += delta.y;
                    }
                    camera_state.pan_offset = Some(event.position);
                }
            } else {
                camera_state.pan_offset = None;
            }

            // Update rotation from mouse input
            if rotation_delta.length() > 0.0 {
                camera_state.free_camera_rotation.y -= rotation_delta.x * MOUSE_SENSITIVITY;
                camera_state.free_camera_rotation.x -= rotation_delta.y * MOUSE_SENSITIVITY;
                camera_state.free_camera_rotation.x = camera_state.free_camera_rotation.x
                    .max(-std::f32::consts::PI / 2.0 + 0.1)
                    .min(std::f32::consts::PI / 2.0 - 0.1);
            }

            // Apply rotation
            let rotation_quat = Quat::from_euler(
                EulerRot::YXZ,
                camera_state.free_camera_rotation.y,
                camera_state.free_camera_rotation.x,
                0.0,
            );
            camera_transform.rotation = rotation_quat;

            // Movement - calculate forward/right from rotation, keep vertical independent
            let forward = rotation_quat * -Vec3::Z;
            let right = rotation_quat * Vec3::X;

            // Flatten to horizontal plane for WASD movement
            let forward_horizontal = Vec3::new(forward.x, 0.0, forward.z).normalize();
            let right_horizontal = Vec3::new(right.x, 0.0, right.z).normalize();

            let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
                FREE_CAMERA_SPEED_FAST
            } else {
                FREE_CAMERA_SPEED
            };

            let mut move_direction = Vec3::ZERO;

            if keyboard_input.pressed(KeyCode::W) {
                move_direction += forward_horizontal;
            }
            if keyboard_input.pressed(KeyCode::S) {
                move_direction -= forward_horizontal;
            }
            if keyboard_input.pressed(KeyCode::A) {
                move_direction -= right_horizontal;
            }
            if keyboard_input.pressed(KeyCode::D) {
                move_direction += right_horizontal;
            }

            // Vertical movement - ALWAYS world-space, never affects rotation
            if keyboard_input.pressed(KeyCode::Space) {
                move_direction += Vec3::Y;
            }
            // Q or Shift for moving down (but Shift also speeds up, so only use Shift if not moving horizontally)
            if keyboard_input.pressed(KeyCode::Q) {
                move_direction -= Vec3::Y;
            }
            // Shift alone (without WASD) moves down
            if (keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight))
                && !keyboard_input.pressed(KeyCode::W)
                && !keyboard_input.pressed(KeyCode::A)
                && !keyboard_input.pressed(KeyCode::S)
                && !keyboard_input.pressed(KeyCode::D) {
                move_direction -= Vec3::Y;
            }

            // Apply movement
            if move_direction.length() > 0.0 {
                move_direction = move_direction.normalize();
                camera_transform.translation += move_direction * speed * delta_time;
            }

            // Prevent going below ground
            let ground_height = get_ground_height(camera_transform.translation, &region_mesh_query);
            let min_height = ground_height + MIN_CAMERA_HEIGHT;
            if camera_transform.translation.y < min_height {
                camera_transform.translation.y = min_height;
            }
        }
    }
}

fn get_ground_height(position: Vec3, region_mesh_query: &Query<&GlobalTransform, With<RegionMesh>>) -> f32 {
    let mut min_height: f32 = 0.0;
    for region_transform in region_mesh_query.iter() {
        let region_y: f32 = region_transform.translation().y;
        let region_top: f32 = region_y + 0.05;
        if position.x >= region_transform.translation().x - 150.0
            && position.x <= region_transform.translation().x + 150.0
            && position.z >= region_transform.translation().z - 150.0
            && position.z <= region_transform.translation().z + 150.0 {
            min_height = min_height.max(region_top);
        }
    }
    min_height
}
