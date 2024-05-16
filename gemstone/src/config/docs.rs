#[derive(uniffi::Enum, Clone)]
pub enum DocsItem {
    WatchWallet,
}

pub fn get_docs_url(item: DocsItem) -> String {
    let path = match item {
        DocsItem::WatchWallet => "faqs/watch-wallet",
    };
    format!("https://docs.gemwallet.com/{}", path)
}
