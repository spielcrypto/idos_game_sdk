/// Ethereum transaction building and signing
/// Matches Unity SDK's WalletBlockchainService functionality
use super::dto::*;
use crate::{IdosError, IdosResult};

#[cfg(feature = "crypto_ethereum")]
use ethers::{
    abi::{encode, Token as AbiToken},
    contract::abigen,
    core::types::{Bytes, TransactionRequest, U256},
    prelude::*,
    signers::{LocalWallet, Signer},
    utils::{hex, keccak256},
};

/// ERC20 token contract ABI definitions
#[cfg(feature = "crypto_ethereum")]
abigen!(
    ERC20,
    r#"[
        function balanceOf(address owner) external view returns (uint256)
        function transfer(address to, uint256 amount) external returns (bool)
        function approve(address spender, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
    ]"#,
);

/// Platform Pool contract ABI for deposits/withdrawals
#[cfg(feature = "crypto_ethereum")]
abigen!(
    PlatformPool,
    r#"[
        function depositERC20(address token, uint256 amount, string memory userID) external returns (bool)
        function withdrawERC20(address token, address to, uint256 amount, uint256 nonce, bytes memory signature) external returns (bool)
    ]"#,
);

/// ERC1155 NFT contract ABI
#[cfg(feature = "crypto_ethereum")]
abigen!(
    ERC1155,
    r#"[
        function balanceOf(address account, uint256 id) external view returns (uint256)
        function balanceOfBatch(address[] memory accounts, uint256[] memory ids) external view returns (uint256[] memory)
        function safeTransferFrom(address from, address to, uint256 id, uint256 amount, bytes memory data) external
    ]"#,
);

/// Approve ERC20 token for spending
/// Matches Unity SDK's ApproveERC20Token
#[cfg(feature = "crypto_ethereum")]
pub async fn approve_erc20(
    rpc_url: &str,
    token_address: &str,
    spender_address: &str,
    amount_wei: &str,
    private_key: &str,
    chain_id: u64,
    gas_price_gwei: f64,
) -> IdosResult<String> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Provider error: {}", e)))?;

    let wallet: LocalWallet = private_key
        .parse()
        .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;
    let wallet = wallet.with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);

    let token_addr: Address = token_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;
    let spender: Address = spender_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid spender address".to_string()))?;
    let amount: U256 = amount_wei
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid amount".to_string()))?;

    let erc20 = ERC20::new(token_addr, std::sync::Arc::new(client));

    let gas_price = ethers::utils::parse_units(gas_price_gwei, "gwei")
        .map_err(|e| IdosError::InvalidInput(format!("Invalid gas price: {}", e)))?;

    let tx = erc20
        .approve(spender, amount)
        .gas_price(gas_price)
        .gas(50000u64);

    let pending_tx = tx
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Transaction failed: {}", e)))?;

    Ok(format!("{:?}", pending_tx.tx_hash()))
}

/// Deposit ERC20 tokens to platform pool
/// Matches Unity SDK's DepositERC20Token
#[cfg(feature = "crypto_ethereum")]
pub async fn deposit_erc20(
    rpc_url: &str,
    platform_pool_address: &str,
    token_address: &str,
    amount_wei: &str,
    user_id: &str,
    private_key: &str,
    chain_id: u64,
    gas_price_gwei: f64,
) -> IdosResult<String> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Provider error: {}", e)))?;

    let wallet: LocalWallet = private_key
        .parse()
        .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;
    let wallet = wallet.with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);

    let pool_addr: Address = platform_pool_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid pool address".to_string()))?;
    let token_addr: Address = token_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;
    let amount: U256 = amount_wei
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid amount".to_string()))?;

    let pool = PlatformPool::new(pool_addr, std::sync::Arc::new(client));

    let gas_price = ethers::utils::parse_units(gas_price_gwei, "gwei")
        .map_err(|e| IdosError::InvalidInput(format!("Invalid gas price: {}", e)))?;

    let tx = pool
        .deposit_erc20(token_addr, amount, user_id.to_string())
        .gas_price(gas_price)
        .gas(90000u64);

    let pending_tx = tx
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Deposit failed: {}", e)))?;

    Ok(format!("{:?}", pending_tx.tx_hash()))
}

