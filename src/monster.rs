use bevy::prelude::*;
use bracket_random::prelude::{DiceType};
use sark_grids::Grid;
use sark_pathfinding::*;
use sark_pathfinding::PathMap2d;
use controlled_astar::{AStar, node::{Node, Direction}};
use bevy_ascii_terminal::color;

use astar;

use crate::{bundle::MovingEntityBundle, map_state::{
    PathBlocker,
    MapObstacles,
    MapActors
}, visibility::{
    MapView,
    VIEW_SYSTEM_LABEL,
    ViewRange
}, turn_system::{
    Energy,
    TakingATurn
}, combat::{
    CombatantBundle,
    HitPoints,
    MaxHitPoints,
    Defense, Strength,
    TargetEvent,
    ActorEffect, AttackDice
}, movement::Position, player::Player, rng::DiceRng, AppState};
use crate::visibility::ViewSystemSet;

pub struct MonstersPlugin;

impl Plugin for MonstersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, monster_ai.after(ViewSystemSet).run_if(in_state(AppState::Lore)));
    }
}

#[derive(Component, Default)]
pub struct Monster;

#[derive(Bundle)]
pub struct MonsterBundle {
    #[bundle()]
    pub movable: MovingEntityBundle,
    #[bundle()]
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
            movable: MovingEntityBundle::new(Color::from(color::RED), 'g', 20),
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
            movable: MovingEntityBundle::new(Color::from(color::RED), 'o', 15),
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
                // Convert the grid to a HashMap of Nodes for AStar
                let grid = get_pathmap_grid_mut(&mut obstacles.0);
                let grid_vec = grid_bool_to_vec_vec_i32(grid);
                let nodes = Node::grid_to_nodes(&grid_vec);
                
                let mut astar = AStar::new(nodes);

                let start = (pos.x as usize, pos.y as usize);
                let goal = (player_pos.x as usize, player_pos.y as usize);
                if let Ok(Some(path)) = astar.find_shortest_path(start, goal) {
                    if path.len() == 2 {
                        let damage = rng.roll(dice.0);
                        attack_events.send(TargetEvent {
                            actor: entity,
                            target: player,
                            effect: ActorEffect::Damage(damage),
                        });
                    } else if path.len() > 1{
                        entities.0[*pos] = None;
                        *pos = IVec2::new(path[1].0 as i32, path[1].1 as i32);
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

fn grid_bool_to_vec_vec_i32(grid: &Grid<bool>) -> Vec<Vec<i32>> {
    let (width, height) = (grid.width(), grid.height());
    let mut result = vec![vec![0; width]; height];
    for y in 0..height {
        for x in 0..width {
            result[y][x] = if grid[IVec2::new(x as i32, y as i32)] { 1 } else { 0 };
        }
    }
    result
}

fn get_pathmap_grid_mut(map: &mut PathMap2d) -> &mut Grid<bool> {
    unsafe {
        // SAFELY cast PathMap2d to a Grid<bool> because PathMap2d wraps Grid<bool>
        &mut *(map as *mut PathMap2d as *mut Grid<bool>)
    }
}
