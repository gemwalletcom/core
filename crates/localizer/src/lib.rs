use std::sync::Arc;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, LanguageLoader, Localizer, RustEmbedNotifyAssets,
};
use i18n_embed_fl::fl;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
pub struct LocalizationsEmbed;

pub static LOCALIZATIONS: Lazy<RustEmbedNotifyAssets<LocalizationsEmbed>> =
    Lazy::new(|| RustEmbedNotifyAssets::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("i18n/")));

macro_rules! fl {
    ($loader:expr, $message_id:literal) => {{
        i18n_embed_fl::fl!($loader, $message_id)
    }};
    ($loader:expr, $message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($loader, $message_id, $($args), *)
    }};
}

pub struct LanguageLocalizer {
    loader: Arc<FluentLanguageLoader>,
    localizer: DefaultLocalizer<'static>,
}

impl Default for LanguageLocalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageLocalizer {
    pub fn new() -> Self {
        let loader = Arc::new(fluent_language_loader!());

        loader.load_fallback_language(&*LOCALIZATIONS).expect("Error while loading fallback language");

        let loader_ref: &'static FluentLanguageLoader = unsafe { &*(Arc::as_ptr(&loader) as *const _) };

        let localizer = DefaultLocalizer::new(loader_ref, &*LOCALIZATIONS);

        Self { loader, localizer }
    }

    pub fn new_with_language(language: &str) -> Self {
        let localizer = Self::new();
        localizer.select_language(language).unwrap_or_default();
        localizer
    }

    pub fn select_language(&self, language: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let langid = language.parse()?;
        self.localizer.select(&[langid])?;
        Ok(true)
    }

    pub fn test(&self) -> String {
        fl!(self.loader.as_ref(), "notification_test")
    }

    pub fn notification_transfer_title(&self, is_sent: bool, value: &str) -> String {
        if is_sent {
            fl!(self.loader.as_ref(), "notification_sent_title", value = value)
        } else {
            fl!(self.loader.as_ref(), "notification_received_title", value = value)
        }
    }

    pub fn notification_transfer_description(&self, is_sent: bool, to_address: &str, from_address: &str) -> String {
        if is_sent {
            fl!(self.loader.as_ref(), "notification_sent_description", address = to_address)
        } else {
            fl!(self.loader.as_ref(), "notification_received_description", address = from_address)
        }
    }

    pub fn notification_token_approval_title(&self, token: &str, address: &str) -> String {
        fl!(self.loader.as_ref(), "notification_token_approval_title", token = token, address = address)
    }

    pub fn notification_stake_title(&self, value: &str, validator: &str) -> String {
        if validator.len() < 12 {
            fl!(self.loader.as_ref(), "notification_stake_validator_title", value = value, validator = validator)
        } else {
            fl!(self.loader.as_ref(), "notification_stake_title", value = value)
        }
    }

    pub fn notification_unstake_title(&self, value: &str, validator: &str) -> String {
        if validator.len() < 12 {
            fl!(
                self.loader.as_ref(),
                "notification_unstake_validator_title",
                value = value,
                validator = validator
            )
        } else {
            fl!(self.loader.as_ref(), "notification_unstake_title", value = value)
        }
    }

    pub fn notification_redelegate_title(&self, value: &str, validator: &str) -> String {
        if validator.len() < 12 {
            fl!(
                self.loader.as_ref(),
                "notification_redelegate_validator_title",
                value = value,
                validator = validator
            )
        } else {
            fl!(self.loader.as_ref(), "notification_redelegate_title", value = value)
        }
    }

    pub fn notification_withdraw_stake_title(&self, value: &str, validator: &str) -> String {
        if validator.len() < 12 {
            fl!(
                self.loader.as_ref(),
                "notification_withdraw_stake_validator_title",
                value = value,
                validator = validator
            )
        } else {
            fl!(self.loader.as_ref(), "notification_withdraw_stake_title", value = value)
        }
    }

    pub fn notification_claim_rewards_title(&self, value: &str) -> String {
        fl!(self.loader.as_ref(), "notification_claim_rewards_title", value = value)
    }

    pub fn notification_swap_title(&self, from_symbol: &str, to_symbol: &str) -> String {
        fl!(
            self.loader.as_ref(),
            "notification_swap_title",
            from_symbol = from_symbol,
            to_symbol = to_symbol
        )
    }

    pub fn notification_swap_description(&self, from_value: &str, to_value: &str) -> String {
        fl!(
            self.loader.as_ref(),
            "notification_swap_description",
            from_value = from_value,
            to_value = to_value
        )
    }
}
