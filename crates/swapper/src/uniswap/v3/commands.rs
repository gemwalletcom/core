use crate::{SwapperError, eth_address, models::*, uniswap::requires_native_wrapping};
use gem_evm::uniswap::command::{ADDRESS_THIS, PayPortion, Permit2Permit, Sweep, Transfer, UniversalRouterCommand, UnwrapWeth, V3SwapExactIn, WrapEth};

use alloy_primitives::{Address, Bytes, U256};
use std::str::FromStr;

pub fn build_commands(
    request: &QuoteRequest,
    token_in: &Address,
    token_out: &Address,
    amount_in: U256,
    amount_out_min: U256,
    path: &Bytes,
    permit: Option<Permit2Permit>,
    fee_token_is_input: bool,
) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
    let fee_options = request.options.fee.as_ref().map(|fees| &fees.evm).filter(|fee| fee.bps > 0);
    let recipient = eth_address::parse_str(&request.wallet_address)?;
    let address_this = Address::from_str(ADDRESS_THIS).unwrap();

    let wrap_input_eth = requires_native_wrapping(&request.from_asset.asset_id());
    let unwrap_output_weth = requires_native_wrapping(&request.to_asset.asset_id());

    let setup_command = if wrap_input_eth {
        // Wrap ETH, recipient is this_address
        Some(UniversalRouterCommand::WRAP_ETH(WrapEth {
            recipient: address_this,
            amount_min: amount_in,
        }))
    } else {
        permit.map(UniversalRouterCommand::PERMIT2_PERMIT)
    };

    // payer_is_user: is true when swapping tokens
    let payer_is_user = !wrap_input_eth;
    let swap_recipient = if unwrap_output_weth { address_this } else { recipient };
    let swap_commands = if let Some(fee_options) = fee_options {
        let fee_recipient = Address::from_str(fee_options.address.as_str()).unwrap();
        if fee_token_is_input {
            // insert TRANSFER fee first
            let fee = amount_in * U256::from(fee_options.bps) / U256::from(10000);
            let fee_command = if wrap_input_eth {
                // if input is native ETH, we can transfer directly because of WRAP_ETH command
                UniversalRouterCommand::TRANSFER(Transfer {
                    token: *token_in,
                    recipient: fee_recipient,
                    value: fee,
                })
            } else {
                // call permit2 transfer instead
                UniversalRouterCommand::PERMIT2_TRANSFER_FROM(Transfer {
                    token: *token_in,
                    recipient: fee_recipient,
                    value: fee,
                })
            };

            vec![
                fee_command,
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: swap_recipient,
                    amount_in: amount_in - fee,
                    amount_out_min,
                    path: path.clone(),
                    payer_is_user,
                }),
            ]
        } else {
            // insert V3_SWAP_EXACT_IN
            // amount_out_min: if needs to pay fees, amount_out_min set to 0 and we will sweep the rest
            let swap_command = UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                recipient: address_this,
                amount_in,
                amount_out_min: U256::from(0),
                path: path.clone(),
                payer_is_user,
            });

            // insert PAY_PORTION to fee_address
            let fee_command = UniversalRouterCommand::PAY_PORTION(PayPortion {
                token: *token_out,
                recipient: fee_recipient,
                bips: U256::from(fee_options.bps),
            });

            if unwrap_output_weth {
                vec![swap_command, fee_command]
            } else {
                // MSG_SENDER should be the address of the caller
                vec![
                    swap_command,
                    fee_command,
                    UniversalRouterCommand::SWEEP(Sweep {
                        token: *token_out,
                        recipient,
                        amount_min: amount_out_min,
                    }),
                ]
            }
        }
    } else {
        // insert V3_SWAP_EXACT_IN
        vec![UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
            recipient: swap_recipient,
            amount_in,
            amount_out_min,
            path: path.clone(),
            payer_is_user,
        })]
    };

    let unwrap_command = if unwrap_output_weth {
        // insert UNWRAP_WETH
        Some(UniversalRouterCommand::UNWRAP_WETH(UnwrapWeth {
            recipient,
            amount_min: amount_out_min,
        }))
    } else {
        None
    };

    let commands = setup_command.into_iter().chain(swap_commands).chain(unwrap_command).collect();
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fees::{ReferralFee, ReferralFees},
        permit2_data::*,
    };
    use alloy_primitives::aliases::U256;
    use gem_evm::uniswap::{FeeTier, path::build_direct_pair};
    use primitives::{
        AssetId, Chain,
        asset_constants::{CELO_USDT_TOKEN_ID, CELO_WETH_TOKEN_ID},
        asset_constants::{ETHEREUM_USDC_TOKEN_ID, ETHEREUM_WETH_TOKEN_ID, OPTIMISM_USDC_E_TOKEN_ID, OPTIMISM_USDC_TOKEN_ID, OPTIMISM_USDT_TOKEN_ID, OPTIMISM_WETH_TOKEN_ID},
        contract_constants::OPTIMISM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT,
    };

    #[test]
    fn test_build_commands_eth_to_token() {
        let request = QuoteRequest {
            // ETH -> USDC
            from_asset: AssetId::from(Chain::Ethereum, None).into(),
            to_asset: AssetId::from(Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "10000000000000000".into(),
            options: Options::default(),
        };

        let token_in = eth_address::parse_str(ETHEREUM_WETH_TOKEN_ID).unwrap();
        let token_out = eth_address::parse_str(ETHEREUM_USDC_TOKEN_ID).unwrap();
        let amount_in = U256::from(1000000000000000u64);

        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred);
        // without fee
        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, U256::from(0), &path, None, false).unwrap();

        assert_eq!(commands.len(), 2);

        assert!(matches!(commands[0], UniversalRouterCommand::WRAP_ETH(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));

        let request = QuoteRequest {
            options: Options {
                slippage: 100.into(),
                fee: Some(ReferralFees::evm(ReferralFee {
                    bps: 25,
                    address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
                })),
                use_max_amount: false,
            },
            ..request
        };

        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, U256::from(0), &path, None, false).unwrap();

        assert_eq!(commands.len(), 4);

        assert!(matches!(commands[0], UniversalRouterCommand::WRAP_ETH(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[3], UniversalRouterCommand::SWEEP(_)));
    }

    #[test]
    fn test_build_commands_usdc_to_usdt() {
        let request = QuoteRequest {
            // USDC -> USDT
            from_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDC_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDT_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "6500000".into(),
            options: Options::default(),
        };

        let token_in = eth_address::parse_str(request.from_asset.asset_id().token_id.as_ref().unwrap()).unwrap();
        let token_out = eth_address::parse_str(request.to_asset.asset_id().token_id.as_ref().unwrap()).unwrap();
        let amount_in = U256::from_str(&request.value).unwrap();

        let permit2_data = Permit2Data {
            permit_single: PermitSingle {
                details: Permit2Detail {
                    token: OPTIMISM_USDC_TOKEN_ID.into(),
                    amount: "1461501637330902918203684832716283019655932542975".into(),
                    expiration: 1732667593,
                    nonce: 0,
                },
                spender: OPTIMISM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT.into(),
                sig_deadline: 1730077393,
            },
            signature: hex::decode("8f32d2e66506a4f424b1b23309ed75d338534d0912129a8aa3381fab4eb8032f160e0988f10f512b19a58c2a689416366c61cc0c483c3b5322dc91f8b60107671b").unwrap(),
        };

        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred);
        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, U256::from(6507936), &path, Some(permit2_data.into()), false).unwrap();

        assert_eq!(commands.len(), 2);

        assert!(matches!(commands[0], UniversalRouterCommand::PERMIT2_PERMIT(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
    }

    #[test]
    fn test_build_commands_uses_min_amount_without_reapplying_slippage() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDC_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDT_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "6500000".into(),
            options: Options::new_with_slippage(100.into()),
        };
        let token_in = eth_address::parse_str(request.from_asset.asset_id().token_id.as_ref().unwrap()).unwrap();
        let token_out = eth_address::parse_str(request.to_asset.asset_id().token_id.as_ref().unwrap()).unwrap();
        let amount_in = U256::from_str(&request.value).unwrap();
        let amount_out_min = U256::from(6_500_000_u64);
        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred);

        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, amount_out_min, &path, None, false).unwrap();

        match &commands[0] {
            UniversalRouterCommand::V3_SWAP_EXACT_IN(swap) => assert_eq!(swap.amount_out_min, amount_out_min),
            _ => panic!("expected V3_SWAP_EXACT_IN"),
        }
    }

    #[test]
    fn test_build_commands_usdc_to_aave() {
        let request = QuoteRequest {
            // USDC -> AAVE
            from_asset: AssetId::from(Chain::Optimism, Some("0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85".into())).into(),
            to_asset: AssetId::from(Chain::Optimism, Some("0x76fb31fb4af56892a25e32cfc43de717950c9278".into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "5064985".into(),
            options: Options {
                slippage: 100.into(),
                fee: Some(ReferralFees::evm(ReferralFee {
                    bps: 25,
                    address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
                })),
                use_max_amount: false,
            },
        };

        let token_in = eth_address::parse_str(request.from_asset.asset_id().token_id.as_ref().unwrap()).unwrap();
        let token_out = eth_address::parse_str(request.to_asset.asset_id().token_id.as_ref().unwrap()).unwrap();
        let amount_in = U256::from_str(&request.value).unwrap();

        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred);
        // fee token is output token
        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, U256::from(33377662359182269u64), &path, None, false).unwrap();

        assert_eq!(commands.len(), 3);

        assert!(matches!(commands[0], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::SWEEP(_)));

        // fee token is input token
        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, U256::from(33377662359182269u64), &path, None, true).unwrap();

        assert_eq!(commands.len(), 2);

        assert!(matches!(commands[0], UniversalRouterCommand::PERMIT2_TRANSFER_FROM(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
    }

    #[test]
    fn test_build_commands_usdce_to_eth() {
        let request = QuoteRequest {
            // USDCE -> ETH
            from_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDC_E_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Optimism, None).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "10000000".into(),
            options: Options {
                slippage: 100.into(),
                fee: Some(ReferralFees::evm(ReferralFee {
                    bps: 25,
                    address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
                })),
                use_max_amount: false,
            },
        };

        let token_in = eth_address::parse_str(request.from_asset.asset_id().token_id.as_ref().unwrap()).unwrap();
        let token_out = eth_address::parse_str(OPTIMISM_WETH_TOKEN_ID).unwrap();
        let amount_in = U256::from_str(&request.value).unwrap();

        let permit2_data = Permit2Data {
            permit_single: PermitSingle {
                details: Permit2Detail {
                    token: request.from_asset.asset_id().token_id.clone().unwrap(),
                    amount: "1461501637330902918203684832716283019655932542975".into(),
                    expiration: 1732667502,
                    nonce: 0,
                },
                spender: OPTIMISM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT.into(),
                sig_deadline: 1730077302,
            },
            signature: hex::decode("00e96ed0f5bf5cca62dc9d9753960d83c8be83224456559a1e93a66d972a019f6f328a470f8257d3950b4cb7cd0024d789b4fcd9e80c4eb43d82a38d9e5332f31b").unwrap(),
        };

        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred);
        let commands = super::build_commands(
            &request,
            &token_in,
            &token_out,
            amount_in,
            U256::from(3997001989341576u64),
            &path,
            Some(permit2_data.into()),
            false,
        )
        .unwrap();

        assert_eq!(commands.len(), 4);

        assert!(matches!(commands[0], UniversalRouterCommand::PERMIT2_PERMIT(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[3], UniversalRouterCommand::UNWRAP_WETH(_)));
    }

    #[test]
    fn test_build_commands_routes_weth_output_to_router_before_unwrap() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDC_E_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Optimism, None).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "10000000".into(),
            options: Options::default(),
        };
        let token_in = eth_address::parse_str(request.from_asset.asset_id().token_id.as_ref().unwrap()).unwrap();
        let token_out = eth_address::parse_str(OPTIMISM_WETH_TOKEN_ID).unwrap();
        let amount_in = U256::from_str(&request.value).unwrap();
        let amount_out_min = U256::from(3997001989341576u64);
        let address_this = Address::from_str(ADDRESS_THIS).unwrap();
        let path = build_direct_pair(&token_in, &token_out, FeeTier::FiveHundred);

        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, amount_out_min, &path, None, false).unwrap();

        assert_eq!(commands.len(), 2);
        match &commands[0] {
            UniversalRouterCommand::V3_SWAP_EXACT_IN(swap) => assert_eq!(swap.recipient, address_this),
            _ => panic!("expected V3_SWAP_EXACT_IN"),
        }
        match &commands[1] {
            UniversalRouterCommand::UNWRAP_WETH(_) => {}
            _ => panic!("expected UNWRAP_WETH"),
        }

        let request = QuoteRequest {
            options: Options {
                slippage: 100.into(),
                fee: Some(ReferralFees::evm(ReferralFee {
                    bps: 25,
                    address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
                })),
                use_max_amount: false,
            },
            ..request
        };
        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, amount_out_min, &path, None, true).unwrap();

        assert_eq!(commands.len(), 3);
        match &commands[0] {
            UniversalRouterCommand::PERMIT2_TRANSFER_FROM(_) => {}
            _ => panic!("expected PERMIT2_TRANSFER_FROM"),
        }
        match &commands[1] {
            UniversalRouterCommand::V3_SWAP_EXACT_IN(swap) => assert_eq!(swap.recipient, address_this),
            _ => panic!("expected V3_SWAP_EXACT_IN"),
        }
        match &commands[2] {
            UniversalRouterCommand::UNWRAP_WETH(_) => {}
            _ => panic!("expected UNWRAP_WETH"),
        }
    }

    #[test]
    fn test_build_commands_eth_to_uni_with_input_fee() {
        // Replicate https://optimistic.etherscan.io/tx/0x18277deea3e273a7fb9abc985269dcdabe3d34c2b604fbd82dcd0a5a5204f72c
        let request = QuoteRequest {
            // ETH -> UNI
            from_asset: AssetId::from(Chain::Optimism, None).into(),
            to_asset: AssetId::from(Chain::Optimism, Some("0x6fd9d7ad17242c41f7131d257212c54a0e816691".into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "1000000000000000".into(),
            options: Options {
                slippage: 100.into(),
                fee: Some(ReferralFees::evm(ReferralFee {
                    bps: 25,
                    address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
                })),
                use_max_amount: false,
            },
        };

        let token_in = eth_address::parse_str(OPTIMISM_WETH_TOKEN_ID).unwrap();
        let token_out = eth_address::parse_str(&request.to_asset.asset_id().token_id.unwrap()).unwrap();
        let amount_in = U256::from_str(request.value.as_str()).unwrap();

        let path = build_direct_pair(&token_in, &token_out, FeeTier::ThreeThousand);
        let commands = super::build_commands(&request, &token_in, &token_out, amount_in, U256::from(244440440678888410_u64), &path, None, true).unwrap();

        assert_eq!(commands.len(), 3);

        assert!(matches!(commands[0], UniversalRouterCommand::WRAP_ETH(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::TRANSFER(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
    }

    #[test]
    fn test_build_commands_celo_tokenized_native() {
        let token_celo = eth_address::parse_str(CELO_WETH_TOKEN_ID).unwrap();
        let token_usdt = eth_address::parse_str(CELO_USDT_TOKEN_ID).unwrap();
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

        // CELO -> USDT: no wrap, just a direct swap through token path
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, None).into(),
            to_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: "22000000000000000000".into(),
            options: Options::default(),
        };
        let path = build_direct_pair(&token_celo, &token_usdt, FeeTier::Hundred);
        let commands = super::build_commands(
            &request,
            &token_celo,
            &token_usdt,
            U256::from_str(&request.value).unwrap(),
            U256::from(14804757u64),
            &path,
            None,
            false,
        )
        .unwrap();

        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));

        // USDT -> CELO with fees: sweep instead of unwrap
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Celo, None).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: "900000".into(),
            options: Options {
                slippage: 50.into(),
                fee: Some(ReferralFees::evm(ReferralFee {
                    bps: 50,
                    address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                })),
                use_max_amount: false,
            },
        };
        let path = build_direct_pair(&token_usdt, &token_celo, FeeTier::Hundred);
        let commands = super::build_commands(
            &request,
            &token_usdt,
            &token_celo,
            U256::from_str(&request.value).unwrap(),
            U256::from(10752991111111111170u128),
            &path,
            None,
            false,
        )
        .unwrap();

        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0], UniversalRouterCommand::V3_SWAP_EXACT_IN(_)));
        assert!(matches!(commands[1], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::SWEEP(_)));
    }
}
