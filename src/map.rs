use bevy::{
    math::IVec2,
    prelude::*,
};
use std::collections::HashSet;

use rand::{prelude::StdRng, Rng, SeedableRng};
use sark_grids::{Grid, SizedGrid};

use crate::{config::MapGenSettings, monster::MonsterBundle, movement::Position, player::Player, shapes::Rect, AppState, GAME_SIZE};
use crate::player::PlayerSpawnSet;
use crate::visibility::{MapMemory, MapView};

pub struct MapGenPlugin;

pub const MAP_GEN_SETUP_LABEL: &str = "MAP_GEN_SETUP";
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MapGenSetupSet;
impl Plugin for MapGenPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(OnEnter(AppState::InGame), MapGenSetupSet.after(PlayerSpawnSet))
            .add_systems(OnEnter(AppState::InGame), setup.in_set(MapGenSetupSet));
    }
}

fn setup(
    mut commands: Commands,
    q_player: Query<(Entity,&Player)>,
) {
  // Gen map
    // let mut settings = match config::try_get_map_settings() {
    //     Ok(settings) => settings,
    //     Err(e) => panic!("{}", e),
    // };

    let mut settings = MapGenSettings::default();
    settings.map_size = GAME_SIZE;

    //settings.map_size;

    //let rng = StdRng::seed_from_u64(settings.seed);
    let mut rng = StdRng::seed_from_u64(settings.seed);

    let player = q_player.get_single().map_or_else(|_|None,|(e,_)|Some(e));
    let entities = MapGenEntities {
        player,
    };

    MapGenerator::build(&mut commands, settings, rng, entities);
}
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}
impl Map {
    pub fn side_index(&self, side: Side) -> u32 {
        match side {
            Side::Left => 0,
            Side::Top => 0,
            Side::Right => (self.0.width() - 1) as u32,
            Side::Bottom => (self.0.height() - 1) as u32,
        }
    }
}


/// A tile on the [Map].
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum MapTile {
    Wall,
    Floor,
}

impl Default for MapTile {
    fn default() -> Self {
        Self::Wall
    }
}

#[derive(Component)]
pub struct Map(pub Grid<MapTile>);

pub struct MapGenEntities {
    pub player: Option<Entity>,
    //pub monsters: Vec<MonsterBundle>,
}

pub struct MapGenerator {
    pub map: Map,
    pub rooms: Vec<Rect>,
}

impl MapGenerator {
    pub fn build(
        commands: &mut Commands,
        settings: MapGenSettings,
        mut rng: StdRng,
        entities: MapGenEntities,
    ) {
        let mut map = Map(Grid::new(settings.map_size));
        let mut rooms: Vec<Rect> = Vec::with_capacity(50);

        generate_rooms(&mut map, &settings, &mut rng, &mut rooms);

        let map = MapGenerator { map, rooms };

        if let Some(player) = entities.player {
            map.place_player(commands, player);
        } else {
            println!("No player found");
        }

        let mut placed: HashSet<IVec2> = HashSet::default();

        map.place_monsters(commands, &settings, &mut rng, &mut placed);

        commands.spawn(map.map);
    }

    pub fn place_player(&self, commands: &mut Commands, player: Entity) {
        let p = self.rooms[0].center();
        let mut entity = commands.entity(player);

        // Set the player's position
        entity.insert(Position::from(p));
        
        let size = self.map.0.size();
        let dims = size.to_array();
        let len = (size.x * size.y) as usize;
        
        commands.entity(player)
            .insert(MapView ( Grid::new(dims) ))
            .insert(MapMemory ( vec![false; len] ));
        
        println!("Setting player position to {}", p);
        println!("Player FOV & Memory initialized to {}×{} ({} cells)", size.x, size.y, len);
    }

    pub fn place_monsters(
        &self,
        commands: &mut Commands,
        settings: &MapGenSettings,
        rng: &mut StdRng,
        placed: &mut HashSet<IVec2>,
    ) {
        // The first room is the player's room
        for room in self.rooms.iter().skip(1) {
            let count = rng.gen_range(settings.monsters_per_room.clone());

            for _ in 0..=count {
                for _ in 0..2 {
                    // If the first try fails, try again
                    let p = get_random_ivec(rng, room.min, room.max);

                    if placed.contains(&p) {
                        continue;
                    }

                    let monster_index = rng.gen_range(0..MonsterBundle::max_index());
                    let mut monster = MonsterBundle::get_from_index(monster_index);
                    monster.movable.position = p.into();
                    placed.insert(p);

                    commands.spawn(monster);

                    break;
                }
            }
        }
    }
}

