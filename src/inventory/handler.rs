/// Inventory handler for items and virtual currency management
use super::dto::*;
use crate::{IdosClient, IdosError, IdosResult};
use bevy::prelude::Resource;
use std::collections::HashMap;

#[derive(Resource, Clone)]
pub struct InventoryHandler {
    client: IdosClient,
    user_id: Option<String>,
    session_ticket: Option<String>,
    // Cached inventory data
    items: HashMap<String, i32>,            // item_id -> quantity
    virtual_currency: HashMap<String, i32>, // currency_id -> amount
}

impl InventoryHandler {
    pub fn new(client: IdosClient) -> Self {
        Self {
            client,
            user_id: None,
            session_ticket: None,
            items: HashMap::new(),
            virtual_currency: HashMap::new(),
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
        self.items.clear();
        self.virtual_currency.clear();
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

    /// Get user's complete inventory from server
    pub async fn get_inventory(&mut self) -> IdosResult<GetUserInventoryResult> {
        use serde::Serialize;

        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct GetInventoryRequest {
            #[serde(rename = "TitleID")]
            title_id: String,
            build_key: String,
            #[serde(rename = "UserID")]
            user_id: String,
            client_session_ticket: String,
        }

        let request = GetInventoryRequest {
            title_id: self.client.game_id().to_string(),
            build_key: String::new(),
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
        };

        let endpoint = "user-data/inventory";
        let result: GetUserInventoryResult = self.client.post(endpoint, &request).await?;

        // Cache the inventory locally
        self.update_cache(&result);

        Ok(result)
    }

    /// Update local cache from inventory result
    fn update_cache(&mut self, result: &GetUserInventoryResult) {
        self.items.clear();
        self.virtual_currency.clear();

        // Cache items
        for item in &result.inventory {
            let count = self.items.get(&item.item_id).unwrap_or(&0);
            let remaining_uses = item.remaining_uses.unwrap_or(1);
            self.items
                .insert(item.item_id.clone(), count + remaining_uses);
        }

        // Cache virtual currency
        for (currency_id, amount) in &result.virtual_currency {
            self.virtual_currency.insert(currency_id.clone(), *amount);
        }
    }

    /// Get item amount from cache (call get_inventory first to refresh)
    pub fn get_item_amount(&self, item_id: &str) -> i32 {
        *self.items.get(item_id).unwrap_or(&0)
    }

    /// Get virtual currency amount from cache (call get_inventory first to refresh)
    pub fn get_virtual_currency_amount(&self, currency_id: &str) -> i32 {
        *self.virtual_currency.get(currency_id).unwrap_or(&0)
    }

    /// Check if user has specific item
    pub fn has_item(&self, item_id: &str) -> bool {
        self.get_item_amount(item_id) > 0
    }

    /// Check if user has enough virtual currency
    pub fn has_currency(&self, currency_id: &str, amount: i32) -> bool {
        self.get_virtual_currency_amount(currency_id) >= amount
    }

    /// Subtract virtual currency (server-side operation)
    pub async fn subtract_virtual_currency(
        &mut self,
        currency_id: &str,
        amount: i32,
    ) -> IdosResult<String> {
        // Check if player has enough (local check)
        if !self.has_currency(currency_id, amount) {
            return Err(IdosError::InvalidInput(format!(
                "Insufficient currency: {} (have: {}, need: {})",
                currency_id,
                self.get_virtual_currency_amount(currency_id),
                amount
            )));
        }

        let request = SubtractVirtualCurrencyRequest {
            title_id: self.client.game_id().to_string(),
            build_key: String::new(),
            function_name: "SubtractVirtualCurrencyHandler".to_string(),
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
            currency_id: currency_id.to_string(),
            amount,
        };

        let endpoint = "inventory/subtract-currency";
        let response: String = self.client.post(endpoint, &request).await?;

        // Update local cache
        if let Some(current) = self.virtual_currency.get_mut(currency_id) {
            *current -= amount;
        }

        Ok(response)
    }

    /// Grant items to user (server-side operation)
    pub async fn grant_items(
        &mut self,
        item_ids: Vec<String>,
        catalog_version: Option<String>,
    ) -> IdosResult<Vec<ItemInstance>> {
        let request = GrantItemsRequest {
            title_id: self.client.game_id().to_string(),
            build_key: String::new(),
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
            item_ids: item_ids.clone(),
            catalog_version,
        };

        let endpoint = "inventory/grant-items";
        let result: Vec<ItemInstance> = self.client.post(endpoint, &request).await?;

        // Update local cache
        for item_id in item_ids {
            let count = self.items.get(&item_id).unwrap_or(&0);
            self.items.insert(item_id, count + 1);
        }

        Ok(result)
    }

    /// Consume an item (reduce remaining uses or remove)
    pub async fn consume_item(
        &mut self,
        item_instance_id: &str,
        consume_count: i32,
    ) -> IdosResult<ConsumeItemResponse> {
        let request = ConsumeItemRequest {
            title_id: self.client.game_id().to_string(),
            build_key: String::new(),
            user_id: self.get_user_id()?,
            client_session_ticket: self.get_session_ticket()?,
            item_instance_id: item_instance_id.to_string(),
            consume_count,
        };

        let endpoint = "inventory/consume-item";
        let response: ConsumeItemResponse = self.client.post(endpoint, &request).await?;

        // Note: Cache update requires knowing item_id from item_instance_id
        // In production, you'd track instance_id -> item_id mapping

        Ok(response)
    }

    /// Get all cached items
    pub fn get_all_items(&self) -> &HashMap<String, i32> {
        &self.items
    }

    /// Get all cached virtual currencies
    pub fn get_all_currencies(&self) -> &HashMap<String, i32> {
        &self.virtual_currency
    }
}
