//! Ball spawner — drops small balls from the top of the screen on a timer.

use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

use crate::backend::{self, BallAssets, POOL_DEPTH, PhysicsMode};

/// Time between ball spawns. Tweak this to control spawn rate.
const SPAWN_INTERVAL: Duration = Duration::from_millis(50);

/// Ball radius in pixels.
const BALL_RADIUS: f32 = 6.0;

/// Ball colour — shared across all modes.
const BALL_COLOR: Color = Color::srgb(0.9, 0.3, 0.2);

/// Horizontal spawn range (inside the walls, with a small margin).
const SPAWN_X_MIN: f32 = -945.0;
const SPAWN_X_MAX: f32 = 945.0;

/// Y position where balls appear (just below the top of screen).
const SPAWN_Y: f32 = 530.0;

/// Marker component for counting balls.
#[derive(Component)]
pub struct Ball;

/// Resource that tracks ball count for the UI.
#[derive(Resource, Default)]
pub struct BallCount(pub usize);

/// Resource that controls how many balls are spawned per timer tick.
#[derive(Resource)]
pub struct BallsPerTick(pub usize);

impl Default for BallsPerTick {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Resource)]
struct SpawnTimer(Timer);

pub fn plugin(app: &mut App) {
    app.insert_resource(SpawnTimer(Timer::new(SPAWN_INTERVAL, TimerMode::Repeating)));
    app.insert_resource(BallCount::default());
    app.insert_resource(BallsPerTick::default());
    app.add_systems(Startup, setup_ball_assets);
    app.add_systems(Update, spawn_balls);
}

fn setup_ball_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(BallAssets {
        mesh2d: meshes.add(Circle::new(BALL_RADIUS)),
        mat2d: color_materials.add(ColorMaterial::from_color(BALL_COLOR)),
        mesh3d: meshes.add(Sphere::new(BALL_RADIUS)),
        mat3d: materials.add(StandardMaterial {
            base_color: BALL_COLOR,
            ..default()
        }),
    });
}

fn spawn_balls(
    mut commands: Commands,
    ball_assets: Res<BallAssets>,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut ball_count: ResMut<BallCount>,
    balls_per_tick: Res<BallsPerTick>,
    mode: Res<State<PhysicsMode>>,
) {
    timer.0.tick(time.delta());

    let mode = *mode.get();
    let ticks = timer.0.times_finished_this_tick();

    for _ in 0..ticks {
        let mut rng = rand::rng();
        for _ in 0..balls_per_tick.0 {
            let x = rng.random_range(SPAWN_X_MIN..=SPAWN_X_MAX);
            let z = match mode {
                PhysicsMode::Avian3d | PhysicsMode::Rapier3d => {
                    let half = POOL_DEPTH / 2.0 - 40.0;
                    rng.random_range(-half..=half)
                }
                _ => 0.0,
            };
            let position = Vec3::new(x, SPAWN_Y, z);

            backend::spawn_ball(&mut commands, mode, position, BALL_RADIUS, &ball_assets);
            ball_count.0 += 1;
        }
    }
}
