/// NFT loading with Metaplex Token Metadata
use super::dto::*;
use crate::{IdosError, IdosResult};
use serde_json;

#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
use mpl_token_metadata::accounts::Metadata;

/// Get the Metaplex metadata PDA (Program Derived Address) for a mint
#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
pub fn get_metadata_pda(mint: &str) -> IdosResult<String> {
    use solana_sdk::pubkey::Pubkey as SdkPubkey;
    use std::str::FromStr;

    let mint_pubkey = SdkPubkey::from_str(mint)
        .map_err(|e| IdosError::InvalidInput(format!("Invalid mint address: {}", e)))?;

    let metadata_program_id = SdkPubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        .map_err(|e| IdosError::InvalidInput(format!("Invalid program ID: {}", e)))?;

    let seeds = &[
        b"metadata",
        metadata_program_id.as_ref(),
        mint_pubkey.as_ref(),
    ];

    let (pda, _bump) = SdkPubkey::find_program_address(seeds, &metadata_program_id);
    Ok(pda.to_string())
}

/// Parse Metaplex metadata account data
#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
pub fn parse_metadata_account(data: &[u8]) -> IdosResult<NftMetadata> {
    let metadata = Metadata::from_bytes(data)
        .map_err(|e| IdosError::SerializationError(format!("Failed to parse metadata: {}", e)))?;

    Ok(NftMetadata {
        mint: metadata.mint.to_string(),
        name: metadata.name.trim_end_matches('\0').to_string(),
        symbol: metadata.symbol.trim_end_matches('\0').to_string(),
        uri: metadata.uri.trim_end_matches('\0').to_string(),
        seller_fee_basis_points: metadata.seller_fee_basis_points,
        creators: metadata.creators.map(|creators| {
            creators
                .into_iter()
                .map(|c| NftCreator {
                    address: c.address.to_string(),
                    verified: c.verified,
                    share: c.share,
                })
                .collect()
        }),
        primary_sale_happened: metadata.primary_sale_happened,
        is_mutable: metadata.is_mutable,
        update_authority: metadata.update_authority.to_string(),
        collection: metadata.collection.map(|c| NftCollection {
            verified: c.verified,
            key: c.key.to_string(),
        }),
        uses: metadata.uses.map(|u| NftUses {
            use_method: format!("{:?}", u.use_method),
            remaining: u.remaining,
            total: u.total,
        }),
    })
}

