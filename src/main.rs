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

// ── Auto-zoom constants ────────────────────────────────────────────────────────

/// Physics pool dimensions in world units (must match walls.rs WIDTH / HEIGHT).
const POOL_W: f32 = 1920.0;
const POOL_H: f32 = 1080.0;

/// 3D camera look-at target (world space).
const CAM3D_LOOK_AT: Vec3 = Vec3::new(0.0, -200.0, 0.0);

/// Offset from the look-at point to the reference camera position.
/// Reference position is (0, 3000, 3200), designed for a 960 × 540 window.
const CAM3D_REF_OFFSET: Vec3 = Vec3::new(0.0, 3200.0, 3200.0);
const CAM3D_REF_W: f32 = 960.0;
const CAM3D_REF_H: f32 = 540.0;

/// Smallest orthographic scale that fits the full pool into a window of `width × height`.
fn ortho_scale_for_window(width: f32, height: f32) -> f32 {
    (POOL_W / width).max(POOL_H / height)
}

/// 3D camera world position that fits the full pool into a window of `width × height`.
/// Scales the camera's distance from the look-at point, keeping the view direction fixed.
fn cam3d_pos_for_window(width: f32, height: f32) -> Vec3 {
    let scale = (CAM3D_REF_W / width).max(CAM3D_REF_H / height);
    CAM3D_LOOK_AT + CAM3D_REF_OFFSET * scale
}

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    title: "Avian vs Rapier".to_string(),
                    #[cfg(feature = "legacy_state_scoped")]
                    resolution: WindowResolution::new(960.0_f32, 540.0_f32),
                    #[cfg(not(feature = "legacy_state_scoped"))]
                    resolution: WindowResolution::new(960_u32, 540_u32),
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
        .init_resource::<WarmupTimer>()
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
                pause_simulation,
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
                pause_simulation,
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
                pause_simulation,
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
                pause_simulation,
            ),
        )
        .add_systems(
            Update,
            (
                tick_warmup_timer,
                update_fps_display.after(tick_warmup_timer),
                update_ball_counter,
                detect_clipped_balls,
                toggle_pause,
                handle_mode_switch,
                handle_balls_per_tick,
                fit_camera_to_pool,
            ),
        )
        .run()
}

// ── Startup ───────────────────────────────────────────────────────────────────

