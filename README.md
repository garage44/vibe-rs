# Vibers RS

A Bevy-based 3D virtual world application ported from Three.js/React/Bun.

## Features

- **Region Storage**: SQLite database for storing regions with geographic coordinates
- **Prim Storage**: SQLite database for storing 3D primitives (boxes, spheres, cylinders, cones, toruses)
- **Avatar Movement**: Walk and fly modes with WASD controls
- **Camera System**: Third-person camera following the avatar

## Controls

- **WASD / Arrow Keys**: Move avatar
- **Space**: Fly up (when in fly mode)
- **Shift**: Fly down (when in fly mode)
- **F**: Toggle fly/walk mode

## Building

Make sure you have Rust and Cargo installed. Then:

```bash
cd vibers-rs
cargo build --release
```

## Running

```bash
cargo run
```

The application will:
1. Initialize the SQLite database at `data/regions.db`
2. Load regions and prims from the database
3. Render regions as planes and prims as 3D shapes
4. Spawn an avatar that you can control

## Project Structure

```
vibers-rs/
├── Cargo.toml          # Rust project configuration
├── src/
│   ├── main.rs         # Application entry point
│   ├── components.rs  # ECS components (Region, Prim, Avatar)
│   ├── resources.rs    # Game resources (Database, GameState, AvatarState)
│   ├── systems/        # Game systems
│   │   ├── mod.rs
│   │   ├── database.rs # Database operations
│   │   ├── avatar.rs   # Avatar movement
│   │   ├── camera.rs   # Camera controls
│   │   └── rendering.rs # Rendering logic
│   └── db/
│       └── schema.rs   # Database schema
└── data/
    └── regions.db      # SQLite database (created on first run)
```

## Database Schema

The application uses SQLite with two main tables:

- **regions**: Stores region data with geographic coordinates
- **prims**: Stores 3D primitive objects with position, rotation, scale, and color

See `src/db/schema.rs` for the complete schema definition.
