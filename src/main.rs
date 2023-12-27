use anyhow::Result;
use ethers::{
    providers::{Middleware, StreamExt},
    types::{
        transaction::eip2930::AccessList, BlockNumber, Bytes, Eip1559TransactionRequest,
        NameOrAddress, H160, U256,
    },
};
use mev::client::Client;
use std::env;
#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = &env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
    let web3 = Client::new(rpc_url.to_string()).await?;
    let stream = web3.client.subscribe_pending_txs().await?;
    let mut stream = stream.transactions_unordered(256).fuse();
    while let Some(tx) = stream.next().await {
        println!("{:?}", tx);
    }

    Ok(())
}
