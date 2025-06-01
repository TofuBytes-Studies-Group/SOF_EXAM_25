use std::sync::Arc;
use bevy::ecs::spawn::{SpawnWith, SpawnableList};
use bevy::platform::collections::Equivalent;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::time::{Timer};
use bevy_ascii_terminal::{
    StringDecorator, Terminal, TerminalBorder, TerminalCamera, TerminalMeshWorldScaling,
    TerminalPlugins, Tile, color,
};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use rand::{Rng, rng};
use sark_grids::GridSize;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use tokio_postgres::Error;
use crate::combat::{Defense, HitPoints, MaxHitPoints, Strength};
use crate::dbs::playerdb;
use crate::dbs::playerdb::PlayerDb;
use crate::dbs::psqldb::Database;
use crate::dbs::redisdb::RedisDatabase;
use crate::dbs::mongodb::LoreDatabase;

use crate::main_menu::{apply_pending_state, save_player_after_creation, CharacterName, PendingState, PlayerSaved};
use crate::player::Player;
use bevy_async_task::*;
use tokio::runtime::Runtime;
use crate::game::poll_lore_save_task;

mod PathMap2dExt;
mod bundle;
mod combat;
mod config;
mod events;
mod map;
mod map_state;
mod monster;
mod movement;
mod player;
mod render;
mod rng;
mod shapes;
mod turn_system;
mod ui;
mod visibility;
mod main_menu;

#[derive(Resource, Deref, DerefMut)]
struct LoreTimer(Timer);
#[derive(Resource, Default)]
struct MenuCleanupPending;



pub const VIEWPORT_SIZE: [u32; 2] = [80, 40];
pub const UI_SIZE: [u32; 2] = [VIEWPORT_SIZE[0], 8];
pub const GAME_SIZE: [u32; 2] = [VIEWPORT_SIZE[0], VIEWPORT_SIZE[1] - UI_SIZE[1]];
pub const TEXT_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
// main.rs
// Main application entrypoint and state setup

mod game;
mod dbs;
mod weapon_prediction;
mod generating_weapon;

// Define the application states
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum AppState {
    #[default]
    Splash,
    MainMenu,
    CharacterCreation,
    SettingsMenu,
    DisplaySettings,
    SoundSettings,
    Lore,
    GeneratingWeapon,
    WeaponSetup,
    InGame,
}

#[derive(Resource)]
struct TokioHandle(Arc<Runtime>);
 fn main() -> Result<(), Error> {

     let rt = Runtime::new().expect("Failed to create Tokio runtime");
     let rt_handle = Arc::new(rt);

     let psql: Database = rt_handle
         .block_on(Database::connect())
         .expect("Postgres init failed");
     let redis: RedisDatabase = rt_handle
         .block_on(RedisDatabase::new("redis://127.0.0.1/"))
         .expect("Redis init failed");
     let mongodb: LoreDatabase = rt_handle
         .block_on(LoreDatabase::new("mongodb://localhost:60000"))
         .expect("MongoDB init failed");

    App::new()
        .insert_resource(TokioHandle(rt_handle.clone()))
        // Standard Bevy and ASCII-terminal plugins
        .add_plugins((DefaultPlugins, TerminalPlugins))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .add_plugins(WorldInspectorPlugin::new())
        // PLUGINS FOR THE GAME / INGAME STATE EXCLUSIVELY
        .add_plugins(player::PlayerPlugin)
        .add_plugins(map::MapGenPlugin)
        .add_plugins(render::RenderPlugin)
        .add_plugins(events::EventsPlugin)
        .add_plugins(visibility::VisibilityPlugin)
        .add_plugins(map_state::MapStatePlugin)
        .add_plugins(turn_system::TurnSystemPlugin)
        .add_plugins(monster::MonstersPlugin)
        .add_plugins(combat::CombatPlugin)
        .add_plugins(ui::UiPlugin)
        // GAME PLUGINS END
        .insert_resource(PendingState::default())
        .insert_resource(CharacterName::default())

        .insert_resource(psql)
        .insert_resource(redis)
        .insert_resource(PlayerSaved::default())
        .insert_resource(mongodb)


        .add_systems(Startup, setup_camera)

        .add_systems(OnEnter(AppState::Splash), setup_terminal)
        .add_systems(Update, setup_splash_input.run_if(in_state(AppState::Splash)))

        // Use Default state as MainMenu
        .init_state::<AppState>()

        // Spawn the global terminal camera once at startup

        // Menu systems
        .add_systems(OnEnter(AppState::MainMenu), main_menu::enter_menu)
        .add_systems(Update, main_menu::menu_input.run_if(in_state(AppState::MainMenu)))
        .add_systems(OnExit(AppState::MainMenu), main_menu::exit_menu)

        .add_systems(Update, apply_pending_state)




        // Settings menu
        .add_systems(OnEnter(AppState::SettingsMenu), main_menu::enter_settings)
        .add_systems(Update, main_menu::settings_input.run_if(in_state(AppState::SettingsMenu)))
        .add_systems(OnExit(AppState::SettingsMenu), main_menu::exit_settings)

        // Display Settings submenu
        .add_systems(OnEnter(AppState::DisplaySettings), main_menu::enter_display)
        .add_systems(Update, main_menu::display_input.run_if(in_state(AppState::DisplaySettings)))
        .add_systems(OnExit(AppState::DisplaySettings), main_menu::exit_display)

        // Sound Settings submenu
        .add_systems(OnEnter(AppState::SoundSettings), main_menu::enter_sound)
        .add_systems(Update, main_menu::sound_input.run_if(in_state(AppState::SoundSettings)))
        .add_systems(OnExit(AppState::SoundSettings), main_menu::exit_sound)

        // Character creation menu
        .add_systems(OnEnter(AppState::CharacterCreation), main_menu::enter_character_creation)
        .add_systems(Update, main_menu::character_creation_input.run_if(in_state(AppState::CharacterCreation)))

        // Lore screen
        .add_systems(OnEnter(AppState::Lore), game::enter_lore)
        .add_systems(Update, game::lore_input.run_if(in_state(AppState::Lore)))
        .add_systems(Update, poll_lore_save_task.run_if(in_state(AppState::Lore)))
        .add_systems(OnExit(AppState::Lore), save_player_after_creation)
        .add_systems(OnExit(AppState::Lore), game::exit_lore)

        .add_systems(OnExit(AppState::Lore), generating_weapon::start_weapon_generation)

        .add_systems(OnEnter(AppState::GeneratingWeapon), generating_weapon::setup_loading_bar)
        .add_systems(
                Update,
                (
                    generating_weapon::update_loading_bar,
                    generating_weapon::show_loading_screen,
                    generating_weapon::poll_weapon_generation,
                    generating_weapon::wait_then_switch_state,
                ).run_if(in_state(AppState::GeneratingWeapon)),
            )
        .add_systems(OnEnter(AppState::WeaponSetup), generating_weapon::display_weapon_info)
        .add_systems(OnExit(AppState::WeaponSetup), generating_weapon::save_weapon_after_creation)
        .add_systems(Update, generating_weapon::weapon_continue_input.run_if(in_state(AppState::WeaponSetup)))
        
        // Weapon setup screen

        // In-game screen
        //.add_systems(OnEnter(AppState::InGame), game::enter_game)
        .add_systems(Update, game::game_input.run_if(in_state(AppState::InGame)))
        .add_systems(OnExit(AppState::InGame), game::exit_game)
        .run();
Ok(())
}

