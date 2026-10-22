//! Per-agent output rate gate driven by burst count, not wall-clock time.
//!
//! Motor output used to go out only when at least `1 / rate_hz` of wall-clock
//! time had passed since the last send. The clock was read after each burst's
//! processing, so it jittered with NPU work. At a rate equal to the burst rate,
//! any burst that finished a little faster than the one before looked "too
//! soon" and its output was dropped. An audio stream lost those hops silently.
//!
//! Each burst now adds `rate_hz / burst_hz` of credit. A burst with at least one
//! whole credit is due. Equal rates publish every burst; lower rates keep their
//! exact average (30 Hz of 100 Hz bursts sends 3 of every 10).
//!
//! @cursor:critical-path Runs once per burst per subscribed agent.
//! @cursor:ffi-safe Pure arithmetic, no allocation.

/// Tolerance for floating-point credit that should read as one whole credit.
const CREDIT_EPSILON: f64 = 1e-9;

/// Credit after this burst. An agent with no credit yet is due on its first burst.
///
/// Returns `None` for a rate or burst frequency that is not positive and finite.
pub fn accrue_output_credit(previous: Option<f64>, rate_hz: f64, burst_hz: f64) -> Option<f64> {
    if !rate_hz.is_finite() || rate_hz <= 0.0 || !burst_hz.is_finite() || burst_hz <= 0.0 {
        return None;
    }
    let step = (rate_hz / burst_hz).min(1.0);
    Some(previous.map_or(1.0, |credit| credit + step))
}

/// True when `credit` covers one output.
pub fn is_output_due(credit: f64) -> bool {
    credit + CREDIT_EPSILON >= 1.0
}

/// Credit to keep after this burst.
///
/// A sent output spends one credit and carries the fraction. An output that was
/// due but not sent (no data, transport not ready) holds one credit so it goes
/// out on the next burst without piling up a backlog.
pub fn remaining_output_credit(credit: f64, sent: bool) -> f64 {
    if sent {
        (credit - 1.0).max(0.0)
    } else {
        credit.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sends over `bursts` bursts for a gate that always publishes when due.
    fn sends(rate_hz: f64, burst_hz: f64, bursts: u32) -> u32 {
        let mut credit = None;
        let mut sent = 0;
        for _ in 0..bursts {
            let accrued = accrue_output_credit(credit, rate_hz, burst_hz).expect("valid rates");
            let due = is_output_due(accrued);
            if due {
                sent += 1;
            }
            credit = Some(remaining_output_credit(accrued, due));
        }
        sent
    }

    #[test]
    fn equal_rates_publish_every_burst() {
        assert_eq!(sends(100.0, 100.0, 1000), 1000);
    }

    #[test]
    fn lower_rates_keep_their_exact_average() {
        assert_eq!(sends(30.0, 100.0, 1000), 300);
        assert_eq!(sends(20.0, 60.0, 600), 200);
    }

    #[test]
    fn a_rate_above_the_burst_rate_sends_once_per_burst() {
        assert_eq!(sends(250.0, 100.0, 100), 100);
    }

    #[test]
    fn first_burst_is_due() {
        let credit = accrue_output_credit(None, 10.0, 100.0).expect("valid rates");
        assert!(is_output_due(credit));
    }

    #[test]
    fn a_due_output_that_was_not_sent_goes_out_next_burst_without_a_backlog() {
        let credit = accrue_output_credit(None, 100.0, 100.0).expect("valid rates");
        let held = remaining_output_credit(credit, false);
        assert_eq!(held, 1.0);
        for _ in 0..10 {
            let accrued = accrue_output_credit(Some(held), 100.0, 100.0).expect("valid rates");
            assert!(is_output_due(accrued));
            assert_eq!(remaining_output_credit(accrued, false), 1.0);
        }
    }

    #[test]
    fn invalid_rates_are_rejected() {
        assert!(accrue_output_credit(None, 0.0, 100.0).is_none());
        assert!(accrue_output_credit(None, 10.0, 0.0).is_none());
        assert!(accrue_output_credit(None, f64::NAN, 100.0).is_none());
    }
}
