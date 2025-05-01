use bevy::prelude::*;
use sark_grids::Grid;
use sark_pathfinding::*;

use crate::{
    map::{Map, MapTile},
    movement::Position,
};

pub const UPDATE_MAP_STATE_SYSTEM_LABEL: &str = "update_map_state_system";

pub struct MapStatePlugin;

impl Plugin for MapStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_map_state_system
                .label(UPDATE_MAP_STATE_SYSTEM_LABEL),
        )
            .init_resource::<MapObstacles>()
            .init_resource::<MapActors>();
    }
}

/// An entity that blocks pathfinding.
#[derive(Component, Default)]
pub struct PathBlocker;

pub struct MapObstacles(pub PathMap2d);

impl Default for MapObstacles {
    fn default() -> Self {
        Self(PathMap2d::new([0, 0]))
    }
}

#[derive(Component, Default)]
pub struct MapActors(pub Grid<Option<Entity>>);

fn get_pathmap_grid_mut(map: &mut PathMap2d) -> &mut Grid<bool> {
    unsafe { &mut *(map as *mut PathMap2d as *mut Grid<bool>) }
}

fn update_map_state_system(
    q_moved_actors: Query<&Position, (With<PathBlocker>, Changed<Position>)>,
    q_blockers: Query<(Entity, &Position), With<PathBlocker>>,
    q_changed_map: Query<&Map, Changed<Map>>,
    q_map: Query<&Map>,
    mut blockers: ResMut<MapObstacles>,
    mut entities: ResMut<MapActors>,
) {
    if q_moved_actors.is_empty() && q_changed_map.is_empty()
        && !blockers.is_changed() && !entities.is_changed()
    {
        return;
    }

    if let Ok(map) = q_map.get_single() {
        if UVec2::from_array(<[u32; 2]>::from(blockers.0.size())) != map.0.size() {
            blockers.0 = PathMap2d::new(map.0.size().to_array());
        }

        if entities.0.len() != map.0.len() {
            entities.0 = Grid::default(map.0.size());
        }

        for (i, tile) in map.0.iter().enumerate() {
            get_pathmap_grid_mut(&mut blockers.0)[i] = *tile == MapTile::Wall;
        }

        for entry in entities.0.iter_mut() {
            *entry = None;
        }

        for (entity, pos) in q_blockers.iter() {
            let i = map.0.pos_to_index(pos.0);
            get_pathmap_grid_mut(&mut blockers.0)[i] = true;
            entities.0[i] = Some(entity);
        }
    }
}


