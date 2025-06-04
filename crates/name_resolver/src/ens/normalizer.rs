use idna::uts46::{AsciiDenyList, DnsLength, Hyphens, Uts46};

pub fn normalize_domain(name: &str) -> Result<String, idna::Errors> {
    let uts46 = Uts46::new();
    let flags = AsciiDenyList::STD3;
    let normalized = uts46.to_ascii(name.as_bytes(), flags, Hyphens::Allow, DnsLength::Ignore)?;
    Ok(normalized.into_owned())
}
