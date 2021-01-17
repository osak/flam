mod client;
mod error;

use reqwest::Url;
use client::html::{Client, Config};

#[tokio::main]
async fn main() {
    let client = Client::new(&Config::default()).unwrap();
    let url = Url::parse("https://osak.jp").unwrap();
    let result = client.get(url).await.unwrap();
    println!("{}", result);
}
