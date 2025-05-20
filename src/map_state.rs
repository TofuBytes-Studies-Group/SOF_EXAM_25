use bevy::prelude::*;
use sark_grids::Grid;
use sark_pathfinding::*;

use crate::{map::{Map, MapTile}, movement::Position, AppState};
use crate::PathMap2dExt::PathMap2dExt;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct UpdateMapStateSet;
pub struct MapStatePlugin;

impl Plugin for MapStatePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, UpdateMapStateSet.run_if(in_state(AppState::Lore)))
            .add_systems(Update, update_map_state_system.in_set(UpdateMapStateSet))
            .init_resource::<MapObstacles>()
            .init_resource::<MapActors>();
    }
}

/// An entity that blocks pathfinding.
#[derive(Component, Default)]
pub struct PathBlocker;

#[derive(Resource)]
pub struct MapObstacles(pub PathMap2d);

impl Default for MapObstacles {
    fn default() -> Self {
        Self(PathMap2d::new([0, 0]))
    }
}

#[derive(Resource)]
pub struct MapActors(pub Grid<Option<Entity>>);

impl Default for MapActors {
        fn default() -> Self {
            Self(Grid::new([0, 0]))
        }
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

    if let Ok(map) = q_map.single() {
        if UVec2::from_array(<[u32; 2]>::from(blockers.0.size())) != map.0.size() {
            blockers.0 = PathMap2d::new(map.0.size().to_array());
        }

        if entities.0.width() * entities.0.height() != map.0.tile_count() {
            entities.0 = Grid::new(map.0.size());
        }

        let grid = blockers.0.grid_mut();
        let grid_len = grid.width() * grid.height();

        for (i, tile) in map.0.iter().enumerate() {
            if i < grid_len {
                grid[i] = *tile == MapTile::Wall;
            }
        }


        for entry in entities.0.iter_mut() {
            *entry = None;
        }

        let entities_len = entities.0.width() * entities.0.height();

        for (entity, pos) in q_blockers.iter() {
            let i = map.0.transform_lti(pos.0);
            if i < grid_len && i < entities_len {
                blockers.0.grid_mut()[i] = true;
                entities.0[i] = Some(entity);
            } else {
                // Optionally log or warn about the out-of-bounds index
                warn!("Entity position {:?} out of bounds for map size {:?}", pos.0, map.0.size());
            }
        }
    }
}


