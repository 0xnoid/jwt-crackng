use ring::hmac;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

pub fn verify_hmac(alg: hmac::Algorithm, secret: &[u8], message: &[u8], expected_sig: &[u8]) -> bool {
    let key = hmac::Key::new(alg, secret);
    let tag = hmac::sign(&key, message);
    
    ring::constant_time::verify_slices_are_equal(tag.as_ref(), expected_sig).is_ok()
}

pub fn verify_hmac_base64(alg: hmac::Algorithm, secret: &str, message: &[u8], expected_sig: &[u8]) -> bool {
    match URL_SAFE_NO_PAD.decode(secret.as_bytes()) {
        Ok(decoded_secret) => verify_hmac(alg, &decoded_secret, message, expected_sig),
        Err(err) => {
            eprintln!("Failed to decode Base64 secret: {}", err);
            false
        }
    }
}