use std::str::FromStr;

use crate::swapper::{eth_address, slippage::apply_slippage_in_bp, GemSwapMode, SwapQuoteRequest, SwapRoute, SwapperError};
use alloy_primitives::{Address, U256};
use gem_evm::uniswap::{
    actions::V4Action::{SETTLE, SWAP_EXACT_IN, TAKE},
    command::{PayPortion, Permit2Permit, Sweep, Transfer, UniversalRouterCommand, ADDRESS_THIS},
    contracts::v4::{IV4Router::ExactInputParams, PathKey},
};

pub fn build_commands(
    request: &SwapQuoteRequest,
    token_in: &Address,
    token_out: &Address,
    amount_in: u128,
    quote_amount: u128,
    swap_routes: &[SwapRoute],
    permit: Option<Permit2Permit>,
    fee_token_is_input: bool,
) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
    let options = request.options.clone();
    let fee_options = options.fee.unwrap_or_default().evm;
    let recipient = eth_address::parse_str(&request.wallet_address)?;

    let mode = request.mode.clone();
    let pay_fees = fee_options.bps > 0;

    let input_is_native = request.from_asset.is_native();
    let mut commands: Vec<UniversalRouterCommand> = vec![];

    match mode {
        GemSwapMode::ExactIn => {
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
        GemSwapMode::ExactOut => {
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
    swap_routes: &[SwapRoute],
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
