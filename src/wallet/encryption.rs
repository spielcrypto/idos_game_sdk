/// Password-based encryption for wallet keys
/// Matches Unity SDK's PrivateKeyManager encryption method
use crate::{IdosError, IdosResult};
use base64::{engine::general_purpose, Engine as _};

/// Encrypt data using XOR cipher (matches Unity SDK implementation)
/// This is the same simple XOR encryption used in Unity's PrivateKeyManager.cs
pub fn encrypt(plain_text: &str, password: &str) -> IdosResult<String> {
    if password.is_empty() {
        return Err(IdosError::InvalidInput(
            "Password cannot be empty".to_string(),
        ));
    }

    let plain_bytes = plain_text.as_bytes();
    let password_bytes = password.as_bytes();
    let mut encrypted_bytes = Vec::with_capacity(plain_bytes.len());

    for (i, &byte) in plain_bytes.iter().enumerate() {
        encrypted_bytes.push(byte ^ password_bytes[i % password_bytes.len()]);
    }

    Ok(general_purpose::STANDARD.encode(&encrypted_bytes))
}

/// Decrypt data using XOR cipher (matches Unity SDK implementation)
pub fn decrypt(encrypted_message: &str, password: &str) -> IdosResult<String> {
    if password.is_empty() {
        return Err(IdosError::InvalidInput(
            "Password cannot be empty".to_string(),
        ));
    }

    let encrypted_bytes = general_purpose::STANDARD
        .decode(encrypted_message)
        .map_err(|e| IdosError::SerializationError(format!("Base64 decode error: {}", e)))?;

    let password_bytes = password.as_bytes();
    let mut plain_bytes = Vec::with_capacity(encrypted_bytes.len());

    for (i, &byte) in encrypted_bytes.iter().enumerate() {
        plain_bytes.push(byte ^ password_bytes[i % password_bytes.len()]);
    }

    String::from_utf8(plain_bytes)
        .map(|s| s.trim_end_matches('\0').to_string())
        .map_err(|e| IdosError::SerializationError(format!("UTF-8 decode error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let plain_text = "test private key 12345";
        let password = "mypassword123";

        let encrypted = encrypt(plain_text, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(plain_text, decrypted);
    }

    #[test]
    fn test_wrong_password() {
        let plain_text = "test private key";
        let password = "correct";
        let wrong_password = "wrong";

        let encrypted = encrypt(plain_text, password).unwrap();
        let decrypted = decrypt(&encrypted, wrong_password).unwrap();

        assert_ne!(plain_text, decrypted);
    }

    #[test]
    fn test_seed_phrase_encryption() {
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let password = "123456";

        let encrypted = encrypt(seed_phrase, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(seed_phrase, decrypted);
    }
}
