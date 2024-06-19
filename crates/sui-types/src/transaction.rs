// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress};
use crate::error::{fp_ensure, UserInputError, UserInputResult};
use crate::governance::EpochId;
use crate::programmable_transaction_builder::ProgrammableTransactionBuilder;
use crate::{
    SUI_AUTHENTICATOR_STATE_OBJECT_ID, SUI_AUTHENTICATOR_STATE_OBJECT_SHARED_VERSION,
    SUI_CLOCK_OBJECT_ID, SUI_CLOCK_OBJECT_SHARED_VERSION, SUI_RANDOMNESS_STATE_OBJECT_ID,
    SUI_SYSTEM_STATE_OBJECT_ID, SUI_SYSTEM_STATE_OBJECT_SHARED_VERSION,
};
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::TypeTag,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::iter::once;
use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
};
use strum::IntoStaticStr;
use sui_protocol_config::ProtocolConfig;

const BLOCKED_MOVE_FUNCTIONS: [(ObjectID, &str, &str); 0] = [];

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum CallArg {
    // contains no structs or objects
    Pure(Vec<u8>),
    // an object
    Object(ObjectArg),
}

impl CallArg {
    pub const SUI_SYSTEM_MUT: Self = Self::Object(ObjectArg::SUI_SYSTEM_MUT);
    pub const CLOCK_IMM: Self = Self::Object(ObjectArg::SharedObject {
        id: SUI_CLOCK_OBJECT_ID,
        initial_shared_version: SUI_CLOCK_OBJECT_SHARED_VERSION,
        mutable: false,
    });
    pub const CLOCK_MUT: Self = Self::Object(ObjectArg::SharedObject {
        id: SUI_CLOCK_OBJECT_ID,
        initial_shared_version: SUI_CLOCK_OBJECT_SHARED_VERSION,
        mutable: true,
    });
    pub const AUTHENTICATOR_MUT: Self = Self::Object(ObjectArg::SharedObject {
        id: SUI_AUTHENTICATOR_STATE_OBJECT_ID,
        initial_shared_version: SUI_AUTHENTICATOR_STATE_OBJECT_SHARED_VERSION,
        mutable: true,
    });
}

impl CallArg {
    fn input_objects(&self) -> Vec<InputObjectKind> {
        match self {
            CallArg::Pure(_) => vec![],
            CallArg::Object(ObjectArg::ImmOrOwnedObject(object_ref)) => {
                vec![InputObjectKind::ImmOrOwnedMoveObject(*object_ref)]
            }
            CallArg::Object(ObjectArg::SharedObject {
                id,
                initial_shared_version,
                mutable,
            }) => {
                let id = *id;
                let initial_shared_version = *initial_shared_version;
                let mutable = *mutable;
                vec![InputObjectKind::SharedMoveObject {
                    id,
                    initial_shared_version,
                    mutable,
                }]
            }
            // Receiving objects are not part of the input objects.
            CallArg::Object(ObjectArg::Receiving(_)) => vec![],
        }
    }

    fn receiving_objects(&self) -> Vec<ObjectRef> {
        match self {
            CallArg::Pure(_) => vec![],
            CallArg::Object(o) => match o {
                ObjectArg::ImmOrOwnedObject(_) => vec![],
                ObjectArg::SharedObject { .. } => vec![],
                ObjectArg::Receiving(obj_ref) => vec![*obj_ref],
            },
        }
    }

    pub fn validity_check(&self, config: &ProtocolConfig) -> UserInputResult {
        match self {
            CallArg::Pure(p) => {
                fp_ensure!(
                    p.len() < config.max_pure_argument_size() as usize,
                    UserInputError::SizeLimitExceeded {
                        limit: "maximum pure argument size".to_string(),
                        value: config.max_pure_argument_size().to_string()
                    }
                );
            }
            CallArg::Object(_) => (),
        }
        Ok(())
    }
}

impl From<bool> for CallArg {
    fn from(b: bool) -> Self {
        // unwrap safe because every u8 value is BCS-serializable
        CallArg::Pure(bcs::to_bytes(&b).unwrap())
    }
}

