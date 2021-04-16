use crate::error::FlamError;
use reqwest::StatusCode;
use reqwest::Url;
use thiserror::Error;

/// Configuration for HTTP client
#[derive(Clone)]
pub struct Config {
    pub user_agent: String,
}

pub struct Client {
    client: reqwest::Client,
}

/// All anticipated errors that could happen during HTTP communication
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("bad HTTP status: {0}")]
    BadStatus(StatusCode),
}

impl Default for Config {
    fn default() -> Config {
        Config {
            user_agent:
                "Flam HTTP(S) downloader client - see https://github.com/osak/flam for details"
                    .to_owned(),
        }
    }
}

impl From<reqwest::Error> for FlamError {
    fn from(e: reqwest::Error) -> FlamError {
        FlamError {
            source: Box::new(e),
        }
    }
}

impl From<HttpError> for FlamError {
    fn from(e: HttpError) -> FlamError {
        FlamError {
            source: Box::new(e),
        }
    }
}

impl Client {
    pub fn new(config: &Config) -> Result<Client, FlamError> {
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .build()?;
        Ok(Client {
            client,
        })
    }

    pub async fn get(&self, uri: Url) -> Result<String, FlamError> {
        let request = self.client.get(uri).build()?;
        let response = self.client.execute(request).await?;
        if response.status().is_success() {
            let body = response.text().await?;
            Ok(body)
        } else {
            Err(HttpError::BadStatus(response.status()).into())
        }
    }
}
