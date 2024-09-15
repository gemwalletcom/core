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
