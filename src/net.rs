use bevy::prelude::*;

use crate::app_state::{GameState, SessionConfig};

#[derive(Resource, Default)]
pub struct NetSession {
    pub connected: bool,
    pub is_host: bool,
    pub peers: usize,
    pub tick: f32,
}

pub struct NetPlugin;
impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetSession>()
            .add_systems(OnEnter(GameState::InGame), connect)
            .add_systems(Update, sync_stub.run_if(in_state(GameState::InGame)));
    }
}

fn connect(mut net: ResMut<NetSession>, cfg: Res<SessionConfig>) {
    *net = NetSession { connected: true, is_host: cfg.host, peers: 2, tick: 0.0 };
    println!("network loopback connected: host={} addr={}", cfg.host, cfg.addr);
}

fn sync_stub(time: Res<Time>, mut net: ResMut<NetSession>) {
    net.tick += time.delta_secs();
}
