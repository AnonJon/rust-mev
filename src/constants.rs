use std::collections::HashMap;

use ethers::{
    prelude::Lazy,
    types::{Address, H160, U256, U64},
};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Env {
    pub https_url: String,
    pub wss_url: String,
    pub chain_id: U64,
    pub private_key: String,
    pub signing_key: String,
}

impl Env {
    pub fn new() -> Self {
        Env {
            https_url: get_env("HTTPS_URL"),
            wss_url: get_env("WSS_URL"),
            chain_id: U64::from_str(&get_env("CHAIN_ID")).unwrap(),
            private_key: get_env("PRIVATE_KEY"),
            signing_key: get_env("SIGNING_KEY"),
        }
    }
}

pub fn func_selectors() -> HashMap<&'static str, &'static str> {
    let mut function_selectors = HashMap::new();

    // ERC-20 Function Selectors
    function_selectors.insert("a9059cbb", "transfer(address,uint256)");
    function_selectors.insert("095ea7b3", "approve(address,uint256)");
    function_selectors.insert("23b872dd", "transferFrom(address,address,uint256)");
    function_selectors.insert("70a08231", "balanceOf(address)");
    function_selectors.insert("18160ddd", "totalSupply()");

    // ERC-721 Function Selectors
    function_selectors.insert("42842e0e", "safeTransferFrom(address,address,uint256)");
    function_selectors.insert("a22cb465", "setApprovalForAll(address,bool)");
    function_selectors.insert("6352211e", "ownerOf(uint256)");

    // Other Common Functions
    function_selectors.insert("8da5cb5b", "owner()");
    function_selectors.insert("2e17de78", "unstake(uint256)");

    function_selectors
}

pub fn get_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("Environment variable '{}' not found", key))
}

pub static BUILDER_URLS: &[&str] = &[
    "https://builder0x69.io",
    "https://rpc.beaverbuild.org",
    "https://relay.flashbots.net",
    "https://rsync-builder.xyz",
    "https://api.blocknative.com/v1/auction",
    "https://builder.gmbit.co/rpc",
    "https://eth-builder.com",
    "https://mev.api.blxrbdn.com",
    "https://rpc.titanbuilder.xyz",
    "https://rpc.payload.de",
    "https://rpc.lightspeedbuilder.info",
    "https://rpc.nfactorial.xyz",
    "https://boba-builder.com/searcher",
    "https://rpc.f1b.io",
];
