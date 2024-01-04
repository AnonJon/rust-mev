// Standard contracts
pub mod erc20 {
    use ethers::prelude::abigen;
    abigen!(IERC20, "./abi/IERC20.json");
}

// Chainlink contracts
pub mod community_pool {
    use ethers::prelude::abigen;
    abigen!(ICommunityPool, "./abi/ICommunityPool.json");
}

// Compound contracts
pub mod comet {
    use ethers::prelude::abigen;
    abigen!(Comet, "./abi/compound/Comet.json");
}
