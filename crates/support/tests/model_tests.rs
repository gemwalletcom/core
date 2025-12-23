use support::ChatwootWebhookPayload;

#[test]
fn test_parse_conversation_updated_payload() {
    let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("testdata/chatwoot_conversation_updated.json")).unwrap();
    assert_eq!(payload.event, "conversation_updated");
    assert_eq!(payload.get_support_device_id(), Some("test-device-id".to_string()));
    assert_eq!(payload.get_unread(), Some(1));

    // supportDeviceId (camelCase) - TODO: remove once all clients use lowercase
    let payload: ChatwootWebhookPayload =
        serde_json::from_str(r#"{"event": "conversation_updated", "meta": {"sender": {"custom_attributes": {"supportDeviceId": "test-camel"}}}}"#).unwrap();
    assert_eq!(payload.get_support_device_id(), Some("test-camel".to_string()));

    // support_device_id (snake_case) - TODO: remove once all clients use lowercase
    let payload: ChatwootWebhookPayload =
        serde_json::from_str(r#"{"event": "conversation_updated", "meta": {"sender": {"custom_attributes": {"support_device_id": "test-snake"}}}}"#).unwrap();
    assert_eq!(payload.get_support_device_id(), Some("test-snake".to_string()));
}
