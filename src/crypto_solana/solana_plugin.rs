use super::{SolanaHandler, SolanaSettings};
use crate::IdosClient;
use bevy::prelude::*;

pub struct SolanaPlugin {
    pub settings: SolanaSettings,
}

impl SolanaPlugin {
    pub fn new(settings: SolanaSettings) -> Self {
        Self { settings }
    }
}

impl Plugin for SolanaPlugin {
    fn build(&self, app: &mut App) {
        // Get the IdosClient resource if it exists
        if let Some(client) = app.world().get_resource::<IdosClient>() {
            let handler = SolanaHandler::new(client.clone(), self.settings.clone());
            app.insert_resource(handler);
        } else {
            warn!("IdosClient not found. SolanaHandler will not be initialized.");
        }

        info!("Solana Wallet Plugin initialized");
    }
}
