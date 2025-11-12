use super::handler::AuthHandler;
use crate::IdosClient;
use crate::IdosConfig;
use bevy::log::error;
use bevy::prelude::*;

pub fn setup_auth(mut commands: Commands, client: Res<IdosClient>, _config: Res<IdosConfig>) {
    #[cfg(target_arch = "wasm32")]
    let storage_prefix = _config.platform.wasm.storage_prefix.clone();

    #[cfg(not(target_arch = "wasm32"))]
    let storage_prefix = "idos_sdk_".to_string();

    match AuthHandler::new(client.clone(), storage_prefix) {
        Ok(handler) => {
            commands.insert_resource(handler);
        }
        Err(err) => {
            error!("Failed to initialize AuthHandler: {err}");
        }
    }
}
