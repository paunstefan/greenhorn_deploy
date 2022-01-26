use axum::{routing::post, Json, Router};
use hex::FromHex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tower_http::trace::TraceLayer;

const X_HUB_SIGNATURE: &'static str = "X-Hub-Signature";

fn app() -> Router {
    Router::new()
}

fn is_valid_signature(signature: &str, body: &str, secret: &str) -> bool {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();

    mac.update(body.as_bytes());

    let signature_hex = match Vec::from_hex(signature) {
        Ok(v) => v,
        Err(_) => return false,
    };

    mac.verify_slice(&signature_hex).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // - Check inavlid auth
    // - Check valid auth
    // - Check invalid POST request (other branch)
    // - Check valid POST request
    #[test]
    fn test_check_invalid_signature() {
        let string = "this is the string to test";
        let secret = "MYG00DSECRET";
        let sig = "4c9249c483c263000000993fbbe5e781d09fd8c85b0e4ef1f347228d0e78ef00";

        assert!(!is_valid_signature(sig, string, secret));
    }

    #[test]
    fn test_check_valid_signature() {
        let string = "this is the string to test";
        let secret = "MYG00DSECRET";
        let sig = "4c9249c483c26354d1bb993fbbe5e781d09fd8c85b0e4ef1f347228d0e78efe6";

        assert!(is_valid_signature(sig, string, secret));
    }
}
