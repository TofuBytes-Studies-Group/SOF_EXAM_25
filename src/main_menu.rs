// menu.rs
// Systems for MainMenu and SettingsMenu (and submenus)

use std::sync::{Arc, Mutex};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, IoTaskPool};
use bevy_ascii_terminal::{color, StringDecorator, Terminal, TerminalBorder};
use tokio::io::AsyncBufReadExt;
use tokio_postgres::Client;
use crate::{AppState, GlobalTerminal};
use crate::combat::{Defense, HitPoints, MaxHitPoints, Strength};
use crate::dbs::playerdb;
use crate::dbs::playerdb::PlayerDb;
use crate::dbs::psqldb::Database;
use crate::dbs::redisdb::RedisDatabase;
use crate::player::Player;

#[derive(Component)]
pub struct MainMenuTag;
#[derive(Component)]
pub struct SettingsMenuTag;
#[derive(Component)]
pub struct DisplaySettingsTag;
#[derive(Component)]
pub struct SoundSettingsTag;
#[derive(Component)]
pub struct CharacterCreationTag;
#[derive(Resource, Default)]
pub struct CharacterName(pub String);

#[derive(Resource, Default)]
pub struct PlayerNameList(pub Vec<String>);
#[derive(Resource, Default)]
pub struct PlayerNameListPending(pub Arc<Mutex<Option<Vec<String>>>>);
#[derive(Resource, Default)]
pub struct SelectedPlayer(pub Option<String>);
#[derive(Resource, Default)]
pub struct PlayerStatsDisplay(pub Option<Vec<String>>);
#[derive(Resource)]
pub struct PlayerStatsPending(Arc<Mutex<Option<Vec<String>>>>);

impl Default for PlayerStatsPending {
    fn default() -> Self {
        PlayerStatsPending(Arc::new(Mutex::new(None)))
    }
}


#[derive(Resource, Default)]
pub struct PendingState(pub Option<AppState>);
// Spawn the main menu terminal on entering MainMenu state
pub fn enter_menu(mut query: Query<&mut Terminal, With<GlobalTerminal>>) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.resize([50, 30]);
        term.put_string([0, 2],  "====================== MENU ======================".fg(color::YELLOW));
        term.put_string([0, 5],  "          [1] Descend into Sector DNK-34          ".fg(color::GREEN));
        term.put_string([0, 7],  "          [2] Configure Terminal Settings         ".fg(color::WHITE));
        term.put_string([0, 9],  "          [3] Player Statistics                   ".fg(color::WHITE));
        term.put_string([0, 11], "          [Esc] Quit                              ".fg(color::WHITE));
    } else {
        warn!("Global terminal not found in MAIN MENU");
    }
}


// Handle input in the main menu
pub fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: ResMut<Events<AppExit>>,
    mut pending: ResMut<PendingState>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        pending.0 = Some(AppState::CharacterCreation);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        // Go to Settings menu
        pending.0 = Some(AppState::SettingsMenu);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        // Go to Player Stats
        pending.0 = Some(AppState::SelectPlayer);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        // Quit the app
        exit.send_default();
    }
}

pub fn apply_pending_state(
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingState>,
) {
    if let Some(state) = pending.0.take() {
        next_state.set(state);
    }
}

// Cleanup (despawn) all Terminals when leaving MainMenu
pub fn exit_menu(mut commands: Commands, query: Query<Entity, With<MainMenuTag>>) {
/*    for e in &query {
        commands.entity(e).despawn();
    }*/
}

pub fn enter_character_creation(mut query: Query<&mut Terminal, With<GlobalTerminal>>, mut character_name: ResMut<CharacterName>) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.put_string([0, 2], "=============== CHARACTER CREATION ===============".fg(color::YELLOW));
        term.put_string([0, 4], "            Enter your character name:            ".fg(color::WHITE));
        term.put_string([0, 6],"              >".fg(color::WHITE));
        term.put_string([15, 6], character_name.0.clone());
        term.put_string([0, 8], "             Press [Enter] to confirm             ".fg(color::GREEN));
    }
}

