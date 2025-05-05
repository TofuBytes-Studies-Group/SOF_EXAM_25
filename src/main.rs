use bevy::{prelude::*};

use bevy_ascii_terminal::{BorderGlyphs, TerminalBundle, TiledCameraBundle};
use std::collections::HashMap;
use bevy::ecs::system::Resource;
mod bundle;
mod config;
mod map;
mod map_state;
mod monster;
mod movement;
mod player;
mod render;
mod shapes;
mod visibility;
mod ui;
mod events;
//mod web_resize;
mod turn_system;
mod combat;
mod rng;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
    InputName,
    WorldIntro,
    Playing,
}

#[derive(Default, Debug)]
struct PlayerData {
    pub name: Option<String>,
    pub world_lore: Option<String>,
}

#[derive(Component)]
struct MenuTerminal;

#[derive(Component)]
pub struct GameTerminal;


pub const VIEWPORT_SIZE: [u32;2] = [80,40];

pub const UI_SIZE: [u32;2] = [VIEWPORT_SIZE[0],8];
// TODO: Map size should be separate
pub const GAME_SIZE: [u32;2] = [VIEWPORT_SIZE[0], VIEWPORT_SIZE[1] - UI_SIZE[1]];
fn setup(mut commands: Commands) {
    //commands.spawn().insert(gen.map);

    let term_y = VIEWPORT_SIZE[1] as f32 / 2.0 - GAME_SIZE[1] as f32 / 2.0; 
    let term_bundle = TerminalBundle {
        transform: Transform::from_xyz(0.0, term_y, 0.0),
        ..TerminalBundle::new().with_size([GAME_SIZE[0], GAME_SIZE[1] + 2])
    };
    //term_bundle.transform = Transform::from_xyz(0.0, 0.0, UI_SIZE[1] as f32 * 2.0);
    commands.spawn_bundle(term_bundle).insert(GameTerminal);

    let totalx = GAME_SIZE[0];
    let totaly = GAME_SIZE[1] + UI_SIZE[1];
    commands.spawn_bundle(TiledCameraBundle::new().with_tile_count([totalx, totaly]));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::MainMenu)
        .insert_resource(PlayerData::default())
        .add_startup_system_set(
            SystemSet::on_enter(GameState::MainMenu).with_system(setup_menu)
        )
        .add_system_set(
            SystemSet::on_update(GameState::MainMenu).with_system(handle_menu_input)
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Playing).with_system(setup)
        )
        .add_plugin(player::PlayerPlugin)
        .add_plugin(map::MapGenPlugin)
        .add_plugin(render::RenderPlugin)
        .add_plugin(events::EventsPlugin)
        .add_plugin(visibility::VisiblityPlugin)
        .add_plugin(map_state::MapStatePlugin)
        .add_plugin(turn_system::TurnSystemPlugin)
        .add_plugin(monster::MonstersPlugin)
        .add_plugin(combat::CombatPlugin)
        .add_plugin(ui::UiPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .run();
}

fn setup_menu(mut commands: Commands) {
    let mut terminal = TerminalBundle::new().with_size([UI_SIZE[0], UI_SIZE[1]]);
    terminal.terminal.draw_border(BorderGlyphs::single_line());
    terminal.terminal.put_string([1, 1], "Main Menu:");
    terminal.terminal.put_string([1, 2], "1. New Game");
    terminal.terminal.put_string([1, 3], "2. Load Game");
    terminal.terminal.put_string([1, 4], "3. Exit");

    commands.spawn_bundle(terminal).insert(MenuTerminal);
    
}

fn handle_menu_input(
    mut state: ResMut<State<GameState>>,
    mut input: ResMut<Input<KeyCode>>,
    mut player_data: ResMut<PlayerData>,
) {
    if input.just_pressed(KeyCode::Key1) {
        state.set(GameState::Playing).unwrap(); // Transition to Playing state
        input.clear();
    } else if input.just_pressed(KeyCode::Key2) {
        // Load game logic (not implemented here)
        input.clear();
    } else if input.just_pressed(KeyCode::Key3) {
        std::process::exit(0);
    }
}

fn handle_name_input(
    mut state: ResMut<State<GameState>>,
    mut input: ResMut<Input<KeyCode>>,
    mut player_data: ResMut<PlayerData>,
) {
    // Simulate name input (replace with actual text input handling)
    if input.just_pressed(KeyCode::Return) {
        player_data.name = Some("PlayerName".to_string());
        state.set(GameState::WorldIntro).unwrap();
        input.clear();
    }
}

fn generate_world_intro(mut player_data: ResMut<PlayerData>) {
    if let Some(name) = &player_data.name {
        player_data.world_lore = Some(format!(
            "Welcome, {}! You are the chosen hero of this world. Your journey begins now...",
            name
        ));
    }
}

fn handle_intro_continue(
    mut state: ResMut<State<GameState>>,
    mut input: ResMut<Input<KeyCode>>,
    player_data: Res<PlayerData>,
) {
    if input.just_pressed(KeyCode::Space) {
        state.set(GameState::Playing).unwrap();
        input.clear();
    }
}

fn start_game() {
    println!("Game started!");
}