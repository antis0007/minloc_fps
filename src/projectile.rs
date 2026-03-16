use bevy::math::primitives::Sphere;
use bevy::prelude::*;

use crate::app_state::GameState;
use crate::combat::spawn_impact;
use crate::map::ArenaMap;
use crate::player::{Player, RemotePlayer};

#[derive(Component)]
pub struct Rocket {
    pub vel: Vec3,
    pub life: f32,
    pub dmg: i32,
}

pub struct ProjectilePlugin;
impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_rockets.run_if(in_state(GameState::InGame)));
    }
}

pub fn spawn_rocket(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec3,
    dir: Vec3,
    dmg: i32,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.12))),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: Color::srgb(1.0, 0.4, 0.1).into(),
            ..default()
        })),
        Transform::from_translation(pos),
        Rocket {
            vel: dir.normalize_or_zero() * 20.0,
            life: 3.0,
            dmg,
        },
    ));
}

fn move_rockets(
    time: Res<Time>,
    map: Res<ArenaMap>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rockets: Query<
        (Entity, &mut Transform, &mut Rocket),
        (With<Rocket>, Without<RemotePlayer>),
    >,
    mut players: Query<(&mut Player, &Transform), (With<RemotePlayer>, Without<Rocket>)>,
) {
    for (e, mut t, mut r) in &mut rockets {
        let old = t.translation;
        let new_pos = old + r.vel * time.delta_secs();
        t.translation = new_pos;
        r.life -= time.delta_secs();

        let mut hit = r.life <= 0.0;
        for (mut p, pt) in &mut players {
            if p.hp > 0 && pt.translation.distance_squared(new_pos) < 1.0 {
                p.hp -= r.dmg;
                hit = true;
                spawn_impact(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    new_pos,
                    Color::srgb(1.0, 0.2, 0.2),
                    0.25,
                );
                break;
            }
        }

        if !hit {
            let travel = new_pos - old;
            let len = travel.length();
            if len > 0.0001 {
                let dir = travel / len;
                for s in &map.solids {
                    let min = Vec3::from_array(s.min) - Vec3::splat(0.12);
                    let max = Vec3::from_array(s.max) + Vec3::splat(0.12);
                    if let Some((dist, n)) = ray_aabb(old, dir, len, min, max) {
                        let pos = old + dir * dist + n * 0.04;
                        spawn_impact(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos,
                            Color::srgb(1.0, 0.85, 0.35),
                            0.7,
                        );
                        hit = true;
                        break;
                    }
                }
            }
        }

        if hit {
            commands.entity(e).despawn();
        }
    }
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

    (0.0..=max_dist).contains(&tmin).then_some((tmin, hit_n))
}
