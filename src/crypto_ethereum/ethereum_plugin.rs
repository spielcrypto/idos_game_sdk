use super::{BlockchainSettings, EthereumHandler};
use crate::IdosClient;
use bevy::prelude::*;

pub struct EthereumPlugin {
    pub settings: BlockchainSettings,
}

impl EthereumPlugin {
    pub fn new(settings: BlockchainSettings) -> Self {
        Self { settings }
    }
}

impl Plugin for EthereumPlugin {
    fn build(&self, app: &mut App) {
        // Get the IdosClient resource if it exists
        if let Some(client) = app.world().get_resource::<IdosClient>() {
            let handler = EthereumHandler::new(client.clone(), self.settings.clone());
            app.insert_resource(handler);
        } else {
            warn!("IdosClient not found. EthereumHandler will not be initialized.");
        }

        info!("Ethereum Wallet Plugin initialized");
    }
}
