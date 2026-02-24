# Avian vs Rapier — Tech Demo Plan

## Goal
Minimal Bevy app that spawns balls into a walled pit, with feature gates for:
- **Physics backend**: `avian` / `rapier`
- **Dimension**: `2d` / `3d`
- **Bevy version**: `bevy16` / `bevy17` / `bevy18`

FPS + ball counter on screen in all builds (including release).

---

## 1. Strip the template

Delete everything we don't need:

- `src/asset_tracking.rs`, `src/audio.rs` — no audio/assets
- `src/demo/` (entire directory) — replaced by our scene
- `src/menus/` (entire directory) — no menus
- `src/screens/` (entire directory) — no screen state machine
- `src/theme/` (entire directory) — no UI theme
- `assets/` (entire directory) — no bundled assets
- `.github/` — not needed for demo
- `.idea/` — IDE config, not needed

Simplify `main.rs` to just boot Bevy with our plugins, camera, and fixed 1920×1080 window.

## 2. Cargo.toml — features & dependencies

### Features

```toml
[features]
default = ["avian", "2d", "bevy18"]

# Physics backend (exactly one)
avian = []
rapier = []

# Dimension (exactly one)
2d = []
3d = []

# Bevy version (exactly one)
bevy16 = []
bevy17 = []
bevy18 = []

# Dev features (keep from template)
dev = ["bevy/dynamic_linking", "bevy/bevy_dev_tools"]
dev_native = ["dev", "bevy/file_watcher", "bevy/embedded_watcher"]
```

### Dependencies (conditional)

Version matrix:
| Gate     | bevy   | avian2d | avian3d | bevy_rapier2d       | bevy_rapier3d       |
|----------|--------|---------|---------|---------------------|---------------------|
| `bevy16` | 0.16   | 0.3     | 0.3     | 0.30                | 0.30                |
| `bevy17` | 0.17   | 0.4     | 0.4     | 0.32                | 0.32                |
| `bevy18` | 0.18   | 0.5     | 0.5     | git (Buncys:bevy-0.18.0) + TODO 0.33 | same           |

Each dependency is gated on the combination of backend + dimension + bevy version, e.g.:
```toml
[target.'cfg(all())'.dependencies]
avian2d = { version = "0.5", optional = true }  # enabled by feature combo
```

Since Cargo features can't do `cfg(all(feature="avian", feature="2d", feature="bevy18"))` in `[dependencies]`, we'll use a flat approach:
- Declare all physics crates as optional dependencies
- Use a `build.rs` or `cfg_aliases` crate to create compound aliases
- In code, use `#[cfg(all(feature = "avian", feature = "2d"))]` etc.

**Approach**: Use the `cfg_aliases` crate (already in the build dep chain via Bevy) to define aliases like `avian_2d`, `rapier_3d`, etc. in `build.rs`.

Actually, simplest: just list all possible deps as optional and enable the right ones via feature prerequisites:
```toml
avian2d_v03 = { package = "avian2d", version = "0.3", optional = true }
avian2d_v04 = { package = "avian2d", version = "0.4", optional = true }
avian2d_v05 = { package = "avian2d", version = "0.5", optional = true }
# ... etc
```

**Problem**: Can't have multiple versions of the same crate. We need separate compilation targets per bevy version anyway (different Bevy ABI). So the bevy version gate is actually **which `Cargo.toml` you use**, not a runtime feature.

### Revised approach for Bevy version gating

The bevy16/bevy17/bevy18 split can't truly coexist in one Cargo.toml since `bevy = "0.16"` and `bevy = "0.18"` are incompatible. Instead:

**Option A — Workspace with 3 crates** (one per bevy version, shared `src/` via path)
**Option B — Single crate, 3 Cargo.toml variants** swapped by script
**Option C — Single Cargo.toml using `cfg_aliases` + careful optional deps** — this actually CAN work if we use `package` renames and only ever enable one set.

Going with **Option A — Cargo workspace**:

