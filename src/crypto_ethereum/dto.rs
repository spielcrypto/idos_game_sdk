/// Data Transfer Objects for Ethereum Wallet
use serde::{Deserialize, Serialize};

/// Crypto transaction type (Token or NFT)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum CryptoTransactionType {
    Token,
    NFT,
}

/// Transaction direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum TransactionDirection {
    UsersCryptoWallet,
    Game,
    ExternalWalletAddress,
}

/// Ethereum transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthTransaction {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl Default for EthTransaction {
    fn default() -> Self {
        Self {
            from: String::new(),
            to: String::new(),
            gas: None,
            gas_price: None,
            value: "0x0".to_string(),
            data: Some("0x".to_string()),
        }
    }
}

/// JSON-RPC request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
    pub id: u64,
}

impl<T> JsonRpcRequest<T> {
    pub fn new(method: String, params: T, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        }
    }
}

/// JSON-RPC response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Wallet transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransactionRequest {
    pub chain_id: i64,
    pub transaction_type: CryptoTransactionType,
    pub direction: TransactionDirection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_wallet_address: Option<String>,
}

/// Withdrawal signature result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalSignatureResult {
    pub contract_address: String,
    pub token_address: String,
    pub wallet_address: String,
    pub amount: String,
    pub nonce: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Balance response for ERC20 tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token_address: String,
    pub balance: String, // In wei, as string to handle large numbers
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
}

/// NFT balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftBalance {
    pub nft_id: String,
    pub balance: String,
}

/// Wallet creation/import result
#[derive(Debug, Clone, Serialize)]
pub struct WalletInfo {
    pub address: String,
    #[serde(skip_serializing)]
    pub private_key: String, // Never serialize this in production
    pub seed_phrase: Option<String>,
}

/// Transaction receipt (custom simplified version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthTransactionReceipt {
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    #[serde(rename = "blockNumber")]
    pub block_number: Option<String>,
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<String>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Blockchain settings
#[derive(Debug, Clone)]
pub struct BlockchainSettings {
    pub rpc_url: String,
    pub chain_id: i64,
    pub platform_pool_contract_address: String,
    pub token_contract_addresses: std::collections::HashMap<String, String>,
    pub nft_contract_address: String,
    pub gas_price_gwei: f64,
}

impl Default for BlockchainSettings {
    fn default() -> Self {
        Self {
            rpc_url: String::new(),
            chain_id: 1,
            platform_pool_contract_address: String::new(),
            token_contract_addresses: std::collections::HashMap::new(),
            nft_contract_address: String::new(),
            gas_price_gwei: 20.0,
        }
    }
}
