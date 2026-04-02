use anyhow::Result;
use serde_json::{Value, json};
use std::env;
use thiserror::Error;

use crate::http_client::blocking_client;
use crate::meta::GoogleAuthError; // Re-use the existing error type

const FIRESTORE_BASE_URL: &str = "https://firestore.googleapis.com/v1/projects";

#[derive(Error, Debug)]
pub enum FirestoreError {
    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    JsonParseFailed(#[from] serde_json::Error),
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    #[error("Value field not found in document")]
    ValueFieldNotFound,
    #[error("Invalid value type in document: {0}")]
    InvalidValueType(String),
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Google authentication error: {0}")]
    GoogleAuth(#[from] GoogleAuthError),
}

/// Constructs the full Firestore document URL.
fn get_env_required(name: &str) -> Result<String, FirestoreError> {
    env::var(name).map_err(|_| FirestoreError::MissingEnvVar(name.to_string()))
}

/// Constructs the full Firestore document URL.
fn get_document_url(project_id: &str, document_id: &str) -> Result<String, FirestoreError> {
    let database_id = get_env_required("UZUMIBI_DATABASE_ID")?;
    let collection_id = get_env_required("UZUMIBI_COLLECTION_ID")?;
    Ok(format!(
        "{}/{}/databases/{}/documents/{}/{}",
        FIRESTORE_BASE_URL, project_id, database_id, collection_id, document_id
    ))
}

/// Retrieves a string value from a Firestore document.
///
/// Assumes the document has a structure like:
/// ```json
/// {
///   "fields": {
///     "value": {
///       "stringValue": "your_string_value"
///     }
///   }
/// }
/// ```
///
/// # Arguments
/// * `project_id` - The Google Cloud project ID.
/// * `auth_token` - The authorization token (e.g., from metadata server).
/// * `key` - The ID of the document to retrieve.
///
/// # Returns
/// A `Result` which is:
/// - `Ok(String)` containing the retrieved string value if successful.
/// - `Err(FirestoreError)` if an error occurs (e.g., document not found, parsing error).
pub fn get_document(
    project_id: &str,
    auth_token: &str,
    key: &str,
) -> Result<String, FirestoreError> {
    let client = blocking_client();
    let url = get_document_url(project_id, key)?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()?
        .error_for_status();

    match response {
        Ok(res) => {
            let doc: Value = res.json()?;
            let value = doc
                .get("fields")
                .and_then(|f| f.get("value"))
                .and_then(|v| v.get("stringValue"))
                .and_then(|s| s.as_str())
                .ok_or(FirestoreError::ValueFieldNotFound)?
                .to_string();
            Ok(value)
        }
        Err(e) => {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                Err(FirestoreError::DocumentNotFound(key.to_string()))
            } else {
                Err(FirestoreError::HttpRequestFailed(e))
            }
        }
    }
}

/// Sets a string value in a Firestore document.
///
/// The document will have a structure like:
/// ```json
/// {
///   "fields": {
///     "value": {
///       "stringValue": "your_string_value"
///     }
///   }
/// }
/// ```
///
/// # Arguments
/// * `project_id` - The Google Cloud project ID.
/// * `auth_token` - The authorization token (e.g., from metadata server).
/// * `key` - The ID of the document to set.
/// * `value` - The string value to set.
///
/// # Returns
/// A `Result` which is:
/// - `Ok(bool)` true if the operation was successful.
/// - `Err(FirestoreError)` if an error occurs.
pub fn set_document(
    project_id: &str,
    auth_token: &str,
    key: &str,
    value: &str,
) -> Result<bool, FirestoreError> {
    let client = blocking_client();
    let url = get_document_url(project_id, key)?;

    let body = json!({
        "fields": {
            "value": {
                "stringValue": value
            }
        }
    });

    let response = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?
        .error_for_status()?;

    Ok(response.status().is_success())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use mockito::{mock, Matcher};

//     // Helper to create a mock Firestore URL
//     fn mock_firestore_url(project_id: &str, document_id: &str) -> String {
//         format!(
//             "/v1/projects/{}/databases/{}/documents/{}/{}",
//             project_id, DATABASE_ID, COLLECTION_ID, document_id
//         )
//     }

//     #[test]
//     fn test_get_document_success() -> Result<(), FirestoreError> {
//         let _m = mock("GET", Matcher::Regex(r"/v1/projects/test-project/databases/\(default\)/documents/uzumibi_data/test-doc".to_string()))
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_body(r#"{"fields":{"value":{"stringValue":"hello_firestore"}}}"#)
//             .create();

//         let value = get_document("test-project", "test-token", "test-doc")?;
//         assert_eq!(value, "hello_firestore");
//         Ok(())
//     }

//     #[test]
//     fn test_get_document_not_found() -> Result<(), FirestoreError> {
//         let _m = mock("GET", Matcher::Regex(r"/v1/projects/test-project/databases/\(default\)/documents/uzumibi_data/non-existent-doc".to_string()))
//             .with_status(404)
//             .create();

//         let result = get_document("test-project", "test-token", "non-existent-doc");
//         assert!(matches!(result, Err(FirestoreError::DocumentNotFound(_))));
//         Ok(())
//     }

//     #[test]
//     fn test_get_document_value_field_not_found() -> Result<(), FirestoreError> {
//         let _m = mock("GET", Matcher::Regex(r"/v1/projects/test-project/databases/\(default\)/documents/uzumibi_data/bad-doc".to_string()))
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_body(r#"{"fields":{"other_field":{"stringValue":"something"}}}"#)
//             .create();

//         let result = get_document("test-project", "test-token", "bad-doc");
//         assert!(matches!(result, Err(FirestoreError::ValueFieldNotFound)));
//         Ok(())
//     }

//     #[test]
//     fn test_set_document_success() -> Result<(), FirestoreError> {
//         let _m = mock("PATCH", Matcher::Regex(r"/v1/projects/test-project/databases/\(default\)/documents/uzumibi_data/set-doc".to_string()))
//             .with_status(200)
//             .with_header("content-type", "application/json")
//             .with_body_from_fn(|body| {
//                 let parsed: Value = serde_json::from_str(body).unwrap();
//                 parsed["fields"]["value"]["stringValue"].as_str() == Some("new_value")
//             })
//             .create();

//         let success = set_document("test-project", "test-token", "set-doc", "new_value")?;
//         assert!(success);
//         Ok(())
//     }
// }
