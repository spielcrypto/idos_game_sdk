/// Solana transaction building for platform pool operations
/// Matches Unity SDK's SolanaPlatformPoolService
use super::anchor::*;
use super::dto::*;
use crate::{IdosError, IdosResult};

#[cfg(feature = "crypto_solana")]
use solana_sdk::{
    hash::Hash,
    instruction::AccountMeta as SdkAccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction as SolanaTransaction,
};

#[cfg(feature = "crypto_solana")]
use base64::{engine::general_purpose, Engine as _};

// Official Solana system program addresses (hardcoded in Solana blockchain)
// These are the same on mainnet, devnet, and testnet
// Reference: https://docs.solana.com/developing/runtime-facilities/programs
pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // SPL Token program
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"; // ATA program
pub const SYSVAR_INSTRUCTIONS_ID: &str = "Sysvar1nstructions1111111111111111111111111"; // Sysvar for instruction introspection
pub const ED25519_PROGRAM_ID: &str = "Ed25519SigVerify111111111111111111111111111"; // Ed25519 signature verification

/// Account metadata for Solana instructions
#[derive(Debug, Clone)]
pub struct AccountMeta {
    pub pubkey: [u8; 32],
    pub is_signer: bool,
    pub is_writable: bool,
}

impl AccountMeta {
    pub fn new(pubkey: [u8; 32], is_signer: bool, is_writable: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable,
        }
    }

    pub fn read_only(pubkey: [u8; 32], is_signer: bool) -> Self {
        Self::new(pubkey, is_signer, false)
    }

    pub fn writable(pubkey: [u8; 32], is_signer: bool) -> Self {
        Self::new(pubkey, is_signer, true)
    }
}

/// Solana transaction instruction
#[derive(Debug, Clone)]
pub struct TransactionInstruction {
    pub program_id: [u8; 32],
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
}

/// Derive Associated Token Account address
/// Simplified version - matches Unity's AssociatedTokenAccountProgram.DeriveAssociatedTokenAccount
#[cfg(feature = "crypto_solana")]
pub fn derive_associated_token_account(
    wallet_address: &[u8; 32],
    mint_address: &[u8; 32],
) -> IdosResult<[u8; 32]> {
    let ata_program_id = bs58::decode(ASSOCIATED_TOKEN_PROGRAM_ID)
        .into_vec()
        .map_err(|e| IdosError::Wallet(format!("Invalid ATA program ID: {}", e)))?;

    let mut ata_program_id_bytes = [0u8; 32];
    ata_program_id_bytes.copy_from_slice(&ata_program_id);

    // PDA seeds: [wallet, token_program, mint]
    let token_program_id = bs58::decode(TOKEN_PROGRAM_ID)
        .into_vec()
        .map_err(|e| IdosError::Wallet(format!("Invalid token program ID: {}", e)))?;

    let (pda, _bump) = find_program_address(
        &[wallet_address, &token_program_id, mint_address],
        &ata_program_id_bytes,
    )?;

    Ok(pda)
}

