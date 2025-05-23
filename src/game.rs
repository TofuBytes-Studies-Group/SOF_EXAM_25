// game.rs
// Systems for Lore and InGame states

use bevy::prelude::*;
use bevy_ascii_terminal::{color, StringDecorator, Terminal, TerminalBorder};
use crate::{AppState, GlobalTerminal, GAME_SIZE, VIEWPORT_SIZE};
use crate::main_menu::CharacterName;

// Spawn the Lore (intro) terminal on entering Lore state
pub fn enter_lore(mut query: Query<&mut Terminal, With<GlobalTerminal>>, character_name: Res<CharacterName>,
) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.resize([50,30]);
        term.put_string([0, 2],  "================ < LORE ARCHIVE > ================".fg(color::YELLOW));
        term.put_string([0, 4],  ":: FILE: DNK-34 // PROTOCOL STATUS: ACTIVE        ".fg(color::LIGHT_GREEN));
        term.put_string([0, 6],  "==================================================".fg(color::WHITE));
        term.put_string([0, 8],  "Beneath the stone of the Northern Reach ".fg(color::LIGHT_GRAY));
        term.put_string([0, 9],  "lies the forgotten fortress-vault DNK-34.".fg(color::LIGHT_GRAY));
        term.put_string([0, 10], "Forged by Iron-Priests in an age of steam and rune".fg(color::LIGHT_GRAY));
        term.put_string([0, 11], "its halls are now ruled by rust-caked automatons".fg(color::LIGHT_GRAY));
        term.put_string([0, 12], "following the last order ever issued:".fg(color::LIGHT_GRAY));
        term.put_string([0, 13], "              **DENY ALL INTRUDERS**              ".fg(color::LIGHT_GRAY));
        term.put_string([0, 14], "Within lie arsenals of brass lightning,".fg(color::LIGHT_GRAY));
        term.put_string([0, 15], "tomes of forbidden craft, and the reactor-heart".fg(color::LIGHT_GRAY));
        term.put_string([0, 16], "whose ash still powers the iron guardians...".fg(color::LIGHT_GRAY));
        term.put_string([0, 17], "and as all others before them has failed..".fg(color::LIGHT_GRAY));
        term.put_string([0, 18], "fallen to the horrors of what lies beyond..".fg(color::LIGHT_GRAY));
        term.put_string([0, 19], "a new soul rises up to the challenge..".fg(color::LIGHT_GRAY));
        term.put_string([0, 20], "in a attempt to uncover the secrets of the vault.".fg(color::LIGHT_GRAY));
        term.put_string([0, 22], "a Hero only known as:".fg(color::LIGHT_GRAY));

        let name_display = format!("{}", character_name.0);
        term.put_string([21, 22], name_display.fg(color::GREEN));

        term.put_string([0, 25], ">>        Press [ENTER] Breach the vault        <<".fg(color::GREEN));
    } else {
        warn!("Global terminal not found ENTER LORE");
    }
}

// Handle input in the Lore state
pub fn lore_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::InGame);
    }
}

pub fn exit_lore(mut commands: Commands, query: Query<Entity, With<Terminal>>) {
/*    for e in &query {
        commands.entity(e).despawn();
    }*/
}

// Spawn the InGame terminal on entering InGame state
pub fn enter_game(mut query: Query<&mut Terminal, With<GlobalTerminal>>) {
    // Attempt to get the single terminal entity
    if let Ok(mut terminal) = query.single_mut() {
        // Clear existing contents
        terminal.clear();

        // Resize the terminal grid to GAME_SIZE width x (GAME_SIZE height + 2)
        terminal.resize([GAME_SIZE[0], GAME_SIZE[1] + 2]);
        

    } else {
        // Terminal not found; log and skip without crashing
        warn!("enter_game: no GlobalTerminal found; skipping terminal setup");
    }
}
// Handle input in the InGame state
pub fn game_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

pub fn exit_game(mut commands: Commands, query: Query<Entity, With<Terminal>>) {
/*    for e in &query {
        commands.entity(e).despawn();
    }*/
}
