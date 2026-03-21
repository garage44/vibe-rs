use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Server configuration (ADR-014).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimConfig {
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default = "default_db")]
    pub database_path: String,
    #[serde(default = "default_tick_hz")]
    pub tick_hz: f32,
    #[serde(default = "default_aoi")]
    pub aoi_radius: f32,
}

fn default_listen() -> String {
    "127.0.0.1:4747".into()
}

fn default_db() -> String {
    "data/regions.db".into()
}

fn default_tick_hz() -> f32 {
    20.0
}

fn default_aoi() -> f32 {
    500.0
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            database_path: default_db(),
            tick_hz: default_tick_hz(),
            aoi_radius: default_aoi(),
        }
    }
}

impl SimConfig {
    /// `vibe.toml` (optional) + env `VIBE_*` (e.g. `VIBE_listen`, `VIBE_database_path`).
    pub fn load() -> anyhow::Result<Self> {
        let mut figment = Figment::new().merge(Serialized::defaults(SimConfig::default()));
        if Path::new("vibe.toml").exists() {
            figment = figment.merge(Toml::file("vibe.toml"));
        }
        figment = figment.merge(Env::prefixed("VIBE_"));
        Ok(figment.extract()?)
    }
}
