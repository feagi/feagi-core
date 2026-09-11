// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Test that quantization_precision is correctly parsed from genome JSON

use feagi_evolutionary::genome::loader::load_genome_from_file;
use std::path::PathBuf;
use std::str::FromStr;
