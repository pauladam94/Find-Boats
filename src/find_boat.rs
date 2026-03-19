use bevy::input::ButtonState;
use bevy::prelude::*;

use bevy::input::mouse::MouseButtonInput;
use bevy::window::PrimaryWindow;
use rand::random_range;

use super::FollowMouse;

use super::setup_cursor;
use bevy::color::palettes::css::BLACK;
use bevy::color::palettes::css::GREEN;

pub(crate) const NUMBER_BOATS: i32 = 10;
pub(crate) const WIDTH: i32 = 100;
pub(crate) const HEIGHT: i32 = 100;
pub(crate) const SCALE_BOATS: f32 = 0.003;
pub(crate) const WIDTH_GRID: f32 = 0.03;
pub(crate) const CURSOR_SIZE: f32 = 0.2;

// CAMERA PARAMETER
pub(crate) const CAMERA_MIN_SCALE: f32 = 0.01;
pub(crate) const CAMERA_MAX_SCALE: f32 = 0.05;
pub(crate) const CAMERA_TRANSLATION_SPEED: f32 = 10.;
pub(crate) const CAMERA_SCALE_SPEED: f32 = 1.;

pub(crate) const EPS: f32 = 0.01;
pub(crate) const Z0: f32 = 0.;
pub(crate) const Z1: f32 = Z0 - EPS;
pub(crate) const Z2: f32 = Z0 - 2. * EPS;
pub(crate) const Z3: f32 = Z0 - 3. * EPS;
pub(crate) const Z4: f32 = Z0 - 4. * EPS;
pub(crate) const Z5: f32 = Z0 - 5. * EPS;
pub(crate) const Z6: f32 = Z0 - 6. * EPS;
pub(crate) const Z7: f32 = Z0 - 7. * EPS;
pub(crate) const Z8: f32 = Z0 - 8. * EPS;

#[derive(Component)]
pub(crate) struct Boat;

pub(crate) fn game_plugin(app: &mut App) {
    app.insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, (setup_cursor, spawn_boats, setup_board))
        .add_systems(Update, (circle_follow_mouse, move_camera, sonar_on_click));
}

pub(crate) fn setup_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Background
    commands.spawn((
        Mesh2d(meshes.add(Rectangle {
            half_size: Vec2::new(WIDTH as f32 / 2., HEIGHT as f32 / 2.),
        })),
        MeshMaterial2d(materials.add(Color::from(GREEN))),
        Transform::from_translation(Vec3::new(WIDTH as f32 / 2., HEIGHT as f32 / 2., Z8)),
    ));

    for x in 0..WIDTH + 1 {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle {
                half_size: Vec2::new(WIDTH_GRID, HEIGHT as f32 / 2.),
            })),
            MeshMaterial2d(materials.add(Color::from(BLACK))),
            Transform::from_translation(Vec3::new(x as f32, HEIGHT as f32 / 2., Z6)),
        ));
    }

    for y in 0..HEIGHT + 1 {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle {
                half_size: Vec2::new(HEIGHT as f32 / 2., WIDTH_GRID),
            })),
            MeshMaterial2d(materials.add(Color::from(BLACK))),
            Transform::from_translation(Vec3::new(WIDTH as f32 / 2., y as f32, Z6)),
        ));
    }
}

pub(crate) fn spawn_boats(mut commands: Commands, asset_server: Res<AssetServer>) {
    for _ in 0..NUMBER_BOATS {
        commands.spawn((
            Sprite::from_image(asset_server.load("boat.png")),
            Transform {
                translation: Vec3::new(
                    random_range(0..WIDTH) as f32,
                    random_range(0..HEIGHT) as f32,
                    Z5,
                ),
                scale: Vec3::ONE * SCALE_BOATS,
                ..default()
            },
            Boat,
        ));
    }
}

pub(crate) fn circle_follow_mouse(
    // mut mouse_motions: EventReader<MouseMotion>,
    mut mouse_motions: EventReader<CursorMoved>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut circle_query: Query<&mut Transform, With<FollowMouse>>,
) {
    let (camera, camera_transform) = *camera_query;

    for motion in mouse_motions.read() {
        for mut transform in &mut circle_query {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, motion.position) {
                transform.translation =
                    Vec3::new(world_pos.x, world_pos.y, transform.translation.z);
            };
        }
    }
}

pub(crate) fn dist_closest_ship(boats: Query<&Transform, With<Boat>>, pos: Vec2) -> Option<f32> {
    boats.iter().fold(None, |dist, transform| match dist {
        None => Some(transform.translation.xy().distance(pos)),
        Some(dist) => Some(dist.min(transform.translation.xy().distance(pos))),
    })
}

pub(crate) fn sonar_on_click(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,

    mut input: EventReader<MouseButtonInput>,
    // mut camera_query: Query<(&mut Camera, &mut Transform)>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    // mut time: Res<Time<Fixed>>,
    window: Single<&Window, With<PrimaryWindow>>,

    boats: Query<&Transform, With<Boat>>,
) {
    let (camera, camera_transform) = *camera_query;

    for mouse_button in input.read() {
        match mouse_button {
            MouseButtonInput {
                state: ButtonState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(pos) = window.cursor_position()
                    && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, pos)
                    && let Some(dist) = dist_closest_ship(boats, world_pos)
                {
                    const WIDTH_ANNULUS: f32 = 0.1;
                    commands.spawn((
                        Mesh2d(meshes.add(Annulus::new(dist, dist + WIDTH_ANNULUS))),
                        MeshMaterial2d(materials.add(Color::from(BLACK))),
                        Transform::from_translation(Vec3::new(world_pos.x, world_pos.y, Z2)),
                    ));
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn move_camera(
    input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut Camera, &mut Transform)>,
    time: Res<Time<Fixed>>,
) {
    let Ok((_, mut transform)) = camera_query.single_mut() else {
        return;
    };

    // translation speed
    let tspeed = CAMERA_TRANSLATION_SPEED * time.delta_secs();
    // scale speed
    let sspeed = CAMERA_SCALE_SPEED * time.delta_secs();

    // Camera Transform
    if input.pressed(KeyCode::ArrowUp) {
        transform.translation.y += tspeed;
    }
    if input.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= tspeed;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= tspeed;
    }
    if input.pressed(KeyCode::ArrowRight) {
        transform.translation.x += tspeed;
    }

    // Camera Scale
    if input.pressed(KeyCode::Enter) {
        transform.scale += sspeed;
    }
    if input.pressed(KeyCode::Space) {
        transform.scale -= sspeed;
    }

    transform.translation.x = transform.translation.x.clamp(0., WIDTH as f32);
    transform.translation.y = transform.translation.y.clamp(0., HEIGHT as f32);

    transform.scale = transform
        .scale
        .clamp(Vec3::splat(CAMERA_MIN_SCALE), Vec3::splat(CAMERA_MAX_SCALE));
}
