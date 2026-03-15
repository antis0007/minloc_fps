use std::f32::consts::PI;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

use crate::app_state::{GameState, SessionConfig};
use crate::map::{ArenaMap, Solid};
use crate::weapon::{WeaponKind, slot_weapon, viewmodel_ascii};

#[derive(Component)]
pub struct LocalPlayer;
#[derive(Component)]
pub struct RemotePlayer;
#[derive(Component)]
pub struct ViewModelAscii;
#[derive(Component, Default)]
pub struct Velocity(pub Vec3);
#[derive(Component)]
pub struct MoveState {
    pub grounded: bool,
    pub crouched: bool,
    pub jump_buffer: f32,
    pub coyote: f32,
}
impl Default for MoveState {
    fn default() -> Self {
        Self {
            grounded: false,
            crouched: false,
            jump_buffer: 0.0,
            coyote: 0.0,
        }
    }
}
#[derive(Component)]
pub struct RespawnTimer(pub f32);

#[derive(Component)]
pub struct Player {
    pub hp: i32,
    pub weapon: WeaponKind,
}

#[derive(Component)]
pub struct ViewModelState {
    pub weapon: WeaponKind,
    pub sway: Vec2,
    pub bob_phase: f32,
    pub recoil: f32,
    pub reload: f32,
    pub last_yaw: f32,
    pub last_pitch: f32,
}
impl ViewModelState {
    fn new(weapon: WeaponKind) -> Self {
        Self {
            weapon,
            sway: Vec2::ZERO,
            bob_phase: 0.0,
            recoil: 0.0,
            reload: 0.0,
            last_yaw: 0.0,
            last_pitch: 0.0,
        }
    }
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
                (
                    ensure_local_player,
                    ensure_remote_player,
                    ensure_viewmodel_ascii,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    mouse_look,
                    movement,
                    choose_weapon,
                    animate_viewmodel,
                    update_remote,
                    respawn,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn cleanup_players(
    mut commands: Commands,
    q: Query<Entity, Or<(With<LocalPlayer>, With<RemotePlayer>, With<ViewModelAscii>)>>,
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
    let weapon = slot_weapon(cfg.selected_weapon);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(p[0], p[1], p[2]),
        LocalPlayer,
        Player { hp: 100, weapon },
        ViewModelState::new(weapon),
        Velocity::default(),
        MoveState::default(),
    ));
}

