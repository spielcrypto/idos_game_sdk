pub struct IapPlugin;
use bevy::prelude::*;

use super::dto::{Product, PurchaseResponse};
use super::handler::IapHandler;

impl Plugin for IapPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<IapEvent>()
            .add_systems(Startup, setup_iap);
    }
}

fn setup_iap(mut commands: Commands, client: Res<crate::IdosClient>) {
    let handler = IapHandler::new(client.clone());
    commands.insert_resource(handler);
}

#[derive(Message, Debug)]
pub enum IapEvent {
    PurchaseSuccess(PurchaseResponse),
    PurchaseFailed(String),
    ProductsLoaded(Vec<Product>),
}