#[derive(Component)]
pub struct GlobalTerminal;
// Spawn a single TerminalCamera to cover all states:contentReference[oaicite:6]{index=6}
fn setup_terminal(mut commands: Commands) {
    commands.spawn((
        Terminal::new([50, 30])
            .with_string([0, 0],  "==================================================".fg(color::WHITE))
            .with_string([0, 2],  ":: DUNGEON MODULE: DNK-34                         ".fg(color::LIGHT_GREEN))
            .with_string([0, 3],  ":: ORC CLUSTER ENGAGED - SECTOR 12                ".fg(color::LIGHT_GREEN))
            .with_string([0, 5],  "==================================================".fg(color::WHITE))
            .with_string([0, 7],  "> Boot sequence.............................. OK  ".fg(color::LIGHT_GREEN))
            .with_string([0, 9], "> Loading terrain map............ ██████████ 85%  ".fg(color::LIGHT_GREEN))
            .with_string([0, 11], "> Signal interference.................. DETECTED  ".fg(color::DARK_ORANGE))
            .with_string([0, 13], "> Orc presence........................ CONFIRMED  ".fg(color::DARK_ORANGE))
            .with_string([0, 15], "> Goblin presence..................... CONFIRMED  ".fg(color::DARK_ORANGE))
            .with_string([0,17],  "  █     █   ██   ████   █    █  █  █    █   ████  ".fg(color::DARK_RED))
            .with_string([0,18],  "  █     █  █  █  █   █  ██   █  █  ██   █  █      ".fg(color::DARK_RED))
            .with_string([0,19],  "  █  █  █  ████  ████   █ █  █  █  █ █  █  █ ███  ".fg(color::DARK_RED))
            .with_string([0,20],  "  █ ███ █  █  █  █   █  █  █ █  █  █  █ █  █   █  ".fg(color::DARK_RED))
            .with_string([0,21],  "   █   █   █  █  █   █  █    █  █  █    █   ████  ".fg(color::DARK_RED))
            .with_string([0,23],  ">> CONTAMINATION LEVEL: 87%                       ".fg(color::RED))
            .with_string([0,25],  ">> PROCEED WITH CAUTION                           ".fg(color::RED))
            .with_string([0,27],  "==================================================".fg(color::WHITE))
            .with_string([0,29],  "         [ENTER] Descend || [ESC] To Quit         ".fg(color::GREEN)),
        TerminalBorder::single_line(),
        GlobalTerminal,
        Transform::default(),
        GlobalTransform::default(),
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(TerminalCamera::new());
}
pub fn setup_splash_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>,) {
    if keyboard.just_pressed(KeyCode::Enter) {
        // Go to Lore (then InGame)
        next_state.set(AppState::MainMenu);
    }
}



