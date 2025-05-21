use bevy::prelude::*;

use bevy::input::keyboard::{KeyCode};
use bracket_random::prelude::DiceType;
use sark_grids::{Grid, SizedGrid};
use sark_pathfinding::PathMap2d;
use crate::{bundle::MovingEntityBundle, map_state::{MapActors, MapObstacles}, monster::Monster, movement::{Movement, Position}, visibility::{MapMemory, MapView, ViewRange}, events::AttackEvent, turn_system::{TakingATurn, Energy}, combat::{CombatantBundle, HitPoints, MaxHitPoints, Defense, Strength, TargetEvent, ActorEffect, AttackDice}, rng::DiceRng, AppState};

pub struct PlayerPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PlayerSpawnSet;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerSpawned>()
            .add_systems(OnEnter(AppState::WeaponSetup), spawn_player.in_set(PlayerSpawnSet))
            .add_systems(Update, player_input.run_if(in_state(AppState::InGame)));

    }
}

#[derive(Resource, Default)]
struct PlayerSpawned(bool);

fn spawn_player(
    mut commands: Commands,
    mut spawned: ResMut<PlayerSpawned>,
    existing_player: Query<(), With<Player>>,
) {
    if spawned.0 || existing_player.single().is_ok() {
        return;
    }

    commands.spawn(PlayerBundle::default());
    spawned.0 = true;
}

#[derive(Component, Default, Debug)]
pub struct Player;

#[derive(Debug, Bundle)]
pub struct PlayerBundle {
    #[bundle()]
    pub move_bundle: MovingEntityBundle,
    #[bundle()]
    pub combatant_bundle: CombatantBundle,
    pub player: Player,
    pub view: MapView,
    pub name: Name,
    pub memory: MapMemory,
    pub view_range: ViewRange,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            move_bundle: MovingEntityBundle::new(Color::WHITE, '@', 25),
            combatant_bundle: CombatantBundle {
                hp: HitPoints(60),
                max_hp: MaxHitPoints(60),
                defense: Defense(1),
                strength: Strength(3),
                attack_dice: AttackDice(DiceType::new(5,3,0)),
            },
            player: Default::default(),
            view: Default::default(),
            name: Name::new("Player"),
            memory: Default::default(),
            view_range: ViewRange(5),

        }
    }
}
fn is_in_bounds(grid: &Grid<bool>, pos: IVec2) -> bool {
    pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < grid.width() && (pos.y as usize) < grid.height()
}
fn player_input(
    mut q_player: Query<(Entity, &Strength, &mut Position, &mut Energy, &AttackDice, &mut Movement), (With<Player>, With<TakingATurn>)>,
    q_monsters: Query<&Name, With<Monster>>,
    input: Res<ButtonInput<KeyCode>>,
    mut obstacles: ResMut<MapObstacles>,
    mut actors: ResMut<MapActors>,
    _event_attack: EventWriter<AttackEvent>,
    mut evt_attack: EventWriter<TargetEvent>,
    mut rng: Local<DiceRng>,
) {
    if let Ok((entity, _attack, mut pos, mut energy, dice, mut movement)) = q_player.single_mut() {
        if read_wait(&input) {
            energy.0 = 0;
            return;
        }

        let move_input = read_movement(&input);
        if move_input.cmpeq(IVec2::ZERO).all() {
            return;
        }

        let curr = IVec2::from(pos.0);
        let next = curr + move_input;
        let attack = rng.roll(dice.0);

        // Access the grid inside PathMap2d
        let grid = get_pathmap_grid_mut(&mut obstacles.0);
        if !is_in_bounds(grid, next) {
            return;
        }

        if grid[next] {
            if let Some(target) = actors.0[next] {
                if let Ok(_name) = q_monsters.get(target) {
                    evt_attack.send(TargetEvent {
                        actor: entity,
                        target,
                        effect: ActorEffect::Damage(attack),
                    });

                    energy.0 = 0;
                }
            }
            return;
        }

        pos.0 = next.into();
        energy.0 = 0;
        actors.0[curr] = None;
        actors.0[next] = Some(entity);
        grid[curr] = false;
        grid[next] = true;
        movement.0 = move_input.into();
    }
}

fn get_pathmap_grid_mut(map: &mut PathMap2d) -> &mut Grid<bool> {
    unsafe {
        // SAFELY cast PathMap2d to a Grid<bool> because PathMap2d wraps Grid<bool>
        &mut *(map as *mut PathMap2d as *mut Grid<bool>)
    }
}

fn read_movement(input: &ButtonInput<KeyCode>) -> IVec2 {
    let mut p = IVec2::ZERO;

    if input.just_pressed(KeyCode::Numpad1) || input.just_pressed(KeyCode::KeyZ) {
        p.x = -1;
        p.y = -1;
    }
    if input.just_pressed(KeyCode::Numpad2) || input.just_pressed(KeyCode::KeyX) || input.just_pressed(KeyCode::ArrowDown) {
        p.y = -1;
    }
    if input.just_pressed(KeyCode::Numpad3) || input.just_pressed(KeyCode::KeyC) {
        p.x = 1;
        p.y = -1;
    }
    if input.just_pressed(KeyCode::Numpad4) || input.just_pressed(KeyCode::KeyA) || input.just_pressed(KeyCode::ArrowLeft) {
        p.x = -1;
    }
    if input.just_pressed(KeyCode::Numpad6) || input.just_pressed(KeyCode::KeyD) || input.just_pressed(KeyCode::ArrowRight) {
        p.x = 1;
    }
    if input.just_pressed(KeyCode::Numpad7) || input.just_pressed(KeyCode::KeyQ) {
        p.x = -1;
        p.y = 1;
    }
    if input.just_pressed(KeyCode::Numpad8) || input.just_pressed(KeyCode::KeyW) || input.just_pressed(KeyCode::ArrowUp) {
        p.y = 1;
    }
    if input.just_pressed(KeyCode::Numpad9) || input.just_pressed(KeyCode::KeyE) {
        p.x = 1;
        p.y = 1;
    }
    p
}

fn read_wait(input: &ButtonInput<KeyCode>) -> bool {
    input.just_pressed(KeyCode::Numpad5) || input.just_pressed(KeyCode::ControlLeft) || input.just_pressed(KeyCode::ControlRight)
}