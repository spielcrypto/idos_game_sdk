use bevy::prelude::Message;
/// Data Transfer Objects for Authentication
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub username: String,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: i64,
    pub last_login_at: i64,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestLoginRequest {
    pub device_id: String,
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

/// Authentication events
#[derive(Message, Debug)]
pub enum AuthEvent {
    LoginSuccess(User),
    LoginFailed(String),
    LogoutSuccess,
    TokenRefreshed,
}
