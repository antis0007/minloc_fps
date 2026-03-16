use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::LineBreak;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::app_state::GameState;
use crate::net::NetSession;
use crate::player::{LocalPlayer, Player, RemotePlayer, RespawnTimer, ViewModelState};
use crate::weapon::{viewmodel_ascii, weapon_name};

#[derive(Component)]
struct ViewModelAscii3d;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (hud_ui, scoreboard_ui, remote_nametags).run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            (ensure_viewmodel_3d, update_viewmodel_3d).run_if(in_state(GameState::InGame)),
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
                        egui::RichText::new(format!("Ammo {}/{}", p.clip.max(0), p.reserve.max(0)))
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

fn ensure_viewmodel_3d(
    mut commands: Commands,
    cameras: Query<Entity, (With<LocalPlayer>, With<Camera3d>)>,
    vm_text: Query<(), With<ViewModelAscii3d>>,
) {
    let Ok(camera) = cameras.single() else { return };
    if !vm_text.is_empty() {
        return;
    }
    commands.entity(camera).with_child((
        ViewModelAscii3d,
        Text2d::new(""),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgb(0.93, 0.93, 0.93)),
        TextLayout::new_with_linebreak(LineBreak::NoWrap),
        Transform::from_xyz(0.34, -0.22, -0.62),
        Anchor::BOTTOM_RIGHT,
    ));
}

fn update_viewmodel_3d(
    q_local: Query<(&Player, Option<&RespawnTimer>, &ViewModelState), With<LocalPlayer>>,
    mut q_text: Query<(&mut Text2d, &mut TextColor, &mut Transform, &mut Visibility), With<ViewModelAscii3d>>,
) {
    let Ok((p, respawn, vm)) = q_local.single() else {
        return;
    };
    let Ok((mut text, mut color, mut transform, mut visibility)) = q_text.single_mut() else {
        return;
    };

    if p.hp <= 0 || respawn.is_some() {
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Visible;

    let ascii = viewmodel_ascii(vm.weapon);
    text.0 = ascii.to_owned();

    let rows = ascii.lines().count().max(1) as f32;
    let cols = ascii.lines().map(|line| line.chars().count()).max().unwrap_or(1) as f32;
    let scale = (0.40 / cols).min(0.26 / rows).max(0.010);

    transform.translation = Vec3::new(
        0.34 + vm.screen_offset.x * 0.0028,
        -0.22 - vm.screen_offset.y * 0.0022,
        -0.62,
    );
    transform.scale = Vec3::new(-scale, scale, 1.0);

    color.0 = if vm.flash > 0.0 {
        Color::srgb(1.0, 0.94, 0.65)
    } else if vm.reload > 0.0 {
        Color::srgb(0.77, 0.77, 0.77)
    } else {
        Color::srgb(0.93, 0.93, 0.93)
    };
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
