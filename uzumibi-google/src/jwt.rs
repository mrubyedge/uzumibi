use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

const GOOGLE_IAP_ISSUER: &str = "https://cloud.google.com/iap";
const GOOGLE_IAP_AUDIENCE_PREFIX: &str = "/projects/";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Invalid JWT token: {0}")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),
    #[error("Missing kid in JWT header")]
    MissingKid,
    #[error("Failed to fetch Google public keys: {0}")]
    KeyFetchError(String),
    #[error("No matching public key found for kid: {0}")]
    NoMatchingKey(String),
    #[error("Invalid audience: {0}")]
    InvalidAudience(String),
    #[error("Invalid issuer: {0}")]
    InvalidIssuer(String),
}

/// Decodes and validates a Google Cloud IAP JWT token.
///
/// This function performs the following validations:
/// 1. Checks the token's signature using Google's public keys.
/// 2. Verifies the issuer (`iss`) is "https://cloud.google.com/iap".
/// 3. Verifies the audience (`aud`) matches the expected audience for the IAP protected resource.
/// 4. Checks the token's expiration time (`exp`).
///
/// # Arguments
/// * `token` - The JWT token string.
/// * `expected_audience` - The expected audience for the token, typically in the format
///   "/projects/PROJECT_NUMBER/apps/APP_ID" or "/projects/PROJECT_NUMBER/global/backendServices/SERVICE_ID".
///
/// # Returns
/// A `Result` which is:
/// - `Ok(Claims)` containing the decoded claims if validation is successful.
/// - `Err(JwtError)` if validation fails or an error occurs during decoding.
pub fn validate_iap_jwt(token: &str, expected_audience: &str) -> Result<Claims, JwtError> {
    let header = decode_header(token)?;
    let kid = header.kid.ok_or(JwtError::MissingKid)?;

    let public_keys = fetch_google_public_keys()?;
    let decoding_key = public_keys
        .get(&kid)
        .ok_or_else(|| JwtError::NoMatchingKey(kid.clone()))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.validate_nbf = false; // IAP tokens do not typically have nbf
    validation.iss = Some(HashSet::from([GOOGLE_IAP_ISSUER.to_string()]));
    validation.aud = Some(HashSet::from([expected_audience.to_string()]));

    let decoded_token = decode::<Claims>(token, decoding_key, &validation)?;

    // Additional check for audience format, as IAP audience can be complex
    if !decoded_token.claims.aud.starts_with(GOOGLE_IAP_AUDIENCE_PREFIX) {
        return Err(JwtError::InvalidAudience(format!(
            "Audience does not start with expected prefix: {}",
            decoded_token.claims.aud
        )));
    }

    Ok(decoded_token.claims)
}

// Fetches Google's public keys for JWT validation.
// These keys are cached for efficiency.
fn fetch_google_public_keys() -> Result<std::collections::HashMap<String, DecodingKey>, JwtError> {
    // In a real application, you would cache these keys and refresh them periodically.
    // For simplicity, this example fetches them every time.
    let client = reqwest::blocking::Client::new();
    let res: std::collections::HashMap<String, String> = client
        .get("https://www.gstatic.com/iap/verify/public_key")
        .send()
        .map_err(|e| JwtError::KeyFetchError(format!("Failed to send request: {}", e)))?
        .json()
        .map_err(|e| JwtError::KeyFetchError(format!("Failed to parse JSON: {}", e)))?;

    let mut keys = std::collections::HashMap::new();
    for (kid, pem) in res {
        keys.insert(kid, DecodingKey::from_rsa_pem(pem.as_bytes())?);
    }
    Ok(keys)
}
