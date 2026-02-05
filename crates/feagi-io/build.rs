// Build script for feagi-io
fn main() {
    // ==========================================================================
    // Feature Compatibility Check: WASM vs Native transports
    // ==========================================================================
    // WASM features cannot be mixed with native transport features.
    // This is a compile-time check to prevent incompatible feature combinations.

    let wasm_features: &[(&str, bool)] = &[
        ("ws-transport-wasm", cfg!(feature = "websocket-transport-wasm")),
        ("js-bridge-wasm", cfg!(feature = "js-bridge-wasm")),
    ];

    let native_features: &[(&str, bool)] = &[
        ("zmq-transport", cfg!(feature = "zmq-transport")),
        ("ws-transport", cfg!(feature = "websocket-transport-std")),
        ("shm-transport", cfg!(feature = "shm-transport")),
        ("bluetooth-transport", cfg!(feature = "bluetooth-transport")),
    ];

    let enabled_wasm: Vec<&str> = wasm_features
        .iter()
        .filter(|(_, enabled)| *enabled)
        .map(|(name, _)| *name)
        .collect();

    let enabled_native: Vec<&str> = native_features
        .iter()
        .filter(|(_, enabled)| *enabled)
        .map(|(name, _)| *name)
        .collect();

    if !enabled_wasm.is_empty() && !enabled_native.is_empty() {
        panic!(
            "\n\
            ╔══════════════════════════════════════════════════════════════════════════════╗\n\
            ║                    FEAGI-IO: INCOMPATIBLE FEATURE FLAGS                      ║\n\
            ╠══════════════════════════════════════════════════════════════════════════════╣\n\
            ║ WASM transport features cannot be combined with native transport features.   ║\n\
            ║                                                                              ║\n\
            ║ Enabled WASM features:   {:50} ║\n\
            ║ Enabled native features: {:50} ║\n\
            ║                                                                              ║\n\
            ║ Please enable ONLY WASM features OR native features, not both.              ║\n\
            ╚══════════════════════════════════════════════════════════════════════════════╝\n",
            enabled_wasm.join(", "),
            enabled_native.join(", ")
        );
    }

    // On Windows, zmq-sys requires advapi32 for security descriptor functions
    // (InitializeSecurityDescriptor, SetSecurityDescriptorDacl) used in TCP socket creation
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
