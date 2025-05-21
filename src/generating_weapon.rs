use std::thread;
use bevy::app::AppExit;
use bevy::ecs::system::command::insert_resource;
use bevy::input::ButtonInput;
use crossbeam_channel::{unbounded, Receiver};
use bevy::prelude::{Commands, Events, KeyCode, NextState, Query, Res, ResMut, Resource, With};
use bevy_ascii_terminal::{color, StringDecorator, Terminal};
use crate::{AppState, GlobalTerminal};
use crate::combat::{Defense, HitPoints, MaxHitPoints, Strength};
use crate::dbs::{playerdb, weapondb};
use crate::dbs::playerdb::{Database, PlayerDb};
use crate::dbs::weapondb::WeaponDB;
use crate::main_menu::{CharacterName, PlayerSaved};
use crate::player::Player;
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit: ResMut<Events<AppExit>>,
    mut commands: Commands,
) {
    if let Some(rx) = receiver.0.as_ref() {
        if let Ok(result) = rx.try_recv() {
            match result {
                Ok(weapon) => {
                    commands.insert_resource(GeneratedWeapon(weapon));
                    next_state.set(AppState::WeaponSetup);
                }
                Err(err) => {
                    if let Ok(mut term) = term_q.single_mut() {
                        term.clear();
                        println!("GENERATION FAILED ==> Error: {}", err);
                        term.put_string([0, 10], format!("Generation failed: {}", err).fg(color::RED));
                        term.put_string([0, 15], "              Press [Enter] to Quit.             ".fg(color::RED));
                    }
                    if keyboard.just_pressed(KeyCode::Enter) {
                        exit.send(AppExit::Success);
                    }
                }
            }

            // Only clear the receiver once the result is handled
            receiver.0 = None;
        }
    }
}




pub fn weapon_continue_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::InGame);
    }
}

#[derive(Resource)]
pub struct GeneratedWeapon(pub Weapon);

#[derive(Resource)]
pub struct SavedWeapon(pub bool);

pub fn display_weapon_info(
    weapon: Res<GeneratedWeapon>,
    mut term_q: Query<&mut Terminal, With<GlobalTerminal>>,
) {
    if let Ok(mut term) = term_q.single_mut() {
        term.clear();
        term.resize([50, 30]);

        term.put_string([0, 3], "GENERATED WEAPON".fg(color::YELLOW));
        term.put_string([0, 5], format!("{}", weapon.0.name).fg(color::GREEN));
        term.put_string([0, 6], format!("{}", weapon.0.damage).fg(color::GREEN));
        term.put_string([0, 7], format!("{}", weapon.0.weight).fg(color::GREEN));
        term.put_string([0, 8], format!("{}", weapon.0.upgrade).fg(color::GREEN));
        term.put_string([0, 9], format!("{}", weapon.0.perk).fg(color::GREEN));
        term.put_string([0, 10], format!("{}", weapon.0.weapon_type).fg(color::GREEN));
        term.put_string([0, 11], format!("{:?}", weapon.0.predicted_price).fg(color::GREEN));

        term.put_string([0, 25], ">> Press [ENTER] to breach the vault <<".fg(color::GREEN));
    }
}
pub fn save_weapon_after_creation(
    mut saved: ResMut<SavedWeapon>,
    generated_weapon: Res<GeneratedWeapon>,
    psql: Res<Database>,
) {
    if saved.0 {
        return; // Already saved
    }

    let weapon = &generated_weapon.0;

    saved.0 = true; // Mark as saved

    let record = WeaponDB {
        id: Default::default(),
        name: weapon.name.clone(),
        damage: weapon.damage,
        weight: weapon.weight,
        upgrade: weapon.upgrade.clone(),
        perk: weapon.perk.clone(),
        weapon_type: weapon.weapon_type.clone(),
        predicted_price: weapon.predicted_price,
    };

    let db_client = psql.client.clone();

    bevy::tasks::IoTaskPool::get().spawn(async move {
        match WeaponDB::create_weapon(
            db_client,
            &record.name,
            record.damage,
            record.weight,
            &record.upgrade,
            &record.perk,
            &record.weapon_type,
            record.predicted_price,
        ).await {
            Ok(saved_weapon) => println!("!!! Saved weapon to DB: {:?}", saved_weapon),
            Err(e) => eprintln!(">>X<< Failed to save weapon: {}", e),
        }
    }).detach();
}

