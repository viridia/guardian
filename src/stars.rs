use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{PLAYFIELD_WIDTH, STARS_DEPTH, Viewpoint};

#[derive(Component, Default, Debug)]
pub struct Star {
    /// Offset of the star relative to the playfield origin. To implement wrapping, the
    /// star will be displayed at a modulus of it's position.
    offset: Vec2,

    /// Speed at which the star parallax moves.
    speed: f32,
}

const NUM_STARS: usize = 200;

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
    let mesh = meshes.add(Rectangle::from_size(Vec2::splat(1.0)));

    // Star
    for _ in 0..NUM_STARS {
        let dist = rng.random_range(0.4..0.9);
        let size = 0.006 * (1.0 - dist * 0.5);
        commands.spawn((
            Mesh3d(mesh.clone()),
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
                    y: rng.random_range(-0.35..0.49),
                },
                speed: 1.0 - dist * 0.7,
            },
            Transform::from_xyz(0.0, 0.0, STARS_DEPTH).with_scale(Vec3::splat(size)),
        ));
    }
}

/// Update the positions of the individual stars in the background.
pub(crate) fn update_stars(
    r_viewpoint: Res<Viewpoint>,
    mut q_stars: Query<(&Star, &mut Transform)>,
) {
    for (star, mut transform) in q_stars.iter_mut() {
        // Parallax scrolling: offset each star by it's speed relative to the camera offset,
        // and then use modulo to implement wrap-around.
        let dist_traveled = PLAYFIELD_WIDTH * star.speed;
        transform.translation.x = (star.offset.x + dist_traveled * 0.5
            - r_viewpoint.position * star.speed)
            .rem_euclid(dist_traveled)
            - dist_traveled * 0.5;
        transform.translation.y = star.offset.y;
    }
}
