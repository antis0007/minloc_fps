use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::app_state::GameState;

pub struct PausePlugin;
impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowFocus>()
            .add_systems(Update, (track_focus, toggle_pause, sync_cursor))
            .add_systems(EguiPrimaryContextPass, pause_ui.run_if(in_state(GameState::Paused)));
    }
}

#[derive(Resource)]
struct WindowFocus(pub bool);
impl Default for WindowFocus {
    fn default() -> Self {
        Self(true)
    }
}

fn track_focus(mut ev: MessageReader<WindowFocused>, mut focus: ResMut<WindowFocus>) {
    for e in ev.read() {
        focus.0 = e.focused;
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

fn sync_cursor(
    state: Res<State<GameState>>,
    focus: Res<WindowFocus>,
    mut q: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let Ok((mut window, mut cursor)) = q.single_mut() else {
        return;
    };
    let lock = *state.get() == GameState::InGame && focus.0;
    cursor.visible = !lock;
    cursor.grab_mode = if lock {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    window.ime_enabled = !lock;
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
