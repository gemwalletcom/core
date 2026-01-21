use localizer::LanguageLocalizer;
use number_formatter::BigNumberFormatter;
use primitives::{CoreEmoji, CoreListItem, CoreListItemBadge, CoreListItemIcon, Deeplink, InAppNotification, NotificationData, NotificationType};

pub fn map_notification(notification: NotificationData, localizer: &LanguageLocalizer) -> InAppNotification {
    let item = map_to_list_item(&notification, localizer);

    InAppNotification {
        wallet_id: notification.wallet_id,
        read_at: notification.read_at,
        created_at: notification.created_at,
        item,
    }
}

fn notification_item(
    id: String,
    title: String,
    subtitle: Option<String>,
    value: Option<String>,
    subvalue: Option<String>,
    emoji: CoreEmoji,
    badge: Option<CoreListItemBadge>,
    url: Option<String>,
) -> CoreListItem {
    CoreListItem {
        id,
        title,
        subtitle,
        value,
        subvalue,
        icon: Some(CoreListItemIcon::Emoji(emoji)),
        badge,
        url,
    }
}

fn map_to_list_item(notification: &NotificationData, localizer: &LanguageLocalizer) -> CoreListItem {
    let metadata = &notification.metadata;
    let points = get_i32(metadata, "points");
    let username = get_string(metadata, "username");
    let id = notification.id.to_string();
    let url = Some(Deeplink::Rewards.to_url());

    match notification.notification_type {
        NotificationType::ReferralJoined => notification_item(
            id,
            localizer.notification_reward_pending_title(),
            Some(localizer.notification_reward_invite_description(Some(&username))),
            Some(format!("+{}", points)),
            None,
            CoreEmoji::Party,
            Some(CoreListItemBadge::New),
            url,
        ),
        NotificationType::RewardsEnabled => notification_item(
            id,
            localizer.notification_rewards_enabled_title(),
            Some(localizer.notification_rewards_enabled_description()),
            None,
            None,
            CoreEmoji::Gift,
            None,
            url,
        ),
        NotificationType::RewardsCodeDisabled => notification_item(
            id,
            localizer.notification_rewards_disabled_title(),
            Some(localizer.notification_rewards_disabled_description()),
            None,
            None,
            CoreEmoji::Warning,
            Some(CoreListItemBadge::New),
            url,
        ),
        NotificationType::RewardsRedeemed => {
            let raw_value = get_string(metadata, "value");
            let value = notification.asset.as_ref().map(|asset| {
                let formatted = BigNumberFormatter::value(&raw_value, asset.decimals).unwrap_or(raw_value.clone());
                format!("+{} {}", formatted, asset.symbol)
            });
            let subvalue = Some(format!("-{}", points));
            notification_item(
                id,
                localizer.notification_reward_redeemed_title(),
                Some(localizer.notification_reward_redeemed_description(points)),
                value,
                subvalue,
                CoreEmoji::Gift,
                Some(CoreListItemBadge::New),
                url,
            )
        }
        NotificationType::RewardsCreateUsername => notification_item(
            id,
            localizer.notification_reward_title(points),
            Some(localizer.notification_reward_create_username_description()),
            Some(format!("+{}", points)),
            None,
            CoreEmoji::Gem,
            Some(CoreListItemBadge::New),
            url,
        ),
        NotificationType::RewardsInvite => notification_item(
            id,
            localizer.notification_reward_title(points),
            Some(localizer.notification_reward_invite_description(Some(&username))),
            Some(format!("+{}", points)),
            None,
            CoreEmoji::Party,
            Some(CoreListItemBadge::New),
            url,
        ),
    }
}

fn get_i32(metadata: &Option<serde_json::Value>, key: &str) -> i32 {
    metadata.as_ref().and_then(|m| m.get(key)).and_then(|v| v.as_i64()).unwrap_or(0) as i32
}

fn get_string(metadata: &Option<serde_json::Value>, key: &str) -> String {
    metadata.as_ref().and_then(|m| m.get(key)).and_then(|v| v.as_str()).unwrap_or("").to_string()
}
