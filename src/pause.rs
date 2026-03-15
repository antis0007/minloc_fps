use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::app_state::GameState;

pub struct PausePlugin;
impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_pause)
            .add_systems(EguiPrimaryContextPass, pause_ui.run_if(in_state(GameState::Paused)));
    }
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::InGame => next.set(GameState::Paused),
            GameState::Paused => next.set(GameState::InGame),
            _ => {}
        }
    }
}

fn pause_ui(mut ctx: EguiContexts, mut next: ResMut<NextState<GameState>>) -> Result {
    egui::Window::new("Paused")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx.ctx_mut()?, |ui| {
            if ui.button("Resume").clicked() {
                next.set(GameState::InGame);
            }
            if ui.button("Main Menu").clicked() {
                next.set(GameState::MainMenu);
            }
        });
    Ok(())
}