use super::handler::MarketplaceHandler;
use crate::IdosClient;
/// Marketplace Bevy plugin
use bevy::prelude::*;

pub struct MarketplacePlugin;

impl Plugin for MarketplacePlugin {
    fn build(&self, app: &mut App) {
        // Initialize marketplace handler when client is available
        if let Some(client) = app.world().get_resource::<IdosClient>() {
            let handler = MarketplaceHandler::new(client.clone());
            app.insert_resource(handler);
        }
    }
}
