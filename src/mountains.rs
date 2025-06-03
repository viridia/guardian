use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::PrimitiveTopology};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{MOUNTAINS_DEPTH, PLAYFIELD_WIDTH, Viewpoint};

#[derive(Component, Default, Debug)]
pub struct Moutains {
    /// Speed at which the star parallax moves.
    speed: f32,
}

pub(crate) fn spawn_mountains(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467712);

    let mountains = create_mountain_mesh(
        LinearRgba::from(Srgba::new(0.05, 0.05, 0.1, 1.0)),
        LinearRgba::from(Srgba::new(0.3, 0.3, 0.35, 1.0)),
        &mut rng,
    );

    commands.spawn((
        Mesh3d(meshes.add(mountains)),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            ..Default::default()
        })),
        Transform::from_translation(Vec3::new(0.0, -0.55, MOUNTAINS_DEPTH + 0.11)),
        Moutains { speed: 1.0 },
    ));

    let mountains = create_mountain_mesh(
        LinearRgba::from(Srgba::new(0.02, 0.02, 0.1, 1.0)),
        LinearRgba::from(Srgba::new(0.2, 0.2, 0.25, 1.0)),
        &mut rng,
    );

    commands.spawn((
        Mesh3d(meshes.add(mountains)),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            ..Default::default()
        })),
        Transform::from_translation(Vec3::new(0.0, -0.4, MOUNTAINS_DEPTH + 0.1))
            .with_scale(Vec3::splat(0.5)),
        Moutains { speed: 0.5 },
    ));

    let mountains = create_mountain_mesh(
        LinearRgba::from(Srgba::new(0.05, 0.05, 0.15, 1.0)),
        LinearRgba::from(Srgba::new(0.08, 0.08, 0.2, 1.0)),
        &mut rng,
    );

    commands.spawn((
        Mesh3d(meshes.add(mountains)),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            ..Default::default()
        })),
        Transform::from_translation(Vec3::new(0.0, -0.3, MOUNTAINS_DEPTH))
            .with_scale(Vec3::splat(0.3)),
        Moutains { speed: 0.3 },
    ));
}

const NUM_SAMPLES: usize = 128;

fn create_mountain_mesh(
    top_color: LinearRgba,
    bottom_color: LinearRgba,
    rng: &mut ChaCha8Rng,
) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::RENDER_WORLD,
    );

    let mut height: Vec<f32> = Vec::with_capacity(NUM_SAMPLES);
    height.resize(NUM_SAMPLES + 1, 0.);
    for i in (0..NUM_SAMPLES).step_by(4) {
        height[i] = rng.random_range(0.2..0.3);
    }
    height[NUM_SAMPLES] = height[0];

    fn gen_fract(height: &mut [f32], i0: usize, i1: usize, rng: &mut ChaCha8Rng) {
        let h0 = height[i0];
        let h1 = height[i1];
        let im = (i0 + i1) / 2;
        height[im] = (h0 + h1) * 0.5 + rng.random_range(-0.02..0.02);
        if i1 > i0 + 1 {
            gen_fract(height, i0, im, rng);
            gen_fract(height, im, i1, rng);
        }
    }

    for i in (0..NUM_SAMPLES).step_by(4) {
        gen_fract(&mut height, i, i + 4, rng);
    }

    // Remove last sample
    height.pop();

    let mut v_pos: Vec<[f32; 3]> = Vec::with_capacity(NUM_SAMPLES * 2);
    let mut v_color: Vec<[f32; 4]> = Vec::with_capacity(NUM_SAMPLES * 2);
    for (i, h) in height.iter().enumerate() {
        let x = i as f32 * PLAYFIELD_WIDTH / NUM_SAMPLES as f32;
        v_pos.push([x, *h, 0.0]);
        v_pos.push([x, 0.0, 0.0]);
        v_color.push(bottom_color.mix(&top_color, *h * 4.0).to_f32_array());
        v_color.push(bottom_color.to_f32_array());
    }
    for (i, h) in height.iter().enumerate() {
        let x = i as f32 * PLAYFIELD_WIDTH / NUM_SAMPLES as f32;
        v_pos.push([x + PLAYFIELD_WIDTH, *h, 0.0]);
        v_pos.push([x + PLAYFIELD_WIDTH, 0.0, 0.0]);
        v_color.push(bottom_color.mix(&top_color, *h * 4.0).to_f32_array());
        v_color.push(bottom_color.to_f32_array());
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, v_color);
    mesh
}

pub(crate) fn update_mountains(
    r_viewpoint: Res<Viewpoint>,
    mut q_mountains: Query<(&Moutains, &mut Transform)>,
) {
    for (mtn, mut transform) in q_mountains.iter_mut() {
        // Parallax scrolling: offset each moutain by it's speed relative to the camera offset,
        // and then use modulo to implement wrap-around.
        let dist_traveled = PLAYFIELD_WIDTH * mtn.speed;
        transform.translation.x =
            (-r_viewpoint.position * mtn.speed).rem_euclid(dist_traveled) - dist_traveled * 1.5;
    }
}
