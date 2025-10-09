use primitives::AssetId;

pub fn get_top_solana_tokens() -> Vec<String> {
    vec![
        "So11111111111111111111111111111111111111112".to_string(),  // SOL (wrapped)
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
        "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string(),  // JUP
        "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So".to_string(),  // mSOL
        "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj".to_string(), // stSOL
        "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1".to_string(),  // bSOL
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(), // BONK
        "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3".to_string(), // PYTH
        "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs".to_string(), // ETH (Wormhole)
        "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh".to_string(), // WBTC (Wormhole)
        "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm".to_string(), // INF
        "MEFNBXixkEbait3xn9bkm8WsJzXtVsaJEn4c8Sam21u".to_string(),  // MEF
        "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE".to_string(),  // ORCA
        "SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y".to_string(),  // SHDW
        "kinXdEcpDQeHPEuQnqmUgtYykqKGVFq6CeVX5iAHJq6".to_string(),  // KIN
        "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac".to_string(),  // MNGO
        "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt".to_string(),  // SRM
        "RLBxxFkseAZ4RgJH3Sqn8jXxhmGoz9jWxDNJMh8pL7a".to_string(),  // RLB
        "HxhWkVpk5NS4Ltg5nij2G671CKXFRKPK8vy271Ub4uEK".to_string(), // WIF
        "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm".to_string(), // WEN
        "ukHH6c7mMyiWCf1b9pnWe25TSpkDDt3H5pQZgZ74J82".to_string(),  // BERN
        "A3HyGZqe451CBesNqieNPfJ4A88WzLj646Fe8LUGK3pW".to_string(), // POPCAT
        "MEW1gQWJ3nEXg2qgERiKu7FAFj79PHvQVREQUzScPP5".to_string(),  // MEW
        "ATLASXmbPQxBUYbxPsV97usA3fPQYEqzQBUHgiFCUsXx".to_string(), // ATLAS
        "PoLisWXnNRwC6oBu1vHiuKQzFjGL4XDSu4g9qjz9qVk".to_string(),  // POLIS
        "6gnCPhXtLnUD76HjQuSYPENLSZdG8RvDB1pTLM5aLSJA".to_string(), // SLND
        "FU1q8vJpZNUrmqsciSjp8bAKKidGsLmouB8CBdf8TKQv".to_string(), // FIDA
        "rndrizKT3MK1iimdxRdWabcF7Zg7AR5T4nud4EkHBof".to_string(),  // RNDR
        "METAewgxyPbgwsseH8T16a39CQ5VyVxZi9zXiDPY18m".to_string(),  // META
        "8upjSpvjcdpuzhfR1zriwg5NXkwDruejqNE9WNbPRtyA".to_string(), // GRIFFIN
        "GDfnEsia2WLAW5t8yx2X5j2mkfA74i5kwGdDuZHt7XmG".to_string(), // GUAC
        "Comp4ssDzXcLeu2MnLuGNNFC4cmLPMng8qWHPzNbdPn".to_string(),  // COMP
        "AGFEad2et2ZJif9jaGpdMixQqvW5i81aBdvKe7PHNfz3".to_string(), // FOXY
        "CKaKtYvz6dKPyMvYq9Rh3UBrnNqYZAyd7iF4hJtjUvks".to_string(), // GOFX
        "C98A4nkJXhpVZNAZdHUA95RpTF3T4whtQubL3YobiUX9".to_string(), // C98
        "zebeczgi5fSEtbpfQKVZKCJ3WgYXxjkMUkNNx7fLKAF".to_string(),  // ZBC
        "EsPKhGTMf3bGoy4Qm7pCv3UCcWqAmbC1UGHBTDxRjjD4".to_string(), // RAY
        "PoRTjZMPXb9T7dyU7tpLEZRQj7e6ssfAE62j2oQuc6y".to_string(),  // PORT
        "SBR6oEiJp5pS5qhFvFQdaKJXHLt6qKWHq5M7FMX4pfN".to_string(),  // SBR
        "8wXtPeU6557ETkp9WHFY1n1EcU6NxDvbAggHGsMYiHsB".to_string(), // SAMO
        "CoRkC1e4uFKSZckLaZJGTWoCKhZnPXVwgJQf4G1bLJKu".to_string(), // CORK
        "GDDMwNyyx8uB6zrqwBFHjLLG3TBYk2F8Az4yrQC5RzMp".to_string(), // LILY
        "CsZ5LZkDS7h9TDKjrbL7VAwQZ9nsRu8vJLhRYfmGaN8K".to_string(), // ALEPH
        "HxRELUQfvvjToVbacjr9YECdfQMUqGgPYB68jVDYxkbr".to_string(), // PUFF
        "DFL1zNkaGPWm1BqAVqRjCZvHmwTFrEaJtbzJWgseoNJh".to_string(), // DFL
        "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo".to_string(), // POLIS
        "AMdnw9H5DFtQwZowVFr4kUgSXJzLruKzbGoe5HisH2Q4".to_string(), // MEDIA
        "StepAscQoEioFxxWGnh2sLBDFp9d8rvKz2Yp39iDpyT".to_string(),  // STEP
        "BLZEEuZUBVqFhj8adcCFPJvPVCiCyVmh3hkJMrU8KuJA".to_string(), // BLZE
        "GTH3wG3NErjwcf7VGCoXEXkgXSHvYhx5gtATeeM5JAS1".to_string(), // GOFX
        "CASHVDm2wsJXfhj6VWxb7GiMdoLc17Du7paH4bNr5woT".to_string(), // CASH
        "7jJZBeFhE1t3MRtNDTdC63K8CSsrRPMi2jTLwGVUL2Ps".to_string(), // SMRT
        "7Q2afV64in6N6SeZsAAB81TJzwDoD6zpqmHkzi9Dcavn".to_string(), // JSOL
        "5yxNbU8DgYJZNi3mPD9rs4XLh9ckXrhPjJ5VCujUWg5H".to_string(), // FRKT
        "SHARKSYJjqaNyxVfrpnBN9pjgkhwDhatnMyicWPnr1s".to_string(),  // SHARK
        "9vMJfxuKxXBoEa7rM12mYLMwTacLMLDJqHozw96WQL8i".to_string(), // UST
        "9S4t2NEAiJVMvPdRYKVrfJpBafPBLtvbvyS3DecojQHw".to_string(), // FRAX
        "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(), // RAY
        "FYpdBuyAHSbdaAyD1sKkxyLWbAP8uUW9h6uvdhK74ij1".to_string(), // DUST
        "GEJpt3Wjmr628FqXxTgxMce1pLntckPf1uB8S6VESsHK".to_string(), // RUNNER
        "FnKE9n6aGjQoNWRBZXy4RW6LZVao7qwBonUbiD7edUmZ".to_string(), // SYP
        "88881Hu2jGMfCs9tMu5Rr7Ah7WBNBuXqde4nR5ZmKYYy".to_string(), // ABR
        "abrUg6w5QyPaXPJCxvdW5mxycq4ytv5LZdJRqhDBH45".to_string(),  // ABR
        "GFX1ZjR2P15tmrSwow6FjyDYcEkoFb4p4gJCpLBjaxHD".to_string(), // GFX
        "METAmTMXwdb8gYzyCPfXXFmZZw4rUsXX58PNsDg7zjL".to_string(),  // METAL
        "CaM1FRr6k8xn4K2D6pWGx1iyMpRqWRz8iCPZ2LLyMvT5".to_string(), // SLIM
        "AZsHEMXd36Bj1EMNXhowJajpUXzrKcK57wW4ZGXVa7yR".to_string(), // GUAC
        "4wjPQJ6PrkC4dHhYghwJzGBVP78DkBzA2U3kHoFNBuhj".to_string(), // LIQ
        "SLRSSpSLUTP7okbCUBYStWCo1vUgyt775faPqz8HUMr".to_string(),  // SLRS
        "DUSTawucrTsGU8hcqRdHDCbuYhCPADMLM2VcCb8VnFnQ".to_string(), // DUST
        "5p2zjqCd1WJzAVgcEnjhb9zWDU7b9XVhFhx4usiyN7jB".to_string(), // APEX
        "4dmKkXNHdgYsXqBHCuMikNQWwVomZURhYvkkX5c4pQ7y".to_string(), // SNY
        "GENEtH5amGSi8kHAtQoezp1XEXwZJ8vcuePYnXdKrMYz".to_string(), // GENE
        "F3nefJBcejYbtdREjui1T9DPh5dBgpkKq7u2GAAMXs5B".to_string(), // SOLC
        "xxxxa1sKNGwFtw2kFn8XauW9xq8hBZ5kVtcSesTT9fW".to_string(),  // CAVE
        "SCHoR9zYgMxMMoouPBfVXR8FGVWuSrRXvU8bxQpNzRe".to_string(),  // SCHOR
        "CKDDr1JdBDQbWdmgmmvVbNTKnp1kPwHUXXzaHbGNmVJu".to_string(), // CHKD
        "5tN42n9vMi6ubp67Uy4NnmM5DMZYN8aS8GeB3bEDHr6E".to_string(), // FAB
        "SLNDpmoWTVADgEdndyvWzroNL7zSi1dF9PC3xHGtPwp".to_string(),  // SLND
        "8PMHT4swUMtBzgHnh5U564N5sjPSiUz2cjEQzFnnP1Fo".to_string(), // RIN
        "9cqNqBGVvmNADH9rVmKjyWBG8G3aK9b5TJUcJVn8vRXg".to_string(), // GST
        "4k7KFpFzKfPvJL4uEFkLmNEMH3fEZzBaGg2txvRn8BCe".to_string(), // APE
        "BLwTnYKqf7u4qjgZrrsKeNs2EzWkMLqVCu6j8iHyrNA3".to_string(), // BANK
        "EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp".to_string(), // FANT
        "FPJUuhUqSLz7V3hE4wjKdZqzL4kV2v59EaZ5mYhCNfLk".to_string(), // SONAR
        "FU1q8vJpZNUrmqsciSjp8bAKKidGsLmouB8CBdf8TKQv".to_string(), // FIDA
        "CKaKtYvz6dKPyMvYq9Rh3UBrnNqYZAyd7iF4hJtjUvks".to_string(), // GOFX
        "BdUJucPJyjkHxLMv6ipKNUhSeY3DWrVtgxAES1iSBAov".to_string(), // PRISM
        "SCSQLUJBMXWyZqhu2czSyLLFYVzJFBP4jAWbH1W7PQn".to_string(),  // SOLAPE
        "9tzZzEHsKnwFL1A3DyFJwj36KnZj3gZ7g4srWp9YTEoh".to_string(), // CRP
        "2Ysp8s3i76YHFzvZHrDcfZNdCVwu7Ax5vGMhBcnCPBzy".to_string(), // MEAN
        "ELXRYfXdPQ7B1wUPzXU9gfkW2APrD3Mxjfh8Zxs9Ds4n".to_string(), // REAL
        "NRVwhjBQiUPYtfDT5zRBVJajkGYQQHvMkRneuJM96kJ".to_string(),  // NIRV
        "HgXxBHx7Qxr6KW1T9VZmLHX3Gp5VTXJFHSqrBdUJsaPW".to_string(), // DGLN
        "5g9VT4mPM3q8h9L2kKU9gQJmF2b8qMnhpZSqNNKk1jSn".to_string(), // SUN
    ]
}

pub fn map_asset_id_to_token(asset_id: &AssetId) -> Option<String> {
    let asset_id_str = asset_id.to_string();
    match asset_id_str.as_str() {
        "solana_So11111111111111111111111111111111111111112" => Some("So11111111111111111111111111111111111111112".to_string()),
        "solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        "solana_Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()),
        "solana_JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" => Some("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".to_string()),
        _ => None,
    }
}
