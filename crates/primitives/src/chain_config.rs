use crate::{AssetType, Chain, ChainType};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChainStack {
    Native,
    Optimism,
    ZkSync,
}

#[derive(Debug, Clone)]
pub struct EvmChainConfig {
    pub min_priority_fee: u64,
    pub chain_stack: ChainStack,
    pub is_ethereum_layer2: bool,
    pub weth_contract: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct StakeChainConfig {
    pub lock_time: u64,
    pub min_stake_amount: u64,
    pub change_amount_on_unstake: bool,
    pub can_redelegate: bool,
    pub can_withdraw: bool,
    pub can_claim_rewards: bool,
    pub reserved_for_fees: u64,
}

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain: Chain,
    pub network_id: &'static str,
    pub denom: Option<&'static str>,
    pub slip44: i64,
    pub chain_type: ChainType,
    pub default_asset_type: Option<AssetType>,
    pub account_activation_fee: Option<i32>,
    pub token_activation_fee: Option<i32>,
    pub minimum_account_balance: Option<u64>,
    pub block_time: u32,
    pub rank: i32,
    pub is_swap_supported: bool,
    pub is_nft_supported: bool,
    pub is_utxo: bool,
    pub evm: Option<EvmChainConfig>,
    pub stake: Option<StakeChainConfig>,
}

const fn base_chain_config(
    chain: Chain,
    network_id: &'static str,
    slip44: i64,
    chain_type: ChainType,
    block_time: u32,
    rank: i32,
) -> ChainConfig {
    ChainConfig {
        chain,
        network_id,
        denom: None,
        slip44,
        chain_type,
        default_asset_type: None,
        account_activation_fee: None,
        token_activation_fee: None,
        minimum_account_balance: None,
        block_time,
        rank,
        is_swap_supported: false,
        is_nft_supported: false,
        is_utxo: false,
        evm: None,
        stake: None,
    }
}

const fn evm_config(
    min_priority_fee: u64,
    chain_stack: ChainStack,
    is_ethereum_layer2: bool,
    weth_contract: Option<&'static str>,
) -> EvmChainConfig {
    EvmChainConfig {
        min_priority_fee,
        chain_stack,
        is_ethereum_layer2,
        weth_contract,
    }
}

const fn stake_config(
    lock_time: u64,
    min_stake_amount: u64,
    change_amount_on_unstake: bool,
    can_redelegate: bool,
    can_withdraw: bool,
    can_claim_rewards: bool,
    reserved_for_fees: u64,
) -> StakeChainConfig {
    StakeChainConfig {
        lock_time,
        min_stake_amount,
        change_amount_on_unstake,
        can_redelegate,
        can_withdraw,
        can_claim_rewards,
        reserved_for_fees,
    }
}

