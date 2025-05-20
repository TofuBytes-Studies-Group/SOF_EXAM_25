
use bevy::prelude::Color;
use bevy_ascii_terminal::color::*;
use bevy::prelude::*;
use bevy_ascii_terminal::{terminal::Terminal, border::TerminalBorder, color, string::DecoratedString, StringDecorator, TerminalPlugins, Tile};
use sark_grids::SizedGrid;
use crate::{map::{Map, MapTile}, movement::Position, player::Player, visibility::{MapMemory, MapView}, GlobalTerminal, combat::ActorKilledEvent, AppState};
use crate::map::MapGenSetupSet;

pub const WALL_COLOR: Color = Color::srgb(0.866, 0.866, 0.882);
pub const FLOOR_COLOR: Color = Color::srgb(0.602, 0.462, 0.325);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RenderSystemSet;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PostMapSetupSet;
/// Plugin managing game rendering systems
pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_sets(OnEnter(AppState::InGame), PostMapSetupSet.after(MapGenSetupSet))
            .add_systems(OnEnter(AppState::InGame), first_frame_render.in_set(PostMapSetupSet));

        app
            .configure_sets(Update, RenderSystemSet.run_if(in_state(AppState::InGame)))
            .add_systems(Update, render.in_set(RenderSystemSet).run_if(should_render));
    }
}

fn first_frame_render(
    q_map: Query<&Map>,
    q_entities: Query<(&Renderable, &Position)>,
    q_player: Query<(Entity, &MapView), With<Player>>,
    q_memory: Query<&MapMemory>,
    mut q_term: Query<&mut Terminal, With<GlobalTerminal>>,
) {
    if let (Ok(mut term), Ok(map)) = (q_term.single_mut(), q_map.single()) {
        if term.size() != map.0.size() {
            term.resize(map.0.size());
        }
        term.clear();

        if let Ok((entity, player_view)) = q_player.single() {
            if let Ok(memory) = q_memory.get(entity) {
                render_memory(memory, map, &mut term);
            }
            render_view(player_view, &mut term, map, q_entities.iter());
        } else {
            render_everything(map, &mut term, q_entities.iter());
        }
    }
}

#[derive(Component, Debug)]
pub struct Renderable {
    pub fg_color: Color,
    pub bg_color: Color,
    pub glyph: char,
}

fn pos_to_index(pos: IVec2, width: u32) -> usize {
    (pos.y as usize * width as usize) + pos.x as usize
}

fn index_to_pos(index: usize, width: u32) -> IVec2 {
    IVec2::new((index % width as usize) as i32, (index / width as usize) as i32)
}

fn render(
    q_map: Query<&Map>,
    q_entities: Query<(&Renderable, &Position)>,
    q_player: Query<(Entity, &MapView), With<Player>>,
    q_memory: Query<&MapMemory>,
    mut q_render_terminal: Query<&mut Terminal, With<GlobalTerminal>>,
) {
    let mut term = match q_render_terminal.single_mut() {
        Ok(term) => term,
        Err(_) => return,
    };

    let map = match q_map.single() {
        Ok(term) => term,
        Err(_) => return,
    };

    if term.size() != map.0.size() {
        term.resize(map.0.size());
    }

    term.clear();

    if let Ok((entity, player_view)) = q_player.single() {
        if let Ok(memory) = q_memory.get(entity) {
            render_memory(memory, map, &mut term);
        }
        render_view(player_view, &mut term, map, q_entities.iter());
    } else {
        render_everything(map, &mut term, q_entities.iter());
    }
}

impl From<MapTile> for Tile {
    fn from(t: MapTile) -> Self {
        match t {
            MapTile::Wall => Tile {
                glyph: '#',
                fg_color: LinearRgba::from(WALL_COLOR),
                bg_color: LinearRgba::from(Color::BLACK),
            },
            MapTile::Floor => Tile {
                glyph: '.',
                fg_color: LinearRgba::from(FLOOR_COLOR),
                bg_color: LinearRgba::from(Color::BLACK),
            },
        }
    }
}

