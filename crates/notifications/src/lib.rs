use localizer::LanguageLocalizer;
use primitives::{CoreEmoji, CoreListItem, CoreListItemBadge, CoreListItemIcon, Notification, NotificationItem, NotificationType};

pub fn map_notification(notification: NotificationItem, localizer: &LanguageLocalizer) -> Notification {
    let item = map_to_list_item(&notification, localizer);

    Notification {
        wallet_id: notification.wallet_id,
        is_read: notification.is_read,
        read_at: notification.read_at,
        created_at: notification.created_at,
        asset: None,
        item,
    }
}

fn map_to_list_item(notification: &NotificationItem, localizer: &LanguageLocalizer) -> CoreListItem {
    let metadata = &notification.metadata;
    let points = get_i32(metadata, "points");
    let username = get_string(metadata, "username");

    match notification.notification_type {
        NotificationType::ReferralJoined => CoreListItem {
            title: localizer.notification_reward_pending_title(),
            subtitle: Some(localizer.notification_reward_invite_description(Some(&username))),
            value: Some(format!("+{}", points)),
            subvalue: None,
            icon: CoreListItemIcon::Emoji(CoreEmoji::Party),
            badge: Some(CoreListItemBadge::New),
            url: None,
        },
        NotificationType::RewardsEnabled => CoreListItem {
            title: localizer.notification_rewards_enabled_title(),
            subtitle: Some(localizer.notification_rewards_enabled_description()),
            value: None,
            subvalue: None,
            icon: CoreListItemIcon::Emoji(CoreEmoji::Gift),
            badge: None,
            url: None,
        },
        NotificationType::RewardsCodeDisabled => CoreListItem {
            title: localizer.notification_rewards_disabled_title(),
            subtitle: Some(localizer.notification_rewards_disabled_description()),
            value: None,
            subvalue: None,
            icon: CoreListItemIcon::Emoji(CoreEmoji::Warning),
            badge: Some(CoreListItemBadge::New),
            url: None,
        },
        NotificationType::RewardsRedeemed => CoreListItem {
            title: localizer.notification_reward_redeemed_title(),
            subtitle: Some(localizer.notification_reward_redeemed_description(points)),
            value: Some(format!("-{}", points.abs())),
            subvalue: None,
            icon: CoreListItemIcon::Emoji(CoreEmoji::Gift),
            badge: Some(CoreListItemBadge::New),
            url: None,
        },
        NotificationType::RewardsCreateUsername => CoreListItem {
            title: localizer.notification_reward_title(points),
            subtitle: Some(localizer.notification_reward_create_username_description()),
            value: Some(format!("+{}", points)),
            subvalue: None,
            icon: CoreListItemIcon::Emoji(CoreEmoji::Gem),
            badge: Some(CoreListItemBadge::New),
            url: None,
        },
        NotificationType::RewardsInvite => CoreListItem {
            title: localizer.notification_reward_title(points),
            subtitle: Some(localizer.notification_reward_invite_description(Some(&username))),
            value: Some(format!("+{}", points)),
            subvalue: None,
            icon: CoreListItemIcon::Emoji(CoreEmoji::Party),
            badge: Some(CoreListItemBadge::New),
            url: None,
        },
    }
}

fn get_i32(metadata: &Option<serde_json::Value>, key: &str) -> i32 {
    metadata.as_ref().and_then(|m| m.get(key)).and_then(|v| v.as_i64()).unwrap_or(0) as i32
}

fn get_string(metadata: &Option<serde_json::Value>, key: &str) -> String {
    metadata.as_ref().and_then(|m| m.get(key)).and_then(|v| v.as_str()).unwrap_or("").to_string()
}
