use std::thread;
use std::time::Duration;
use bevy::app::AppExit;
use bevy::ecs::system::command::insert_resource;
use bevy::input::ButtonInput;
use crossbeam_channel::{unbounded, Receiver};
use bevy::prelude::{Commands, Events, KeyCode, NextState, Query, Res, ResMut, Resource, Time, Timer, TimerMode, With};
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
#[derive(Resource)]
pub struct LoadingBarTimer(Timer);
#[derive(Resource, Default)]
pub struct WeaponGenerationProgress(pub u8);

pub(crate) fn setup_loading_bar(mut commands: Commands) {
    commands.insert_resource(WeaponGenerationProgress(0));
    commands.insert_resource(LoadingBarTimer(Timer::from_seconds(0.6, TimerMode::Repeating)));
}
pub(crate) fn update_loading_bar(
    time: Res<Time>,
    mut timer: ResMut<LoadingBarTimer>,
    mut progress: ResMut<WeaponGenerationProgress>,
) {
    if timer.0.tick(time.delta()).just_finished() && progress.0 < 100 {
        progress.0 += 1;
    }
}
pub(crate) fn show_loading_screen(
    mut term_q: Query<&mut Terminal, With<GlobalTerminal>>,
    progress: Res<WeaponGenerationProgress>,
) {
    if let Ok(mut term) = term_q.single_mut() {
        term.clear();
        term.put_string([0, 12], "            >> Generating weapon... <<            ".fg(color::YELLOW));

        let bar_len = 30;
        let filled = (progress.0 as f32 / 100.0 * bar_len as f32).round() as usize;
        let bar = format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            " ".repeat(bar_len - filled),
            progress.0
        );

        term.put_string([9, 14], bar.fg(color::GREEN));
    }
}
#[derive(Resource)]
pub struct PostGenDelayTimer(pub Timer);

pub fn poll_weapon_generation(
    mut receiver: ResMut<WeaponGenReceiver>,
    mut next_state: ResMut<NextState<AppState>>,
    mut term_q: Query<&mut Terminal, With<GlobalTerminal>>,
    mut progress: ResMut<WeaponGenerationProgress>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit: ResMut<Events<AppExit>>,
    mut commands: Commands,
) {
    if let Some(rx) = receiver.0.as_ref() {
        if let Ok(result) = rx.try_recv() {
            match result {
                Ok(weapon) => {
                    progress.0 = 100;
                    // Force loading bar to jump to 100 and update the terminal
                    if let Ok(mut term) = term_q.single_mut() {
                        term.clear();

                        // Force progress bar full
                        let bar_len = 30;
                        let bar = format!(
                            "   [{}{}] {}%",
                            "☻".repeat(bar_len),
                            "",
                            100
                        );
                        term.put_string([0, 11], "            >> GENERATION COMPLETE <<           ".fg(color::GREEN));
                        term.put_string([10, 15], bar.fg(color::GREEN));
                    }

                    // Insert weapon and move state AFTER the visual update
                    commands.insert_resource(GeneratedWeapon(weapon));
                    commands.insert_resource(SavedWeapon(false));

                    // Add 0.5-second delay before changing state
                    commands.insert_resource(PostGenDelayTimer(Timer::from_seconds(5.0, TimerMode::Once)));

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

pub fn wait_then_switch_state(
    time: Res<Time>,
    mut timer: Option<ResMut<PostGenDelayTimer>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    if let Some(mut t) = timer {
        if t.0.tick(time.delta()).finished() {
            next_state.set(AppState::WeaponSetup);
            commands.remove_resource::<PostGenDelayTimer>();
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
        term.put_string([0, 5], " Weapon Name:".fg(color::WHITE));
        term.put_string([0, 6], format!("  {}", weapon.0.name).fg(color::WHITE));
        term.put_string([0, 8], " Damage:".fg(color::WHITE));
        term.put_string([0, 9], format!("  {}", weapon.0.damage).fg(color::WHITE));
        term.put_string([0, 11], " Weight:".fg(color::WHITE));
        term.put_string([0, 12], format!("  {}", weapon.0.weight).fg(color::WHITE));
        term.put_string([0, 14], " Upgrade:".fg(color::WHITE));
        term.put_string([0, 15], format!("  {}", weapon.0.upgrade).fg(color::WHITE));
        term.put_string([0, 17], " Perk:".fg(color::WHITE));
        term.put_string([2, 18], format!("{}", weapon.0.perk).fg(color::WHITE));
        term.put_string([0, 21], " Weapon Type:".fg(color::WHITE));
        term.put_string([0, 22], format!("  {}", weapon.0.weapon_type).fg(color::WHITE));
        term.put_string([0, 24], " Predicted Price:".fg(color::WHITE));
        term.put_string([0, 25], format!("  {:?}", weapon.0.predicted_price).fg(color::WHITE));

        term.put_string([0, 29], "          [ENTER] to accept your weapon.          ".fg(color::WHITE));
    }
}
pub fn save_weapon_after_creation(
    mut saved: ResMut<SavedWeapon>,
    generated_weapon: Res<GeneratedWeapon>,
    psql: Res<Database>,
) {

    if saved.0 {
        return; // if weapon is saved already, do nothing
    }

    let weapon = &generated_weapon.0; //if it is not saved, set weapon to the generated one

    saved.0 = true; // and then mark it as saved

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