/// Build Anchor instruction for deposit_spl
/// Matches Unity SDK's DepositSplAsync instruction building
#[cfg(feature = "crypto_solana")]
pub fn build_deposit_spl_instruction(
    program_id: &[u8; 32],
    config_pda: &[u8; 32],
    vault_pda: &[u8; 32],
    mint: &[u8; 32],
    user_pubkey: &[u8; 32],
    user_ata: &[u8; 32],
    vault_ata: &[u8; 32],
    amount: u64,
    user_id: &str,
) -> TransactionInstruction {
    // Anchor discriminator for "deposit_spl"
    let discriminator = anchor_discriminator("deposit_spl");

    // Encode arguments: amount (u64) + user_id (string)
    let amount_bytes = encode_u64(amount);
    let user_id_bytes = encode_string(user_id);

    let data = borsh_cat(&[&discriminator, &amount_bytes, &user_id_bytes]);

    // System program IDs
    let token_program = bs58::decode(TOKEN_PROGRAM_ID).into_vec().unwrap();
    let ata_program = bs58::decode(ASSOCIATED_TOKEN_PROGRAM_ID)
        .into_vec()
        .unwrap();
    let system_program = bs58::decode(SYSTEM_PROGRAM_ID).into_vec().unwrap();

    let mut token_program_id = [0u8; 32];
    let mut ata_program_id = [0u8; 32];
    let mut system_program_id = [0u8; 32];

    token_program_id.copy_from_slice(&token_program);
    ata_program_id.copy_from_slice(&ata_program);
    system_program_id.copy_from_slice(&system_program);

    let accounts = vec![
        AccountMeta::read_only(*config_pda, false),
        AccountMeta::writable(*vault_pda, false),
        AccountMeta::read_only(*mint, false),
        AccountMeta::read_only(*user_pubkey, true), // user signer
        AccountMeta::writable(*user_ata, false),
        AccountMeta::writable(*vault_ata, false),
        AccountMeta::read_only(token_program_id, false),
        AccountMeta::read_only(ata_program_id, false),
        AccountMeta::read_only(system_program_id, false),
    ];

    TransactionInstruction {
        program_id: *program_id,
        accounts,
        data,
    }
}

/// Build Anchor instruction for withdraw_spl
/// Matches Unity SDK's WithdrawSplAsync instruction building
#[cfg(feature = "crypto_solana")]
pub fn build_withdraw_spl_instruction(
    program_id: &[u8; 32],
    config_pda: &[u8; 32],
    payer_pubkey: &[u8; 32],
    vault_pda: &[u8; 32],
    nonce_marker_pda: &[u8; 32],
    mint: &[u8; 32],
    to_pubkey: &[u8; 32],
    vault_ata: &[u8; 32],
    to_ata: &[u8; 32],
    amount: u64,
    nonce: u64,
    user_id: &str,
    sig_ix_index: u8,
) -> TransactionInstruction {
    // Anchor discriminator for "withdraw_spl"
    let discriminator = anchor_discriminator("withdraw_spl");

    // Encode arguments: amount (u64) + nonce (u64) + user_id (string) + sig_ix_index (u8)
    let amount_bytes = encode_u64(amount);
    let nonce_bytes = encode_u64(nonce);
    let user_id_bytes = encode_string(user_id);
    let sig_ix_bytes = [sig_ix_index];

    let data = borsh_cat(&[
        &discriminator,
        &amount_bytes,
        &nonce_bytes,
        &user_id_bytes,
        &sig_ix_bytes,
    ]);

    // System program IDs
    let sysvar_instructions = bs58::decode(SYSVAR_INSTRUCTIONS_ID).into_vec().unwrap();
    let token_program = bs58::decode(TOKEN_PROGRAM_ID).into_vec().unwrap();
    let ata_program = bs58::decode(ASSOCIATED_TOKEN_PROGRAM_ID)
        .into_vec()
        .unwrap();
    let system_program = bs58::decode(SYSTEM_PROGRAM_ID).into_vec().unwrap();

    let mut sysvar_instructions_id = [0u8; 32];
    let mut token_program_id = [0u8; 32];
    let mut ata_program_id = [0u8; 32];
    let mut system_program_id = [0u8; 32];

    sysvar_instructions_id.copy_from_slice(&sysvar_instructions);
    token_program_id.copy_from_slice(&token_program);
    ata_program_id.copy_from_slice(&ata_program);
    system_program_id.copy_from_slice(&system_program);

    let accounts = vec![
        AccountMeta::read_only(*config_pda, false),
        AccountMeta::read_only(*payer_pubkey, true), // payer signer
        AccountMeta::read_only(*vault_pda, false),
        AccountMeta::writable(*nonce_marker_pda, false),
        AccountMeta::read_only(*mint, false),
        AccountMeta::read_only(*to_pubkey, false),
        AccountMeta::writable(*vault_ata, false),
        AccountMeta::writable(*to_ata, false),
        AccountMeta::read_only(sysvar_instructions_id, false),
        AccountMeta::read_only(token_program_id, false),
        AccountMeta::read_only(ata_program_id, false),
        AccountMeta::read_only(system_program_id, false),
    ];

    TransactionInstruction {
        program_id: *program_id,
        accounts,
        data,
    }
}

