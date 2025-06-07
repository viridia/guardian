//! Shots from player ship
use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, RigidBody};
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::{ENEMY_LAYER, EnemyHit, FX_DEPTH, PLAYER_SHOT_LAYER, UnitPosition, ship::Facing};

/// * Abductor is destroyed, and treasure is rescued (absorbed) by player ship.
#[derive(Component, Default, Debug)]
pub struct LaserShot {
    /// Remaining time until this shot expires
    expiration: f32,

    /// Length of the shot (increases over time)
    size: f32,

    /// Horizontal velocity
    speed: f32,
}

#[derive(Resource, Default, Debug)]
pub struct ShotMesh {
    mesh: Handle<Mesh>,
    material: Handle<LaserMaterial>,
    hue: f32,
}

pub(crate) fn setup_laser(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LaserMaterial>>,
    mut shot_mesh: ResMut<ShotMesh>,
) {
    shot_mesh.mesh = meshes.add(Rectangle::from_size(Vec2::new(1.0, 0.007)));
    shot_mesh.material = materials.add(LaserMaterial {
        base: StandardMaterial {
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        },
        extension: LaserMaterialExt {
            color: LinearRgba::from(Color::srgb(0.0, 1.0, 0.0)).to_vec4(),
        },
    });
}

pub(crate) fn spawn_laser(
    commands: &mut Commands,
    position: Vec2,
    facing: Facing,
    shot_mesh: Res<ShotMesh>,
) {
    commands.spawn((
        LaserShot {
            expiration: 0.3,
            speed: match facing {
                Facing::Right => 3.0,
                Facing::Left => -3.0,
            },
            size: 0.2,
        },
        RigidBody::Kinematic,
        Collider::capsule_endpoints(0.003, Vec2::new(-0.5, 0.), Vec2::new(0.5, 0.)),
        CollisionLayers::from_bits(PLAYER_SHOT_LAYER, ENEMY_LAYER),
        CollidingEntities::default(),
        UnitPosition(Vec2::new(
            match facing {
                Facing::Right => position.x + 0.18,
                Facing::Left => position.x - 0.18,
            },
            position.y,
        )),
        Mesh3d(shot_mesh.mesh.clone()),
        MeshMaterial3d(shot_mesh.material.clone()),
        Transform::from_xyz(0., 0., FX_DEPTH).with_scale(Vec3::new(0.2, 1.0, 1.0)),
    ));
}

/// Laser animations:
/// * Overall velocity
/// * Expansion
/// * Color rotation
/// * Expiration
pub(crate) fn update_laser(
    mut commands: Commands,
    mut q_shots: Query<(Entity, &mut LaserShot, &mut UnitPosition, &mut Transform)>,
    mut materials: ResMut<Assets<LaserMaterial>>,
    r_time: Res<Time>,
    mut shot_mesh: ResMut<ShotMesh>,
) {
    // Rotate shot color
    if let Some(material) = materials.get_mut(shot_mesh.material.id()) {
        shot_mesh.hue = (shot_mesh.hue + r_time.delta_secs() * 360.0).rem_euclid(360.0);
        material.extension.color =
            LinearRgba::from(Hsla::new(shot_mesh.hue, 1.0, 0.5, 1.0)).to_vec4()
    }

    for (ent, mut shot, mut position, mut transform) in q_shots.iter_mut() {
        shot.expiration -= r_time.delta_secs();
        if shot.expiration <= 0. {
            commands.entity(ent).despawn();
        } else {
            position.0.x += shot.speed * r_time.delta_secs();
            // Update
        }
        shot.size += r_time.delta_secs();
        transform.scale.x = shot.size;
    }
}

pub(crate) fn detect_enemy_kills(
    mut commands: Commands,
    q_enemies: Query<(Entity, &CollidingEntities), With<LaserShot>>,
) {
    for (entity, collisions) in q_enemies {
        if !collisions.is_empty() {
            commands.entity(entity).despawn();
        }
        collisions.iter().for_each(|enemy| {
            commands.entity(*enemy).trigger(EnemyHit);
        });
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct LaserMaterialExt {
    #[uniform(100)]
    pub(crate) color: Vec4,
}

impl MaterialExtension for LaserMaterialExt {
    fn fragment_shader() -> ShaderRef {
        "embedded://guardian/assets/shaders/laser.wgsl".into()
    }
}

pub(crate) type LaserMaterial = ExtendedMaterial<StandardMaterial, LaserMaterialExt>;
