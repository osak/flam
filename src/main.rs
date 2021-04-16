mod client;
mod entry;
mod error;
mod parser;

use client::{http::Config, rss::Client};
use reqwest::Url;

#[tokio::main]
async fn main() {
    env_logger::init();

    let client = Client::new(&Config::default()).unwrap();
    let url = Url::parse("https://lwn.net/headlines/rss").unwrap();
    let result = client.get(url).await.unwrap();

    for r in result {
        println!("{:?}", r);
    }
}
