use bevy::{color::palettes::css::PURPLE, prelude::*};

#[derive(Default, States, Debug, Eq, PartialEq, Hash, Clone)]
enum GameState {
    #[default]
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins((menu_plugin, find_boat::game_plugin))
        .run();
}

#[derive(Component)]
struct FollowMouse;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera { ..default() },
        Transform::from_xyz(0., 0., find_boat::Z0),
    ));
}

fn setup_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle {
            radius: find_boat::CURSOR_SIZE,
        })),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform::from_xyz(0., 0., find_boat::Z3),
        FollowMouse,
    ));
}

fn menu_plugin(app: &mut App) {
    app.add_systems(Update, quit_menu);
}

fn quit_menu(
    input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    println!("QUIT MENU FUNCTION");
    if input.pressed(KeyCode::KeyQ) {
        app_exit_events.write(AppExit::Success);
    }
    if input.pressed(KeyCode::Enter) {
        game_state.set(GameState::Game);
    }
}

mod find_boat;
