use crate::responders::ApiError;
use reqwest::Url;
use std::path::Path;

const PRICE_API_URL: &str = "https://price-api.mayan.finance";
const EXPLORER_API_URL: &str = "https://explorer-api.mayan.finance";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MayanApi {
    Price,
    Explorer,
}

impl MayanApi {
    pub(super) fn from_path(path: &str) -> Result<Self, ApiError> {
        match path {
            "price" => Ok(Self::Price),
            "explorer" => Ok(Self::Explorer),
            _ => Err(ApiError::BadRequest(format!("Unsupported Mayan API: {path}"))),
        }
    }

    pub(super) fn url(self, version: &str, path: &Path, query: Option<&str>) -> Result<Url, ApiError> {
        let mut url = Url::parse(self.base_url()).map_err(ApiError::internal_server_error)?;
        {
            let mut segments = url.path_segments_mut().map_err(|_| ApiError::InternalServerError("Invalid Mayan API URL".to_string()))?;
            segments.push(version);
            for segment in path.iter() {
                segments.push(&segment.to_string_lossy());
            }
        }
        if let Some(query) = query.filter(|query| !query.is_empty()) {
            url.set_query(Some(query));
        }
        Ok(url)
    }

    fn base_url(self) -> &'static str {
        match self {
            Self::Price => PRICE_API_URL,
            Self::Explorer => EXPLORER_API_URL,
        }
    }
}
