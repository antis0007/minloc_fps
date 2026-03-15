use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeaponKind {
    HeavyPistol,
    Smg,
    AssaultRifle,
    SniperRifle,
    RocketLauncher,
}

#[derive(Component, Default)]
pub struct Cooldown(pub f32);

pub struct WeaponPlugin;
impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick_cooldowns);
    }
}

pub fn slot_weapon(slot: usize) -> WeaponKind {
    match slot {
        1 => WeaponKind::HeavyPistol,
        2 => WeaponKind::Smg,
        3 => WeaponKind::AssaultRifle,
        4 => WeaponKind::SniperRifle,
        5 => WeaponKind::RocketLauncher,
        _ => WeaponKind::AssaultRifle,
    }
}

pub fn weapon_name(w: WeaponKind) -> &'static str {
    match w {
        WeaponKind::HeavyPistol => "Heavy Pistol",
        WeaponKind::Smg => "SMG",
        WeaponKind::AssaultRifle => "Assault Rifle",
        WeaponKind::SniperRifle => "Sniper Rifle",
        WeaponKind::RocketLauncher => "Rocket Launcher",
    }
}

pub fn viewmodel_ascii(w: WeaponKind) -> &'static str {
    match w {
        WeaponKind::HeavyPistol => "  __\n |==]\n/___>",
        WeaponKind::Smg => "  ____\n |====]\n/_____/",
        WeaponKind::AssaultRifle => "   ______\n  |======]\n_/_______/",
        WeaponKind::SniperRifle => "   __________\n  |==========]\n_/___________/",
        WeaponKind::RocketLauncher => "   .----.\n  |=====>]\n_/______/'",
    }
}
pub fn fire_interval(w: WeaponKind) -> f32 {
    match w {
        WeaponKind::HeavyPistol => 0.35,
        WeaponKind::Smg => 0.08,
        WeaponKind::AssaultRifle => 0.11,
        WeaponKind::SniperRifle => 0.9,
        WeaponKind::RocketLauncher => 0.8,
    }
}

pub fn damage(w: WeaponKind) -> i32 {
    match w {
        WeaponKind::HeavyPistol => 45,
        WeaponKind::Smg => 12,
        WeaponKind::AssaultRifle => 22,
        WeaponKind::SniperRifle => 95,
        WeaponKind::RocketLauncher => 120,
    }
}

pub fn recoil(w: WeaponKind) -> f32 {
    match w {
        WeaponKind::HeavyPistol => 0.05,
        WeaponKind::Smg => 0.015,
        WeaponKind::AssaultRifle => 0.025,
        WeaponKind::SniperRifle => 0.09,
        WeaponKind::RocketLauncher => 0.12,
    }
}

pub fn is_projectile(w: WeaponKind) -> bool {
    matches!(w, WeaponKind::RocketLauncher)
}

pub fn auto_fire(w: WeaponKind) -> bool {
    matches!(w, WeaponKind::Smg | WeaponKind::AssaultRifle)
}

fn tick_cooldowns(time: Res<Time>, mut q: Query<&mut Cooldown>) {
    for mut cd in &mut q {
        cd.0 = (cd.0 - time.delta_secs()).max(0.0);
    }
}
