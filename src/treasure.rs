use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{PLAYFIELD_WIDTH, TREASURE_DEPTH, UnitPosition};

/// State of a treasure item
#[derive(Default, Debug, Copy, Clone)]
pub enum TreasureState {
    /// On the ground
    #[default]
    Standing,

    /// Being abducted by a saucer
    Abducted,

    /// Falling to the ground after abductor has been destroyed
    Falling,
}

/// An abductable treasure item. Treasure items spawn on the ground at the start of a level.
/// A treasure item can cease to exist in one of three ways:
/// * Abduction is successful (saucer brings it to the top of the screen)
/// * Abductor is destroyed, and treasure crashes to the ground
/// * Abductor is destroyed, and treasure is rescued (absorbed) by player ship.
#[derive(Component, Default, Debug)]
pub struct Treasure {
    /// What's happening with this treasure
    state: TreasureState,
    // /// Horizontal velocity
    // speed: f32,

    // /// Current ship orientation - follows facing but smoothed
    // pitch: f32,

    // /// Yaw is affected by both spin and up / down movements.
    // yaw: f32,
}

const NUM_TREASURES: usize = 16;

pub(crate) fn spawn_treasure(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TODO: Seed this with the current level number
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467712);

    // Distance between treasures
    let treasure_interval = PLAYFIELD_WIDTH / NUM_TREASURES as f32;

    // Random displaement of initial position
    let treasure_displacement = treasure_interval * 0.3;

    for i in 0..NUM_TREASURES {
        let pos = i as f32 * treasure_interval
            + rng.random_range(-treasure_displacement..treasure_displacement);
        let treasure_type = rng.random_range(0..3);
        // Treasure model
        commands.spawn((
            SceneRoot(asset_server.load(match treasure_type {
                0 => GltfAssetLabel::Scene(0).from_asset("models/tank.glb"),
                1 => GltfAssetLabel::Scene(0).from_asset("models/dish.glb"),
                2 => GltfAssetLabel::Scene(0).from_asset("models/rover.glb"),
                _ => unreachable!(),
            })),
            Transform::from_scale(Vec3::splat(0.013))
                .with_translation(Vec3::new(pos, -0.47, TREASURE_DEPTH))
                .with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    0.1,
                    rng.random_range(0.1..5.0),
                    0.0,
                )),
            Treasure {
                state: TreasureState::Standing,
            },
            UnitPosition(Vec2::new(pos, -0.47)),
        ));
    }
}
