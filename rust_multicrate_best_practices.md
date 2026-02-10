
# Best Practices for Organizing a Large Multi-Crate Rust Project

This document describes the recommended structure for managing a large Rust codebase (10–20+ crates) in a single GitHub repository. The structure outlined here follows proven patterns used by major Rust projects including Tokio, Bevy, Serde, WGPU, and Rust Analyzer.

## 1. Use a Single Repository with a Cargo Workspace

A multi-crate Rust project should live in one GitHub repository. This gives you unified issues, a single CI pipeline, easier cross-crate refactoring, atomic PRs across crates, consistent versioning, and simpler dependency management.

At the root of the repository, create a workspace-level `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/*",
    "examples/*",
]

resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
```

## 2. Store All Crates in a `crates/` Directory

Place each crate in its own folder inside `crates/`:

```
crates/
    core/
    gpu/
    wasm/
    runtime/
    io/
    models/
    cli/
    utils/
```

## 3. Provide an Umbrella (Facade) Crate

Create one high-level crate that re-exports the internal crates:

```
crates/neuro/
```

Example `lib.rs`:

```rust
pub use neuro_core as core;
pub use neuro_gpu as gpu;
pub use neuro_wasm as wasm;
pub use neuro_runtime as runtime;
```

## 4. Use Feature Flags to Gate Heavy Crates

Optional dependencies in the umbrella crate allow consumers to enable only what they need.

```toml
[features]
default = ["core"]
gpu = ["neuro_gpu"]
wasm = ["neuro_wasm"]
io   = ["neuro_io"]
```

## 5. Use a `/examples` Directory

Organize runnable examples in:

```
examples/example_name/src/main.rs
```

## 6. Use a `/docs` Directory

Create markdown files documenting architecture, design, and components:

```
docs/
    architecture.md
    roadmap.md
```

## 7. Use GitHub Actions for CI

Set up matrix builds for testing, formatting, clippy, and cross-compilation for WASM and embedded targets.

## 8. Final Project Layout

```
myproject/
  Cargo.toml
  README.md
  LICENSE
  docs/
  scripts/
  benches/
  examples/
  crates/
      neuro/
      neuro_core/
      neuro_runtime/
      neuro_gpu/
      neuro_models/
      neuro_io/
      neuro_wasm/
      neuro_utils/
```
