use support::ChatwootWebhookPayload;

#[test]
fn test_parse_conversation_updated_payload() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_conversation_updated.json")).unwrap();
    assert_eq!(payload.event, "conversation_updated");
    assert_eq!(payload.get_support_device_id(), Some("test-device-id".to_string()));
    assert_eq!(payload.get_unread(), Some(1));

    let messages = payload.get_messages();
    assert_eq!(messages.len(), 1);

    let message = &messages[0];
    assert_eq!(message.content, Some("Test message".to_string()));
    assert!(!message.is_incoming());

    let sender = message.sender.as_ref().unwrap();
    assert!(sender.custom_attributes.is_none());
}

#[test]
fn test_parse_support_device_id_aliases() {
    let payload: ChatwootWebhookPayload =
        serde_json::from_str(r#"{"event": "conversation_updated", "meta": {"sender": {"custom_attributes": {"supportDeviceId": "test-camel"}}}}"#).unwrap();
    assert_eq!(payload.get_support_device_id(), Some("test-camel".to_string()));

    let payload: ChatwootWebhookPayload =
        serde_json::from_str(r#"{"event": "conversation_updated", "meta": {"sender": {"custom_attributes": {"support_device_id": "test-snake"}}}}"#).unwrap();
    assert_eq!(payload.get_support_device_id(), Some("test-snake".to_string()));

    let payload: ChatwootWebhookPayload =
        serde_json::from_str(r#"{"event": "conversation_updated", "meta": {"sender": {"custom_attributes": {"supportdeviceid": "test-lower"}}}}"#).unwrap();
    assert_eq!(payload.get_support_device_id(), Some("test-lower".to_string()));
}

#[test]
fn test_parse_message_created_payload() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_message_created.json")).unwrap();
    assert_eq!(payload.event, "message_created");
    assert_eq!(payload.content, Some("Test message".to_string()));
    assert_eq!(payload.get_support_device_id(), Some("test-device-id".to_string()));
    assert_eq!(payload.get_unread(), Some(1));
    assert!(payload.is_incoming_message());

    let messages = payload.get_messages();
    assert_eq!(messages.len(), 1);

    let message = &messages[0];
    assert!(message.is_incoming());
}

#[test]
fn test_get_unread() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "test", "unread_count": 5}"#).unwrap();
    assert_eq!(payload.get_unread(), Some(5));

    let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "test", "conversation": {"meta": {"sender": {}}, "unread_count": 3}}"#).unwrap();
    assert_eq!(payload.get_unread(), Some(3));

    let payload: ChatwootWebhookPayload =
        serde_json::from_str(r#"{"event": "test", "unread_count": 2, "conversation": {"meta": {"sender": {}}, "unread_count": 10}}"#).unwrap();
    assert_eq!(payload.get_unread(), Some(2));
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
