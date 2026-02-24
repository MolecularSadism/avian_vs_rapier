# Avian vs Rapier

Minimal Bevy tech demo for comparing **Avian** and **Rapier** physics performance — both 2D and 3D — all in a single binary with no feature flags required.
Balls spawn into a walled pit at a fixed rate while an FPS counter and ball counter track performance.

## Running

```sh
# Dev build (default — hot-reload, dynamic linking)
cargo run

# Release build (no dev overhead)
cargo run --release
```

That's it. All four physics modes are compiled in and switchable at runtime.

## Controls

| Key       | Action                                      |
|-----------|---------------------------------------------|
| `Enter`   | Cycle to next mode (Avian 2D → Avian 3D → Rapier 2D → Rapier 3D → …) |
| `1`       | Switch to Avian 2D                          |
| `2`       | Switch to Avian 3D                          |
| `3`       | Switch to Rapier 2D                         |
| `4`       | Switch to Rapier 3D                         |
| `Space`   | Pause / unpause simulation                  |

The active mode is shown in the **top-centre** of the screen. FPS is top-left, ball count top-right.

On a mode switch all physics entities (walls + balls) are despawned automatically via Bevy's `StateScoped` and the new mode's walls are respawned immediately.

## Bevy version swapping

The project defaults to **Bevy 0.18**. To test against older versions, change the
dependency versions in `Cargo.toml` according to this table:

| Bevy | avian2d/3d | bevy_rapier2d/3d |
|------|-----------|------------------|
| 0.18 | 0.5       | git: `Buncys/bevy_rapier` branch `bevy-0.18.0` (TODO: upgrade to 0.33) |
| 0.17 | 0.4       | 0.32 |
| 0.16 | 0.3       | 0.30 |

## Tweakable constants

| Constant         | File         | Default | Description            |
|------------------|--------------|---------|------------------------|
| `SPAWN_INTERVAL` | `spawner.rs` | 0.05s   | Time between ball spawns |
| `BALL_RADIUS`    | `spawner.rs` | 1.5 px  | Ball radius (diameter 3 px) |
| `WALL_THICKNESS` | `walls.rs`   | 10 px   | Wall thickness at screen edges |

## Project structure

```
src/
  main.rs      App setup, OnEnter systems, camera management, HUD, input
  backend.rs   PhysicsMode state, physics plugins, spawn_wall / spawn_ball helpers
  walls.rs     Floor + side walls at screen edges (no top wall)
  spawner.rs   Timed ball spawner
build.rs       No-op (physics backend is runtime-switchable, no build-time config needed)
```
