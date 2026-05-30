use bech32::{primitives::gf32::Fe32, segwit::VERSION_0, segwit::VERSION_1};
use bitcoin::{
    ScriptBuf,
    blockdata::{opcodes::all::*, script::Builder},
};
use primitives::SignerError;

use crate::hash::{HASH160_LEN, hash20};

const WITNESS_PROGRAM_LEN: usize = 32;
const P2PKH_HASH_OFFSET: usize = 3;
const P2WPKH_HASH_OFFSET: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LockingScript {
    P2pkh,
    P2sh,
    P2wpkh,
    P2wsh,
    P2tr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnlockingScript {
    P2pkh,
    P2wpkh,
}

#[derive(Debug, Clone)]
pub(crate) struct AddressScript {
    pub(crate) script_pubkey: ScriptBuf,
    pub(crate) locking_script: LockingScript,
}

impl AddressScript {
    pub(crate) fn new(script_pubkey: ScriptBuf, locking_script: LockingScript) -> Self {
        Self { script_pubkey, locking_script }
    }

    pub(crate) fn unlocking_script(&self) -> Option<UnlockingScript> {
        match (self.locking_script, self.public_key_hash()) {
            (LockingScript::P2pkh, Some(_)) => Some(UnlockingScript::P2pkh),
            (LockingScript::P2wpkh, Some(_)) => Some(UnlockingScript::P2wpkh),
            _ => None,
        }
    }

    pub(crate) fn public_key_hash(&self) -> Option<[u8; HASH160_LEN]> {
        let offset = match self.locking_script {
            LockingScript::P2pkh if self.script_pubkey.is_p2pkh() => P2PKH_HASH_OFFSET,
            LockingScript::P2wpkh if self.script_pubkey.is_p2wpkh() => P2WPKH_HASH_OFFSET,
            LockingScript::P2sh | LockingScript::P2wsh | LockingScript::P2tr | LockingScript::P2pkh | LockingScript::P2wpkh => return None,
        };
        self.script_pubkey.as_bytes().get(offset..offset + HASH160_LEN)?.try_into().ok()
    }

    pub(super) fn from_prefixed_address(address: &str, p2pkh_versions: &[u8], p2sh_versions: &[u8], hrp: Option<&str>) -> Result<Self, SignerError> {
        if let Some(expected_hrp) = hrp
            && let Ok((address_hrp, version, program)) = bech32::segwit::decode(address)
            && address_hrp.as_str() == expected_hrp
        {
            return Self::from_segwit(version, &program);
        }

        let payload = bs58::decode(address).with_check(None).into_vec().map_err(SignerError::from_display)?;
        let Some((&version, hash)) = payload.split_first() else {
            return Err(SignerError::from_display("invalid base58 address"));
        };
        let hash = hash20(hash)?;

        if p2pkh_versions.contains(&version) {
            Ok(Self::new(p2pkh_script(hash), LockingScript::P2pkh))
        } else if p2sh_versions.contains(&version) {
            Ok(Self::new(p2sh_script(hash), LockingScript::P2sh))
        } else {
            Err(SignerError::from_display("unsupported address version"))
        }
    }

    fn from_segwit(version: Fe32, program: &[u8]) -> Result<Self, SignerError> {
        let (script_pubkey, locking_script) = match (version, program.len()) {
            (version, HASH160_LEN) if version == VERSION_0 => (p2wpkh_script(hash20(program)?), LockingScript::P2wpkh),
            (version, WITNESS_PROGRAM_LEN) if version == VERSION_0 => (p2wsh_script(hash32(program)?), LockingScript::P2wsh),
            (version, WITNESS_PROGRAM_LEN) if version == VERSION_1 => (p2tr_script(hash32(program)?), LockingScript::P2tr),
            _ => return Err(SignerError::from_display("unsupported segwit address type")),
        };

        Ok(Self::new(script_pubkey, locking_script))
    }
}

pub(crate) fn script_for_public_key_hash(unlocking_script: UnlockingScript, hash: [u8; HASH160_LEN]) -> ScriptBuf {
    match unlocking_script {
        UnlockingScript::P2pkh => p2pkh_script(hash),
        UnlockingScript::P2wpkh => p2wpkh_script(hash),
    }
}

pub(super) fn p2pkh_script(hash: [u8; HASH160_LEN]) -> ScriptBuf {
    Builder::new()
        .push_opcode(OP_DUP)
        .push_opcode(OP_HASH160)
        .push_slice(hash)
        .push_opcode(OP_EQUALVERIFY)
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

pub(super) fn p2sh_script(hash: [u8; HASH160_LEN]) -> ScriptBuf {
    Builder::new().push_opcode(OP_HASH160).push_slice(hash).push_opcode(OP_EQUAL).into_script()
}

fn p2wpkh_script(hash: [u8; HASH160_LEN]) -> ScriptBuf {
    Builder::new().push_opcode(OP_PUSHBYTES_0).push_slice(hash).into_script()
}

fn p2wsh_script(hash: [u8; WITNESS_PROGRAM_LEN]) -> ScriptBuf {
    Builder::new().push_opcode(OP_PUSHBYTES_0).push_slice(hash).into_script()
}

fn p2tr_script(output_key: [u8; WITNESS_PROGRAM_LEN]) -> ScriptBuf {
    Builder::new().push_opcode(OP_PUSHNUM_1).push_slice(output_key).into_script()
}

fn hash32(bytes: &[u8]) -> Result<[u8; WITNESS_PROGRAM_LEN], SignerError> {
    bytes.try_into().map_err(SignerError::from_display)
}
