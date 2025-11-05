/// Helper functions for WASM Solana operations
#[cfg(target_arch = "wasm32")]
use super::dto::*;
#[cfg(target_arch = "wasm32")]
use crate::{IdosError, IdosResult};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{Request, RequestInit, RequestMode, Response};

/// Send Solana JSON-RPC request (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn send_solana_rpc_request<T: serde::de::DeserializeOwned>(
    rpc_url: &str,
    method: &str,
    params: serde_json::Value,
) -> IdosResult<T> {
    let request_body = SolanaRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: 1,
    };

    let body = serde_json::to_string(&request_body)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&JsValue::from_str(&body)));

    let request = Request::new_with_str_and_init(rpc_url, &opts)
        .map_err(|e| IdosError::NetworkError(format!("Request creation failed: {:?}", e)))?;

    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| IdosError::NetworkError(format!("Header set failed: {:?}", e)))?;

    let window = web_sys::window()
        .ok_or_else(|| IdosError::PlatformNotSupported("No window object".to_string()))?;

    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| IdosError::NetworkError(format!("Fetch failed: {:?}", e)))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| IdosError::NetworkError("Response cast failed".to_string()))?;

    let json = wasm_bindgen_futures::JsFuture::from(
        resp.json()
            .map_err(|e| IdosError::NetworkError(format!("JSON parse failed: {:?}", e)))?,
    )
    .await
    .map_err(|e| IdosError::NetworkError(format!("JSON future failed: {:?}", e)))?;

    let response: SolanaRpcResponse<T> = serde_wasm_bindgen::from_value(json)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    if let Some(error) = response.error {
        return Err(IdosError::NetworkError(format!(
            "Solana RPC Error: {}",
            error.message
        )));
    }

    response
        .result
        .ok_or_else(|| IdosError::NetworkError("No result in response".to_string()))
}

/// Get SOL balance (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn solana_get_balance(rpc_url: &str, address: &str) -> IdosResult<u64> {
    let params = serde_json::json!([address]);
    let balance_response: BalanceResponse =
        send_solana_rpc_request(rpc_url, "getBalance", params).await?;
    Ok(balance_response.value)
}

/// Get SPL token balance (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn solana_get_token_balance(
    rpc_url: &str,
    wallet_address: &str,
    mint_address: &str,
) -> IdosResult<TokenAmount> {
    // First, get token accounts by owner
    let params = serde_json::json!([
        wallet_address,
        {
            "mint": mint_address
        },
        {
            "encoding": "jsonParsed"
        }
    ]);

    let response: super::dto::TokenAccountsResponse =
        send_solana_rpc_request(rpc_url, "getTokenAccountsByOwner", params).await?;

    if let Some(account) = response.value.first() {
        Ok(account.account.data.parsed.token_amount.clone())
    } else {
        // No token account found, balance is 0
        Ok(TokenAmount {
            amount: "0".to_string(),
            decimals: 9,
            ui_amount: Some(0.0),
            ui_amount_string: Some("0".to_string()),
        })
    }
}

/// Get transaction status (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn solana_get_transaction(
    rpc_url: &str,
    signature: &str,
) -> IdosResult<TransactionResult> {
    let params = serde_json::json!([
        signature,
        {
            "encoding": "json",
            "maxSupportedTransactionVersion": 0
        }
    ]);

    match send_solana_rpc_request::<super::dto::TransactionDetailResponse>(
        rpc_url,
        "getTransaction",
        params,
    )
    .await
    {
        Ok(tx) => Ok(TransactionResult {
            signature: signature.to_string(),
            slot: Some(tx.slot),
            confirmed: true,
        }),
        Err(_) => Ok(TransactionResult {
            signature: signature.to_string(),
            slot: None,
            confirmed: false,
        }),
    }
}

