use localizer::LanguageLocalizer;

/// Test that the expected languages and fallback language are
/// available.
#[test]
fn test_specific_language() {
    let localizer = LanguageLocalizer::new_with_language("es");
    assert_eq!(&localizer.test(), "Prueba");

    localizer.select_language("pt-BR").unwrap();

    assert_eq!(&localizer.test(), "Teste");
}

#[test]
fn test_invalid_language_fallback() {
    let localizer = LanguageLocalizer::new_with_language("unknown");
    assert_eq!(&localizer.test(), "Test");
}

#[test]
fn test_pass_argment() {
    let localizer = LanguageLocalizer::new_with_language("es");
    assert_eq!(&localizer.notification_transfer_title(true, "1 BTC"), "💸 Enviado: \u{2068}1 BTC\u{2069}");
}

#[test]
fn test_reward_redeemed_description() {
    let localizer = LanguageLocalizer::new_with_language("en");

    assert_eq!(
        &localizer.notification_reward_redeemed_description(650, Some("1 USDT")),
        "You redeemed \u{2068}650\u{2069} points for \u{2068}1 USDT\u{2069}."
    );
    assert_eq!(&localizer.notification_reward_redeemed_description(650, None), "You redeemed \u{2068}650\u{2069} points.");
}

#[test]
fn test_fiat_purchase_title() {
    let localizer = LanguageLocalizer::new_with_language("en");

    assert_eq!(&localizer.notification_fiat_purchase_title("0.01 ETH"), "🚀 Bought \u{2068}0.01 ETH\u{2069}");
}
