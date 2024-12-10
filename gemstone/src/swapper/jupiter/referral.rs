use gem_solana::pubkey::Pubkey;

const REFERRAL_ATA: &str = "referral_ata";
const REFERRAL_PROGRAM: &str = "REFER4ZgmyYx9c6He5XfaTMiGfdLwRnkV4RPp9t9iF3";

pub fn get_referral_account(referral_key: &str, token_mint: &str) -> String {
    let referral_program = Pubkey::try_from(REFERRAL_PROGRAM).unwrap();
    let referral_key: Pubkey = Pubkey::try_from(referral_key).unwrap();
    let mint = Pubkey::try_from(token_mint).unwrap();
    let seeds = vec![REFERRAL_ATA.as_bytes(), referral_key.as_ref(), mint.as_ref()];

    Pubkey::try_find_program_address(&seeds, &referral_program).unwrap().0.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_solana::{USDC_TOKEN_MINT, WSOL_TOKEN_ADDRESS};
    #[test]
    fn test_get_referral_account() {
        let fee_token_account = get_referral_account("CK8n55Y664YjfifoVYfud8jXuSx9JV4NgVakEaRceVXu", USDC_TOKEN_MINT);

        assert_eq!(fee_token_account, "8zENcuZni4EMpoy8fyGQ6FZffX7utkDCx9fL3SySuTWn");

        let fee_token_account = get_referral_account("CK8n55Y664YjfifoVYfud8jXuSx9JV4NgVakEaRceVXu", WSOL_TOKEN_ADDRESS);

        assert_eq!(fee_token_account, "6n5sDEwnejH1PC7ymh8WcoraXWuJeZAQ5WsPr3yebChd");

        let fee_token_account = get_referral_account("97q7TdebuvmxXCM1JzgqzpB1i7Wgvk4ACUWanhiL6Dk1", USDC_TOKEN_MINT);

        assert_eq!(fee_token_account, "9HyUVfNr858FHzLyZt3Cdr3b5Mp1dGwjaudpVWD6Qawv");

        let fee_token_account = get_referral_account("97q7TdebuvmxXCM1JzgqzpB1i7Wgvk4ACUWanhiL6Dk1", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN");

        assert_eq!(fee_token_account, "Emk6jUQPPs5adT5Ve6EvR9XaAX4xDUDyGHCFXzt4FKgr");
    }
}
