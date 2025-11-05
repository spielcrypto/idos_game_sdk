/// Basic usage example of iDos Games SDK with Bevy
use bevy::prelude::*;
use idos_game_sdk::{IdosConfig, IdosGamesPlugin};

#[cfg(feature = "auth")]
use idos_game_sdk::auth::{dto::AuthEvent, handler::AuthHandler};

#[cfg(feature = "analytics")]
use idos_game_sdk::analytics::handler::AnalyticsHandler;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "iDos Games SDK Example".to_string(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(IdosGamesPlugin::new(IdosConfig {
            api_key: "your_api_key_here".to_string(),
            game_id: "your_game_id_here".to_string(),
            debug: true,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, listen_auth_events))
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a camera
    commands.spawn(Camera2d);

    // Spawn some UI text
    commands.spawn((
        Text::new(
            "iDos Games SDK Example\nPress L to login as guest\nPress A to track analytics event",
        ),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            ..default()
        },
    ));

    info!("Example started!");
}

#[cfg(feature = "auth")]
fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    auth: Option<Res<AuthHandler>>,
    analytics: Option<Res<AnalyticsHandler>>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        if let Some(auth) = auth {
            info!("Attempting guest login...");

            #[cfg(target_arch = "wasm32")]
            {
                let auth = auth.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match auth.login_guest().await {
                        Ok(response) => {
                            info!("Guest login successful! User: {:?}", response.user);
                        }
                        Err(e) => error!("Guest login failed: {}", e),
                    }
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let auth = auth.clone();
                tokio::spawn(async move {
                    match auth.login_guest().await {
                        Ok(response) => {
                            info!("Guest login successful! User: {:?}", response.user);
                        }
                        Err(e) => error!("Guest login failed: {}", e),
                    }
                });
            }
        }
    }

    #[cfg(feature = "analytics")]
    if keyboard.just_pressed(KeyCode::KeyA) {
        if let Some(analytics) = analytics {
            info!("Tracking analytics event...");

            let mut props = std::collections::HashMap::new();
            props.insert("example".to_string(), serde_json::json!("button_pressed"));

            #[cfg(target_arch = "wasm32")]
            {
                let analytics = analytics.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    analytics.track_event("test_event", props).await.ok();
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let analytics = analytics.clone();
                tokio::spawn(async move {
                    analytics.track_event("test_event", props).await.ok();
                });
            }
        }
    }
}

#[cfg(not(feature = "auth"))]
fn handle_input() {
    // No-op if auth feature is disabled
}

#[cfg(feature = "auth")]
fn listen_auth_events(mut events: MessageReader<AuthEvent>) {
    for event in events.read() {
        match event {
            AuthEvent::LoginSuccess(user) => {
                info!("Login successful! Welcome, {}!", user.username);
            }
            AuthEvent::LoginFailed(error) => {
                error!("Login failed: {}", error);
            }
            AuthEvent::LogoutSuccess => {
                info!("Logged out successfully");
            }
            AuthEvent::TokenRefreshed => {
                info!("Token refreshed");
            }
        }
    }
}

#[cfg(not(feature = "auth"))]
fn listen_auth_events() {
    // No-op if auth feature is disabled
}
