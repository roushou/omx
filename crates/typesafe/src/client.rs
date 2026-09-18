//! HTTP transport, credentials, and bounded response collection.

use crate::{Candidate, Ranking, evaluation::Evaluation};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Set TYPESAFE_API_KEY in the launcher process environment to use semantic search")]
    MissingKey,
    #[error("invalid TypeSafe API key")]
    InvalidKey,
    #[error("semantic search request exceeds the configured size limit")]
    TooLarge,
    #[error("TypeSafe request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("TypeSafe returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("invalid TypeSafe response: {0}")]
    Response(String),
}

/// Reusable HTTP connection pool. Requests time out after three seconds; failed
/// evaluations are not automatically retried. Credentials never enter Debug output.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeSafeClient").finish_non_exhaustive()
    }
}

impl Client {
    pub fn from_env() -> Result<Self, Error> {
        let key = std::env::var("TYPESAFE_API_KEY").map_err(|_| Error::MissingKey)?;
        if key.trim().is_empty() {
            return Err(Error::MissingKey);
        }

        let mut authorization = reqwest::header::HeaderValue::from_str(&format!("Bearer {key}"))
            .map_err(|_| Error::InvalidKey)?;
        authorization.set_sensitive(true);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::AUTHORIZATION, authorization);

        Ok(Self {
            http: reqwest::Client::builder()
                .default_headers(headers)
                .timeout(Duration::from_secs(3))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
        })
    }

    pub async fn rank(&self, query: &str, candidates: &[Candidate]) -> Result<Ranking, Error> {
        let request = Evaluation::request(query, candidates)?;
        let mut response = self
            .http
            .post("https://api.typesafe.ai/v1/systemone")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }

        if response.content_length().is_some_and(|n| n > 1024 * 1024) {
            return Err(Error::TooLarge);
        }

        let mut bytes = Vec::new();

        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > 1024 * 1024 {
                return Err(Error::TooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }

        Evaluation::decode(&bytes, candidates)
    }
}