impl From<u8> for CallArg {
    fn from(n: u8) -> Self {
        // unwrap safe because every u8 value is BCS-serializable
        CallArg::Pure(bcs::to_bytes(&n).unwrap())
    }
}

impl From<u16> for CallArg {
    fn from(n: u16) -> Self {
        // unwrap safe because every u16 value is BCS-serializable
        CallArg::Pure(bcs::to_bytes(&n).unwrap())
    }
}

impl From<u32> for CallArg {
    fn from(n: u32) -> Self {
        // unwrap safe because every u32 value is BCS-serializable
        CallArg::Pure(bcs::to_bytes(&n).unwrap())
    }
}

impl From<u64> for CallArg {
    fn from(n: u64) -> Self {
        // unwrap safe because every u64 value is BCS-serializable
        CallArg::Pure(bcs::to_bytes(&n).unwrap())
    }
}

impl From<u128> for CallArg {
    fn from(n: u128) -> Self {
        // unwrap safe because every u128 value is BCS-serializable
        CallArg::Pure(bcs::to_bytes(&n).unwrap())
    }
}

impl From<&Vec<u8>> for CallArg {
    fn from(v: &Vec<u8>) -> Self {
        // unwrap safe because every vec<u8> value is BCS-serializable
        CallArg::Pure(bcs::to_bytes(v).unwrap())
    }
}

