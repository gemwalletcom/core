use crate::{SuiError, sui_framework_package_address, tx_builder::move_call};
use sui_transaction_builder::{Argument, TransactionBuilder};

const MODULE_COIN: &str = "coin";
const MODULE_BALANCE: &str = "balance";
const FUNCTION_INTO_BALANCE: &str = "into_balance";
const FUNCTION_FROM_BALANCE: &str = "from_balance";
const FUNCTION_BALANCE_ZERO: &str = "zero";
const FUNCTION_BALANCE_VALUE: &str = "value";
const FUNCTION_BALANCE_DESTROY_ZERO: &str = "destroy_zero";

pub fn into_balance(txb: &mut TransactionBuilder, coin_type: &str, coin: Argument) -> Result<Argument, SuiError> {
    move_call(txb, sui_framework_package_address(), MODULE_COIN, FUNCTION_INTO_BALANCE, &[coin_type], vec![coin])
}

pub fn from_balance(txb: &mut TransactionBuilder, coin_type: &str, balance: Argument) -> Result<Argument, SuiError> {
    move_call(txb, sui_framework_package_address(), MODULE_COIN, FUNCTION_FROM_BALANCE, &[coin_type], vec![balance])
}

pub fn balance_zero(txb: &mut TransactionBuilder, coin_type: &str) -> Result<Argument, SuiError> {
    move_call(txb, sui_framework_package_address(), MODULE_BALANCE, FUNCTION_BALANCE_ZERO, &[coin_type], vec![])
}

pub fn balance_value(txb: &mut TransactionBuilder, coin_type: &str, balance: Argument) -> Result<Argument, SuiError> {
    move_call(txb, sui_framework_package_address(), MODULE_BALANCE, FUNCTION_BALANCE_VALUE, &[coin_type], vec![balance])
}

pub fn destroy_zero_balance(txb: &mut TransactionBuilder, coin_type: &str, balance: Argument) -> Result<(), SuiError> {
    move_call(
        txb,
        sui_framework_package_address(),
        MODULE_BALANCE,
        FUNCTION_BALANCE_DESTROY_ZERO,
        &[coin_type],
        vec![balance],
    )?;
    Ok(())
}
