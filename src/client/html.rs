use crate::error::FlamError;
use reqwest::StatusCode;
use reqwest::Url;
use thiserror::Error;

#[derive(Clone)]
pub struct Config {
    pub user_agent: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            user_agent:
                "Flam HTML downloader client - see https://github.com/osak/flam for details"
                    .to_owned(),
        }
    }
}

pub struct Client {
    config: Config,
    client: reqwest::Client,
}

impl From<reqwest::Error> for FlamError {
    fn from(e: reqwest::Error) -> FlamError {
        FlamError {
            source: Box::new(e),
        }
    }
}

#[derive(Error, Debug)]
pub enum HtmlError {
    #[error("bad HTTP status: {0}")]
    BadStatus(StatusCode),
}

impl From<HtmlError> for FlamError {
    fn from(e: HtmlError) -> FlamError {
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
            config: config.clone(),
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
            Err(HtmlError::BadStatus(response.status()).into())
        }
    }
}
