use alloy_core::sol;

sol! {
    #[derive(Debug, PartialEq)]
    interface WETH9 {
        function withdraw(uint wad) public;
    }
}
