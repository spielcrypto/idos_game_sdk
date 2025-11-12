use crate::IdosError;
use bevy::prelude::Message;
/// Data Transfer Objects for Authentication
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "platform")]
    pub platform: String,
    #[serde(rename = "device")]
    pub device: String,
    #[serde(rename = "deviceID")]
    pub device_id: String,
    #[serde(rename = "ip")]
    pub ip: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "Message")]
    pub message: Option<String>,
    #[serde(rename = "AuthContext")]
    pub auth_context: Option<AuthContext>,
    #[serde(rename = "UserInventoryResult")]
    pub user_inventory_result: Option<UserInventoryResult>,
    #[serde(rename = "CustomUserDataResult")]
    pub custom_user_data_result: Option<CustomUserDataResult>,
    #[serde(rename = "PlayerProfile")]
    pub player_profile: Option<PlayerProfile>,
    #[serde(rename = "SettingsForUser")]
    pub settings_for_user: Option<Value>,
    #[serde(rename = "SessionExpiration")]
    pub session_expiration: Option<String>,
    #[serde(rename = "LastLoginTime")]
    pub last_login_time: Option<String>,
    #[serde(rename = "UserName")]
    pub user_name: Option<String>,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub custom_data: Option<Value>,
}

impl AuthResponse {
    pub fn to_user(&self) -> Result<User, IdosError> {
        if let Some(message) = &self.message {
            if !message.eq_ignore_ascii_case("success") {
                return Err(IdosError::Auth(message.clone()));
            }
        }

        let user_id = self.resolve_user_id().ok_or_else(|| {
            IdosError::SerializationError("Missing UserID in auth response".into())
        })?;

        let display_name = self
            .player_profile
            .as_ref()
            .and_then(|profile| profile.display_name.clone())
            .or_else(|| self.user_name.clone());

        let username = self
            .user_name
            .clone()
            .or_else(|| display_name.clone())
            .unwrap_or_else(|| user_id.clone());

        let custom_data = self
            .custom_user_data_result
            .as_ref()
            .and_then(|data| data.data.clone());

        Ok(User {
            id: user_id,
            username,
            display_name,
            custom_data,
        })
    }

    pub fn refresh_token(&self) -> Option<String> {
        self.additional_fields
            .get("RefreshToken")
            .and_then(|value| value.as_str())
            .map(|value| value.to_owned())
    }

    pub fn client_session_ticket(&self) -> Option<String> {
        self.auth_context
            .as_ref()
            .and_then(|ctx| ctx.client_session_ticket.clone())
    }

    pub fn resolve_user_id(&self) -> Option<String> {
        self.auth_context
            .as_ref()
            .and_then(|ctx| ctx.user_id.clone())
            .or_else(|| {
                self.user_inventory_result
                    .as_ref()
                    .and_then(|result| result.user_id.clone())
            })
            .or_else(|| {
                self.custom_user_data_result
                    .as_ref()
                    .and_then(|result| result.user_id.clone())
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    #[serde(rename = "ClientSessionTicket")]
    pub client_session_ticket: Option<String>,
    #[serde(rename = "UserID")]
    pub user_id: Option<String>,
    #[serde(rename = "EntityId")]
    pub entity_id: Option<String>,
    #[serde(rename = "EntityToken")]
    pub entity_token: Option<String>,
    #[serde(rename = "EntityType")]
    pub entity_type: Option<String>,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInventoryResult {
    #[serde(rename = "UserID")]
    pub user_id: Option<String>,
    #[serde(rename = "Inventory")]
    pub inventory: Option<Value>,
    #[serde(rename = "VirtualCurrency")]
    pub virtual_currency: Option<Value>,
    #[serde(rename = "VirtualCurrencyRechargeTimes")]
    pub virtual_currency_recharge_times: Option<Value>,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomUserDataResult {
    #[serde(rename = "UserID")]
    pub user_id: Option<String>,
    #[serde(rename = "DataVersion")]
    pub data_version: Option<i64>,
    #[serde(rename = "Data")]
    pub data: Option<Value>,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshSessionRequest {
    #[serde(rename = "ClientSessionTicket")]
    pub client_session_ticket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestLoginRequest {
    #[serde(rename = "deviceID")]
    pub device_id: String,
    #[serde(rename = "platform")]
    pub platform: String,
    #[serde(rename = "device")]
    pub device: String,
    #[serde(rename = "ip")]
    pub ip: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLoginRequest {
    pub provider: SocialProvider,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SocialProvider {
    Google,
    Facebook,
    Twitter,
    Discord,
    Telegram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletLoginRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
    pub chain: WalletChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WalletChain {
    Ethereum,
    Solana,
    Polygon,
    BinanceSmartChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    #[serde(rename = "PlayerId")]
    pub player_id: Option<String>,
    #[serde(rename = "DisplayName")]
    pub display_name: Option<String>,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
}

/// Authentication events
#[derive(Message, Debug)]
pub enum AuthEvent {
    LoginSuccess(User),
    LoginFailed(String),
    LogoutSuccess,
    TokenRefreshed,
}
