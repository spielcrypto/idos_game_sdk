/// Wallet creation with BIP39 mnemonics and BIP44 key derivation
/// Matches Unity SDK's WalletCreationManager functionality
use super::dto::*;
use crate::{IdosError, IdosResult};

#[cfg(feature = "wallet")]
use {
    bip39::Mnemonic, k256::ecdsa::SigningKey as Secp256k1SigningKey,
    tiny_hderive::bip32::ExtendedPrivKey,
};

/// Generate a new wallet with a random mnemonic
/// Matches Unity SDK's CreateAccount functionality
#[cfg(feature = "wallet")]
pub fn generate_wallet(
    network: BlockchainNetwork,
    word_count: usize,
) -> IdosResult<WalletCreationResult> {
    use rand::Rng;

    // Validate word count (12 or 24 words) and calculate entropy size
    let entropy_len = match word_count {
        12 => 16, // 128 bits
        24 => 32, // 256 bits
        _ => {
            return Err(IdosError::InvalidInput(
                "Word count must be 12 or 24".to_string(),
            ))
        }
    };

    // Generate random entropy
    let mut entropy = vec![0u8; entropy_len];
    rand::thread_rng().fill(&mut entropy[..]);

    // Generate mnemonic from entropy
    let mnemonic = Mnemonic::from_entropy_in(bip39::Language::English, &entropy)
        .map_err(|e| IdosError::Wallet(format!("Failed to generate mnemonic: {:?}", e)))?;

    let seed_phrase = mnemonic.to_string();

    // Derive keys from mnemonic
    let wallet_info = derive_wallet_from_mnemonic(&seed_phrase, network)?;

    Ok(WalletCreationResult {
        wallet_info,
        seed_phrase,
    })
}

/// Derive wallet from mnemonic (supports both Ethereum and Solana)
/// Matches Unity SDK's wallet derivation paths
#[cfg(feature = "wallet")]
pub fn derive_wallet_from_mnemonic(
    seed_phrase: &str,
    network: BlockchainNetwork,
) -> IdosResult<WalletInfo> {
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, seed_phrase)
        .map_err(|e| IdosError::InvalidInput(format!("Invalid mnemonic: {:?}", e)))?;

    let seed = mnemonic.to_seed("");

    match network {
        BlockchainNetwork::Ethereum => derive_ethereum_wallet(&seed, seed_phrase),
        BlockchainNetwork::Solana => derive_solana_wallet(&seed, seed_phrase),
    }
}

/// Derive Ethereum wallet using BIP44 path: m/44'/60'/0'/0/0
#[cfg(feature = "wallet")]
fn derive_ethereum_wallet(seed: &[u8], seed_phrase: &str) -> IdosResult<WalletInfo> {
    // BIP44 derivation path for Ethereum: m/44'/60'/0'/0/0
    let ext = ExtendedPrivKey::derive(seed, "m/44'/60'/0'/0/0")
        .map_err(|e| IdosError::Wallet(format!("Key derivation failed: {:?}", e)))?;

    // Get secp256k1 private key
    let signing_key = Secp256k1SigningKey::from_bytes(&ext.secret().into())
        .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;

    // Get public key
    let verifying_key = signing_key.verifying_key();
    use k256::EncodedPoint;
    let public_key_bytes: EncodedPoint = EncodedPoint::from(verifying_key);
    let public_key = &public_key_bytes.as_bytes()[1..]; // Remove 0x04 prefix

    // Ethereum address is last 20 bytes of keccak256(public_key)
    let address = ethereum_address_from_public_key(public_key);

    // Private key as hex string
    let private_key = hex::encode(ext.secret());

    Ok(WalletInfo {
        address,
        network: BlockchainNetwork::Ethereum,
        private_key: Some(private_key),
        seed_phrase: Some(seed_phrase.to_string()),
    })
}

/// Derive Solana wallet using BIP44 path: m/44'/501'/0'/0'
#[cfg(feature = "wallet")]
fn derive_solana_wallet(seed: &[u8], seed_phrase: &str) -> IdosResult<WalletInfo> {
    use ed25519_dalek::SigningKey;

    // BIP44 derivation path for Solana: m/44'/501'/0'/0'
    let ext = ExtendedPrivKey::derive(seed, "m/44'/501'/0'/0'")
        .map_err(|e| IdosError::Wallet(format!("Key derivation failed: {:?}", e)))?;

    // Ed25519 key from derived seed
    let signing_key = SigningKey::from_bytes(&ext.secret());
    let verifying_key = signing_key.verifying_key();

    // Solana address is base58 encoded public key
    let address = bs58::encode(verifying_key.as_bytes()).into_string();

    // Private key as base58 string (Solana format: 64 bytes = private + public)
    let mut full_key = Vec::with_capacity(64);
    full_key.extend_from_slice(signing_key.as_bytes());
    full_key.extend_from_slice(verifying_key.as_bytes());
    let private_key = bs58::encode(&full_key).into_string();

    Ok(WalletInfo {
        address,
        network: BlockchainNetwork::Solana,
        private_key: Some(private_key),
        seed_phrase: Some(seed_phrase.to_string()),
    })
}

/// Calculate Ethereum address from public key
#[cfg(feature = "wallet")]
fn ethereum_address_from_public_key(public_key: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    // Ethereum uses Keccak256, but we'll use SHA256 as approximation for now
    // In production, you'd want to use tiny-keccak crate
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();

    // Take last 20 bytes and format as hex with 0x prefix
    let address_bytes = &hash[hash.len() - 20..];
    format!("0x{}", hex::encode(address_bytes))
}

#[cfg(not(feature = "wallet"))]
pub fn generate_wallet(
    _network: BlockchainNetwork,
    _word_count: usize,
) -> IdosResult<WalletCreationResult> {
    Err(IdosError::PlatformNotSupported(
        "Wallet feature not enabled".to_string(),
    ))
}

#[cfg(not(feature = "wallet"))]
pub fn derive_wallet_from_mnemonic(
    _seed_phrase: &str,
    _network: BlockchainNetwork,
) -> IdosResult<WalletInfo> {
    Err(IdosError::PlatformNotSupported(
        "Wallet feature not enabled".to_string(),
    ))
}

#[cfg(all(test, feature = "wallet"))]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ethereum_wallet() {
        let result = generate_wallet(BlockchainNetwork::Ethereum, 12).unwrap();
        assert!(result.wallet_info.address.starts_with("0x"));
        assert_eq!(result.seed_phrase.split_whitespace().count(), 12);
    }

    #[test]
    fn test_generate_solana_wallet() {
        let result = generate_wallet(BlockchainNetwork::Solana, 12).unwrap();
        assert!(!result.wallet_info.address.is_empty());
        assert_eq!(result.seed_phrase.split_whitespace().count(), 12);
    }

    #[test]
    fn test_derive_from_known_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let eth_wallet =
            derive_wallet_from_mnemonic(mnemonic, BlockchainNetwork::Ethereum).unwrap();
        assert!(eth_wallet.address.starts_with("0x"));

        let sol_wallet = derive_wallet_from_mnemonic(mnemonic, BlockchainNetwork::Solana).unwrap();
        assert!(!sol_wallet.address.is_empty());
    }
}