fn get_random_ivec(rng: &mut StdRng, min: IVec2, max: IVec2) -> IVec2 {
    let p_x = rng.gen_range(min.x..max.x);
    let p_y = rng.gen_range(min.y..max.y);

    IVec2::new(p_x, p_y)
}

fn generate_rooms(
    map: &mut Map,
    settings: &MapGenSettings,
    rng: &mut StdRng,
    rooms: &mut Vec<Rect>,
) {
    // create a guaranteed starting room
    let smallest_u = settings.room_size.start;
    let smallest = smallest_u as i32;

    let map_width  = map.0.width() as i32;
    let map_height = map.0.height() as i32;
    let center_x   = map_width  / 2;
    let center_y   = map_height / 2;
    
    let first_room = Rect::from_position_size(
        (center_x, center_y),
        (smallest, smallest)
    );
    build_room(map, &first_room);
    rooms.push(first_room);
    
    for _ in 0..settings.iterations {
        let w = rng.random_range(settings.room_size.clone());
        let h = rng.random_range(settings.room_size.clone());

        let max_x = map.side_index(Side::Right).saturating_sub(w + 1);
        let max_y = map.side_index(Side::Top).saturating_sub(h + 1);

        if max_x <= 2 || max_y <= 2 {
            // Skip this iteration if the room won't fit
            continue;
        }

        let x = rng.random_range(2..=max_x);
        let y = rng.random_range(2..=max_y);

        let new_room = Rect::from_position_size((x as i32, y as i32), (w as i32, h as i32));

        // //println!("Creating room {}", new_room);

        let mut ok = true;

        for room in rooms.iter() {
            if new_room.overlaps(room) {
                //println!("New room overlaps {}!", room);
                ok = false;
                break;
            }
        }

        if ok {
            //println!("Building new room!");
            build_room(map, &new_room);

            if !rooms.is_empty() {
                let prev_room = &rooms[rooms.len() - 1];
                build_tunnels_between_rooms(map, rng, prev_room, &new_room);
            }

            rooms.push(new_room);
        }
    }
}

fn build_room(map: &mut Map, room: &Rect) {
    for pos in room.iter() {
        map.0[pos] = MapTile::Floor;
    }
}

fn build_tunnels_between_rooms(map: &mut Map, rng: &mut StdRng, room_a: &Rect, room_b: &Rect) {
    let (new_x, new_y) = room_b.center().into();
    let (prev_x, prev_y) = room_a.center().into();

    if rng.gen_bool(0.5) {
        build_horizontal_tunnel(map, prev_x, new_x, prev_y);
        build_vertical_tunnel(map, prev_y, new_y, new_x);
    } else {
        build_vertical_tunnel(map, prev_y, new_y, prev_x);
        build_horizontal_tunnel(map, prev_x, new_x, new_y);
    }
}

fn build_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    let min = x1.min(x2);
    let max = x1.max(x2);

    for x in min..=max {
        map.0[ [x as u32, y as u32] ] = MapTile::Floor;
    }
}

fn build_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    let min = y1.min(y2);
    let max = y1.max(y2);

    for y in min..=max {
        map.0[ [x as u32, y as u32] ] = MapTile::Floor;
    }
}
#[derive(Resource, Deref, DerefMut, Default)]
pub struct PathMap2d {
    grid: Grid<bool>,
}

impl PathMap2d {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid::new([width, height]),
        }
    }

    pub fn grid_mut(&mut self) -> &mut Grid<bool> {
        &mut self.grid
    }

    pub fn grid(&self) -> &Grid<bool> {
        &self.grid
    }

    pub fn size(&self) -> UVec2 {
        self.grid.size()
    }

    pub fn clear(&mut self) {
        for cell in self.grid.iter_mut() {
            *cell = false;
        }
    }
}
