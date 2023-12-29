use ethers::{
    providers::{Middleware, Provider, StreamExt, Ws},
    types::{Log, Transaction, U256, U64},
};
use log::error;

use std::sync::Arc;
use tokio::sync::broadcast::Sender;

#[derive(Default, Debug, Clone)]
pub struct NewBlock {
    pub block_number: U64,
    pub base_fee: U256,
    pub next_base_fee: U256,
}

#[derive(Debug, Clone)]
pub enum Event {
    Block(NewBlock),
    PendingTx(Transaction),
    Log(Log),
}

pub async fn stream_pending_transactions(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
    let stream = provider.subscribe_pending_txs().await.unwrap();
    let mut stream = stream.transactions_unordered(256).fuse();

    while let Some(result) = stream.next().await {
        match result {
            Ok(tx) => match event_sender.send(Event::PendingTx(tx)) {
                Ok(_) => {}
                Err(_) => {
                    error!("Failed to send pending tx");
                }
            },
            Err(e) => {
                error!("Failed to get pending tx | {:?}", e);
            }
        };
    }
}
