use url::Url;

pub fn extract_host(url_or_domain: &str) -> Option<String> {
    if url_or_domain.is_empty() {
        return None;
    }

    if !url_or_domain.contains("://") {
        return Some(url_or_domain.to_string());
    }

    let url = Url::parse(url_or_domain).ok()?;
    let host = url.host_str()?;
    match url.port() {
        Some(p) => Some(format!("{host}:{p}")),
        None => Some(host.to_string()),
    }
}

pub fn parse_url(domain: &str) -> Option<Url> {
    let url_str = if domain.contains("://") {
        domain.to_string()
    } else {
        format!("https://{domain}")
    };
    Url::parse(&url_str).ok()
}

pub fn host(url_string: &str) -> String {
    Url::parse(url_string)
        .ok()
        .and_then(|url| url.host_str().map(|h| h.to_lowercase()))
        .unwrap_or_else(|| url_string.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host() {
        assert_eq!(extract_host(""), None);
        assert_eq!(extract_host("example.com"), Some("example.com".to_string()));
        assert_eq!(extract_host("https://example.com"), Some("example.com".to_string()));
        assert_eq!(extract_host("example.com:8080"), Some("example.com:8080".to_string()));
        assert_eq!(extract_host("https://example.com:8080"), Some("example.com:8080".to_string()));
    }

    #[test]
    fn test_parse_url() {
        let url = parse_url("example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));

        let url = parse_url("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_host() {
        assert_eq!(host("https://example.com"), "example.com");
        assert_eq!(host("https://EXAMPLE.COM"), "example.com");
        assert_eq!(host("EXAMPLE.COM"), "example.com");
    }
}
