use anyhow::Context;
use glam::Vec3;
use rusqlite::Connection;
use vibe_core::world::{lat_lng_to_tile, REGION_ZOOM_LEVEL};
use vibe_core::{PrimDto, RegionDto};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub fn open_and_migrate(path: &str) -> anyhow::Result<Connection> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create_dir_all {parent:?}"))?;
    }
    let mut conn = Connection::open(path).with_context(|| format!("open sqlite {path}"))?;
    embedded::migrations::runner()
        .run(&mut conn)
        .context("refinery migrate")?;
    seed_default_region(&conn)?;
    Ok(conn)
}

fn seed_default_region(conn: &Connection) -> anyhow::Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM regions", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let groningen_lat = 53.2194_f64;
    let groningen_lng = 6.5665_f64;
    let (tile_x, tile_y) = lat_lng_to_tile(groningen_lat, groningen_lng, REGION_ZOOM_LEVEL);
    conn.execute(
        "INSERT INTO regions (name, latitude, longitude, tile_x, tile_y, tile_z, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))",
        rusqlite::params![
            "Groningen",
            groningen_lat,
            groningen_lng,
            tile_x,
            tile_y,
            REGION_ZOOM_LEVEL as i64,
        ],
    )?;
    tracing::info!("seeded default region Groningen");
    Ok(())
}

pub fn load_world(conn: &Connection) -> anyhow::Result<(Vec<RegionDto>, Vec<PrimDto>)> {
    let mut stmt = conn.prepare("SELECT id, name, latitude, longitude, tile_x, tile_y, tile_z FROM regions ORDER BY id")?;
    let regions = stmt
        .query_map([], |row| {
            Ok(RegionDto {
                id: row.get(0)?,
                name: row.get(1)?,
                latitude: row.get(2)?,
                longitude: row.get(3)?,
                tile_x: row.get(4)?,
                tile_y: row.get(5)?,
                tile_z: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT id, region_id, name, shape, position_x, position_y, position_z,
                rotation_x, rotation_y, rotation_z, scale_x, scale_y, scale_z,
                color_r, color_g, color_b FROM prims ORDER BY id",
    )?;
    let prims = stmt
        .query_map([], |row| {
            Ok(PrimDto {
                id: row.get(0)?,
                region_id: row.get(1)?,
                name: row.get(2)?,
                shape: row.get(3)?,
                position: Vec3::new(row.get(4)?, row.get(5)?, row.get(6)?),
                rotation: Vec3::new(row.get(7)?, row.get(8)?, row.get(9)?),
                scale: Vec3::new(row.get(10)?, row.get(11)?, row.get(12)?),
                color: [row.get(13)?, row.get(14)?, row.get(15)?],
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok((regions, prims))
}