impl From<ObjectRef> for CallArg {
    fn from(obj: ObjectRef) -> Self {
        CallArg::Object(ObjectArg::ImmOrOwnedObject(obj))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ObjectArg {
    // A Move object, either immutable, or owned mutable.
    ImmOrOwnedObject(ObjectRef),
    // A Move object that's shared.
    // SharedObject::mutable controls whether caller asks for a mutable reference to shared object.
    SharedObject {
        id: ObjectID,
        initial_shared_version: SequenceNumber,
        mutable: bool,
    },
    // A Move object that can be received in this transaction.
    Receiving(ObjectRef),
}

fn type_tag_validity_check(
    tag: &TypeTag,
    config: &ProtocolConfig,
    starting_count: &mut usize,
) -> UserInputResult<()> {
    let mut stack = vec![(tag, 1)];
    while let Some((tag, depth)) = stack.pop() {
        *starting_count += 1;
        fp_ensure!(
            *starting_count < config.max_type_arguments() as usize,
            UserInputError::SizeLimitExceeded {
                limit: "maximum type arguments in a call transaction".to_string(),
                value: config.max_type_arguments().to_string()
            }
        );
        fp_ensure!(
            depth < config.max_type_argument_depth(),
            UserInputError::SizeLimitExceeded {
                limit: "maximum type argument depth in a call transaction".to_string(),
                value: config.max_type_argument_depth().to_string()
            }
        );
        match tag {
            TypeTag::Bool
            | TypeTag::U8
            | TypeTag::U64
            | TypeTag::U128
            | TypeTag::Address
            | TypeTag::Signer
            | TypeTag::U16
            | TypeTag::U32
            | TypeTag::U256 => (),
            TypeTag::Vector(t) => {
                stack.push((t, depth + 1));
            }
            TypeTag::Struct(s) => {
                let next_depth = depth + 1;
                stack.extend(s.type_params.iter().map(|t| (t, next_depth)));
            }
        }
    }
    Ok(())
}

impl ObjectArg {
    pub const SUI_SYSTEM_MUT: Self = Self::SharedObject {
        id: SUI_SYSTEM_STATE_OBJECT_ID,
        initial_shared_version: SUI_SYSTEM_STATE_OBJECT_SHARED_VERSION,
        mutable: true,
    };

    pub fn id(&self) -> ObjectID {
        match self {
            ObjectArg::Receiving((id, _, _))
            | ObjectArg::ImmOrOwnedObject((id, _, _))
            | ObjectArg::SharedObject { id, .. } => *id,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct GasData {
    pub payment: Vec<ObjectRef>,
    pub owner: SuiAddress,
    pub price: u64,
    pub budget: u64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum TransactionExpiration {
    /// The transaction has no expiration
    None,
    /// Validators wont sign a transaction unless the expiration Epoch
    /// is greater than or equal to the current epoch
    Epoch(EpochId),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    V1(TransactionDataV1),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct TransactionDataV1 {
    pub kind: TransactionKind,
    pub sender: SuiAddress,
    pub gas_data: GasData,
    pub expiration: TransactionExpiration,
}

impl TransactionData {
    pub fn new(
        kind: TransactionKind,
        sender: SuiAddress,
        gas_payment: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        TransactionData::V1(TransactionDataV1 {
            kind,
            sender,
            gas_data: GasData {
                price: gas_price,
                owner: sender,
                payment: vec![gas_payment],
                budget: gas_budget,
            },
            expiration: TransactionExpiration::None,
        })
    }

    pub fn new_with_gas_coins(
        kind: TransactionKind,
        sender: SuiAddress,
        gas_payment: Vec<ObjectRef>,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        Self::new_with_gas_coins_allow_sponsor(
            kind,
            sender,
            gas_payment,
            gas_budget,
            gas_price,
            sender,
        )
    }

    pub fn new_with_gas_coins_allow_sponsor(
        kind: TransactionKind,
        sender: SuiAddress,
        gas_payment: Vec<ObjectRef>,
        gas_budget: u64,
        gas_price: u64,
        gas_sponsor: SuiAddress,
    ) -> Self {
        TransactionData::V1(TransactionDataV1 {
            kind,
            sender,
            gas_data: GasData {
                price: gas_price,
                owner: gas_sponsor,
                payment: gas_payment,
                budget: gas_budget,
            },
            expiration: TransactionExpiration::None,
        })
    }

    pub fn new_with_gas_data(kind: TransactionKind, sender: SuiAddress, gas_data: GasData) -> Self {
        TransactionData::V1(TransactionDataV1 {
            kind,
            sender,
            gas_data,
            expiration: TransactionExpiration::None,
        })
    }

    pub fn new_move_call(
        sender: SuiAddress,
        package: ObjectID,
        module: Identifier,
        function: Identifier,
        type_arguments: Vec<TypeTag>,
        gas_payment: ObjectRef,
        arguments: Vec<CallArg>,
        gas_budget: u64,
        gas_price: u64,
    ) -> anyhow::Result<Self> {
        Self::new_move_call_with_gas_coins(
            sender,
            package,
            module,
            function,
            type_arguments,
            vec![gas_payment],
            arguments,
            gas_budget,
            gas_price,
        )
    }

    pub fn new_move_call_with_gas_coins(
        sender: SuiAddress,
        package: ObjectID,
        module: Identifier,
        function: Identifier,
        type_arguments: Vec<TypeTag>,
        gas_payment: Vec<ObjectRef>,
        arguments: Vec<CallArg>,
        gas_budget: u64,
        gas_price: u64,
    ) -> anyhow::Result<Self> {
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            builder.move_call(package, module, function, type_arguments, arguments)?;
            builder.finish()
        };
        Ok(Self::new_programmable(
            sender,
            gas_payment,
            pt,
            gas_budget,
            gas_price,
        ))
    }

    pub fn new_transfer(
        recipient: SuiAddress,
        object_ref: ObjectRef,
        sender: SuiAddress,
        gas_payment: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            builder.transfer_object(recipient, object_ref).unwrap();
            builder.finish()
        };
        Self::new_programmable(sender, vec![gas_payment], pt, gas_budget, gas_price)
    }

    pub fn new_transfer_sui(
        recipient: SuiAddress,
        sender: SuiAddress,
        amount: Option<u64>,
        gas_payment: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        Self::new_transfer_sui_allow_sponsor(
            recipient,
            sender,
            amount,
            gas_payment,
            gas_budget,
            gas_price,
            sender,
        )
    }

    pub fn new_transfer_sui_allow_sponsor(
        recipient: SuiAddress,
        sender: SuiAddress,
        amount: Option<u64>,
        gas_payment: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
        gas_sponsor: SuiAddress,
    ) -> Self {
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            builder.transfer_sui(recipient, amount);
            builder.finish()
        };
        Self::new_programmable_allow_sponsor(
            sender,
            vec![gas_payment],
            pt,
            gas_budget,
            gas_price,
            gas_sponsor,
        )
    }

    pub fn new_pay(
        sender: SuiAddress,
        coins: Vec<ObjectRef>,
        recipients: Vec<SuiAddress>,
        amounts: Vec<u64>,
        gas_payment: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
    ) -> anyhow::Result<Self> {
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            builder.pay(coins, recipients, amounts)?;
            builder.finish()
        };
        Ok(Self::new_programmable(
            sender,
            vec![gas_payment],
            pt,
            gas_budget,
            gas_price,
        ))
    }

    pub fn new_pay_sui(
        sender: SuiAddress,
        mut coins: Vec<ObjectRef>,
        recipients: Vec<SuiAddress>,
        amounts: Vec<u64>,
        gas_payment: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
    ) -> anyhow::Result<Self> {
        coins.insert(0, gas_payment);
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            builder.pay_sui(recipients, amounts)?;
            builder.finish()
        };
        Ok(Self::new_programmable(
            sender, coins, pt, gas_budget, gas_price,
        ))
    }

    pub fn new_pay_all_sui(
        sender: SuiAddress,
        mut coins: Vec<ObjectRef>,
        recipient: SuiAddress,
        gas_payment: ObjectRef,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        coins.insert(0, gas_payment);
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            builder.pay_all_sui(recipient);
            builder.finish()
        };
        Self::new_programmable(sender, coins, pt, gas_budget, gas_price)
    }

    pub fn new_module(
        sender: SuiAddress,
        gas_payment: ObjectRef,
        modules: Vec<Vec<u8>>,
        dep_ids: Vec<ObjectID>,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            let upgrade_cap = builder.publish_upgradeable(modules, dep_ids);
            builder.transfer_arg(sender, upgrade_cap);
            builder.finish()
        };
        Self::new_programmable(sender, vec![gas_payment], pt, gas_budget, gas_price)
    }

