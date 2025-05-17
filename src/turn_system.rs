use bevy::prelude::*;
use crate::AppState;

pub struct TurnSystemPlugin;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TurnBeginSet;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TurnEndSet;

impl Plugin for TurnSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_sets(Update, TurnBeginSet.run_if(in_state(AppState::InGame)))
            .configure_sets(Update, TurnEndSet.after(TurnBeginSet).run_if(in_state(AppState::InGame)))
            .add_systems(Update, turn_begin_system.in_set(TurnBeginSet))
            .add_systems(Update, turn_end_system.in_set(TurnEndSet));
    }
}

/// When an actor's energy reaches or exceeds 100, it will be given a turn.
#[derive(Default, Debug, Component)]
pub struct Energy(pub i32);

/// Determines how frequently an actor gets to take their turn,
/// relative to other actors.
#[derive(Debug, Component)]
pub struct Speed(pub i32);

/// A tag for actors that can perform actions and take turns.
#[derive(Default, Debug, Component)]
pub struct Actor;

/// A component that gets added to an actor when it's time for it to take it's turn.
#[derive(Debug, Component)]
pub struct TakingATurn;

fn turn_begin_system(
    mut commands: Commands,
    mut q_waiting_actors: Query<(Entity, &mut Energy, &Speed), (With<Actor>, Without<TakingATurn>)>,
    q_acting_actors: Query<&Actor, (With<Energy>, With<Speed>, With<TakingATurn>)>,
) {
    if !q_acting_actors.is_empty() {
        return;
    }

    let mut done = false;
    while !done {
        for (entity, mut energy, speed) in q_waiting_actors.iter_mut() {
            energy.0 += speed.0;

            if energy.0 >= 100 {
                done = true;
                commands.entity(entity).insert(TakingATurn);
            }
        }
    }
}

fn turn_end_system(
    mut commands: Commands, 
    q_actors: Query<(Entity, &Energy), (With<Actor>, With<TakingATurn>)>,
) {
    for (entity, energy) in q_actors.iter() {
        if energy.0 < 100 {
            commands.entity(entity).remove::<TakingATurn>();
        }
    }
}