use bevy::{prelude::*, scene::SceneInstanceReady};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{SHIP_DEPTH, UnitPosition};

/// State of a saucer
#[derive(Default, Debug, Copy, Clone)]
pub enum SaucerState {
    /// Arrival animation
    #[default]
    Arriving,

    /// Wandering around
    Patrolling,

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
    // /// Horizontal velocity
    // speed: f32,

    // /// Current ship orientation - follows facing but smoothed
    // pitch: f32,

    // /// Yaw is affected by both spin and up / down movements.
    // yaw: f32,
}

#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

pub(crate) fn spawn_saucer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // TODO: Seed this with the current level number
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467712);

    let animation = asset_server.load(GltfAssetLabel::Animation(0).from_asset("models/saucer.glb"));
    let (graph, index) = AnimationGraph::from_clip(animation);
    let graph_handle = graphs.add(graph);

    for i in 0..24 {
        let pos = i as f32 * 0.63;
        // Saucer model
        commands
            .spawn((
                SceneRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/saucer.glb")),
                ),
                Saucer {
                    state: SaucerState::Arriving,
                },
                UnitPosition(Vec2::new(pos, rng.random_range(-0.25..0.25))),
                AnimationToPlay {
                    graph_handle: graph_handle.clone(),
                    index,
                },
                Transform::from_scale(Vec3::splat(0.013))
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.1, 0.2, 0.0))
                    .with_translation(Vec3::new(0., 0., SHIP_DEPTH)),
                // AudioPlayer::new(asset_server.load("sounds/thrust.ogg")),
                // PlaybackSettings {
                //     mode: PlaybackMode::Loop,
                //     speed: 0.2,
                //     volume: Volume::Linear(0.),
                //     ..default()
                // },
            ))
            .observe(play_animation_when_ready);
    }
}

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
