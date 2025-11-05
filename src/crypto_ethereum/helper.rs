/// Helper functions for WASM Ethereum operations
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

/// Send JSON-RPC request (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn send_rpc_request<T: serde::de::DeserializeOwned>(
    rpc_url: &str,
    method: &str,
    params: serde_json::Value,
) -> IdosResult<T> {
    let request_body = JsonRpcRequest {
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

    let response: JsonRpcResponse<T> = serde_wasm_bindgen::from_value(json)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    if let Some(error) = response.error {
        return Err(IdosError::NetworkError(format!(
            "RPC Error: {}",
            error.message
        )));
    }

    response
        .result
        .ok_or_else(|| IdosError::NetworkError("No result in response".to_string()))
}

/// Get ETH balance (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn eth_get_balance(rpc_url: &str, address: &str) -> IdosResult<String> {
    let params = serde_json::json!([address, "latest"]);
    send_rpc_request::<String>(rpc_url, "eth_getBalance", params).await
}

/// Call ERC20 balanceOf function (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn eth_call_balance_of(
    rpc_url: &str,
    wallet_address: &str,
    token_address: &str,
) -> IdosResult<String> {
    // ERC20 balanceOf selector: 0x70a08231
    let selector = "0x70a08231";
    let address_padded = format!("{:0>64}", wallet_address.trim_start_matches("0x"));
    let data = format!("{}{}", selector, address_padded);

    let call_data = serde_json::json!({
        "to": token_address,
        "data": data
    });

    let params = serde_json::json!([call_data, "latest"]);
    send_rpc_request::<String>(rpc_url, "eth_call", params).await
}

/// Call ERC20 allowance function (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn eth_call_allowance(
    rpc_url: &str,
    token_address: &str,
    owner_address: &str,
    spender_address: &str,
) -> IdosResult<String> {
    // ERC20 allowance selector: 0xdd62ed3e
    let selector = "0xdd62ed3e";
    let owner_padded = format!("{:0>64}", owner_address.trim_start_matches("0x"));
    let spender_padded = format!("{:0>64}", spender_address.trim_start_matches("0x"));
    let data = format!("{}{}{}", selector, owner_padded, spender_padded);

    let call_data = serde_json::json!({
        "to": token_address,
        "data": data
    });

    let params = serde_json::json!([call_data, "latest"]);
    send_rpc_request::<String>(rpc_url, "eth_call", params).await
}

/// Get transaction receipt (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn eth_get_transaction_receipt(
    rpc_url: &str,
    transaction_hash: &str,
) -> IdosResult<EthTransactionReceipt> {
    let params = serde_json::json!([transaction_hash]);
    send_rpc_request::<EthTransactionReceipt>(rpc_url, "eth_getTransactionReceipt", params).await
}

/// Get transaction count (nonce) (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn eth_get_transaction_count(rpc_url: &str, address: &str) -> IdosResult<String> {
    let params = serde_json::json!([address, "latest"]);
    send_rpc_request::<String>(rpc_url, "eth_getTransactionCount", params).await
}

/// Send raw transaction (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn eth_send_raw_transaction(
    rpc_url: &str,
    signed_transaction: &str,
) -> IdosResult<String> {
    let params = serde_json::json!([signed_transaction]);
    send_rpc_request::<String>(rpc_url, "eth_sendRawTransaction", params).await
}

/// MetaMask integration (WASM only)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = ethereum)]
    pub static ETHEREUM: JsValue;

    #[wasm_bindgen(js_namespace = ["window", "ethereum"], js_name = request)]
    pub fn ethereum_request(args: JsValue) -> js_sys::Promise;
}

/// Check if MetaMask is available (WASM only)
#[cfg(target_arch = "wasm32")]
pub fn is_metamask_available() -> bool {
    !ETHEREUM.is_undefined() && !ETHEREUM.is_null()
}

/// Request MetaMask accounts (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn metamask_request_accounts() -> IdosResult<Vec<String>> {
    if !is_metamask_available() {
        return Err(IdosError::PlatformNotSupported(
            "MetaMask not available".to_string(),
        ));
    }

    let request = serde_json::json!({
        "method": "eth_requestAccounts"
    });

    let request_js = serde_wasm_bindgen::to_value(&request)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    let promise = ethereum_request(request_js);
    let result = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| IdosError::NetworkError(format!("MetaMask request failed: {:?}", e)))?;

    let accounts: Vec<String> = serde_wasm_bindgen::from_value(result)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    Ok(accounts)
}

/// Get current MetaMask chain ID (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn metamask_get_chain_id() -> IdosResult<String> {
    if !is_metamask_available() {
        return Err(IdosError::PlatformNotSupported(
            "MetaMask not available".to_string(),
        ));
    }

    let request = serde_json::json!({
        "method": "eth_chainId"
    });

    let request_js = serde_wasm_bindgen::to_value(&request)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    let promise = ethereum_request(request_js);
    let result = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| IdosError::NetworkError(format!("MetaMask request failed: {:?}", e)))?;

    let chain_id: String = serde_wasm_bindgen::from_value(result)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    Ok(chain_id)
}

/// Send transaction via MetaMask (WASM only)
#[cfg(target_arch = "wasm32")]
pub async fn metamask_send_transaction(transaction: EthTransaction) -> IdosResult<String> {
    if !is_metamask_available() {
        return Err(IdosError::PlatformNotSupported(
            "MetaMask not available".to_string(),
        ));
    }

    let request = serde_json::json!({
        "method": "eth_sendTransaction",
        "params": [transaction]
    });

    let request_js = serde_wasm_bindgen::to_value(&request)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    let promise = ethereum_request(request_js);
    let result = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| IdosError::NetworkError(format!("MetaMask transaction failed: {:?}", e)))?;

    let tx_hash: String = serde_wasm_bindgen::from_value(result)
        .map_err(|e| IdosError::SerializationError(e.to_string()))?;

    Ok(tx_hash)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn placeholder_for_native() {
    // This module is primarily for WASM, native implementations are in handler.rs
}
