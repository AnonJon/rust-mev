use crate::constants::{Config, BUILDER_URLS};
use anyhow::{anyhow, Result};
use ethers::prelude::*;
use ethers::types::{
    transaction::{eip2718::TypedTransaction, eip2930::AccessList},
    Eip1559TransactionRequest, U256,
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
    pub config: Config,
    pub sender: LocalWallet,
    pub provider: SignerProvider,
    pub flashbots:
        SignerMiddleware<BroadcasterMiddleware<SignerProvider, LocalWallet>, LocalWallet>,
}

impl Bundler {
    pub fn new(config: &Config) -> Self {
        let sender = config
            .private_key
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(config.chain_id.as_u64());
        let signer = config
            .signing_key
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(config.chain_id.as_u64());

        let provider = Provider::<Http>::try_from(&config.https_url)
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
            config: config.clone(),
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

    pub fn to_bundle<T: Into<BundleTransaction>>(
        &self,
        signed_txs: Vec<T>,
        block_number: U64,
    ) -> BundleRequest {
        let mut bundle = BundleRequest::new();

        for tx in signed_txs {
            let bundle_tx: BundleTransaction = tx.into();
            bundle = bundle.push_transaction(bundle_tx);
        }

        bundle
            .set_block(block_number + 1)
            .set_simulation_block(block_number)
            .set_simulation_timestamp(0)
    }

    pub async fn send_bundle(&self, bundle: BundleRequest) -> Result<TxHash> {
        let simulated = self.flashbots.inner().simulate_bundle(&bundle).await?;

        for tx in &simulated.transactions {
            if let Some(e) = &tx.error {
                return Err(anyhow!("Simulation error: {:?}", e));
            }
            if let Some(r) = &tx.revert {
                return Err(anyhow!("Simulation revert: {:?}", r));
            }
        }

        let pending_bundle = self.flashbots.inner().send_bundle(&bundle).await?;
        for result in pending_bundle {
            match result {
                Ok(pending_bundle) => match pending_bundle.await {
                    Ok(bundle_hash) => return Ok(bundle_hash),
                    Err(PendingBundleError::BundleNotIncluded) => {
                        return Err(anyhow!("Bundle was not included in target block."));
                    }
                    Err(e) => {
                        return Err(anyhow!("Bundle error: {:?}", e));
                    }
                },
                Err(e) => {
                    return Err(anyhow!("Bundle error: {:?}", e));
                }
            }
        }
        Err(anyhow!("No bundles to process"))
    }

    pub async fn create_tx(
        &self,
        to: Option<NameOrAddress>,
        data: Bytes,
        gas: Option<U256>,
        value: Option<U256>,
        max_priority_fee_per_gas: Option<U256>,
        max_fee_per_gas: Option<U256>,
    ) -> Result<Eip1559TransactionRequest> {
        let nonce = self
            .provider
            .get_transaction_count(self.sender.address(), None)
            .await?;
        let tx = Eip1559TransactionRequest {
            to,
            from: Some(self.sender.address()),
            data: Some(data),
            gas,
            nonce: Some(nonce),
            value,
            access_list: AccessList::default(),
            chain_id: Some(self.config.chain_id),
            max_priority_fee_per_gas,
            max_fee_per_gas,
        };
        Ok(tx)
    }

    pub async fn send_tx(&self, tx: Eip1559TransactionRequest) -> Result<TxHash> {
        let pending_tx = self.provider.send_transaction(tx, None).await?;
        let receipt = pending_tx.await?.ok_or_else(|| anyhow!("Tx dropped"))?;
        Ok(receipt.transaction_hash)
    }
}
