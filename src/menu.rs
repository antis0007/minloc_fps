use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::app_state::{GameState, SessionConfig};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, menu_ui.run_if(in_state(GameState::MainMenu)));
    }
}

fn menu_ui(
    mut ctx: EguiContexts,
    mut next: ResMut<NextState<GameState>>,
    mut cfg: ResMut<SessionConfig>,
) {
    let Ok(ctx) = ctx.ctx_mut() else { return };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.heading("Micro FPS");
            ui.label("Minimal Rust multiplayer FPS MVP");
            ui.separator();
            ui.text_edit_singleline(&mut cfg.name);
            ui.horizontal(|ui| {
                ui.label("Address:");
                ui.text_edit_singleline(&mut cfg.addr);
            });
            ui.horizontal(|ui| {
                ui.label("Map:");
                ui.text_edit_singleline(&mut cfg.map_path);
            });
            ui.horizontal(|ui| {
                if ui.button("Host").clicked() {
                    cfg.host = true;
                    next.set(GameState::InGame);
                }
                if ui.button("Join").clicked() {
                    cfg.host = false;
                    next.set(GameState::InGame);
                }
            });
            ui.label("Press 1-5 to choose weapon, LMB to fire.");
        });
    });
}