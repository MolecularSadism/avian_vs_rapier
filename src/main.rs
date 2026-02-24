// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod backend;
mod spawner;
mod walls;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::WindowResolution,
};

use std::time::Duration;

use crate::backend::PhysicsMode;
use crate::spawner::{Ball, BallCount, BallsPerTick};

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    title: "Avian vs Rapier".to_string(),
                    #[cfg(feature = "legacy_state_scoped")]
                    resolution: WindowResolution::new(1920.0_f32, 1080.0_f32),
                    #[cfg(not(feature = "legacy_state_scoped"))]
                    resolution: WindowResolution::new(1920_u32, 1080_u32),
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
                despawn_top_light,
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
                despawn_top_light,
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
                tick_mode_switch_timer,
                handle_balls_per_tick,
            ),
        )
        .run()
}

// ── Startup ───────────────────────────────────────────────────────────────────

fn setup(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    time.pause();

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

    // HUD — balls/tick (top-right, below ball counter)
    commands.spawn((
        Name::new("Balls Per Tick Display"),
        BallsPerTickText,
        Text::new("Balls/tick: 1"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.9, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(44.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));

    // HUD — clipped ball counter (top-right, below balls/tick)
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
            top: Val::Px(78.0),
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
                Text::new("Avian 2D"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
            ));
        });

    // HUD — button instructions (bottom-center)
    commands
        .spawn((
            Name::new("Button Instructions Container"),
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Button Instructions"),
                Text::new("Next mode: Enter  |  Pause: Space  |  Balls/tick: Up/Down"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
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
struct BallsPerTickText;

#[derive(Component)]
struct ClippedBallCounterText;

#[derive(Component)]
struct TopLight;

#[derive(Resource, Default)]
struct ClippedBallCount(usize);

/// Real-time buffer between requesting a mode switch and executing it, giving
/// the framerate a chance to settle before the next simulation starts.
const MODE_SWITCH_DELAY: Duration = Duration::from_millis(300);

/// Inserted when a mode switch is requested. Counts down in real time so the
/// framerate has a chance to settle before the next simulation starts.
#[derive(Resource)]
struct PendingModeSwitch {
    mode: PhysicsMode,
    timer: Timer,
}

/// Real-time delay after a mode starts before FPS milestones are recorded,
/// so frame-0 spikes don't register.
const PERF_WARMUP: Duration = Duration::from_millis(150);

/// Milestone ball counts recorded when FPS first crosses below a threshold.
#[derive(Resource, Default)]
struct PerfStats {
    /// Real time elapsed since this mode started. Milestones are ignored until
    /// this exceeds `PERF_WARMUP`.
    warmup_elapsed: Duration,
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
    commands.spawn((Name::new("Camera"), Camera2d, IsDefaultUiCamera));
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
    // Position above and in front of the pool, angled down to show all four
    // walls, the floor, and the open top.
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        IsDefaultUiCamera,
        Transform::from_xyz(0.0, 1400.0, 1600.0).looking_at(Vec3::new(0.0, -200.0, 0.0), Vec3::Y),
    ));

    // Point light positioned above the pool center.
    commands.spawn((
        Name::new("Top Light"),
        TopLight,
        PointLight {
            intensity: 50_000_000_000.0,
            range: 5_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1200.0, 0.0),
    ));
}

fn despawn_top_light(mut commands: Commands, lights: Query<Entity, With<TopLight>>) {
    for e in &lights {
        commands.entity(e).despawn();
    }
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
        **text = state.get().label().to_string();
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
    time: Res<Time<Real>>,
    mut query: Query<&mut Text, With<FpsDisplayText>>,
) {
    stats.warmup_elapsed += time.delta();

    let diag = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
    let fps = diag.and_then(|d| d.value()).unwrap_or(0.0);
    let fps_avg = diag.and_then(|d| d.average()).unwrap_or(0.0);
    let balls = ball_count.0;

    // Record milestones on first crossing, but only after the warmup window.
    if stats.warmup_elapsed >= PERF_WARMUP {
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
    }

    let fmt = |opt: Option<usize>| -> String {
        opt.map_or_else(|| "-".to_string(), |n| format!("{n} balls"))
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

/// Up/Down arrows increase or decrease balls spawned per tick (min 1).
fn handle_balls_per_tick(
    input: Res<ButtonInput<KeyCode>>,
    mut balls_per_tick: ResMut<BallsPerTick>,
    mut query: Query<&mut Text, With<BallsPerTickText>>,
) {
    let changed = if input.just_pressed(KeyCode::ArrowUp) {
        balls_per_tick.0 += 1;
        true
    } else if input.just_pressed(KeyCode::ArrowDown) {
        balls_per_tick.0 = balls_per_tick.0.saturating_sub(1).max(1);
        true
    } else {
        false
    };

    if changed {
        for mut text in &mut query {
            **text = format!("Balls/tick: {}", balls_per_tick.0);
        }
    }
}

/// Keys 1-4 jump to a specific mode; Enter cycles to the next one.
/// On press: immediately pause the simulation and start a 300 ms real-time
/// buffer. The actual state transition happens in `tick_mode_switch_timer`.
fn handle_mode_switch(
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<PhysicsMode>>,
    pending: Option<Res<PendingModeSwitch>>,
    mut vtime: ResMut<Time<Virtual>>,
    mut commands: Commands,
) {
    // Don't queue another switch while one is already pending.
    if pending.is_some() {
        return;
    }

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

    // Pause first so the framerate settles, then buffer for 300 ms.
    vtime.pause();
    commands.insert_resource(PendingModeSwitch {
        mode: new_mode,
        timer: Timer::new(MODE_SWITCH_DELAY, TimerMode::Once),
    });
}

/// Ticks the mode-switch buffer using real (wall-clock) time.
/// Once the timer finishes, transitions to the queued state and removes itself.
/// The simulation stays paused — the user must press Space to resume.
fn tick_mode_switch_timer(
    mut commands: Commands,
    pending: Option<ResMut<PendingModeSwitch>>,
    mut next_state: ResMut<NextState<PhysicsMode>>,
    time: Res<Time<Real>>,
) {
    let Some(mut pending) = pending else { return };

    pending.timer.tick(time.delta());
    if pending.timer.just_finished() {
        next_state.set(pending.mode);
        commands.remove_resource::<PendingModeSwitch>();
    }
}
