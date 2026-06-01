use primitives::{SupportAgent, SupportConversationStatus, SupportMessageDeliveryStatus, SupportMessageSender};
use support::ChatwootWebhookPayload;

#[test]
fn test_parse_conversation_updated_payload() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_conversation_updated.json")).unwrap();
    assert_eq!(payload.event, "conversation_updated");
    assert_eq!(payload.get_device_id(), Some("test-device-id".to_string()));
    assert_eq!(payload.get_unread(), Some(1));

    let messages = payload.get_messages();
    assert_eq!(messages.len(), 1);

    let message = &messages[0];
    assert_eq!(message.content, Some("Test message".to_string()));
    assert!(!message.is_incoming());
    assert_eq!(message.private, Some(false));

    let sender = message.sender.as_ref().unwrap();
    assert!(sender.custom_attributes.is_none());
}

#[test]
fn test_parse_device_id() {
    let payload: ChatwootWebhookPayload =
        serde_json::from_str(r#"{"event": "conversation_updated", "meta": {"sender": {"custom_attributes": {"device_id": "test-device"}}}}"#).unwrap();
    assert_eq!(payload.get_device_id(), Some("test-device".to_string()));
}

#[test]
fn test_parse_message_created_payload() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_message_created.json")).unwrap();
    assert_eq!(payload.event, "message_created");
    assert_eq!(payload.content, Some("from agent".to_string()));
    assert_eq!(payload.get_device_id(), Some("test-device-id".to_string()));
    assert_eq!(payload.get_unread(), Some(1));
    assert!(payload.is_outgoing_message());
    assert!(payload.is_public_outgoing_message());

    let messages = payload.get_messages();
    assert_eq!(messages.len(), 1);

    let message = &messages[0];
    assert!(!message.is_incoming());
    assert_eq!(message.private, Some(false));
}

#[test]
fn test_get_unread() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "test", "unread_count": 5}"#).unwrap();
    assert_eq!(payload.get_unread(), Some(5));

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "test", "conversation": {"meta": {"sender": {}}, "unread_count": 3}}"#).unwrap();
    assert_eq!(payload.get_unread(), Some(3));

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "test", "unread_count": 2, "conversation": {"meta": {"sender": {}}, "unread_count": 10}}"#).unwrap();
    assert_eq!(payload.get_unread(), Some(2));
}

#[test]
fn test_is_outgoing_message() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing"}"#).unwrap();
    assert!(payload.is_outgoing_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "incoming"}"#).unwrap();
    assert!(!payload.is_outgoing_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created"}"#).unwrap();
    assert!(!payload.is_outgoing_message());
}

#[test]
fn test_is_incoming_message() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "incoming"}"#).unwrap();
    assert!(payload.is_incoming_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing"}"#).unwrap();
    assert!(!payload.is_incoming_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created"}"#).unwrap();
    assert!(!payload.is_incoming_message());
}

#[test]
fn test_is_public_outgoing_message() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing", "private": false}"#).unwrap();
    assert!(payload.is_public_outgoing_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing", "private": true}"#).unwrap();
    assert!(!payload.is_public_outgoing_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "incoming", "private": false}"#).unwrap();
    assert!(!payload.is_public_outgoing_message());

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing"}"#).unwrap();
    assert!(!payload.is_public_outgoing_message());
}

#[test]
fn test_get_account_id() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_message_created.json")).unwrap();
    assert_eq!(payload.get_account_id(), Some(1));

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "test"}"#).unwrap();
    assert_eq!(payload.get_account_id(), None);
}

#[test]
fn test_get_conversation_id() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_message_created.json")).unwrap();
    assert_eq!(payload.get_conversation_id(), Some(1));

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "test"}"#).unwrap();
    assert_eq!(payload.get_conversation_id(), None);
}

#[test]
fn test_support_mapping() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_message_created.json")).unwrap();
    let message = payload.support_message().unwrap();
    assert_eq!(message.id, "1");
    assert_eq!(message.conversation_id, "1");
    assert_eq!(message.content, "from agent");
    assert_eq!(
        message.sender,
        SupportMessageSender::Agent(SupportAgent {
            name: "Test Agent".to_string(),
            avatar_url: None,
        })
    );
    assert_eq!(message.delivery_status, SupportMessageDeliveryStatus::Sent);

    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_conversation_updated.json")).unwrap();
    let conversation = payload.support_conversation().unwrap();
    assert_eq!(conversation.id, "1");
    assert_eq!(conversation.status, SupportConversationStatus::Open);
    assert_eq!(conversation.first_message, None);
    assert_eq!(conversation.last_message, Some("Test message".to_string()));
    assert_eq!(conversation.unread_count, 1);
}