pub fn character_creation_input(
    mut evr_kbd: EventReader<KeyboardInput>,
    mut char_name: ResMut<CharacterName>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<&mut Terminal, With<GlobalTerminal>>,
) {
    if let Ok(mut term) = query.single_mut() {
        for ev in evr_kbd.read() {
            if ev.state == ButtonState::Released {
                continue;
            }
            match &ev.logical_key {
                Key::Backspace => {
                    char_name.0.pop();
                }
                Key::Enter => {
                    if !char_name.0.is_empty() {
                        next_state.set(AppState::Lore);
                    }
                }
                Key::Character(s) => {
                    if s.chars().all(|c| !c.is_control()) && char_name.0.len() < 20 {
                        char_name.0.push_str(s);
                    }
                }
                _ => {}
            }
        }
        term.clear();
        term.put_string([0, 2], "=============== CHARACTER CREATION ===============".fg(color::YELLOW));
        term.put_string([0, 4], "            Enter your character name:            ".fg(color::WHITE));
        term.put_string([0, 6],"              >".fg(color::WHITE));
        term.put_string([15, 6],char_name.0.clone());
        term.put_string([0, 8], "             Press [Enter] to confirm             ".fg(color::GREEN));
    }
}

#[derive(Resource, Default)]
pub struct PlayerSaved(bool);

pub fn save_player_after_creation(
    mut saved: ResMut<PlayerSaved>,
    char_name: Res<CharacterName>,
    player_query: Query<(&HitPoints, &MaxHitPoints, &Defense, &Strength), With<Player>>,
    psql: Res<Database>,
    redis: Res<RedisDatabase>,
) {
    if saved.0 {
        return; // Already saved
    }

    if let Ok((hp, max_hp, defense, strength)) = player_query.single() {
        saved.0 = true; // Mark as saved

        let record = PlayerDb {
            id: Default::default(),
            name: char_name.0.clone(),
            hp: hp.0,
            max_hp: max_hp.0,
            defense: defense.0,
            strength: strength.0,
            inventory_id: None,
        };

        let db_client = psql.client.clone();
        let redis = redis.clone();

        bevy::tasks::IoTaskPool::get().spawn(async move {
            match PlayerDb::create(db_client, Arc::from(redis), &record.name, record.hp, record.max_hp, record.defense, record.strength).await {
                Ok(p) => println!("!!! Saved player to DB: {:?}", p),
                Err(e) => eprintln!(">>X<< Failed to save player: {}", e),
            }
        }).detach();
    }
}


// Spawn the settings menu terminal on entering SettingsMenu state
pub fn enter_settings(mut query: Query<&mut Terminal, With<GlobalTerminal>>) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.put_string([0, 2],  "==================== SETTINGS ====================".fg(color::YELLOW));
        term.put_string([0, 5],  "               [1] Display Settings               ".fg(color::WHITE));
        term.put_string([0, 7],  "               [2] Sound Settings                 ".fg(color::WHITE));
        term.put_string([0, 10], "               [Esc] Back                         ".fg(color::WHITE));
    } else {
        warn!("Global terminal not found ENTER SETTINGS");
    }
}

// Handle input in the settings menu
pub fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        next_state.set(AppState::DisplaySettings);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        next_state.set(AppState::SoundSettings);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

// Cleanup Terminals when leaving SettingsMenu
pub fn exit_settings(mut commands: Commands, query: Query<Entity, With<Terminal>>) {
/*    for e in &query {
        commands.entity(e).despawn_recursive();
    }*/
}

// Display Settings submenu
pub fn enter_display(mut query: Query<&mut Terminal, With<GlobalTerminal>>) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.put_string([0, 2], "---------------- DISPLAY SETTINGS ----------------".fg(color::YELLOW));
        term.put_string([0, 5], "              Press [Esc] to go back              ".fg(color::WHITE));
    } else {
        warn!("Global terminal not found in ENTER DISPLAY");
    }
}

