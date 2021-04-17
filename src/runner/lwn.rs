use crate::client::{http, rss};
use crate::error::FlamError;
use crate::notifier::log::LoggingNotifier;

use reqwest::Url;

pub async fn run() -> Result<(), FlamError> {
    let items = load_items().await?;

    let notifier = LoggingNotifier::new();
    for item in items {
        notifier.notify(&item.title, &item.description);
    }
    Ok(())
}

async fn load_items() -> Result<Vec<rss::Item>, FlamError> {
    let client = rss::Client::new(&http::Config::default())?;
    let url = Url::parse("https://lwn.net/headlines/rss").unwrap();
    client.get(url).await
}
