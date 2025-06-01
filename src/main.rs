use bevy::prelude::*;
use game_state::{GameState, PauseState};

mod game_state;

#[derive(Resource)]
pub struct UiCamera(pub Entity);

impl Default for UiCamera {
    fn default() -> Self {
        UiCamera(Entity::PLACEHOLDER)
    }
}

/// Marker component for game camera
#[derive(Component, Default, Debug)]
struct PlayfieldCamera;

/// Marker component for main content area
#[derive(Component, Default, Debug)]
struct MainContent;

fn main() {
    // Customize the window title and size
    let window = Window {
        title: "Guardian 2".into(),
        resize_constraints: bevy::window::WindowResizeConstraints {
            min_width: 400.0,
            min_height: 300.0,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        },
        ..default()
    };
    // load_window_settings(&mut prefs, &mut window);

    let mut app = App::new();
    app.add_plugins((DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        })
        .set(AssetPlugin::default()),))
        .init_state::<GameState>()
        .init_state::<PauseState>()
        .init_resource::<UiCamera>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_viewport_area)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut r_ui_camera: ResMut<UiCamera>,
) {
    // UI camera
    let ui_camera = commands
        .spawn((
            Camera2d,
            Camera {
                clear_color: Color::srgb(0.0, 0.0, 0.0).into(),
                ..default()
            },
        ))
        .id();

    r_ui_camera.0 = ui_camera;

    // UI root entity
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        },
        UiTargetCamera(ui_camera),
        children![
            (
                // Header section with minimap
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    min_height: Val::Px(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.0, 0.0, 0.1)),
                children![(
                    Node {
                        min_height: Val::Percent(80.0),
                        aspect_ratio: Some(5.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.0, 0.5, 0.0))
                ),],
            ),
            // Main content section
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                },
                // BackgroundColor(Color::srgb(1.0, 0.0, 1.0)),
                MainContent
            ),
        ],
    ));

    // ortho camera
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        PlayfieldCamera,
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::AutoMax {
                max_width: 1.0,
                max_height: 1.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Starfield (Nebula)
    let nebula = asset_server.load("textures/stars.jpg");
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1.5, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(nebula),
            unlit: true,
            ..Default::default()
        })),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));
}

const MIN_ASPECT: f32 = 1.5;
const MAX_ASPECT: f32 = 2.5;

fn update_viewport_area(
    q_main_content: Single<(&ComputedNode, &GlobalTransform), With<MainContent>>,
    q_camera: Single<&mut Camera, With<PlayfieldCamera>>,
    q_window: Single<&Window>,
) {
    let window = q_window.into_inner();
    let window_rect = Rect {
        min: Vec2::ZERO,
        max: Vec2::new(
            window.resolution.physical_width() as f32,
            window.resolution.physical_height() as f32,
        ),
    };
    // let sf = window.resolution.scale_factor();

    let (main_content, main_content_transform) = q_main_content.into_inner();

    // Avoid division by zero if window is too small
    if main_content.size().y < 1.0 {
        return;
    }

    let content_pos = main_content_transform.translation().truncate() - main_content.size() / 2.0;

    // Calculate the viewport size based on the available aspect ratio. If the available space is
    // too narrow, letterbox on top and bottom; if it's too wide, letterbox on the sides.
    let mut viewport_size = main_content.size();
    let content_aspect = viewport_size.x / viewport_size.y; // Avoid division by zero
    if content_aspect < MIN_ASPECT {
        viewport_size.y = viewport_size.x / MIN_ASPECT;
    } else if content_aspect > MAX_ASPECT {
        viewport_size.x = viewport_size.y * MAX_ASPECT;
    }

    let viewport_pos = (main_content.size() - viewport_size) * 0.5 + content_pos;
    let viewport_rect = Rect {
        min: viewport_pos,
        max: viewport_pos + viewport_size,
    }
    .intersect(window_rect);

    let mut camera = q_camera.into_inner();
    camera.viewport = Some(bevy::render::camera::Viewport {
        physical_position: viewport_rect.min.as_uvec2(),
        physical_size: viewport_rect.size().as_uvec2(),
        ..default()
    });
}
