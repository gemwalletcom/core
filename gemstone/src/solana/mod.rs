use gem_solana::models::jito;

#[uniffi::export]
pub fn solana_create_jito_tip_instruction(from: String, lamports: u64) -> String {
    jito::create_jito_tip_instruction_json(&from, lamports)
}
