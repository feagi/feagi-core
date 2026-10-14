use super::error::ZenohConfigBuildError;

/// Prefix applied to every key expression on the session.
///
/// Outgoing key expressions are prefixed with this value, and the prefix is stripped
/// from incoming key expressions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZenohNamespace {
    /// Encoded as the JSON string `namespace`.
    ///
    /// A Zenoh key expression with no `*`, `$`, `?`, or `#`, and no empty `/` chunks.
    pub prefix: String,
}

impl ZenohNamespace {
    /// Builds a namespace prefix.
    ///
    /// Rejects an empty string, a leading or trailing `/`, an empty chunk, and the
    /// key-expression characters `*`, `$`, `?`, and `#`.
    pub fn new(prefix: impl Into<String>) -> Result<Self, ZenohConfigBuildError> {
        let namespace = Self { prefix: prefix.into() };
        namespace.validate()?;
        Ok(namespace)
    }

    /// Checks the prefix against Zenoh key-expression rules for a namespace.
    pub fn validate(&self) -> Result<(), ZenohConfigBuildError> {
        validate_namespace_prefix(&self.prefix)
    }
}

/// Checks a namespace prefix against Zenoh key-expression rules, with wildcards forbidden.
fn validate_namespace_prefix(prefix: &str) -> Result<(), ZenohConfigBuildError> {
    let invalid = |reason: &'static str| ZenohConfigBuildError::InvalidNamespace {
        prefix: prefix.to_string(),
        reason,
    };
    if prefix.is_empty() {
        return Err(invalid("it must not be empty"));
    }
    if prefix.starts_with('/') || prefix.ends_with('/') {
        return Err(invalid("it must not start or end with '/'"));
    }
    for chunk in prefix.split('/') {
        if chunk.is_empty() {
            return Err(invalid("it must not contain an empty chunk"));
        }
        if chunk.bytes().any(|byte| matches!(byte, b'*' | b'$' | b'?' | b'#')) {
            return Err(invalid("it must not contain '*', '$', '?', or '#'"));
        }
    }
    Ok(())
}