pub fn display_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::SettingsMenu);
    }
}

pub fn exit_display(mut commands: Commands, query: Query<Entity, With<Terminal>>) {
/*    for e in &query {
        commands.entity(e).despawn_recursive();
    }*/
}

// Sound Settings submenu
pub fn enter_sound(mut query: Query<&mut Terminal, With<GlobalTerminal>>) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.put_string([0, 2], "----------------- SOUND SETTINGS -----------------".fg(color::YELLOW));
        term.put_string([0, 5], "              Press [Esc] to go back              ".fg(color::WHITE));
    } else {
        warn!("Global terminal not  in ENTER SOUND");
    }
}

pub fn sound_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::SettingsMenu);
    }
}

pub fn enter_select_player(
    mut query: Query<&mut Terminal, With<GlobalTerminal>>,
    db: Res<Database>,
    pending_names: Res<PlayerNameListPending>,
) {
    println!("🎮 Entering SelectPlayer screen...");
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.put_string([0, 2], "=========================== SELECT  PLAYER ===========================".fg(color::YELLOW));
        term.put_string([0, 4], "                           Fetching players...".fg(color::WHITE));

        let client = db.client.clone();
        let pending_clone = Arc::clone(&pending_names.0);

        bevy::tasks::IoTaskPool::get().spawn(async move {
            println!("🚀 Fetching player names...");
            let rows = client.lock().await.query("SELECT name FROM player", &[]).await.unwrap_or_default();
            let names: Vec<String> = rows.iter().map(|r| r.get(0)).collect();

            println!("✅ Fetched {} players", names.len());

            let mut guard = pending_clone.lock().unwrap();
            *guard = Some(names);
        }).detach();
    }
}

pub fn update_player_name_list(
    pending: Res<PlayerNameListPending>,
    mut list: ResMut<PlayerNameList>,
) {
    let mut guard = pending.0.lock().unwrap();
    if let Some(new_names) = guard.take() {
        println!("📥 Updating PlayerNameList with {} names", new_names.len());
        list.0 = new_names;
    }
}


pub fn draw_player_name_list(
    names: Res<PlayerNameList>,
    mut query: Query<&mut Terminal, With<GlobalTerminal>>,
) {
    println!(" draw_player_name_list system running...");
    if names.is_changed() {
        println!(" PlayerNameList changed — redrawing terminal");
        if let Ok(mut term) = query.single_mut() {
            term.clear();
            term.resize([70, 40]);
            term.put_string([0, 2], "=========================== SELECT  PLAYER ===========================".fg(color::YELLOW));

            if names.0.is_empty() {
                term.put_string([0, 4], "                           No players found                           ".fg(color::RED));
                println!("⚠ No players found.");
            } else {
                term.put_string([0, 4], "                           Choose a player:                           ".fg(color::WHITE));
                for (i, name) in names.0.iter().enumerate() {
                    let line = format!("[{}] {}", i + 1, name);
                    term.put_string([27, 4 + (i + 2) as i32], line.fg(color::GREEN));
                }
                term.put_string([0, 6 + names.0.len() as i32 + 2], "                              [Esc] Back                              ".fg(color::WHITE));
                println!(" Rendered {} player names.", names.0.len());
            }
        } else {
            println!(" Could not access terminal.");
        }
    } else {
        println!(" PlayerNameList unchanged — skipping redraw");
    }
}