/// Withdraw ERC20 tokens with backend signature
/// Matches Unity SDK's WithdrawERC20Token
#[cfg(feature = "crypto_ethereum")]
pub async fn withdraw_erc20(
    rpc_url: &str,
    withdrawal_data: &WithdrawalSignatureResult,
    private_key: &str,
    chain_id: u64,
    gas_price_gwei: f64,
) -> IdosResult<String> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Provider error: {}", e)))?;

    let wallet: LocalWallet = private_key
        .parse()
        .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;
    let wallet = wallet.with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);

    let pool_addr: Address = withdrawal_data
        .contract_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid contract address".to_string()))?;
    let token_addr: Address = withdrawal_data
        .token_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;
    let to_addr: Address = withdrawal_data
        .wallet_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid wallet address".to_string()))?;
    let amount: U256 = withdrawal_data
        .amount
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid amount".to_string()))?;
    let nonce: U256 = withdrawal_data
        .nonce
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid nonce".to_string()))?;

    // Decode signature
    let signature_bytes = hex::decode(withdrawal_data.signature.trim_start_matches("0x"))
        .map_err(|e| IdosError::InvalidInput(format!("Invalid signature: {}", e)))?;

    let gas_price = ethers::utils::parse_units(gas_price_gwei, "gwei")
        .map_err(|e| IdosError::InvalidInput(format!("Invalid gas price: {}", e)))?;

    let signature_bytes_ethers = Bytes::from(signature_bytes.clone());

    // Handle both versions: with and without userID
    // Unity SDK has WithdrawERC20Function (5 params) and WithdrawERC20FunctionV2 (6 params)
    let tx_hash = if let Some(user_id) = &withdrawal_data.user_id {
        // V2: withdrawERC20(address token, address to, uint256 amount, uint256 nonce, bytes signature, string userID)
        // Manually encode calldata since Solidity function overloading needs different signatures

        // Calculate function selector from signature
        let function_sig = "withdrawERC20(address,address,uint256,uint256,bytes,string)";
        let selector_hash = keccak256(function_sig.as_bytes());
        let selector = &selector_hash[0..4];

        let tokens = vec![
            AbiToken::Address(token_addr),
            AbiToken::Address(to_addr),
            AbiToken::Uint(amount),
            AbiToken::Uint(nonce),
            AbiToken::Bytes(signature_bytes.clone()),
            AbiToken::String(user_id.clone()),
        ];

        let encoded = encode(&tokens);
        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&encoded);

        // Send transaction with manual calldata
        let tx_request = TransactionRequest::new()
            .to(pool_addr)
            .data(Bytes::from(calldata))
            .gas_price(gas_price)
            .gas(150000u64);

        let pending_tx = client
            .send_transaction(tx_request, None)
            .await
            .map_err(|e| IdosError::NetworkError(format!("Withdrawal V2 failed: {}", e)))?;

        format!("{:?}", pending_tx.tx_hash())
    } else {
        // V1: withdrawERC20(address token, address to, uint256 amount, uint256 nonce, bytes signature)
        let pool = PlatformPool::new(pool_addr, std::sync::Arc::new(client.clone()));

        let tx = pool
            .withdraw_erc20(token_addr, to_addr, amount, nonce, signature_bytes_ethers)
            .gas_price(gas_price)
            .gas(150000u64);

        let pending_tx = tx
            .send()
            .await
            .map_err(|e| IdosError::NetworkError(format!("Withdrawal failed: {}", e)))?;

        format!("{:?}", pending_tx.tx_hash())
    };

    Ok(tx_hash)
}

