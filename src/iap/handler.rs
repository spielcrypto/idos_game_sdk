/// In-App Purchase handler
use super::dto::*;
use crate::{IdosClient, IdosResult};
use bevy::prelude::Resource;

#[derive(Resource, Clone)]
pub struct IapHandler {
    client: IdosClient,
}

impl IapHandler {
    pub fn new(client: IdosClient) -> Self {
        Self { client }
    }

    /// Get available products
    pub async fn get_products(&self) -> IdosResult<Vec<Product>> {
        let response: GetProductsResponse = self.client.get("iap/products").await?;
        Ok(response.products)
    }

    /// Purchase a product
    pub async fn purchase(
        &self,
        product_id: String,
        payment_method: PaymentMethod,
    ) -> IdosResult<PurchaseResponse> {
        let request = PurchaseRequest {
            product_id,
            payment_method,
        };

        let response: PurchaseResponse = self.client.post("iap/purchase", &request).await?;

        // On web, open payment URL if provided
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(payment_url) = &response.payment_url {
                if let Some(window) = web_sys::window() {
                    window.open_with_url(payment_url).ok();
                }
            }
        }

        Ok(response)
    }

    /// Restore purchases (mainly for mobile/native)
    pub async fn restore_purchases(&self) -> IdosResult<Vec<PurchaseResponse>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let response: Vec<PurchaseResponse> = self.client.get("iap/restore").await?;
            Ok(response)
        }

        #[cfg(target_arch = "wasm32")]
        {
            use crate::IdosError;
            Err(IdosError::PlatformNotSupported(
                "Restore purchases is not supported on web".to_string(),
            ))
        }
    }
}
