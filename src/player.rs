use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

use crate::app_state::{GameState, SessionConfig};
use crate::map::{ArenaMap, Solid};
use crate::weapon::{slot_weapon, WeaponKind};

#[derive(Component)]
pub struct LocalPlayer;
#[derive(Component)]
pub struct RemotePlayer;
#[derive(Component, Default)]
pub struct Velocity(pub Vec3);
#[derive(Component)]
pub struct RespawnTimer(pub f32);

#[derive(Component)]
pub struct Player {
    pub hp: i32,
    pub weapon: WeaponKind,
}

#[derive(Resource, Default)]
pub struct LookState {
    pub yaw: f32,
    pub pitch: f32,
    pub kick: f32,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LookState>()
            .add_systems(OnEnter(GameState::MainMenu), cleanup_players)
            .add_systems(
                Update,
                (ensure_local_player, ensure_remote_player).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (mouse_look, movement, choose_weapon, update_remote, respawn)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn cleanup_players(
    mut commands: Commands,
    q: Query<Entity, Or<(With<LocalPlayer>, With<RemotePlayer>)>>,
    mut look: ResMut<LookState>,
) {
    for e in &q {
        commands.entity(e).despawn();
    }
    *look = LookState::default();
}

fn ensure_local_player(
    mut commands: Commands,
    cfg: Res<SessionConfig>,
    map: Res<ArenaMap>,
    q: Query<(), With<LocalPlayer>>,
) {
    if !q.is_empty() {
        return;
    }
    let p = map
        .spawn_points
        .first()
        .map(|s| s.pos)
        .unwrap_or([0.0, 1.5, 0.0]);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(p[0], p[1], p[2]),
        LocalPlayer,
        Player {
            hp: 100,
            weapon: slot_weapon(cfg.selected_weapon),
        },
        Velocity::default(),
    ));
}

fn ensure_remote_player(
    mut commands: Commands,
    map: Res<ArenaMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    q: Query<(), With<RemotePlayer>>,
) {
    if !q.is_empty() {
        return;
    }
    let p = map
        .spawn_points
        .get(1)
        .map(|s| s.pos)
        .unwrap_or([6.0, 1.5, 0.0]);
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Capsule3d::new(0.35, 1.0)))),
        MeshMaterial3d(mats.add(Color::srgb(0.85, 0.2, 0.2))),
        Transform::from_xyz(p[0], p[1], p[2]),
        RemotePlayer,
        Player {
            hp: 100,
            weapon: WeaponKind::AssaultRifle,
        },
    ));
}

fn mouse_look(
    mut ev: MessageReader<MouseMotion>,
    mut look: ResMut<LookState>,
    mut q: Query<&mut Transform, (With<LocalPlayer>, Without<RespawnTimer>)>,
) {
    let delta = ev.read().fold(Vec2::ZERO, |a, e| a + e.delta) * 0.002;
    look.yaw -= delta.x;
    look.pitch = (look.pitch - delta.y).clamp(-1.54, 1.54);
    let pitch = (look.pitch + look.kick).clamp(-1.54, 1.54);
    look.kick *= 0.85;
    let Ok(mut t) = q.single_mut() else { return };
    t.rotation = Quat::from_axis_angle(Vec3::Y, look.yaw) * Quat::from_axis_angle(Vec3::X, pitch);
}

fn movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    map: Res<ArenaMap>,
    mut q: Query<(&mut Transform, &mut Velocity), (With<LocalPlayer>, Without<RespawnTimer>)>,
) {
    let Ok((mut t, mut v)) = q.single_mut() else {
        return;
    };
    let mut input = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        input.z -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        input.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        input.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        input.x += 1.0;
    }

    let f = t.forward().as_vec3();
    let r = t.right().as_vec3();
    let wish = (Vec3::new(f.x, 0.0, f.z) * -input.z + Vec3::new(r.x, 0.0, r.z) * input.x)
        .normalize_or_zero();

    let speed = if keys.pressed(KeyCode::ShiftLeft) {
        10.0
    } else {
        7.0
    };
    v.0.x = wish.x * speed;
    v.0.z = wish.z * speed;
    v.0.y -= 20.0 * time.delta_secs();

    if t.translation.y <= 1.5 {
        t.translation.y = 1.5;
        v.0.y = 0.0;
        if keys.just_pressed(KeyCode::Space) {
            v.0.y = 7.5;
        }
    }

    let old = t.translation;
    t.translation += v.0 * time.delta_secs();
    if map.solids.iter().any(|s| in_solid(t.translation, s)) {
        t.translation = old;
    }
}

fn in_solid(p: Vec3, s: &Solid) -> bool {
    let min = Vec3::from_array(s.min) + Vec3::splat(0.2);
    let max = Vec3::from_array(s.max) - Vec3::splat(0.2);
    p.cmpge(min).all() && p.cmple(max).all()
}

fn choose_weapon(
    keys: Res<ButtonInput<KeyCode>>,
    mut cfg: ResMut<SessionConfig>,
    mut q: Query<&mut Player, With<LocalPlayer>>,
) {
    let slot = if keys.just_pressed(KeyCode::Digit1) {
        1
    } else if keys.just_pressed(KeyCode::Digit2) {
        2
    } else if keys.just_pressed(KeyCode::Digit3) {
        3
    } else if keys.just_pressed(KeyCode::Digit4) {
        4
    } else if keys.just_pressed(KeyCode::Digit5) {
        5
    } else {
        return;
    };
    cfg.selected_weapon = slot;
    let Ok(mut p) = q.single_mut() else { return };
    p.weapon = slot_weapon(slot);
}

fn update_remote(time: Res<Time>, mut q: Query<(&mut Transform, &mut Player), With<RemotePlayer>>) {
    let Ok((mut t, p)) = q.single_mut() else {
        return;
    };
    if p.hp > 0 {
        t.translation.x = 6.0 + (time.elapsed_secs() * 0.8).sin() * 3.0;
        t.translation.z = (time.elapsed_secs() * 0.5).cos() * 2.0;
    }
}

fn respawn(
    time: Res<Time>,
    cfg: Res<SessionConfig>,
    map: Res<ArenaMap>,
    mut commands: Commands,
    mut local: Query<
        (
            Entity,
            &mut Transform,
            &mut Player,
            Option<&mut RespawnTimer>,
        ),
        With<LocalPlayer>,
    >,
    mut remote: Query<(&mut Transform, &mut Player), (With<RemotePlayer>, Without<LocalPlayer>)>,
) {
    let Ok((e, mut t, mut p, timer)) = local.single_mut() else {
        return;
    };
    if p.hp <= 0 {
        if let Some(mut timer) = timer {
            timer.0 -= time.delta_secs();
            if timer.0 <= 0.0 {
                let s = map
                    .spawn_points
                    .first()
                    .map(|p| p.pos)
                    .unwrap_or([0.0, 1.5, 0.0]);
                t.translation = Vec3::from_array(s);
                p.hp = 100;
                p.weapon = slot_weapon(cfg.selected_weapon);
                commands.entity(e).remove::<RespawnTimer>();
            }
        } else {
            commands.entity(e).insert(RespawnTimer(3.0));
        }
    }

    let Ok((mut rt, mut rp)) = remote.single_mut() else {
        return;
    };
    if rp.hp <= 0 {
        let s = map
            .spawn_points
            .get(1)
            .map(|p| p.pos)
            .unwrap_or([8.0, 1.5, 0.0]);
        rt.translation = Vec3::from_array(s);
        rp.hp = 100;
    }
}