    pub fn new_programmable(
        sender: SuiAddress,
        gas_payment: Vec<ObjectRef>,
        pt: ProgrammableTransaction,
        gas_budget: u64,
        gas_price: u64,
    ) -> Self {
        Self::new_programmable_allow_sponsor(sender, gas_payment, pt, gas_budget, gas_price, sender)
    }

    pub fn new_programmable_allow_sponsor(
        sender: SuiAddress,
        gas_payment: Vec<ObjectRef>,
        pt: ProgrammableTransaction,
        gas_budget: u64,
        gas_price: u64,
        sponsor: SuiAddress,
    ) -> Self {
        let kind = TransactionKind::ProgrammableTransaction(pt);
        Self::new_with_gas_coins_allow_sponsor(
            kind,
            sender,
            gas_payment,
            gas_budget,
            gas_price,
            sponsor,
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, IntoStaticStr)]
pub enum TransactionKind {
    /// A transaction that allows the interleaving of native commands and Move calls
    ProgrammableTransaction(ProgrammableTransaction),
}

/// A series of commands where the results of one command can be used in future
/// commands
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ProgrammableTransaction {
    /// Input objects or primitive values
    pub inputs: Vec<CallArg>,
    /// The commands to be executed sequentially. A failure in any command will
    /// result in the failure of the entire transaction.
    pub commands: Vec<Command>,
}

/// A single command in a programmable transaction.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum Command {
    /// A call to either an entry or a public Move function
    MoveCall(Box<ProgrammableMoveCall>),
    /// `(Vec<forall T:key+store. T>, address)`
    /// It sends n-objects to the specified address. These objects must have store
    /// (public transfer) and either the previous owner must be an address or the object must
    /// be newly created.
    TransferObjects(Vec<Argument>, Argument),
    /// `(&mut Coin<T>, Vec<u64>)` -> `Vec<Coin<T>>`
    /// It splits off some amounts into a new coins with those amounts
    SplitCoins(Argument, Vec<Argument>),
    /// `(&mut Coin<T>, Vec<Coin<T>>)`
    /// It merges n-coins into the first coin
    MergeCoins(Argument, Vec<Argument>),
    /// Publishes a Move package. It takes the package bytes and a list of the package's transitive
    /// dependencies to link against on-chain.
    Publish(Vec<Vec<u8>>, Vec<ObjectID>),
    /// `forall T: Vec<T> -> vector<T>`
    /// Given n-values of the same type, it constructs a vector. For non objects or an empty vector,
    /// the type tag must be specified.
    MakeMoveVec(Option<TypeTag>, Vec<Argument>),
    /// Upgrades a Move package
    /// Takes (in order):
    /// 1. A vector of serialized modules for the package.
    /// 2. A vector of object ids for the transitive dependencies of the new package.
    /// 3. The object ID of the package being upgraded.
    /// 4. An argument holding the `UpgradeTicket` that must have been produced from an earlier command in the same
    ///    programmable transaction.
    Upgrade(Vec<Vec<u8>>, Vec<ObjectID>, ObjectID, Argument),
}

