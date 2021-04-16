use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Url;
use crate::client;
use thiserror::Error;

use crate::error::FlamError;

/// RSS client
pub struct Client {
    client: client::http::Client,
}

#[derive(Debug)]
pub struct Item {
    title: String,
    link: String,
    created: DateTime<Utc>,
    author: String,
    description: String,
}

impl Default for Item {
    fn default() -> Item {
        Item {
            title: "".to_owned(),
            link: "".to_owned(),
            created: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
            author: "".to_owned(),
            description: "".to_owned(),
        }
    }
}

impl Client {
    pub fn new(config: &client::http::Config) -> Result<Client, FlamError> {
        let client = client::http::Client::new(config)?;
        Ok(Client {
            client,
        })
    }

    pub async fn get(&self, uri: Url) -> Result<String, FlamError> {
        self.client.get(uri).await
    }
}
