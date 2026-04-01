use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const PUBSUB_BASE_URL: &str = "https://pubsub.googleapis.com/v1";

/// Pub/Sub message payload for subscriber-side JSON.
///
/// This is intended for deserializing raw request bodies delivered by Pub/Sub.
#[derive(Deserialize, Debug)]
pub struct ReceivedPubsubMessage {
    pub data: Option<String>,
    pub attributes: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "messageId", alias = "message_id")]
    pub message_id: Option<String>,
    #[serde(rename = "publishTime", alias = "publish_time")]
    pub publish_time: Option<String>,
    #[serde(rename = "orderingKey")]
    pub ordering_key: Option<String>,
}

/// Subscriber-side wrapper used for pull responses or normalized internal handling.
#[derive(Deserialize, Debug)]
pub struct ReceivedMessage {
    #[serde(rename = "ackId", alias = "ack_id")]
    pub ack_id: Option<String>,
    pub message: ReceivedPubsubMessage,
    #[serde(rename = "deliveryAttempt", alias = "delivery_attempt")]
    pub delivery_attempt: Option<i32>,
}

/// Push endpoint request body wrapper.
#[derive(Deserialize, Debug)]
pub struct PushRequestBody {
    pub message: ReceivedPubsubMessage,
    pub subscription: Option<String>,
    #[serde(rename = "deliveryAttempt", alias = "delivery_attempt")]
    pub delivery_attempt: Option<i32>,
}

#[derive(Serialize, Debug)]
pub struct PubsubMessage {
    pub data: Option<String>,
    pub attributes: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "messageId")]
    pub message_id: Option<String>,
    #[serde(rename = "publishTime")]
    pub publish_time: Option<String>,
    #[serde(rename = "orderingKey")]
    pub ordering_key: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct PublishRequest {
    pub messages: Vec<PubsubMessage>,
}

#[derive(Deserialize, Debug)]
pub struct PublishResponse {
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
}

/// Publishes one or more messages to the specified topic.
///
/// # Arguments
/// * `access_token` - The OAuth 2.0 access token.
/// * `topic` - The full resource name of the topic (e.g., "projects/your-project-id/topics/your-topic-name").
/// * `messages` - A vector of `PubsubMessage` to publish.
///
/// # Returns
/// A `Result` which is:
/// - `Ok(PublishResponse)` containing the message IDs if successful.
/// - `Err(String)` if an error occurs.
pub fn publish(
    access_token: &str,
    topic: &str,
    messages: Vec<PubsubMessage>,
) -> Result<PublishResponse, String> {
    let client = Client::new();
    let url = format!(
        "{}/projects/{}/topics/{}:publish",
        PUBSUB_BASE_URL,
        get_project_id_from_topic(topic)?,
        get_topic_name(topic)?
    );
    let req_body = PublishRequest { messages };

    let res = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&req_body)
        .send()
        .map_err(|e| format!("Failed to send publish request: {}", e))?;

    if res.status().is_success() {
        res.json()
            .map_err(|e| format!("Failed to parse publish response: {}", e))
    } else {
        Err(format!(
            "Publish request failed with status: {} - {:?}",
            res.status(),
            res.text()
        ))
    }
}

#[derive(Serialize, Debug)]
pub struct AcknowledgeRequest {
    #[serde(rename = "ackIds")]
    pub ack_ids: Vec<String>,
}

/// Acknowledges the successful processing of messages.
///
/// The Cloud Pub/Sub system can remove the given message from the subscription.
/// Acknowledging a message whose ack deadline has expired may succeed, but
/// such a message may be redelivered later.
///
/// # Arguments
/// * `access_token` - The OAuth 2.0 access token.
/// * `subscription` - The full resource name of the subscription (e.g., "projects/your-project-id/subscriptions/your-subscription-name").
/// * `ack_ids` - The acknowledgment ID for the messages being acknowledged.
///
/// # Returns
/// A `Result` which is:
/// - `Ok(())` if the operation was successful.
/// - `Err(String)` if an error occurs.
pub fn acknowledge(
    access_token: &str,
    subscription: &str,
    ack_ids: Vec<String>,
) -> Result<(), String> {
    let client = Client::new();
    let url = format!(
        "{}/projects/{}/subscriptions/{}:acknowledge",
        PUBSUB_BASE_URL,
        get_project_id_from_subscription(subscription)?,
        get_subscription_name(subscription)?
    );
    let req_body = AcknowledgeRequest { ack_ids };

    let res = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&req_body)
        .send()
        .map_err(|e| format!("Failed to send acknowledge request: {}", e))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "Acknowledge request failed with status: {} - {:?}",
            res.status(),
            res.text()
        ))
    }
}

#[derive(Serialize, Debug)]
pub struct ModifyAckDeadlineRequest {
    #[serde(rename = "ackIds")]
    pub ack_ids: Vec<String>,
    #[serde(rename = "ackDeadlineSeconds")]
    pub ack_deadline_seconds: i32,
}

/// Modifies the ack deadline for a specific message.
///
/// This method is useful to indicate that more time is needed to process a message
/// than was originally allocated.
///
/// # Arguments
/// * `access_token` - The OAuth 2.0 access token.
/// * `subscription` - The full resource name of the subscription (e.g., "projects/your-project-id/subscriptions/your-subscription-name").
/// * `ack_ids` - List of acknowledgment IDs.
/// * `ack_deadline_seconds` - The new ack deadline with respect to the time this request was sent to the Pub/Sub system.
///
/// # Returns
/// A `Result` which is:
/// - `Ok(())` if the operation was successful.
/// - `Err(String)` if an error occurs.
pub fn modify_ack_deadline(
    access_token: &str,
    subscription: &str,
    ack_ids: Vec<String>,
    ack_deadline_seconds: i32,
) -> Result<(), String> {
    let client = Client::new();
    let url = format!(
        "{}/projects/{}/subscriptions/{}:modifyAckDeadline",
        PUBSUB_BASE_URL,
        get_project_id_from_subscription(subscription)?,
        get_subscription_name(subscription)?
    );
    let req_body = ModifyAckDeadlineRequest {
        ack_ids,
        ack_deadline_seconds,
    };

    let res = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&req_body)
        .send()
        .map_err(|e| format!("Failed to send modifyAckDeadline request: {}", e))?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "ModifyAckDeadline request failed with status: {} - {:?}",
            res.status(),
            res.text()
        ))
    }
}

// Helper functions to extract project ID, topic name, and subscription name from full resource paths
fn get_project_id_from_topic(topic_path: &str) -> Result<&str, String> {
    topic_path
        .split('/')
        .nth(1)
        .ok_or_else(|| "Invalid topic path format".to_string())
}

fn get_topic_name(topic_path: &str) -> Result<&str, String> {
    topic_path
        .split('/')
        .nth(3)
        .ok_or_else(|| "Invalid topic path format".to_string())
}

fn get_project_id_from_subscription(subscription_path: &str) -> Result<&str, String> {
    subscription_path
        .split('/')
        .nth(1)
        .ok_or_else(|| "Invalid subscription path format".to_string())
}

fn get_subscription_name(subscription_path: &str) -> Result<&str, String> {
    subscription_path
        .split('/')
        .nth(3)
        .ok_or_else(|| "Invalid subscription path format".to_string())
}
