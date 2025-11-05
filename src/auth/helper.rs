use super::handler::AuthHandler;
use crate::IdosClient;
use crate::IdosConfig;
use bevy::prelude::*;

pub fn setup_auth(mut commands: Commands, client: Res<IdosClient>, _config: Res<IdosConfig>) {
    #[cfg(target_arch = "wasm32")]
    let storage_prefix = _config.platform.wasm.storage_prefix.clone();

    #[cfg(not(target_arch = "wasm32"))]
    let storage_prefix = "idos_sdk_".to_string();

    let handler = AuthHandler::new(client.clone(), storage_prefix);
    commands.insert_resource(handler);
}
