use bevy::prelude::*;

use crate::app_state::GameState;
use crate::player::{LocalPlayer, LookState, Player, RemotePlayer, ViewModelState};
use crate::projectile::spawn_rocket;
use crate::weapon::{
    Cooldown, auto_fire, damage, fire_interval, is_projectile, recoil, weapon_name,
};

pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ensure_cooldown)
            .add_systems(Update, fire.run_if(in_state(GameState::InGame)));
    }
}

fn ensure_cooldown(mut commands: Commands, q: Query<Entity, (With<Player>, Without<Cooldown>)>) {
    for e in &q {
        commands.entity(e).insert(Cooldown::default());
    }
}

fn fire(
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut look: ResMut<LookState>,
    mut local: Query<
        (&Transform, &Player, &mut Cooldown, &mut ViewModelState),
        (With<LocalPlayer>, Without<RemotePlayer>),
    >,
    mut remote: Query<(&Transform, &mut Player), (With<RemotePlayer>, Without<LocalPlayer>)>,
) {
    let Ok((t, p, mut cd, mut vm)) = local.single_mut() else {
        return;
    };
    let wants_shot = if auto_fire(p.weapon) {
        buttons.pressed(MouseButton::Left)
    } else {
        buttons.just_pressed(MouseButton::Left)
    };
    if !wants_shot || cd.0 > 0.0 || p.hp <= 0 || vm.reload > 0.0 {
        return;
    }

    cd.0 = fire_interval(p.weapon);
    let recoil = recoil(p.weapon);
    look.kick += recoil;
    vm.recoil = (vm.recoil + recoil * 1.8).min(1.0);
    println!("synth shot: {}", weapon_name(p.weapon));

    let d = damage(p.weapon);
    let dir = t.forward().as_vec3();
    if is_projectile(p.weapon) {
        spawn_rocket(
            &mut commands,
            &mut meshes,
            &mut materials,
            t.translation + dir * 0.8,
            dir,
            d,
        );
    } else {
        for (rt, mut rp) in &mut remote {
            let to = rt.translation - t.translation;
            if rp.hp > 0 && to.length_squared() < 6400.0 && dir.dot(to.normalize_or_zero()) > 0.995
            {
                rp.hp -= d;
            }
        }
    }
}
