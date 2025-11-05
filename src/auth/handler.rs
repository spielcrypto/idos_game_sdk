/// Authentication handler logic
use super::dto::*;
use crate::storage::Storage;
use crate::{IdosClient, IdosError, IdosResult};
use bevy::prelude::*;

const TOKEN_KEY: &str = "auth_token";
const REFRESH_TOKEN_KEY: &str = "auth_refresh_token";
const USER_KEY: &str = "auth_user";

#[derive(Resource, Clone)]
pub struct AuthHandler {
    client: IdosClient,
    storage: Storage,
}

impl AuthHandler {
    pub fn new(client: IdosClient, storage_prefix: String) -> Self {
        Self {
            client,
            storage: Storage::new(storage_prefix),
        }
    }

    /// Login with email and password
    pub async fn login(&self, email: String, password: String) -> IdosResult<AuthResponse> {
        let request = LoginRequest { email, password };
        let response: AuthResponse = self.client.post("auth/login", &request).await?;

        // Store tokens and user info
        self.store_auth(&response)?;

        Ok(response)
    }

    /// Register a new user
    pub async fn register(
        &self,
        email: String,
        password: String,
        username: String,
    ) -> IdosResult<AuthResponse> {
        let request = RegisterRequest {
            email,
            password,
            username,
            device_id: self.get_device_id(),
        };
        let response: AuthResponse = self.client.post("auth/register", &request).await?;

        // Store tokens and user info
        self.store_auth(&response)?;

        Ok(response)
    }

    /// Login as guest
    pub async fn login_guest(&self) -> IdosResult<AuthResponse> {
        let device_id = self.get_device_id().ok_or_else(|| {
            IdosError::Auth("Cannot create guest account without device ID".to_string())
        })?;

        let request = GuestLoginRequest { device_id };
        let response: AuthResponse = self.client.post("auth/guest", &request).await?;

        self.store_auth(&response)?;

        Ok(response)
    }

    /// Login with social provider
    pub async fn login_social(
        &self,
        provider: SocialProvider,
        access_token: String,
    ) -> IdosResult<AuthResponse> {
        #[cfg(target_arch = "wasm32")]
        {
            let request = SocialLoginRequest {
                provider,
                access_token,
            };
            let response: AuthResponse = self.client.post("auth/social", &request).await?;

            self.store_auth(&response)?;

            Ok(response)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (provider, access_token);
            Err(IdosError::PlatformNotSupported(
                "Social login is only supported on WASM/Web".to_string(),
            ))
        }
    }

    /// Login with crypto wallet
    pub async fn login_wallet(
        &self,
        wallet_address: String,
        signature: String,
        message: String,
        chain: WalletChain,
    ) -> IdosResult<AuthResponse> {
        let request = WalletLoginRequest {
            wallet_address,
            signature,
            message,
            chain,
        };
        let response: AuthResponse = self.client.post("auth/wallet", &request).await?;

        self.store_auth(&response)?;

        Ok(response)
    }

    /// Refresh access token
    pub async fn refresh_token(&self) -> IdosResult<AuthResponse> {
        let refresh_token = self
            .storage
            .get(REFRESH_TOKEN_KEY)?
            .ok_or_else(|| IdosError::Auth("No refresh token found".to_string()))?;

        let request = RefreshTokenRequest { refresh_token };
        let response: AuthResponse = self.client.post("auth/refresh", &request).await?;

        self.store_auth(&response)?;

        Ok(response)
    }

    /// Logout
    pub fn logout(&self) -> IdosResult<()> {
        self.storage.remove(TOKEN_KEY)?;
        self.storage.remove(REFRESH_TOKEN_KEY)?;
        self.storage.remove(USER_KEY)?;
        Ok(())
    }

    /// Get current user
    pub fn get_current_user(&self) -> IdosResult<Option<User>> {
        let user_json_opt: Option<String> = self.storage.get(USER_KEY)?;
        if user_json_opt.is_none() {
            return Ok(None);
        }

        let user_json: String = user_json_opt.unwrap();
        let user: User = serde_json::from_str(&user_json)?;
        Ok(Some(user))
    }

    /// Get current auth token
    pub fn get_token(&self) -> IdosResult<Option<String>> {
        self.storage.get(TOKEN_KEY)
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.get_token().ok().flatten().is_some()
    }

    // Helper methods

    fn store_auth(&self, response: &AuthResponse) -> IdosResult<()> {
        self.storage.set(TOKEN_KEY, &response.token)?;
        self.storage
            .set(REFRESH_TOKEN_KEY, &response.refresh_token)?;

        let user_json = serde_json::to_string(&response.user)?;
        self.storage.set(USER_KEY, &user_json)?;

        Ok(())
    }

    fn get_device_id(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            use uuid::Uuid;
            // Try to get from storage first
            if let Ok(Some(device_id)) = self.storage.get("device_id") {
                return Some(device_id);
            }

            // Generate new one
            let device_id = Uuid::new_v4().to_string();
            self.storage.set("device_id", &device_id).ok()?;
            Some(device_id)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Some(uuid::Uuid::new_v4().to_string())
        }
    }
}
