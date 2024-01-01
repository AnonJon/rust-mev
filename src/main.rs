use anyhow::Result;
use log::info;
use mev::client::Client;
use mev::constants::Config;
use mev::handler::event_handler;
use mev::streams::{stream_pending_transactions, Event};
use mev::utils::setup_logger;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    let config = Config::new();
    let client = Client::new(config).await?;
    let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let mut set = JoinSet::new();

    println!("Starting MEV bot...");

    set.spawn(stream_pending_transactions(
        client.provider.clone(),
        event_sender.clone(),
    ));
    set.spawn(event_handler(client.provider.clone(), event_sender.clone()));

    // will run forever
    while let Some(res) = set.join_next().await {
        info!("Shutting down... {:?}", res);
    }

    Ok(())
}
