use crate::{
    bundler::Bundler,
    client::Client,
    contract_interfaces::{community_pool::UnstakeCall, erc20::TransferFromCall},
    streams::Event,
    utils::utils::calculate_next_block_base_fee,
};
use anyhow::{anyhow, Result};
use ethers::{
    core::abi::AbiDecode,
    providers::Middleware,
    types::{Bytes, Transaction, H160, H256, U256, U64},
};
use log::{error, info};
use reqwest;
use serde_json::Value;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::Sender;
use tokio::sync::Notify;

pub async fn event_handler(
    client: Client,
    event_sender: Sender<Event>,
    shutdown_notify: Arc<Notify>,
) {
    let mut event_receiver = event_sender.subscribe();

    const UNSTAKE: &str = "2e17de78";
    const TRANSFER_FROM: &str = "23b872dd";

    loop {
        select! {
            event = event_receiver.recv() => {
                if let Ok(event) = event {
                    match event {
                        Event::PendingTx(tx) => {
                            if tx.input.len() >= 4 {
                                let func_selector_bytes = &tx.input[0..4];
                                let func_selector_hex = hex::encode(func_selector_bytes);
                                match func_selector_hex.as_str() {
                                    UNSTAKE | TRANSFER_FROM => {
                                        match client.bundler.provider.get_block_number().await {
                                            Ok(block_number) => match func_selector_hex.as_str() {
                                                UNSTAKE => handle_unstake(tx.clone(), block_number, false),
                                                TRANSFER_FROM => {
                                                    handle_transfer(tx.clone(), block_number, false)
                                                }
                                                _ => unreachable!(),
                                            },
                                            Err(_) => {
                                                error!("Failed to get block number for tx {}", tx.hash);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        },
                        Event::Block(block) => {
                            info!("New block built | number: {:?}", block.number.unwrap());
                        let block = client
                            .provider
                            .get_block_with_txs(block.number.unwrap())
                            .await
                            .unwrap()
                            .unwrap();

                        for tx in block.transactions {
                            tokio::spawn(async move {
                                parse_tx(tx.clone()).await.unwrap();
                            });
                        }
                        },
                        _ => {}
                    }
                } else {
                    error!("Failed to get event");
                }
            },
            _ = shutdown_notify.notified() => {
                info!("Shutting down event handler.");
                break;
            },
        }
    }
}

fn handle_unstake(tx: Transaction, _block_number: U64, paused: bool) {
    let staking_contract: &str = "0xBc10f2E862ED4502144c7d632a3459F49DFCDB5e";

    if paused || tx.to.unwrap() != staking_contract.parse::<H160>().unwrap() {
        return;
    }
    let decoded: UnstakeCall = UnstakeCall::decode(&tx.input).unwrap();
    let amount: U256 = decoded.amount;
    info!(
        "Found pending unstake tx | hash: {:?} Amount: {:?}",
        tx.hash, amount
    );
    // bundle_transactions(bundler, block_num)
}

fn handle_transfer(tx: Transaction, block_number: U64, paused: bool) {
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

async fn parse_tx(tx: Transaction) -> Result<()> {
    match get_function_selector(tx.clone()).await {
        Ok(sig) => {
            println!(
                "Function signature: {} | Tx Hash: {:?} | Hex: {:?}",
                sig.0, tx.hash, sig.1
            );
        }
        Err(e) => {
            error!(
                "Failed to get function signature for tx {} | error: {:?}",
                tx.hash, e
            );
            return Ok(());
        }
    };
    Ok(())
}

pub async fn get_function_selector(tx: Transaction) -> Result<(String, String)> {
    if tx.input.len() >= 4 {
        let func_selector_bytes = &tx.input[0..4];
        let func_selector_hex = hex::encode(func_selector_bytes);
        let url = format!(
            "https://www.4byte.directory/api/v1/signatures/?hex_signature={}",
            func_selector_hex
        );
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch function signature: {}",
                response.status()
            ));
        }

        let json: Value = response.json().await?;
        if let Some(results) = json["results"].as_array() {
            if let Some(first_result) = results.last() {
                if let Some(signature) = first_result["text_signature"].as_str() {
                    return Ok((signature.to_string(), func_selector_hex));
                }
            }
        }

        Err(anyhow!("No matching function signature found"))
    } else {
        Err(anyhow!("Input data is too short"))
    }
}

async fn _bundle_transactions(bundler: Bundler, block_num: U64) -> Result<H256> {
    // todo!("create calldata for txs");
    let block = bundler
        .provider
        .get_block(block_num)
        .await
        .unwrap()
        .unwrap();
    let next_base_fee = U256::from(calculate_next_block_base_fee(
        block.gas_used,
        block.gas_limit,
        block.base_fee_per_gas.unwrap_or_default(),
    ));
    let max_priority_fee_per_gas = U256::from(1); // maybe have this as a config to change on the fly
    let max_fee_per_gas = next_base_fee + max_priority_fee_per_gas;
    // let calldata = self.bot.encode("recoverToken", (token_address,))?;
    let tx = bundler
        .create_tx(
            None,
            Bytes(bytes::Bytes::new()),
            Some(U256::from(30000)),
            None,
            Some(max_priority_fee_per_gas),
            Some(max_fee_per_gas),
        )
        .await?;
    let signed_tx = bundler.sign_tx(tx).await?;
    let bundle = bundler.to_bundle(vec![signed_tx], block_num);
    let bundle_hash = bundler.send_bundle(bundle).await?;

    Ok(bundle_hash)
}