// ==================== TRANSACTION SERIALIZATION & SIGNING ====================
// Full transaction support using solana-sdk

/// Convert our TransactionInstruction to solana-sdk Instruction
#[cfg(feature = "crypto_solana")]
fn to_solana_instruction(ix: &TransactionInstruction) -> solana_sdk::instruction::Instruction {
    let program_id = Pubkey::new_from_array(ix.program_id);
    let accounts: Vec<SdkAccountMeta> = ix
        .accounts
        .iter()
        .map(|acc| SdkAccountMeta {
            pubkey: Pubkey::new_from_array(acc.pubkey),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    solana_sdk::instruction::Instruction {
        program_id,
        accounts,
        data: ix.data.clone(),
    }
}

/// Transaction builder that can sign and serialize transactions
#[cfg(feature = "crypto_solana")]
#[derive(Debug)]
pub struct TransactionBuilder {
    pub instructions: Vec<TransactionInstruction>,
    pub fee_payer: [u8; 32],
    pub recent_blockhash: Option<String>,
}

#[cfg(feature = "crypto_solana")]
impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(fee_payer: [u8; 32]) -> Self {
        Self {
            instructions: Vec::new(),
            fee_payer,
            recent_blockhash: None,
        }
    }

    /// Add an instruction to the transaction
    pub fn add_instruction(&mut self, instruction: TransactionInstruction) -> &mut Self {
        self.instructions.push(instruction);
        self
    }

    /// Set the recent blockhash (required before signing)
    pub fn set_recent_blockhash(&mut self, blockhash: &str) -> &mut Self {
        self.recent_blockhash = Some(blockhash.to_string());
        self
    }

    /// Sign the transaction with the given keypair
    /// Returns the signed transaction bytes serialized to base64 (ready for RPC)
    pub fn sign_and_serialize(&self, keypair_bytes: &[u8]) -> IdosResult<String> {
        if self.instructions.is_empty() {
            return Err(IdosError::Wallet(
                "No instructions in transaction".to_string(),
            ));
        }

        let blockhash_str = self
            .recent_blockhash
            .as_ref()
            .ok_or_else(|| IdosError::Wallet("Recent blockhash not set".to_string()))?;

        // Parse keypair from bytes
        let keypair = if keypair_bytes.len() == 64 {
            // Full keypair (secret + public) - Use first 32 bytes as secret key
            let secret_bytes: [u8; 32] = keypair_bytes[..32].try_into().unwrap();
            Keypair::new_from_array(secret_bytes)
        } else if keypair_bytes.len() == 32 {
            // Just secret key
            let secret_bytes: [u8; 32] = keypair_bytes.try_into().unwrap();
            Keypair::new_from_array(secret_bytes)
        } else {
            return Err(IdosError::Wallet(format!(
                "Invalid keypair length: {}",
                keypair_bytes.len()
            )));
        };

        // Parse blockhash
        let blockhash = blockhash_str
            .parse::<Hash>()
            .map_err(|e| IdosError::Wallet(format!("Invalid blockhash: {}", e)))?;

        // Convert instructions to solana-sdk format
        let solana_instructions: Vec<solana_sdk::instruction::Instruction> = self
            .instructions
            .iter()
            .map(to_solana_instruction)
            .collect();

        // Create and sign transaction
        let mut transaction =
            SolanaTransaction::new_with_payer(&solana_instructions, Some(&keypair.pubkey()));

        transaction.message.recent_blockhash = blockhash;
        transaction.sign(&[&keypair], blockhash);

        // Serialize to bytes then base64 (bincode v2.0 with serde compatibility)
        // Note: Solana Transaction implements serde::Serialize, so we use serde module
        let config = bincode::config::standard()
            .with_little_endian()
            .with_fixed_int_encoding();

        let serialized = bincode::serde::encode_to_vec(&transaction, config).map_err(|e| {
            IdosError::SerializationError(format!("Failed to serialize transaction: {}", e))
        })?;

        Ok(general_purpose::STANDARD.encode(&serialized))
    }

    /// Get the transaction size estimate in bytes (for fee calculation)
    pub fn estimate_size(&self) -> usize {
        // Rough estimate:
        // - 1 signature: 64 bytes
        // - Message header: ~3 bytes
        // - Recent blockhash: 32 bytes
        // - Account keys: variable
        // - Instructions: variable

        let base_size = 64 + 3 + 32;

        // Estimate account keys (unique pubkeys)
        let mut unique_pubkeys = std::collections::HashSet::new();
        unique_pubkeys.insert(self.fee_payer);

        for ix in &self.instructions {
            unique_pubkeys.insert(ix.program_id);
            for acc in &ix.accounts {
                unique_pubkeys.insert(acc.pubkey);
            }
        }

        let accounts_size = unique_pubkeys.len() * 32;

        // Estimate instructions size
        let instructions_size: usize = self
            .instructions
            .iter()
            .map(|ix| {
                1 + // program_id index
            1 + // accounts length
            ix.accounts.len() + // account indices
            2 + // data length (compact-u16)
            ix.data.len() // data
            })
            .sum();

        base_size + accounts_size + instructions_size
    }
}

/// Estimate transaction fees for Solana
/// Solana uses a deterministic fee model based on signatures
#[cfg(feature = "crypto_solana")]
pub fn estimate_transaction_fee(num_signatures: usize) -> u64 {
    // Base fee per signature: 5000 lamports (0.000005 SOL)
    // This is the standard Solana fee as of 2024
    const LAMPORTS_PER_SIGNATURE: u64 = 5000;

    (num_signatures as u64) * LAMPORTS_PER_SIGNATURE
}

/// Simulate a transaction to check if it will succeed
/// This makes an RPC call to simulate the transaction
#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
pub async fn simulate_transaction(
    rpc_url: &str,
    transaction_base64: &str,
) -> IdosResult<SimulationResult> {
    let client = reqwest::Client::new();
    let request = SimulateTransactionRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "simulateTransaction".to_string(),
        params: (
            transaction_base64.to_string(),
            SimulateConfig {
                encoding: "base64".to_string(),
                commitment: Some("processed".to_string()),
            },
        ),
    };

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Simulation request failed: {}", e)))?
        .json::<SimulateTransactionResponse>()
        .await
        .map_err(|e| {
            IdosError::NetworkError(format!("Failed to parse simulation response: {}", e))
        })?;

    let value = response.result.value;

    Ok(SimulationResult {
        success: value.err.is_none(),
        error: value.err.map(|e| e.to_string()),
        logs: value.logs.unwrap_or_default(),
        units_consumed: value.units_consumed.unwrap_or(0),
    })
}

