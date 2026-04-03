use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

use crate::http_client::blocking_client;

const METADATA_SERVER_URL: &str =
    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";
const PROJECT_ID_METADATA_URL: &str =
    "http://metadata.google.internal/computeMetadata/v1/project/project-id";
const PROJECT_NUMBER_METADATA_URL: &str =
    "http://metadata.google.internal/computeMetadata/v1/project/numeric-project-id";

#[derive(Error, Debug)]
pub enum GoogleAuthError {
    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    JsonParseFailed(#[from] serde_json::Error),
    #[error("Access token not found in response")]
    AccessTokenNotFound,
    #[error("Invalid metadata server response: {0}")]
    InvalidMetadataResponse(String),
    #[error("Project ID not found in response")]
    ProjectIdNotFound,
    #[error("Project number not found in response")]
    ProjectNumberNotFound,
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    #[allow(unused)]
    expires_in: u32,
    #[allow(unused)]
    token_type: String,
}

/// Obtains an authorization token from the Google Cloud metadata server.
///
/// This function is intended to be used within Google Cloud environments (e.g., Compute Engine, Cloud Run)
/// where the metadata server is accessible. It makes a blocking HTTP request to the metadata server
/// to retrieve an access token for the default service account.
///
/// # Returns
/// A `Result` which is:
/// - `Ok(String)` containing the access token if successful.
/// - `Err(GoogleAuthError)` if an error occurs during the request, JSON parsing, or if the token is not found.
pub fn get_authorization_token_from_metadata() -> Result<String, GoogleAuthError> {
    let client = blocking_client();
    let response = client
        .get(METADATA_SERVER_URL)
        .header("Metadata-Flavor", "Google")
        .send()?
        .error_for_status()?;

    let token_response: TokenResponse = response.json()?;

    if token_response.access_token.is_empty() {
        return Err(GoogleAuthError::AccessTokenNotFound);
    }

    Ok(token_response.access_token)
}

/// Obtains the Google Cloud project ID from the metadata server.
///
/// This function is intended to be used within Google Cloud environments (e.g., Compute Engine, Cloud Run)
/// where the metadata server is accessible. It makes a blocking HTTP request to the metadata server
/// to retrieve the project ID.
///
/// # Returns
/// A `Result` which is:
/// - `Ok(String)` containing the project ID if successful.
/// - `Err(GoogleAuthError)` if an error occurs during the request or if the project ID is not found.
pub fn get_project_id_from_metadata() -> Result<String, GoogleAuthError> {
    let client = blocking_client();
    let response = client
        .get(PROJECT_ID_METADATA_URL)
        .header("Metadata-Flavor", "Google")
        .send()?
        .error_for_status()?;

    let project_id = response.text()?;

    if project_id.is_empty() {
        return Err(GoogleAuthError::ProjectIdNotFound);
    }

    Ok(project_id)
}

/// Obtains the Google Cloud numeric project ID from the metadata server.
pub fn get_project_number_from_metadata() -> Result<String, GoogleAuthError> {
    let client = blocking_client();
    let response = client
        .get(PROJECT_NUMBER_METADATA_URL)
        .header("Metadata-Flavor", "Google")
        .send()?
        .error_for_status()?;

    let project_number = response.text()?;

    if project_number.is_empty() {
        return Err(GoogleAuthError::ProjectNumberNotFound);
    }

    Ok(project_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_response_deserialization() {
        let json = r#"{
            "access_token": "test_token_123",
            "expires_in": 3600,
            "token_type": "Bearer"
        }"#;
        let response: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "test_token_123");
        assert_eq!(response.expires_in, 3600);
        assert_eq!(response.token_type, "Bearer");
    }

    #[test]
    fn test_token_response_deserialization_missing_token() {
        let json = r#"{
            "expires_in": 3600,
            "token_type": "Bearer"
        }"#;
        let result: Result<TokenResponse, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
