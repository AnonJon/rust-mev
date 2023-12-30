use crate::{
    contract_interfaces::{TransferFromCall, UnstakeCall},
    streams::Event,
};
use ethers::{
    core::abi::AbiDecode,
    providers::{Provider, Ws},
    types::{Transaction, H160, U256},
};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub async fn event_handler(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
    let mut event_receiver = event_sender.subscribe();

    const UNSTAKE: &str = "2e17de78";
    const TRANSFER_FROM: &str = "23b872dd";
    loop {
        match event_receiver.recv().await {
            Ok(event) => match event {
                Event::PendingTx(tx) => {
                    if tx.input.len() >= 4 {
                        let func_selector_bytes = &tx.input[0..4];
                        let func_selector_hex = hex::encode(func_selector_bytes);
                        match func_selector_hex.as_str() {
                            UNSTAKE => {
                                handle_unstake(tx.clone(), false);
                            }
                            TRANSFER_FROM => {
                                handle_transfer(tx.clone(), false);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Err(_) => {
                error!("Failed to get event");
            }
        }
    }
}

fn handle_unstake(tx: Transaction, paused: bool) {
    let staking_contract: &str = "0xBc10f2E862ED4502144c7d632a3459F49DFCDB5e";

    if paused || tx.to.unwrap() != staking_contract.parse::<H160>().unwrap() {
        return;
    }

    info!("Found pending unstake tx | hash: {:?}", tx.hash);
    let decoded: UnstakeCall = UnstakeCall::decode(&tx.input).unwrap();
    let amount: U256 = decoded.amount;
    println!("Unstaking {:?}", amount);
}

fn handle_transfer(tx: Transaction, paused: bool) {
    if paused {
        return;
    }
    info!("Found pending transfer tx | hash: {:?}", tx.hash);
    let decoded: TransferFromCall = TransferFromCall::decode(&tx.input).unwrap();
    println!(
        "Transferring | Amount: {:?}  From: {:?} To: {:?} Contract: {:?}",
        decoded.wad,
        decoded.src,
        decoded.dst,
        tx.to.expect("No contract address found")
    );
}
