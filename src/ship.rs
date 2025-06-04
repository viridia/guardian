use std::f32::consts::PI;

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

use crate::{MainInput, Move, PLAYFIELD_WIDTH, SHIP_DEPTH, Viewpoint};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Facing {
    #[default]
    Right,
    Left,
}

/// State of the player's ship
#[derive(Component, Default, Debug)]
pub struct PlayerShip {
    /// Direction we want to be facing, sticky based on thrust
    facing: Facing,

    /// Where ship is relative to playfield origin
    position: f32,

    /// This is based on facing, but smoothed.
    pub camera_offset: f32,

    /// Horizontal velocity
    speed: f32,

    /// Current ship orientation - follows facing but smoothed
    pitch: f32,

    /// Yaw is affected by both spin and up / down movements.
    yaw: f32,

    /// The size of the thrust animation
    thrust: f32,
}

#[derive(Component, Default, Debug)]
pub struct Thrust;

pub(crate) fn spawn_ship(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut thrust_cone = ConicalFrustum {
        radius_top: 0.2,
        radius_bottom: 0.6,
        height: 4.0,
    }
    .mesh()
    .build();
    // Derive vertex colors from positions
    let v_pos: Vec<[f32; 4]> = thrust_cone
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|pos| {
            LinearRgba::new(0.05, 0.05, 0.5, (0.0 - pos[1] / 4.0).clamp(0.0, 0.4)).to_f32_array()
        })
        .collect();
    thrust_cone.insert_attribute(Mesh::ATTRIBUTE_COLOR, v_pos);
    thrust_cone.translate_by(Vec3::new(0.0, 2.0, 0.0));

    let mesh = meshes.add(thrust_cone);

    // Player ship model
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/ship.glb"))),
        Transform::from_scale(Vec3::splat(0.015)).with_translation(Vec3::new(0.0, 0.0, SHIP_DEPTH)),
        PlayerShip {
            facing: Facing::Right,
            position: 0.,
            camera_offset: 0.,
            speed: 0.,
            pitch: 0.,
            yaw: 0.,
            thrust: 0.,
        },
        Actions::<MainInput>::default(),
        AudioPlayer::new(asset_server.load("sounds/thrust.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            speed: 0.2,
            volume: Volume::Linear(0.),
            ..default()
        },
        children![
            (
                Mesh3d(mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    alpha_mode: AlphaMode::Add,
                    unlit: true,
                    ..default()
                })),
                Transform::from_rotation(Quat::from_rotation_z(PI * 0.5))
                    .with_translation(Vec3::new(-3.6, 0.1, -0.8)),
                Thrust
            ),
            (
                Mesh3d(mesh),
                MeshMaterial3d(materials.add(StandardMaterial {
                    alpha_mode: AlphaMode::Add,
                    unlit: true,
                    ..default()
                })),
                Transform::from_rotation(Quat::from_rotation_z(PI * 0.5))
                    .with_translation(Vec3::new(-3.6, 0.1, 0.8)),
                Thrust
            )
        ],
    ));
}

pub(crate) fn move_ship(
    player: Single<
        (
            &Actions<MainInput>,
            &mut PlayerShip,
            &mut Transform,
            &mut AudioSink,
        ),
        Without<Thrust>,
    >,
    mut q_thrust: Query<&mut Transform, With<Thrust>>,
    r_time: Res<Time>,
    mut r_viewpoint: ResMut<Viewpoint>,
) -> Result<()> {
    let (actions, mut ship, mut transform, mut audio) = player.into_inner();
    let move_action = actions.get::<Move>()?.value().as_axis2d();

    // Move the ship
    let accel = (-ship.speed * 2.0 + move_action.x * 10.) * r_time.delta_secs();
    ship.speed = (ship.speed + accel).clamp(-1.5, 1.5);
    ship.position = (ship.position + ship.speed * r_time.delta_secs()).rem_euclid(PLAYFIELD_WIDTH);
    transform.translation.y = (transform.translation.y + move_action.y * 0.005).clamp(-0.4, 0.45);

    // Facing is sticky: ship orientation matches most recent thrust action.
    let mut target_thrust = 0.;
    if move_action.x > 0. {
        ship.facing = Facing::Right;
        target_thrust = 1.0;
    } else if move_action.x < 0. {
        ship.facing = Facing::Left;
        target_thrust = 1.0;
    }

    // Adjust pitch if we flipped direction
    let target_pitch = match ship.facing {
        Facing::Right => 0.0,
        Facing::Left => -PI,
    };

    // Yaw to show top or bottom of ship when climbing or turning.
    let target_yaw = if target_pitch > ship.pitch + 0.5 {
        -0.5
    } else if target_pitch < ship.pitch - 0.5 {
        0.5
    } else if move_action.y > 0. {
        if ship.facing == Facing::Right {
            -0.2
        } else {
            0.2
        }
    } else if move_action.y < 0. {
        if ship.facing == Facing::Right {
            0.2
        } else {
            -0.2
        }
    } else {
        0.0
    };

    // Offset camera so there is more room in front of the ship than behind.
    let target_camera_offset = match ship.facing {
        Facing::Right => -0.3,
        Facing::Left => 0.3,
    };

    // TODO: Replace this with some kind of cheap noise source.
    let thrust_noise = 1.0 + (r_time.elapsed_secs() * 100.0).sin() * 0.3;

    // Smooth moves
    ship.yaw = transition_to_target(ship.yaw, target_yaw, r_time.delta_secs() * 3.);
    ship.pitch = transition_to_target(ship.pitch, target_pitch, r_time.delta_secs() * 15.);
    ship.camera_offset = transition_to_target(
        ship.camera_offset,
        target_camera_offset,
        r_time.delta_secs() * 0.3,
    );
    ship.thrust = transition_to_target(ship.thrust, target_thrust, r_time.delta_secs() * 15.);
    transform.translation.x = ship.camera_offset;
    transform.rotation = Quat::from_euler(EulerRot::YXZ, ship.pitch, ship.yaw, 0.0);
    r_viewpoint.position = (ship.position - ship.camera_offset).rem_euclid(PLAYFIELD_WIDTH);

    // Adjust shock cone scale
    for mut trust_transform in q_thrust.iter_mut() {
        trust_transform.scale = Vec3::new(1.0, ship.thrust * thrust_noise, 1.0);
    }

    // Adjust thrust sound
    audio.set_volume(Volume::Linear(ship.thrust * 0.8));

    Ok(())
}

pub(crate) fn transition_to_target(current: f32, target: f32, delta: f32) -> f32 {
    if current < target {
        (current + delta).min(target)
    } else if current > target {
        (current - delta).max(target)
    } else {
        target
    }
}
