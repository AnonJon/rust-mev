use crate::constants::Config;
use anyhow::Result;
use ethers::providers::{Provider, Ws};
use std::sync::Arc;

use crate::bundler::Bundler;

pub struct Client {
    pub provider: Arc<Provider<Ws>>,
    pub bundler: Bundler,
}

impl Client {
    pub async fn new(config: Config) -> Result<Self> {
        let provider = Arc::new(Provider::<Ws>::connect(&config.wss_url).await?);
        let bundler = Bundler::new(&config);
        Ok(Self { provider, bundler })
    }
}
