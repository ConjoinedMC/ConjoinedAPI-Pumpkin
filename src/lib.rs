//! Asynchronous binary storage client for PumpkinMC plugin code.
//! The crate does not link a particular Pumpkin plugin ABI.
use reqwest::{Client, Method, StatusCode, Url};
use std::{error::Error, fmt, sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(Debug)]
pub enum ClientError {
    InvalidInput,
    Busy,
    Http(StatusCode),
    Transport(reqwest::Error),
}
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("invalid endpoint, credentials, key, or value"),
            Self::Busy => f.write_str("too many in-flight requests"),
            Self::Http(code) => write!(f, "ConjoinedDB returned HTTP {code}"),
            Self::Transport(_) => f.write_str("ConjoinedDB transport failed"),
        }
    }
}
impl Error for ClientError {}
impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Transport(error.without_url())
    }
}

#[derive(Clone)]
pub struct ConjoinedClient {
    http: Client,
    endpoint: Url,
    namespace: String,
    token: String,
    requests: Arc<Semaphore>,
}
fn identifier(value: &str, max: usize, key: bool) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value.bytes().all(|b| {
            b.is_ascii_lowercase()
                || b.is_ascii_digit()
                || b == b'_'
                || b == b'-'
                || (key && (b.is_ascii_uppercase() || b == b'.'))
        })
}
impl ConjoinedClient {
    pub fn new(endpoint: &str, namespace: &str, token: &str) -> Result<Self, ClientError> {
        let endpoint = Url::parse(endpoint).map_err(|_| ClientError::InvalidInput)?;
        if endpoint.scheme() != "https"
            || endpoint.host_str().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.path() != "/"
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !identifier(namespace, 64, false)
            || !(32..=256).contains(&token.len())
            || !token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(ClientError::InvalidInput);
        }
        Ok(Self {
            http: Client::builder()
                .https_only(true)
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(10))
                .build()?,
            endpoint,
            namespace: namespace.into(),
            token: token.into(),
            requests: Arc::new(Semaphore::new(64)),
        })
    }
    async fn send(
        &self,
        method: Method,
        player: Option<Uuid>,
        key: &str,
        value: Vec<u8>,
    ) -> Result<(StatusCode, Vec<u8>), ClientError> {
        if !identifier(key, 128, true) || value.len() > 65536 {
            return Err(ClientError::InvalidInput);
        }
        let _permit = self.requests.try_acquire().map_err(|_| ClientError::Busy)?;
        let owner = player
            .map(|id| id.to_string())
            .unwrap_or_else(|| "global".into());
        let url = self
            .endpoint
            .join(&format!("/v1/{}/{owner}/{key}", self.namespace))
            .map_err(|_| ClientError::InvalidInput)?;
        let mut response = self
            .http
            .request(method, url)
            .bearer_auth(&self.token)
            .body(value)
            .send()
            .await?;
        let status = response.status();
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if body.len() + chunk.len() > 65536 {
                return Err(ClientError::InvalidInput);
            }
            body.extend_from_slice(&chunk);
        }
        Ok((status, body))
    }
    /// `None` selects non-player data; `Some(uuid)` selects player data.
    pub async fn get(
        &self,
        player: Option<Uuid>,
        key: &str,
    ) -> Result<Option<Vec<u8>>, ClientError> {
        let (status, body) = self.send(Method::GET, player, key, Vec::new()).await?;
        match status {
            StatusCode::OK => Ok(Some(body)),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(ClientError::Http(status)),
        }
    }
    pub async fn put(
        &self,
        player: Option<Uuid>,
        key: &str,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        let (status, _) = self.send(Method::PUT, player, key, value).await?;
        if status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(ClientError::Http(status))
        }
    }
    pub async fn delete(&self, player: Option<Uuid>, key: &str) -> Result<(), ClientError> {
        let (status, _) = self.send(Method::DELETE, player, key, Vec::new()).await?;
        if status == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(ClientError::Http(status))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_credentials_and_identifiers() {
        assert!(ConjoinedClient::new("http://localhost", "example", &"a".repeat(32)).is_err());
        assert!(ConjoinedClient::new("https://localhost", "../escape", &"a".repeat(32)).is_err());
        assert!(ConjoinedClient::new("https://localhost", "example", "short").is_err());
        assert!(!identifier("../escape", 128, true));
        assert!(identifier("balance.v1", 128, true));
    }
}
