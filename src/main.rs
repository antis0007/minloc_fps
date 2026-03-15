use bevy::prelude::*;

mod app_state;
mod arena;
mod combat;
mod hud;
mod map;
mod menu;
mod net;
mod pause;
mod player;
mod projectile;
mod weapon;

use app_state::AppStatePlugin;
use arena::ArenaPlugin;
use combat::CombatPlugin;
use hud::HudPlugin;
use map::MapPlugin;
use menu::MenuPlugin;
use net::NetPlugin;
use pause::PausePlugin;
use player::PlayerPlugin;
use projectile::ProjectilePlugin;
use weapon::WeaponPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Micro FPS".into(), ..default() }),
            ..default()
        }))
        .add_plugins(bevy_egui::EguiPlugin::default())
        .add_plugins((
            AppStatePlugin,
            MenuPlugin,
            PausePlugin,
            NetPlugin,
            MapPlugin,
            ArenaPlugin,
            PlayerPlugin,
            WeaponPlugin,
            ProjectilePlugin,
            CombatPlugin,
            HudPlugin,
        ))
        .run();
}
