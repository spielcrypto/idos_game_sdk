use super::dto::AuthEvent;
use super::helper::setup_auth;
/// Authentication plugin
use bevy::prelude::*;

pub struct AuthPlugin;

impl Plugin for AuthPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AuthEvent>()
            .add_systems(Startup, setup_auth);
    }
}
