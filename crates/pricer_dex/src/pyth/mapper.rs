use primitives::Chain;

// https://www.pyth.network/price-feeds
pub fn price_account_for_chain(chain: Chain) -> &'static str {
    match chain {
        Chain::Bitcoin => "GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU",
        Chain::BitcoinCash => "5ALDzwcRJfSyGdGyhP3kP628aqBNHZzLuVww7o9kdspe",
        Chain::Litecoin => "8RMnV1eD55iqUFJLMguPkYBkq8DCtx81XcmAja93LvRR",
        Chain::Ethereum
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::Base
        | Chain::Linea
        | Chain::Manta
        | Chain::ZkSync
        | Chain::Abstract
        | Chain::Ink
        | Chain::Unichain
        | Chain::Blast
        | Chain::World => "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB",
        Chain::SmartChain | Chain::OpBNB => "4CkQJBxhU8EZ2UjhigbtdaPbpTe6mqf811fipYBFbSYN",
        Chain::Solana => "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG",
        Chain::Polygon => "JDbuHmbAJvsBsLKSnZG8Fa9nkiQRyYfg8fGdcYwfu2u7",
        Chain::Thorchain => "7T1CEv5TXeheCiJeoXY7MwgeDH4YGEkVXkF3gwQP8EGD",
        Chain::Cosmos => "CrCpTerNqtZvqLcKqz1k13oVeXV9WkMD2zA9hBKXrsbN",
        Chain::Osmosis => "6x6pVu4drvkYVMMbELBourAjYL5hoeJco4tdiAdn53bT",
        Chain::Ton => "AFJXXYuniABNnoEE7DLtkxwqLDkcda4xG5k2F4FB86hj",
        Chain::Tron => "7yjcov5hfpr6iG67sh848jiSXDzrp4A1JbFkrpZrnFbg",
        Chain::Doge => "FsSM3s38PX9K7Dn6eGzuE29S2Dsk1Sss1baytTQdCaQj",
        Chain::Aptos => "FNNvb1AFDnDVPkocEri8mWbJ1952HQZtFLuwPiUjSJQ",
        Chain::AvalancheC => "Ax9ujW5B9oqcv59N8m6f1BpTBq2rGeGaBcpKjC5UYsXU",
        Chain::Sui => "3Qub3HaAJaa2xNY7SUqPKd3vVwTqDfDDkEUMPjXD2c1q",
        Chain::Xrp => "Guffb8DAAxNH6kdoawYjPXTbwUhjmveh8R4LM6uEqRV1",
        Chain::Celestia => "funeUsHgi2QKkLdUPASRLuYkaK8JaazCEz3HikbkhVt",
        Chain::Injective => "9EdtbaivHQYA4Nh3XzGR6DwRaoorqXYnmpfsnFhvwuVj",
        Chain::Sei => "9EdtbaivHQYA4Nh3XzGR6DwRaoorqXYnmpfsnFhvwuVj",
        Chain::Noble => "Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD",
        Chain::Mantle => "6GDSnZb6UEnrvoVWGQPgMY28vV6XtnNQkQZ2eJrnNLFv",
        Chain::Celo => "9SWiag8E2zG3hF4UL8VzXU84hqZwfEVqvCeyqwfTdfAp",
        Chain::Near => "ECSFWQ1bnnpqPVvoy9237t2wddZAaHisW88mYxuEHKWf",
        Chain::Stellar => "DMw1AaDmW9g8cCcB4KqCAdWiDNkyFJg7cQ6nhdHpMN6j",
        Chain::Algorand => "HqFyq1wh1xKvL7KDqqT7NJeSPdAqsDqnmBisUC2XdXAX",
        Chain::Polkadot => "EcV1X1gY2yb4KXxjVQtTHTbioum2gvmPnFk4zYAt7zne",
        Chain::Cardano => "3pyn4svBbxJ9Wnn3RVeafyLWfzie6yC5eTig2S62v9SC",
        Chain::Berachain => "B72vp52SUipn1gaBadkBk5MSMjMqS8gSaNUz4jBkAm9E",
        Chain::Hyperliquid => "5UVhvt9NzyyVuVaePfkaEaHD6hoD9Dw7PFUCfRoMh8i6",
        Chain::Fantom | Chain::Sonic => "HTgSrKu2XfDc7wEm9ZJn6tavhKFD6EHNcnEBxBsArFzL",
        Chain::Gnosis => "CtJ8EkqLmeYyGB8s4jevpeNsvmD4dxVR2krfsDLcvV8Y",
        Chain::Rootstock => "",
        Chain::Monad => "Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD", //TODO: Monad. USDC. FIX in the future.
    }
}
