//! Physics backend abstraction.
//!
//! All four physics crates (avian2d, avian3d, bevy_rapier2d, bevy_rapier3d) are
//! compiled in. A runtime [`PhysicsMode`] state controls which backend is
//! active. [`DespawnOnExit`] tags on each spawned entity handle automatic cleanup
//! when the state transitions away.
//!
//! Key API differences normalised here:
//! - Avian `rectangle(w,h)` takes full extents; Rapier `cuboid(hx,hy)` takes half-extents.
//! - Avian `circle(r)` / `sphere(r)` vs Rapier `ball(r)`.
//! - Avian `RigidBody::Static` vs Rapier `RigidBody::Fixed`.

use bevy::prelude::*;

// Bevy 0.16 called this `StateScoped`; 0.17+ renamed it to `DespawnOnExit`.
// Cargo16.toml enables `legacy_state_scoped` by default to activate this shim.
#[cfg(not(feature = "legacy_state_scoped"))]
use bevy::prelude::DespawnOnExit;
#[cfg(feature = "legacy_state_scoped")]
use bevy::prelude::StateScoped as DespawnOnExit;

/// Pixels per meter — passed to every physics plugin so unit conversion matches.
pub const LENGTH_UNIT: f32 = 10.0;

/// Gravitational acceleration in m/s².
pub const GRAVITY: f32 = 9.81 * LENGTH_UNIT;

/// Z depth of the 3D pool (full extent). Balls spawn within ±POOL_DEPTH/2.
/// Matches WIDTH in walls.rs (1920) so the pool floor is square.
pub const POOL_DEPTH: f32 = 1920.0;

// ── Physics mode ─────────────────────────────────────────────────────────────

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PhysicsMode {
    #[default]
    Avian2d,
    Avian3d,
    Rapier2d,
    Rapier3d,
}

impl PhysicsMode {
    pub fn label(self) -> &'static str {
        match self {
            PhysicsMode::Avian2d => "Avian 2D",
            PhysicsMode::Avian3d => "Avian 3D",
            PhysicsMode::Rapier2d => "Rapier 2D",
            PhysicsMode::Rapier3d => "Rapier 3D",
        }
    }

    /// Cycle to the next mode: Avian2D → Avian3D → Rapier2D → Rapier3D → Avian2D.
    pub fn next(self) -> Self {
        match self {
            PhysicsMode::Avian2d => PhysicsMode::Avian3d,
            PhysicsMode::Avian3d => PhysicsMode::Rapier2d,
            PhysicsMode::Rapier2d => PhysicsMode::Rapier3d,
            PhysicsMode::Rapier3d => PhysicsMode::Avian2d,
        }
    }
}

// ── Plugin registration ──────────────────────────────────────────────────────

pub fn plugin(app: &mut App) {
    app.init_state::<PhysicsMode>();
    // Bevy 0.16 requires this call to register the StateScoped cleanup systems.
    // In 0.17+, DespawnOnExit registers itself automatically.
    #[cfg(feature = "legacy_state_scoped")]
    app.enable_state_scoped_entities::<PhysicsMode>();

    // Register all four physics plugins — idle ones just have no entities to process.
    app.add_plugins(avian2d::PhysicsPlugins::default().with_length_unit(LENGTH_UNIT));
    // Disable PhysicsInterpolationPlugin on avian3d to avoid a duplicate-plugin panic:
    // both avian2d and avian3d unconditionally add TransformInterpolationPlugin through it.
    app.add_plugins(
        avian3d::PhysicsPlugins::default()
            .with_length_unit(LENGTH_UNIT)
            .build()
            .disable::<avian3d::interpolation::PhysicsInterpolationPlugin>(),
    );
    app.add_plugins(
        bevy_rapier2d::plugin::RapierPhysicsPlugin::<bevy_rapier2d::plugin::NoUserData>::default()
            .with_length_unit(LENGTH_UNIT),
    );
    app.add_plugins(
        bevy_rapier3d::plugin::RapierPhysicsPlugin::<bevy_rapier3d::plugin::NoUserData>::default()
            .with_length_unit(LENGTH_UNIT),
    );

    // Avian gravity is in m/s²; Vec2/Vec3 NEG_Y * 9.81.
    app.insert_resource(avian2d::prelude::Gravity(Vec2::NEG_Y * GRAVITY));
    app.insert_resource(avian3d::prelude::Gravity(Vec3::NEG_Y * GRAVITY));

    // Rapier's RapierConfiguration::new(length_unit) defaults gravity to
    // -9.81 * length_unit, which is 10× too strong with LENGTH_UNIT=10.
    // RapierConfiguration is a Component (not a Resource) in newer bevy_rapier,
    // so we patch it via startup systems after the plugin inserts it.
    app.add_systems(Startup, (set_rapier2d_gravity, set_rapier3d_gravity));
}

fn set_rapier2d_gravity(mut rapier_config: Query<&mut bevy_rapier2d::plugin::RapierConfiguration>) {
    rapier_config.single_mut().unwrap().gravity = bevy_rapier2d::math::Vect::new(0.0, -GRAVITY);
}

fn set_rapier3d_gravity(mut rapier_config: Query<&mut bevy_rapier3d::plugin::RapierConfiguration>) {
    rapier_config.single_mut().unwrap().gravity =
        bevy_rapier3d::math::Vect::new(0.0, -GRAVITY, 0.0);
}

