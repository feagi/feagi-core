//! `feagi-trainer` command-line entry point (plan Phase 1e).
//!
//! Subcommands:
//! - `run --config <path> [--out <path>]` — load a [`RunConfig`] bundle, validate its selectors,
//!   ingest + plan the dataset, then (with the `remote-runtime` feature) drive a closed-loop
//!   rollout against a live FEAGI and emit a `Scorecard`.
//! - `versions` (default) — print the crate identity and contract schema versions.
//!
//! The binary keeps no orchestration logic of its own: it parses arguments, performs file I/O,
//! and delegates to the library ([`feagi_trainer::run_config`]). Transport endpoints are read
//! from the environment at execution time so they never enter run provenance.

use std::error::Error;

use feagi_trainer::contracts::{
    dataset_manifest, evaluation_spec, ir_sample, prediction_record, run_spec, run_summary,
    scorecard,
};
use feagi_trainer::plugins::DatasetSource;
use feagi_trainer::run_config::RunConfig;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("run") => {
            if let Err(error) = run(&args[2..]) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        Some("versions") | None => print_versions(),
        Some(other) => {
            eprintln!("unknown command '{other}'");
            print_usage();
            std::process::exit(2);
        }
    }
}

/// Executes the `run` subcommand: load + validate + plan + execute.
fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (config_path, out_path) = parse_run_args(args)?;

    let json = std::fs::read_to_string(&config_path)?;
    let config = RunConfig::from_json(&json)?;
    config.validate_supported()?;

    let bytes = std::fs::read(&config.dataset.path)?;
    let source = DatasetSource {
        uri: config.dataset.path.clone(),
        bytes,
    };
    let (manifest, samples) = config.plan(&source)?;
    eprintln!(
        "planned {} sample(s) for split '{}'",
        samples.len(),
        config.run_spec.split_id.0
    );

    execute(&config, &manifest, &samples, out_path.as_deref())
}

/// Parses `--config <path>` (required) and `--out <path>` (optional).
fn parse_run_args(args: &[String]) -> Result<(String, Option<String>), Box<dyn Error>> {
    let mut config_path: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                config_path = Some(
                    args.get(i + 1)
                        .ok_or("--config requires a path argument")?
                        .clone(),
                );
                i += 2;
            }
            "--out" => {
                out_path = Some(
                    args.get(i + 1)
                        .ok_or("--out requires a path argument")?
                        .clone(),
                );
                i += 2;
            }
            other => return Err(format!("unexpected argument '{other}'").into()),
        }
    }
    let config_path = config_path.ok_or("missing required --config <path>")?;
    Ok((config_path, out_path))
}

/// Runs the closed-loop rollout against a live FEAGI and writes the scorecard.
///
/// The registration endpoint and burst frequency are read from the environment at execution time
/// so they are never persisted in run provenance.
#[cfg(feature = "remote-runtime")]
fn execute(
    config: &RunConfig,
    manifest: &feagi_trainer::contracts::DatasetManifest,
    samples: &[feagi_trainer::contracts::IRSample],
    out_path: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    use feagi_trainer::run_config::RemoteConnection;

    const ENDPOINT_ENV: &str = "FEAGI_TRAINER_REGISTRATION_ENDPOINT";
    const BURST_HZ_ENV: &str = "FEAGI_TRAINER_BURST_HZ";

    let registration_endpoint = std::env::var(ENDPOINT_ENV)
        .map_err(|_| format!("{ENDPOINT_ENV} must be set (e.g. tcp://127.0.0.1:30001)"))?;
    let burst_raw = std::env::var(BURST_HZ_ENV)
        .map_err(|_| format!("{BURST_HZ_ENV} must be set (FEAGI burst frequency in Hz)"))?;
    let burst_frequency_hz: f64 = burst_raw
        .trim()
        .parse()
        .map_err(|e| format!("invalid {BURST_HZ_ENV} value '{burst_raw}': {e}"))?;

    let connection = RemoteConnection {
        registration_endpoint,
        burst_frequency_hz,
    };
    let (summary, scorecard) = config.execute_remote(manifest, samples, &connection)?;

    let serialized = serde_json::to_string_pretty(&scorecard)?;
    match out_path {
        Some(path) => {
            std::fs::write(path, &serialized)?;
            eprintln!("wrote scorecard to {path}");
        }
        None => println!("{serialized}"),
    }
    eprintln!(
        "run '{}' status {:?}; evaluated {} sample(s); metrics: {:?}",
        summary.run_id.0, summary.status, summary.evaluated_samples, summary.metrics
    );
    Ok(())
}

/// Without the `remote-runtime` feature there is no concrete runtime, so closed-loop execution
/// and scorecard emission are unavailable — fail explicitly rather than silently.
#[cfg(not(feature = "remote-runtime"))]
fn execute(
    _config: &RunConfig,
    _manifest: &feagi_trainer::contracts::DatasetManifest,
    _samples: &[feagi_trainer::contracts::IRSample],
    _out_path: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    Err(
        "closed-loop execution and scorecard emission require building with \
         `--features remote-runtime` and a live FEAGI instance"
            .into(),
    )
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  feagi-trainer run --config <path> [--out <path>]");
    eprintln!("  feagi-trainer versions");
}

fn print_versions() {
    println!("feagi-trainer {}", env!("CARGO_PKG_VERSION"));
    println!("contract schema versions:");
    println!("  DatasetManifest  v{}", dataset_manifest::SCHEMA_VERSION);
    println!("  IRSample         v{}", ir_sample::SCHEMA_VERSION);
    println!("  RunSpec          v{}", run_spec::SCHEMA_VERSION);
    println!("  EvaluationSpec   v{}", evaluation_spec::SCHEMA_VERSION);
    println!("  PredictionRecord v{}", prediction_record::SCHEMA_VERSION);
    println!("  RunSummary       v{}", run_summary::SCHEMA_VERSION);
    println!("  Scorecard        v{}", scorecard::SCHEMA_VERSION);
}
