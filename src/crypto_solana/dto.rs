/// Data Transfer Objects for Solana Wallet
use serde::{Deserialize, Serialize};

/// Solana cluster types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SolanaCluster {
    Mainnet,
    Devnet,
    Testnet,
    Custom,
}

impl SolanaCluster {
    pub fn rpc_url(&self) -> &str {
        match self {
            SolanaCluster::Mainnet => "https://api.mainnet-beta.solana.com",
            SolanaCluster::Devnet => "https://api.devnet.solana.com",
            SolanaCluster::Testnet => "https://api.testnet.solana.com",
            SolanaCluster::Custom => "",
        }
    }
}

/// Solana blockchain settings
#[derive(Debug, Clone)]
pub struct SolanaSettings {
    pub cluster: SolanaCluster,
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub program_id: String, // Platform pool program ID
}

impl Default for SolanaSettings {
    fn default() -> Self {
        Self {
            cluster: SolanaCluster::Devnet,
            rpc_url: SolanaCluster::Devnet.rpc_url().to_string(),
            ws_url: None,
            program_id: String::new(),
        }
    }
}

/// SPL Token deposit request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositSplRequest {
    pub mint: String,    // Token mint address
    pub amount: u64,     // Amount in smallest units
    pub user_id: String, // User ID for backend
}

/// SPL Token withdraw request (from server signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawSplRequest {
    pub mint: String,
    pub to: String, // Recipient wallet address
    pub amount: u64,
    pub nonce: u64,
    pub user_id: String,
    pub ed25519_public_key_hex: String,
    pub ed25519_message_hex: String,
    pub ed25519_signature_hex: String,
    pub sig_ix_index: u8, // Ed25519 instruction index (usually 0)
}

/// Server withdrawal payload (from backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerWithdrawPayload {
    #[serde(rename = "Mint")]
    pub mint: String,
    #[serde(rename = "WalletAddress")]
    pub wallet_address: String,
    #[serde(rename = "Amount")]
    pub amount: String, // "5000000"
    #[serde(rename = "Nonce")]
    pub nonce: String, // "1757111418234"
    #[serde(rename = "ProgramID")]
    pub program_id: String,
    #[serde(rename = "SignatureHex")]
    pub signature_hex: String,
    #[serde(rename = "SigIxIndex")]
    pub sig_ix_index: i32,
    #[serde(rename = "Ed25519PublicKey")]
    pub ed25519_public_key: String,
    #[serde(rename = "Ed25519Message")]
    pub ed25519_message: String,
    #[serde(rename = "UserID")]
    pub user_id: String,
}

impl From<ServerWithdrawPayload> for WithdrawSplRequest {
    fn from(srv: ServerWithdrawPayload) -> Self {
        Self {
            mint: srv.mint,
            to: srv.wallet_address,
            amount: srv.amount.parse().unwrap_or(0),
            nonce: srv.nonce.parse().unwrap_or(0),
            user_id: srv.user_id,
            ed25519_public_key_hex: srv.ed25519_public_key,
            ed25519_message_hex: srv.ed25519_message,
            ed25519_signature_hex: srv.signature_hex,
            sig_ix_index: srv.sig_ix_index as u8,
        }
    }
}

/// JSON-RPC request for Solana
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaRpcRequest<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

impl<T> SolanaRpcRequest<T> {
    pub fn new(method: String, params: T, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        }
    }
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaRpcResponse<T> {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SolanaRpcError>,
    pub id: u64,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Account balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub context: RpcContext,
    pub value: u64,
}

/// RPC Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcContext {
    pub slot: u64,
}

/// Token account info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountInfo {
    pub mint: String,
    pub owner: String,
    pub token_amount: TokenAmount,
}

/// Token amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: Option<String>,
}

/// Transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub signature: String,
    pub slot: Option<u64>,
    pub confirmed: bool,
}

/// Wallet information
#[derive(Debug, Clone, Serialize)]
pub struct SolanaWalletInfo {
    pub public_key: String,
    #[serde(skip_serializing)]
    pub secret_key: Option<Vec<u8>>, // Never serialize this
    pub mnemonic: Option<String>,
}

/// Transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetadata {
    pub block_time: Option<i64>,
    pub slot: u64,
    pub fee: u64,
}

/// Platform pool transaction request (for backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPoolTransactionRequest {
    pub transaction_type: String, // "Token" or "NFT"
    pub direction: String,        // "Game", "UsersCryptoWallet"
    pub transaction_hash: Option<String>,
    pub currency_id: Option<String>,
    pub amount: Option<u64>,
    pub wallet_address: String,
}

// ==================== RPC Request/Response Structs ====================
// Transaction simulation

/// Transaction simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub success: bool,
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub units_consumed: u64,
}

/// Simulate transaction RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateTransactionRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: (String, SimulateConfig),
}

/// Simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateConfig {
    pub encoding: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<String>,
}

/// Simulate transaction RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateTransactionResponse {
    pub result: SimulateTransactionResult,
}

/// Simulation result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateTransactionResult {
    pub value: SimulateValue,
}

/// Simulation value details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateValue {
    pub err: Option<serde_json::Value>,
    pub logs: Option<Vec<String>>,
    #[serde(rename = "unitsConsumed")]
    pub units_consumed: Option<u64>,
}

// Get blockhash

/// Get latest blockhash RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlockhashRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

/// Get blockhash RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlockhashResponse {
    pub result: BlockhashResult,
}

/// Blockhash result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockhashResult {
    pub value: BlockhashValue,
}

/// Blockhash value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockhashValue {
    pub blockhash: String,
}

// Send transaction

/// Send transaction RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: (String, SendTransactionConfig),
}

/// Send transaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTransactionConfig {
    pub encoding: String,
    pub skip_preflight: bool,
    pub preflight_commitment: String,
}

/// Send transaction RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTransactionResponse {
    pub result: String,
}

// Token accounts

/// Token accounts RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountsResponse {
    pub value: Vec<TokenAccountValue>,
}

/// Token account value wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountValue {
    pub account: TokenAccountData,
}

/// Token account data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountData {
    pub data: TokenAccountParsed,
}

/// Token account parsed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountParsed {
    pub parsed: TokenAccountInfo,
}

// Transaction details

/// Transaction detail response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetailResponse {
    pub slot: u64,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
}

// Transaction status checking (for examples)

/// Transaction status request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Vec<String>,
}

/// Transaction status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatusResponse {
    pub result: Option<serde_json::Value>,
}

// ==================== NFT / Metaplex Structs ====================

/// NFT Metadata from Metaplex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftMetadata {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<NftCreator>>,
    pub primary_sale_happened: bool,
    pub is_mutable: bool,
    pub update_authority: String,
    pub collection: Option<NftCollection>,
    pub uses: Option<NftUses>,
}

/// NFT Creator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}

/// NFT Collection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftCollection {
    pub verified: bool,
    pub key: String,
}

/// NFT Uses configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftUses {
    pub use_method: String,
    pub remaining: u64,
    pub total: u64,
}

/// Off-chain JSON metadata (from URI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftJsonMetadata {
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub animation_url: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Option<Vec<NftAttribute>>,
    pub properties: Option<serde_json::Value>,
}

/// NFT Attribute (trait)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftAttribute {
    pub trait_type: String,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_type: Option<String>,
}

/// Complete NFT data (on-chain + off-chain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nft {
    pub metadata: NftMetadata,
    pub json_metadata: Option<NftJsonMetadata>,
    pub owner: String,
}

/// NFT loading result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftLoadResult {
    pub nfts: Vec<Nft>,
    pub count: usize,
}
