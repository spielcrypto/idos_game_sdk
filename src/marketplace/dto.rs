/// Data Transfer Objects for Marketplace
use serde::{Deserialize, Serialize};

/// Marketplace panel types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketplacePanel {
    GroupedOffers,
    ActiveOffersByItemID,
    PlayerActiveOffers,
    PlayerHistory,
}

impl std::fmt::Display for MarketplacePanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketplacePanel::GroupedOffers => write!(f, "GroupedOffers"),
            MarketplacePanel::ActiveOffersByItemID => write!(f, "ActiveOffersByItemID"),
            MarketplacePanel::PlayerActiveOffers => write!(f, "PlayerActiveOffers"),
            MarketplacePanel::PlayerHistory => write!(f, "PlayerHistory"),
        }
    }
}

/// Marketplace actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketplaceAction {
    CreateOffer,
    CreateDemand,
    UpdateOffer,
    DeleteOffer,
    BuyOffer,
}

impl std::fmt::Display for MarketplaceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketplaceAction::CreateOffer => write!(f, "CreateOffer"),
            MarketplaceAction::CreateDemand => write!(f, "CreateDemand"),
            MarketplaceAction::UpdateOffer => write!(f, "UpdateOffer"),
            MarketplaceAction::DeleteOffer => write!(f, "DeleteOffer"),
            MarketplaceAction::BuyOffer => write!(f, "BuyOffer"),
        }
    }
}

/// Sort order for marketplace results
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MarketplaceSortOrder {
    #[serde(rename = "ASC")]
    Asc,
    #[serde(rename = "DESC")]
    Desc,
}

/// Order by field
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketplaceOrderBy {
    Date,
    Price,
}

/// Active marketplace offer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MarketplaceActiveOffer {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "ItemID")]
    pub item_id: String,
    #[serde(rename = "SellerID")]
    pub seller_id: String,
    #[serde(rename = "CurrencyID")]
    pub currency_id: String,
    pub price: f64,
}

/// Grouped offer (multiple offers for same item)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MarketplaceGroupedOffer {
    #[serde(rename = "ItemID")]
    pub item_id: String,
    pub offer_count: i32,
}

/// Request to get marketplace data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MarketplaceGetDataRequest {
    #[serde(rename = "MarketplacePanel")]
    pub panel: MarketplacePanel,
    #[serde(rename = "TitleID")]
    pub title_id: String,
    pub web_app_link: Option<String>,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub client_session_ticket: String,
    pub entity_token: Option<String>,
    pub build_key: String,
    #[serde(rename = "MaxItemCount")]
    pub items_in_one_page: i32,
    pub continuation_token: Option<String>,
    #[serde(rename = "ItemID")]
    pub item_id: Option<String>,
    #[serde(rename = "VirtualCurrencyID")]
    pub currency_id: Option<String>,
    pub sort_order: Option<MarketplaceSortOrder>,
    pub order_by: Option<MarketplaceOrderBy>,
}

/// Request to perform marketplace action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MarketplaceActionRequest {
    #[serde(rename = "MarketplaceAction")]
    pub action: MarketplaceAction,
    #[serde(rename = "TitleID")]
    pub title_id: String,
    pub web_app_link: Option<String>,
    #[serde(rename = "UserID")]
    pub user_id: String,
    pub client_session_ticket: String,
    pub entity_token: Option<String>,
    pub build_key: String,
    #[serde(rename = "VirtualCurrencyID")]
    pub currency_id: Option<String>,
    #[serde(rename = "ItemID")]
    pub item_id: Option<String>,
    pub price: Option<i32>,
    #[serde(rename = "ID")]
    pub offer_id: Option<String>,
}

/// Response for marketplace data request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceDataResponse {
    #[serde(rename = "ContinuationToken")]
    pub continuation_token: Option<String>,
    #[serde(rename = "Data")]
    pub data: serde_json::Value,
}

/// Response for marketplace action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceActionResponse {
    #[serde(rename = "Message")]
    pub message: Option<String>,
    #[serde(rename = "Success")]
    pub success: bool,
}

/// Marketplace commission configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceCommission {
    #[serde(rename = "Company")]
    pub company: i32,
    #[serde(rename = "Referral")]
    pub referral: i32,
    #[serde(rename = "Author")]
    pub author: i32,
}

impl MarketplaceCommission {
    pub fn total(&self) -> i32 {
        self.company + self.referral + self.author
    }

    pub fn calculate_player_receives(&self, price: i32) -> i32 {
        let total_commission_percent = self.total();
        let commission_amount = (price * total_commission_percent) / 100;
        price - commission_amount
    }
}
