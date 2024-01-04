use anyhow::Result;
use log::info;
use mev::{
    client::Client,
    constants::Config,
    handler::event_handler,
    streams::{stream_new_blocks, stream_pending_transactions, Event},
    utils::utils::{setup_logger, shutdown_signal},
};
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::Notify;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    let config = Config::new();
    let client = Client::new(config).await?;
    let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let set: JoinSet<()> = JoinSet::new();
    let shutdown_notify = Arc::new(tokio::sync::Notify::new());

    println!("Starting MEV bot...");
    start(set, client, event_sender.clone(), shutdown_notify.clone()).await;
    info!("Shut down gracefully");

    Ok(())
}

async fn start(
    mut set: JoinSet<()>,
    client: Client,
    event_sender: Sender<Event>,
    shutdown_notify: Arc<Notify>,
) {
    set.spawn(stream_pending_transactions(
        client.provider.clone(),
        event_sender.clone(),
        shutdown_notify.clone(),
    ));
    set.spawn(stream_new_blocks(
        client.provider.clone(),
        event_sender.clone(),
        shutdown_notify.clone(),
    ));
    set.spawn(event_handler(
        client,
        event_sender.clone(),
        shutdown_notify.clone(),
    ));

    tokio::spawn(async move {
        shutdown_signal().await;
        shutdown_notify.notify_waiters();
    });

    while let Some(res) = set.join_next().await {
        match res {
            Ok(_) => {}
            Err(e) => {
                info!("Error in join set: {:?}", e);
            }
        }
    }
}