// ── Shared ball assets ───────────────────────────────────────────────────────

/// Pre-created mesh and material handles shared by every ball entity.
/// Holding a single set of handles lets Bevy batch/instance all ball draw calls
/// instead of issuing one draw call per unique asset.
#[derive(Resource)]
pub struct BallAssets {
    pub mesh2d: Handle<Mesh>,
    pub mat2d: Handle<ColorMaterial>,
    pub mesh3d: Handle<Mesh>,
    pub mat3d: Handle<StandardMaterial>,
}

// ── Spawn helpers ────────────────────────────────────────────────────────────

/// Spawn a static wall with the correct backend components.
/// `size` is full pixel extents: (width, height, depth). Depth is only used in 3D modes.
/// The entity is tagged [`DespawnOnExit`] so it is automatically despawned
/// when the state transitions away from `mode`.
pub fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mode: PhysicsMode,
    position: Vec3,
    size: Vec3,
    color: Color,
) {
    let (width, height, depth) = (size.x, size.y, size.z);
    let sprite = (
        Sprite {
            color,
            custom_size: Some(Vec2::new(width, height)),
            ..default()
        },
        Transform::from_translation(position),
    );

    match mode {
        PhysicsMode::Avian2d => {
            commands.spawn((
                Name::new("Wall"),
                DespawnOnExit(mode),
                sprite,
                avian2d::prelude::RigidBody::Static,
                avian2d::prelude::Collider::rectangle(width, height),
            ));
        }
        PhysicsMode::Avian3d => {
            let alpha = color.to_srgba().alpha;
            let alpha_mode = if alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
            let mesh = meshes.add(Cuboid::new(width, height, depth));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                alpha_mode,
                ..default()
            });
            commands.spawn((
                Name::new("Wall"),
                DespawnOnExit(mode),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(position),
                avian3d::prelude::RigidBody::Static,
                avian3d::prelude::Collider::cuboid(width, height, depth),
            ));
        }
        PhysicsMode::Rapier2d => {
            commands.spawn((
                Name::new("Wall"),
                DespawnOnExit(mode),
                sprite,
                bevy_rapier2d::prelude::RigidBody::Fixed,
                bevy_rapier2d::prelude::Collider::cuboid(width / 2.0, height / 2.0),
            ));
        }
        PhysicsMode::Rapier3d => {
            let alpha = color.to_srgba().alpha;
            let alpha_mode = if alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
            let mesh = meshes.add(Cuboid::new(width, height, depth));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                alpha_mode,
                ..default()
            });
            commands.spawn((
                Name::new("Wall"),
                DespawnOnExit(mode),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(position),
                bevy_rapier3d::prelude::RigidBody::Fixed,
                bevy_rapier3d::prelude::Collider::cuboid(width / 2.0, height / 2.0, depth / 2.0),
            ));
        }
    }
}

/// Spawn a dynamic ball with the correct backend components.
/// Tagged [`DespawnOnExit`] so it is automatically despawned on state exit.
///
/// `assets` holds pre-created, shared handles — all balls reference the same
/// mesh and material assets, enabling GPU instancing/batching.
pub fn spawn_ball(
    commands: &mut Commands,
    mode: PhysicsMode,
    position: Vec3,
    radius: f32,
    assets: &BallAssets,
) {
    let BallAssets { mesh2d, mat2d, mesh3d, mat3d } = assets;
    match mode {
        PhysicsMode::Avian2d => {
            commands.spawn((
                Name::new("Ball"),
                DespawnOnExit(mode),
                crate::spawner::Ball,
                Mesh2d(mesh2d.clone()),
                MeshMaterial2d(mat2d.clone()),
                Transform::from_translation(position),
                avian2d::prelude::RigidBody::Dynamic,
                avian2d::prelude::Collider::circle(radius),
            ));
        }
        PhysicsMode::Avian3d => {
            commands.spawn((
                Name::new("Ball"),
                DespawnOnExit(mode),
                crate::spawner::Ball,
                Mesh3d(mesh3d.clone()),
                MeshMaterial3d(mat3d.clone()),
                Transform::from_translation(position),
                avian3d::prelude::RigidBody::Dynamic,
                avian3d::prelude::Collider::sphere(radius),
            ));
        }
        PhysicsMode::Rapier2d => {
            commands.spawn((
                Name::new("Ball"),
                DespawnOnExit(mode),
                crate::spawner::Ball,
                Mesh2d(mesh2d.clone()),
                MeshMaterial2d(mat2d.clone()),
                Transform::from_translation(position),
                bevy_rapier2d::prelude::RigidBody::Dynamic,
                bevy_rapier2d::prelude::Collider::ball(radius),
            ));
        }
        PhysicsMode::Rapier3d => {
            commands.spawn((
                Name::new("Ball"),
                DespawnOnExit(mode),
                crate::spawner::Ball,
                Mesh3d(mesh3d.clone()),
                MeshMaterial3d(mat3d.clone()),
                Transform::from_translation(position),
                bevy_rapier3d::prelude::RigidBody::Dynamic,
                bevy_rapier3d::prelude::Collider::ball(radius),
            ));
        }
    }
}
