use bevy::prelude::*;
use bevy_ascii_terminal::{terminal::Terminal, border::TerminalBorder, color, string::DecoratedString, StringDecorator, TerminalMeshPivot};
use interpolation::Lerp;

use crate::{UI_SIZE, VIEWPORT_SIZE, events::AttackEvent, combat::{HitPoints, MaxHitPoints}, player::Player, AppState};
use crate::map::Side;

pub struct UiPlugin;

#[derive(Component)]
pub struct UiTerminal;

#[derive(Default, Resource)]
pub struct PrintLog {
    log: Vec<String>,
}

impl PrintLog {
    pub fn push(&mut self, message: String) {
        self.log.push(message);
    }
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_ui)
            .add_systems(Update, handle_attacks.run_if(in_state(AppState::InGame)))
            .add_systems(Update, handle_print.run_if(in_state(AppState::InGame).and(has_ui_terminal)))
            .init_resource::<PrintLog>();
    }
}

fn setup_ui(mut commands: Commands) {
    let mut term = Terminal::new(UI_SIZE);

    commands.spawn((
        term,
        TerminalBorder::single_line(),
        TerminalMeshPivot::TopLeft, // makes coordinate origin top-left
        UiTerminal,
        Transform::from_xyz(0.0, 0.0, 0.0), 
        GlobalTransform::default(),
    ));
}
fn has_ui_terminal(q: Query<(), With<UiTerminal>>) -> bool {
    !q.is_empty()
}
fn handle_attacks(
    _print_log: ResMut<PrintLog>,
    mut event_attacked: EventReader<AttackEvent>,
) {
    for _ev in event_attacked.read() {
        // Log attack events if needed
    }
}

fn handle_print(
    mut print_log: ResMut<PrintLog>,
    mut q_term: Query<&mut Terminal, With<UiTerminal>>,
    q_player: Query<(&HitPoints, &MaxHitPoints), With<Player>>,
) {
    if !print_log.is_changed() && q_player.is_empty() {
        warn!("Player not found for HP rendering");
    }

    let mut term = match q_term.single_mut() {
        Ok(term) => term,
        Err(_) => {
            warn!("UiTerminal not found during print update");
            return;
        }
    };

    term.clear();

    // Optional headers
    term.put_string([1, 0], "SYSTEM LOG:".fg(color::CYAN));
    // Render log messages (newest at bottom)
    let log_start_y = 1;
    let max_lines = 6;
    let log_slice = print_log.log.iter().rev().take(max_lines);
    for (i, text) in log_slice.enumerate() {
        let y = log_start_y + i as i32;
        let col = match i {
            0 => color::WHITE,
            1 => color::LIGHT_GRAY,
            2 => color::GRAY,
            3 => color::DARK_GRAY,
            _ => color::DARK_GRAY.with_alpha(0.5),
        };
        term.put_string([1, y], DecoratedString::from(text).fg(col));
    }

    // Render HP bar
    if let Ok((hp, max)) = q_player.single() {
        let hp_val = hp.0;
        let max_val = max.0;
        let bar_width = term.width() as usize - 20;

        let filled_len = ((hp_val as f32 / max_val as f32) * bar_width as f32).round() as usize;
        let empty_len = bar_width - filled_len;

        let bar_x = term.width() as i32 - bar_width as i32 - 1;
        let bar_y = term.height() as i32 - 2;

        let hp_label = format!("HP: {}/{}", hp_val, max_val);
        let label_x = bar_x - hp_label.len() as i32 - 1;

        term.put_string([label_x, 7], hp_label.fg(color::YELLOW));
        term.put_string([bar_x, 7], "█".repeat(filled_len).fg(color::RED));
        term.put_string([bar_x + filled_len as i32, 7], "□".repeat(empty_len).fg(color::DARK_GRAY));
    }
}