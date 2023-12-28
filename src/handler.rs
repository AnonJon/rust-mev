use crate::streams::{Event, NewBlock};
use ethers::{
    providers::{Middleware, Provider, StreamExt, Ws},
    types::{Filter, Log, Transaction, U256, U64},
};
use log::info;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub async fn event_handler(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
    let mut event_receiver = event_sender.subscribe();
    loop {
        match event_receiver.recv().await {
            Ok(event) => match event {
                Event::PendingTx(tx) => {
                    info!("Got pending tx: {:?}", tx);
                }
                _ => {}
            },
            Err(_) => {
                info!("Failed to get event");
            }
        }
    }
}
