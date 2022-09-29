use axum::{
    extract::Extension, http, response::IntoResponse, routing::post, AddExtensionLayer, Router,
};
use hex::FromHex;
use hmac::{Hmac, Mac};
use hyper::{Body, Request};
use serde_json::Value;
use sha2::Sha256;
use std::{path::PathBuf, sync::Arc};
use tower_http::trace::TraceLayer;

const X_HUB_SIGNATURE: &str = "X-Hub-Signature-256";

struct State {
    path: PathBuf,
    signature: String,
    main_branch: String,
    repo_name: String,
}

/// Initializes the axum routes
pub fn app(path: PathBuf, signature: String, main_branch: String, repo_name: String) -> Router {
    let shared_state = Arc::new(State {
        path,
        signature,
        main_branch,
        repo_name,
    });

    Router::new()
        .route("/payload", post(process_request_body))
        .layer(AddExtensionLayer::new(shared_state))
        .layer(TraceLayer::new_for_http())
}

async fn process_request_body(
    Extension(state): Extension<Arc<State>>,
    payload: Request<Body>,
) -> impl IntoResponse {
    // Get signature from header
    let auth_header = payload
        .headers()
        .get(X_HUB_SIGNATURE)
        .and_then(|header| header.to_str().ok());

    // Split signature as the format is: sha256=[signature]
    let signature = match auth_header {
        Some(auth_header) => auth_header.split('=').nth(1),
        _ => {
            tracing::error!("Signature was not present in header");
            return (
                http::StatusCode::UNAUTHORIZED,
                "Invalid HMAC signature".to_string(),
            );
        }
    };

    // Check if signature format was respected
    let signature = match signature {
        Some(sig) => sig.to_string(),
        None => {
            tracing::error!("Signature was did not respect format");
            return (
                http::StatusCode::UNAUTHORIZED,
                "Invalid HMAC signature".to_string(),
            );
        }
    };

    // Try to turn the body into String
    let body = match hyper::body::to_bytes(payload.into_body()).await {
        Ok(b) => String::from_utf8(b.to_vec())
            .map_err(|e| (http::StatusCode::UNAUTHORIZED, e.to_string())),
        Err(_) => Err((
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Could not receive body".to_string(),
        )),
    };

    // Check if getting body was successful
    let body = match body {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("{}", e.1);
            return e;
        }
    };

    let ret;
    if check_main_branch(&body, &state.repo_name, &state.main_branch) {
        if is_valid_signature(&signature, &body, &state.signature) {
            ret = match crate::pull::execute_pull_request(&state.path) {
                Ok(a) => {
                    tracing::info!("Pull executed: {:?}", a);
                    (http::StatusCode::OK, format!("{:?}", a))
                }
                Err(e) => {
                    tracing::error!("Pull executed: {:?}", e.to_string());
                    (http::StatusCode::OK, e.to_string())
                }
            };
        } else {
            tracing::error!("Invalid HMAC signature: {}", &signature);
            ret = (
                http::StatusCode::UNAUTHORIZED,
                "Invalid HMAC signature".to_string(),
            );
        }
    } else {
        tracing::error!("Not main branch");
        ret = (
            http::StatusCode::UNAUTHORIZED,
            "Not main branch".to_string(),
        );
    }

    ret
}

fn check_main_branch(body: &str, repo_name: &str, main_branch: &str) -> bool {
    let payload: Value = match serde_json::from_str(body) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let branch = &payload["ref"];
    let repo = &payload["repository"]["full_name"];

    repo == repo_name && branch == main_branch
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