/// An argument to a programmable transaction command
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Argument {
    /// The gas coin. The gas coin can only be used by-ref, except for with
    /// `TransferObjects`, which can use it by-value.
    GasCoin,
    /// One of the input objects or primitive values (from
    /// `ProgrammableTransaction` inputs)
    Input(u16),
    /// The result of another command (from `ProgrammableTransaction` commands)
    Result(u16),
    /// Like a `Result` but it accesses a nested result. Currently, the only usage
    /// of this is to access a value from a Move call with multiple return values.
    NestedResult(u16, u16),
}

/// The command for calling a Move function, either an entry function or a public
/// function (which cannot return references).
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ProgrammableMoveCall {
    /// The package containing the module and function.
    pub package: ObjectID,
    /// The specific module in the package containing the function.
    pub module: Identifier,
    /// The function to be called.
    pub function: Identifier,
    /// The type arguments to the function.
    pub type_arguments: Vec<TypeTag>,
    /// The arguments to the function.
    pub arguments: Vec<Argument>,
}

impl ProgrammableMoveCall {
    fn input_objects(&self) -> Vec<InputObjectKind> {
        let ProgrammableMoveCall {
            package,
            type_arguments,
            ..
        } = self;
        let mut packages = BTreeSet::from([*package]);
        for type_argument in type_arguments {
            add_type_tag_packages(&mut packages, type_argument)
        }
        packages
            .into_iter()
            .map(InputObjectKind::MovePackage)
            .collect()
    }

    pub fn validity_check(&self, config: &ProtocolConfig) -> UserInputResult {
        let is_blocked = BLOCKED_MOVE_FUNCTIONS.contains(&(
            self.package,
            self.module.as_str(),
            self.function.as_str(),
        ));
        fp_ensure!(!is_blocked, UserInputError::BlockedMoveFunction);
        let mut type_arguments_count = 0;
        for tag in &self.type_arguments {
            type_tag_validity_check(tag, config, &mut type_arguments_count)?;
        }
        fp_ensure!(
            self.arguments.len() < config.max_arguments() as usize,
            UserInputError::SizeLimitExceeded {
                limit: "maximum arguments in a move call".to_string(),
                value: config.max_arguments().to_string()
            }
        );
        Ok(())
    }

    fn is_input_arg_used(&self, arg: u16) -> bool {
        self.arguments
            .iter()
            .any(|a| matches!(a, Argument::Input(inp) if *inp == arg))
    }
}

impl Command {
    pub fn move_call(
        package: ObjectID,
        module: Identifier,
        function: Identifier,
        type_arguments: Vec<TypeTag>,
        arguments: Vec<Argument>,
    ) -> Self {
        Command::MoveCall(Box::new(ProgrammableMoveCall {
            package,
            module,
            function,
            type_arguments,
            arguments,
        }))
    }

    fn input_objects(&self) -> Vec<InputObjectKind> {
        match self {
            Command::Upgrade(_, deps, package_id, _) => deps
                .iter()
                .map(|id| InputObjectKind::MovePackage(*id))
                .chain(Some(InputObjectKind::MovePackage(*package_id)))
                .collect(),
            Command::Publish(_, deps) => deps
                .iter()
                .map(|id| InputObjectKind::MovePackage(*id))
                .collect(),
            Command::MoveCall(c) => c.input_objects(),
            Command::MakeMoveVec(Some(t), _) => {
                let mut packages = BTreeSet::new();
                add_type_tag_packages(&mut packages, t);
                packages
                    .into_iter()
                    .map(InputObjectKind::MovePackage)
                    .collect()
            }
            Command::MakeMoveVec(None, _)
            | Command::TransferObjects(_, _)
            | Command::SplitCoins(_, _)
            | Command::MergeCoins(_, _) => vec![],
        }
    }

