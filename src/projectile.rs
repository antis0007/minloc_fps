use bevy::math::primitives::Sphere;
use bevy::prelude::*;

use crate::app_state::GameState;
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
        Rocket { vel: dir.normalize_or_zero() * 20.0, life: 3.0, dmg },
    ));
}

fn move_rockets(
    time: Res<Time>,
    mut commands: Commands,
    mut rockets: Query<(Entity, &mut Transform, &mut Rocket), (With<Rocket>, Without<RemotePlayer>)>,
    mut players: Query<(&mut Player, &Transform), (With<RemotePlayer>, Without<Rocket>)>,
) {
    for (e, mut t, mut r) in &mut rockets {
        t.translation += r.vel * time.delta_secs();
        r.life -= time.delta_secs();

        let mut hit = r.life <= 0.0;
        for (mut p, pt) in &mut players {
            if p.hp > 0 && pt.translation.distance_squared(t.translation) < 1.0 {
                p.hp -= r.dmg;
                hit = true;
                break;
            }
        }

        if hit {
            commands.entity(e).despawn();
        }
    }
}