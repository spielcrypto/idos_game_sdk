/// Solana wallet handler - WASM compatible
use super::dto::*;
use crate::{IdosClient, IdosError, IdosResult};
use bevy::prelude::Resource;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use super::helper::{
    is_solana_wallet_available, solana_connect_wallet, solana_deposit_spl, solana_get_balance,
    solana_get_token_balance, solana_get_transaction, solana_send_transaction, solana_withdraw_spl,
};

#[derive(Resource, Clone)]
pub struct SolanaHandler {
    client: IdosClient,
    settings: SolanaSettings,
}

impl SolanaHandler {
    pub fn new(client: IdosClient, settings: SolanaSettings) -> Self {
        Self { client, settings }
    }

    /// Get Solana settings
    pub fn settings(&self) -> &SolanaSettings {
        &self.settings
    }

    /// Check if Phantom/Solflare wallet is available (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub fn is_wallet_available(&self) -> bool {
        is_solana_wallet_available()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_wallet_available(&self) -> bool {
        false // Native wallet support would need solana-sdk
    }

    /// Get SOL balance for a wallet address
    pub async fn get_balance(&self, address: &str) -> IdosResult<u64> {
        #[cfg(target_arch = "wasm32")]
        {
            solana_get_balance(&self.settings.rpc_url, address).await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native implementation would use solana_client
            let _ = address;
            Err(IdosError::PlatformNotSupported(
                "Native Solana support requires solana-client crate".to_string(),
            ))
        }
    }

    /// Get SPL token balance
    pub async fn get_token_balance(
        &self,
        wallet_address: &str,
        mint_address: &str,
    ) -> IdosResult<TokenAmount> {
        #[cfg(target_arch = "wasm32")]
        {
            solana_get_token_balance(&self.settings.rpc_url, wallet_address, mint_address).await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (wallet_address, mint_address);
            Err(IdosError::PlatformNotSupported(
                "Native Solana support requires solana-client crate".to_string(),
            ))
        }
    }

    /// Connect wallet (WASM only - Phantom/Solflare)
    #[cfg(target_arch = "wasm32")]
    pub async fn connect_wallet(&self) -> IdosResult<String> {
        solana_connect_wallet().await
    }

    /// Request withdrawal signature from backend
    pub async fn get_withdrawal_signature(
        &self,
        mint: &str,
        amount: u64,
        wallet_address: &str,
    ) -> IdosResult<ServerWithdrawPayload> {
        let request = PlatformPoolTransactionRequest {
            transaction_type: "Token".to_string(),
            direction: "UsersCryptoWallet".to_string(),
            transaction_hash: None,
            currency_id: Some(mint.to_string()),
            amount: Some(amount),
            wallet_address: wallet_address.to_string(),
        };

        self.client
            .post("solana/withdraw-signature", &request)
            .await
    }

    /// Submit deposit transaction to backend
    pub async fn submit_deposit(
        &self,
        transaction_signature: &str,
        mint: &str,
        amount: u64,
    ) -> IdosResult<String> {
        let request = PlatformPoolTransactionRequest {
            transaction_type: "Token".to_string(),
            direction: "Game".to_string(),
            transaction_hash: Some(transaction_signature.to_string()),
            currency_id: Some(mint.to_string()),
            amount: Some(amount),
            wallet_address: String::new(),
        };

        self.client.post("solana/deposit", &request).await
    }

    /// Submit withdrawal transaction to backend
    pub async fn submit_withdrawal(&self, transaction_signature: &str) -> IdosResult<String> {
        let request = PlatformPoolTransactionRequest {
            transaction_type: "Token".to_string(),
            direction: "UsersCryptoWallet".to_string(),
            transaction_hash: Some(transaction_signature.to_string()),
            currency_id: None,
            amount: None,
            wallet_address: String::new(),
        };

        self.client.post("solana/withdrawal", &request).await
    }

    /// Send transaction (WASM - via wallet adapter)
    #[cfg(target_arch = "wasm32")]
    pub async fn send_transaction(&self, transaction_base64: &str) -> IdosResult<String> {
        solana_send_transaction(transaction_base64).await
    }

    /// Sign and send deposit transaction (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub async fn deposit_spl_token(
        &self,
        mint: &str,
        amount: u64,
        user_id: &str,
    ) -> IdosResult<String> {
        solana_deposit_spl(
            &self.settings.rpc_url,
            &self.settings.program_id,
            mint,
            amount,
            user_id,
        )
        .await
    }

    /// Sign and send withdrawal transaction (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub async fn withdraw_spl_token(
        &self,
        withdraw_request: WithdrawSplRequest,
    ) -> IdosResult<String> {
        solana_withdraw_spl(
            &self.settings.rpc_url,
            &self.settings.program_id,
            withdraw_request,
        )
        .await
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, signature: &str) -> IdosResult<TransactionResult> {
        #[cfg(target_arch = "wasm32")]
        {
            solana_get_transaction(&self.settings.rpc_url, signature).await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = signature;
            Err(IdosError::PlatformNotSupported(
                "Native Solana support requires solana-client crate".to_string(),
            ))
        }
    }

    /// Wait for transaction confirmation
    pub async fn confirm_transaction(
        &self,
        signature: &str,
        max_attempts: u32,
    ) -> IdosResult<bool> {
        for _ in 0..max_attempts {
            match self.get_transaction_status(signature).await {
                Ok(result) => {
                    if result.confirmed {
                        return Ok(true);
                    }
                }
                Err(_) => {
                    // Transaction not found yet, continue waiting
                }
            }

            #[cfg(target_arch = "wasm32")]
            {
                // Wait 2 seconds
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    let window = web_sys::window().unwrap();
                    window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 2000)
                        .ok();
                });
                wasm_bindgen_futures::JsFuture::from(promise).await.ok();
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }

        Err(IdosError::TimeoutError(
            "Transaction not confirmed".to_string(),
        ))
    }

    /// Convert lamports to SOL
    pub fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / 1_000_000_000.0
    }

    /// Convert SOL to lamports
    pub fn sol_to_lamports(sol: f64) -> u64 {
        (sol * 1_000_000_000.0) as u64
    }

    /// Calculate token amount with decimals
    pub fn calculate_token_amount(amount: u64, decimals: u8) -> f64 {
        amount as f64 / 10_f64.powi(decimals as i32)
    }

    /// Load all NFTs owned by a wallet address
    /// Uses Metaplex Token Metadata to fetch NFT data
    pub async fn load_nfts(&self, owner_address: &str) -> IdosResult<NftLoadResult> {
        super::nft::load_nfts_by_owner(&self.settings.rpc_url, owner_address).await
    }

    /// Load metadata for a specific NFT mint
    pub async fn load_nft(&self, mint_address: &str, owner_address: &str) -> IdosResult<Nft> {
        super::nft::load_nft_metadata(&self.settings.rpc_url, mint_address, owner_address).await
    }
}
