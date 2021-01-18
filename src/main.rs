mod client;
mod entry;
mod error;
mod parser;

use client::html::{Client, Config};
use reqwest::Url;

#[tokio::main]
async fn main() {
    env_logger::init();

    let client = Client::new(&Config::default()).unwrap();
    let url = Url::parse("https://lwn.net").unwrap();
    let result = client.get(url).await.unwrap();

    let entries = parser::lwn::parse(&result);
    for e in entries {
        println!("{:?}", e);
    }
}
