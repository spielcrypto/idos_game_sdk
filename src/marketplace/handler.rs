/// Marketplace handler for player-to-player trading
use super::dto::*;
use crate::{IdosClient, IdosError, IdosResult};
use bevy::prelude::Resource;

#[derive(Resource, Clone)]
pub struct MarketplaceHandler {
    client: IdosClient,
    user_id: Option<String>,
    session_ticket: Option<String>,
}

impl MarketplaceHandler {
    pub fn new(client: IdosClient) -> Self {
        Self {
            client,
            user_id: None,
            session_ticket: None,
        }
    }

    /// Set user authentication info (call after login)
    pub fn set_auth(&mut self, user_id: String, session_ticket: String) {
        self.user_id = Some(user_id);
        self.session_ticket = Some(session_ticket);
    }

    /// Clear authentication info (call on logout)
    pub fn clear_auth(&mut self) {
        self.user_id = None;
        self.session_ticket = None;
    }

    fn get_user_id(&self) -> IdosResult<String> {
        self.user_id
            .clone()
            .ok_or_else(|| IdosError::Auth("User not logged in".to_string()))
    }

    fn get_session_ticket(&self) -> IdosResult<String> {
        self.session_ticket
            .clone()
            .ok_or_else(|| IdosError::Auth("No session ticket available".to_string()))
    }

    /// Get marketplace data (WASM and native compatible)
    /// Panel types: GroupedOffers, ActiveOffersByItemID, PlayerActiveOffers, PlayerHistory
    pub async fn get_data(
        &self,
        panel: MarketplacePanel,
        items_per_page: i32,
        continuation_token: Option<String>,
        item_id: Option<String>,
        currency_id: Option<String>,
        sort_order: Option<MarketplaceSortOrder>,
        order_by: Option<MarketplaceOrderBy>,
    ) -> IdosResult<String> {
        let request = MarketplaceGetDataRequest {
            panel,
            title_id: self.client.game_id().to_string(),
            web_app_link: None,
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
            entity_token: None,
            build_key: String::new(),
            items_in_one_page: items_per_page,
            continuation_token,
            item_id,
            currency_id,
            sort_order,
            order_by,
        };

        let endpoint = format!("marketplace/data/{}", panel);
        self.client.post(&endpoint, &request).await
    }

    /// Get grouped offers (all items with offers)
    pub async fn get_grouped_offers(
        &self,
        items_per_page: i32,
        continuation_token: Option<String>,
    ) -> IdosResult<String> {
        self.get_data(
            MarketplacePanel::GroupedOffers,
            items_per_page,
            continuation_token,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// Get offers for a specific item
    pub async fn get_offers_by_item(
        &self,
        item_id: &str,
        items_per_page: i32,
        continuation_token: Option<String>,
        currency_id: Option<String>,
        sort_order: Option<MarketplaceSortOrder>,
        order_by: Option<MarketplaceOrderBy>,
    ) -> IdosResult<String> {
        self.get_data(
            MarketplacePanel::ActiveOffersByItemID,
            items_per_page,
            continuation_token,
            Some(item_id.to_string()),
            currency_id,
            sort_order,
            order_by,
        )
        .await
    }

    /// Get player's active offers
    pub async fn get_player_active_offers(
        &self,
        items_per_page: i32,
        continuation_token: Option<String>,
    ) -> IdosResult<String> {
        self.get_data(
            MarketplacePanel::PlayerActiveOffers,
            items_per_page,
            continuation_token,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// Get player's marketplace history
    pub async fn get_player_history(
        &self,
        items_per_page: i32,
        continuation_token: Option<String>,
    ) -> IdosResult<String> {
        self.get_data(
            MarketplacePanel::PlayerHistory,
            items_per_page,
            continuation_token,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// Perform marketplace action
    pub async fn do_action(
        &self,
        action: MarketplaceAction,
        item_id: Option<String>,
        currency_id: Option<String>,
        price: Option<i32>,
        offer_id: Option<String>,
    ) -> IdosResult<String> {
        let request = MarketplaceActionRequest {
            action,
            title_id: self.client.game_id().to_string(),
            web_app_link: None,
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
            entity_token: None,
            build_key: String::new(),
            currency_id,
            item_id,
            price,
            offer_id,
        };

        let endpoint = format!("marketplace/action/{}", action);
        self.client.post(&endpoint, &request).await
    }

    /// Create a marketplace offer
    pub async fn create_offer(
        &self,
        item_id: &str,
        currency_id: &str,
        price: i32,
    ) -> IdosResult<String> {
        self.do_action(
            MarketplaceAction::CreateOffer,
            Some(item_id.to_string()),
            Some(currency_id.to_string()),
            Some(price),
            None,
        )
        .await
    }

    /// Update an existing offer
    pub async fn update_offer(
        &self,
        offer_id: &str,
        currency_id: &str,
        price: i32,
    ) -> IdosResult<String> {
        self.do_action(
            MarketplaceAction::UpdateOffer,
            None,
            Some(currency_id.to_string()),
            Some(price),
            Some(offer_id.to_string()),
        )
        .await
    }

    /// Delete an offer
    pub async fn delete_offer(&self, offer_id: &str) -> IdosResult<String> {
        self.do_action(
            MarketplaceAction::DeleteOffer,
            None,
            None,
            None,
            Some(offer_id.to_string()),
        )
        .await
    }

    /// Buy an offer
    pub async fn buy_offer(&self, offer_id: &str) -> IdosResult<String> {
        self.do_action(
            MarketplaceAction::BuyOffer,
            None,
            None,
            None,
            Some(offer_id.to_string()),
        )
        .await
    }

    /// Create a demand (buy request)
    pub async fn create_demand(
        &self,
        item_id: &str,
        currency_id: &str,
        price: i32,
    ) -> IdosResult<String> {
        self.do_action(
            MarketplaceAction::CreateDemand,
            Some(item_id.to_string()),
            Some(currency_id.to_string()),
            Some(price),
            None,
        )
        .await
    }
}
