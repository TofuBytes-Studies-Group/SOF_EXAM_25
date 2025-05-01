use bevy::prelude::*;
use bracket_random::prelude::{DiceType};
use sark_grids::Grid;
use sark_pathfinding::*;
use sark_pathfinding::PathMap2d;
use sark_pathfinding::AStar;
use sark_pathfinding::PathingMap;
use astar;

use crate::{
    bundle::MovingEntityBundle, map_state::{
        PathBlocker,
        MapObstacles,
        MapActors
    },
    visibility::{
        MapView,
        VIEW_SYSTEM_LABEL,
        ViewRange
    },
    turn_system::{
        Energy,
        TakingATurn
    },
    combat::{
        CombatantBundle,
        HitPoints,
        MaxHitPoints,
        Defense, Strength,
        TargetEvent,
        ActorEffect, AttackDice
    }, movement::Position, player::Player, rng::DiceRng};

pub struct MonstersPlugin;

impl Plugin for MonstersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(monster_ai.after(VIEW_SYSTEM_LABEL));
    }
}

#[derive(Component, Default)]
pub struct Monster;

#[derive(Bundle)]
pub struct MonsterBundle {
    #[bundle]
    pub movable: MovingEntityBundle,
    #[bundle]
    pub combatant_bundle: CombatantBundle,
    pub monster: Monster,
    pub name: Name,
    pub blocker: PathBlocker,
    pub vision: MapView,
    pub view_range: ViewRange,
}

impl MonsterBundle {
    pub fn new_goblin() -> Self {
        MonsterBundle {
            movable: MovingEntityBundle::new(Color::RED, 'g', 20),
            combatant_bundle: CombatantBundle {
                hp: HitPoints(15),
                max_hp: MaxHitPoints(15),
                defense: Defense(0),
                strength: Strength(1),
                attack_dice: AttackDice(DiceType::new(1,4,0)),
            },
            monster: Default::default(),
            name: Name::new("Goblin"),
            blocker: Default::default(),
            vision: Default::default(),
            view_range: ViewRange(4),
        }
    }

    pub fn new_orc() -> Self {
        Self {
            movable: MovingEntityBundle::new(Color::RED, 'o', 15),
            combatant_bundle: CombatantBundle {
                hp: HitPoints(25),
                max_hp: MaxHitPoints(25),
                defense: Defense(1),
                strength: Strength(3),
                attack_dice: AttackDice(DiceType::new(2,6,0)),
            },
            monster: Default::default(),
            name: Name::new("Orc"),
            blocker: Default::default(),
            vision: Default::default(),
            view_range: ViewRange(4),
        }
    }

    pub fn get_from_index(index: u32) -> MonsterBundle {
        match index {
            0 => MonsterBundle::new_goblin(),
            1 => MonsterBundle::new_orc(),
            _ => MonsterBundle::new_goblin(),
        }
    }

    pub fn max_index() -> u32 {
        2
    }
}

fn monster_ai(
    mut obstacles: ResMut<MapObstacles>,
    mut entities: ResMut<MapActors>,
    q_player: Query<(Entity, &Position), With<Player>>,
    mut q_monster: Query<(Entity, &mut Position, &mut Energy, &AttackDice, &MapView, &Name), (With<Monster>, Without<Player>, With<TakingATurn>)>,
    mut attack_events: EventWriter<TargetEvent>,
    mut rng: Local<DiceRng>,
) {
    for (entity, mut pos, mut energy, dice, view, _name) in q_monster.iter_mut() {
        let pos = &mut pos.0;

        if let Ok((player, player_pos)) = q_player.get_single() {
            let player_pos = player_pos.0;

            if view.0[player_pos] {
                {
                    // Access the grid inside PathMap2d in a separate scope
                    let grid = get_pathmap_grid_mut(&mut obstacles.0);
                    grid[*pos] = false;    // Remove the obstacle at monster's position
                    grid[player_pos] = false;  // Remove the obstacle at player's position
                }

                let mut astar = AStar::new(5);
                if let Some(path) = astar.find_path(&obstacles.0, <[i32; 2]>::from(*pos), <[i32; 2]>::from(player_pos)) {
                    if path.len() == 2 {
                        let damage = rng.roll(dice.0);
                        attack_events.send(TargetEvent {
                            actor: entity,
                            target: player,
                            effect: ActorEffect::Damage(damage),
                        });
                    } else {
                        entities.0[*pos] = None;
                        *pos = IVec2::from(path[1]);
                        entities.0[*pos] = Some(entity);
                    }
                }

                {
                    // Access the grid again to restore obstacles
                    let grid = get_pathmap_grid_mut(&mut obstacles.0);
                    grid[*pos] = true;    // Restore the obstacle at monster's position
                    grid[player_pos] = true;  // Restore the obstacle at player's position
                }
            }
        }

        energy.0 = 0;
    }
}

fn get_pathmap_grid_mut(map: &mut PathMap2d) -> &mut Grid<bool> {
    unsafe {
        // SAFELY cast PathMap2d to a Grid<bool> because PathMap2d wraps Grid<bool>
        &mut *(map as *mut PathMap2d as *mut Grid<bool>)
    }
}
