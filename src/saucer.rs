use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::{audio::PlaybackMode, prelude::*, scene::SceneInstanceReady};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::{
    ENEMY_LAYER, Enemy, EnemyHit, PLAYER_LAYER, PLAYER_SHOT_LAYER, PLAYFIELD_WIDTH,
    RandomGenerator, SHIP_DEPTH, UnitPosition,
    explosion::{FlareEffect, ShrapnelEffect},
};

/// State of a saucer
#[derive(Default, Debug, Copy, Clone)]
pub enum SaucerState {
    /// Arrival animation
    #[default]
    Arriving,

    /// Wandering around
    Patrolling(Vec2),

    /// Located an abduction target, moving to location
    Seeking,

    /// Grabbing the treasure
    Grabbing,

    /// Moving upwards with the loot
    Escaping,

    /// Once the saucer has reached the top, it is replaced with a more powerful enemy.
    Queened,
}

/// Saucers are a type of enemy that abducts treasure
#[derive(Component, Default, Debug)]
pub struct Saucer {
    /// What's happening with this saucer
    state: SaucerState,

    /// Where we are going to
    timer: f32,
}

const SAUCER_SPEED_X: f32 = 0.4;
const SAUCER_SPEED_Y: f32 = 0.2;

#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

pub(crate) fn spawn_saucer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut rng: ResMut<RandomGenerator>,
) {
    let animation = asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/saucer.glb"));
    let (graph, index) = AnimationGraph::from_clip(animation);
    let graph_handle = graphs.add(graph);

    for _ in 0..24 {
        // Saucer model
        commands
            .spawn((
                SceneRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/saucer.glb")),
                ),
                Saucer {
                    state: SaucerState::Arriving,
                    timer: rng.0.random_range(1.0..2.0),
                },
                Enemy,
                RigidBody::Kinematic,
                Collider::capsule_endpoints(2.0, Vec2::new(-2., 0.2), Vec2::new(2., 0.2)),
                CollisionLayers::from_bits(ENEMY_LAYER, PLAYER_LAYER | PLAYER_SHOT_LAYER),
                UnitPosition(Vec2::new(
                    rng.0.random_range(0.0..PLAYFIELD_WIDTH),
                    rng.0.random_range(0.6..0.7),
                )),
                AnimationToPlay {
                    graph_handle: graph_handle.clone(),
                    index,
                },
                Transform::from_scale(Vec3::splat(0.013))
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.1, 0.2, 0.0))
                    .with_translation(Vec3::new(0., 0., SHIP_DEPTH)),
            ))
            .observe(play_animation_when_ready)
            .observe(saucer_hit);
    }
}

// TODO: Clean this up, right now it's pasted from the example.
fn play_animation_when_ready(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    // The entity we spawned in `setup_mesh_and_animation` is the trigger's target.
    // Start by finding the AnimationToPlay component we added to that entity.
    if let Ok(animation_to_play) = animations_to_play.get(trigger.target()) {
        // The SceneRoot component will have spawned the scene as a hierarchy
        // of entities parented to our entity. Since the asset contained a skinned
        // mesh and animations, it will also have spawned an animation player
        // component. Search our entity's descendants to find the animation player.
        for child in children.iter_descendants(trigger.target()) {
            if let Ok(mut player) = players.get_mut(child) {
                // Tell the animation player to start the animation and keep
                // repeating it.
                //
                // If you want to try stopping and switching animations, see the
                // `animated_mesh_control.rs` example.
                player.play(animation_to_play.index).repeat();

                // Add the animation graph. This only needs to be done once to
                // connect the animation player to the mesh.
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}

pub(crate) fn animate_saucers(
    mut q_saucers: Query<(&mut Saucer, &mut UnitPosition)>,
    time: Res<Time>,
    mut rng: ResMut<RandomGenerator>,
) {
    // let move_dist = 0.5 * time.delta_secs();
    for (mut saucer, mut position) in q_saucers.iter_mut() {
        match saucer.state {
            SaucerState::Arriving => {
                saucer.state = SaucerState::Patrolling(choose_random_angle(&mut rng.0));
                saucer.timer = rng.0.random_range(1.0..2.0);
            }

            SaucerState::Patrolling(vel) => {
                saucer.timer -= time.delta_secs();

                position.0 += vel * time.delta_secs();
                position.0.x = (position.0.x + PLAYFIELD_WIDTH * 0.5).rem_euclid(PLAYFIELD_WIDTH)
                    - PLAYFIELD_WIDTH * 0.5;
                if position.0.y > 0.4 {
                    saucer.state = SaucerState::Patrolling(Vec2::new(vel.x, -SAUCER_SPEED_Y));
                } else if position.0.y < -0.4 {
                    saucer.state = SaucerState::Patrolling(Vec2::new(vel.x, SAUCER_SPEED_Y));
                } else if saucer.timer <= 0.0 {
                    saucer.state = SaucerState::Patrolling(choose_random_angle(&mut rng.0));
                    saucer.timer = rng.0.random_range(1.0..2.0);
                }
            }
            _ => todo!(),
        };
    }
}

fn choose_random_angle(rng: &mut ChaCha8Rng) -> Vec2 {
    let dir: f32 = rng.random_range(0.0..8.0);
    let angle = dir.trunc() * std::f32::consts::FRAC_PI_4; // 0, 45, ..., 315 deg
    Vec2::new(angle.cos(), angle.sin()) * Vec2::new(SAUCER_SPEED_X, SAUCER_SPEED_Y)
}

/// Action triggered when a saucer is hit by a player shot. We despawn the saucer and replace
/// it with an explosion (both sound and visuals).
fn saucer_hit(
    trigger: Trigger<EnemyHit>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_position: Query<&UnitPosition>,
) {
    let Ok(unit_pos) = q_position.get(trigger.target()) else {
        return;
    };
    let position = unit_pos.0;
    commands.entity(trigger.target()).despawn();
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/softexplode.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    ));
    commands.spawn((
        FlareEffect {
            size: 0.01,
            velocity: Vec2::default(),
        },
        UnitPosition(position),
    ));
    commands.spawn((
        ShrapnelEffect {
            velocity: Vec2::default(),
        },
        UnitPosition(position),
    ));
}