// Centralized chain configurations. Add new chains here.
static CHAIN_CONFIGS: &[ChainConfig] = &[
    ChainConfig {
        is_swap_supported: true,
        is_utxo: true,
        ..base_chain_config(
            Chain::Bitcoin,
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            0,
            ChainType::Bitcoin,
            600_000,
            100,
        )
    },
    ChainConfig {
        is_swap_supported: true,
        is_utxo: true,
        ..base_chain_config(
            Chain::BitcoinCash,
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            145,
            ChainType::Bitcoin,
            600_000,
            40,
        )
    },
    ChainConfig {
        is_swap_supported: true,
        is_utxo: true,
        ..base_chain_config(
            Chain::Litecoin,
            "12a765e31ffd4059bada1e25190f6e98c99d9714d334efa41a195a7e7e04bfe2",
            2,
            ChainType::Bitcoin,
            120_000,
            30,
        )
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        is_nft_supported: true,
        evm: Some(evm_config(
            100_000_000,
            ChainStack::Native,
            false,
            Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
        )),
        stake: Some(stake_config(
            1_209_600,
            100_000_000_000_000_000,
            true,
            false,
            true,
            false,
            5_000_000_000_000_000,
        )),
        ..base_chain_config(Chain::Ethereum, "1", 60, ChainType::Ethereum, 12_000, 85)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::BEP20),
        is_swap_supported: true,
        is_nft_supported: true,
        evm: Some(evm_config(
            50_000_000,
            ChainStack::Native,
            false,
            Some("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"),
        )),
        stake: Some(stake_config(
            604_800,
            1_000_000_000_000_000_000,
            true,
            true,
            true,
            false,
            250_000_000_000_000,
        )),
        ..base_chain_config(Chain::SmartChain, "56", 9006, ChainType::Ethereum, 1_000, 80)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::SPL),
        token_activation_fee: Some(2_039_280),
        minimum_account_balance: Some(890_880),
        is_swap_supported: true,
        is_nft_supported: true,
        stake: Some(stake_config(259_200, 10_000_000, false, false, true, false, 5_000_000)),
        ..base_chain_config(
            Chain::Solana,
            "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d",
            501,
            ChainType::Solana,
            500,
            80,
        )
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        is_nft_supported: true,
        evm: Some(evm_config(
            30_000_000_000,
            ChainStack::Native,
            false,
            Some("0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"),
        )),
        ..base_chain_config(Chain::Polygon, "137", 60, ChainType::Ethereum, 3_000, 30)
    },
    ChainConfig {
        denom: Some("rune"),
        is_swap_supported: true,
        ..base_chain_config(Chain::Thorchain, "thorchain-1", 931, ChainType::Cosmos, 2_000, 30)
    },
    ChainConfig {
        denom: Some("uatom"),
        is_swap_supported: true,
        stake: Some(stake_config(1_814_400, 0, true, true, false, true, 25_000)),
        ..base_chain_config(Chain::Cosmos, "cosmoshub-4", 118, ChainType::Cosmos, 6_000, 40)
    },
    ChainConfig {
        denom: Some("uosmo"),
        stake: Some(stake_config(1_209_600, 0, true, true, false, true, 10_000)),
        ..base_chain_config(Chain::Osmosis, "osmosis-1", 118, ChainType::Cosmos, 6_000, 50)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            10_000_000,
            ChainStack::Native,
            true,
            Some("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"),
        )),
        ..base_chain_config(Chain::Arbitrum, "42161", 60, ChainType::Ethereum, 1_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::JETTON),
        is_swap_supported: true,
        ..base_chain_config(
            Chain::Ton,
            "F6OpKZKqvqeFp6CQmFomXNMfMj2EnaUSOXN+Mh+wVWk=",
            607,
            ChainType::Ton,
            5_000,
            50,
        )
    },
    ChainConfig {
        default_asset_type: Some(AssetType::TRC20),
        is_swap_supported: true,
        stake: Some(stake_config(1_209_600, 1_000_000, true, true, true, true, 10_000_000)),
        ..base_chain_config(Chain::Tron, "", 195, ChainType::Tron, 3_000, 70)
    },
    ChainConfig {
        is_swap_supported: true,
        is_utxo: true,
        ..base_chain_config(
            Chain::Doge,
            "1a91e3dace36e2be3bf030a65679fe821aa1d6ef92e7c9902eb318182c355691",
            3,
            ChainType::Bitcoin,
            60_000,
            30,
        )
    },
    ChainConfig {
        is_swap_supported: true,
        is_utxo: true,
        ..base_chain_config(
            Chain::Zcash,
            "00040fe8ec8471911baa1db1266ea15dd06b4a8a5c453883c000b031973dce08",
            133,
            ChainType::Bitcoin,
            75_000,
            30,
        )
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000,
            ChainStack::Optimism,
            true,
            Some("0x4200000000000000000000000000000000000006"),
        )),
        ..base_chain_config(Chain::Optimism, "10", 60, ChainType::Ethereum, 2_000, 30)
    },
    ChainConfig {
        denom: Some("0x1::aptos_coin::AptosCoin"),
        default_asset_type: Some(AssetType::TOKEN),
        is_swap_supported: true,
        stake: Some(stake_config(
            2_592_000,
            1_100_000_000,
            false,
            false,
            true,
            false,
            1_000_000,
        )),
        ..base_chain_config(Chain::Aptos, "1", 637, ChainType::Aptos, 500, 40)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            5_000_000,
            ChainStack::Optimism,
            true,
            Some("0x4200000000000000000000000000000000000006"),
        )),
        ..base_chain_config(Chain::Base, "8453", 60, ChainType::Ethereum, 2_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            25_000_000_000,
            ChainStack::Native,
            false,
            Some("0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7"),
        )),
        ..base_chain_config(Chain::AvalancheC, "43114", 9005, ChainType::Ethereum, 2_000, 30)
    },
    ChainConfig {
        denom: Some("0x2::sui::SUI"),
        default_asset_type: Some(AssetType::TOKEN),
        is_swap_supported: true,
        stake: Some(stake_config(86_400, 1_000_000_000, false, false, false, false, 100_000_000)),
        ..base_chain_config(Chain::Sui, "35834a8a", 784, ChainType::Sui, 500, 40)
    },
    ChainConfig {
        account_activation_fee: Some(1_000_000),
        token_activation_fee: Some(200_000),
        is_swap_supported: true,
        ..base_chain_config(Chain::Xrp, "", 144, ChainType::Xrp, 4_000, 40)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::BEP20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000,
            ChainStack::Optimism,
            false,
            Some("0x4200000000000000000000000000000000000006"),
        )),
        ..base_chain_config(Chain::OpBNB, "204", 60, ChainType::Ethereum, 1_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            3_500_000_000,
            ChainStack::Native,
            false,
            Some("0x21be370D5312f44cB42ce377BC9b8a0cEF1A4C83"),
        )),
        ..base_chain_config(Chain::Fantom, "250", 60, ChainType::Ethereum, 1_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            3_000_000_000,
            ChainStack::Native,
            false,
            Some("0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d"),
        )),
        ..base_chain_config(Chain::Gnosis, "100", 60, ChainType::Ethereum, 5_000, 30)
    },
    ChainConfig {
        denom: Some("utia"),
        stake: Some(stake_config(1_814_400, 0, true, true, false, true, 100_000)),
        ..base_chain_config(Chain::Celestia, "celestia", 118, ChainType::Cosmos, 6_000, 40)
    },
    ChainConfig {
        denom: Some("inj"),
        stake: Some(stake_config(
            1_814_400,
            0,
            true,
            true,
            false,
            true,
            10_000_000_000_000_000,
        )),
        ..base_chain_config(Chain::Injective, "injective-1", 60, ChainType::Cosmos, 6_000, 40)
    },
    ChainConfig {
        denom: Some("usei"),
        stake: Some(stake_config(1_814_400, 0, true, true, false, true, 100_000)),
        ..base_chain_config(Chain::Sei, "pacific-1", 118, ChainType::Cosmos, 1_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            10_000_000,
            ChainStack::Native,
            false,
            Some("0x0dc808adce2099a9f62aa87d9670745aba741746"),
        )),
        ..base_chain_config(Chain::Manta, "169", 60, ChainType::Ethereum, 2_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            200_000_000,
            ChainStack::Native,
            true,
            Some("0x4300000000000000000000000000000000000004"),
        )),
        ..base_chain_config(Chain::Blast, "81457", 60, ChainType::Ethereum, 2_000, 30)
    },
    ChainConfig {
        denom: Some("uusdc"),
        ..base_chain_config(Chain::Noble, "noble-1", 118, ChainType::Cosmos, 6_000, 20)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            20_000_000,
            ChainStack::ZkSync,
            true,
            Some("0x5AEa5775959fBC2557Cc8789bC1bf90A239D9a91"),
        )),
        ..base_chain_config(Chain::ZkSync, "324", 60, ChainType::Ethereum, 1_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            50_000_000,
            ChainStack::Native,
            true,
            Some("0xe5D7C2a44FfDDf6b295A15c148167daaAf5Cf34f"),
        )),
        ..base_chain_config(Chain::Linea, "59144", 60, ChainType::Ethereum, 1_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            10_000_000,
            ChainStack::Native,
            true,
            Some("0x78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8"),
        )),
        ..base_chain_config(Chain::Mantle, "5000", 60, ChainType::Ethereum, 1_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            10_000_000,
            ChainStack::Optimism,
            true,
            Some("0x471EcE3750Da237f93B8E339c536989b8978a438"),
        )),
        ..base_chain_config(Chain::Celo, "42220", 60, ChainType::Ethereum, 1_000, 30)
    },
    ChainConfig {
        is_swap_supported: true,
        ..base_chain_config(Chain::Near, "mainnet", 397, ChainType::Near, 1_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000,
            ChainStack::Optimism,
            true,
            Some("0x4200000000000000000000000000000000000006"),
        )),
        ..base_chain_config(Chain::World, "480", 60, ChainType::Ethereum, 2_000, 30)
    },
    ChainConfig {
        account_activation_fee: Some(10_000_000),
        is_swap_supported: true,
        ..base_chain_config(
            Chain::Stellar,
            "Public Global Stellar Network ; September 2015",
            148,
            ChainType::Stellar,
            6_000,
            30,
        )
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            10_000_000,
            ChainStack::Native,
            false,
            Some("0x039e2fB66102314Ce7b64Ce5Ce3E5183bc94aD38"),
        )),
        ..base_chain_config(Chain::Sonic, "146", 60, ChainType::Ethereum, 500, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ASA),
        account_activation_fee: Some(100_000),
        ..base_chain_config(Chain::Algorand, "mainnet-v1.0", 283, ChainType::Algorand, 4_000, 30)
    },
    ChainConfig {
        minimum_account_balance: Some(10_000_000_000),
        ..base_chain_config(Chain::Polkadot, "Polkadot Asset Hub", 354, ChainType::Polkadot, 5_000, 40)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            100_000,
            ChainStack::Native,
            false,
            Some("0x6100E367285b01F48D07953803A2d8dCA5D19873"),
        )),
        ..base_chain_config(Chain::Plasma, "9745", 60, ChainType::Ethereum, 2_000, 30)
    },
    ChainConfig {
        is_swap_supported: true,
        is_utxo: true,
        ..base_chain_config(Chain::Cardano, "764824073", 1_815, ChainType::Cardano, 20_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000,
            ChainStack::ZkSync,
            true,
            Some("0x3439153EB7AF838Ad19d56E1571FBD09333C2809"),
        )),
        ..base_chain_config(Chain::Abstract, "2741", 60, ChainType::Ethereum, 1_000, 35)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000_000,
            ChainStack::Native,
            false,
            Some("0x6969696969696969696969696969696969696969"),
        )),
        ..base_chain_config(Chain::Berachain, "80094", 60, ChainType::Ethereum, 2_000, 35)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000,
            ChainStack::Optimism,
            true,
            Some("0x4200000000000000000000000000000000000006"),
        )),
        ..base_chain_config(Chain::Ink, "57073", 60, ChainType::Ethereum, 1_000, 35)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000,
            ChainStack::Optimism,
            true,
            Some("0x4200000000000000000000000000000000000006"),
        )),
        ..base_chain_config(Chain::Unichain, "130", 60, ChainType::Ethereum, 1_000, 35)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000_000,
            ChainStack::Native,
            false,
            Some("0x5555555555555555555555555555555555555555"),
        )),
        ..base_chain_config(Chain::Hyperliquid, "999", 60, ChainType::Ethereum, 2_000, 40)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        stake: Some(stake_config(604_800, 1_000_000, true, false, false, false, 0)),
        ..base_chain_config(Chain::HyperCore, "1337", 60, ChainType::HyperCore, 2_000, 40)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        is_swap_supported: true,
        evm: Some(evm_config(
            1_000_000_000,
            ChainStack::Native,
            false,
            Some("0x3bd359C1119dA7Da1D913D1C4D2B7c461115433A"),
        )),
        stake: Some(stake_config(
            86_400,
            100_000_000_000_000_000,
            true,
            false,
            true,
            true,
            50_000_000_000_000_000,
        )),
        ..base_chain_config(Chain::Monad, "143", 60, ChainType::Ethereum, 500, 40)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        evm: Some(evm_config(
            1_000_000_000,
            ChainStack::Native,
            true,
            Some("0xe538905cf8410324e03a5a23c1c177a474d59b2b"),
        )),
        ..base_chain_config(Chain::XLayer, "196", 60, ChainType::Ethereum, 2_000, 30)
    },
    ChainConfig {
        default_asset_type: Some(AssetType::ERC20),
        evm: Some(evm_config(
            1_000_000,
            ChainStack::Native,
            false,
            Some("0x779Ded0c9e1022225f8E0630b35a9b54bE713736"),
        )),
        ..base_chain_config(Chain::Stable, "988", 60, ChainType::Ethereum, 1_000, 30)
    },
];

pub fn get_chain_config(chain: Chain) -> &'static ChainConfig {
    CHAIN_CONFIGS
        .iter()
        .find(|config| config.chain == chain)
        .unwrap_or_else(|| panic!("Missing chain config for {chain}"))
}