    fn non_system_packages_to_be_published(&self) -> Option<&Vec<Vec<u8>>> {
        match self {
            Command::Upgrade(v, _, _, _) => Some(v),
            Command::Publish(v, _) => Some(v),
            Command::MoveCall(_)
            | Command::TransferObjects(_, _)
            | Command::SplitCoins(_, _)
            | Command::MergeCoins(_, _)
            | Command::MakeMoveVec(_, _) => None,
        }
    }

    fn validity_check(&self, config: &ProtocolConfig) -> UserInputResult {
        match self {
            Command::MoveCall(call) => call.validity_check(config)?,
            Command::TransferObjects(args, _)
            | Command::MergeCoins(_, args)
            | Command::SplitCoins(_, args) => {
                fp_ensure!(!args.is_empty(), UserInputError::EmptyCommandInput);
                fp_ensure!(
                    args.len() < config.max_arguments() as usize,
                    UserInputError::SizeLimitExceeded {
                        limit: "maximum arguments in a programmable transaction command"
                            .to_string(),
                        value: config.max_arguments().to_string()
                    }
                );
            }
            Command::MakeMoveVec(ty_opt, args) => {
                // ty_opt.is_none() ==> !args.is_empty()
                fp_ensure!(
                    ty_opt.is_some() || !args.is_empty(),
                    UserInputError::EmptyCommandInput
                );
                if let Some(ty) = ty_opt {
                    let mut type_arguments_count = 0;
                    type_tag_validity_check(ty, config, &mut type_arguments_count)?;
                }
                fp_ensure!(
                    args.len() < config.max_arguments() as usize,
                    UserInputError::SizeLimitExceeded {
                        limit: "maximum arguments in a programmable transaction command"
                            .to_string(),
                        value: config.max_arguments().to_string()
                    }
                );
            }
            Command::Publish(modules, deps) | Command::Upgrade(modules, deps, _, _) => {
                fp_ensure!(!modules.is_empty(), UserInputError::EmptyCommandInput);
                fp_ensure!(
                    modules.len() < config.max_modules_in_publish() as usize,
                    UserInputError::SizeLimitExceeded {
                        limit: "maximum modules in a programmable transaction upgrade command"
                            .to_string(),
                        value: config.max_modules_in_publish().to_string()
                    }
                );
                if let Some(max_package_dependencies) = config.max_package_dependencies_as_option()
                {
                    fp_ensure!(
                        deps.len() < max_package_dependencies as usize,
                        UserInputError::SizeLimitExceeded {
                            limit: "maximum package dependencies".to_string(),
                            value: max_package_dependencies.to_string()
                        }
                    );
                };
            }
        };
        Ok(())
    }

    fn is_input_arg_used(&self, input_arg: u16) -> bool {
        match self {
            Command::MoveCall(c) => c.is_input_arg_used(input_arg),
            Command::TransferObjects(args, arg)
            | Command::MergeCoins(arg, args)
            | Command::SplitCoins(arg, args) => args
                .iter()
                .chain(once(arg))
                .any(|a| matches!(a, Argument::Input(inp) if *inp == input_arg)),
            Command::MakeMoveVec(_, args) => args
                .iter()
                .any(|a| matches!(a, Argument::Input(inp) if *inp == input_arg)),
            Command::Upgrade(_, _, _, arg) => {
                matches!(arg, Argument::Input(inp) if *inp == input_arg)
            }
            Command::Publish(_, _) => false,
        }
    }
}

pub fn write_sep<T: Display>(
    f: &mut Formatter<'_>,
    items: impl IntoIterator<Item = T>,
    sep: &str,
) -> std::fmt::Result {
    let mut xs = items.into_iter();
    let Some(x) = xs.next() else {
        return Ok(());
    };
    write!(f, "{x}")?;
    for x in xs {
        write!(f, "{sep}{x}")?;
    }
    Ok(())
}

