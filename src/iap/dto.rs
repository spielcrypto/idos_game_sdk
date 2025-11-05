/// Data Transfer Objects for In-App Purchases
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub currency: String,
    pub product_type: ProductType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProductType {
    Consumable,
    NonConsumable,
    Subscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequest {
    pub product_id: String,
    pub payment_method: PaymentMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    CreditCard,
    Crypto { chain: String, token: String },
    Telegram,
    WebMoney,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseResponse {
    pub transaction_id: Uuid,
    pub status: PurchaseStatus,
    pub payment_url: Option<String>,
    pub product: Product,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PurchaseStatus {
    Pending,
    Completed,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProductsResponse {
    pub products: Vec<Product>,
}
