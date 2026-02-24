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

/// Pixels per meter — passed to every physics plugin so unit conversion matches.
pub const LENGTH_UNIT: f32 = 10.0;

/// Gravitational acceleration in m/s².
pub const GRAVITY: f32 = 9.81;

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

}

// ── Spawn helpers ────────────────────────────────────────────────────────────

/// Spawn a static wall with the correct backend components.
/// `width` and `height` are in full pixel extents.
/// The entity is tagged [`DespawnOnExit(mode)`] so it is automatically despawned
/// when the state transitions away from `mode`.
pub fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    mode: PhysicsMode,
    position: Vec3,
    width: f32,
    height: f32,
    color: Color,
) {
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
            let depth = 10.0;
            let mesh = meshes.add(Cuboid::new(width, height, depth));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
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
            let half_depth = 5.0;
            let mesh = meshes.add(Cuboid::new(width, height, half_depth * 2.0));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            });
            commands.spawn((
                Name::new("Wall"),
                DespawnOnExit(mode),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(position),
                bevy_rapier3d::prelude::RigidBody::Fixed,
                bevy_rapier3d::prelude::Collider::cuboid(width / 2.0, height / 2.0, half_depth),
            ));
        }
    }
}

/// Spawn a dynamic ball with the correct backend components.
/// Tagged [`DespawnOnExit(mode)`] so it is automatically despawned on state exit.
pub fn spawn_ball(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    color_materials: &mut Assets<ColorMaterial>,
    mode: PhysicsMode,
    position: Vec3,
    radius: f32,
    color: Color,
) {
    match mode {
        PhysicsMode::Avian2d => {
            let mesh = meshes.add(Circle::new(radius));
            let mat = color_materials.add(ColorMaterial::from_color(color));
            commands.spawn((
                Name::new("Ball"),
                DespawnOnExit(mode),
                crate::spawner::Ball,
                Mesh2d(mesh),
                MeshMaterial2d(mat),
                Transform::from_translation(position),
                avian2d::prelude::RigidBody::Dynamic,
                avian2d::prelude::Collider::circle(radius),
            ));
        }
        PhysicsMode::Avian3d => {
            let mesh = meshes.add(Sphere::new(radius));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            });
            commands.spawn((
                Name::new("Ball"),
                DespawnOnExit(mode),
                crate::spawner::Ball,
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(position),
                avian3d::prelude::RigidBody::Dynamic,
                avian3d::prelude::Collider::sphere(radius),
            ));
        }
        PhysicsMode::Rapier2d => {
            let mesh = meshes.add(Circle::new(radius));
            let mat = color_materials.add(ColorMaterial::from_color(color));
            commands.spawn((
                Name::new("Ball"),
                DespawnOnExit(mode),
                crate::spawner::Ball,
                Mesh2d(mesh),
                MeshMaterial2d(mat),
                Transform::from_translation(position),
                bevy_rapier2d::prelude::RigidBody::Dynamic,
                bevy_rapier2d::prelude::Collider::ball(radius),
            ));
        }
        PhysicsMode::Rapier3d => {
            let mesh = meshes.add(Sphere::new(radius));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            });
            commands.spawn((
                Name::new("Ball"),
                DespawnOnExit(mode),
                crate::spawner::Ball,
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(position),
                bevy_rapier3d::prelude::RigidBody::Dynamic,
                bevy_rapier3d::prelude::Collider::ball(radius),
            ));
        }
    }
}
