use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid token format")]
    InvalidFormat,
    #[error("Invalid base64 encoding")]
    InvalidBase64(#[from] base64::DecodeError),
    #[error("Invalid JSON in header")]
    InvalidJson(#[from] serde_json::Error),
    #[error("Invalid or unsupported algorithm: {0}")]
    InvalidAlgorithm(String),
}

pub struct JwtValidator;

impl JwtValidator {
    pub fn validate_token(token: &str) -> Result<bool, ValidationError> {
        if !Self::validate_format(token) {
            return Err(ValidationError::InvalidFormat);
        }
        
        Self::validate_header(token)?;
        Ok(true)
    }

    fn validate_format(token: &str) -> bool {
        let parts: Vec<&str> = token.split('.').collect();
        parts.len() == 3 && parts.iter().all(|part| !part.is_empty())
    }

    fn validate_header(token: &str) -> Result<bool, ValidationError> {
        let parts: Vec<&str> = token.split('.').collect();
        let header_bytes = URL_SAFE_NO_PAD.decode(parts[0])?;
        let header: Value = serde_json::from_slice(&header_bytes)?;

        match header.get("alg").and_then(Value::as_str) {
            Some(alg) => match alg {
                "HS256" | "HS384" | "HS512" => Ok(true),
                _ => Err(ValidationError::InvalidAlgorithm(alg.to_string())),
            },
            None => Err(ValidationError::InvalidAlgorithm("missing".to_string())),
        }
    }
}