fn ensure_viewmodel_ascii(
    mut commands: Commands,
    local: Query<&ViewModelState, With<LocalPlayer>>,
    q: Query<(), With<ViewModelAscii>>,
) {
    if !q.is_empty() {
        return;
    }
    let Ok(vm) = local.single() else {
        return;
    };
    commands.spawn((
        Text::new(viewmodel_ascii(vm.weapon)),
        TextFont {
            font_size: 21.0,
            ..default()
        },
        TextColor(Color::srgb(0.93, 0.93, 0.93)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(56.0),
            bottom: Val::Px(18.0),
            ..default()
        },
        ViewModelAscii,
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
    mut q: Query<
        (&mut Transform, &mut Velocity, &mut MoveState),
        (With<LocalPlayer>, Without<RespawnTimer>),
    >,
) {
    let Ok((mut t, mut v, mut m)) = q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
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

    let crouch = keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::KeyC);
    m.crouched = crouch;

    let speed = if crouch {
        4.0
    } else if keys.pressed(KeyCode::ShiftLeft) {
        10.0
    } else {
        7.0
    };
    let accel = if m.grounded { 40.0 } else { 14.0 };
    let target = wish * speed;
    let blend = (accel * dt).min(1.0);
    v.0.x += (target.x - v.0.x) * blend;
    v.0.z += (target.z - v.0.z) * blend;

    m.jump_buffer = (m.jump_buffer - dt).max(0.0);
    m.coyote = (m.coyote - dt).max(0.0);
    if keys.just_pressed(KeyCode::Space) {
        m.jump_buffer = 0.12;
    }
    if m.grounded {
        m.coyote = 0.1;
    }
    if m.jump_buffer > 0.0 && m.coyote > 0.0 {
        m.jump_buffer = 0.0;
        m.coyote = 0.0;
        m.grounded = false;
        v.0.y = 8.0;
    }

    v.0.y -= 26.0 * dt;

    let old = t.translation;
    t.translation.x += v.0.x * dt;
    if touches_solid(t.translation, &map, m.crouched) {
        t.translation.x = old.x;
        v.0.x = 0.0;
    }

    t.translation.z += v.0.z * dt;
    if touches_solid(t.translation, &map, m.crouched) {
        t.translation.z = old.z;
        v.0.z = 0.0;
    }

    t.translation.y += v.0.y * dt;
    m.grounded = false;
    if touches_solid(t.translation, &map, m.crouched) {
        t.translation.y = old.y;
        if v.0.y < 0.0 {
            m.grounded = true;
        }
        v.0.y = 0.0;
    }

    if m.crouched && !crouch {
        let stand_y = t.translation.y + (stand_eye_offset() - crouch_eye_offset());
        if !touches_solid(
            Vec3::new(t.translation.x, stand_y, t.translation.z),
            &map,
            false,
        ) {
            t.translation.y = stand_y;
            m.crouched = false;
        }
    }
}

fn animate_viewmodel(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    look: Res<LookState>,
    mut local: Query<
        (
            &Player,
            &Velocity,
            Option<&RespawnTimer>,
            &mut ViewModelState,
        ),
        With<LocalPlayer>,
    >,
    mut ascii: Query<(&mut Text, &mut Node, &mut TextColor), With<ViewModelAscii>>,
) {
    let Ok((player, velocity, dead, mut vm)) = local.single_mut() else {
        return;
    };
    let Ok((mut text, mut node, mut color)) = ascii.single_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::KeyR) && vm.reload <= 0.0 {
        vm.reload = 0.65;
    }
    vm.reload = (vm.reload - time.delta_secs()).max(0.0);
    vm.recoil = (vm.recoil - time.delta_secs() * 5.0).max(0.0);

    if vm.weapon != player.weapon {
        *text = Text::new(viewmodel_ascii(player.weapon));
        vm.weapon = player.weapon;
    }

    if player.hp <= 0 || dead.is_some() {
        node.display = Display::None;
        return;
    }
    node.display = Display::Flex;

    let dyaw = shortest_angle_delta(look.yaw, vm.last_yaw);
    let dpitch = look.pitch - vm.last_pitch;
    vm.last_yaw = look.yaw;
    vm.last_pitch = look.pitch;
    let target_sway = Vec2::new(
        (dyaw * 16.0).clamp(-1.0, 1.0),
        (dpitch * 12.0).clamp(-1.0, 1.0),
    );
    let sway_now = vm.sway;
    vm.sway = sway_now + (target_sway - sway_now) * (time.delta_secs() * 16.0).min(1.0);

    let speed = Vec2::new(velocity.0.x, velocity.0.z).length();
    vm.bob_phase += time.delta_secs() * (2.2 + speed * 0.35);

    let bob_x = (vm.bob_phase * 2.0).cos() * 4.0 * (speed * 0.08).min(1.0);
    let bob_y = (vm.bob_phase * 4.0).sin().abs() * 6.0 * (speed * 0.08).min(1.0);

    let reload_t = if vm.reload > 0.0 {
        1.0 - vm.reload / 0.65
    } else {
        0.0
    };
    let reload_curve = (reload_t * PI).sin().max(0.0);

    node.right = Val::Px(56.0 - vm.sway.x * 26.0 - bob_x - reload_curve * 22.0);
    node.bottom = Val::Px(18.0 - vm.sway.y * 20.0 + bob_y - vm.recoil * 32.0 - reload_curve * 34.0);

    color.0 = if vm.reload > 0.0 {
        Color::srgb(0.78, 0.78, 0.78)
    } else {
        Color::srgb(0.93, 0.93, 0.93)
    };
}

fn touches_solid(eye: Vec3, map: &ArenaMap, crouched: bool) -> bool {
    map.solids
        .iter()
        .any(|s| capsule_hits_solid(eye, s, crouched))
}

fn capsule_hits_solid(eye: Vec3, s: &Solid, crouched: bool) -> bool {
    let radius = 0.28;
    let eye_to_feet = if crouched {
        crouch_eye_offset()
    } else {
        stand_eye_offset()
    };
    let height = if crouched { 0.95 } else { 1.35 };
    let foot = eye.y - eye_to_feet;
    let head = foot + height;
    let min = Vec3::from_array(s.min);
    let max = Vec3::from_array(s.max);

    if head <= min.y || foot >= max.y {
        return false;
    }
    let cx = eye.x.clamp(min.x, max.x);
    let cz = eye.z.clamp(min.z, max.z);
    let dx = eye.x - cx;
    let dz = eye.z - cz;
    dx * dx + dz * dz <= radius * radius
}

fn stand_eye_offset() -> f32 {
    0.5
}

fn crouch_eye_offset() -> f32 {
    0.2
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
            &mut ViewModelState,
            Option<&mut RespawnTimer>,
        ),
        With<LocalPlayer>,
    >,
    mut remote: Query<(&mut Transform, &mut Player), (With<RemotePlayer>, Without<LocalPlayer>)>,
) {
    let Ok((e, mut t, mut p, mut vm, timer)) = local.single_mut() else {
        return;
    };
    if p.hp <= 0 {
        vm.reload = 0.0;
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
                vm.weapon = p.weapon;
                vm.recoil = 0.0;
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

fn shortest_angle_delta(now: f32, before: f32) -> f32 {
    ((now - before) + PI).rem_euclid(2.0 * PI) - PI
}
