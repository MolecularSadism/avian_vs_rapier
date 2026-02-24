// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod backend;
mod spawner;
mod walls;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::backend::PhysicsMode;
use crate::spawner::{Ball, BallCount};

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Avian vs Rapier".to_string(),
                        resolution: (1920, 1080).into(),
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(backend::plugin)
        .add_plugins(spawner::plugin)
        .init_resource::<PerfStats>()
        .init_resource::<ClippedBallCount>()
        .add_systems(Startup, setup)
        // Per-mode OnEnter: camera, walls, ball-count reset, mode label update.
        .add_systems(
            OnEnter(PhysicsMode::Avian2d),
            (
                enter_2d_camera,
                spawn_walls_system,
                reset_ball_count,
                reset_clipped_ball_count,
                reset_perf_stats,
                update_mode_text,
            ),
        )
        .add_systems(
            OnEnter(PhysicsMode::Avian3d),
            (
                enter_3d_camera,
                spawn_walls_system,
                reset_ball_count,
                reset_clipped_ball_count,
                reset_perf_stats,
                update_mode_text,
            ),
        )
        .add_systems(
            OnEnter(PhysicsMode::Rapier2d),
            (
                enter_2d_camera,
                spawn_walls_system,
                reset_ball_count,
                reset_clipped_ball_count,
                reset_perf_stats,
                update_mode_text,
            ),
        )
        .add_systems(
            OnEnter(PhysicsMode::Rapier3d),
            (
                enter_3d_camera,
                spawn_walls_system,
                reset_ball_count,
                reset_clipped_ball_count,
                reset_perf_stats,
                update_mode_text,
            ),
        )
        .add_systems(
            Update,
            (
                update_fps_display,
                update_ball_counter,
                detect_clipped_balls,
                toggle_pause,
                handle_mode_switch,
            ),
        )
        .run()
}

// ── Startup ───────────────────────────────────────────────────────────────────

