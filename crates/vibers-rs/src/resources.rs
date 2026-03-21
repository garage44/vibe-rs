use bevy::prelude::*;
use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard};
use tokio::sync::mpsc::UnboundedSender;
use vibe_core::NetMessage;

#[derive(Resource)]
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

#[derive(Resource)]
pub struct CameraState {
    pub mode: CameraMode,
    pub distance: f32,
    pub azimuth: f32,
    pub pitch: f32,
    pub pan_offset: Option<Vec2>, // Last mouse position for delta calculation
    pub free_camera_rotation: Vec2, // pitch, yaw
}

#[derive(Resource, Default)]
pub struct MouseState {
    pub last_position: Option<Vec2>,
    pub is_dragging: bool,
    pub is_panning: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CameraMode {
    Avatar, // Default: camera follows avatar
    Free,   // Free camera mode (FPS-style)
}

impl Default for CameraMode {
    fn default() -> Self {
        CameraMode::Avatar
    }
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            mode: CameraMode::Avatar,
            distance: 5.0,
            azimuth: 0.0,
            pitch: std::f32::consts::PI / 6.0,
            pan_offset: None,
            free_camera_rotation: Vec2::new(0.0, 0.0),
        }
    }
}

/// When set, client connects to `vibers-sim` instead of loading local SQLite world.
#[derive(Resource, Clone)]
pub struct ConnectAddr(pub String);

#[derive(Resource)]
pub struct OnlineSession {
    pub intent_tx: UnboundedSender<NetMessage>,
}

/// Incoming messages from the network thread (`Receiver` is not `Sync`; wrap in `Mutex`).
#[derive(Resource)]
pub struct NetworkMailbox {
    pub rx: Mutex<std::sync::mpsc::Receiver<NetMessage>>,
}

impl NetworkMailbox {
    pub fn lock_rx(&self) -> MutexGuard<'_, std::sync::mpsc::Receiver<NetMessage>> {
        self.rx.lock().expect("network mailbox mutex poisoned")
    }
}
