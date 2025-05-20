use bevy::prelude::*;
        use sark_grids::{Grid, SizedGrid};
        use crate::{map::{Map, MapTile}, movement::Position, AppState};
        
        use adam_fov_rs::{self, compute_fov, GridPoint};
        
        pub const VIEW_SYSTEM_LABEL: &str = "VIEW_SYSTEM";
        
        #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
        pub struct ViewSystemSet;
        
        pub struct VisibilityPlugin;
        
        impl Plugin for VisibilityPlugin {
            fn build(&self, app: &mut App) {
                app.configure_sets(Update, ViewSystemSet.run_if(in_state(AppState::Lore)))
                    .add_systems(Update, view_system.in_set(ViewSystemSet).run_if(in_state(AppState::Lore)))
                    .add_systems(Update, view_memory_system.after(ViewSystemSet).run_if(in_state(AppState::Lore)));
            }
        }
        
        #[derive(Component, Debug, Default)]
        pub struct MapMemory(pub Vec<bool>);
        
        #[derive(Component, Debug, Default)]
        pub struct MapView(pub Grid<bool>);
        
        #[derive(Component, Debug, Default)]
        pub struct ViewRange(pub u32);
        
        #[allow(clippy::type_complexity)]
        fn view_system(
            mut q_view: Query<(&mut MapView, &Position, &ViewRange), (Changed<Position>, Without<MapMemory>)>,
            q_map: Query<&Map>,
        ) {
            if let Ok(map) = q_map.single() {
                let map_size = map.0.size();
                let grid_size = [map_size.x, map_size.y];
        
                for (mut view, pos, range) in q_view.iter_mut() {
                    if view.0.size() != map_size {
                        view.0 = Grid::new(map_size);
                    }
        
                    for cell in view.0.iter_mut() {
                        *cell = false;
                    }
        
                    compute_fov(
                        pos.0,
                        range.0 as usize,
                        grid_size,
                        |p| !map.0.in_bounds(p) || map.0[p] == MapTile::Wall,
                        |p| {
                            if map.0.in_bounds(p) {
                                view.0[p] = true;
                            }
                        },
                    );
                }
            }
        }
        
        fn view_memory_system(
            mut q_view: Query<(&mut MapView, &mut MapMemory, &Position, &ViewRange), Changed<Position>>,
            q_map: Query<&Map>,
        ) {
            if let Ok(map) = q_map.single() {
                let map_size = map.0.size();
                let grid_size = [map_size.x, map_size.y];
        
                for (mut view, mut memory, pos, range) in q_view.iter_mut() {
                    if view.0.size() != map_size {
                        view.0 = Grid::new(map_size);
                    }
        
                    for cell in view.0.iter_mut() {
                        *cell = false;
                    }
        
                    let total_tiles = map.0.tile_count();
                    if memory.0.len() != total_tiles {
                        memory.0 = vec![false; total_tiles];
                    }
        
                    compute_fov(
                        pos.0,
                        range.0 as usize,
                        grid_size,
                        |p| !map.0.in_bounds(p) || map.0[p] == MapTile::Wall,
                        |p| {
                            if map.0.in_bounds(p) {
                                let i = map.0.transform_lti(p);
                                view.0[p] = true;
                                memory.0[i] = true;
                            }
                        },
                    );
                }
            }
        }