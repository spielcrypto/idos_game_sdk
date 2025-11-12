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
    template_title_id: String,
    title_id: String,
}

impl AuthHandler {
    pub fn new(client: IdosClient, storage_prefix: String) -> IdosResult<Self> {
        let template_title_id = std::env::var("IDOS_TEMPLATE_TITLE_ID").map_err(|_| {
            IdosError::Config("Missing IDOS_TEMPLATE_TITLE_ID environment variable".to_string())
        })?;
        let title_id = std::env::var("IDOS_TITLE_ID").map_err(|_| {
            IdosError::Config("Missing IDOS_TITLE_ID environment variable".to_string())
        })?;

        Ok(Self {
            client,
            storage: Storage::new(storage_prefix),
            template_title_id,
            title_id,
        })
    }

    fn auth_endpoint(&self, action: &str) -> String {
        format!(
            "api/{}/{}/Client/Authentication/{}",
            self.template_title_id, self.title_id, action
        )
    }

    fn default_platform(&self) -> String {
        std::env::var("IDOS_PLATFORM")
            .ok()
            .unwrap_or_else(|| std::env::consts::OS.to_string())
    }

    fn default_device(&self) -> String {
        std::env::var("IDOS_DEVICE")
            .ok()
            .unwrap_or_else(|| format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
    }

    fn default_ip(&self) -> Option<String> {
        std::env::var("IDOS_IP").ok()
    }

    /// Login with email and password
    pub async fn login(&self, email: String, password: String) -> IdosResult<AuthResponse> {
        let request = LoginRequest { email, password };
        let response: AuthResponse = self
            .client
            .post(&self.auth_endpoint("LoginWithEmail"), &request)
            .await?;

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
        let device_id = self.get_device_id().ok_or_else(|| {
            IdosError::Auth("Cannot register account without device ID".to_string())
        })?;
        let request = RegisterRequest {
            email,
            password,
            platform: self.default_platform(),
            device: self.default_device(),
            device_id,
            ip: self.default_ip(),
            user_name: Some(username.clone()),
        };
        let response: AuthResponse = self
            .client
            .post(&self.auth_endpoint("RegisterUserByEmail"), &request)
            .await?;

        // Store tokens and user info
        self.store_auth(&response)?;

        Ok(response)
    }

    /// Login as guest
    pub async fn login_guest(&self) -> IdosResult<AuthResponse> {
        let device_id = self.get_device_id().ok_or_else(|| {
            IdosError::Auth("Cannot create guest account without device ID".to_string())
        })?;

        let request = GuestLoginRequest {
            device_id,
            platform: self.default_platform(),
            device: self.default_device(),
            ip: self.default_ip(),
            user_name: None,
        };
        let response: AuthResponse = self
            .client
            .post(&self.auth_endpoint("LoginWithDeviceID"), &request)
            .await?;

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
        let session_ticket = self
            .storage
            .get(TOKEN_KEY)?
            .ok_or_else(|| IdosError::Auth("No session ticket found".to_string()))?;

        let request = RefreshSessionRequest {
            client_session_ticket: session_ticket.clone(),
        };
        let response: AuthResponse = self
            .client
            .post(&self.auth_endpoint("RefreshSession"), &request)
            .await?;

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
        if let Some(message) = &response.message {
            let serialized = serde_json::to_string(response)
                .unwrap_or_else(|err| format!("<unserializable auth response: {err}>"));

            if message.eq_ignore_ascii_case("success") {
                debug!("Authentication success payload: {}", serialized);
            } else {
                error!(
                    "Authentication failure: {}; response body: {}",
                    message, serialized
                );

                if message.eq_ignore_ascii_case("INCORRECT_EMAIL_OR_PASSWORD") {
                    return Err(IdosError::Auth("Incorrect email or password".to_string()));
                }
                return Err(IdosError::Auth(message.clone()));
            }
        }

        let session_ticket = response.client_session_ticket().ok_or_else(|| {
            IdosError::SerializationError(
                "Missing ClientSessionTicket in auth response".to_string(),
            )
        })?;

        self.storage.set(TOKEN_KEY, &session_ticket)?;

        let refresh_token = response
            .refresh_token()
            .unwrap_or_else(|| session_ticket.clone());
        self.storage.set(REFRESH_TOKEN_KEY, &refresh_token)?;

        let user = response.to_user()?;
        let user_json = serde_json::to_string(&user)?;
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
