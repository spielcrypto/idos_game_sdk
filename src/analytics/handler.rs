/// Analytics handler - tracks events and user behavior
use super::dto::*;
use crate::{IdosClient, IdosResult};
use bevy::prelude::Resource;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Resource, Clone)]
pub struct AnalyticsHandler {
    client: IdosClient,
    session_id: String,
    enabled: bool,
}

impl AnalyticsHandler {
    pub fn new(client: IdosClient, enabled: bool) -> Self {
        Self {
            client,
            session_id: Uuid::new_v4().to_string(),
            enabled,
        }
    }

    /// Track a custom event
    pub async fn track_event(
        &self,
        event_name: impl Into<String>,
        properties: HashMap<String, serde_json::Value>,
    ) -> IdosResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = AnalyticsEvent {
            event_name: event_name.into(),
            properties,
            timestamp: chrono::Utc::now().timestamp(),
            session_id: self.session_id.clone(),
        };

        // Fire and forget - don't wait for response
        let client = self.client.clone();
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let _: Result<serde_json::Value, _> = client.post("analytics/event", &event).await;
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Try to use existing runtime, otherwise spawn thread with new runtime
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let _: Result<serde_json::Value, _> =
                        client.post("analytics/event", &event).await;
                });
            } else {
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        let _: Result<serde_json::Value, _> =
                            client.post("analytics/event", &event).await;
                    });
                });
            }
        }

        Ok(())
    }

    /// Track session start
    pub async fn track_session_start(&self) -> IdosResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let device_info = self.get_device_info();
        let event = SessionStartEvent {
            platform: self.get_platform_name(),
            device_info,
        };

        let client = self.client.clone();
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let _: Result<serde_json::Value, _> =
                    client.post("analytics/session/start", &event).await;
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Try to use existing runtime, otherwise spawn thread with new runtime
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let _: Result<serde_json::Value, _> =
                        client.post("analytics/session/start", &event).await;
                });
            } else {
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        let _: Result<serde_json::Value, _> =
                            client.post("analytics/session/start", &event).await;
                    });
                });
            }
        }

        Ok(())
    }

    fn get_platform_name(&self) -> String {
        #[cfg(target_arch = "wasm32")]
        {
            "web".to_string()
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            std::env::consts::OS.to_string()
        }
    }

    fn get_device_info(&self) -> DeviceInfo {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;

            let mut info = DeviceInfo {
                platform: "web".to_string(),
                browser: None,
                os: None,
                screen_resolution: None,
                language: None,
            };

            if let Some(window) = window() {
                if let Some(navigator) = window.navigator().user_agent().ok() {
                    info.browser = Some(navigator);
                }

                if let Some(screen) = window.screen().ok() {
                    info.screen_resolution = Some(format!(
                        "{}x{}",
                        screen.width().unwrap_or(0),
                        screen.height().unwrap_or(0)
                    ));
                }

                info.language = window.navigator().language();
            }

            info
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            DeviceInfo {
                platform: std::env::consts::OS.to_string(),
                browser: None,
                os: Some(std::env::consts::OS.to_string()),
                screen_resolution: None,
                language: None,
            }
        }
    }
}
