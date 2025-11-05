/// High-level Solana platform pool service  
/// Matches Unity SDK's SolanaPlatformPoolService API exactly
use super::{anchor::*, dto::*, handler::SolanaHandler, transactions::*};
use crate::{IdosError, IdosResult};

/// Solana Platform Pool Service
/// Provides deposit and withdrawal functionality for SPL tokens
/// Matches Unity SDK's SolanaPlatformPoolService.cs
pub struct SolanaPlatformPoolService {
    handler: SolanaHandler,
    private_key: Option<Vec<u8>>, // 64 bytes for Solana (32 secret + 32 public)
}

impl SolanaPlatformPoolService {
    pub fn new(handler: SolanaHandler) -> Self {
        Self {
            handler,
            private_key: None,
        }
    }

    /// Set private key for signing transactions (base58 format)
    pub fn set_private_key(&mut self, private_key_base58: &str) -> IdosResult<()> {
        let key_bytes = bs58::decode(private_key_base58)
            .into_vec()
            .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;

        if key_bytes.len() != 64 {
            return Err(IdosError::Wallet(
                "Solana private key must be 64 bytes".to_string(),
            ));
        }

        self.private_key = Some(key_bytes);
        Ok(())
    }

    /// Clear private key from memory
    pub fn clear_private_key(&mut self) {
        self.private_key = None;
    }

    fn get_private_key(&self) -> IdosResult<&[u8]> {
        self.private_key
            .as_deref()
            .ok_or_else(|| IdosError::Wallet("Private key not set".to_string()))
    }

    fn get_public_key(&self) -> IdosResult<[u8; 32]> {
        let key = self.get_private_key()?;
        let mut pubkey = [0u8; 32];
        pubkey.copy_from_slice(&key[32..]); // Second half is public key
        Ok(pubkey)
    }

    /// Deposit SPL tokens to platform pool
    /// Matches Unity SDK's DepositSplAsync
    /// Returns transaction signature
    #[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
    pub async fn deposit_spl(
        &self,
        mint_address: &str,
        amount: u64,
        user_id: &str,
    ) -> IdosResult<String> {
        let settings = self.handler.settings();
        let rpc_url = &settings.rpc_url;
        let program_id_str = &settings.program_id;

        // Parse addresses
        let program_id_bytes = bs58::decode(program_id_str)
            .into_vec()
            .map_err(|e| IdosError::InvalidInput(format!("Invalid program ID: {}", e)))?;
        let mut program_id = [0u8; 32];
        program_id.copy_from_slice(&program_id_bytes);

        let mint_bytes = bs58::decode(mint_address)
            .into_vec()
            .map_err(|e| IdosError::InvalidInput(format!("Invalid mint address: {}", e)))?;
        let mut mint = [0u8; 32];
        mint.copy_from_slice(&mint_bytes);

        let user_pubkey = self.get_public_key()?;

        // Derive PDAs
        let (config_pda, _) = find_program_address(&[b"config"], &program_id)?;
        let (vault_pda, _) = find_program_address(&[b"vault"], &program_id)?;

        // Derive ATAs
        let user_ata = derive_associated_token_account(&user_pubkey, &mint)?;
        let vault_ata = derive_associated_token_account(&vault_pda, &mint)?;

        // Build deposit instruction
        let deposit_ix = build_deposit_spl_instruction(
            &program_id,
            &config_pda,
            &vault_pda,
            &mint,
            &user_pubkey,
            &user_ata,
            &vault_ata,
            amount,
            user_id,
        );

        // Build, sign, and send transaction
        let blockhash = get_recent_blockhash(rpc_url).await?;

        let mut tx_builder = TransactionBuilder::new(user_pubkey);
        tx_builder
            .add_instruction(deposit_ix)
            .set_recent_blockhash(&blockhash);

        let signed_tx = tx_builder.sign_and_serialize(self.get_private_key()?)?;

        // Send transaction (with preflight checks)
        let signature = send_transaction(rpc_url, &signed_tx, false).await?;

        Ok(signature)
    }

    #[cfg(any(not(feature = "crypto_solana"), target_arch = "wasm32"))]
    pub async fn deposit_spl(
        &self,
        _mint_address: &str,
        _amount: u64,
        _user_id: &str,
    ) -> IdosResult<String> {
        Err(IdosError::PlatformNotSupported(
            "Native Solana transaction building requires full solana-sdk. Use WASM wallet adapter or backend API.".to_string(),
        ))
    }