```
avian_vs_rapier/
├── Cargo.toml          (workspace)
├── shared/             (symlink or path dep with shared source)
│   └── src/
├── bevy16/
│   └── Cargo.toml      (bevy 0.16, avian 0.3, rapier 0.30)
├── bevy17/
│   └── Cargo.toml      (bevy 0.17, avian 0.4, rapier 0.32)
└── bevy18/
    └── Cargo.toml      (bevy 0.18, avian 0.5, rapier git)
```

Actually this is over-engineered. The simplest and most practical approach:

### Final approach: Single crate, pick version via features + `cfg_aliases`

We CAN have one Cargo.toml if we accept that only one bevy version compiles at a time. Use `cfg_aliases` in build.rs to create shorthand. The user picks features at build time:

```sh
cargo run --features "avian,2d"       # defaults to bevy18
cargo run --features "rapier,3d"
```

For switching Bevy versions: **just edit the version in Cargo.toml** or use a simple shell script / justfile. This is a tech demo — manual version pin swaps are fine.

**Simplification**: Drop the bevy-version feature gate from Cargo.toml. Instead, document which versions to pin for each comparison. The `backend.rs` abstraction handles avian-vs-rapier differences. The 2d/3d gate handles dimension differences. Bevy version is a manual Cargo.toml edit.

This is much cleaner. The user can still compare all 12 combinations by editing 1 line + toggling 2 features.

---

## Revised Plan (simplified)

### Features
```toml
[features]
default = ["avian", "2d", "dev_native"]
avian = []
rapier = []
2d = []
3d = []
dev = ["bevy/dynamic_linking", "bevy/bevy_dev_tools"]
dev_native = ["dev", "bevy/file_watcher", "bevy/embedded_watcher"]
```

### Dependencies (for bevy 0.18 — the default)
```toml
bevy = { version = "0.18", default-features = false, features = ["bevy_dev_tools"] }
# Enable "2d" or "3d" bevy features via cfg_aliases or manually

avian2d = { version = "0.5", optional = true }
avian3d = { version = "0.5", optional = true }
bevy_rapier2d = { git = "https://github.com/Buncys/bevy_rapier", branch = "bevy-0.18.0", optional = true }
bevy_rapier3d = { git = "https://github.com/Buncys/bevy_rapier", branch = "bevy-0.18.0", optional = true }
rand = "0.9"
```

To test bevy 0.17: change `bevy` to `0.17`, `avian2d` to `0.4`, `bevy_rapier2d` to `0.32`.
To test bevy 0.16: change `bevy` to `0.16`, `avian2d` to `0.3`, `bevy_rapier2d` to `0.30`.

A comment block at the top of Cargo.toml documents all three version sets.

### build.rs — cfg_aliases
```rust
use cfg_aliases::cfg_aliases;
fn main() {
    cfg_aliases! {
        avian_2d: { all(feature = "avian", feature = "2d") },
        avian_3d: { all(feature = "avian", feature = "3d") },
        rapier_2d: { all(feature = "rapier", feature = "2d") },
        rapier_3d: { all(feature = "rapier", feature = "3d") },
        dim2: { feature = "2d" },
        dim3: { feature = "3d" },
        use_avian: { feature = "avian" },
        use_rapier: { feature = "rapier" },
    }
}
```

## 3. File structure (new)

```
src/
├── main.rs          — App setup, window, camera, FPS overlay, ball counter UI
├── backend.rs       — Re-exports for physics: Collider, RigidBody, plugin
├── walls.rs         — Floor + side walls (feature-gated 2d/3d)
└── spawner.rs       — Ball spawner on a timer
```

## 4. `src/backend.rs` — Physics abstraction