/// Transfer ERC20 tokens to external address
/// Matches Unity SDK's TransferERC20TokenAndGetHash
#[cfg(feature = "crypto_ethereum")]
pub async fn transfer_erc20(
    rpc_url: &str,
    token_address: &str,
    _from_address: &str, // Derived from private key, kept for API compatibility
    to_address: &str,
    amount: u64,
    private_key: &str,
    chain_id: u64,
    gas_price_gwei: f64,
) -> IdosResult<String> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Provider error: {}", e)))?;

    let wallet: LocalWallet = private_key
        .parse()
        .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;
    let wallet = wallet.with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);

    let token_addr: Address = token_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;
    let to_addr: Address = to_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid recipient address".to_string()))?;

    // Convert amount to wei (assuming 18 decimals)
    let amount_wei: U256 = ethers::utils::parse_units(amount, 18)
        .map_err(|e| IdosError::InvalidInput(format!("Invalid amount: {}", e)))?
        .into();

    let erc20 = ERC20::new(token_addr, std::sync::Arc::new(client));

    let gas_price = ethers::utils::parse_units(gas_price_gwei, "gwei")
        .map_err(|e| IdosError::InvalidInput(format!("Invalid gas price: {}", e)))?;

    let tx = erc20
        .transfer(to_addr, amount_wei)
        .gas_price(gas_price)
        .gas(100000u64);

    let pending_tx = tx
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Transfer failed: {}", e)))?;

    Ok(format!("{:?}", pending_tx.tx_hash()))
}

/// Get ERC1155 NFT balance for multiple token IDs
/// Matches Unity SDK's GetNFTBalance
#[cfg(feature = "crypto_ethereum")]
pub async fn get_nft_balance(
    rpc_url: &str,
    nft_contract_address: &str,
    wallet_address: &str,
    token_ids: Vec<String>,
) -> IdosResult<Vec<String>> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Provider error: {}", e)))?;

    let nft_addr: Address = nft_contract_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid NFT contract address".to_string()))?;
    let wallet: Address = wallet_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid wallet address".to_string()))?;

    let erc1155 = ERC1155::new(nft_addr, std::sync::Arc::new(provider));

    // Convert token IDs to U256
    let ids: Vec<U256> = token_ids
        .iter()
        .map(|id| id.parse::<U256>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| IdosError::InvalidInput(format!("Invalid token ID: {}", e)))?;

    // Create accounts array (same wallet for all IDs)
    let accounts = vec![wallet; ids.len()];

    let balances = erc1155
        .balance_of_batch(accounts, ids)
        .call()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Balance query failed: {}", e)))?;

    Ok(balances.iter().map(|b| b.to_string()).collect())
}

/// Transfer ERC1155 NFT
/// Matches Unity SDK's TransferNFT1155AndGetHash
#[cfg(feature = "crypto_ethereum")]
pub async fn transfer_nft_erc1155(
    rpc_url: &str,
    nft_contract_address: &str,
    from_address: &str,
    to_address: &str,
    token_id: &str,
    amount: u64,
    user_id: Option<&str>,
    private_key: &str,
    chain_id: u64,
    gas_price_gwei: f64,
) -> IdosResult<String> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Provider error: {}", e)))?;

    let wallet: LocalWallet = private_key
        .parse()
        .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;
    let wallet = wallet.with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);

    let nft_addr: Address = nft_contract_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid NFT contract address".to_string()))?;
    let from_addr: Address = from_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid from address".to_string()))?;
    let to_addr: Address = to_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid to address".to_string()))?;
    let id: U256 = token_id
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token ID".to_string()))?;

    let erc1155 = ERC1155::new(nft_addr, std::sync::Arc::new(client));

    let gas_price = ethers::utils::parse_units(gas_price_gwei, "gwei")
        .map_err(|e| IdosError::InvalidInput(format!("Invalid gas price: {}", e)))?;

    // Data field: encode userID if present (matches Unity SDK)
    let data = if let Some(uid) = user_id {
        Bytes::from(uid.as_bytes().to_vec())
    } else {
        Bytes::from(vec![])
    };

    let tx = erc1155
        .safe_transfer_from(from_addr, to_addr, id, amount.into(), data)
        .gas_price(gas_price)
        .gas(100000u64);

    let pending_tx = tx
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(format!("NFT transfer failed: {}", e)))?;

    Ok(format!("{:?}", pending_tx.tx_hash()))
}

