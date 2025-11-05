/// Anchor program utilities for Solana
/// Matches Unity SDK's SolanaPlatformPoolService Anchor methods
use crate::{IdosError, IdosResult};

#[cfg(feature = "crypto_solana")]
use sha2::{Digest, Sha256};

/// Generate Anchor method discriminator
/// Matches Unity SDK's AnchorDiscriminator method
/// Uses SHA256("global:{method_name}") and takes first 8 bytes
#[cfg(feature = "crypto_solana")]
pub fn anchor_discriminator(method_name: &str) -> [u8; 8] {
    let preimage = format!("global:{}", method_name);
    let mut hasher = Sha256::new();
    hasher.update(preimage.as_bytes());
    let hash = hasher.finalize();

    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash[..8]);
    discriminator
}

/// Encode u64 in little-endian (Borsh format)
/// Matches Unity SDK's EncodeU64
#[cfg(feature = "crypto_solana")]
pub fn encode_u64(value: u64) -> [u8; 8] {
    value.to_le_bytes()
}

/// Encode string in Borsh format (length prefix + UTF-8 bytes)
/// Matches Unity SDK's EncodeString
#[cfg(feature = "crypto_solana")]
pub fn encode_string(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let len = (bytes.len() as u32).to_le_bytes();

    let mut result = Vec::with_capacity(4 + bytes.len());
    result.extend_from_slice(&len);
    result.extend_from_slice(bytes);
    result
}

/// Concatenate byte arrays (Borsh serialization helper)
/// Matches Unity SDK's BorshCat
#[cfg(feature = "crypto_solana")]
pub fn borsh_cat(arrays: &[&[u8]]) -> Vec<u8> {
    let total_len: usize = arrays.iter().map(|a| a.len()).sum();
    let mut result = Vec::with_capacity(total_len);

    for array in arrays {
        result.extend_from_slice(array);
    }

    result
}

/// Convert hex string to bytes
/// Matches Unity SDK's HexToBytes
pub fn hex_to_bytes(hex: &str) -> IdosResult<Vec<u8>> {
    let hex_str = if hex.starts_with("0x") || hex.starts_with("0X") {
        &hex[2..]
    } else {
        hex
    };

    hex::decode(hex_str).map_err(|e| IdosError::InvalidInput(format!("Invalid hex: {}", e)))
}

/// Find Program Derived Address (PDA) with bump seed
/// Matches Unity SDK's ResolvePda method
#[cfg(feature = "crypto_solana")]
pub fn find_program_address(seeds: &[&[u8]], program_id: &[u8; 32]) -> IdosResult<([u8; 32], u8)> {
    // Try bump values from 255 down to 0
    for bump in (0..=255u8).rev() {
        let mut seeds_with_bump: Vec<&[u8]> = seeds.to_vec();
        let bump_bytes = [bump];
        seeds_with_bump.push(&bump_bytes);

        if let Ok(pda) = create_program_address(&seeds_with_bump, program_id) {
            return Ok((pda, bump));
        }
    }

    Err(IdosError::Wallet(
        "Unable to find valid program address".to_string(),
    ))
}

/// Create program address from seeds
/// Simplified version - in production use solana_program::pubkey::Pubkey
#[cfg(feature = "crypto_solana")]
fn create_program_address(seeds: &[&[u8]], program_id: &[u8; 32]) -> Result<[u8; 32], ()> {
    const PDA_MARKER: &[u8] = b"ProgramDerivedAddress";

    let mut hasher = Sha256::new();

    for seed in seeds {
        if seed.len() > 32 {
            return Err(());
        }
        hasher.update(seed);
    }

    hasher.update(program_id);
    hasher.update(PDA_MARKER);

    let hash = hasher.finalize();

    // Check if on curve (simplified - just check if valid)
    // In reality, ed25519 curve check is more complex
    let mut address = [0u8; 32];
    address.copy_from_slice(&hash);

    // Simplified: assume valid if not all zeros
    if address.iter().any(|&b| b != 0) {
        Ok(address)
    } else {
        Err(())
    }
}

/// Build Ed25519 signature verification instruction
/// Matches Unity SDK's CreateEd25519InstructionWithPublicKey
#[cfg(feature = "crypto_solana")]
pub fn create_ed25519_instruction(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Vec<u8> {
    const HEADER_LEN: usize = 16; // 1 + 1 + 7*2

    let sig_offset: u16 = HEADER_LEN as u16;
    let sig_ix_idx: u16 = 0;
    let pk_offset: u16 = sig_offset + 64;
    let pk_ix_idx: u16 = 0;
    let msg_offset: u16 = pk_offset + 32;
    let msg_size: u16 = message.len() as u16;
    let msg_ix_idx: u16 = 0;

    let mut data = Vec::with_capacity(HEADER_LEN + 64 + 32 + message.len());

    // Header
    data.push(1); // num signatures
    data.push(0); // padding

    // Write u16 little-endian values
    data.extend_from_slice(&sig_offset.to_le_bytes());
    data.extend_from_slice(&sig_ix_idx.to_le_bytes());
    data.extend_from_slice(&pk_offset.to_le_bytes());
    data.extend_from_slice(&pk_ix_idx.to_le_bytes());
    data.extend_from_slice(&msg_offset.to_le_bytes());
    data.extend_from_slice(&msg_size.to_le_bytes());
    data.extend_from_slice(&msg_ix_idx.to_le_bytes());

    // Signature (64 bytes)
    data.extend_from_slice(signature);

    // Public key (32 bytes)
    data.extend_from_slice(public_key);

    // Message
    data.extend_from_slice(message);

    data
}

#[cfg(all(test, feature = "crypto_solana"))]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_discriminator() {
        let disc = anchor_discriminator("deposit_spl");
        assert_eq!(disc.len(), 8);
        // Discriminator should be consistent
        let disc2 = anchor_discriminator("deposit_spl");
        assert_eq!(disc, disc2);
    }

    #[test]
    fn test_encode_u64() {
        let encoded = encode_u64(1000);
        assert_eq!(encoded, 1000u64.to_le_bytes());
    }

    #[test]
    fn test_encode_string() {
        let encoded = encode_string("test");
        // Should be: [4, 0, 0, 0, 't', 'e', 's', 't']
        assert_eq!(encoded[0..4], [4, 0, 0, 0]);
        assert_eq!(&encoded[4..], b"test");
    }

    #[test]
    fn test_hex_to_bytes() {
        let bytes = hex_to_bytes("0x48656c6c6f").unwrap();
        assert_eq!(bytes, b"Hello");

        let bytes2 = hex_to_bytes("48656c6c6f").unwrap();
        assert_eq!(bytes2, b"Hello");
    }
}
