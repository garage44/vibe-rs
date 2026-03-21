use glam::Vec3;
use std::collections::HashMap;
use vibe_core::{AvatarStateDto, NetMessage, PrimDto, RegionDto};

struct AvatarSim {
    position: Vec3,
    yaw: f32,
    velocity: Vec3,
    fly_vertical: f32,
}

pub struct SimWorld {
    regions: Vec<RegionDto>,
    prims: Vec<PrimDto>,
    /// Region id -> approximate sim origin (for AOI); v0 single region at origin.
    region_sim_origin: HashMap<i64, Vec3>,
    avatars: HashMap<u64, AvatarSim>,
    next_avatar_id: u64,
    observer: Vec3,
    aoi_radius_sq: f32,
}

impl SimWorld {
    pub fn new(regions: Vec<RegionDto>, prims: Vec<PrimDto>, aoi_radius: f32) -> Self {
        let mut region_sim_origin = HashMap::new();
        let n = regions.len().max(1);
        let grid_size = (n as f32).sqrt().ceil() as usize;
        let spacing = 300.0_f32;
        let grid_f = grid_size as f32;
        for (i, r) in regions.iter().enumerate() {
            let row = i / grid_size;
            let col = i % grid_size;
            let pos = Vec3::new(
                (col as f32 - grid_f / 2.0) * spacing,
                0.0,
                (row as f32 - grid_f / 2.0) * spacing,
            );
            region_sim_origin.insert(r.id, pos);
        }
        Self {
            regions,
            prims,
            region_sim_origin,
            avatars: HashMap::new(),
            next_avatar_id: 1,
            observer: Vec3::ZERO,
            aoi_radius_sq: aoi_radius * aoi_radius,
        }
    }

    pub fn spawn_avatar(&mut self) -> u64 {
        let id = self.next_avatar_id;
        self.next_avatar_id += 1;
        let start = *self
            .region_sim_origin
            .values()
            .next()
            .unwrap_or(&Vec3::ZERO);
        self.avatars.insert(
            id,
            AvatarSim {
                position: start + Vec3::new(0.0, 1.0, 0.0),
                yaw: 0.0,
                velocity: Vec3::ZERO,
                fly_vertical: 0.0,
            },
        );
        id
    }

    pub fn set_observer(&mut self, p: Vec3) {
        self.observer = p;
    }

    pub fn apply_intent(&mut self, avatar_id: u64, move_x: f32, move_z: f32, fly_up: bool, fly_down: bool) {
        let Some(av) = self.avatars.get_mut(&avatar_id) else {
            return;
        };
        let speed = 8.0_f32;
        let mut v = Vec3::new(move_x, 0.0, move_z);
        if v.length_squared() > 1.0 {
            v = v.normalize() * speed;
        } else {
            v *= speed;
        }
        av.velocity.x = v.x;
        av.velocity.z = v.z;
        let fly_speed = 5.0_f32;
        av.fly_vertical = if fly_up {
            fly_speed
        } else if fly_down {
            -fly_speed
        } else {
            0.0
        };
    }

    pub fn step(&mut self, dt: f32) {
        for av in self.avatars.values_mut() {
            av.position.x += av.velocity.x * dt;
            av.position.z += av.velocity.z * dt;
            av.position.y += av.fly_vertical * dt;
            if av.position.y < 0.0 {
                av.position.y = 0.0;
            }
        }
    }

    /// ADR-012: filter regions/prims by distance from observer to region origin (v0 heuristic).
    pub fn snapshot(&self, tick: u64) -> NetMessage {
        let regions: Vec<RegionDto> = self
            .regions
            .iter()
            .filter(|r| {
                let Some(origin) = self.region_sim_origin.get(&r.id) else {
                    return true;
                };
                (*origin - self.observer).length_squared() <= self.aoi_radius_sq
            })
            .cloned()
            .collect();

        let region_ids: std::collections::HashSet<i64> = regions.iter().map(|r| r.id).collect();
        let prims: Vec<PrimDto> = self
            .prims
            .iter()
            .filter(|p| region_ids.contains(&p.region_id))
            .cloned()
            .collect();

        let avatars: Vec<AvatarStateDto> = self
            .avatars
            .iter()
            .map(|(&id, a)| AvatarStateDto {
                id,
                position: a.position,
                yaw: a.yaw,
            })
            .collect();

        NetMessage::WorldSnapshot {
            tick,
            regions,
            prims,
            avatars,
        }
    }
}
