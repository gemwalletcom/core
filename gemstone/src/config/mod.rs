use std::collections::HashMap;

use primitives::Chain;

pub fn get_validators() -> HashMap<String, Vec<String>> {
    [
        (
            Chain::Cosmos.to_string(),
            vec![
                "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3".to_string(), // everstake
                "cosmosvaloper1n229vhepft6wnkt5tjpwmxdmcnfz55jv3vp77d".to_string(), // allnodes
            ],
        ),
        (
            Chain::Osmosis.to_string(),
            vec![
                "osmovaloper1wgmdcxzp49vjgrqusgcagq6qefk4mtjv5c0k7q".to_string(), // everstake
                "osmovaloper1e9ucjn5fjmetky5wezzcsccp7hqcwzrrdulz7n".to_string(), // allnodes
            ],
        ),
        (
            Chain::Celestia.to_string(),
            vec![
                "celestiavaloper1eualhqh07w7p45g45hvrjagkcxsfnflzdw5jzg".to_string(), // everstake
                "celestiavaloper1rcm7tth05klgkqpucdhm5hexnk49dfda3l3hak".to_string(), // allnodes
            ],
        ),
        (
            Chain::Injective.to_string(),
            vec![
                "injvaloper134dct56cq5v7uerxcy2cn4m06mqf4dxrlgpp24".to_string(), // everstake
                "injvaloper14w4qu60azpz5zkw5yrcvmaxqh6xg5sny7qm6rm".to_string(), // allnodes
            ],
        ),
        (
            Chain::Sei.to_string(),
            vec![
                "seivaloper1ummny4p645xraxc4m7nphf7vxawfzt3p5hn47t".to_string(), // everstake
                "seivaloper19n9amk4u0dhfvvaj0myhgc3nlkmj29vakw7ynz".to_string(), // allnodes
            ],
        ),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>()
}
