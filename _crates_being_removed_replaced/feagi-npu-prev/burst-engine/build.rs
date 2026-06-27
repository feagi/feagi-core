// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Build script for CUDA kernel compilation
 *
 * This script compiles .cu CUDA kernel files to PTX at build time
 * Handles graceful degradation when CUDA toolkit is not available
 */

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/backend/shaders/cuda/");

    // Only compile CUDA kernels if cuda feature is enabled
    if !cfg!(feature = "cuda") {
        return;
    }

    println!("CUDA feature enabled, attempting PTX compilation...");

    // Check if nvcc is available
    let nvcc_available = Command::new("nvcc").arg("--version").output().is_ok();

    if !nvcc_available {
        eprintln!("nvcc not found in PATH. CUDA kernels will not be compiled.");
        eprintln!("Install CUDA Toolkit to enable CUDA support.");
        eprintln!("Build will continue but CUDA backend will fail at runtime.");
        return;
    }

    // Get output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Compile each CUDA kernel to PTX
    compile_kernel(
        "src/backend/shaders/cuda/synaptic_propagation_fcl.cu",
        &out_dir.join("synaptic_propagation_fcl.ptx"),
    );

    compile_kernel(
        "src/backend/shaders/cuda/neural_dynamics_fcl.cu",
        &out_dir.join("neural_dynamics_fcl.ptx"),
    );

    println!("CUDA PTX compilation complete");
}

fn compile_kernel(input: &str, output: &PathBuf) {
    println!("cargo:rerun-if-changed={}", input);
    println!("Compiling {} to PTX...", input);

    let status = Command::new("nvcc")
        .arg("--ptx") // Compile to PTX
        .arg("-O3") // Optimize
        .arg("--std=c++14") // C++14 standard
        .arg("--gpu-architecture=sm_70") // Compute Capability 7.0 (Volta+)
        .arg("-o")
        .arg(output) // Output file
        .arg(input) // Input file
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("Compiled {} successfully", input);
        }
        Ok(status) => {
            panic!("nvcc failed with status: {}", status);
        }
        Err(e) => {
            panic!("Failed to run nvcc: {}", e);
        }
    }
}
