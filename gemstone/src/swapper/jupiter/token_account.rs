use gem_solana::{pubkey::Pubkey, ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, TOKEN_PROGRAM};

pub fn get_token_account(wallet: &str, token_mint: &str) -> String {
    let owner = Pubkey::try_from(wallet).unwrap();
    let token_program = Pubkey::try_from(TOKEN_PROGRAM).unwrap();
    let mint = Pubkey::try_from(token_mint).unwrap();
    let associated_token_program = Pubkey::try_from(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM).unwrap();
    let seeds = vec![owner.as_ref(), token_program.as_ref(), mint.as_ref()];

    Pubkey::try_find_program_address(&seeds, &associated_token_program).unwrap().0.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_solana::{USDC_TOKEN_MINT, USDS_TOKEN_MINT, USDT_TOKEN_MINT, WSOL_TOKEN_ADDRESS};
    #[test]
    fn test_get_token_account() {
        // Define test cases as (wallet, token_mint, expected_token_account)
        let test_cases = [
            (
                "CzVqG98YbFNiMREwgTswSML59CNrfobsNX4N9j6K8fbC",
                USDT_TOKEN_MINT,
                "3gV5dwdpdBTQU3hQsXJqBKdJF5fD3Fv3RZSFZgKWcjfh",
            ),
            (
                "AzDByJsGm9gAVQPX8v8WS3iAs3PPdTwZZDDUNP2u5nVj",
                WSOL_TOKEN_ADDRESS,
                "GkEZoxULxLSXo267QkdM1nLg87zTcpMJwUbxgkRxmLnV",
            ),
            (
                "9CXNmcRzZenixwtzAEVA3Jdo3yC6Hscxcpa4R7tQ6tTV",
                USDC_TOKEN_MINT,
                "8vErkepS3arxRAQk66XiJuKLGi8jajRw2DXoNooHPejf",
            ),
            (
                "FC2QFuyPj5cRBkfi83f2EA9gJvFp8EEq2TKVbhqod1vz",
                USDS_TOKEN_MINT,
                "APL8v7ptdmteuJqbByfQ64LmNYrCJPrLPbV5VEnMegwC",
            ),
        ];

        for (wallet, token_mint, expected_token_account) in test_cases.iter() {
            let fee_token_account = get_token_account(wallet, token_mint);
            assert_eq!(fee_token_account, *expected_token_account);
        }
    }
}
