/// Data Transfer Objects for Inventory
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User inventory result from server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetUserInventoryResult {
    pub inventory: Vec<ItemInstance>,
    pub virtual_currency: HashMap<String, i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_currency_recharge_times: Option<HashMap<String, VirtualCurrencyRechargeTime>>,
}

/// Item instance in player's inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemInstance {
    #[serde(rename = "ItemId")]
    pub item_id: String,
    #[serde(rename = "ItemInstanceId")]
    pub item_instance_id: Option<String>,
    pub display_name: Option<String>,
    pub item_class: Option<String>,
    pub catalog_version: Option<String>,
    pub remaining_uses: Option<i32>,
    pub uses_incremented_by: Option<i32>,
    pub annotation: Option<String>,
    pub bundle_contents: Option<Vec<String>>,
    pub bundle_parent: Option<String>,
    pub custom_data: Option<String>,
    pub expiration: Option<String>,
    pub purchase_date: Option<String>,
    pub unit_currency: Option<String>,
    pub unit_price: Option<u32>,
}

/// Virtual currency recharge time
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VirtualCurrencyRechargeTime {
    pub recharge_max: i32,
    pub recharge_time: String,
    pub seconds_to_recharge: i32,
}

/// Request to subtract virtual currency
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SubtractVirtualCurrencyRequest {
    #[serde(rename = "TitleID")]
    pub title_id: String,
    pub build_key: String,
    pub function_name: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub client_session_ticket: String,
    #[serde(rename = "CurrencyID")]
    pub currency_id: String,
    pub amount: i32,
}

/// Request to grant items to user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GrantItemsRequest {
    #[serde(rename = "TitleID")]
    pub title_id: String,
    pub build_key: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub client_session_ticket: String,
    pub item_ids: Vec<String>,
    pub catalog_version: Option<String>,
}

/// Request to consume an item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConsumeItemRequest {
    #[serde(rename = "TitleID")]
    pub title_id: String,
    pub build_key: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub client_session_ticket: String,
    #[serde(rename = "ItemInstanceId")]
    pub item_instance_id: String,
    pub consume_count: i32,
}

/// Response after consuming an item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConsumeItemResponse {
    #[serde(rename = "ItemInstanceId")]
    pub item_instance_id: String,
    pub remaining_uses: i32,
}
