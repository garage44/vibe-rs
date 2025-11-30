# Vibe-RS

A Bevy-based 3D virtual world application.

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

## Development

### Prerequisites

- Rust and Cargo installed (see [rustup.rs](https://rustup.rs/))

### Running in Development Mode

To run the application with debug symbols and hot reloading:

```bash
cargo run
```

This will:
1. Compile the project in debug mode (faster compilation, slower runtime)
2. Initialize the SQLite database at `data/regions.db` if it doesn't exist
3. Load regions and prims from the database
4. Render regions as planes and prims as 3D shapes
5. Spawn an avatar that you can control

### Building for Release

To build an optimized release version:

```bash
cargo build --release
```

The optimized binary will be in `target/release/vibers-rs`.

### Development Workflow

1. **Make changes** to source files in `src/`
2. **Run** `cargo run` to test changes
3. **Check for errors** - Rust's compiler will catch type errors and many logic errors at compile time
4. **Iterate** - Bevy's ECS architecture makes it easy to add new systems and components

### Adding New Features

The project uses Bevy's Entity Component System (ECS) architecture:

- **Components** (`src/components.rs`): Data attached to entities (e.g., `Region`, `Prim`, `Avatar`)
- **Resources** (`src/resources.rs`): Global state (e.g., `Database`, `GameState`, `AvatarState`)
- **Systems** (`src/systems/`): Logic that operates on components and resources

To add a new feature:
1. Define components/resources in their respective files
2. Create a system function in `src/systems/`
3. Register the system in `src/main.rs` using `.add_systems()`

### Project Structure

```
vibers-rs/
├── Cargo.toml          # Rust project configuration and dependencies
├── src/
│   ├── main.rs         # Application entry point - sets up Bevy app and registers systems
│   ├── components.rs   # ECS components (Region, Prim, Avatar, PrimShape)
│   ├── resources.rs    # Global resources (Database, GameState, AvatarState)
│   ├── systems/        # Game systems - logic that runs each frame
│   │   ├── mod.rs      # Module exports
│   │   ├── database.rs # Database initialization and data loading
│   │   ├── avatar.rs   # Avatar movement and controls
│   │   ├── camera.rs   # Camera positioning and following
│   │   └── rendering.rs # Spawning meshes for regions and prims
│   └── db/
│       └── schema.rs   # Database schema definitions and initialization
└── data/
    └── regions.db      # SQLite database (created automatically on first run)
```

### Key Files

- **`src/main.rs`**: Entry point that initializes Bevy, sets up plugins, and registers all systems
- **`src/components.rs`**: Component definitions for entities (Region, Prim, Avatar)
- **`src/resources.rs`**: Resource definitions for global state
- **`src/systems/database.rs`**: Handles SQLite database connection and loading data
- **`src/systems/avatar.rs`**: Implements avatar movement, physics, and input handling
- **`src/systems/camera.rs`**: Manages camera positioning and following the avatar
- **`src/systems/rendering.rs`**: Creates and spawns 3D meshes for regions and prims
- **`src/db/schema.rs`**: Database schema and table creation logic

## Database Schema

The application uses SQLite with two main tables:

- **regions**: Stores region data with geographic coordinates (latitude, longitude, tile coordinates)
- **prims**: Stores 3D primitive objects with position, rotation, scale, and color

The database is automatically initialized on first run at `data/regions.db`. See `src/db/schema.rs` for the complete schema definition.

## Troubleshooting

### Build Errors on Steam Deck

If you encounter build errors, reinstall the missing packages:

1. **Missing `libudev.pc`**:
   ```bash
   sudo pacman -S --overwrite '*' systemd-libs
   ```

2. **Missing C compiler/headers (`stdio.h` or `linux/types.h: No such file or directory`)**:
   ```bash
   sudo pacman -S --overwrite '*' glibc base-devel linux-api-headers
   ```

After installing, rebuild:
```bash
cargo clean
cargo build --release
```

### Other Issues

- **Database errors**: Ensure the `data/` directory exists and is writable
- **Compilation errors**: Run `cargo clean` and rebuild if you encounter strange build errors
- **Performance issues**: Use `cargo run --release` for better performance (slower compilation but faster runtime)
