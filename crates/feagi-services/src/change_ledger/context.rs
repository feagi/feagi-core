// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Per-request change attribution.
//!
//! Transports set a [`ChangeContext`] around the handling of one request; the
//! recording services read it when they record a change. Work spawned onto another
//! task does not inherit the context, so changes made there are recorded without an
//! agent id and in their own group.

use std::future::Future;

/// Identity of the original change being replayed from another instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaySource {
    pub change_id: String,
    pub group_id: String,
    pub timestamp_ms: i64,
}

/// Attribution applied to changes recorded within one request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChangeContext {
    /// Caller identity (already validated by the transport).
    pub agent_id: Option<String>,
    /// Shared by every change the request produces; `None` gives each change its own group.
    pub group_id: Option<String>,
    /// Set only while replaying a change received from another instance.
    pub replay: Option<ReplaySource>,
}

tokio::task_local! {
    static CHANGE_CONTEXT: ChangeContext;
}

/// Run `future` with `context` as the change attribution for every change it records.
pub async fn with_change_context<F: Future>(context: ChangeContext, future: F) -> F::Output {
    CHANGE_CONTEXT.scope(context, future).await
}

/// Attribution for the current task; empty when no transport set one.
pub fn current_change_context() -> ChangeContext {
    CHANGE_CONTEXT
        .try_with(ChangeContext::clone)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn context_is_visible_inside_scope_only() {
        assert_eq!(current_change_context(), ChangeContext::default());
        let context = ChangeContext {
            agent_id: Some("member-a".to_string()),
            group_id: Some("group-1".to_string()),
            replay: None,
        };
        let seen = with_change_context(context.clone(), async { current_change_context() }).await;
        assert_eq!(seen, context);
        assert_eq!(current_change_context(), ChangeContext::default());
    }

    #[tokio::test]
    async fn spawned_task_does_not_inherit_context() {
        let context = ChangeContext {
            agent_id: Some("member-a".to_string()),
            ..ChangeContext::default()
        };
        let seen = with_change_context(context, async {
            tokio::spawn(async { current_change_context() })
                .await
                .expect("spawned task")
        })
        .await;
        assert_eq!(seen, ChangeContext::default());
    }
}
