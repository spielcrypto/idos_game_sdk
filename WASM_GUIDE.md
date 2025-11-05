# WASM Build Guide for iDos Games SDK

## ✅ WASM Support Confirmed

The library successfully compiles for both:
- **Native** (Linux, Windows, macOS)
- **WebAssembly** (wasm32-unknown-unknown)

## Quick WASM Build

```bash
# Build for WASM
cargo build --release --target wasm32-unknown-unknown --features all

# Or use the included script
./build_wasm.sh
```

## Key WASM Configurations

### 1. **Platform-Specific Dependencies**

The SDK uses different HTTP clients and async runtimes:

**Native (Tokio):**
```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.12", features = ["rustls-tls"] }
```

**WASM (wasm-bindgen):**
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = [...] }
js-sys = "0.3"
getrandom = { version = "0.2", features = ["js"] }
```

### 2. **Async Code Pattern**

Use platform-specific spawning:

```rust
#[cfg(target_arch = "wasm32")]
{
    wasm_bindgen_futures::spawn_local(async move {
        // Your async code
    });
}

#[cfg(not(target_arch = "wasm32"))]
{
    tokio::spawn(async move {
        // Your async code
    });
}
```

### 3. **Storage**

- **WASM**: Uses browser LocalStorage
- **Native**: Extensible (currently no-op, can add file storage)

## Features

All features work on both platforms:
- ✅ Authentication (email, guest, social*, wallet*)
- ✅ Analytics (event tracking)
- ✅ IAP (in-app purchases)
- ✅ HTTP client (platform-agnostic)
- ✅ Storage (LocalStorage on web)

*Social and wallet login optimized for WASM/web

## Integration Example

```rust
use bevy::prelude::*;
use idos_game_sdk::{IdosGamesPlugin, IdosConfig};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IdosGamesPlugin::new(IdosConfig {
            api_key: "your_key".to_string(),
            game_id: "your_game".to_string(),
            ..default()
        }))
        .run();
}
```

## Testing

```bash
# Native
cargo check --features all

# WASM
cargo check --target wasm32-unknown-unknown --features all
```

Both should compile successfully! ✅


