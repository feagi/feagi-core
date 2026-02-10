// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! CI performance benchmark comparator.
//!
//! Compares Criterion outputs under `target/criterion/**/new/estimates.json`
//! against a checked-in baseline JSON and fails (exit 1) on regressions.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use serde_json::Value;

fn usage_and_exit() -> ! {
    eprintln!(
        "Usage: perf_compare [--baseline <path>] [--criterion-dir <path>]\n\n\
         Defaults:\n\
         - baseline: perf/ci_microbench_baseline.json\n\
         - criterion-dir: target/criterion\n"
    );
    process::exit(2);
}

fn parse_args() -> (PathBuf, PathBuf) {
    let mut baseline = PathBuf::from("perf/ci_microbench_baseline.json");
    let mut criterion_dir = PathBuf::from("target/criterion");

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--baseline" => {
                let v = args.next().unwrap_or_else(|| usage_and_exit());
                baseline = PathBuf::from(v);
            }
            "--criterion-dir" => {
                let v = args.next().unwrap_or_else(|| usage_and_exit());
                criterion_dir = PathBuf::from(v);
            }
            "-h" | "--help" => usage_and_exit(),
            other => {
                eprintln!("Unknown argument: {other}");
                usage_and_exit();
            }
        }
    }

    (baseline, criterion_dir)
}

fn read_json(path: &Path) -> Value {
    let raw = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read JSON file {}: {e}", path.display());
        process::exit(2);
    });
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("Failed to parse JSON file {}: {e}", path.display());
        process::exit(2);
    })
}

fn walk_estimates(dir: &Path) -> HashMap<String, Value> {
    let mut out = HashMap::new();

    fn walk(base: &Path, current: &Path, out: &mut HashMap<String, Value>) {
        let entries = match fs::read_dir(current) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out);
                continue;
            }

            if path.file_name().and_then(|n| n.to_str()) != Some("estimates.json") {
                continue;
            }
            // Expect .../<bench...>/new/estimates.json
            if path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                != Some("new")
            {
                continue;
            }

            let rel = match path.strip_prefix(base) {
                Ok(r) => r,
                Err(_) => continue,
            };

            // Build ID: rel path without trailing /new/estimates.json, using '/' separators.
            let rel_str = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let id = rel_str
                .strip_suffix("/new/estimates.json")
                .unwrap_or(&rel_str)
                .to_string();

            out.insert(id, read_json(&path));
        }
    }

    walk(dir, dir, &mut out);
    out
}

fn get_point_estimate(estimates: &Value, stat: &str) -> Option<f64> {
    estimates.get(stat)?.get("point_estimate")?.as_f64()
}

fn main() {
    let (baseline_path, criterion_dir) = parse_args();

    let baseline = read_json(&baseline_path);
    let suite = baseline
        .get("suite")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let stat = baseline
        .get("stat")
        .and_then(|v| v.as_str())
        .unwrap_or("median");
    let default_max_regression_ratio = baseline
        .get("default_max_regression_ratio")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.5);
    let benchmarks = baseline
        .get("benchmarks")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .clone();

    if default_max_regression_ratio <= 1.0 {
        eprintln!(
            "Invalid default_max_regression_ratio in baseline (must be > 1.0): {}",
            default_max_regression_ratio
        );
        process::exit(2);
    }

    let current = walk_estimates(&criterion_dir);
    if current.is_empty() {
        eprintln!(
            "No Criterion estimates found under {}",
            criterion_dir.display()
        );
        process::exit(2);
    }

    if stat != "median" && stat != "mean" {
        eprintln!("Unsupported stat in baseline: {stat} (expected 'median' or 'mean')");
        process::exit(2);
    }

    println!(
        "[perf_compare] suite={} stat={} baseline={} criterion_dir={}",
        suite,
        stat,
        baseline_path.display(),
        criterion_dir.display()
    );

    let mut failed = 0usize;
    for b in benchmarks {
        let Some(id) = b.get("id").and_then(|v| v.as_str()) else {
            eprintln!("[perf_compare] INVALID baseline entry (missing id): {b}");
            failed += 1;
            continue;
        };
        let Some(baseline_ns) = b.get("baseline_ns").and_then(|v| v.as_f64()) else {
            eprintln!(
                "[perf_compare] INVALID baseline entry (missing baseline_ns): {}",
                id
            );
            failed += 1;
            continue;
        };
        let max_ratio = b
            .get("max_regression_ratio")
            .and_then(|v| v.as_f64())
            .unwrap_or(default_max_regression_ratio);

        let Some(estimates) = current.get(id) else {
            eprintln!("[perf_compare] MISSING: {}", id);
            failed += 1;
            continue;
        };

        let Some(current_ns) = get_point_estimate(estimates, stat) else {
            eprintln!("[perf_compare] INVALID estimates.json for {}", id);
            failed += 1;
            continue;
        };

        if baseline_ns <= 0.0 {
            eprintln!(
                "[perf_compare] INVALID baseline_ns for {}: {}",
                id, baseline_ns
            );
            failed += 1;
            continue;
        }

        let ratio = current_ns / baseline_ns;
        let status = if ratio > max_ratio { "FAIL" } else { "OK" };
        println!(
            "[perf_compare] {} {}: baseline={:.3}ns current={:.3}ns ratio={:.3} max={:.3}",
            status, id, baseline_ns, current_ns, ratio, max_ratio
        );
        if ratio > max_ratio {
            failed += 1;
        }
    }

    if failed > 0 {
        eprintln!(
            "[perf_compare] Regression detected: {} failing benchmark(s)",
            failed
        );
        process::exit(1);
    }

    println!("[perf_compare] All benchmarks within regression thresholds.");
}
