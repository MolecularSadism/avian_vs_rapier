//! Ball spawner — drops small balls from the top of the screen on a timer.

use bevy::prelude::*;
use rand::Rng;

use crate::backend::{self, PhysicsMode};

/// Time between ball spawns (seconds). Tweak this to control spawn rate.
const SPAWN_INTERVAL: f32 = 0.05; // 50 ms → 20 balls/sec

/// Ball radius in pixels.
const BALL_RADIUS: f32 = 1.5; // diameter = 3 px

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

#[derive(Resource)]
struct SpawnTimer(Timer);

pub fn plugin(app: &mut App) {
    app.insert_resource(SpawnTimer(Timer::from_seconds(
        SPAWN_INTERVAL,
        TimerMode::Repeating,
    )));
    app.insert_resource(BallCount::default());
    app.add_systems(Update, spawn_balls);
}

fn spawn_balls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut ball_count: ResMut<BallCount>,
    mode: Res<State<PhysicsMode>>,
) {
    timer.0.tick(time.delta());

    let mode = *mode.get();
    let ball_color = Color::srgb(0.9, 0.3, 0.2);

    for _ in 0..timer.0.times_finished_this_tick() {
        let mut rng = rand::rng();
        let x = rng.random_range(SPAWN_X_MIN..=SPAWN_X_MAX);
        let position = Vec3::new(x, SPAWN_Y, 0.0);

        backend::spawn_ball(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut color_materials,
            mode,
            position,
            BALL_RADIUS,
            ball_color,
        );
        ball_count.0 += 1;
    }
}
