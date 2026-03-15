use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

use crate::app_state::GameState;
use crate::map::ArenaMap;

#[derive(Component)]
pub struct ArenaEntity;

pub struct ArenaPlugin;
impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)))
            .insert_resource(GlobalAmbientLight {
                color: Color::WHITE,
                brightness: 300.0,
                affects_lightmapped_meshes: true,
            })
            .add_systems(
                Update,
                (setup_scene, spawn_arena).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnEnter(GameState::MainMenu), cleanup)
            .add_systems(OnExit(GameState::InGame), cleanup);
    }
}

fn setup_scene(mut commands: Commands, q: Query<(), (With<DirectionalLight>, With<ArenaEntity>)>) {
    if !q.is_empty() {
        return;
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ArenaEntity,
    ));
}

fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<ArenaMap>,
    q: Query<(), (With<ArenaEntity>, With<Mesh3d>)>,
) {
    if !q.is_empty() {
        return;
    }
    for s in &map.solids {
        let min = Vec3::from_array(s.min);
        let max = Vec3::from_array(s.max);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(max - min))),
            MeshMaterial3d(materials.add(Color::srgb(0.22, 0.24, 0.28))),
            Transform::from_translation((min + max) * 0.5),
            ArenaEntity,
        ));
    }
}

fn cleanup(mut commands: Commands, q: Query<Entity, With<ArenaEntity>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}