/// Load NFTs for a wallet using RPC (WASM compatible via RPC)
pub async fn load_nfts_by_owner(rpc_url: &str, owner_address: &str) -> IdosResult<NftLoadResult> {
    // Get all token accounts owned by this wallet
    let token_accounts = get_token_accounts_by_owner(rpc_url, owner_address).await?;

    let mut nfts = Vec::new();

    for account in token_accounts {
        // Check if this is an NFT (amount = 1, decimals = 0)
        if let Some(ui_amount) = account.token_amount.ui_amount {
            if ui_amount == 1.0 && account.token_amount.decimals == 0 {
                // This is likely an NFT
                match load_nft_metadata(rpc_url, &account.mint, owner_address).await {
                    Ok(nft) => nfts.push(nft),
                    Err(e) => {
                        // Log error but continue with other NFTs
                        log::warn!(
                            "Failed to load NFT metadata for mint {}: {}",
                            account.mint,
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(NftLoadResult {
        count: nfts.len(),
        nfts,
    })
}

/// Get token accounts owned by a wallet
async fn get_token_accounts_by_owner(
    rpc_url: &str,
    owner_address: &str,
) -> IdosResult<Vec<TokenAccountInfo>> {
    let client = reqwest::Client::new();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTokenAccountsByOwner",
        "params": [
            owner_address,
            {
                "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            },
            {
                "encoding": "jsonParsed"
            }
        ]
    });

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(e.to_string()))?;

    let rpc_response: SolanaRpcResponse<TokenAccountsResponse> = response
        .json()
        .await
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    if let Some(error) = rpc_response.error {
        return Err(IdosError::NetworkError(error.message));
    }

    let result = rpc_response
        .result
        .ok_or_else(|| IdosError::NetworkError("No result in response".to_string()))?;

    Ok(result
        .value
        .into_iter()
        .map(|v| v.account.data.parsed)
        .collect())
}

/// Load NFT metadata (on-chain + off-chain)
pub async fn load_nft_metadata(
    rpc_url: &str,
    mint_address: &str,
    owner_address: &str,
) -> IdosResult<Nft> {
    // Get metadata PDA
    #[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
    let metadata_address = get_metadata_pda(mint_address)?;

    #[cfg(not(all(feature = "crypto_solana", not(target_arch = "wasm32"))))]
    let metadata_address = derive_metadata_pda_string(mint_address)?;

    // Get account data from RPC
    let account_data = get_account_data(rpc_url, &metadata_address).await?;

    // Parse metadata
    #[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
    let metadata = parse_metadata_account(&account_data)?;

    #[cfg(not(all(feature = "crypto_solana", not(target_arch = "wasm32"))))]
    let metadata = parse_metadata_from_raw(&account_data)?;

    // Fetch JSON metadata from URI
    let json_metadata = if !metadata.uri.is_empty() {
        match fetch_json_metadata(&metadata.uri).await {
            Ok(json) => Some(json),
            Err(e) => {
                log::warn!("Failed to fetch JSON metadata from {}: {}", metadata.uri, e);
                None
            }
        }
    } else {
        None
    };

    Ok(Nft {
        metadata,
        json_metadata,
        owner: owner_address.to_string(),
    })
}

/// Get account data from RPC
async fn get_account_data(rpc_url: &str, address: &str) -> IdosResult<Vec<u8>> {
    let client = reqwest::Client::new();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [
            address,
            {
                "encoding": "base64"
            }
        ]
    });

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(e.to_string()))?;

    #[derive(serde::Deserialize)]
    struct AccountInfoResponse {
        value: Option<AccountInfo>,
    }

    #[derive(serde::Deserialize)]
    struct AccountInfo {
        data: (String, String), // (data, encoding)
    }

    let rpc_response: SolanaRpcResponse<AccountInfoResponse> = response
        .json()
        .await
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    if let Some(error) = rpc_response.error {
        return Err(IdosError::NetworkError(error.message));
    }

    let result = rpc_response
        .result
        .ok_or_else(|| IdosError::NetworkError("No result in response".to_string()))?;

    let account_info = result
        .value
        .ok_or_else(|| IdosError::NetworkError("Account not found".to_string()))?;

    // Decode base64 data
    use base64::{engine::general_purpose, Engine as _};
    let data = general_purpose::STANDARD
        .decode(&account_info.data.0)
        .map_err(|e| IdosError::SerializationError(format!("Failed to decode base64: {}", e)))?;

    Ok(data)
}

/// Fetch JSON metadata from URI (IPFS, Arweave, etc.)
async fn fetch_json_metadata(uri: &str) -> IdosResult<NftJsonMetadata> {
    // Convert IPFS/Arweave URIs to HTTP gateways
    let http_uri = if uri.starts_with("ipfs://") {
        format!("https://ipfs.io/ipfs/{}", &uri[7..])
    } else if uri.starts_with("ar://") {
        format!("https://arweave.net/{}", &uri[5..])
    } else {
        uri.to_string()
    };

    let client = reqwest::Client::new();
    let response = client
        .get(&http_uri)
        .send()
        .await
        .map_err(|e| IdosError::NetworkError(format!("Failed to fetch metadata: {}", e)))?;

    let json: NftJsonMetadata = response
        .json()
        .await
        .map_err(|e| IdosError::SerializationError(format!("Failed to parse JSON: {}", e)))?;

    Ok(json)
}

/// Derive Metaplex metadata PDA without solana-sdk (WASM fallback)
#[cfg(not(all(feature = "crypto_solana", not(target_arch = "wasm32"))))]
fn derive_metadata_pda_string(mint_address: &str) -> IdosResult<String> {
    // For WASM, we'd need to implement PDA derivation using web3.js
    // Or use a pre-computed PDA from backend
    // For now, return error suggesting RPC-based loading
    Err(IdosError::PlatformNotSupported(
        "PDA derivation in WASM requires web3.js integration. Use backend API for NFT loading."
            .to_string(),
    ))
}

/// Parse metadata from raw bytes without full borsh deserialization (WASM fallback)
#[cfg(not(all(feature = "crypto_solana", not(target_arch = "wasm32"))))]
fn parse_metadata_from_raw(_data: &[u8]) -> IdosResult<NftMetadata> {
    // This is a simplified parser for WASM
    // In production, you'd want to implement full borsh deserialization
    Err(IdosError::PlatformNotSupported(
        "Metadata parsing in WASM requires borsh. Use backend API for NFT loading.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
    async fn test_get_metadata_pda() {
        // Test with a known mint
        let mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let pda = get_metadata_pda(&mint);
        assert_ne!(pda.to_string(), "");
    }
}