fn setup(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    time.pause();

    // HUD root — full-screen flex container; all HUD elements are children.
    commands
        .spawn((
            Name::new("HUD Root"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Top row: FPS (left) | Mode label (center) | Ball counters (right)
            root.spawn((
                Name::new("HUD Top Row"),
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
            ))
            .with_children(|top| {
                // Left: FPS / perf stats
                top.spawn((
                    Name::new("FPS Display"),
                    FpsDisplayText,
                    Node::default(),
                    Text::new(""),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                // Center: mode label
                top.spawn((
                    Name::new("Mode Container"),
                    Node {
                        flex_grow: 1.0,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ))
                .with_children(|center| {
                    center.spawn((
                        Name::new("Mode Label"),
                        ModeText,
                        Node::default(),
                        Text::new("Avian 2D"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.2)),
                    ));
                });

                // Right: ball counter column
                top.spawn((
                    Name::new("Ball Info Column"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                ))
                .with_children(|right| {
                    right.spawn((
                        Name::new("Ball Counter"),
                        BallCounterText,
                        Node::default(),
                        Text::new("Balls: 0"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    right.spawn((
                        Name::new("Balls Per Tick Display"),
                        BallsPerTickText,
                        Node::default(),
                        Text::new("Balls/tick: 1"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.9, 0.5)),
                    ));
                    right.spawn((
                        Name::new("Clipped Ball Counter"),
                        ClippedBallCounterText,
                        Node::default(),
                        Text::new("Clipped balls: 0"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.5, 0.2)),
                    ));
                });
            });

            // Bottom row: button instructions (center)
            root.spawn((
                Name::new("Button Instructions Container"),
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|bottom| {
                bottom.spawn((
                    Name::new("Button Instructions"),
                    Node::default(),
                    Text::new(
                        "Next mode: Enter  |  Pause: Space  |  Balls/tick: Up/Down",
                    ),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            });
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

/// Real-time delay after entering a mode before FPS milestones are recorded,
/// so frame-0 spikes don't register.
const PERF_WARMUP: Duration = Duration::from_millis(1000);

/// Separate timer resource so state transitions are never blocked.
/// Reset via `Changed<State<PhysicsMode>>` in `tick_warmup_timer`.
#[derive(Resource)]
struct WarmupTimer(Timer);

impl Default for WarmupTimer {
    fn default() -> Self {
        Self(Timer::new(PERF_WARMUP, TimerMode::Once))
    }
}

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
    windows: Query<&Window>,
) {
    if !camera_2d.is_empty() {
        return;
    }
    for e in &camera_3d {
        commands.entity(e).despawn();
    }
    let scale = windows
        .get_single()
        .map(|w| ortho_scale_for_window(w.width(), w.height()))
        .unwrap_or(2.0);
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        IsDefaultUiCamera,
        Projection::Orthographic(OrthographicProjection {
            scale,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn enter_3d_camera(
    mut commands: Commands,
    camera_2d: Query<Entity, With<Camera2d>>,
    camera_3d: Query<Entity, With<Camera3d>>,
    windows: Query<&Window>,
) {
    if !camera_3d.is_empty() {
        return;
    }
    for e in &camera_2d {
        commands.entity(e).despawn();
    }
    // Position above and in front of the pool, angled down to show all four
    // walls, the floor, and the open top. Zoom is adjusted for the current window.
    let cam_pos = windows
        .get_single()
        .map(|w| cam3d_pos_for_window(w.width(), w.height()))
        .unwrap_or(CAM3D_LOOK_AT + CAM3D_REF_OFFSET);
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        IsDefaultUiCamera,
        Transform::from_translation(cam_pos).looking_at(CAM3D_LOOK_AT, Vec3::Y),
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

fn pause_simulation(mut vtime: ResMut<Time<Virtual>>) {
    vtime.pause();
}

fn update_mode_text(state: Res<State<PhysicsMode>>, mut query: Query<&mut Text, With<ModeText>>) {
    for mut text in &mut query {
        **text = state.get().label().to_string();
    }
}

// ── Update systems ────────────────────────────────────────────────────────────

/// Listens for window resize events and adjusts camera zoom / position so the
/// whole physics pool always fits inside the window without being clipped.
///
/// - 2D camera: updates the orthographic projection scale.
/// - 3D camera: scales its distance from the look-at point along the fixed
///   view direction, which is equivalent to perspective zoom.
fn fit_camera_to_pool(
    mut resize_events: EventReader<WindowResized>,
    mut cam2d: Query<&mut Projection, With<Camera2d>>,
    mut cam3d: Query<&mut Transform, With<Camera3d>>,
) {
    let Some(event) = resize_events.read().last() else {
        return;
    };
    let (w, h) = (event.width, event.height);

    for mut proj in &mut cam2d {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scale = ortho_scale_for_window(w, h);
        }
    }

    for mut transform in &mut cam3d {
        transform.translation = cam3d_pos_for_window(w, h);
    }
}

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

/// Resets and ticks `WarmupTimer`. Detects state changes via `Changed<State>` so
/// a single system covers all modes without 4× `OnEnter` registrations.
fn tick_warmup_timer(
    state: Res<State<PhysicsMode>>,
    mut warmup: ResMut<WarmupTimer>,
    time: Res<Time<Real>>,
) {
    if state.is_changed() {
        warmup.0 = Timer::new(PERF_WARMUP, TimerMode::Once);
    }
    warmup.0.tick(time.delta());
}

fn update_fps_display(
    diagnostics: Res<DiagnosticsStore>,
    ball_count: Res<BallCount>,
    mut stats: ResMut<PerfStats>,
    warmup: Res<WarmupTimer>,
    mut query: Query<&mut Text, With<FpsDisplayText>>,
) {
    let diag = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
    let fps = diag.and_then(|d| d.value()).unwrap_or(0.0);
    let fps_avg = diag.and_then(|d| d.average()).unwrap_or(0.0);
    let balls = ball_count.0;

    // Record milestones on first crossing, but only after the 300 ms warmup.
    if warmup.0.elapsed() >= warmup.0.duration() {
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
/// The transition is immediate; `OnEnter` handles pausing and timer reset.
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
