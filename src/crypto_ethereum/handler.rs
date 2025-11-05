/// Ethereum wallet handler - WASM compatible
use super::dto::*;
use crate::{IdosClient, IdosError, IdosResult};
use bevy::prelude::Resource;

#[cfg(not(target_arch = "wasm32"))]
use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::{Address, Bytes},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[cfg(target_arch = "wasm32")]
use super::helper::{
    eth_call_allowance, eth_call_balance_of, eth_get_balance, eth_get_transaction_receipt,
};

#[derive(Resource, Clone)]
pub struct EthereumHandler {
    client: IdosClient,
    settings: BlockchainSettings,
    #[cfg(not(target_arch = "wasm32"))]
    provider: Option<Provider<Http>>,
}

impl EthereumHandler {
    pub fn new(client: IdosClient, settings: BlockchainSettings) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let provider = Provider::<Http>::try_from(&settings.rpc_url).ok();

        Self {
            client,
            settings,
            #[cfg(not(target_arch = "wasm32"))]
            provider,
        }
    }

    /// Get blockchain settings
    pub fn settings(&self) -> &BlockchainSettings {
        &self.settings
    }

    /// Check if MetaMask is available (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub fn is_metamask_available(&self) -> bool {
        if let Some(win) = window() {
            js_sys::Reflect::has(&win, &JsValue::from_str("ethereum")).unwrap_or(false)
        } else {
            false
        }
    }

    /// Check if wallet is ready
    pub fn is_wallet_ready(&self, wallet_address: Option<&str>) -> bool {
        wallet_address.map_or(false, |addr| !addr.is_empty())
    }

    /// Get native token balance (ETH, MATIC, BNB, etc.)
    pub async fn get_native_balance(&self, wallet_address: &str) -> IdosResult<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(provider) = &self.provider {
                let address: Address = wallet_address
                    .parse()
                    .map_err(|_| IdosError::InvalidInput("Invalid wallet address".to_string()))?;

                let balance = provider
                    .get_balance(address, None)
                    .await
                    .map_err(|e| IdosError::NetworkError(e.to_string()))?;

                Ok(balance.to_string())
            } else {
                Err(IdosError::ConfigurationError(
                    "Provider not initialized".to_string(),
                ))
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            eth_get_balance(&self.settings.rpc_url, wallet_address).await
        }
    }

    /// Get ERC20 token balance
    pub async fn get_erc20_balance(
        &self,
        wallet_address: &str,
        token_address: &str,
    ) -> IdosResult<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(provider) = &self.provider {
                let _wallet: Address = wallet_address
                    .parse()
                    .map_err(|_| IdosError::InvalidInput("Invalid wallet address".to_string()))?;
                let token: Address = token_address
                    .parse()
                    .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;

                // ERC20 balanceOf selector: 0x70a08231
                let selector = "0x70a08231";
                let encoded_address = format!("{:0>64}", wallet_address.trim_start_matches("0x"));
                let data = format!("{}{}", selector, encoded_address);

                let call_data = ethers::types::transaction::eip2718::TypedTransaction::Legacy(
                    ethers::types::TransactionRequest {
                        to: Some(ethers::types::NameOrAddress::Address(token)),
                        data: Some(Bytes::from(
                            hex::decode(data.trim_start_matches("0x")).unwrap(),
                        )),
                        ..Default::default()
                    },
                );

                let result = provider
                    .call(&call_data, None)
                    .await
                    .map_err(|e| IdosError::NetworkError(e.to_string()))?;

                let balance = U256::from_big_endian(result.as_ref());
                Ok(balance.to_string())
            } else {
                Err(IdosError::ConfigurationError(
                    "Provider not initialized".to_string(),
                ))
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            eth_call_balance_of(&self.settings.rpc_url, wallet_address, token_address).await
        }
    }

    /// Get ERC20 allowance
    pub async fn get_erc20_allowance(
        &self,
        token_address: &str,
        owner_address: &str,
        spender_address: &str,
    ) -> IdosResult<String> {
        #[cfg(target_arch = "wasm32")]
        {
            eth_call_allowance(
                &self.settings.rpc_url,
                token_address,
                owner_address,
                spender_address,
            )
            .await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Similar to get_erc20_balance but with allowance(address,address) function
            // Selector: 0xdd62ed3e
            if let Some(provider) = &self.provider {
                let token: Address = token_address
                    .parse()
                    .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;

                let selector = "0xdd62ed3e";
                let owner_padded = format!("{:0>64}", owner_address.trim_start_matches("0x"));
                let spender_padded = format!("{:0>64}", spender_address.trim_start_matches("0x"));
                let data = format!("{}{}{}", selector, owner_padded, spender_padded);

                let call_data = ethers::types::transaction::eip2718::TypedTransaction::Legacy(
                    ethers::types::TransactionRequest {
                        to: Some(ethers::types::NameOrAddress::Address(token)),
                        data: Some(Bytes::from(
                            hex::decode(data.trim_start_matches("0x")).unwrap(),
                        )),
                        ..Default::default()
                    },
                );

                let result = provider
                    .call(&call_data, None)
                    .await
                    .map_err(|e| IdosError::NetworkError(e.to_string()))?;

                let allowance = U256::from_big_endian(result.as_ref());
                Ok(allowance.to_string())
            } else {
                Err(IdosError::ConfigurationError(
                    "Provider not initialized".to_string(),
                ))
            }
        }
    }

    /// Request withdrawal signature from backend
    pub async fn get_token_withdrawal_signature(
        &self,
        currency_id: &str,
        amount: i64,
        wallet_address: &str,
    ) -> IdosResult<WithdrawalSignatureResult> {
        let request = WalletTransactionRequest {
            chain_id: self.settings.chain_id,
            transaction_type: CryptoTransactionType::Token,
            direction: TransactionDirection::UsersCryptoWallet,
            transaction_hash: None,
            currency_id: Some(currency_id.to_string()),
            skin_id: None,
            amount: Some(amount),
            connected_wallet_address: Some(wallet_address.to_string()),
        };

        self.client.post("wallet/transaction", &request).await
    }

    /// Request NFT withdrawal signature from backend
    pub async fn get_nft_withdrawal_signature(
        &self,
        skin_id: &str,
        amount: i64,
        wallet_address: &str,
    ) -> IdosResult<WithdrawalSignatureResult> {
        let request = WalletTransactionRequest {
            chain_id: self.settings.chain_id,
            transaction_type: CryptoTransactionType::NFT,
            direction: TransactionDirection::UsersCryptoWallet,
            transaction_hash: None,
            currency_id: None,
            skin_id: Some(skin_id.to_string()),
            amount: Some(amount),
            connected_wallet_address: Some(wallet_address.to_string()),
        };

        self.client.post("wallet/transaction", &request).await
    }

    /// Submit transaction to backend after on-chain confirmation
    pub async fn submit_transaction(
        &self,
        transaction_hash: &str,
        transaction_type: CryptoTransactionType,
        direction: TransactionDirection,
    ) -> IdosResult<String> {
        let request = WalletTransactionRequest {
            chain_id: self.settings.chain_id,
            transaction_type,
            direction,
            transaction_hash: Some(transaction_hash.to_string()),
            currency_id: None,
            skin_id: None,
            amount: None,
            connected_wallet_address: None,
        };

        self.client.post("wallet/transaction", &request).await
    }

    /// Check if sufficient balance for gas
    pub async fn has_sufficient_gas(
        &self,
        wallet_address: &str,
        estimated_gas: u64,
    ) -> IdosResult<bool> {
        let balance_wei = self.get_native_balance(wallet_address).await?;

        // Parse balance as u128
        let balance: u128 = balance_wei
            .parse()
            .map_err(|_| IdosError::InvalidInput("Invalid balance format".to_string()))?;

        // Calculate required gas in wei
        let gas_price_wei = (self.settings.gas_price_gwei * 1_000_000_000.0) as u128;
        let required_gas_wei = gas_price_wei * estimated_gas as u128;

        Ok(balance >= required_gas_wei)
    }

    /// Wait for transaction receipt
    pub async fn wait_for_transaction(
        &self,
        transaction_hash: &str,
        max_attempts: u32,
    ) -> IdosResult<EthTransactionReceipt> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(provider) = &self.provider {
                let tx_hash: H256 = transaction_hash
                    .parse()
                    .map_err(|_| IdosError::InvalidInput("Invalid transaction hash".to_string()))?;

                for _ in 0..max_attempts {
                    if let Some(receipt) = provider
                        .get_transaction_receipt(tx_hash)
                        .await
                        .map_err(|e| IdosError::NetworkError(e.to_string()))?
                    {
                        return Ok(EthTransactionReceipt {
                            transaction_hash: format!("{:?}", receipt.transaction_hash),
                            block_number: receipt.block_number.map(|bn| bn.to_string()),
                            gas_used: receipt.gas_used.map(|gu| gu.to_string()),
                            status: receipt.status.map(|s| s.to_string()),
                            from: Some(format!("{:?}", receipt.from)),
                            to: receipt.to.map(|addr| format!("{:?}", addr)),
                        });
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }

                Err(IdosError::TimeoutError(
                    "Transaction not confirmed".to_string(),
                ))
            } else {
                Err(IdosError::ConfigurationError(
                    "Provider not initialized".to_string(),
                ))
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            for _ in 0..max_attempts {
                if let Ok(receipt) =
                    eth_get_transaction_receipt(&self.settings.rpc_url, transaction_hash).await
                {
                    return Ok(receipt);
                }

                // Wait 3 seconds
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    let window = web_sys::window().unwrap();
                    window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 3000)
                        .ok();
                });
                wasm_bindgen_futures::JsFuture::from(promise).await.ok();
            }

            Err(IdosError::TimeoutError(
                "Transaction not confirmed".to_string(),
            ))
        }
    }
}
