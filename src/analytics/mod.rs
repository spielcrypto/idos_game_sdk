/// Analytics module - track events and user behavior
pub mod dto;
pub mod handler;

use bevy::prelude::*;
use handler::AnalyticsHandler;

pub use dto::*;

pub struct AnalyticsPlugin;

impl Plugin for AnalyticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_analytics);
    }
}

fn setup_analytics(
    mut commands: Commands,
    client: Res<crate::IdosClient>,
    config: Res<crate::IdosConfig>,
) {
    let handler = AnalyticsHandler::new(client.clone(), config.enable_analytics);

    // Track session start
    #[cfg(target_arch = "wasm32")]
    {
        let h = handler.clone();
        wasm_bindgen_futures::spawn_local(async move {
            h.track_session_start().await.ok();
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let h = handler.clone();
        // Try to use existing runtime, otherwise spawn thread with new runtime
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                h.track_session_start().await.ok();
            });
        } else {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    h.track_session_start().await.ok();
                });
            });
        }
    }

    commands.insert_resource(handler);
}
