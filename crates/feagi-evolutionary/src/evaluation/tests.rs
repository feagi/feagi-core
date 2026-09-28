// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the `GenomeEvaluation` v1 contract.

use super::*;
use feagi_dataset_contracts::PluginId;

fn hash(fill: char) -> ContentHash {
    ContentHash(format!(
        "sha256:{}",
        fill.to_string().repeat(SHA256_HEX_LEN)
    ))
}

fn plugin(id: &str) -> PluginRef {
    PluginRef {
        id: PluginId(id.to_string()),
        version: "1.0.0".to_string(),
    }
}

fn key() -> ComparabilityKey {
    ComparabilityKey {
        experiment_id: ExperimentId("ex-1".to_string()),
        dataset_asset_id: DatasetAssetId("local:iris".to_string()),
        dataset_version: "1.0.0".to_string(),
        dataset_content_hash: ContentHash("sha256:dataset".to_string()),
        evaluation_protocol_version: EvaluationProtocolVersion("clf-v1".to_string()),
        metric_pack: plugin("classification"),
        reward_policy: plugin("label_reward"),
        fitness: FitnessSpec {
            metric: "accuracy".to_string(),
            split_id: SplitId("val".to_string()),
            objective: FitnessObjective::Maximize,
        },
        run_config_hash: hash('c'),
        genome_schema_version: 3,
        feagi_core_version: "0.0.37".to_string(),
        backend: BackendKind::Cpu,
    }
}

fn manual_evaluation() -> GenomeEvaluation {
    GenomeEvaluation {
        schema_version: SCHEMA_VERSION,
        evaluation_id: EvaluationId("ev-1".to_string()),
        genome_hash: hash('a'),
        key: key(),
        scorecard_ids: vec![
            ScorecardId("sc-val".to_string()),
            ScorecardId("sc-test".to_string()),
        ],
        fitness: FitnessOutcome::Scored(FitnessEstimate {
            value: 0.93,
            n: 1,
            interval: None,
        }),
        lineage: Lineage {
            origin: GenomeOrigin::Manual,
            generation: 0,
            parents: vec![],
        },
        pinned_connectome: None,
    }
}

fn repeated(value: f64, low: f64, high: f64, level: f64) -> FitnessOutcome {
    FitnessOutcome::Scored(FitnessEstimate {
        value,
        n: 5,
        interval: Some(ConfidenceInterval { low, high, level }),
    })
}

fn lineage_error(eval: &GenomeEvaluation) -> bool {
    matches!(eval.validate(), Err(EvaluationError::InvalidLineage(_)))
}

fn fitness_error(eval: &GenomeEvaluation) -> bool {
    matches!(eval.validate(), Err(EvaluationError::InvalidFitness(_)))
}

#[test]
fn manual_generation_zero_is_valid() {
    assert_eq!(manual_evaluation().validate(), Ok(()));
}

#[test]
fn json_round_trip_preserves_record() {
    let mut eval = manual_evaluation();
    eval.fitness = repeated(0.9, 0.85, 0.95, 0.95);
    eval.pinned_connectome = Some(ConnectomeHash("sha256:connectome".to_string()));
    let json = serde_json::to_string(&eval).expect("serialize");
    let restored: GenomeEvaluation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(eval, restored);
}

#[test]
fn wire_form_is_pinned() {
    let value = serde_json::to_value(manual_evaluation()).expect("serialize");
    assert_eq!(
        value["fitness"],
        serde_json::json!({"status": "scored", "value": 0.93, "n": 1})
    );
    assert_eq!(value["lineage"]["origin"], "manual");
    assert_eq!(value["key"]["fitness"]["objective"], "maximize");
    assert_eq!(value["key"]["backend"], "cpu");
    assert_eq!(value["genome_hash"], hash('a').0);
    assert!(value.get("pinned_connectome").is_none());

    let mut incomplete = manual_evaluation();
    incomplete.fitness = FitnessOutcome::NoFitnessSplit;
    let value = serde_json::to_value(incomplete).expect("serialize");
    assert_eq!(
        value["fitness"],
        serde_json::json!({"status": "no_fitness_split"})
    );
}

#[test]
fn only_scored_evaluations_are_selectable() {
    let mut eval = manual_evaluation();
    assert_eq!(eval.selectable_fitness().map(|f| f.value), Some(0.93));
    eval.fitness = FitnessOutcome::NoFitnessSplit;
    assert!(eval.selectable_fitness().is_none());
    assert_eq!(eval.validate(), Ok(()));
    eval.fitness = FitnessOutcome::Incomplete;
    assert!(eval.selectable_fitness().is_none());
    assert_eq!(eval.validate(), Ok(()));
}

#[test]
fn keys_differing_in_run_config_are_not_comparable() {
    let mut other = key();
    other.run_config_hash = hash('d');
    assert_ne!(key(), other);
    assert_eq!(key(), key());
}

#[test]
fn rejects_wrong_schema_version() {
    let mut eval = manual_evaluation();
    eval.schema_version = SCHEMA_VERSION + 1;
    assert!(matches!(
        eval.validate(),
        Err(EvaluationError::SchemaVersion { .. })
    ));
}

