use alloy_core::sol;

sol! {
    interface AcrossConfigStore {
        function l1TokenConfig(address l1Token) returns (string);
    }
}

// cast call 0x3B03509645713718B78951126E0A6de6f10043f5 "l1TokenConfig(address)(string)" 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 --rpc-url
