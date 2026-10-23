/// Default timeout applied to queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ZenohQueriesDefaultTimeout {
    /// Milliseconds. Encoded as the JSON number `queries_default_timeout`.
    pub milliseconds: u64,
}

impl ZenohQueriesDefaultTimeout {
    /// Milliseconds used by [`Default`].
    pub const DEFAULT_MILLISECONDS: u64 = 10_000;
}

impl Default for ZenohQueriesDefaultTimeout {
    /// 10000 milliseconds, Zenoh's published query timeout.
    fn default() -> Self {
        Self {
            milliseconds: Self::DEFAULT_MILLISECONDS,
        }
    }
}
