/// HTTP client for API requests - WASM compatible
use crate::{IdosConfig, IdosError, IdosResult};
use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Resource, Clone)]
pub struct IdosClient {
    http_client: reqwest::Client,
    config: IdosConfig,
}

impl IdosClient {
    pub fn new(config: IdosConfig) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        #[cfg(target_arch = "wasm32")]
        let http_client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            config,
        }
    }

    /// Make a GET request
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> IdosResult<T> {
        let url = format!("{}/{}", self.config.api_url, endpoint);

        if self.config.debug {
            info!("GET {}", url);
        }

        let response = self
            .http_client
            .get(&url)
            .header("X-API-Key", &self.config.api_key)
            .header("X-Game-ID", &self.config.game_id)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(IdosError::Api(format!(
                "HTTP {} for {}",
                response.status(),
                url
            )));
        }

        Ok(response.json().await?)
    }

    /// Make a POST request
    pub async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> IdosResult<R> {
        let url = format!("{}/{}", self.config.api_url, endpoint);

        if self.config.debug {
            info!("POST {}", url);
        }

        let response = self
            .http_client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .header("X-Game-ID", &self.config.game_id)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "<unreadable body>".to_string());
            error!(
                "POST {} failed with status {}. Body: {}",
                url, status, text
            );
            return Err(IdosError::Api(format!("HTTP {} for {}", status, url)));
        }

        let bytes = response.bytes().await?;
        if self.config.debug {
            debug!(
                "POST {} responded with: {}",
                url,
                String::from_utf8_lossy(&bytes)
            );
        }

        serde_json::from_slice(&bytes).map_err(|err| {
            error!(
                "Failed to deserialize POST {} response: {}. Body: {}",
                url,
                err,
                String::from_utf8_lossy(&bytes)
            );
            IdosError::SerializationError(format!(
                "Failed to decode response from {}: {}",
                url, err
            ))
        })
    }

    /// Make a PUT request
    pub async fn put<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> IdosResult<R> {
        let url = format!("{}/{}", self.config.api_url, endpoint);

        if self.config.debug {
            info!("PUT {}", url);
        }

        let response = self
            .http_client
            .put(&url)
            .header("X-API-Key", &self.config.api_key)
            .header("X-Game-ID", &self.config.game_id)
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(IdosError::Api(format!(
                "HTTP {} for {}",
                response.status(),
                url
            )));
        }

        Ok(response.json().await?)
    }

    /// Make a DELETE request
    pub async fn delete<R: DeserializeOwned>(&self, endpoint: &str) -> IdosResult<R> {
        let url = format!("{}/{}", self.config.api_url, endpoint);

        if self.config.debug {
            info!("DELETE {}", url);
        }

        let response = self
            .http_client
            .delete(&url)
            .header("X-API-Key", &self.config.api_key)
            .header("X-Game-ID", &self.config.game_id)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(IdosError::Api(format!(
                "HTTP {} for {}",
                response.status(),
                url
            )));
        }

        Ok(response.json().await?)
    }

    /// Get the game ID from config
    pub fn game_id(&self) -> &str {
        &self.config.game_id
    }

    /// Get the API key from config
    pub fn api_key(&self) -> &str {
        &self.config.api_key
    }

    /// Get the full config
    pub fn config(&self) -> &IdosConfig {
        &self.config
    }
}
