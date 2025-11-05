/// High-level Ethereum wallet service
/// Matches Unity SDK's WalletService.cs API exactly
use super::{dto::*, handler::EthereumHandler, transactions};
use crate::{IdosError, IdosResult};

/// High-level service for Ethereum wallet operations
/// Provides the same API as Unity SDK's WalletService.cs
pub struct EthereumWalletService {
    handler: EthereumHandler,
    private_key: Option<String>,
}

impl EthereumWalletService {
    pub fn new(handler: EthereumHandler) -> Self {
        Self {
            handler,
            private_key: None,
        }
    }

    /// Set private key for signing transactions
    pub fn set_private_key(&mut self, private_key: String) {
        self.private_key = Some(private_key);
    }

    /// Clear private key from memory
    pub fn clear_private_key(&mut self) {
        self.private_key = None;
    }

    fn get_private_key(&self) -> IdosResult<&str> {
        self.private_key
            .as_deref()
            .ok_or_else(|| IdosError::Wallet("Private key not set".to_string()))
    }

    /// Transfer tokens to game platform pool
    /// Matches Unity SDK's TransferTokenToGame
    /// Full flow: check allowance -> approve if needed -> deposit -> submit to backend
    pub async fn transfer_token_to_game(
        &self,
        rpc_url: &str,
        token_address: &str,
        amount: u64,
        user_id: &str,
        wallet_address: &str,
    ) -> IdosResult<String> {
        let private_key = self.get_private_key()?;
        let settings = self.handler.settings();
        let chain_id = settings.chain_id as u64;
        let gas_price_gwei = settings.gas_price_gwei;
        let platform_pool = &settings.platform_pool_contract_address;

        // Convert amount to wei (assuming 18 decimals)
        let amount_wei = (amount as u128 * 1_000_000_000_000_000_000).to_string();

        // 1. Check current allowance
        let current_allowance = self
            .handler
            .get_erc20_allowance(token_address, wallet_address, platform_pool)
            .await?;

        let current_allowance_u128: u128 = current_allowance
            .parse()
            .map_err(|_| IdosError::InvalidInput("Invalid allowance".to_string()))?;
        let required_allowance: u128 = amount_wei.parse().unwrap();

        // 2. Approve if needed
        if current_allowance_u128 < required_allowance {
            // Use max uint256 for unlimited approval (matches Unity SDK)
            let max_allowance =
                "115792089237316195423570985008687907853269984665640564039457584007913129639935";

            let approve_hash = transactions::approve_erc20(
                rpc_url,
                token_address,
                platform_pool,
                max_allowance,
                private_key,
                chain_id,
                gas_price_gwei,
            )
            .await?;

            // Wait for approval confirmation
            self.handler.wait_for_transaction(&approve_hash, 20).await?;
        }

        // 3. Deposit tokens to platform pool
        let deposit_hash = transactions::deposit_erc20(
            rpc_url,
            platform_pool,
            token_address,
            &amount_wei,
            user_id,
            private_key,
            chain_id,
            gas_price_gwei,
        )
        .await?;

        // 4. Submit transaction to backend
        let result = self
            .handler
            .submit_transaction(
                &deposit_hash,
                CryptoTransactionType::Token,
                TransactionDirection::Game,
            )
            .await?;

        Ok(result)
    }

    /// Transfer tokens from game to user wallet
    /// Matches Unity SDK's TransferTokenToUser
    pub async fn transfer_token_to_user(
        &self,
        rpc_url: &str,
        withdrawal_signature: WithdrawalSignatureResult,
    ) -> IdosResult<String> {
        let private_key = self.get_private_key()?;
        let settings = self.handler.settings();
        let chain_id = settings.chain_id as u64;
        let gas_price_gwei = settings.gas_price_gwei;

        // Execute withdrawal with backend signature
        let tx_hash = transactions::withdraw_erc20(
            rpc_url,
            &withdrawal_signature,
            private_key,
            chain_id,
            gas_price_gwei,
        )
        .await?;

        Ok(tx_hash)
    }