/// Solana wallet integration (Phantom, Solflare) (WASM only)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = solana)]
    pub static SOLANA: JsValue;

    #[wasm_bindgen(js_namespace = ["window", "solana"], js_name = connect)]
    pub fn solana_wallet_connect() -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "solana"], js_name = disconnect)]
    pub fn solana_wallet_disconnect() -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "solana"], js_name = signAndSendTransaction)]
    pub fn solana_wallet_sign_and_send(transaction: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "solana"], js_name = signTransaction)]
    pub fn solana_wallet_sign(transaction: JsValue) -> js_sys::Promise;
}

/// Check if Solana wallet is available (Phantom/Solflare) (WASM only)
#[cfg(target_arch = "wasm32")]
pub fn is_solana_wallet_available() -> bool {
    !SOLANA.is_undefined() && !SOLANA.is_null()
}

/// Connect to Solana wallet (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn solana_connect_wallet() -> IdosResult<String> {
    if !is_solana_wallet_available() {
        return Err(IdosError::PlatformNotSupported(
            "Solana wallet not available. Please install Phantom or Solflare".to_string(),
        ));
    }

    let promise = solana_wallet_connect();
    let result = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| IdosError::NetworkError(format!("Wallet connect failed: {:?}", e)))?;

    // Extract publicKey from result
    let public_key = js_sys::Reflect::get(&result, &JsValue::from_str("publicKey"))
        .map_err(|_| IdosError::NetworkError("Failed to get public key".to_string()))?;

    let public_key_str = js_sys::Reflect::get(&public_key, &JsValue::from_str("toString"))
        .and_then(|to_string| {
            let func = to_string
                .dyn_ref::<js_sys::Function>()
                .ok_or(JsValue::NULL)?;
            func.call0(&public_key)
        })
        .and_then(|s| s.as_string().ok_or(JsValue::NULL))
        .map_err(|_| {
            IdosError::NetworkError("Failed to convert public key to string".to_string())
        })?;

    Ok(public_key_str)
}

/// Send transaction via wallet (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn solana_send_transaction(transaction_base64: &str) -> IdosResult<String> {
    if !is_solana_wallet_available() {
        return Err(IdosError::PlatformNotSupported(
            "Solana wallet not available".to_string(),
        ));
    }

    let tx_obj = serde_json::json!({
        "transaction": transaction_base64
    });

    let tx_js = serde_wasm_bindgen::to_value(&tx_obj)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    let promise = solana_wallet_sign_and_send(tx_js);
    let result = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| IdosError::NetworkError(format!("Send transaction failed: {:?}", e)))?;

    let signature = js_sys::Reflect::get(&result, &JsValue::from_str("signature"))
        .and_then(|s| s.as_string().ok_or(JsValue::NULL))
        .map_err(|_| IdosError::NetworkError("Failed to get signature".to_string()))?;

    Ok(signature)
}

/// Deposit SPL token (WASM only - simplified version)
#[cfg(target_arch = "wasm32")]
pub async fn solana_deposit_spl(
    _rpc_url: &str,
    _program_id: &str,
    _mint: &str,
    _amount: u64,
    _user_id: &str,
) -> IdosResult<String> {
    // This would need full transaction building logic
    // For now, return placeholder
    Err(IdosError::PlatformNotSupported(
        "Direct deposit requires transaction building - use backend API or full SDK".to_string(),
    ))
}

/// Withdraw SPL token (WASM only - simplified version)
#[cfg(target_arch = "wasm32")]
pub async fn solana_withdraw_spl(
    _rpc_url: &str,
    _program_id: &str,
    _withdraw_request: WithdrawSplRequest,
) -> IdosResult<String> {
    // This would need full transaction building logic with Ed25519 instruction
    // For now, return placeholder
    Err(IdosError::PlatformNotSupported(
        "Direct withdrawal requires transaction building - use backend API or full SDK".to_string(),
    ))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn placeholder_for_native() {
    // This module is primarily for WASM, native implementations would use solana-client
}
