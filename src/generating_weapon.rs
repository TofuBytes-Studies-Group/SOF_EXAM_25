use std::thread;
use crossbeam_channel::{unbounded, Receiver};
use bevy::prelude::{Commands, NextState, Query, Res, ResMut, Resource, With};
use bevy_ascii_terminal::{color, StringDecorator, Terminal};
use crate::{AppState, GlobalTerminal};
use crate::main_menu::CharacterName;
use crate::weapon_prediction::bridge::{generate_weapon, Weapon};

#[derive(Resource)]
pub struct WeaponGenReceiver(pub Option<Receiver<Result<Weapon, String>>>);

pub fn start_weapon_generation(
    mut commands: Commands,
    character_name: Res<CharacterName>,
) {
    let (tx, rx) = unbounded();

    let name = character_name.0.clone();
    thread::spawn(move || {
        let result = generate_weapon(&name)
            .map_err(|e| format!("{:?}", e));
        tx.send(result).ok(); // Ignore send errors
    });

    commands.insert_resource(WeaponGenReceiver(Some(rx)));
}
pub fn show_loading_screen(mut term_q: Query<&mut Terminal, With<GlobalTerminal>>) {
    if let Ok(mut term) = term_q.single_mut() {
        term.clear();
        term.put_string([15, 14], ">> Generating weapon... <<".fg(color::YELLOW));
    }
}

pub fn poll_weapon_generation(
    mut receiver: ResMut<WeaponGenReceiver>,
    mut next_state: ResMut<NextState<AppState>>,
    mut term_q: Query<&mut Terminal, With<GlobalTerminal>>,
) {
    if let Some(rx) = receiver.0.as_ref() {
        if let Ok(result) = rx.try_recv() {
            match result {
                Ok(weapon) => {
                    if let Ok(mut term) = term_q.single_mut() {
                        term.clear();
                        term.resize([50, 30]);

                        term.put_string([0, 13], "GENERATED WEAPON".fg(color::LIGHT_GRAY));
                        let w_name = format!("{}", weapon.name);
                        term.put_string([24, 14], w_name.fg(color::GREEN));

                        let w_dam = format!("{}", weapon.damage);
                        term.put_string([24, 15], w_dam.fg(color::GREEN));

                        let w_weight = format!("{}", weapon.weight);
                        term.put_string([24, 16], w_weight.fg(color::GREEN));

                        let w_upgrade = format!("{}", weapon.upgrade);
                        term.put_string([24, 17], w_upgrade.fg(color::GREEN));

                        let w_perk = format!("{}", weapon.perk);
                        term.put_string([24, 18], w_perk.fg(color::GREEN));

                        let w_type = format!("{}", weapon.weapon_type);
                        term.put_string([24, 19], w_type.fg(color::GREEN));

                        let w_price = format!("{:?}", weapon.predicted_price);
                        term.put_string([24, 20], w_price.fg(color::GREEN));

                        term.put_string([0, 25], ">>        Press [ENTER] Breach the vault        <<".fg(color::GREEN));                        // Add more display if needed
                    }
                    next_state.set(AppState::InGame);
                }
                Err(err) => {
                    if let Ok(mut term) = term_q.single_mut() {
                        term.clear();
                        term.resize([50, 30]);

                        term.put_string([10, 10], format!("Generation failed: {}", err).fg(color::RED));
                    }
                    next_state.set(AppState::MainMenu); // Or another fallback state
                }
            }
            receiver.0 = None;
        }
    }
}
