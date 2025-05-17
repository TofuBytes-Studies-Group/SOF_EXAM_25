use sark_pathfinding::PathMap2d;
use sark_grids::Grid;

pub trait PathMap2dExt {
    fn grid_mut(&mut self) -> &mut Grid<bool>;
    fn grid(&self) -> &Grid<bool>;
}

impl PathMap2dExt for PathMap2d {
    fn grid_mut(&mut self) -> &mut Grid<bool> {
        // SAFETY: This relies on PathMap2d being a thin wrapper over Grid<bool>
        unsafe { &mut *(self as *mut PathMap2d as *mut Grid<bool>) }
    }

    fn grid(&self) -> &Grid<bool> {
        unsafe { &*(self as *const PathMap2d as *const Grid<bool>) }
    }
}
