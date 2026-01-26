// Build script for feagi-io
fn main() {
    // On Windows, zmq-sys requires advapi32 for security descriptor functions
    // (InitializeSecurityDescriptor, SetSecurityDescriptorDacl) used in TCP socket creation
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
