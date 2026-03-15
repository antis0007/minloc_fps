use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::app_state::GameState;
use crate::net::NetSession;
use crate::player::{LocalPlayer, Player, RemotePlayer, RespawnTimer};
use crate::weapon::weapon_name;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hud_ui.run_if(in_state(GameState::InGame)));
    }
}

fn hud_ui(
    mut ctx: EguiContexts,
    q: Query<(&Player, Option<&RespawnTimer>), With<LocalPlayer>>,
    r: Query<&Player, With<RemotePlayer>>,
    net: Res<NetSession>,
) {
    let Ok(ctx) = ctx.ctx_mut() else { return };
    let Ok((p, respawn)) = q.single() else { return; };
    let enemy_hp = r.single().map(|p| p.hp).unwrap_or(0);
    egui::Area::new("hud".into()).fixed_pos([12.0, 12.0]).show(ctx, |ui| {
        ui.label(format!("HP: {}", p.hp.max(0)));
        ui.label(format!("Weapon: {}", weapon_name(p.weapon)));
        ui.label(format!("Enemy HP: {}", enemy_hp.max(0)));
        ui.label(format!("Peers: {} host:{}", net.peers, net.is_host));
        if let Some(t) = respawn {
            ui.colored_label(egui::Color32::RED, format!("Respawn in {:.1}", t.0.max(0.0)));
        }
    });
    egui::Area::new("crosshair".into())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("+");
        });
}
