use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::app_state::GameState;
use crate::net::NetSession;
use crate::player::{LocalPlayer, Player, RemotePlayer, RespawnTimer};
use crate::weapon::weapon_name;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (hud_ui, scoreboard_ui, remote_nametags).run_if(in_state(GameState::InGame)),
        );
    }
}

fn hud_ui(
    mut ctx: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    q: Query<(&Player, Option<&RespawnTimer>), With<LocalPlayer>>,
    net: Res<NetSession>,
) -> Result {
    let (p, respawn) = q.single()?;
    let root = ctx.ctx_mut()?;

    egui::Area::new("hp_weapon".into())
        .anchor(egui::Align2::LEFT_BOTTOM, [14.0, -14.0])
        .show(root, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(170))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(80)))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("HP {:03}", p.hp.max(0)))
                            .size(24.0)
                            .strong()
                            .color(egui::Color32::from_rgb(120, 255, 140)),
                    );
                    ui.label(
                        egui::RichText::new(weapon_name(p.weapon))
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    );
                    ui.label(
                        egui::RichText::new("Ammo --/--")
                            .size(13.0)
                            .color(egui::Color32::from_gray(210)),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "Peers {}{}",
                            net.peers,
                            if net.is_host { " (host)" } else { "" }
                        ))
                        .size(12.0)
                        .color(if net.connected {
                            egui::Color32::from_gray(180)
                        } else {
                            egui::Color32::RED
                        }),
                    );
                    if let Some(t) = respawn {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("Respawn in {:.1}", t.0.max(0.0)),
                        );
                    }
                });
        });

    egui::Area::new("hotbar".into())
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -14.0])
        .show(root, |ui| {
            ui.horizontal(|ui| {
                for slot in 1..=5 {
                    let active = slot_weapon_index(p.weapon) == slot;
                    let (bg, fg) = if active {
                        (
                            egui::Color32::from_rgb(220, 190, 80),
                            egui::Color32::from_rgb(20, 20, 20),
                        )
                    } else {
                        (
                            egui::Color32::from_black_alpha(180),
                            egui::Color32::from_gray(220),
                        )
                    };
                    egui::Frame::new()
                        .fill(bg)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(90)))
                        .inner_margin(egui::Margin::symmetric(9, 6))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(format!("{}", slot)).color(fg).strong());
                        });
                }
            });
        });

    egui::Area::new("crosshair".into())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(root, |ui| {
            ui.label(
                egui::RichText::new("+")
                    .size(24.0)
                    .color(egui::Color32::WHITE),
            );
        });

    if keys.pressed(KeyCode::Tab) {
        egui::Area::new("tab_hint".into())
            .anchor(egui::Align2::CENTER_TOP, [0.0, 18.0])
            .show(root, |ui| {
                ui.label(
                    egui::RichText::new("TAB: Scoreboard")
                        .size(12.0)
                        .color(egui::Color32::from_gray(180)),
                );
            });
    }

    Ok(())
}

fn scoreboard_ui(
    mut ctx: EguiContexts,
    keys: Res<ButtonInput<KeyCode>>,
    local: Query<&Player, With<LocalPlayer>>,
    remote: Query<&Player, With<RemotePlayer>>,
) -> Result {
    if !keys.pressed(KeyCode::Tab) {
        return Ok(());
    }

    let local_hp = local.single()?.hp.max(0);
    let mut remote_rows = Vec::new();
    for (idx, p) in remote.iter().enumerate() {
        remote_rows.push((format!("Remote {}", idx + 1), p.hp.max(0)));
    }

    egui::Window::new("scoreboard")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 40.0])
        .fixed_size([360.0, 200.0])
        .show(ctx.ctx_mut()?, |ui| {
            ui.heading("Players");
            ui.separator();
            egui::Grid::new("players_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("HP");
                    ui.end_row();

                    ui.label("You");
                    ui.label(local_hp.to_string());
                    ui.end_row();

                    for (name, hp) in &remote_rows {
                        ui.label(name);
                        ui.label(hp.to_string());
                        ui.end_row();
                    }
                });
        });

    Ok(())
}

fn remote_nametags(
    mut ctx: EguiContexts,
    cam: Query<(&Camera, &GlobalTransform), With<LocalPlayer>>,
    remotes: Query<(Entity, &Transform, &Player), With<RemotePlayer>>,
) -> Result {
    let root = ctx.ctx_mut()?;
    let Ok((camera, camera_tf)) = cam.single() else {
        return Ok(());
    };

    for (e, t, p) in &remotes {
        let world = t.translation + Vec3::Y * 1.4;
        let Ok(screen) = camera.world_to_viewport(camera_tf, world) else {
            continue;
        };
        let hp = p.hp.max(0);
        let frac = (hp as f32 / 100.0).clamp(0.0, 1.0);
        let bar_w = 64.0;

        egui::Area::new(format!("remote_tag_{}", e.index()).into())
            .fixed_pos([screen.x - 40.0, screen.y - 34.0])
            .show(root, |ui| {
                ui.label(
                    egui::RichText::new(format!("Remote {}", e.index()))
                        .size(12.0)
                        .color(egui::Color32::WHITE),
                );
                let (_id, rect) = ui.allocate_space(egui::vec2(bar_w, 8.0));
                ui.painter()
                    .rect_filled(rect, 2.0, egui::Color32::from_gray(45));
                let filled = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2((bar_w - 2.0) * frac, rect.height() - 2.0),
                );
                ui.painter().rect_filled(
                    filled.translate(egui::vec2(1.0, 1.0)),
                    2.0,
                    egui::Color32::from_rgb(100, 230, 120),
                );
            });
    }

    Ok(())
}

fn slot_weapon_index(w: crate::weapon::WeaponKind) -> usize {
    match w {
        crate::weapon::WeaponKind::HeavyPistol => 1,
        crate::weapon::WeaponKind::Smg => 2,
        crate::weapon::WeaponKind::AssaultRifle => 3,
        crate::weapon::WeaponKind::SniperRifle => 4,
        crate::weapon::WeaponKind::RocketLauncher => 5,
    }
}
