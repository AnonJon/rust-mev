use ethers::{
    providers::{Middleware, Provider, StreamExt, Ws},
    types::{Block, Log, Transaction, H256},
};
use log::{error, info};

use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::Sender;
use tokio::sync::Notify;

#[derive(Debug, Clone)]
pub enum Event {
    Block(Block<H256>),
    PendingTx(Transaction),
    Log(Log),
}

pub async fn stream_pending_transactions(
    provider: Arc<Provider<Ws>>,
    event_sender: Sender<Event>,
    shutdown_notify: Arc<Notify>,
) {
    let stream = provider.subscribe_pending_txs().await.unwrap();
    let mut stream = stream.transactions_unordered(256).fuse();

    loop {
        select! {
            Some(result) = stream.next() => {
                match result {
                    Ok(tx) => match event_sender.send(Event::PendingTx(tx)) {
                        Ok(_) => {}
                        Err(_) => {
                            error!("Failed to send pending tx");
                        }
                    },
                    Err(e) => {
                        let error_string = e.to_string();
                        if error_string.contains("not found") {
                        } else {
                            error!("Failed to get pending tx | {:?}", e);
                        }
                    }
                }
            },
            _ = shutdown_notify.notified() => {
                info!("Shutting down pending transactions stream.");
                break;
            },
        }
    }
}

pub async fn stream_new_blocks(
    provider: Arc<Provider<Ws>>,
    event_sender: Sender<Event>,
    shutdown_notify: Arc<Notify>,
) {
    let mut stream = provider.subscribe_blocks().await.unwrap();

    loop {
        select! {
        Some(result) = stream.next() => {
            match event_sender.send(Event::Block(result)) {
                    Ok(_) => {}
                    Err(_) => {
                        error!("Failed to send new block");
                    }
                }
            },
            _ = shutdown_notify.notified() => {
                info!("Shutting down block finder stream.");
                break;
            },
        }
    }
}