#[test]
fn rejects_malformed_hashes() {
    let bad_values = [
        "a".repeat(SHA256_HEX_LEN),
        format!("sha256:{}", "A".repeat(SHA256_HEX_LEN)),
        format!("sha256:{}", "a".repeat(SHA256_HEX_LEN - 1)),
        format!("sha256:{}", "g".repeat(SHA256_HEX_LEN)),
    ];
    for bad in bad_values {
        let mut eval = manual_evaluation();
        eval.genome_hash = ContentHash(bad.clone());
        assert!(
            matches!(
                eval.validate(),
                Err(EvaluationError::InvalidHash {
                    field: "genome_hash",
                    ..
                })
            ),
            "{bad} must be rejected"
        );
    }
    let mut eval = manual_evaluation();
    eval.key.run_config_hash = ContentHash("sha256:short".to_string());
    assert!(matches!(
        eval.validate(),
        Err(EvaluationError::InvalidHash {
            field: "key.run_config_hash",
            ..
        })
    ));
}

#[test]
fn rejects_empty_key_fields() {
    let mut eval = manual_evaluation();
    eval.key.fitness.metric = "  ".to_string();
    assert_eq!(
        eval.validate(),
        Err(EvaluationError::EmptyField("key.fitness.metric"))
    );
    let mut eval = manual_evaluation();
    eval.key.experiment_id = ExperimentId(String::new());
    assert_eq!(
        eval.validate(),
        Err(EvaluationError::EmptyField("key.experiment_id"))
    );
}

#[test]
fn rejects_missing_or_duplicate_scorecards() {
    let mut eval = manual_evaluation();
    eval.scorecard_ids.clear();
    assert_eq!(eval.validate(), Err(EvaluationError::NoScorecards));
    let mut eval = manual_evaluation();
    eval.scorecard_ids.push(ScorecardId("sc-val".to_string()));
    assert_eq!(
        eval.validate(),
        Err(EvaluationError::DuplicateScorecard("sc-val".to_string()))
    );
}

#[test]
fn rejects_invalid_fitness_estimates() {
    let mut eval = manual_evaluation();
    eval.fitness = FitnessOutcome::Scored(FitnessEstimate {
        value: f64::NAN,
        n: 1,
        interval: None,
    });
    assert!(fitness_error(&eval));

    eval.fitness = FitnessOutcome::Scored(FitnessEstimate {
        value: 0.5,
        n: 0,
        interval: None,
    });
    assert!(fitness_error(&eval));

    eval.fitness = FitnessOutcome::Scored(FitnessEstimate {
        value: 0.5,
        n: 1,
        interval: Some(ConfidenceInterval {
            low: 0.4,
            high: 0.6,
            level: 0.95,
        }),
    });
    assert!(fitness_error(&eval));

    eval.fitness = FitnessOutcome::Scored(FitnessEstimate {
        value: 0.5,
        n: 3,
        interval: None,
    });
    assert!(fitness_error(&eval));

    for outcome in [
        repeated(0.5, 0.6, 0.7, 0.95),
        repeated(0.5, 0.4, f64::INFINITY, 0.95),
        repeated(0.5, 0.4, 0.6, 1.0),
        repeated(0.5, 0.4, 0.6, 0.0),
    ] {
        eval.fitness = outcome;
        assert!(fitness_error(&eval));
    }

    eval.fitness = repeated(0.5, 0.4, 0.6, 0.95);
    assert_eq!(eval.validate(), Ok(()));
}

#[test]
fn lineage_rules_per_origin() {
    let mut eval = manual_evaluation();
    eval.lineage.generation = 1;
    assert!(lineage_error(&eval));

    let mut eval = manual_evaluation();
    eval.lineage.origin = GenomeOrigin::Imported;
    eval.lineage.parents = vec![hash('b')];
    assert!(lineage_error(&eval));

    let mut eval = manual_evaluation();
    eval.lineage = Lineage {
        origin: GenomeOrigin::Mutation,
        generation: 1,
        parents: vec![hash('b')],
    };
    assert_eq!(eval.validate(), Ok(()));
    eval.lineage.generation = 0;
    assert!(lineage_error(&eval));
    eval.lineage.generation = 1;
    eval.lineage.parents.push(hash('c'));
    assert!(lineage_error(&eval));

    let mut eval = manual_evaluation();
    eval.lineage = Lineage {
        origin: GenomeOrigin::Crossover,
        generation: 2,
        parents: vec![hash('b'), hash('c')],
    };
    assert_eq!(eval.validate(), Ok(()));
    eval.lineage.parents.truncate(1);
    assert!(lineage_error(&eval));
}

#[test]
fn rejects_duplicate_or_self_parents() {
    let mut eval = manual_evaluation();
    eval.lineage = Lineage {
        origin: GenomeOrigin::Crossover,
        generation: 1,
        parents: vec![hash('b'), hash('b')],
    };
    assert!(lineage_error(&eval));

    eval.lineage = Lineage {
        origin: GenomeOrigin::Mutation,
        generation: 1,
        parents: vec![hash('a')],
    };
    assert!(lineage_error(&eval));

    eval.lineage.parents = vec![ContentHash("sha256:bad".to_string())];
    assert!(matches!(
        eval.validate(),
        Err(EvaluationError::InvalidHash {
            field: "lineage.parents[]",
            ..
        })
    ));
}
