use crate::constants::func_selectors;
use crate::streams::{Event, NewBlock};
use ethers::{
    providers::{Middleware, Provider, StreamExt, Ws},
    types::{Action, Filter, Log, TraceType, Transaction, U256, U64},
};
use log::{error, info, trace, warn};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub async fn event_handler(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
    let mut event_receiver = event_sender.subscribe();
    loop {
        match event_receiver.recv().await {
            Ok(event) => match event {
                Event::PendingTx(tx) => {
                    info!("Got pending tx | hash: {:?}", tx.hash);
                    match provider.trace_call(&tx, vec![TraceType::Trace], None).await {
                        Ok(traces) => {
                            if let Some(traces) = traces.trace {
                                for trace in traces {
                                    if let Action::Call(call) = trace.action {
                                        if call.input.len() >= 4 {
                                            let func_selector_bytes = &call.input[0..4]; // First 4 bytes
                                            let func_selector_hex =
                                                hex::encode(func_selector_bytes); // Convert bytes to hex string
                                            if let Some(function_name) =
                                                func_selectors().get(&func_selector_hex[..])
                                            {
                                                trace!("Function Name: {}", function_name);
                                            } else {
                                                warn!(
                                                    "Unknown function selector: {}",
                                                    func_selector_hex
                                                );
                                            }
                                        } else {
                                            warn!("Input data too short to contain a function selector");
                                        }
                                        // info!("trace input data | {:?}", call.input);
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