impl From<&Renderable> for Tile {
    fn from(r: &Renderable) -> Self {
        Tile {
            glyph: r.glyph,
            fg_color: LinearRgba::from(r.fg_color),
            bg_color: LinearRgba::from(r.bg_color),
        }
    }
}

fn render_view<'a, Actors>(view: &MapView, term: &mut Terminal, map: &Map, actors: Actors)
where
    Actors: Iterator<Item = (&'a Renderable, &'a Position)>,
{
    render_map_in_view(view, map, term);
    render_actors_in_view(view, map, term, actors);
}

fn render_map_in_view(view: &MapView, map: &Map, term: &mut Terminal) {
    for (i, seen) in view.0.iter().enumerate() {
        if *seen {
            let p = index_to_pos(i, map.0.width() as u32);
            let tile = map.0[p];
            term.put_tile(p, Tile::from(tile));
        }
    }
}

fn render_actors_in_view<'a, Actors>(view: &MapView, map: &Map, term: &mut Terminal, actors: Actors)
where
    Actors: Iterator<Item = (&'a Renderable, &'a Position)>,
{
    for (renderable, pos) in actors {
        let i = pos_to_index(pos.0, map.0.width() as u32);

        if view.0[i] {
            term.put_tile(pos.0, Tile::from(renderable));
        }
    }
}

fn render_memory(memory: &MapMemory, map: &Map, term: &mut Terminal) {
    for (i, remembered) in memory.0.iter().enumerate() {
        if *remembered {
            let p = index_to_pos(i, map.0.width() as u32);
            let tile = map.0[p];

            let mut tile: Tile = tile.into();
            tile.fg_color = LinearRgba::from(greyscale(Color::from(tile.fg_color)));

            term.put_tile(p, tile);
        }
    }
}

fn greyscale(c: Color) -> Color {
    if let Color::Srgba { .. } = c {
        let grey = 0.2126 * RED + 0.7152 * GREEN + 0.0722 * BLUE;
        Color::Srgba(Srgba::from(grey))
    } else {
        c // fallback: return the original color
    }
}

fn render_everything<'a, Actors>(map: &Map, term: &mut Terminal, actors: Actors)
where
    Actors: Iterator<Item = (&'a Renderable, &'a Position)>,
{
    render_full_map(map, term);
    render_all_entities(term, actors);
}

fn render_full_map(map: &Map, term: &mut Terminal) {
    for x in 0..map.0.width() as i32 {
        for y in 0..map.0.height() as i32 {
            let tile: Tile = match map.0[ [x as u32, y as u32] ] {
                MapTile::Wall => Tile {
                    glyph: '#',
                    fg_color: LinearRgba::from(WALL_COLOR),
                    bg_color: LinearRgba::from(Color::BLACK),
                },
                MapTile::Floor => Tile {
                    glyph: '.',
                    fg_color: LinearRgba::from(FLOOR_COLOR),
                    bg_color: LinearRgba::from(Color::BLACK),
                },
            };
            term.put_tile([x as i32, y as i32], tile);
        }
    }
}

fn render_all_entities<'a, Entities>(term: &mut Terminal, entities: Entities)
where
    Entities: Iterator<Item = (&'a Renderable, &'a Position)>,
{
    for (r, pos) in entities {
        term.put_tile(pos.0, Tile::from(r));
    }
}

fn should_render(
    q_entities_changed: Query<(&Renderable, &Position), Changed<Position>>,
    q_map_changed: Query<&Map, Changed<Map>>,
    mut evt_killed: EventReader<ActorKilledEvent>,
) -> bool {
    q_entities_changed.iter().next().is_some()
        || q_map_changed.iter().next().is_some()
        || evt_killed.read().next().is_some()
}