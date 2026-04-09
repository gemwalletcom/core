use std::str::FromStr;

use crate::{QuoteRequest, Route, SwapperError, SwapperMode, eth_address, fees::apply_slippage_in_bp};
use alloy_primitives::{Address, U256};
use gem_evm::uniswap::{
    actions::V4Action::{SETTLE, SWAP_EXACT_IN, TAKE},
    command::{ADDRESS_THIS, PayPortion, Permit2Permit, Sweep, Transfer, UniversalRouterCommand},
    contracts::v4::{IV4Router::ExactInputParams, PathKey},
};

pub fn build_commands(
    request: &QuoteRequest,
    token_in: &Address,
    token_out: &Address,
    amount_in: u128,
    quote_amount: u128,
    swap_routes: &[Route],
    permit: Option<Permit2Permit>,
    fee_token_is_input: bool,
    input_is_native: bool,
) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
    let options = request.options.clone();
    let fee_options = options.fee.unwrap_or_default().evm;
    let recipient = eth_address::parse_str(&request.wallet_address)?;

    let mode = request.mode;
    let pay_fees = fee_options.bps > 0;

    let mut commands: Vec<UniversalRouterCommand> = vec![];

    match mode {
        SwapperMode::ExactIn => {
            let amount_out = apply_slippage_in_bp(&quote_amount, options.slippage.bps + fee_options.bps);
            // Insert permit2 if needed
            if let Some(permit) = permit {
                commands.push(UniversalRouterCommand::PERMIT2_PERMIT(permit));
            }

            if pay_fees {
                if fee_token_is_input {
                    // insert TRANSFER fee first
                    let fee = amount_in * (fee_options.bps as u128) / 10000_u128;
                    let fee_recipient = Address::from_str(fee_options.address.as_str()).unwrap();
                    if input_is_native {
                        // if input is native ETH, we can transfer directly
                        commands.push(UniversalRouterCommand::TRANSFER(Transfer {
                            token: *token_in,
                            recipient: fee_recipient,
                            value: U256::from(fee),
                        }));
                    } else {
                        // call permit2 transfer instead
                        commands.push(UniversalRouterCommand::PERMIT2_TRANSFER_FROM(Transfer {
                            token: *token_in,
                            recipient: fee_recipient,
                            value: U256::from(fee),
                        }));
                    };
                    // insert V4_SWAP with amount - fee
                    // fee charged in token_in, so we need to use recipient as recipient
                    let command = build_v4_swap_command(token_in, token_out, amount_in - fee, amount_out, swap_routes, &recipient)?;
                    commands.push(command);
                } else {
                    // insert V4 SWAP
                    // if needs to pay fees, amount_out_min set to 0 and we will sweep the rest
                    let address_this = ADDRESS_THIS.parse().unwrap();
                    let amount_out_min = if pay_fees { 0 } else { amount_out };
                    let command = build_v4_swap_command(token_in, token_out, amount_in, amount_out_min, swap_routes, &address_this)?;
                    commands.push(command);

                    // insert PAY_PORTION to fee_address
                    commands.push(UniversalRouterCommand::PAY_PORTION(PayPortion {
                        token: *token_out,
                        recipient: Address::from_str(fee_options.address.as_str()).unwrap(),
                        bips: U256::from(fee_options.bps),
                    }));

                    commands.push(UniversalRouterCommand::SWEEP(Sweep {
                        token: *token_out,
                        recipient,
                        amount_min: U256::from(amount_out),
                    }));
                }
            } else {
                let command = build_v4_swap_command(token_in, token_out, amount_in, amount_out, swap_routes, &recipient)?;
                commands.push(command);
            }
        }
        SwapperMode::ExactOut => {
            todo!("swap exact out not implemented");
        }
    }
    Ok(commands)
}

fn build_v4_swap_command(
    token_in: &Address,
    token_out: &Address,
    amount_in: u128,
    amount_out_min: u128,
    swap_routes: &[Route],
    recipient: &Address,
) -> Result<UniversalRouterCommand, SwapperError> {
    if swap_routes.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
    // V4_SWAP {actions}
    // Dispatcher -> BaseActionsRouter::_executeActions -> PoolManager::_executeActionsWithoutUnlock -> V4Router::_handleAction
    let path: Vec<PathKey> = swap_routes
        .iter()
        .map(|route| PathKey::try_from(route).map_err(|_| SwapperError::InvalidRoute))
        .collect::<Result<Vec<PathKey>, SwapperError>>()?;
    let actions = vec![
        SWAP_EXACT_IN(ExactInputParams {
            currencyIn: *token_in,
            path,
            amountIn: amount_in,
            amountOutMinimum: amount_out_min,
        }),
        SETTLE {
            currency: *token_in,
            amount: U256::from(0),
            payer_is_user: true,
        },
        TAKE {
            currency: *token_out,
            recipient: recipient.to_owned(),
            amount: U256::from(0),
        },
    ];
    Ok(UniversalRouterCommand::V4_SWAP { actions })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fees::{ReferralFee, ReferralFees},
        models::Options,
    };
    use primitives::{
        AssetId, Chain,
        asset_constants::{CELO_USDT_TOKEN_ID, CELO_WETH_TOKEN_ID},
    };

    #[test]
    fn test_build_commands_celo_tokenized_native() {
        let token_celo = Address::from_str(CELO_WETH_TOKEN_ID).unwrap();
        let token_usdt = Address::from_str(CELO_USDT_TOKEN_ID).unwrap();
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let routes = vec![Route::mock(
            AssetId::from(Chain::Celo, Some(CELO_WETH_TOKEN_ID.into())),
            AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())),
        )];

        // CELO -> USDT: no wrap, direct swap through token path
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, None).into(),
            to_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: "22000000000000000000".into(),
            mode: SwapperMode::ExactIn,
            options: Options::default(),
        };
        let commands = build_commands(&request, &token_celo, &token_usdt, 22_000_000_000_000_000_000, 14_804_757, &routes, None, false, false).unwrap();

        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], UniversalRouterCommand::V4_SWAP { .. }));

        // USDT -> CELO with fees: sweep instead of unwrap
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Celo, None).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: "900000".into(),
            mode: SwapperMode::ExactIn,
            options: Options {
                slippage: 50.into(),
                fee: Some(ReferralFees::evm(ReferralFee { bps: 50, address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into() })),
                preferred_providers: vec![],
                use_max_amount: false,
            },
        };
        let routes = vec![Route::mock(
            AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())),
            AssetId::from(Chain::Celo, Some(CELO_WETH_TOKEN_ID.into())),
        )];
        let commands = build_commands(&request, &token_usdt, &token_celo, 900_000, 10_752_991_111_111_111_170, &routes, None, false, false).unwrap();

        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0], UniversalRouterCommand::V4_SWAP { .. }));
        assert!(matches!(commands[1], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::SWEEP(_)));
    }
}
