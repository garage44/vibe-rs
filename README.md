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

**For Fedora/Distrobox users:**
```bash
# Run the setup script (installs all required dependencies)
./setup-fedora.sh
```

### Running in Development Mode

This repo is a **workspace** (`vibe_core`, `vibers-sim`, `vibers-rs`). The game client is `vibers-rs`:

```bash
cargo run -p vibers-rs
```

**Compile-time tuning (already in root `Cargo.toml`):**
- Dependencies compile at **opt-level 1** with **many codegen units** (not O3 + single codegen unit, which makes Bevy rebuilds very slow).
- Your crate uses a similar dev profile so iteration stays reasonable.

**Optional faster incremental links** (especially after changing `vibers-rs` only):

```bash
cargo run -p vibers-rs --features fast-dev
```

This enables Bevy’s `dynamic_linking` (loads Bevy as a shared library). Use for local dev only—not for release builds you ship.

**Linker:** `.cargo/config.toml` uses **clang + lld**. Install (`pacman -S lld clang`). **mold** is often faster than lld for huge links—swap `fuse-ld=lld` for `fuse-ld=mold` if you install mold.

**Fast Compilation:**
- First full workspace build still takes a while (Bevy + deps). After that, touching only `vibers-rs` or `vibe_core` rebuilds a subset.
- Use `cargo run -p vibers-sim` when working on the server only (no Bevy).

This will:
1. Compile the project in debug mode
2. Initialize the SQLite database at `data/regions.db` if it doesn't exist
3. Load regions and prims from the database
4. Render regions as planes and prims as 3D shapes
5. Spawn an avatar that you can control

### Building for Release

```bash
cargo build -p vibers-rs --release
```

Binary: `target/release/vibers-rs` (do **not** use `--features fast-dev` for release).

### Development Workflow

1. **Make changes** under `crates/vibers-rs/src/` (or other workspace crates)
2. **Run** `cargo run -p vibers-rs` to test changes
3. **Check for errors** - Rust's compiler will catch type errors and many logic errors at compile time
4. **Iterate** - Bevy's ECS architecture makes it easy to add new systems and components

### Adding New Features

The project uses Bevy's Entity Component System (ECS) architecture:

- **Components** (`crates/vibers-rs/src/components.rs`): Data attached to entities
- **Resources** (`crates/vibers-rs/src/resources.rs`): Global state
- **Systems** (`crates/vibers-rs/src/systems/`): Logic that operates on components and resources

To add a new feature:
1. Define components/resources in their respective files
2. Create a system under `crates/vibers-rs/src/systems/`
3. Register the system in `crates/vibers-rs/src/main.rs` using `.add_systems()`

### Architecture

Architectural decisions are documented in [docs/architecture/adr/](docs/architecture/adr/). See the [ADR index](docs/architecture/adr/index.md) for technology choices (Bevy, SQLite, OSM tiles) and patterns (ECS layout, system ordering).

### Project Structure

```
vibe-rs/
├── Cargo.toml              # Workspace root (shared dev profiles)
├── crates/
│   ├── vibe_core/src/      # Shared protocol + OSM/tile types
│   ├── vibers-sim/         # Headless server binary
│   └── vibers-rs/src/      # Bevy client
│       ├── main.rs
│       ├── components.rs
│       ├── resources.rs
│       ├── systems/
│       └── db/
└── data/regions.db         # Created on first local run (client or sim)
```

### Key Files

- **`crates/vibers-rs/src/main.rs`**: Bevy app, systems, `--connect` for online mode
- **`crates/vibers-sim/src/main.rs`**: TCP sim + SQLite migrations
- **`crates/vibe_core/`**: `NetMessage`, `TileKey`, coordinate helpers

## Database Schema

The application uses SQLite with two main tables:

- **regions**: Stores region data with geographic coordinates (latitude, longitude, tile coordinates)
- **prims**: Stores 3D primitive objects with position, rotation, scale, and color

The database is initialized on first run at `data/regions.db` (client `schema.rs` locally; server uses `vibers-sim/migrations/`).

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
cargo build -p vibers-rs --release
```

### Other Issues

- **Database errors**: Ensure the `data/` directory exists and is writable
- **Compilation errors**: Run `cargo clean` and rebuild if you encounter strange build errors
- **Performance issues**: Use `cargo run --release` for better performance (slower compilation but faster runtime)
