use rusqlite::{Connection, Result};

pub fn init_database(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Create regions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS regions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            tile_x INTEGER NOT NULL,
            tile_y INTEGER NOT NULL,
            tile_z INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Create index on tile coordinates
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_regions_tile ON regions(tile_x, tile_y, tile_z)",
        [],
    )?;

    // Create prims table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prims (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            region_id INTEGER NOT NULL,
            name TEXT NOT NULL DEFAULT 'Prim',
            shape TEXT NOT NULL DEFAULT 'box',
            position_x REAL NOT NULL DEFAULT 0,
            position_y REAL NOT NULL DEFAULT 0,
            position_z REAL NOT NULL DEFAULT 0,
            rotation_x REAL NOT NULL DEFAULT 0,
            rotation_y REAL NOT NULL DEFAULT 0,
            rotation_z REAL NOT NULL DEFAULT 0,
            scale_x REAL NOT NULL DEFAULT 1,
            scale_y REAL NOT NULL DEFAULT 1,
            scale_z REAL NOT NULL DEFAULT 1,
            color_r REAL NOT NULL DEFAULT 0.5,
            color_g REAL NOT NULL DEFAULT 0.5,
            color_b REAL NOT NULL DEFAULT 0.5,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (region_id) REFERENCES regions(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create index on region_id
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prims_region ON prims(region_id)",
        [],
    )?;

    Ok(conn)
}

#[derive(Debug, Clone)]
pub struct RegionRow {
    pub id: i64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub tile_x: i64,
    pub tile_y: i64,
    pub tile_z: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct PrimRow {
    pub id: i64,
    pub region_id: i64,
    pub name: String,
    pub shape: String,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub created_at: String,
    pub updated_at: String,
}