    /// Transfer NFT to game platform pool
    /// Matches Unity SDK's TransferNFTToGame
    pub async fn transfer_nft_to_game(
        &self,
        rpc_url: &str,
        nft_contract_address: &str,
        wallet_address: &str,
        nft_id: &str,
        amount: u64,
        user_id: &str,
    ) -> IdosResult<String> {
        let private_key = self.get_private_key()?;
        let settings = self.handler.settings();
        let chain_id = settings.chain_id as u64;
        let gas_price_gwei = settings.gas_price_gwei;
        let platform_pool = &settings.platform_pool_contract_address;

        // Transfer NFT to platform pool
        let tx_hash = transactions::transfer_nft_erc1155(
            rpc_url,
            nft_contract_address,
            wallet_address,
            platform_pool,
            nft_id,
            amount,
            Some(user_id),
            private_key,
            chain_id,
            gas_price_gwei,
        )
        .await?;

        // Submit to backend
        self.handler
            .submit_transaction(
                &tx_hash,
                CryptoTransactionType::NFT,
                TransactionDirection::Game,
            )
            .await?;

        Ok(tx_hash)
    }

    /// Transfer NFT from game to user wallet
    /// Matches Unity SDK's TransferNFTToUser
    pub async fn transfer_nft_to_user(
        &self,
        rpc_url: &str,
        withdrawal_signature: WithdrawalSignatureResult,
    ) -> IdosResult<String> {
        let private_key = self.get_private_key()?;
        let settings = self.handler.settings();
        let chain_id = settings.chain_id as u64;
        let gas_price_gwei = settings.gas_price_gwei;

        // Execute NFT withdrawal with backend signature
        let tx_hash = transactions::withdraw_nft_erc1155(
            rpc_url,
            &withdrawal_signature,
            private_key,
            chain_id,
            gas_price_gwei,
        )
        .await?;

        Ok(tx_hash)
    }

    /// Transfer tokens to external address
    /// Matches Unity SDK's TransferTokenToExternalAddress
    pub async fn transfer_token_to_external_address(
        &self,
        rpc_url: &str,
        token_address: &str,
        from_address: &str,
        to_address: &str,
        amount: u64,
    ) -> IdosResult<String> {
        let private_key = self.get_private_key()?;
        let settings = self.handler.settings();
        let chain_id = settings.chain_id as u64;
        let gas_price_gwei = settings.gas_price_gwei;

        transactions::transfer_erc20(
            rpc_url,
            token_address,
            from_address,
            to_address,
            amount,
            private_key,
            chain_id,
            gas_price_gwei,
        )
        .await
    }

    /// Transfer NFT to external address
    /// Matches Unity SDK's TransferNFTToExternalAddress
    pub async fn transfer_nft_to_external_address(
        &self,
        rpc_url: &str,
        nft_contract_address: &str,
        from_address: &str,
        to_address: &str,
        nft_id: &str,
        amount: u64,
    ) -> IdosResult<String> {
        let private_key = self.get_private_key()?;
        let settings = self.handler.settings();
        let chain_id = settings.chain_id as u64;
        let gas_price_gwei = settings.gas_price_gwei;

        transactions::transfer_nft_erc1155(
            rpc_url,
            nft_contract_address,
            from_address,
            to_address,
            nft_id,
            amount,
            None, // No userID for external transfers
            private_key,
            chain_id,
            gas_price_gwei,
        )
        .await
    }

    /// Get token balance
    /// Matches Unity SDK's GetTokenBalance
    pub async fn get_token_balance(
        &self,
        wallet_address: &str,
        token_address: &str,
    ) -> IdosResult<String> {
        self.handler
            .get_erc20_balance(wallet_address, token_address)
            .await
    }

    /// Get NFT balance
    /// Matches Unity SDK's GetNFTBalance
    pub async fn get_nft_balance(
        &self,
        rpc_url: &str,
        nft_contract_address: &str,
        wallet_address: &str,
        nft_ids: Vec<String>,
    ) -> IdosResult<Vec<String>> {
        transactions::get_nft_balance(rpc_url, nft_contract_address, wallet_address, nft_ids).await
    }

    /// Get native token balance in wei
    /// Matches Unity SDK's GetNativeTokenBalanceInWei
    pub async fn get_native_token_balance_in_wei(
        &self,
        wallet_address: &str,
    ) -> IdosResult<String> {
        self.handler.get_native_balance(wallet_address).await
    }

    /// Check if has sufficient balance for gas
    /// Matches Unity SDK's HasSufficientBalanceForGas
    pub async fn has_sufficient_balance_for_gas(
        &self,
        wallet_address: &str,
        gas_estimate: u64,
    ) -> IdosResult<bool> {
        self.handler
            .has_sufficient_gas(wallet_address, gas_estimate)
            .await
    }
}
