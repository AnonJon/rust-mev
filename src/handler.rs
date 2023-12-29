use crate::{
    contract_interfaces::{TransferFromCall, UnstakeCall},
    streams::Event,
};
use ethers::{
    core::abi::AbiDecode,
    providers::{Middleware, Provider, Ws},
    types::{Action, Call, TraceType, Transaction, U256},
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
                    // info!("Got pending tx | hash: {:?}", tx.hash);
                    match provider.trace_call(&tx, vec![TraceType::Trace], None).await {
                        Ok(traces) => {
                            if let Some(traces) = traces.trace {
                                for trace in traces {
                                    if let Action::Call(call) = trace.action {
                                        if call.input.len() >= 4 {
                                            let func_selector_bytes = &call.input[0..4];
                                            let func_selector_hex =
                                                hex::encode(func_selector_bytes);
                                            match func_selector_hex.as_str() {
                                                UNSTAKE => {
                                                    handle_unstake(tx.clone(), call.clone(), false);
                                                }
                                                TRANSFER_FROM => {
                                                    handle_transfer(
                                                        tx.clone(),
                                                        call.clone(),
                                                        false,
                                                    );
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error!("Failed to trace pending tx | {:?}", err);
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

fn handle_unstake(tx: Transaction, call: Call, paused: bool) {
    if paused {
        return;
    }
    info!("Found pending unstake tx | hash: {:?}", tx.hash);
    let decoded: UnstakeCall = UnstakeCall::decode(&call.input).unwrap();
    let amount: U256 = decoded.amount;
    println!("Unstaking {:?}", amount);
    // let amount = decoded.amount;
}

fn handle_transfer(tx: Transaction, call: Call, paused: bool) {
    if paused {
        return;
    }
    info!("Found pending transfer tx | hash: {:?}", tx.hash);
    let decoded: TransferFromCall = TransferFromCall::decode(&call.input).unwrap();
    println!(
        "Transferring | Amount: {:?}  From: {:?} To: {:?} Contract: {:?}",
        decoded.wad, decoded.src, decoded.dst, call.from
    );
}
