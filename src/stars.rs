use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{PLAYFIELD_WIDTH, Viewpoint};

#[derive(Component, Default, Debug)]
pub struct Star {
    /// Offset of the star relative to the playfield origin. To implement wrapping, the
    /// star will be displayed at a modulus of it's position.
    offset: Vec2,

    /// Speed at which the star parallax moves.
    speed: f32,
}

const NUM_STARS: usize = 200;
const STAR_DEPTH: f32 = 0.1;

/// Spawn the star sprites. Note that because we're using an ortho, rather than a 2d camera,
/// we can't actually use Bevy `Sprite` but instead are using planar meshes.
pub(crate) fn spawn_stars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let star = asset_server.load("textures/star.png");
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467712);

    // Star
    for _ in 0..NUM_STARS {
        let dist = rng.random_range(0.0..0.9);
        let size = 0.005 * (1.0 - dist * 0.5);
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 1.0 - dist),
                base_color_texture: Some(star.clone()),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..Default::default()
            })),
            Star {
                offset: Vec2 {
                    x: rng.random_range(0.0..PLAYFIELD_WIDTH),
                    y: rng.random_range(-0.49..0.35),
                },
                speed: 1.0 - dist * 0.7,
            },
            Transform::from_xyz(0.0, STAR_DEPTH, 0.0),
        ));
    }
}

/// Update the positions of the individual stars in the background.
pub(crate) fn update_stars(
    r_viewpoint: Res<Viewpoint>,
    mut q_stars: Query<(&Star, &mut Transform)>,
) {
    for (star, mut transform) in q_stars.iter_mut() {
        // Parallax scrolling: offset each start by it's speed relative to the camera offset,
        // and then use modulo to implement wrap-around.
        let dist_traveled = PLAYFIELD_WIDTH * star.speed;
        transform.translation.x =
            (r_viewpoint.position * star.speed + star.offset.x + dist_traveled * 0.5)
                .rem_euclid(dist_traveled)
                - dist_traveled * 0.5;
        transform.translation.z = star.offset.y;
    }
}
