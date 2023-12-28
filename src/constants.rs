use std::collections::HashMap;

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

    function_selectors
}
