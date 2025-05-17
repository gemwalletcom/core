use rand::Rng;
use rand::RngCore;

pub fn generate_random_seed(max_bytes: usize) -> Vec<u8> {
    if max_bytes == 0 || max_bytes > 32 {
        return vec![];
    }

    let mut rng = rand::rng();
    let num_bytes = rng.random_range(1..=max_bytes);

    let mut seed_bytes = vec![0u8; num_bytes];
    rng.fill_bytes(&mut seed_bytes);

    seed_bytes
}
