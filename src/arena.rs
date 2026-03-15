use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

use crate::app_state::GameState;
use crate::map::ArenaMap;

#[derive(Component)]
pub struct ArenaEntity;

pub struct ArenaPlugin;
impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), (setup_scene, spawn_arena).chain())
            .add_systems(OnEnter(GameState::MainMenu), cleanup);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ArenaEntity,
    ));
}

fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<ArenaMap>,
) {
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
