use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use crate::crypto;
use ring::hmac;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Invalid token format")]
    InvalidFormat,
    #[error("Invalid base64 encoding")]
    InvalidBase64(#[from] base64::DecodeError),
}

pub struct JwtParts {
    pub content: String,
    pub expected_sig: String,
}

pub fn parse_token(token: &str) -> Result<JwtParts, JwtError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(JwtError::InvalidFormat);
    }

    Ok(JwtParts {
        content: format!("{}.{}", parts[0], parts[1]),
        expected_sig: parts[2].to_string(),
    })
}

pub fn verify_signature(jwt: &JwtParts, secret: &str, algorithm: &str, use_base64: bool) -> bool {
    let decoded_secret = if use_base64 {
        match URL_SAFE_NO_PAD.decode(secret.as_bytes()) {
            Ok(decoded) => decoded,
            Err(err) => {
                eprintln!("Failed to decode Base64 secret: {}", err);
                return false;
            },
        }
    } else {
        secret.as_bytes().to_vec()
    };

    let decoded_expected_sig = match URL_SAFE_NO_PAD.decode(jwt.expected_sig.as_bytes()) {
        Ok(decoded) => decoded,
        Err(err) => {
            eprintln!("Failed to decode Base64 expected signature: {}", err);
            return false;
        },
    };

    match algorithm.to_uppercase().as_str() {
        "HS256" | "HMACSHA256" => crypto::verify_hmac(hmac::HMAC_SHA256, &decoded_secret, jwt.content.as_bytes(), &decoded_expected_sig),
        "HS384" | "HMACSHA384" => crypto::verify_hmac(hmac::HMAC_SHA384, &decoded_secret, jwt.content.as_bytes(), &decoded_expected_sig),
        "HS512" | "HMACSHA512" => crypto::verify_hmac(hmac::HMAC_SHA512, &decoded_secret, jwt.content.as_bytes(), &decoded_expected_sig),
        _ => {
            eprintln!("Unsupported algorithm: {}", algorithm);
            false
        },
    }
}