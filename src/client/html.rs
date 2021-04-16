use crate::client;
use crate::error::FlamError;
use reqwest::Url;

/// HTML downloader client
pub struct Client {
    client: client::http::Client,
}

impl Client {
    pub fn new(config: &client::http::Config) -> Result<Client, FlamError> {
        let client = client::http::Client::new(config)?;
        Ok(Client { client })
    }

    pub async fn get(&self, uri: Url) -> Result<String, FlamError> {
        self.client.get(uri).await
    }
}
