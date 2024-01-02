use crate::bundler::Bundler;
use crate::client::Client;
use crate::{
    contract_interfaces::{TransferFromCall, UnstakeCall},
    streams::Event,
};
use anyhow::Result;
use ethers::{
    core::abi::AbiDecode,
    providers::Middleware,
    types::{Bytes, Transaction, H160, H256, U256, U64},
};
use log::{error, info};
use tokio::sync::broadcast::Sender;

pub async fn event_handler(client: Client, event_sender: Sender<Event>) {
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
                                let block_number =
                                    match client.bundler.provider.get_block_number().await {
                                        Ok(block_number) => block_number,
                                        Err(_) => {
                                            error!("Failed to get block number");
                                            continue;
                                        }
                                    };
                                handle_unstake(tx.clone(), block_number, false);
                            }
                            TRANSFER_FROM => {
                                let block_number =
                                    match client.bundler.provider.get_block_number().await {
                                        Ok(block_number) => block_number,
                                        Err(_) => {
                                            error!("Failed to get block number");
                                            continue;
                                        }
                                    };
                                handle_transfer(tx.clone(), block_number, false);
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
    println!("block number: {:?}", block_number);
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

async fn _bundle_transactions(bundler: Bundler, block_num: U64) -> Result<H256> {
    // let calldata = self.bot.encode("recoverToken", (token_address,))?;
    let tx = bundler
        .create_tx(
            None,
            Bytes(bytes::Bytes::new()),
            Some(U256::from(30000)),
            None,
            None,
            None,
        )
        .await?;
    let signed_tx = bundler.sign_tx(tx).await?;
    let bundle = bundler.to_bundle(vec![signed_tx], block_num);
    let bundle_hash = bundler.send_bundle(bundle).await?;

    Ok(bundle_hash)
}
