/// Authentication method (stub)
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Anonymous (no authentication) - default for now
    Anonymous,
    /// API key authentication (future)
    ApiKey,
    /// JWT authentication (future)
    Jwt,
    /// Mutual TLS (future)
    MutualTls,
}

/// Authentication context (stub)
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Principal ID (user/service identifier)
    pub principal_id: String,
    
    /// Authentication method used
    pub auth_method: AuthMethod,
    
    /// User roles (future RBAC)
    pub roles: Vec<String>,
    
    /// Whether the principal is authenticated
    pub is_authenticated: bool,
}

impl AuthContext {
    /// Create anonymous context (default - no authentication)
    pub fn anonymous() -> Self {
        Self {
            principal_id: "anonymous".to_string(),
            auth_method: AuthMethod::Anonymous,
            roles: vec!["viewer".to_string()],
            is_authenticated: false,
        }
    }

    /// Create authenticated context (future)
    #[allow(dead_code)]
    pub fn authenticated(principal_id: impl Into<String>, method: AuthMethod, roles: Vec<String>) -> Self {
        Self {
            principal_id: principal_id.into(),
            auth_method: method,
            roles,
            is_authenticated: true,
        }
    }

    /// Check if user has role (stub - always returns true for now)
    pub fn has_role(&self, _role: &str) -> bool {
        true  // Stub: always allow
    }

    /// Require authentication (stub - always succeeds for now)
    pub fn require_auth(&self) -> Result<(), AuthError> {
        Ok(())  // Stub: always allow
    }

    /// Require specific role (stub - always succeeds for now)
    pub fn require_role(&self, _role: &str) -> Result<(), AuthError> {
        Ok(())  // Stub: always allow
    }
}

/// Authentication error (stub)
#[derive(Debug, Clone)]
pub struct AuthError {
    pub message: String,
}

impl AuthError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AuthError {}