/// Get recent blockhash from Solana RPC
#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
pub async fn get_recent_blockhash(rpc_url: &str) -> IdosResult<String> {
    let client = reqwest::Client::new();
    let request = GetBlockhashRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "getLatestBlockhash".to_string(),
        params: vec![serde_json::json!({
            "commitment": "finalized"
        })],
    };

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Blockhash request failed: {}", e)))?
        .json::<GetBlockhashResponse>()
        .await
        .map_err(|e| {
            IdosError::NetworkError(format!("Failed to parse blockhash response: {}", e))
        })?;

    Ok(response.result.value.blockhash)
}

/// Send a signed transaction to Solana RPC
#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
pub async fn send_transaction(
    rpc_url: &str,
    transaction_base64: &str,
    skip_preflight: bool,
) -> IdosResult<String> {
    let client = reqwest::Client::new();
    let request = SendTransactionRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "sendTransaction".to_string(),
        params: (
            transaction_base64.to_string(),
            SendTransactionConfig {
                encoding: "base64".to_string(),
                skip_preflight,
                preflight_commitment: "processed".to_string(),
            },
        ),
    };

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Send transaction request failed: {}", e)))?
        .json::<SendTransactionResponse>()
        .await
        .map_err(|e| {
            IdosError::NetworkError(format!("Failed to parse send transaction response: {}", e))
        })?;

    Ok(response.result)
}
