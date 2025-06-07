//! Shrapnel from explosions
use avian2d::math::PI;
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::{FX_DEPTH, RandomGenerator, UnitPosition};

/// Determines the lifetime of the effect
#[derive(Component, Default, Debug)]
pub struct EffectTimer {
    total: f32,
    elapsed: f32,
}

impl EffectTimer {
    /// Return the elapsed time as a proportion of the total time.
    pub fn t(&self) -> f32 {
        self.elapsed / self.total
    }
}

/// Explosion effect: shower of metal fragments
#[derive(Component, Default, Debug)]
pub struct ShrapnelEffect {
    /// Velocity
    pub velocity: Vec2,
}

/// Explosion effect: one fragment of shrapnel
#[derive(Component, Default, Debug)]
pub struct ShrapnelFragment {
    /// Velocity of fragment
    pub velocity: Vec2,

    /// Rotation of fragment
    pub spin_axis: Vec3,
}

const NUM_FRAGMENTS: usize = 64;

/// Explosion effect: expanding sphere of light
#[derive(Component, Default, Debug)]
pub struct FlareEffect {
    /// Size of the flare
    pub size: f32,

    /// Velocity
    pub velocity: Vec2,
}

/// Stores shared materials and meshes used by effects.
#[derive(Resource, Default, Debug)]
pub struct ExplosionHandles {
    shrapnel_mesh: Handle<Mesh>,
    flare_mesh: Handle<Mesh>,
}

pub(crate) fn setup_explosions(
    mut meshes: ResMut<Assets<Mesh>>,
    mut handles: ResMut<ExplosionHandles>,
) {
    // Vec2::Y * 0.5, Vec2::new(-0.5, -0.5), Vec2::new(0.5, -0.5)
    handles.shrapnel_mesh = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 0.01),
        Vec2::new(-0.01, -0.01),
        Vec2::new(0.01, -0.007),
    ));
    handles.flare_mesh = meshes.add(Rectangle::default());
}

pub(crate) fn on_add_shrapnel(
    trigger: Trigger<OnAdd, ShrapnelEffect>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    handles: Res<ExplosionHandles>,
    random: ResMut<RandomGenerator>,
) {
    let mesh = handles.shrapnel_mesh.clone();
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.05,
        metallic: 0.7,
        ..default()
    });

    let mut rng = random.0.clone();
    commands.entity(trigger.target()).insert((
        EffectTimer {
            total: 0.8,
            elapsed: 0.,
        },
        Transform::from_xyz(0., 0., FX_DEPTH),
        Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
            for _ in 0..NUM_FRAGMENTS {
                let rot = Quat::from_euler(
                    EulerRot::XYZ,
                    rng.random_range(0.0..PI * 2.0),
                    rng.random_range(0.0..PI),
                    rng.random_range(0.0..PI * 2.0),
                );
                let velocity =
                    Vec2::from_angle(rng.random_range(0.0..PI * 2.0)) * rng.random_range(0.2..0.6);
                parent.spawn((
                    ShrapnelFragment {
                        velocity,
                        spin_axis: random_unit_vector(&mut rng),
                    },
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_rotation(rot),
                ));
            }
        })),
    ));
}

/// Animate the flare
pub(crate) fn on_add_flare(
    trigger: Trigger<OnAdd, FlareEffect>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    handles: Res<ExplosionHandles>,
    asset_server: Res<AssetServer>,
) {
    commands.entity(trigger.target()).insert((
        EffectTimer {
            total: 0.2,
            elapsed: 0.,
        },
        Mesh3d(handles.flare_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/glowspark.png")),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0., 0., FX_DEPTH),
    ));
}

/// Animate the shrapnel
pub(crate) fn update_shrapnel(
    mut commands: Commands,
    mut q_shots: Query<
        (
            Entity,
            &mut ShrapnelEffect,
            &mut EffectTimer,
            &mut UnitPosition,
            &mut Transform,
            &Children,
        ),
        With<ShrapnelEffect>,
    >,
    mut q_fragments: Query<
        (
            &mut ShrapnelFragment,
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        Without<ShrapnelEffect>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    r_time: Res<Time>,
) {
    for (ent, effect, mut timer, mut position, _transform, children) in q_shots.iter_mut() {
        timer.elapsed += r_time.delta_secs();
        if timer.elapsed >= timer.total {
            commands.entity(ent).despawn();
            continue;
        } else {
            position.0 += effect.velocity * r_time.delta_secs();
        }

        let fade = 1.0 - timer.t();
        let mut material_updated = false;
        for child_id in children.iter() {
            if let Ok((fragment, mut frag_xform, material)) = q_fragments.get_mut(child_id) {
                // Since all the children share the same material handle, we only need to do this
                // once
                if !material_updated {
                    if let Some(material) = materials.get_mut(material.id()) {
                        material.emissive =
                            LinearRgba::new(fade.powf(3.0), fade.powf(4.0), 0.0, 1.0);
                        material.base_color.set_alpha((fade * 4.0).min(1.0));
                    }
                    material_updated = true;
                }

                frag_xform.rotate_axis(
                    Dir3::new(fragment.spin_axis).unwrap(),
                    8.1 * r_time.delta_secs(),
                );
                frag_xform.translation.x += fragment.velocity.x * r_time.delta_secs();
                frag_xform.translation.y += fragment.velocity.y * r_time.delta_secs();
            }
        }
    }
}

pub(crate) fn update_flare(
    mut commands: Commands,
    mut q_shots: Query<(
        Entity,
        &mut FlareEffect,
        &mut EffectTimer,
        &MeshMaterial3d<StandardMaterial>,
        &mut UnitPosition,
        &mut Transform,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    r_time: Res<Time>,
) {
    for (ent, mut effect, mut timer, material, mut position, mut transform) in q_shots.iter_mut() {
        timer.elapsed += r_time.delta_secs();
        if timer.elapsed >= timer.total {
            commands.entity(ent).despawn();
            continue;
        } else {
            // Update position
            position.0 += effect.velocity * r_time.delta_secs();
            // Update color
            // TODO: Use animation curves for this
            if let Some(material) = materials.get_mut(material.id()) {
                let t = timer.t();
                material.base_color =
                    LinearRgba::from(Srgba::new(1.0, 1.0 - t * 0.5, 1.0 - t, 1.0 - t)).into()
            }
        }
        effect.size += r_time.delta_secs() * 2.;
        transform.scale.x = effect.size;
        transform.scale.y = effect.size;
    }
}

fn random_unit_vector(rng: &mut ChaCha8Rng) -> Vec3 {
    let theta = rng.random_range(0.0..(2.0 * PI));
    let phi = rng.random_range(0.0..PI);
    let x = phi.sin() * theta.cos();
    let y = phi.sin() * theta.sin();
    let z = phi.cos();
    Vec3::new(x, y, z)
}
