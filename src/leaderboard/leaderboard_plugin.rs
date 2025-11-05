/// Leaderboard Bevy plugin
use super::handler::LeaderboardHandler;
use crate::IdosClient;
use bevy::prelude::*;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        // Initialize leaderboard handler when client is available
        if let Some(client) = app.world().get_resource::<IdosClient>() {
            let handler = LeaderboardHandler::new(client.clone());
            app.insert_resource(handler);
        }
    }
}
