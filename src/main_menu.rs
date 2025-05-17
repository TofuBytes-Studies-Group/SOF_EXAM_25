// menu.rs
// Systems for MainMenu and SettingsMenu (and submenus)

use bevy::prelude::*;
use bevy_ascii_terminal::{color, StringDecorator, Terminal, TerminalBorder};
use crate::{AppState, GlobalTerminal};

#[derive(Component)]
pub struct MainMenuTag;
#[derive(Component)]
pub struct SettingsMenuTag;
#[derive(Component)]
pub struct DisplaySettingsTag;
#[derive(Component)]
pub struct SoundSettingsTag;

#[derive(Resource, Default)]
pub struct PendingState(pub Option<AppState>);
// Spawn the main menu terminal on entering MainMenu state
pub fn enter_menu(mut query: Query<&mut Terminal, With<GlobalTerminal>>) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.put_string([0, 2], "========================================".fg(color::WHITE));
        term.put_string([0, 3], ":: DUNGEON INTERFACE - MAIN MENU        ".fg(color::LIGHT_GREEN));
        term.put_string([0, 4], "========================================".fg(color::WHITE));
        term.put_string([0, 6], "  > [1] DESCEND INTO SECTOR DNK-34      ".fg(color::GREEN));
        term.put_string([0, 8], "  > [2] CONFIGURE TERMINAL SETTINGS     ".fg(color::GREEN));
        term.put_string([0, 11],"________________________________________".fg(color::WHITE));
        term.put_string([0, 13],"  > [Esc] EXIT                          ".fg(color::RED));
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
        // Go to Lore (then InGame)
        pending.0 = Some(AppState::Lore);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        // Go to Settings menu
        pending.0 = Some(AppState::SettingsMenu);
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

// Spawn the settings menu terminal on entering SettingsMenu state
pub fn enter_settings(mut query: Query<&mut Terminal, With<GlobalTerminal>>) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.put_string([0, 2], "=== SETTINGS ===".fg(color::YELLOW));
        term.put_string([0, 4], "[1] Display Settings".fg(color::WHITE));
        term.put_string([0, 5], "[2] Sound Settings".fg(color::WHITE));
        term.put_string([0, 6], "[Esc] Back".fg(color::WHITE));
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
        term.put_string([0, 2], "--- DISPLAY SETTINGS ---".fg(color::YELLOW));
        term.put_string([0, 4], "Press [Esc] to go back".fg(color::WHITE));
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
        term.put_string([0, 2], "--- SOUND SETTINGS ---".fg(color::YELLOW));
        term.put_string([0, 4], "Press [Esc] to go back".fg(color::WHITE));
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

pub fn exit_sound(mut commands: Commands, query: Query<Entity, With<Terminal>>) {
/*    for e in &query {
        commands.entity(e).despawn_recursive();
    }*/
}
