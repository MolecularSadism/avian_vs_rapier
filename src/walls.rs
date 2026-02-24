//! Floor, side walls, and (for 3D) front/back walls.
//! No top wall so balls can drop in.

use bevy::prelude::*;

use crate::backend::{self, POOL_DEPTH, PhysicsMode};

const WIDTH: f32 = 1920.0;
const HEIGHT: f32 = 1080.0;
const WALL_THICKNESS: f32 = 10.0;

pub fn spawn_walls(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mode: PhysicsMode,
) {
    let wall_color = Color::srgb(0.4, 0.4, 0.4);
    let is_3d = matches!(mode, PhysicsMode::Avian3d | PhysicsMode::Rapier3d);

    // In 3D, trim side walls so they fit between the front and back walls (no corner overlap).
    // The front/back walls are WALL_THICKNESS deep at each end, so side walls use the interior depth.
    let side_depth = if is_3d {
        POOL_DEPTH - 2.0 * WALL_THICKNESS
    } else {
        WALL_THICKNESS
    };

    // Floor — in 3D trimmed in both X and Z to fit inside the side and front/back walls.
    let floor_width = if is_3d {
        WIDTH - 2.0 * WALL_THICKNESS
    } else {
        WIDTH
    };
    backend::spawn_wall(
        commands,
        meshes,
        materials,
        mode,
        Vec3::new(0.0, -HEIGHT / 2.0 + WALL_THICKNESS / 2.0, 0.0),
        Vec3::new(floor_width, WALL_THICKNESS, side_depth),
        wall_color,
    );

    // Left wall — full height at the left edge
    backend::spawn_wall(
        commands,
        meshes,
        materials,
        mode,
        Vec3::new(-WIDTH / 2.0 + WALL_THICKNESS / 2.0, 0.0, 0.0),
        Vec3::new(WALL_THICKNESS, HEIGHT, side_depth),
        wall_color,
    );

    // Right wall — full height at the right edge
    backend::spawn_wall(
        commands,
        meshes,
        materials,
        mode,
        Vec3::new(WIDTH / 2.0 - WALL_THICKNESS / 2.0, 0.0, 0.0),
        Vec3::new(WALL_THICKNESS, HEIGHT, side_depth),
        wall_color,
    );

    if is_3d {
        // Back wall (away from camera)
        backend::spawn_wall(
            commands,
            meshes,
            materials,
            mode,
            Vec3::new(0.0, 0.0, -POOL_DEPTH / 2.0 + WALL_THICKNESS / 2.0),
            Vec3::new(WIDTH, HEIGHT, WALL_THICKNESS),
            wall_color,
        );

        // Front wall (toward camera) — semi-transparent glass so we can see inside
        backend::spawn_wall(
            commands,
            meshes,
            materials,
            mode,
            Vec3::new(0.0, 0.0, POOL_DEPTH / 2.0 - WALL_THICKNESS / 2.0),
            Vec3::new(WIDTH, HEIGHT, WALL_THICKNESS),
            Color::srgba(0.5, 0.7, 1.0, 0.15),
        );
    }
}