/// Withdraw ERC1155 NFT with backend signature
/// Matches Unity SDK's WithdrawERC1155Token (both V1 and V2)
#[cfg(feature = "crypto_ethereum")]
pub async fn withdraw_nft_erc1155(
    rpc_url: &str,
    withdrawal_data: &WithdrawalSignatureResult,
    private_key: &str,
    chain_id: u64,
    gas_price_gwei: f64,
) -> IdosResult<String> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Provider error: {}", e)))?;

    let wallet: LocalWallet = private_key
        .parse()
        .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;
    let wallet = wallet.with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);

    let pool_addr: Address = withdrawal_data
        .contract_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid contract address".to_string()))?;
    let token_addr: Address = withdrawal_data
        .token_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;
    let to_addr: Address = withdrawal_data
        .wallet_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid wallet address".to_string()))?;
    let token_id: U256 = withdrawal_data
        .token_id
        .as_ref()
        .ok_or_else(|| IdosError::InvalidInput("Missing token ID for NFT".to_string()))?
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token ID".to_string()))?;
    let amount: U256 = withdrawal_data
        .amount
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid amount".to_string()))?;
    let nonce: U256 = withdrawal_data
        .nonce
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid nonce".to_string()))?;

    let signature_bytes = hex::decode(withdrawal_data.signature.trim_start_matches("0x"))
        .map_err(|e| IdosError::InvalidInput(format!("Invalid signature: {}", e)))?;

    let gas_price = ethers::utils::parse_units(gas_price_gwei, "gwei")
        .map_err(|e| IdosError::InvalidInput(format!("Invalid gas price: {}", e)))?;

    // Handle both V1 and V2 (with userID)
    let tx_hash = if let Some(user_id) = &withdrawal_data.user_id {
        // V2: withdrawERC1155(address token, address to, uint256 id, uint256 amount, uint256 nonce, bytes signature, string userID)

        let function_sig = "withdrawERC1155(address,address,uint256,uint256,uint256,bytes,string)";
        let selector_hash = keccak256(function_sig.as_bytes());
        let selector = &selector_hash[0..4];

        let tokens = vec![
            AbiToken::Address(token_addr),
            AbiToken::Address(to_addr),
            AbiToken::Uint(token_id),
            AbiToken::Uint(amount),
            AbiToken::Uint(nonce),
            AbiToken::Bytes(signature_bytes),
            AbiToken::String(user_id.clone()),
        ];

        let encoded = encode(&tokens);
        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&encoded);

        let tx_request = TransactionRequest::new()
            .to(pool_addr)
            .data(Bytes::from(calldata))
            .gas_price(gas_price)
            .gas(150000u64);

        let pending_tx = client
            .send_transaction(tx_request, None)
            .await
            .map_err(|e| IdosError::NetworkError(format!("NFT withdrawal V2 failed: {}", e)))?;

        format!("{:?}", pending_tx.tx_hash())
    } else {
        // V1: withdrawERC1155(address token, address to, uint256 id, uint256 amount, uint256 nonce, bytes signature)

        let function_sig = "withdrawERC1155(address,address,uint256,uint256,uint256,bytes)";
        let selector_hash = keccak256(function_sig.as_bytes());
        let selector = &selector_hash[0..4];

        let tokens = vec![
            AbiToken::Address(token_addr),
            AbiToken::Address(to_addr),
            AbiToken::Uint(token_id),
            AbiToken::Uint(amount),
            AbiToken::Uint(nonce),
            AbiToken::Bytes(signature_bytes),
        ];

        let encoded = encode(&tokens);
        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&encoded);

        let tx_request = TransactionRequest::new()
            .to(pool_addr)
            .data(Bytes::from(calldata))
            .gas_price(gas_price)
            .gas(150000u64);

        let pending_tx = client
            .send_transaction(tx_request, None)
            .await
            .map_err(|e| IdosError::NetworkError(format!("NFT withdrawal failed: {}", e)))?;

        format!("{:?}", pending_tx.tx_hash())
    };

    Ok(tx_hash)
}

// ==================== GAS ESTIMATION ====================