```rust
// Unifies the API differences between avian and rapier.
// Key differences to handle:
//   - Rapier: Collider::ball(radius), Collider::cuboid(half_x, half_y)
//   - Avian:  Collider::circle(radius), Collider::rectangle(width, height)  [2D]
//   - Avian:  Collider::sphere(radius), Collider::cuboid(width, height, depth) [3D]
//   NOTE: Avian rectangle/cuboid takes FULL extents, Rapier cuboid takes HALF extents.

// Re-export the physics plugin
#[cfg(avian_2d)] pub use avian2d::prelude::*;
#[cfg(avian_3d)] pub use avian3d::prelude::*;
#[cfg(rapier_2d)] pub use bevy_rapier2d::prelude::*;
#[cfg(rapier_3d)] pub use bevy_rapier3d::prelude::*;

// Helper: make a box collider from full width/height
pub fn box_collider(width: f32, height: f32) -> Collider {
    #[cfg(use_avian)] { Collider::rectangle(width, height) }       // 2D avian
    #[cfg(use_rapier)] { Collider::cuboid(width / 2.0, height / 2.0) } // 2D rapier
    // 3D variants add depth param
}

pub fn ball_collider(radius: f32) -> Collider {
    #[cfg(use_avian)]  { Collider::circle(radius) }   // or sphere for 3d
    #[cfg(use_rapier)] { Collider::ball(radius) }
}

// Re-export the plugin to add
pub fn physics_plugin() -> impl Plugin { ... }
```

## 5. `src/walls.rs` — Scene walls

- Screen is 1920×1080. Camera is centered at (0,0).
- Walls are 10px thick, positioned at screen edges:
  - **Floor**: pos (0, -540+5, 0), size (1920, 10)
  - **Left wall**: pos (-960+5, 0, 0), size (10, 1080)
  - **Right wall**: pos (960-5, 0, 0), size (10, 1080)
- No top wall (balls drop in from above).
- All z-values are 0.0 in 2D mode.
- Walls are `RigidBody::Static` with box colliders.
- Visualized with colored sprites (simple rectangles via `Sprite`).

## 6. `src/spawner.rs` — Ball spawner

- Spawn balls at top of visible area (y ≈ 530, just below top edge).
- Random x position within the walls (roughly -950..950).
- Ball diameter = 3px → radius = 1.5px.
- `RigidBody::Dynamic` + ball collider + small colored sprite.
- Timer: `Timer::from_seconds(0.05, TimerMode::Repeating)` (50ms = 20 balls/sec).
- `SPAWN_INTERVAL` const for easy tweaking.
- Balls get a `Ball` marker component for counting.

## 7. `src/main.rs` — Minimal app

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Window {
                resolution: (1920.0, 1080.0).into(),
                title: "Avian vs Rapier".into(),
                ..default()
            }.into(),
            ..default()
        }))
        .add_plugins(backend::physics_plugin())
        .add_plugins(FpsOverlayPlugin::default())  // from bevy_dev_tools — always on
        .add_systems(Startup, (setup_camera, walls::spawn_walls, spawner::setup))
        .add_systems(Update, (spawner::spawn_balls, update_ball_counter))
        .run();
}
```

### FPS counter in release mode
- `bevy_dev_tools` must be enabled even in release. We add it to the base bevy features, not just `dev`.
- Use `FpsOverlayPlugin` from `bevy::dev_tools::fps_overlay`.

### Ball counter
- Simple `Text` entity updated each frame with `Query<&Ball>.iter().count()`.

## 8. Summary of run commands

```sh
# 2D Avian (default)
cargo run --features "avian,2d"

# 2D Rapier
cargo run --features "rapier,2d"

# 3D Avian
cargo run --features "avian,3d"

# 3D Rapier
cargo run --features "rapier,3d"

# Release benchmark
cargo run --release --no-default-features --features "avian,2d"
```

## 9. Version swap guide (comment in Cargo.toml)

```
# BEVY 0.18 (default): bevy 0.18, avian2d 0.5, bevy_rapier2d git:Buncys/bevy-0.18.0
# BEVY 0.17: bevy 0.17, avian2d 0.4, bevy_rapier2d 0.32
# BEVY 0.16: bevy 0.16, avian2d 0.3, bevy_rapier2d 0.30
```
