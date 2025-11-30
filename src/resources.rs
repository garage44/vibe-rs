use bevy::prelude::*;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

#[derive(Resource, Default)]
pub struct GameState {
    pub selected_prim_id: Option<i64>,
    pub regions_loaded: bool,
    pub prims_loaded: bool,
}

#[derive(Resource)]
pub struct AvatarState {
    pub position: Vec3,
    pub rotation: f32,
    pub is_flying: bool,
    pub is_walking: bool,
}

impl Default for AvatarState {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 2.2, 0.0),
            rotation: 0.0,
            is_flying: false,
            is_walking: false,
        }
    }
}
