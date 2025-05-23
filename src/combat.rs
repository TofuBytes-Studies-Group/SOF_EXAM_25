use bevy::prelude::*;
use bracket_random::prelude::DiceType;
use sark_grids::Grid;
use sark_pathfinding::PathMap2d;
use crate::{ui::PrintLog, map_state::{MapObstacles, MapActors}, movement::Position, AppState};
use bevy::app::PostUpdate;


pub struct CombatPlugin;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ResolveTargetEventsSet;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DeathSystemSet;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TargetEvent>()
            .add_event::<ActorKilledEvent>()
            .configure_sets(
                PostUpdate, 
                ResolveTargetEventsSet.run_if(in_state(AppState::InGame)),
            )
            .configure_sets(
                PostUpdate, 
                DeathSystemSet
                    .after(ResolveTargetEventsSet)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(PostUpdate, resolve_target_events.in_set(ResolveTargetEventsSet))
            .add_systems(PostUpdate, death_system.in_set(DeathSystemSet));
    }
}

#[derive(Debug, Component)]
pub struct MaxHitPoints(pub i32);

#[derive(Debug, Component)]
pub struct HitPoints(pub i32);

#[derive(Default, Debug,Component)]
pub struct Defense(pub i32);

#[derive(Default, Debug, Component)]
pub struct Strength(pub i32);

#[derive(Default, Debug, Component)]
pub struct AttackDice(pub DiceType);

#[derive(Debug, Bundle)]
pub struct CombatantBundle {
    pub hp: HitPoints,
    pub max_hp: MaxHitPoints,
    pub defense: Defense,
    pub strength: Strength,
    pub attack_dice: AttackDice,
}

pub enum ActorEffect {
    Heal(i32),
    Damage(i32),
}
#[derive(Event)]
pub struct TargetEvent {
    pub actor: Entity,
    pub target: Entity,
    pub effect: ActorEffect,
}

#[derive(Event)]
pub struct ActorKilledEvent {
    name: String,
}

fn resolve_target_events(
    q_names: Query<&Name>,
    q_attack: Query<&mut Strength>,
    mut q_defend: Query<(&mut HitPoints, &MaxHitPoints, &Defense)>,
    mut log: ResMut<PrintLog>,
    mut target_events: EventReader<TargetEvent>,
) {
    for ev in target_events.read() {
        let tar = ev.target;
        let actor = ev.actor;
        match ev.effect {
            ActorEffect::Heal(amount) => {
                if let Ok((mut hp, max, _)) = q_defend.get_mut(tar) {
                    let amount = i32::min(amount, max.0 - hp.0);
                    if amount <= 0 {
                        continue;
                    }
                    hp.0 += amount;
                                  
                    // TODO: Move this into ui? No reason to handle it here, would make it simpler + cleaner
                    if let Ok(actor_name) = q_names.get(actor) {
                        if let Ok(target_name) = q_names.get(tar) {
                            log.push(format!("{} heals {} for {} damage.", actor_name.as_str(), target_name.as_str(), amount));
                        }
                    }
                }
            },
            ActorEffect::Damage(amount) => {
                if let Ok(_attack) = q_attack.get(actor) {
                    if let Ok((mut hp, _, def)) = q_defend.get_mut(tar) {
                        let amount = amount - def.0;

                        if amount <= 0 {
                            continue;
                        }
                        hp.0 -= amount;

                    // TODO: Move this into ui? No reason to handle it here, would make it simpler + cleaner
                        if let Ok(actor_name) = q_names.get(actor) {
                            if let Ok(target_name) = q_names.get(tar) {

                                log.push(format!("{} attacks {} for {} damage.", actor_name.as_str(), target_name.as_str(), amount));
                            } 
                        } 
                    }
                }
            },
        };
    }
}

fn death_system(
    mut commands: Commands,
    mut log: ResMut<PrintLog>,
    mut obstacles: ResMut<MapObstacles>,
    mut blockers: ResMut<MapActors>,
    q_combatants: Query<(Entity, &HitPoints, &Position, &Name)>,
    mut evt_killed: EventWriter<ActorKilledEvent>,
) {
    for (entity, hp, pos, name) in q_combatants.iter() {
        if hp.0 <= 0 {
            commands.entity(entity).despawn();
            let pos = IVec2::from(pos.0).as_uvec2();

            // Access the grid inside PathMap2d
            let grid = get_pathmap_grid_mut(&mut obstacles.0);

            grid[pos] = false;
            blockers.0[pos] = None;

            evt_killed.write(ActorKilledEvent {
                name: name.to_string(),
            });

            // TODO: Move to UI
            log.push(format!("{} was killed!", name.as_str()));
        }
    }
}

fn get_pathmap_grid_mut(map: &mut PathMap2d) -> &mut Grid<bool> {
    unsafe {
        // SAFELY cast PathMap2d to a Grid<bool> because PathMap2d wraps Grid<bool>
        &mut *(map as *mut PathMap2d as *mut Grid<bool>)
    }
}