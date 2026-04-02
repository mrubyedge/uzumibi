use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, decode, decode_header,
    jwk::{JwkSet, KeyAlgorithm},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

use crate::http_client::blocking_client;

const GOOGLE_IAP_ISSUER: &str = "https://cloud.google.com/iap";
const GOOGLE_IAP_AUDIENCE_PREFIX: &str = "/projects/";
const GOOGLE_IAP_JWK_URL: &str = "https://www.gstatic.com/iap/verify/public_key-jwk";

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
    #[error("Failed to parse JWT header: {0}")]
    InvalidHeader(jsonwebtoken::errors::Error),
    #[error("Missing kid in JWT header")]
    MissingKid,
    #[error("Failed to fetch Google public keys: {0}")]
    KeyFetchError(String),
    #[error("Failed to parse Google public keys response: {0}")]
    KeyResponseParseError(String),
    #[error("Unsupported JWT algorithm for IAP validation: {0}")]
    UnsupportedAlgorithm(String),
    #[error("No matching public key found for kid: {0}")]
    NoMatchingKey(String),
    #[error("Public key algorithm mismatch for kid {kid}: token={token_alg}, key={key_alg}")]
    KeyAlgorithmMismatch {
        kid: String,
        token_alg: String,
        key_alg: String,
    },
    #[error("Failed to build decoding key for kid {kid}: {source}")]
    InvalidPublicKeyFormat {
        kid: String,
        source: jsonwebtoken::errors::Error,
    },
    #[error("JWT signature or claim validation failed: {0}")]
    ValidationFailed(jsonwebtoken::errors::Error),
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
    let header = decode_header(token).map_err(JwtError::InvalidHeader)?;
    let algorithm = header.alg;
    let kid = header.kid.ok_or(JwtError::MissingKid)?;

    let decoding_key = fetch_google_public_key(&kid, algorithm)?;

    let mut validation = Validation::new(algorithm);
    validation.validate_exp = true;
    validation.validate_nbf = false; // IAP tokens do not typically have nbf
    validation.iss = Some(HashSet::from([GOOGLE_IAP_ISSUER.to_string()]));
    validation.aud = Some(HashSet::from([expected_audience.to_string()]));

    let decoded_token =
        decode::<Claims>(token, &decoding_key, &validation).map_err(JwtError::ValidationFailed)?;

    // Additional check for audience format, as IAP audience can be complex
    if !decoded_token
        .claims
        .aud
        .starts_with(GOOGLE_IAP_AUDIENCE_PREFIX)
    {
        return Err(JwtError::InvalidAudience(format!(
            "Audience does not start with expected prefix: {}",
            decoded_token.claims.aud
        )));
    }

    Ok(decoded_token.claims)
}

fn fetch_google_public_key(kid: &str, algorithm: Algorithm) -> Result<DecodingKey, JwtError> {
    ensure_supported_algorithm(algorithm)?;

    let client = blocking_client();
    let jwk_set: JwkSet = client
        .get(GOOGLE_IAP_JWK_URL)
        .send()
        .map_err(|e| JwtError::KeyFetchError(format!("Failed to send request: {}", e)))?
        .json()
        .map_err(|e| JwtError::KeyResponseParseError(format!("{}", e)))?;

    let jwk = jwk_set
        .find(kid)
        .ok_or_else(|| JwtError::NoMatchingKey(kid.to_string()))?;

    if let Some(key_alg) = jwk.common.key_algorithm
        && !matches_algorithm(key_alg, algorithm)
    {
        return Err(JwtError::KeyAlgorithmMismatch {
            kid: kid.to_string(),
            token_alg: format!("{:?}", algorithm),
            key_alg: format!("{:?}", key_alg),
        });
    }

    DecodingKey::from_jwk(jwk).map_err(|source| JwtError::InvalidPublicKeyFormat {
        kid: kid.to_string(),
        source,
    })
}

fn ensure_supported_algorithm(algorithm: Algorithm) -> Result<(), JwtError> {
    if matches!(
        algorithm,
        Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512
            | Algorithm::ES256
            | Algorithm::ES384
            | Algorithm::EdDSA
    ) {
        Ok(())
    } else {
        Err(JwtError::UnsupportedAlgorithm(format!("{:?}", algorithm)))
    }
}

fn matches_algorithm(key_alg: KeyAlgorithm, token_alg: Algorithm) -> bool {
    matches!(
        (key_alg, token_alg),
        (KeyAlgorithm::RS256, Algorithm::RS256)
            | (KeyAlgorithm::RS384, Algorithm::RS384)
            | (KeyAlgorithm::RS512, Algorithm::RS512)
            | (KeyAlgorithm::PS256, Algorithm::PS256)
            | (KeyAlgorithm::PS384, Algorithm::PS384)
            | (KeyAlgorithm::PS512, Algorithm::PS512)
            | (KeyAlgorithm::ES256, Algorithm::ES256)
            | (KeyAlgorithm::ES384, Algorithm::ES384)
            | (KeyAlgorithm::EdDSA, Algorithm::EdDSA)
            | (KeyAlgorithm::UNKNOWN_ALGORITHM, _)
    )
}