pub fn update_player_statistics_ui(
    mut stats_display: ResMut<PlayerStatsDisplay>,
    pending: Res<PlayerStatsPending>,
    mut query: Query<&mut Terminal, With<GlobalTerminal>>,
) {
    // Check if new data arrived
    if let Some(new_data) = pending.0.lock().unwrap().take() {
        stats_display.0 = Some(new_data.clone());

        if let Ok(mut term) = query.single_mut() {
            term.clear();
            term.put_string([0, 2], "------------------------- PLAYER  STATISTICS -------------------------".fg(color::YELLOW));
            term.put_string([0, 29], "                        Press [Esc] to go back                        ".fg(color::WHITE));

            for (i, line) in new_data.iter().enumerate() {
                term.put_string([0, 5 + i as i32], line.fg(color::WHITE));
            }
        }
    }
}


pub fn select_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    names: Res<PlayerNameList>,
    mut selected: ResMut<SelectedPlayer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (i, _) in names.0.iter().enumerate() {
        let key = match i {
            0 => KeyCode::Digit1,
            1 => KeyCode::Digit2,
            2 => KeyCode::Digit3,
            3 => KeyCode::Digit4,
            4 => KeyCode::Digit5,
            _ => continue,
        };
        if keyboard.just_pressed(key) {
            selected.0 = Some(names.0[i].clone());
            next_state.set(AppState::PlayerStatistics);
        }
        if keyboard.just_pressed(KeyCode::Escape) {
            next_state.set(AppState::MainMenu);
        }
    }
}




pub fn enter_player_statistics(
    mut query: Query<&mut Terminal, With<GlobalTerminal>>,
    selected: Res<SelectedPlayer>,
    db: Res<Database>,
    mut stats_display: ResMut<PlayerStatsDisplay>,
    pending: Res<PlayerStatsPending>,
) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.put_string([0, 2], "------------------------- PLAYER  STATISTICS -------------------------".fg(color::YELLOW));
        term.put_string([0, 29], "                        Press [Esc] to go back                        ".fg(color::WHITE));

        if let Some(name) = &selected.0 {
            // Show loading feedback
            stats_display.0 = Some(vec!["Fetching stats...".to_string()]);
            *pending.0.lock().unwrap() = None; // Clear old data

            let client = db.client.clone();
            let name_clone = name.clone();
            let pending_clone = Arc::clone(&pending.0);

            // Spawn async task to fetch player data from DB
            IoTaskPool::get().spawn(async move {
                let result = PlayerDb::get_player_full_data(client, &name_clone).await;

                let lines = match result {
                    Ok(data) => {
                        let mut lines = vec![
                            format!("{:^70}", format!("> {}'s statistics <", data.player.name)),
                            "".to_string(),
                            format!("Character:  {}", data.player.name),
                            format!(" HP:  {}/MAX HP:  {}", data.player.hp, data.player.max_hp),
                            format!(" Defense:  {}", data.player.defense),
                            format!(" Strength:  {}", data.player.strength),
                            format!(" Gold:  {}", data.inventory.gold),
                            "".to_string(),
                            "Weapon:".to_string(),
                        ];

                        if data.weapons.is_empty() {
                            lines.push("  (No weapons equipped)".to_string());
                        } else {
                            for weapon in data.weapons {
                                lines.push(format!(" Name  {}", weapon.name));
                                lines.push(format!(" Damage  {}", weapon.damage));
                                lines.push(format!(" Weight  {}", weapon.weight));
                                lines.push(format!(" Upgrade  {}", weapon.upgrade));
                                lines.push(format!(" Perk  {}", weapon.perk));
                                if weapon.perk.len() > 30 {
                                    lines.push("".to_string());
                                    lines.push("".to_string());
                                }
                                lines.push(format!(" Type  {}", weapon.weapon_type));
                                lines.push(format!(
                                    "Price  {:?}",
                                    weapon.predicted_price
                                ));
                                lines.push("".to_string()); // spacer between weapons
                            }
                        }

                        lines
                    }
                    Err(e) => vec![format!("Error fetching data: {}", e)],
                };

                let mut lock = pending_clone.lock().unwrap();
                *lock = Some(lines);
            }).detach();

        }
    }
}
pub fn player_statistics_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}