    /// Withdraw SPL tokens from platform pool with backend signature
    /// Matches Unity SDK's WithdrawSplAsync
    #[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
    pub async fn withdraw_spl(&self, withdraw_request: WithdrawSplRequest) -> IdosResult<String> {
        let settings = self.handler.settings();
        let rpc_url = &settings.rpc_url;
        let program_id_str = &settings.program_id;

        // Parse program ID
        let program_id_bytes = bs58::decode(program_id_str)
            .into_vec()
            .map_err(|e| IdosError::InvalidInput(format!("Invalid program ID: {}", e)))?;
        let mut program_id = [0u8; 32];
        program_id.copy_from_slice(&program_id_bytes);

        // Parse mint and recipient
        let mint_bytes = bs58::decode(&withdraw_request.mint)
            .into_vec()
            .map_err(|e| IdosError::InvalidInput(format!("Invalid mint: {}", e)))?;
        let mut mint = [0u8; 32];
        mint.copy_from_slice(&mint_bytes);

        let to_bytes = bs58::decode(&withdraw_request.to)
            .into_vec()
            .map_err(|e| IdosError::InvalidInput(format!("Invalid recipient: {}", e)))?;
        let mut to_pubkey = [0u8; 32];
        to_pubkey.copy_from_slice(&to_bytes);

        let payer_pubkey = self.get_public_key()?;

        // Derive PDAs
        let (config_pda, _) = find_program_address(&[b"config"], &program_id)?;
        let (vault_pda, _) = find_program_address(&[b"vault"], &program_id)?;

        // Derive nonce marker PDA
        let nonce_bytes = encode_u64(withdraw_request.nonce);
        let (nonce_marker_pda, _) = find_program_address(&[b"nonce", &nonce_bytes], &program_id)?;

        // Derive ATAs
        let vault_ata = derive_associated_token_account(&vault_pda, &mint)?;
        let to_ata = derive_associated_token_account(&to_pubkey, &mint)?;

        // Build Ed25519 verification instruction
        let ed25519_pubkey = hex_to_bytes(&withdraw_request.ed25519_public_key_hex)?;
        let ed25519_message = hex_to_bytes(&withdraw_request.ed25519_message_hex)?;
        let ed25519_signature = hex_to_bytes(&withdraw_request.ed25519_signature_hex)?;

        let mut ed_pubkey = [0u8; 32];
        let mut ed_sig = [0u8; 64];
        ed_pubkey.copy_from_slice(&ed25519_pubkey);
        ed_sig.copy_from_slice(&ed25519_signature);

        let ed25519_ix = create_ed25519_instruction(&ed_pubkey, &ed25519_message, &ed_sig);

        // Convert Ed25519 instruction data to TransactionInstruction
        let ed25519_program_id = bs58::decode(ED25519_PROGRAM_ID).into_vec().unwrap();
        let mut ed_program_id = [0u8; 32];
        ed_program_id.copy_from_slice(&ed25519_program_id);

        let ed25519_tx_ix = TransactionInstruction {
            program_id: ed_program_id,
            accounts: vec![],
            data: ed25519_ix,
        };

        // Build withdraw instruction
        let withdraw_ix = build_withdraw_spl_instruction(
            &program_id,
            &config_pda,
            &payer_pubkey,
            &vault_pda,
            &nonce_marker_pda,
            &mint,
            &to_pubkey,
            &vault_ata,
            &to_ata,
            withdraw_request.amount,
            withdraw_request.nonce,
            &withdraw_request.user_id,
            withdraw_request.sig_ix_index,
        );

        // Build, sign, and send transaction with both instructions
        let blockhash = get_recent_blockhash(rpc_url).await?;

        let mut tx_builder = TransactionBuilder::new(payer_pubkey);
        tx_builder
            .add_instruction(ed25519_tx_ix)
            .add_instruction(withdraw_ix)
            .set_recent_blockhash(&blockhash);

        let signed_tx = tx_builder.sign_and_serialize(self.get_private_key()?)?;

        // Send transaction (with preflight checks)
        let signature = send_transaction(rpc_url, &signed_tx, false).await?;

        Ok(signature)
    }

    #[cfg(any(not(feature = "crypto_solana"), target_arch = "wasm32"))]
    pub async fn withdraw_spl(&self, _withdraw_request: WithdrawSplRequest) -> IdosResult<String> {
        Err(IdosError::PlatformNotSupported(
            "Native Solana transaction building requires full solana-sdk. Use WASM wallet adapter or backend API.".to_string(),
        ))
    }

    /// Helper: Derive PDA from string seeds
    fn derive_pda_from_seeds(seeds: &[&str], program_id: &[u8; 32]) -> IdosResult<([u8; 32], u8)> {
        let byte_seeds: Vec<&[u8]> = seeds.iter().map(|s| s.as_bytes()).collect();
        find_program_address(&byte_seeds, program_id)
    }
}
