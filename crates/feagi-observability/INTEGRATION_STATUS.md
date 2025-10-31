# feagi-observability Integration Status

## Current Status

### ✅ Integrated
- **feagi** binary - Uses `feagi-observability` for debug flags
- **feagi-inference-engine** binary - Uses `feagi-observability` for debug flags

### ❌ NOT Integrated (Still using `log` or no logging)
- **feagi-services** - Uses `log = "0.4"` directly
- **feagi-bdu** - Uses `log = "0.4"` directly
- **feagi-evo** - Uses `log = "0.4"` directly
- **feagi-agent-sdk** - Uses `log = "0.4"` directly
- **feagi-api** - Uses `tracing` but NOT `feagi-observability` macros
- **feagi-burst-engine** - No logging dependency at all
- **feagi-pns** - No logging dependency
- **feagi-transports** - No logging dependency
- **feagi-config** - No logging dependency
- **feagi-plasticity** - Unknown (need to check)
- **feagi-state-manager** - Unknown (need to check)
- **feagi-connectome-serialization** - Unknown (need to check)

## Integration Plan

### Phase 1: Core Crates (Priority)
1. **feagi-services** - Replace `log` with `feagi-observability` macros
2. **feagi-bdu** - Replace `log` with `feagi-observability` macros
3. **feagi-evo** - Replace `log` with `feagi-observability` macros
4. **feagi-burst-engine** - Add `feagi-observability` and add logging

### Phase 2: API and SDK Crates
5. **feagi-api** - Add `feagi-observability` dependency, use its macros
6. **feagi-agent-sdk** - Replace `log` with `feagi-observability` macros

### Phase 3: Transport and Infrastructure
7. **feagi-pns** - Add `feagi-observability` for logging
8. **feagi-transports** - Add `feagi-observability` for logging
9. **feagi-config** - Add `feagi-observability` for logging (if needed)

### Phase 4: Remaining Crates
10. Check and integrate remaining crates as needed

## Migration Steps Per Crate

1. Add `feagi-observability` dependency to `Cargo.toml`
2. Replace `log = "0.4"` with `feagi-observability` (or add if missing)
3. Replace `use log::*` with `use tracing::*` (via `feagi-observability`)
4. Replace `log::info!` → `tracing::info!`
5. Replace `log::debug!` → `tracing::debug!`
6. Replace `log::warn!` → `tracing::warn!`
7. Replace `log::error!` → `tracing::error!`
8. Add `#[instrument]` attributes to key functions
9. Use crate-specific debug flags: `tracing::info!(target: "feagi-{crate-name}", ...)`

## Notes

- Foundation crates (`feagi-types`) don't need logging
- Some crates may not need logging at all (e.g., serialization-only crates)
- The `feagi-observability` crate provides re-exports of `tracing`, so crates can use `tracing::*` directly

