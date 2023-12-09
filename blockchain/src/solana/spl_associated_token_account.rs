use solana_sdk::pubkey::Pubkey;

pub fn get_associated_token_address(
    program_id: &str,
    seeds: Vec<&str>,
    wallet_address: &str,
    token_mint_address: &str,
) -> String {
    let referral_program = Pubkey::try_from(program_id).unwrap();

    let mut bytes = Vec::new();

    for seed in seeds {
        bytes.push(seed.as_bytes())
    }

    let referral_key: Pubkey = Pubkey::try_from(wallet_address).unwrap();
    let mint = Pubkey::try_from(token_mint_address).unwrap();

    let binding = referral_key.to_bytes();
    bytes.push(&binding);

    let binding = mint.to_bytes();
    bytes.push(&binding);

    Pubkey::try_find_program_address(&bytes, &referral_program)
        .unwrap()
        .0
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_associated_token_address() {
        let associated_token_address = get_associated_token_address(
            "REFER4ZgmyYx9c6He5XfaTMiGfdLwRnkV4RPp9t9iF3",
            vec!["referral_ata"],
            "CK8n55Y664YjfifoVYfud8jXuSx9JV4NgVakEaRceVXu",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        );
        assert_eq!(
            associated_token_address,
            "8zENcuZni4EMpoy8fyGQ6FZffX7utkDCx9fL3SySuTWn"
        );

        let associated_token_address = get_associated_token_address(
            "REFER4ZgmyYx9c6He5XfaTMiGfdLwRnkV4RPp9t9iF3",
            vec!["referral_ata"],
            "CK8n55Y664YjfifoVYfud8jXuSx9JV4NgVakEaRceVXu",
            "So11111111111111111111111111111111111111112",
        );
        assert_eq!(
            associated_token_address,
            "6n5sDEwnejH1PC7ymh8WcoraXWuJeZAQ5WsPr3yebChd"
        );
    }
}
