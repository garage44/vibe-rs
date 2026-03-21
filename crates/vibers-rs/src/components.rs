use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Region {
    pub id: i64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub tile_x: i64,
    pub tile_y: i64,
    pub tile_z: i64,
    /// When set (network snapshot), region mesh uses this sim position; otherwise legacy grid layout.
    pub sim_origin: Option<Vec3>,
}

#[derive(Component, Debug, Clone)]
pub struct Prim {
    pub id: i64,
    pub region_id: i64,
    pub name: String,
    pub shape: PrimShape,
    pub color: Color,
}

#[derive(Component, Debug, Clone)]
pub struct Avatar;

/// Another client’s avatar (sim id from `WorldSnapshot`); same fox mesh as [`Avatar`].
#[derive(Component, Debug, Clone, Copy)]
pub struct RemoteAvatar {
    pub sim_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimShape {
    Box,
    Sphere,
    Cylinder,
    Cone,
    Torus,
}

impl PrimShape {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "box" => PrimShape::Box,
            "sphere" => PrimShape::Sphere,
            "cylinder" => PrimShape::Cylinder,
            "cone" => PrimShape::Cone,
            "torus" => PrimShape::Torus,
            _ => PrimShape::Box,
        }
    }
}




