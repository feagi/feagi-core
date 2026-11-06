use core::time::Duration;
use feagi_basis::target_frequency::{TargetFrequency, TargetFrequencyError};

#[test]
fn valid_rates_round_trip() {
    let rate = TargetFrequency::new_from_frequency(20.0).unwrap();
    assert_eq!(rate.duration_between_ticks(), Duration::from_millis(50));
    assert_eq!(TargetFrequency::new_from_time_between_ticks(0.05).unwrap(), rate);
    assert_eq!(TargetFrequency::new_from_duration_between_ticks(Duration::from_millis(50)).unwrap(), rate);
    assert_eq!(TargetFrequency::try_from(Duration::from_millis(50)).unwrap(), rate);
}

#[test]
fn frequency_must_be_finite_and_positive() {
    assert_eq!(TargetFrequency::new_from_frequency(0.0), Err(TargetFrequencyError::NotPositive(0.0)));
    assert_eq!(TargetFrequency::new_from_frequency(-5.0), Err(TargetFrequencyError::NotPositive(-5.0)));
    assert!(matches!(TargetFrequency::new_from_frequency(f64::NAN), Err(TargetFrequencyError::NotFinite(_))));
    assert_eq!(TargetFrequency::new_from_frequency(f64::INFINITY), Err(TargetFrequencyError::NotFinite(f64::INFINITY)));
}

#[test]
fn time_between_ticks_must_be_finite_and_positive() {
    assert_eq!(TargetFrequency::new_from_time_between_ticks(0.0), Err(TargetFrequencyError::NotPositive(0.0)));
    assert_eq!(TargetFrequency::new_from_time_between_ticks(-1.0), Err(TargetFrequencyError::NotPositive(-1.0)));
    assert!(matches!(TargetFrequency::new_from_time_between_ticks(f64::NAN), Err(TargetFrequencyError::NotFinite(_))));
}

#[test]
fn rates_that_round_to_zero_or_overflow_are_refused() {
    assert_eq!(TargetFrequency::new_from_duration_between_ticks(Duration::ZERO), Err(TargetFrequencyError::ZeroDuration));
    // Under a nanosecond between ticks
    assert_eq!(TargetFrequency::new_from_frequency(1e12), Err(TargetFrequencyError::ZeroDuration));
    // More seconds between ticks than a `Duration` holds
    assert_eq!(TargetFrequency::new_from_time_between_ticks(1e30), Err(TargetFrequencyError::TooLong));
    // So small that one over it is infinite
    assert_eq!(TargetFrequency::new_from_frequency(1e-320), Err(TargetFrequencyError::TooLong));
}

#[test]
fn time_between_ticks_is_capped_at_24_hours() {
    const DAY: Duration = Duration::from_secs(24 * 60 * 60);
    assert_eq!(TargetFrequency::MAX_DURATION_BETWEEN_TICKS, DAY);

    // Exactly 24 hours is allowed
    assert_eq!(TargetFrequency::new_from_duration_between_ticks(DAY).unwrap().duration_between_ticks(), DAY);
    assert_eq!(TargetFrequency::new_from_time_between_ticks(86_400.0).unwrap().duration_between_ticks(), DAY);

    // Anything longer is refused, however it is given
    assert_eq!(TargetFrequency::new_from_duration_between_ticks(DAY + Duration::from_nanos(1)), Err(TargetFrequencyError::TooLong));
    assert_eq!(TargetFrequency::new_from_time_between_ticks(86_401.0), Err(TargetFrequencyError::TooLong));
    // One tick every two days
    assert_eq!(TargetFrequency::new_from_frequency(1.0 / 172_800.0), Err(TargetFrequencyError::TooLong));

    // And when deserialized
    let two_days = r#"{"duration":{"secs":172800,"nanos":0}}"#;
    assert!(serde_json::from_str::<TargetFrequency>(two_days).is_err());
}

#[test]
fn deserializing_checks_the_rate() {
    let rate = TargetFrequency::new_from_frequency(20.0).unwrap();
    let json = serde_json::to_string(&rate).unwrap();
    assert_eq!(serde_json::from_str::<TargetFrequency>(&json).unwrap(), rate);

    let zero = r#"{"duration":{"secs":0,"nanos":0}}"#;
    assert!(serde_json::from_str::<TargetFrequency>(zero).is_err());
}
