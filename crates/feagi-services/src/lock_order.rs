// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! The one global lock order for the genome store, the connectome manager and the NPU.
//!
//! A thread that needs more than one of these locks takes them in this order, and never
//! takes an earlier one while holding a later one:
//!
//! 1. genome store (`current_genome`)
//! 2. connectome manager (`ConnectomeManager`)
//! 3. NPU (`Mutex<DynamicNPU>`, taken inside `ConnectomeManager` methods)
//! 4. state manager (`StateManager::instance()`, a leaf lock)
//!
//! The burst loop and sensory injection take only the NPU lock (and leaf mutexes); they read
//! area names from caches that the services refresh, so they never wait on the genome store
//! or the connectome manager. Moves and renames that hold genome + connectome therefore only
//! ever wait on the NPU for the length of one burst.
//!
//! Sites that were taking the genome store and the connectome manager together carry a
//! `Lock order` comment. Reads count: a waiting writer blocks later readers, so a read taken
//! out of order can deadlock too. `tests/test_move_io_groups.rs` runs moves against a bursting,
//! injecting NPU thread to guard the order.
