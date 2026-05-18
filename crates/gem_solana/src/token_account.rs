use std::sync::LazyLock;

use crate::{ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, Pubkey, SolanaError, find_program_address};

static ASSOCIATED_TOKEN_PROGRAM: LazyLock<Pubkey> = LazyLock::new(|| Pubkey::from_base58(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM).unwrap());

pub fn get_token_account(wallet: &str, token_mint: &str, token_program: &str) -> Result<String, SolanaError> {
    let owner = Pubkey::from_base58(wallet)?;
    let token_program = Pubkey::from_base58(token_program)?;
    let mint = Pubkey::from_base58(token_mint)?;
    let seeds = [owner.as_bytes().as_ref(), token_program.as_bytes().as_ref(), mint.as_bytes().as_ref()];

    Ok(find_program_address(&ASSOCIATED_TOKEN_PROGRAM, &seeds)?.0.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PYUSD_TOKEN_MINT, TOKEN_PROGRAM, TOKEN_PROGRAM_2022, USDC_TOKEN_MINT, USDS_TOKEN_MINT, USDT_TOKEN_MINT, WSOL_TOKEN_ADDRESS};

    #[test]
    fn test_get_token_account() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let test_cases = [
            (
                "CzVqG98YbFNiMREwgTswSML59CNrfobsNX4N9j6K8fbC",
                USDT_TOKEN_MINT,
                TOKEN_PROGRAM,
                "3gV5dwdpdBTQU3hQsXJqBKdJF5fD3Fv3RZSFZgKWcjfh",
            ),
            (
                "AzDByJsGm9gAVQPX8v8WS3iAs3PPdTwZZDDUNP2u5nVj",
                WSOL_TOKEN_ADDRESS,
                TOKEN_PROGRAM,
                "GkEZoxULxLSXo267QkdM1nLg87zTcpMJwUbxgkRxmLnV",
            ),
            (
                "9CXNmcRzZenixwtzAEVA3Jdo3yC6Hscxcpa4R7tQ6tTV",
                USDC_TOKEN_MINT,
                TOKEN_PROGRAM,
                "8vErkepS3arxRAQk66XiJuKLGi8jajRw2DXoNooHPejf",
            ),
            (
                "FC2QFuyPj5cRBkfi83f2EA9gJvFp8EEq2TKVbhqod1vz",
                USDS_TOKEN_MINT,
                TOKEN_PROGRAM,
                "APL8v7ptdmteuJqbByfQ64LmNYrCJPrLPbV5VEnMegwC",
            ),
            (
                "fr6yQkDmWy6R6pecbUsxXaw6EvRJznZ2HsK5frQgud8",
                PYUSD_TOKEN_MINT,
                TOKEN_PROGRAM_2022,
                "Ffeie177PRngys3SwNH44AYdT9yExm63GAmTXHdmL1k1",
            ),
        ];

        for (wallet, token_mint, token_program, expected_token_account) in test_cases.iter() {
            let fee_token_account = get_token_account(wallet, token_mint, token_program)?;
            assert_eq!(fee_token_account, *expected_token_account);
        }

        assert!(get_token_account("invalid", USDC_TOKEN_MINT, TOKEN_PROGRAM).is_err());

        Ok(())
    }
}
