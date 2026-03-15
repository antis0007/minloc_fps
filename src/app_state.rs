use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}

#[derive(Resource)]
pub struct SessionConfig {
    pub name: String,
    pub host: bool,
    pub addr: String,
    pub selected_weapon: usize,
    pub map_path: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            name: "Player".into(),
            host: true,
            addr: "127.0.0.1:5000".into(),
            selected_weapon: 3,
            map_path: "maps/arena.ron".into(),
        }
    }
}

pub struct AppStatePlugin;
impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().init_resource::<SessionConfig>();
    }
}