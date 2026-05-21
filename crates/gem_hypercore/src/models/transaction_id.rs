use std::fmt::{Display, Formatter};

use crate::models::action::{ACTION_ID_KEY, ExchangeAction, ExchangeRequest};

const ACTION_ORDER: &str = "order";
const ACTION_C_DEPOSIT: &str = "cDeposit";
const ACTION_C_WITHDRAW: &str = "cWithdraw";
const ACTION_TOKEN_DELEGATE: &str = "tokenDelegate";
const TOKEN_DELEGATE_STAKE: &str = "stake";
const TOKEN_DELEGATE_UNSTAKE: &str = "unstake";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyperCoreTransactionId {
    Order(u64),
    Action(HyperCoreActionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyperCoreActionId {
    Nonce(u64),
    Order(u64),
    CDeposit { wei: u64, nonce: u64 },
    CWithdraw { wei: u64, nonce: u64 },
    TokenDelegate { wei: u64, is_undelegate: bool, nonce: u64 },
}

impl HyperCoreActionId {
    pub fn nonce(&self) -> u64 {
        match self {
            Self::Nonce(nonce) | Self::Order(nonce) | Self::CDeposit { nonce, .. } | Self::CWithdraw { nonce, .. } | Self::TokenDelegate { nonce, .. } => *nonce,
        }
    }
}

impl From<ExchangeRequest> for HyperCoreActionId {
    fn from(request: ExchangeRequest) -> Self {
        match request.action {
            ExchangeAction::Order => Self::Order(request.nonce),
            ExchangeAction::CDeposit { wei } => Self::CDeposit { wei, nonce: request.nonce },
            ExchangeAction::CWithdraw { wei } => Self::CWithdraw { wei, nonce: request.nonce },
            ExchangeAction::TokenDelegate { wei, is_undelegate } => Self::TokenDelegate {
                wei,
                is_undelegate,
                nonce: request.nonce,
            },
            ExchangeAction::Other => Self::Nonce(request.nonce),
        }
    }
}

impl HyperCoreTransactionId {
    pub fn parse(id: &str) -> Option<Self> {
        match id.split_once(':') {
            Some((ACTION_ID_KEY, rest)) => parse_action_id(rest).map(Self::Action),
            Some((ACTION_ORDER, value)) => value.parse().ok().map(Self::Order),
            _ => id.parse().ok().map(Self::Order),
        }
    }
}

impl Display for HyperCoreTransactionId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Order(value) => write!(formatter, "{ACTION_ORDER}:{value}"),
            Self::Action(action) => write!(formatter, "{ACTION_ID_KEY}:{action}"),
        }
    }
}

impl Display for HyperCoreActionId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nonce(nonce) => write!(formatter, "{nonce}"),
            Self::Order(nonce) => write!(formatter, "{ACTION_ORDER}:{nonce}"),
            Self::CDeposit { wei, nonce } => write!(formatter, "{ACTION_C_DEPOSIT}:{wei}:{nonce}"),
            Self::CWithdraw { wei, nonce } => write!(formatter, "{ACTION_C_WITHDRAW}:{wei}:{nonce}"),
            Self::TokenDelegate { wei, is_undelegate, nonce } => {
                let direction = if *is_undelegate { TOKEN_DELEGATE_UNSTAKE } else { TOKEN_DELEGATE_STAKE };
                write!(formatter, "{ACTION_TOKEN_DELEGATE}:{wei}:{direction}:{nonce}")
            }
        }
    }
}

fn parse_action_id(value: &str) -> Option<HyperCoreActionId> {
    let parts = value.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        [nonce] => nonce.parse().ok().map(HyperCoreActionId::Nonce),
        [ACTION_ORDER, nonce] => Some(HyperCoreActionId::Order(nonce.parse().ok()?)),
        [ACTION_C_DEPOSIT, wei, nonce] => Some(HyperCoreActionId::CDeposit {
            wei: wei.parse().ok()?,
            nonce: nonce.parse().ok()?,
        }),
        [ACTION_C_WITHDRAW, wei, nonce] => Some(HyperCoreActionId::CWithdraw {
            wei: wei.parse().ok()?,
            nonce: nonce.parse().ok()?,
        }),
        [ACTION_TOKEN_DELEGATE, wei, TOKEN_DELEGATE_STAKE, nonce] => Some(HyperCoreActionId::TokenDelegate {
            wei: wei.parse().ok()?,
            is_undelegate: false,
            nonce: nonce.parse().ok()?,
        }),
        [ACTION_TOKEN_DELEGATE, wei, TOKEN_DELEGATE_UNSTAKE, nonce] => Some(HyperCoreActionId::TokenDelegate {
            wei: wei.parse().ok()?,
            is_undelegate: true,
            nonce: nonce.parse().ok()?,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypercore_transaction_id() {
        assert_eq!(HyperCoreTransactionId::parse("order:413978262893"), Some(HyperCoreTransactionId::Order(413978262893)));
        assert_eq!(HyperCoreTransactionId::parse("413978262893"), Some(HyperCoreTransactionId::Order(413978262893)));
        assert_eq!(
            HyperCoreTransactionId::parse("action:1778110454168"),
            Some(HyperCoreTransactionId::Action(HyperCoreActionId::Nonce(1778110454168)))
        );
        assert_eq!(
            HyperCoreTransactionId::parse("action:order:1778110454168"),
            Some(HyperCoreTransactionId::Action(HyperCoreActionId::Order(1778110454168)))
        );
        assert_eq!(
            HyperCoreTransactionId::parse("action:cDeposit:1000000:1778110454168"),
            Some(HyperCoreTransactionId::Action(HyperCoreActionId::CDeposit {
                wei: 1000000,
                nonce: 1778110454168
            }))
        );
        assert_eq!(
            HyperCoreTransactionId::parse("action:tokenDelegate:1000000:unstake:1778110454168"),
            Some(HyperCoreTransactionId::Action(HyperCoreActionId::TokenDelegate {
                wei: 1000000,
                is_undelegate: true,
                nonce: 1778110454168
            }))
        );
        assert_eq!(HyperCoreTransactionId::parse("0xba3b"), None);

        assert_eq!(HyperCoreTransactionId::Order(413978262893).to_string(), "order:413978262893");
        assert_eq!(HyperCoreTransactionId::Action(HyperCoreActionId::Nonce(1778110454168)).to_string(), "action:1778110454168");
        assert_eq!(
            HyperCoreTransactionId::Action(HyperCoreActionId::TokenDelegate {
                wei: 1000000,
                is_undelegate: false,
                nonce: 1778110454168
            })
            .to_string(),
            "action:tokenDelegate:1000000:stake:1778110454168"
        );
        assert_eq!(HyperCoreActionId::Nonce(1778110454168).nonce(), 1778110454168);
    }
}
