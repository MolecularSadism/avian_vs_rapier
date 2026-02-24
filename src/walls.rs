//! Floor and side walls — 10 px thick, positioned at screen edges.
//! No top wall so balls can drop in.

use bevy::prelude::*;

use crate::backend::{self, PhysicsMode};

const WIDTH: f32 = 1920.0;
const HEIGHT: f32 = 1080.0;
const WALL_THICKNESS: f32 = 10.0;

pub fn spawn_walls(commands: &mut Commands, mode: PhysicsMode) {
    let wall_color = Color::srgb(0.4, 0.4, 0.4);

    // Floor — full width, at the very bottom edge
    backend::spawn_wall(
        commands,
        mode,
        Vec3::new(0.0, -HEIGHT / 2.0 + WALL_THICKNESS / 2.0, 0.0),
        WIDTH,
        WALL_THICKNESS,
        wall_color,
    );

    // Left wall — full height, at the very left edge
    backend::spawn_wall(
        commands,
        mode,
        Vec3::new(-WIDTH / 2.0 + WALL_THICKNESS / 2.0, 0.0, 0.0),
        WALL_THICKNESS,
        HEIGHT,
        wall_color,
    );

    // Right wall — full height, at the very right edge
    backend::spawn_wall(
        commands,
        mode,
        Vec3::new(WIDTH / 2.0 - WALL_THICKNESS / 2.0, 0.0, 0.0),
        WALL_THICKNESS,
        HEIGHT,
        wall_color,
    );
}
