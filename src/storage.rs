/// Platform-agnostic storage abstraction
#[cfg(target_arch = "wasm32")]
use crate::{IdosError, IdosResult};

#[cfg(not(target_arch = "wasm32"))]
use crate::IdosResult;

#[cfg(target_arch = "wasm32")]
use web_sys::window;

/// Storage interface that works on both native and WASM
#[derive(Clone)]
pub struct Storage {
    prefix: String,
}

impl Storage {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    fn key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// Store a value
    pub fn set(&self, key: &str, value: &str) -> IdosResult<()> {
        let full_key = self.key(key);

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    storage
                        .set_item(&full_key, value)
                        .map_err(|e| IdosError::Unknown(format!("Storage error: {:?}", e)))?;
                    return Ok(());
                }
            }
            Err(IdosError::PlatformNotSupported(
                "LocalStorage not available".to_string(),
            ))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // For native, we could use a file-based system or memory
            // For now, just log it (in a real implementation, use a proper storage)
            let _ = (full_key, value);
            Ok(())
        }
    }

    /// Get a value
    pub fn get(&self, key: &str) -> IdosResult<Option<String>> {
        let full_key = self.key(key);

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    return storage
                        .get_item(&full_key)
                        .map_err(|e| IdosError::Unknown(format!("Storage error: {:?}", e)));
                }
            }
            Err(IdosError::PlatformNotSupported(
                "LocalStorage not available".to_string(),
            ))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = full_key;
            // For native, return None for now
            Ok(None)
        }
    }

    /// Remove a value
    pub fn remove(&self, key: &str) -> IdosResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let full_key = self.key(key);
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    storage
                        .remove_item(&full_key)
                        .map_err(|e| IdosError::Unknown(format!("Storage error: {:?}", e)))?;
                    return Ok(());
                }
            }
            Err(IdosError::PlatformNotSupported(
                "LocalStorage not available".to_string(),
            ))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            Ok(())
        }
    }

    /// Clear all values with this prefix
    pub fn clear(&self) -> IdosResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let length = storage.length().unwrap_or(0);
                    let mut keys_to_remove = Vec::new();

                    for i in 0..length {
                        if let Ok(Some(key)) = storage.key(i) {
                            if key.starts_with(&self.prefix) {
                                keys_to_remove.push(key);
                            }
                        }
                    }

                    for key in keys_to_remove {
                        storage.remove_item(&key).ok();
                    }

                    return Ok(());
                }
            }
            Err(IdosError::PlatformNotSupported(
                "LocalStorage not available".to_string(),
            ))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(())
        }
    }
}
