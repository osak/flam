mod client;
mod entry;
mod error;
mod notifier;
mod parser;
mod runner;

use crate::error::FlamError;

#[tokio::main]
async fn main() -> Result<(), FlamError> {
    env_logger::init();

    runner::lwn::run().await
}
