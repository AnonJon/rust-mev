use crate::constants::{Env, BUILDER_URLS};
use anyhow::{anyhow, Result};
use ethers::prelude::*;
use ethers::types::{
    transaction::{eip2718::TypedTransaction, eip2930::AccessList},
    Address, Eip1559TransactionRequest, U256,
};
use ethers::{
    middleware::MiddlewareBuilder,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
};
use ethers_flashbots::*;
use url::Url;

type SignerProvider = SignerMiddleware<Provider<Http>, LocalWallet>;
pub struct Bundler {
    pub env: Env,
    pub sender: LocalWallet,
    pub provider: SignerProvider,
    pub flashbots:
        SignerMiddleware<BroadcasterMiddleware<SignerProvider, LocalWallet>, LocalWallet>,
}

impl Bundler {
    pub fn new<'a>() -> Self {
        let env = Env::new();

        let sender = env
            .private_key
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(env.chain_id.as_u64());
        let signer = env
            .signing_key
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(env.chain_id.as_u64());

        let provider = Provider::<Http>::try_from(&env.https_url)
            .unwrap()
            .with_signer(sender.clone());

        let flashbots = SignerMiddleware::new(
            BroadcasterMiddleware::new(
                provider.clone(),
                BUILDER_URLS
                    .iter()
                    .map(|url| url.parse().unwrap())
                    .collect(),
                Url::parse("https://relay.flashbots.net").unwrap(),
                signer,
            ),
            sender.clone(),
        );

        Self {
            env,
            sender,
            provider,
            flashbots,
        }
    }

    pub async fn sign_tx(&self, tx: Eip1559TransactionRequest) -> Result<Bytes> {
        let typed = TypedTransaction::Eip1559(tx);
        let signature = self.sender.sign_transaction(&typed).await?;
        let signed = typed.rlp_signed(&signature);
        Ok(signed)
    }
}
