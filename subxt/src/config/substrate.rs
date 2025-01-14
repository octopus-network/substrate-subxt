// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! Substrate specific configuration

use super::{
    extrinsic_params::{
        BaseExtrinsicParams,
        BaseExtrinsicParamsBuilder,
    },
    Config,
    Hasher,
    Header,
};
use codec::{
    Decode,
    Encode,
};
use serde::{
    Deserialize,
    Serialize,
};

pub use crate::utils::{
    account_id::AccountId32,
    multi_address::MultiAddress,
    multi_signature::MultiSignature,
};
pub use primitive_types::{
    H256,
    U256,
};

/// Default set of commonly used types by Substrate runtimes.
// Note: We only use this at the type level, so it should be impossible to
// create an instance of it.
pub enum SubstrateConfig {}

impl Config for SubstrateConfig {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = H256;
    type AccountId = AccountId32;
    type Address = MultiAddress<Self::AccountId, u32>;
    type Signature = MultiSignature;
    type Hasher = BlakeTwo256;
    type Header = SubstrateHeader<Self::BlockNumber, BlakeTwo256>;
    type ExtrinsicParams = SubstrateExtrinsicParams<Self>;
}

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction for the default substrate node.
pub type SubstrateExtrinsicParams<T> = BaseExtrinsicParams<T, AssetTip>;

/// A builder which leads to [`SubstrateExtrinsicParams`] being constructed.
/// This is what you provide to methods like `sign_and_submit()`.
pub type SubstrateExtrinsicParamsBuilder<T> = BaseExtrinsicParamsBuilder<T, AssetTip>;

// Because Era is one of the args to our extrinsic params.
pub use super::extrinsic_params::Era;

/// A tip payment made in the form of a specific asset.
#[derive(Copy, Clone, Debug, Default, Encode)]
pub struct AssetTip {
    #[codec(compact)]
    tip: u128,
    asset: Option<u32>,
}

impl AssetTip {
    /// Create a new tip of the amount provided.
    pub fn new(amount: u128) -> Self {
        AssetTip {
            tip: amount,
            asset: None,
        }
    }

    /// Designate the tip as being of a particular asset class.
    /// If this is not set, then the native currency is used.
    pub fn of_asset(mut self, asset: u32) -> Self {
        self.asset = Some(asset);
        self
    }
}

impl From<u128> for AssetTip {
    fn from(n: u128) -> Self {
        AssetTip::new(n)
    }
}

/// A type that can hash values using the blaks2_256 algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode)]
pub struct BlakeTwo256;

impl Hasher for BlakeTwo256 {
    type Output = H256;
    fn hash(s: &[u8]) -> Self::Output {
        sp_core_hashing::blake2_256(s).into()
    }
}

/// A generic Substrate header type, adapted from `sp_runtime::generic::Header`.
/// The block number and hasher can be configured to adapt this for other nodes.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubstrateHeader<N: Copy + Into<U256> + TryFrom<U256>, H: Hasher> {
    /// The parent hash.
    pub parent_hash: H::Output,
    /// The block number.
    #[serde(
        serialize_with = "serialize_number",
        deserialize_with = "deserialize_number"
    )]
    #[codec(compact)]
    pub number: N,
    /// The state trie merkle root
    pub state_root: H::Output,
    /// The merkle root of the extrinsics.
    pub extrinsics_root: H::Output,
    /// A chain-specific digest of data useful for light clients or referencing auxiliary data.
    pub digest: Digest,
}

impl<N: Copy + Into<U256> + TryFrom<U256> + Encode, H: Hasher + Encode> Header
    for SubstrateHeader<N, H>
where
    N: Copy + Into<U256> + TryFrom<U256> + Encode,
    H: Hasher + Encode,
    SubstrateHeader<N, H>: Encode,
{
    type Number = N;
    type Hasher = H;
    fn number(&self) -> Self::Number {
        self.number
    }
}

/// Generic header digest. From `sp_runtime::generic::digest`.
#[derive(
    Encode, Decode, Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default,
)]
pub struct Digest {
    /// A list of digest items.
    pub logs: Vec<DigestItem>,
}

