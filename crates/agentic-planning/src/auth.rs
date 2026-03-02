//! Server-mode authentication with constant-time token comparison.
//!
//! Reads `AGENTIC_AUTH_TOKEN` and `AGENTIC_AUTH_MODE` from environment.

use std::env;

/// Authentication mode for the MCP/HTTP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthMode {
    /// No authentication required.
    None,
    /// A valid token must be provided.
    Required,
    /// Token is checked if provided, but not mandatory.
    Optional,
}

/// Token-based authenticator.
#[derive(Debug, Clone)]
pub struct TokenAuth {
    mode: AuthMode,
    token: Option<String>,
}

impl TokenAuth {
    /// Create from environment variables.
    ///
    /// - `AGENTIC_AUTH_MODE`: "none" | "required" | "optional" (default: "none")
    /// - `AGENTIC_AUTH_TOKEN`: the secret token
    pub fn from_env() -> Self {
        let mode = match env::var("AGENTIC_AUTH_MODE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "required" => AuthMode::Required,
            "optional" => AuthMode::Optional,
            _ => AuthMode::None,
        };

        let token = env::var("AGENTIC_AUTH_TOKEN")
            .ok()
            .filter(|t| !t.is_empty());

        if mode == AuthMode::Required && token.is_none() {
            eprintln!(
                "WARNING: AGENTIC_AUTH_MODE=required but AGENTIC_AUTH_TOKEN is not set. \
                 All requests will be rejected."
            );
        }

        Self { mode, token }
    }

    /// Create with explicit values (for testing).
    pub fn new(mode: AuthMode, token: Option<String>) -> Self {
        Self { mode, token }
    }

    /// Validate a provided token against the configured secret.
    pub fn validate(&self, provided: Option<&str>) -> Result<(), AuthError> {
        match self.mode {
            AuthMode::None => Ok(()),
            AuthMode::Required => {
                let provided = provided.ok_or(AuthError::TokenMissing)?;
                let expected = self
                    .token
                    .as_deref()
                    .ok_or(AuthError::ServerMisconfigured)?;
                if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
                    Ok(())
                } else {
                    Err(AuthError::TokenInvalid)
                }
            }
            AuthMode::Optional => {
                if let Some(provided) = provided {
                    let expected = self
                        .token
                        .as_deref()
                        .ok_or(AuthError::ServerMisconfigured)?;
                    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
                        Ok(())
                    } else {
                        Err(AuthError::TokenInvalid)
                    }
                } else {
                    // Optional mode: no token provided is OK
                    Ok(())
                }
            }
        }
    }

    pub fn mode(&self) -> &AuthMode {
        &self.mode
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuthError {
    /// Token was required but not provided.
    TokenMissing,
    /// Provided token does not match.
    TokenInvalid,
    /// Auth mode is Required but no server token is configured.
    ServerMisconfigured,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::TokenMissing => write!(f, "authentication token required"),
            AuthError::TokenInvalid => write!(f, "invalid authentication token"),
            AuthError::ServerMisconfigured => {
                write!(f, "server requires auth but no token is configured")
            }
        }
    }
}

impl std::error::Error for AuthError {}

/// Constant-time byte comparison to prevent timing attacks.
///
/// Always compares the full length of `a`, even if `b` differs in length.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        // Still do some work to avoid leaking length via timing,
        // but result is always false.
        let mut _acc: u8 = 1;
        for byte in a {
            _acc |= byte;
        }
        return false;
    }

    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_mode_always_passes() {
        let auth = TokenAuth::new(AuthMode::None, None);
        assert!(auth.validate(None).is_ok());
        assert!(auth.validate(Some("anything")).is_ok());
    }

    #[test]
    fn required_mode_needs_token() {
        let auth = TokenAuth::new(AuthMode::Required, Some("secret123".to_string()));
        assert_eq!(auth.validate(None), Err(AuthError::TokenMissing));
        assert_eq!(auth.validate(Some("wrong")), Err(AuthError::TokenInvalid));
        assert!(auth.validate(Some("secret123")).is_ok());
    }

    #[test]
    fn required_mode_no_server_token() {
        let auth = TokenAuth::new(AuthMode::Required, None);
        assert_eq!(
            auth.validate(Some("anything")),
            Err(AuthError::ServerMisconfigured)
        );
    }

    #[test]
    fn optional_mode_works() {
        let auth = TokenAuth::new(AuthMode::Optional, Some("secret".to_string()));
        assert!(auth.validate(None).is_ok()); // no token is fine
        assert!(auth.validate(Some("secret")).is_ok());
        assert_eq!(auth.validate(Some("wrong")), Err(AuthError::TokenInvalid));
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
        assert!(!constant_time_eq(b"", b"x"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn constant_time_eq_different_lengths() {
        assert!(!constant_time_eq(b"short", b"longer-string"));
        assert!(!constant_time_eq(b"longer-string", b"short"));
    }
}
