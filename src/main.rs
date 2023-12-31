use anyhow::Result;
use log::info;
use mev::client::Client;
use mev::constants::Env;
use mev::handler::event_handler;
use mev::streams::{stream_pending_transactions, Event};
use mev::utils::setup_logger;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;
#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    let env = Env::new();
    let web3 = Client::new(env.wss_url.to_string()).await?;
    let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let mut set = JoinSet::new();

    println!("Starting MEV bot...");

    set.spawn(stream_pending_transactions(
        web3.client.clone(),
        event_sender.clone(),
    ));
    set.spawn(event_handler(web3.client.clone(), event_sender.clone()));

    // will run forever
    while let Some(res) = set.join_next().await {
        info!("Shutting down... {:?}", res);
    }

    Ok(())
}
