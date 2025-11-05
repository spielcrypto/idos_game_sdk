/// Inventory Bevy plugin
use super::handler::InventoryHandler;
use crate::IdosClient;
use bevy::prelude::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        // Initialize inventory handler when client is available
        if let Some(client) = app.world().get_resource::<IdosClient>() {
            let handler = InventoryHandler::new(client.clone());
            app.insert_resource(handler);
        }
    }
}