/// Digest item that is able to encode/decode 'system' digest items and
/// provide opaque access to other items. From `sp_runtime::generic::digest`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DigestItem {
    /// A pre-runtime digest.
    ///
    /// These are messages from the consensus engine to the runtime, although
    /// the consensus engine can (and should) read them itself to avoid
    /// code and state duplication. It is erroneous for a runtime to produce
    /// these, but this is not (yet) checked.
    ///
    /// NOTE: the runtime is not allowed to panic or fail in an `on_initialize`
    /// call if an expected `PreRuntime` digest is not present. It is the
    /// responsibility of a external block verifier to check this. Runtime API calls
    /// will initialize the block without pre-runtime digests, so initialization
    /// cannot fail when they are missing.
    PreRuntime(ConsensusEngineId, Vec<u8>),

    /// A message from the runtime to the consensus engine. This should *never*
    /// be generated by the native code of any consensus engine, but this is not
    /// checked (yet).
    Consensus(ConsensusEngineId, Vec<u8>),

    /// Put a Seal on it. This is only used by native code, and is never seen
    /// by runtimes.
    Seal(ConsensusEngineId, Vec<u8>),

    /// Some other thing. Unsupported and experimental.
    Other(Vec<u8>),

    /// An indication for the light clients that the runtime execution
    /// environment is updated.
    ///
    /// Currently this is triggered when:
    /// 1. Runtime code blob is changed or
    /// 2. `heap_pages` value is changed.
    RuntimeEnvironmentUpdated,
}

// From sp_runtime::generic, DigestItem enum indexes are encoded using this:
#[repr(u32)]
#[derive(Encode, Decode)]
enum DigestItemType {
    Other = 0u32,
    Consensus = 4u32,
    Seal = 5u32,
    PreRuntime = 6u32,
    RuntimeEnvironmentUpdated = 8u32,
}
impl Encode for DigestItem {
    fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();

        match self {
            Self::Consensus(val, data) => {
                DigestItemType::Consensus.encode_to(&mut v);
                (val, data).encode_to(&mut v);
            }
            Self::Seal(val, sig) => {
                DigestItemType::Seal.encode_to(&mut v);
                (val, sig).encode_to(&mut v);
            }
            Self::PreRuntime(val, data) => {
                DigestItemType::PreRuntime.encode_to(&mut v);
                (val, data).encode_to(&mut v);
            }
            Self::Other(val) => {
                DigestItemType::Other.encode_to(&mut v);
                val.encode_to(&mut v);
            }
            Self::RuntimeEnvironmentUpdated => {
                DigestItemType::RuntimeEnvironmentUpdated.encode_to(&mut v);
            }
        }

        v
    }
}
impl Decode for DigestItem {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let item_type: DigestItemType = Decode::decode(input)?;
        match item_type {
            DigestItemType::PreRuntime => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::PreRuntime(vals.0, vals.1))
            }
            DigestItemType::Consensus => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::Consensus(vals.0, vals.1))
            }
            DigestItemType::Seal => {
                let vals: (ConsensusEngineId, Vec<u8>) = Decode::decode(input)?;
                Ok(Self::Seal(vals.0, vals.1))
            }
            DigestItemType::Other => Ok(Self::Other(Decode::decode(input)?)),
            DigestItemType::RuntimeEnvironmentUpdated => {
                Ok(Self::RuntimeEnvironmentUpdated)
            }
        }
    }
}

/// Consensus engine unique ID. From `sp_runtime::ConsensusEngineId`.
pub type ConsensusEngineId = [u8; 4];

impl serde::Serialize for DigestItem {
    fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.using_encoded(|bytes| impl_serde::serialize::serialize(bytes, seq))
    }
}

impl<'a> serde::Deserialize<'a> for DigestItem {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let r = impl_serde::serialize::deserialize(de)?;
        Decode::decode(&mut &r[..])
            .map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
    }
}

fn serialize_number<S, T: Copy + Into<U256> + TryFrom<U256>>(
    val: &T,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let u256: U256 = (*val).into();
    serde::Serialize::serialize(&u256, s)
}

fn deserialize_number<'a, D, T: Copy + Into<U256> + TryFrom<U256>>(
    d: D,
) -> Result<T, D::Error>
where
    D: serde::Deserializer<'a>,
{
    let u256: U256 = serde::Deserialize::deserialize(d)?;
    TryFrom::try_from(u256).map_err(|_| serde::de::Error::custom("Try from failed"))
}
