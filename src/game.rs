// game.rs
// Systems for Lore and InGame states

use std::sync::Arc;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_ascii_terminal::{color, StringDecorator, Terminal, TerminalBorder};
use runtime::Runtime;
use serde::Serialize;
use crate::{AppState, GlobalTerminal, TokioHandle, GAME_SIZE, VIEWPORT_SIZE};
use crate::dbs::mongodb::{LoreDatabase, LoreEntry};
use crate::main_menu::CharacterName;
use crate::weapon_prediction::bridge::generate_weapon;
use serde_json::json;
use tokio::runtime;

#[derive(Resource)]
pub struct SaveLoreTask(Task<Result<(), anyhow::Error>>);


pub fn enter_lore(
    mut commands: Commands,
    mut query: Query<&mut Terminal, With<GlobalTerminal>>,
    character_name: Res<CharacterName>,
    lore_db: Res<LoreDatabase>,
    tokio_handle: Res<TokioHandle>,
) {
    if let Ok(mut term) = query.single_mut() {
        term.clear();
        term.resize([50, 30]);
        // Define the lore lines as static
        let lines: Vec<&'static str> = vec![
            "================ < LORE ARCHIVE > ================",
            "",
            ":: FILE: DNK-34 // PROTOCOL STATUS: ACTIVE        ",
            "",
            "==================================================",
            "",
            "Beneath the stone of the Northern Reach ",
            "lies the forgotten fortress-vault DNK-34.",
            "Forged by Iron-Priests in an age of steam and rune",
            "its halls are now ruled by rust-caked automatons",
            "following the last order ever issued:",
            "              **DENY ALL INTRUDERS**              ",
            "Within lie arsenals of brass lightning,",
            "tomes of forbidden craft, and the reactor-heart",
            "whose ash still powers the iron guardians...",
            "and as all others before them has failed..",
            "fallen to the horrors of what lies beyond..",
            "a new soul rises up to the challenge..",
            "in a attempt to uncover the secrets of the vault.",
            "",
            "a Hero only known as:",
        ];

        // Print to terminal
        for (i, line) in lines.iter().enumerate() {
            term.put_string([0, (i + 2) as i32], line.fg(color::LIGHT_GRAY));
        }

        let name_display = format!("{}", character_name.0);
        term.put_string([21, 22], name_display.fg(color::GREEN));

        term.put_string([0, 25], ">>        Press [ENTER] Breach the vault        <<".fg(color::GREEN));

        // --- Save LORE to MongoDB as JSON ---

        let lore_entry = crate::dbs::mongodb::LoreEntry {
            id: None,
            character_name: character_name.0.clone(),
            world_lore_lines: lines.iter().map(|s| s.to_string()).collect(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let db = lore_db.clone();

        let task_pool = AsyncComputeTaskPool::get();

        let fut = async move {
            db
                .create_lore_entry(lore_entry)
                .await
                .map_err(|e| anyhow::Error::new(e))
        };
        // This ensures the Mongo call runs in a Tokio reactor, not Bevy’s CPU thread:
        tokio_handle.0.spawn(fut);


    }else {
        warn!("Global terminal not found ENTER LORE");
    }
}

pub fn poll_lore_save_task(
    mut commands: Commands,
    mut task_res: Option<ResMut<SaveLoreTask>>,
) {
    if let Some(mut task_res) = task_res {
        if let Some(result) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task_res.0)) {
            match result {
                Ok(_) => info!("Lore saved successfully."),
                Err(e) => error!("Failed to save lore: {:?}", e),
            }
            // Fjern resource når task er færdig
            commands.remove_resource::<SaveLoreTask>();
        }
    }
}


// Handle input in the Lore state
pub fn lore_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::GeneratingWeapon);
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
