use std::collections::HashMap;

use primitives::Chain;

pub fn get_validators() -> HashMap<String, Vec<String>> {
    [
        (
            Chain::Cosmos.to_string(),
            vec![
                "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3".to_string(), // everstake
            ],
        ),
        (
            Chain::Osmosis.to_string(),
            vec![
                "osmovaloper1wgmdcxzp49vjgrqusgcagq6qefk4mtjv5c0k7q".to_string(), // everstake
            ],
        ),
        (
            Chain::Celestia.to_string(),
            vec![
                "celestiavaloper1eualhqh07w7p45g45hvrjagkcxsfnflzdw5jzg".to_string(), // everstake
            ],
        ),
        (
            Chain::Injective.to_string(),
            vec![
                "injvaloper134dct56cq5v7uerxcy2cn4m06mqf4dxrlgpp24".to_string(), // everstake
            ],
        ),
        (
            Chain::Sei.to_string(),
            vec![
                "seivaloper1ummny4p645xraxc4m7nphf7vxawfzt3p5hn47t".to_string(), // everstake
            ],
        ),
        (
            Chain::Sui.to_string(),
            vec![
                "0xbba318294a51ddeafa50c335c8e77202170e1f272599a2edc40592100863f638".to_string(), // everstake
            ],
        ),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>()
}
