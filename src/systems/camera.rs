use bevy::prelude::*;
use crate::components::Avatar;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));
}

pub fn follow_avatar_camera(
    avatar_query: Query<&Transform, (With<Avatar>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Avatar>)>,
) {
    if avatar_query.is_empty() || camera_query.is_empty() {
        return;
    }

    let avatar_transform = avatar_query.single();
    let mut camera_transform = camera_query.single_mut();

    // Third-person camera: position behind and above avatar
    let offset = Vec3::new(0.0, 5.0, 10.0);
    let camera_position = avatar_transform.translation + offset;

    // Look at avatar
    camera_transform.translation = camera_position;
    camera_transform.look_at(avatar_transform.translation + Vec3::Y * 1.0, Vec3::Y);
}
