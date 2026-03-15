use anyhow::Result;
use bevy::prelude::*;
use serde::Deserialize;

use crate::app_state::{GameState, SessionConfig};

#[derive(Deserialize, Clone, Default, Resource)]
pub struct ArenaMap {
    pub spawn_points: Vec<SpawnPoint>,
    pub solids: Vec<Solid>,
}

#[derive(Deserialize, Clone)]
pub struct SpawnPoint {
    pub pos: [f32; 3],
}

#[derive(Deserialize, Clone)]
pub struct Solid {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(fallback_map())
            .add_systems(OnEnter(GameState::InGame), load_map);
    }
}

fn load_map(mut map: ResMut<ArenaMap>, cfg: Res<SessionConfig>) {
    *map = read_map(&cfg.map_path).unwrap_or_else(|_| fallback_map());
}

fn read_map(path: &str) -> Result<ArenaMap> {
    Ok(ron::from_str(&std::fs::read_to_string(path)?)?)
}

fn fallback_map() -> ArenaMap {
    ArenaMap {
        spawn_points: vec![SpawnPoint { pos: [0.0, 1.5, 0.0] }, SpawnPoint { pos: [8.0, 1.5, 0.0] }],
        solids: vec![
            Solid { min: [-12.0, 0.0, -12.0], max: [12.0, 1.0, 12.0] },
            Solid { min: [-12.0, 0.0, -12.0], max: [-11.0, 4.0, 12.0] },
            Solid { min: [11.0, 0.0, -12.0], max: [12.0, 4.0, 12.0] },
            Solid { min: [-12.0, 0.0, -12.0], max: [12.0, 4.0, -11.0] },
            Solid { min: [-12.0, 0.0, 11.0], max: [12.0, 4.0, 12.0] },
        ],
    }
}