#[allow(dead_code)]
impl ProgrammableTransaction {
    pub fn input_objects(&self) -> UserInputResult<Vec<InputObjectKind>> {
        let ProgrammableTransaction { inputs, commands } = self;
        let input_arg_objects = inputs
            .iter()
            .flat_map(|arg| arg.input_objects())
            .collect::<Vec<_>>();
        // all objects, not just mutable, must be unique
        let mut used = HashSet::new();
        if !input_arg_objects.iter().all(|o| used.insert(o.object_id())) {
            return Err(UserInputError::DuplicateObjectRefInput);
        }
        // do not duplicate packages referred to in commands
        let command_input_objects: BTreeSet<InputObjectKind> = commands
            .iter()
            .flat_map(|command| command.input_objects())
            .collect();
        Ok(input_arg_objects
            .into_iter()
            .chain(command_input_objects)
            .collect())
    }

    fn receiving_objects(&self) -> Vec<ObjectRef> {
        let ProgrammableTransaction { inputs, .. } = self;
        inputs
            .iter()
            .flat_map(|arg| arg.receiving_objects())
            .collect()
    }

    fn validity_check(&self, config: &ProtocolConfig) -> UserInputResult {
        let ProgrammableTransaction { inputs, commands } = self;
        fp_ensure!(
            commands.len() < config.max_programmable_tx_commands() as usize,
            UserInputError::SizeLimitExceeded {
                limit: "maximum commands in a programmable transaction".to_string(),
                value: config.max_programmable_tx_commands().to_string()
            }
        );
        let total_inputs = self.input_objects()?.len() + self.receiving_objects().len();
        fp_ensure!(
            total_inputs <= config.max_input_objects() as usize,
            UserInputError::SizeLimitExceeded {
                limit: "maximum input + receiving objects in a transaction".to_string(),
                value: config.max_input_objects().to_string()
            }
        );
        for input in inputs {
            input.validity_check(config)?
        }
        if let Some(max_publish_commands) = config.max_publish_or_upgrade_per_ptb_as_option() {
            let publish_count = commands
                .iter()
                .filter(|c| matches!(c, Command::Publish(_, _) | Command::Upgrade(_, _, _, _)))
                .count() as u64;
            fp_ensure!(
                publish_count <= max_publish_commands,
                UserInputError::MaxPublishCountExceeded {
                    max_publish_commands,
                    publish_count,
                }
            );
        }
        for command in commands {
            command.validity_check(config)?;
        }

        // A command that uses Random can only be followed by TransferObjects or MergeCoins.
        if let Some(random_index) = inputs.iter().position(|obj| {
            matches!(obj, CallArg::Object(ObjectArg::SharedObject { id, .. }) if *id == SUI_RANDOMNESS_STATE_OBJECT_ID)
        }) {
            let mut used_random_object = false;
            let random_index = random_index.try_into().unwrap();
            for command in commands {
                if !used_random_object {
                    used_random_object = command.is_input_arg_used(random_index);
                } else {
                    fp_ensure!(
                        matches!(
                            command,
                            Command::TransferObjects(_, _) | Command::MergeCoins(_, _)
                        ),
                        UserInputError::PostRandomCommandRestrictions
                    );
                }
            }
        }

        Ok(())
    }

    fn shared_input_objects(&self) -> impl Iterator<Item = SharedInputObject> + '_ {
        self.inputs
            .iter()
            .filter_map(|arg| match arg {
                CallArg::Pure(_)
                | CallArg::Object(ObjectArg::Receiving(_))
                | CallArg::Object(ObjectArg::ImmOrOwnedObject(_)) => None,
                CallArg::Object(ObjectArg::SharedObject {
                    id,
                    initial_shared_version,
                    mutable,
                }) => Some(vec![SharedInputObject {
                    id: *id,
                    initial_shared_version: *initial_shared_version,
                    mutable: *mutable,
                }]),
            })
            .flatten()
    }

    fn move_calls(&self) -> Vec<(&ObjectID, &IdentStr, &IdentStr)> {
        self.commands
            .iter()
            .filter_map(|command| match command {
                Command::MoveCall(m) => Some((
                    &m.package,
                    m.module.as_ident_str(),
                    m.function.as_ident_str(),
                )),
                _ => None,
            })
            .collect()
    }

    pub fn non_system_packages_to_be_published(&self) -> impl Iterator<Item = &Vec<Vec<u8>>> + '_ {
        self.commands
            .iter()
            .filter_map(|q| q.non_system_packages_to_be_published())
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Argument::GasCoin => write!(f, "GasCoin"),
            Argument::Input(i) => write!(f, "Input({i})"),
            Argument::Result(i) => write!(f, "Result({i})"),
            Argument::NestedResult(i, j) => write!(f, "NestedResult({i},{j})"),
        }
    }
}