fn setup(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    time.pause();

    // Camera — 2D for the default Avian2d state.
    commands.spawn((Name::new("Camera"), Camera2d));

    // HUD — FPS / perf stats (top-left)
    commands.spawn((
        Name::new("FPS Display"),
        FpsDisplayText,
        Text::new(""),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // HUD — ball counter (top-right)
    commands.spawn((
        Name::new("Ball Counter"),
        BallCounterText,
        Text::new("Balls: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));

    // HUD — clipped ball counter (top-right, below ball counter with an empty-line gap)
    commands.spawn((
        Name::new("Clipped Ball Counter"),
        ClippedBallCounterText,
        Text::new("Clipped balls: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.5, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(70.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));

    // HUD — mode label (top-center).
    commands
        .spawn((
            Name::new("Mode Container"),
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Mode Label"),
                ModeText,
                Text::new("Avian 2D  [Enter: next mode · Space: pause]"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
            ));
        });
}

// ── Marker components & resources ─────────────────────────────────────────────

#[derive(Component)]
struct FpsDisplayText;

#[derive(Component)]
struct BallCounterText;

#[derive(Component)]
struct ModeText;

#[derive(Component)]
struct ClippedBallCounterText;

#[derive(Resource, Default)]
struct ClippedBallCount(usize);

/// Milestone ball counts recorded when FPS first crosses below a threshold.
#[derive(Resource, Default)]
struct PerfStats {
    /// Ball count when instantaneous FPS first dropped below 50.
    first_below_50: Option<usize>,
    /// Ball count when 1-sec average FPS first dropped below 50.
    avg_below_50: Option<usize>,
    /// Ball count when instantaneous FPS first dropped below 15.
    first_below_15: Option<usize>,
    /// Ball count when 1-sec average FPS first dropped below 15.
    avg_below_15: Option<usize>,
}

// ── OnEnter helpers ───────────────────────────────────────────────────────────

fn enter_2d_camera(
    mut commands: Commands,
    camera_2d: Query<Entity, With<Camera2d>>,
    camera_3d: Query<Entity, With<Camera3d>>,
) {
    if !camera_2d.is_empty() {
        return;
    }
    for e in &camera_3d {
        commands.entity(e).despawn();
    }
    commands.spawn((Name::new("Camera"), Camera2d));
}

fn enter_3d_camera(
    mut commands: Commands,
    camera_2d: Query<Entity, With<Camera2d>>,
    camera_3d: Query<Entity, With<Camera3d>>,
) {
    if !camera_3d.is_empty() {
        return;
    }
    for e in &camera_2d {
        commands.entity(e).despawn();
    }
    // Pool floor sits at y ≈ -535, top opening at y ≈ +540.
    // Place the camera just above the pool rim and tilt it down toward the floor.
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 800.0, 1800.0)
            .looking_at(Vec3::new(0.0, -535.0, 0.0), Vec3::Y),
    ));
}

fn spawn_walls_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<State<PhysicsMode>>,
) {
    walls::spawn_walls(&mut commands, &mut meshes, &mut materials, *state.get());
}

fn reset_ball_count(mut ball_count: ResMut<BallCount>) {
    ball_count.0 = 0;
}

fn reset_clipped_ball_count(
    mut clipped: ResMut<ClippedBallCount>,
    mut query: Query<&mut Text, With<ClippedBallCounterText>>,
) {
    clipped.0 = 0;
    for mut text in &mut query {
        **text = "Clipped balls: 0".to_string();
    }
}

fn reset_perf_stats(mut stats: ResMut<PerfStats>) {
    *stats = PerfStats::default();
}

fn update_mode_text(state: Res<State<PhysicsMode>>, mut query: Query<&mut Text, With<ModeText>>) {
    for mut text in &mut query {
        **text = format!(
            "{}  [Enter: next mode · Space: pause]",
            state.get().label()
        );
    }
}

// ── Update systems ────────────────────────────────────────────────────────────

fn toggle_pause(keys: Res<ButtonInput<KeyCode>>, mut time: ResMut<Time<Virtual>>) {
    if keys.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

fn update_ball_counter(
    ball_count: Res<BallCount>,
    mut query: Query<&mut Text, With<BallCounterText>>,
) {
    if ball_count.is_changed() {
        for mut text in &mut query {
            **text = format!("Balls: {}", ball_count.0);
        }
    }
}

fn update_fps_display(
    diagnostics: Res<DiagnosticsStore>,
    ball_count: Res<BallCount>,
    mut stats: ResMut<PerfStats>,
    mut query: Query<&mut Text, With<FpsDisplayText>>,
) {
    let diag = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
    let fps = diag.and_then(|d| d.value()).unwrap_or(0.0);
    let fps_avg = diag.and_then(|d| d.average()).unwrap_or(0.0);
    let balls = ball_count.0;

    // Record milestones on first crossing.
    if fps < 50.0 && fps > 0.0 && stats.first_below_50.is_none() {
        stats.first_below_50 = Some(balls);
    }
    if fps_avg < 50.0 && fps_avg > 0.0 && stats.avg_below_50.is_none() {
        stats.avg_below_50 = Some(balls);
    }
    if fps < 15.0 && fps > 0.0 && stats.first_below_15.is_none() {
        stats.first_below_15 = Some(balls);
    }
    if fps_avg < 15.0 && fps_avg > 0.0 && stats.avg_below_15.is_none() {
        stats.avg_below_15 = Some(balls);
    }

    let fmt = |opt: Option<usize>| -> String {
        opt.map_or_else(|| "—".to_string(), |n| format!("{n} balls"))
    };

    let display = format!(
        "FPS:  {fps:.0}\nAvg:  {fps_avg:.0}\n\nFirst <50:  {}\nAvg <50:    {}\nFirst <15:  {}\nAvg <15:    {}",
        fmt(stats.first_below_50),
        fmt(stats.avg_below_50),
        fmt(stats.first_below_15),
        fmt(stats.avg_below_15),
    );

    for mut text in &mut query {
        **text = display.clone();
    }
}

/// Despawns any ball whose Y falls below the screen bottom (clipped through the floor).
/// Tracks the cumulative count via `ClippedBallCount` resource and updates the UI counter.
fn detect_clipped_balls(
    mut commands: Commands,
    mut ball_count: ResMut<BallCount>,
    balls: Query<(Entity, &Transform), With<Ball>>,
    mut clipped: ResMut<ClippedBallCount>,
    mut query: Query<&mut Text, With<ClippedBallCounterText>>,
) {
    // Screen bottom is at -540 (HEIGHT / 2 = 540).
    const FLOOR_Y: f32 = -540.0;

    for (entity, transform) in &balls {
        if transform.translation.y < FLOOR_Y {
            commands.entity(entity).despawn();
            ball_count.0 = ball_count.0.saturating_sub(1);
            clipped.0 += 1;
        }
    }

    for mut text in &mut query {
        **text = format!("Clipped balls: {}", clipped.0);
    }
}

/// Keys 1-4 jump to a specific mode; Enter cycles to the next one.
fn handle_mode_switch(
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<PhysicsMode>>,
    mut next_state: ResMut<NextState<PhysicsMode>>,
) {
    let new_mode = if input.just_pressed(KeyCode::Digit1) {
        Some(PhysicsMode::Avian2d)
    } else if input.just_pressed(KeyCode::Digit2) {
        Some(PhysicsMode::Avian3d)
    } else if input.just_pressed(KeyCode::Digit3) {
        Some(PhysicsMode::Rapier2d)
    } else if input.just_pressed(KeyCode::Digit4) {
        Some(PhysicsMode::Rapier3d)
    } else if input.just_pressed(KeyCode::Enter) {
        Some(state.get().next())
    } else {
        None
    };

    let Some(new_mode) = new_mode else { return };
    if *state.get() == new_mode {
        return;
    }
    next_state.set(new_mode);
}