/// Estimate gas for a generic Ethereum transaction
/// Matches Unity SDK's eth_estimateGas functionality
#[cfg(feature = "crypto_ethereum")]
pub async fn estimate_gas(
    rpc_url: &str,
    from_address: &str,
    to_address: &str,
    data: Option<&str>,
    value_wei: Option<&str>,
) -> IdosResult<u64> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Failed to create provider: {}", e)))?;

    let from: Address = from_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid from address".to_string()))?;
    let to: Address = to_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid to address".to_string()))?;

    let mut tx = TransactionRequest::new().from(from).to(to);

    if let Some(data_hex) = data {
        let bytes = hex::decode(data_hex.trim_start_matches("0x"))
            .map_err(|e| IdosError::InvalidInput(format!("Invalid data hex: {}", e)))?;
        tx = tx.data(bytes);
    }

    if let Some(value_str) = value_wei {
        let value: U256 = value_str
            .parse()
            .map_err(|_| IdosError::InvalidInput("Invalid value".to_string()))?;
        tx = tx.value(value);
    }

    let gas_estimate = provider
        .estimate_gas(&tx.into(), None)
        .await
        .map_err(|e| IdosError::NetworkError(format!("Gas estimation failed: {}", e)))?;

    Ok(gas_estimate.as_u64())
}

/// Estimate gas for ERC20 transfer
/// Provides a convenient wrapper for token transfers
#[cfg(feature = "crypto_ethereum")]
pub async fn estimate_gas_erc20_transfer(
    rpc_url: &str,
    token_address: &str,
    from_address: &str,
    to_address: &str,
    amount: &str,
) -> IdosResult<u64> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Failed to create provider: {}", e)))?;

    let token_addr: Address = token_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;

    let contract = ERC20::new(token_addr, provider.into());

    let to: Address = to_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid recipient address".to_string()))?;

    let amount_u256: U256 = amount
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid amount".to_string()))?;

    let from: Address = from_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid from address".to_string()))?;

    let call = contract.transfer(to, amount_u256).from(from);

    let gas_estimate = call
        .estimate_gas()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Gas estimation failed: {}", e)))?;

    Ok(gas_estimate.as_u64())
}

/// Estimate gas for ERC1155 NFT transfer (safeTransferFrom)
/// Matches Unity SDK's EstimateGasNFTAsync
#[cfg(feature = "crypto_ethereum")]
pub async fn estimate_gas_nft_transfer(
    rpc_url: &str,
    nft_contract_address: &str,
    from_address: &str,
    to_address: &str,
    token_id: u64,
    amount: u64,
    data: Option<Vec<u8>>,
) -> IdosResult<u64> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Failed to create provider: {}", e)))?;

    let nft_addr: Address = nft_contract_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid NFT contract address".to_string()))?;

    let contract = ERC1155::new(nft_addr, provider.into());

    let from: Address = from_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid from address".to_string()))?;

    let to: Address = to_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid to address".to_string()))?;

    let token_id_u256 = U256::from(token_id);
    let amount_u256 = U256::from(amount);
    let data_bytes = Bytes::from(data.unwrap_or_default());

    let call = contract
        .safe_transfer_from(from, to, token_id_u256, amount_u256, data_bytes)
        .from(from);

    let gas_estimate = call
        .estimate_gas()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Gas estimation failed: {}", e)))?;

    Ok(gas_estimate.as_u64())
}

/// Estimate gas for ERC20 approval
#[cfg(feature = "crypto_ethereum")]
pub async fn estimate_gas_erc20_approval(
    rpc_url: &str,
    token_address: &str,
    from_address: &str,
    spender_address: &str,
    amount: &str,
) -> IdosResult<u64> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| IdosError::NetworkError(format!("Failed to create provider: {}", e)))?;

    let token_addr: Address = token_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid token address".to_string()))?;

    let contract = ERC20::new(token_addr, provider.into());

    let spender: Address = spender_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid spender address".to_string()))?;

    let amount_u256: U256 = amount
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid amount".to_string()))?;

    let from: Address = from_address
        .parse()
        .map_err(|_| IdosError::InvalidInput("Invalid from address".to_string()))?;

    let call = contract.approve(spender, amount_u256).from(from);

    let gas_estimate = call
        .estimate_gas()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Gas estimation failed: {}", e)))?;

    Ok(gas_estimate.as_u64())
}