impl Display for ProgrammableMoveCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ProgrammableMoveCall {
            package,
            module,
            function,
            type_arguments,
            arguments,
        } = self;
        write!(f, "{package}::{module}::{function}")?;
        if !type_arguments.is_empty() {
            write!(f, "<")?;
            write_sep(f, type_arguments, ",")?;
            write!(f, ">")?;
        }
        write!(f, "(")?;
        write_sep(f, arguments, ",")?;
        write!(f, ")")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord, Hash)]
pub enum InputObjectKind {
    // A Move package, must be immutable.
    MovePackage(ObjectID),
    // A Move object, either immutable, or owned mutable.
    ImmOrOwnedMoveObject(ObjectRef),
    // A Move object that's shared and mutable.
    SharedMoveObject {
        id: ObjectID,
        initial_shared_version: SequenceNumber,
        mutable: bool,
    },
}

impl InputObjectKind {
    pub fn object_id(&self) -> ObjectID {
        match self {
            Self::MovePackage(id) => *id,
            Self::ImmOrOwnedMoveObject((id, _, _)) => *id,
            Self::SharedMoveObject { id, .. } => *id,
        }
    }

    pub fn version(&self) -> Option<SequenceNumber> {
        match self {
            Self::MovePackage(..) => None,
            Self::ImmOrOwnedMoveObject((_, version, _)) => Some(*version),
            Self::SharedMoveObject { .. } => None,
        }
    }

    pub fn object_not_found_error(&self) -> UserInputError {
        match *self {
            Self::MovePackage(package_id) => {
                UserInputError::DependentPackageNotFound { package_id }
            }
            Self::ImmOrOwnedMoveObject((object_id, version, _)) => UserInputError::ObjectNotFound {
                object_id,
                version: Some(version),
            },
            Self::SharedMoveObject { id, .. } => UserInputError::ObjectNotFound {
                object_id: id,
                version: None,
            },
        }
    }

    pub fn is_shared_object(&self) -> bool {
        matches!(self, Self::SharedMoveObject { .. })
    }

    pub fn is_mutable(&self) -> bool {
        match self {
            Self::MovePackage(..) => false,
            Self::ImmOrOwnedMoveObject((_, _, _)) => true,
            Self::SharedMoveObject { mutable, .. } => *mutable,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SharedInputObject {
    pub id: ObjectID,
    pub initial_shared_version: SequenceNumber,
    pub mutable: bool,
}

impl SharedInputObject {
    pub const SUI_SYSTEM_OBJ: Self = Self {
        id: SUI_SYSTEM_STATE_OBJECT_ID,
        initial_shared_version: SUI_SYSTEM_STATE_OBJECT_SHARED_VERSION,
        mutable: true,
    };

    pub fn id(&self) -> ObjectID {
        self.id
    }

    pub fn into_id_and_version(self) -> (ObjectID, SequenceNumber) {
        (self.id, self.initial_shared_version)
    }
}

// Add package IDs, `ObjectID`, for types defined in modules.
fn add_type_tag_packages(packages: &mut BTreeSet<ObjectID>, type_argument: &TypeTag) {
    let mut stack = vec![type_argument];
    while let Some(cur) = stack.pop() {
        match cur {
            TypeTag::Bool
            | TypeTag::U8
            | TypeTag::U64
            | TypeTag::U128
            | TypeTag::Address
            | TypeTag::Signer
            | TypeTag::U16
            | TypeTag::U32
            | TypeTag::U256 => (),
            TypeTag::Vector(inner) => stack.push(inner),
            TypeTag::Struct(struct_tag) => {
                packages.insert(struct_tag.address.into());
                stack.extend(struct_tag.type_params.iter())
            }
        }
    }
}
