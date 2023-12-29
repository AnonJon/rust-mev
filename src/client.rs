use anyhow::Result;
use ethers::providers::{Provider, Ws};
use std::sync::Arc;

pub struct Client {
    pub client: Arc<Provider<Ws>>,
}

impl Client {
    pub async fn new(rpc_url: String) -> Result<Self> {
        let provider = Provider::<Ws>::connect(rpc_url).await?;
        let client = Arc::new(provider);
        Ok(Self { client })
    }
}
