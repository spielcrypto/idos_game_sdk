/// Configuration for iDos Games SDK
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct IdosConfig {
    /// Your iDos Games API key
    pub api_key: String,

    /// Your game ID
    pub game_id: String,

    /// API base URL (default: https://api.idosgames.com)
    pub api_url: String,

    /// Enable debug logging
    pub debug: bool,

    /// Enable analytics
    pub enable_analytics: bool,

    /// Enable crash reporting
    pub enable_crash_reporting: bool,

    /// Platform-specific settings
    pub platform: PlatformConfig,
}

impl Default for IdosConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            game_id: String::new(),
            api_url: "https://api.idosgames.com".to_string(),
            debug: cfg!(debug_assertions),
            enable_analytics: true,
            enable_crash_reporting: true,
            platform: PlatformConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// WASM-specific configuration
    #[cfg(target_arch = "wasm32")]
    pub wasm: WasmConfig,

    /// Native-specific configuration
    #[cfg(not(target_arch = "wasm32"))]
    pub native: NativeConfig,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            wasm: WasmConfig::default(),

            #[cfg(not(target_arch = "wasm32"))]
            native: NativeConfig::default(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Use local storage for caching
    pub use_local_storage: bool,

    /// Storage key prefix
    pub storage_prefix: String,

    /// Enable Web3 wallet integration (MetaMask, etc.)
    pub enable_web3: bool,

    /// Enable Solana wallet integration (Phantom, etc.)
    pub enable_solana_wallet: bool,
}

#[cfg(target_arch = "wasm32")]
impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            use_local_storage: true,
            storage_prefix: "idos_sdk_".to_string(),
            enable_web3: true,
            enable_solana_wallet: true,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeConfig {
    /// Cache directory
    pub cache_dir: Option<std::path::PathBuf>,

    /// Enable native crypto wallets
    pub enable_native_wallets: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for NativeConfig {
    fn default() -> Self {
        Self {
            cache_dir: None,
            enable_native_wallets: true,
        }
    }
}
