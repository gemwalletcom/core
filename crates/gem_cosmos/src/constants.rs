use primitives::chain_cosmos::CosmosChain;

pub const MESSAGE_DELEGATE: &str = "/cosmos.staking.v1beta1.MsgDelegate";
pub const MESSAGE_UNDELEGATE: &str = "/cosmos.staking.v1beta1.MsgUndelegate";
pub const MESSAGE_REDELEGATE: &str = "/cosmos.staking.v1beta1.MsgBeginRedelegate";
pub const MESSAGE_SEND_BETA: &str = "/cosmos.bank.v1beta1.MsgSend";
pub const MESSAGE_REWARD_BETA: &str = "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward";
pub const MESSAGE_SEND: &str = "/types.MsgSend"; // thorchain

pub const SUPPORTED_MESSAGES: &[&str] = &[
    MESSAGE_SEND,
    MESSAGE_SEND_BETA,
    MESSAGE_DELEGATE,
    MESSAGE_UNDELEGATE,
    MESSAGE_REDELEGATE,
    MESSAGE_REWARD_BETA,
];

pub const EVENTS_WITHDRAW_REWARDS_TYPE: &str = "withdraw_rewards";
pub const EVENTS_ATTRIBUTE_AMOUNT: &str = "amount";

pub fn get_base_fee(chain: CosmosChain) -> u64 {
    match chain {
        CosmosChain::Thorchain => 2_000_000,
        CosmosChain::Cosmos => 3_000,
        CosmosChain::Osmosis => 10_000,
        CosmosChain::Celestia => 3_000,
        CosmosChain::Sei => 100_000,
        CosmosChain::Injective => 100_000_000_000_000,
        CosmosChain::Noble => 25_000,
    }
}
