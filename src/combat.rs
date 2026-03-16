use bevy::math::primitives::Sphere;
use bevy::prelude::*;

use crate::app_state::GameState;
use crate::map::ArenaMap;
use crate::player::{LocalPlayer, LookState, Player, RemotePlayer, ViewModelState};
use crate::projectile::spawn_rocket;
use crate::weapon::{
    Cooldown, auto_fire, damage, fire_interval, is_projectile, mag_size, recoil, reload_time,
    reserve_ammo, weapon_name,
};

pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ensure_cooldown).add_systems(
            Update,
            (tick_impacts, fire)
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct ImpactMarker {
    pub life: f32,
}

fn ensure_cooldown(mut commands: Commands, q: Query<Entity, (With<Player>, Without<Cooldown>)>) {
    for e in &q {
        commands.entity(e).insert(Cooldown::default());
    }
}

fn fire(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<ArenaMap>,
    mut look: ResMut<LookState>,
    mut local: Query<
        (&Transform, &mut Player, &mut Cooldown, &mut ViewModelState),
        (With<LocalPlayer>, Without<RemotePlayer>),
    >,
    mut remote: Query<(&Transform, &mut Player), (With<RemotePlayer>, Without<LocalPlayer>)>,
) {
    let Ok((t, mut p, mut cd, mut vm)) = local.single_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::KeyR)
        && vm.reload <= 0.0
        && p.clip < mag_size(p.weapon)
        && p.reserve > 0
    {
        vm.reload = reload_time(p.weapon);
    }

    if vm.reload > 0.0 {
        vm.reload = (vm.reload - time.delta_secs()).max(0.0);
        if vm.reload <= 0.0 {
            let need = (mag_size(p.weapon) - p.clip).max(0);
            let take = need.min(p.reserve);
            p.clip += take;
            p.reserve -= take;
        }
        return;
    }

    let wants_shot = if auto_fire(p.weapon) {
        buttons.pressed(MouseButton::Left)
    } else {
        buttons.just_pressed(MouseButton::Left)
    };
    if !wants_shot || cd.0 > 0.0 || p.hp <= 0 {
        return;
    }

    if p.clip <= 0 {
        if p.reserve > 0 {
            vm.reload = reload_time(p.weapon);
        }
        return;
    }

    p.clip -= 1;
    cd.0 = fire_interval(p.weapon);
    let r = recoil(p.weapon);
    look.kick += r;
    vm.recoil = (vm.recoil + r * 1.8).min(1.0);
    vm.flash = 0.06;
    println!("synth shot: {}", weapon_name(p.weapon));

    let d = damage(p.weapon);
    let origin = t.translation;
    let dir = t.forward().as_vec3();
    if is_projectile(p.weapon) {
        spawn_rocket(
            &mut commands,
            &mut meshes,
            &mut materials,
            origin + dir * 0.8,
            dir,
            d,
        );
    } else {
        fire_hitscan(
            &mut commands,
            &mut meshes,
            &mut materials,
            &map,
            origin,
            dir,
            d,
            &mut remote,
        );
    }
}

fn fire_hitscan(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    map: &ArenaMap,
    origin: Vec3,
    dir: Vec3,
    dmg: i32,
    remote: &mut Query<(&Transform, &mut Player), (With<RemotePlayer>, Without<LocalPlayer>)>,
) {
    let max_dist = 80.0;
    let mut wall_hit: Option<(f32, Vec3)> = None;
    for s in &map.solids {
        let min = Vec3::from_array(s.min);
        let max = Vec3::from_array(s.max);
        if let Some((t, n)) = ray_aabb(origin, dir, max_dist, min, max) {
            if wall_hit.is_none_or(|(best, _)| t < best) {
                wall_hit = Some((t, n));
            }
        }
    }

    let mut player_hit: Option<(f32, Vec3)> = None;
    for (rt, rp) in remote.iter_mut() {
        if rp.hp <= 0 {
            continue;
        }
        let center = rt.translation + Vec3::Y * 0.4;
        if let Some(t) = ray_sphere(origin, dir, max_dist, center, 0.7) {
            if player_hit.is_none_or(|(best, _)| t < best) {
                player_hit = Some((t, center));
            }
        }
    }

    if let Some((pt, center)) = player_hit {
        if wall_hit.is_none_or(|(wt, _)| pt <= wt) {
            for (rt, mut rp) in remote.iter_mut() {
                let rc = rt.translation + Vec3::Y * 0.4;
                if rc.distance_squared(center) < 0.01 && rp.hp > 0 {
                    rp.hp -= dmg;
                    let pos = origin + dir * pt;
                    spawn_impact(
                        commands,
                        meshes,
                        materials,
                        pos,
                        Color::srgb(1.0, 0.18, 0.18),
                        0.2,
                    );
                    return;
                }
            }
        }
    }

    if let Some((wt, n)) = wall_hit {
        let pos = origin + dir * wt + n * 0.03;
        spawn_impact(
            commands,
            meshes,
            materials,
            pos,
            Color::srgb(0.95, 0.9, 0.75),
            0.6,
        );
    }
}

pub fn spawn_impact(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec3,
    color: Color,
    life: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.05))),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: color.into(),
            base_color: color,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(pos),
        ImpactMarker { life },
    ));
}

fn tick_impacts(
    time: Res<Time>,
    mut commands: Commands,
    mut impacts: Query<(Entity, &mut ImpactMarker)>,
) {
    for (e, mut i) in &mut impacts {
        i.life -= time.delta_secs();
        if i.life <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

fn ray_sphere(origin: Vec3, dir: Vec3, max_dist: f32, center: Vec3, radius: f32) -> Option<f32> {
    let m = origin - center;
    let b = m.dot(dir);
    let c = m.length_squared() - radius * radius;
    if c > 0.0 && b > 0.0 {
        return None;
    }
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()).max(0.0);
    (t <= max_dist).then_some(t)
}

fn ray_aabb(origin: Vec3, dir: Vec3, max_dist: f32, min: Vec3, max: Vec3) -> Option<(f32, Vec3)> {
    let mut tmin = 0.0;
    let mut tmax = max_dist;
    let mut hit_n = Vec3::ZERO;

    for axis in 0..3 {
        let o = origin[axis];
        let d = dir[axis];
        let amin = min[axis];
        let amax = max[axis];
        if d.abs() < 1e-6 {
            if o < amin || o > amax {
                return None;
            }
            continue;
        }

        let inv = 1.0 / d;
        let mut t1 = (amin - o) * inv;
        let mut t2 = (amax - o) * inv;
        let mut n = match axis {
            0 => Vec3::new(-1.0, 0.0, 0.0),
            1 => Vec3::new(0.0, -1.0, 0.0),
            _ => Vec3::new(0.0, 0.0, -1.0),
        };
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
            n = -n;
        }
        if t1 > tmin {
            tmin = t1;
            hit_n = n;
        }
        tmax = tmax.min(t2);
        if tmin > tmax {
            return None;
        }
    }

    if (0.0..=max_dist).contains(&tmin) {
        Some((tmin, hit_n))
    } else {
        None
    }
}

pub fn refill_for_weapon(p: &mut Player) {
    p.clip = mag_size(p.weapon);
    p.reserve = reserve_ammo(p.weapon);
}
